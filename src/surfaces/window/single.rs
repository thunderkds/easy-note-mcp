//! One dashboard window per notes dir (HUN-290, FR-045). The window holds an exclusive lock on
//! `<notes dir>/.easy-note/window.lock` and listens on a Unix socket beside it. A second
//! `dashboard` finds the lock held, sends one line (`focus` or `focus <id>`) and exits. The system
//! drops the lock when the window process ends, however it ends, so a killed window never blocks.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

/// How long a second call waits for a window that is still starting.
pub const WAIT: Duration = Duration::from_secs(2);
const STEP: Duration = Duration::from_millis(50);
/// A socket path must stay under the 108-byte `sun_path` limit; keep a margin.
const MAX_SOCKET_PATH: usize = 100;

/// The held lock and the bound socket of the one window of a notes dir.
pub struct Owner {
    _lock: File,
    listener: UnixListener,
}

/// The socket's path: beside the lock, or a short one under `$XDG_RUNTIME_DIR` (else the temp dir),
/// named by a hash of the notes dir, when the first would be too long.
pub fn socket_path(root: &Path) -> PathBuf {
    let beside = root.join(".easy-note").join("window.sock");
    if beside.as_os_str().len() < MAX_SOCKET_PATH {
        return beside;
    }
    let hash = Sha256::digest(root.as_os_str().as_encoded_bytes());
    let name: String = hash.iter().take(8).map(|b| format!("{b:02x}")).collect();
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join(format!("easy-note-{name}.sock"))
}

/// Take the window lock. `Ok(None)`: another window holds it. A lock left by a dead process is free.
pub fn acquire(root: &Path) -> Result<Option<Owner>, String> {
    let dir = root.join(".easy-note");
    let path = dir.join("window.lock");
    let lock = fs::create_dir_all(&dir)
        .and_then(|()| {
            OpenOptions::new()
                .create(true)
                .truncate(false)
                .write(true)
                .open(&path)
        })
        .map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    match lock.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => return Ok(None),
        Err(TryLockError::Error(e)) => {
            return Err(format!("cannot lock {}: {e}", path.display()));
        }
    }
    // The lock is ours, so any socket file is a dead window's.
    let sock = socket_path(root);
    let _ = fs::remove_file(&sock);
    let listener = UnixListener::bind(&sock)
        .map_err(|e| format!("cannot listen on {}: {e}", sock.display()))?;
    Ok(Some(Owner {
        _lock: lock,
        listener,
    }))
}

/// A request line: `Some(None)` for `focus`, `Some(Some(id))` for `focus <id>`, else `None`.
pub fn parse(line: &str) -> Option<Option<String>> {
    let rest = line.trim_end_matches(['\n', '\r']).strip_prefix("focus")?;
    if rest.is_empty() {
        return Some(None);
    }
    let id = rest.strip_prefix(' ')?;
    (!id.is_empty() && !id.contains(char::is_whitespace)).then(|| Some(id.to_string()))
}

impl Owner {
    /// Answer requests on a background thread for as long as the process lives: `on_request`
    /// gets the note id of each valid request. The lock stays held by the thread.
    pub fn serve(self, on_request: impl Fn(Option<String>) + Send + 'static) {
        std::thread::spawn(move || {
            for stream in self.listener.incoming().flatten() {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
                // A request line is short; a longer one is cut and then fails `parse`.
                let mut line = Vec::new();
                if BufReader::new(stream.take(4096))
                    .read_until(b'\n', &mut line)
                    .is_ok()
                {
                    if let Some(note) = parse(&String::from_utf8_lossy(&line)) {
                        on_request(note);
                    }
                }
            }
        });
    }
}

/// Ask the open window to come to the front (and show `note`); wait up to `WAIT` for a window that
/// is still starting.
pub fn request_focus(root: &Path, note: Option<&str>) -> Result<(), String> {
    request_focus_within(root, note, WAIT)
}

/// `request_focus` with its own wait.
pub fn request_focus_within(root: &Path, note: Option<&str>, wait: Duration) -> Result<(), String> {
    let sock = socket_path(root);
    let line = match note {
        Some(id) => format!("focus {id}\n"),
        None => "focus\n".to_string(),
    };
    let until = Instant::now() + wait;
    loop {
        match UnixStream::connect(&sock) {
            Ok(mut stream) => {
                return stream
                    .write_all(line.as_bytes())
                    .map_err(|e| format!("cannot send to {}: {e}", sock.display()));
            }
            Err(e) if Instant::now() >= until => {
                return Err(format!("no answer from {}: {e}", sock.display()));
            }
            Err(_) => std::thread::sleep(STEP),
        }
    }
}
