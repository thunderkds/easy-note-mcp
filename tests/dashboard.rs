//! HUN-273: `easy-note dashboard` opens `easy-note.base` in Obsidian (FR-034, NFR-001, DDR 0006).
//! Every test puts stub `xdg-open` and `xdg-mime` scripts on a temp `PATH` that holds nothing else
//! and uses a temp HOME and EASY_NOTE_DIR. No test starts a real program.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const OPEN_STUB: &str = "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"$STUB_LOG/open.args\"\necho called >> \"$STUB_LOG/open.calls\"\n";
const MIME_STUB: &str =
    "#!/bin/sh\nif [ -n \"$STUB_HANDLER\" ]; then printf '%s\\n' \"$STUB_HANDLER\"; fi\n";

struct Sb {
    dir: tempfile::TempDir,
}

impl Sb {
    fn new(name: &str) -> Sb {
        let sb = Sb {
            dir: tempfile::Builder::new()
                .prefix(&format!("easy-note-dashboard-{name}-"))
                .tempdir()
                .unwrap(),
        };
        for d in ["home", "bin", "log"] {
            std::fs::create_dir_all(sb.root().join(d)).unwrap();
        }
        sb
    }
    fn root(&self) -> &Path {
        self.dir.path()
    }
    fn notes(&self) -> PathBuf {
        self.root().join("notes dir é")
    }
    fn stub(&self, name: &str, body: &str) {
        let path = self.root().join("bin").join(name);
        std::fs::write(&path, body).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    fn log(&self, file: &str) -> String {
        std::fs::read_to_string(self.root().join("log").join(file)).unwrap_or_default()
    }
    fn run(&self, args: &[&str], handler: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_easy-note"))
            .args(args)
            .env_clear()
            .env("PATH", self.root().join("bin"))
            .env("HOME", self.root().join("home"))
            .env("EASY_NOTE_DIR", self.notes())
            .env("STUB_LOG", self.root().join("log"))
            .env("STUB_HANDLER", handler)
            .current_dir(self.root().join("home"))
            .stdin(Stdio::null())
            .output()
            .unwrap()
    }
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

fn with_stubs(name: &str) -> Sb {
    let sb = Sb::new(name);
    sb.stub("xdg-open", OPEN_STUB);
    sb.stub("xdg-mime", MIME_STUB);
    sb
}

/// AC 3: the file is there and Obsidian handles `obsidian://`: one argv, the encoded URI.
#[test]
fn ac3_dashboard_opens_the_base_file_through_the_obsidian_scheme() {
    let sb = with_stubs("open");
    assert_eq!(sb.run(&["install", "obsidian"], "").status.code(), Some(0));
    let o = sb.run(&["dashboard"], "obsidian.desktop");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let base = sb.notes().join("easy-note.base");
    let want = format!(
        "obsidian://open?path={}",
        sb.notes()
            .join("easy-note.base")
            .to_str()
            .unwrap()
            .replace('/', "%2F")
            .replace(' ', "%20")
            .replace('é', "%C3%A9")
    );
    assert!(base.exists());
    assert_eq!(sb.log("open.args"), format!("{want}\n"));
    assert_eq!(sb.log("open.calls").lines().count(), 1);
}

/// AC 4: no file: the hint, exit 1, nothing started.
#[test]
fn ac4_dashboard_without_the_file_says_to_install() {
    let sb = with_stubs("nofile");
    let o = sb.run(&["dashboard"], "obsidian.desktop");
    assert_eq!(o.status.code(), Some(1));
    assert!(
        text(&o.stderr).contains("no dashboard yet: run easy-note install obsidian"),
        "{}",
        text(&o.stderr)
    );
    assert_eq!(sb.log("open.calls"), "", "nothing started");
    assert!(!sb.notes().exists(), "nothing written");
}

/// AC 5: no handler: say so, open the notes folder, exit 0. A missing `xdg-mime` is the same.
#[test]
fn ac5_dashboard_without_obsidian_opens_the_folder() {
    for missing_mime in [false, true] {
        let sb = with_stubs("nohandler");
        if missing_mime {
            std::fs::remove_file(sb.root().join("bin/xdg-mime")).unwrap();
        }
        sb.run(&["install", "obsidian"], "");
        let o = sb.run(&["dashboard"], "");
        assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
        assert!(
            text(&o.stderr).contains("Obsidian is not installed"),
            "{}",
            text(&o.stderr)
        );
        assert_eq!(
            sb.log("open.args"),
            format!("{}\n", sb.notes().display()),
            "missing_mime={missing_mime}"
        );
    }
}

/// `xdg-open` missing: one stderr line, exit 1.
#[test]
fn dashboard_without_xdg_open_fails_in_one_line() {
    let sb = Sb::new("noopen");
    sb.stub("xdg-mime", MIME_STUB);
    sb.run(&["install", "obsidian"], "");
    let o = sb.run(&["dashboard"], "obsidian.desktop");
    assert_eq!(o.status.code(), Some(1));
    assert!(text(&o.stderr).contains("xdg-open"), "{}", text(&o.stderr));
}
