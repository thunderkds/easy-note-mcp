//! Small file helpers shared by config and store: a temp file made with `create_new`.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Create `.<unique>.tmp` in `dir` with an exclusive create (mode 0600); never reuses a name.
pub(crate) fn create_temp(dir: &Path) -> io::Result<(File, PathBuf)> {
    loop {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = dir.join(format!(".{:x}-{nanos:x}-{n:x}.tmp", std::process::id()));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
        {
            Ok(file) => return Ok((file, path)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
}

/// Replace the whole content of `file` with `data` and fsync it.
pub(crate) fn write_all_sync(file: &mut File, data: &[u8]) -> io::Result<()> {
    use std::io::Seek;
    file.set_len(0)?;
    file.rewind()?;
    file.write_all(data)?;
    file.sync_all()
}

pub(crate) fn unlink_quiet(path: &Path) {
    let _ = std::fs::remove_file(path);
}
