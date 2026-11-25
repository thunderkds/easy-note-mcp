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

/// HUN-277 AC 9: Obsidian's vault list (`$HOME/.config/obsidian/obsidian.json` here, as
/// XDG_CONFIG_HOME is cleared) does not cover the notes dir: one hint line, and the URI still opens.
#[test]
fn hun277_ac9_dashboard_hints_when_the_notes_dir_is_not_a_vault_yet() {
    let hint = "the notes dir is not an Obsidian vault yet: run easy-note setup";
    let list = |sb: &Sb, vault: &str| {
        let path = sb.root().join("home/.config/obsidian/obsidian.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            path,
            format!(r#"{{"vaults":{{"aaaa":{{"path":"{vault}","ts":1,"open":true}}}}}}"#),
        )
        .unwrap();
    };

    let sb = with_stubs("hint");
    assert_eq!(sb.run(&["install", "obsidian"], "").status.code(), Some(0));
    list(&sb, &format!("{}/notes", sb.notes().display()));
    let o = sb.run(&["dashboard"], "obsidian.desktop");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert_eq!(
        text(&o.stderr).lines().filter(|l| l.contains(hint)).count(),
        1,
        "{}",
        text(&o.stderr)
    );
    assert!(sb.log("open.args").starts_with("obsidian://open?path="));

    // Covered (the notes dir itself): no hint.
    list(&sb, sb.notes().to_str().unwrap());
    let o = sb.run(&["dashboard"], "obsidian.desktop");
    assert_eq!(o.status.code(), Some(0));
    assert!(!text(&o.stderr).contains(hint), "{}", text(&o.stderr));

    // No vault list at all: no hint (AC 3 above keeps its single open call).
    let sb = with_stubs("hint-nolist");
    sb.run(&["install", "obsidian"], "");
    let o = sb.run(&["dashboard"], "obsidian.desktop");
    assert!(!text(&o.stderr).contains(hint), "{}", text(&o.stderr));
}

/// HUN-281: a `window` build with `DISPLAY` set to a display that does not answer must not
/// panic in GTK start-up; it falls back to the FR-034 Obsidian path (here: no file yet, so the
/// install hint and exit 1).
#[cfg(feature = "window")]
#[test]
fn hun281_unusable_display_falls_back_to_obsidian_without_a_panic() {
    let sb = with_stubs("baddisplay");
    let o = Command::new(env!("CARGO_BIN_EXE_easy-note"))
        .arg("dashboard")
        .env_clear()
        .env("PATH", sb.root().join("bin"))
        .env("HOME", sb.root().join("home"))
        .env("EASY_NOTE_DIR", sb.notes())
        .env("STUB_LOG", sb.root().join("log"))
        .env("STUB_HANDLER", "obsidian.desktop")
        .env("DISPLAY", ":99317")
        .current_dir(sb.root().join("home"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let err = text(&o.stderr);
    assert!(!err.contains("panicked"), "{err}");
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(
        err.contains("no dashboard yet: run easy-note install obsidian"),
        "{err}"
    );
}

/// HUN-286 AC 7, Success Criterion 5: an id that fails the store's id check is refused before
/// anything opens: exit 2, the usage message, no `xdg-open`, nothing written.
#[test]
fn dashboard_note_refuses_a_bad_id_before_anything_opens() {
    for bad in [
        "../x",
        "2026/11/20261119-090000-aaaa",
        "inbox/../x",
        "inbox/.hidden",
        "",
    ] {
        let sb = with_stubs("bad-note");
        assert_eq!(sb.run(&["install", "obsidian"], "").status.code(), Some(0));
        let o = sb.run(&["dashboard", "--note", bad], "obsidian.desktop");
        assert_eq!(o.status.code(), Some(2), "{bad:?}: {}", text(&o.stderr));
        assert!(
            text(&o.stderr).contains("not a note id"),
            "{bad:?}: {}",
            text(&o.stderr)
        );
        assert_eq!(sb.log("open.calls"), "", "{bad:?}: nothing opened");
    }
}

/// A valid id that names no note is no error: without the window (default build) the Obsidian
/// path runs as for a bare `dashboard`.
#[test]
fn dashboard_note_with_a_valid_unknown_id_opens_the_dashboard() {
    let sb = with_stubs("unknown-note");
    assert_eq!(sb.run(&["install", "obsidian"], "").status.code(), Some(0));
    for id in ["20261119-090000-ffff", "inbox/phone"] {
        let o = sb.run(&["dashboard", "--note", id], "obsidian.desktop");
        assert_eq!(o.status.code(), Some(0), "{id}: {}", text(&o.stderr));
    }
    assert_eq!(sb.log("open.calls").lines().count(), 2);
}

#[test]
fn dashboard_help_names_the_note_option() {
    let sb = with_stubs("note-help");
    let o = sb.run(&["dashboard", "--help"], "");
    assert_eq!(o.status.code(), Some(0));
    assert!(
        text(&o.stdout).contains("--note <ID>"),
        "{}",
        text(&o.stdout)
    );
}

/// HUN-290 AC 1, 3, 5: with the window lock held, `dashboard` (and `dashboard --note`, the tray's
/// recent-note click) starts no window and no Obsidian, sends a focus request and exits 0.
#[cfg(feature = "window")]
#[test]
fn hun290_a_second_dashboard_call_focuses_the_open_window() {
    use easy_note_mcp::surfaces::window::single;
    let sb = with_stubs("second");
    let owner = single::acquire(&sb.notes()).unwrap().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    owner.serve(move |n| {
        let _ = tx.send(n);
    });
    for (args, want) in [
        (vec!["dashboard"], None),
        (
            vec!["dashboard", "--note", "20261119-090000-abcd"],
            Some("20261119-090000-abcd".to_string()),
        ),
    ] {
        let began = std::time::Instant::now();
        let o = Command::new(env!("CARGO_BIN_EXE_easy-note"))
            .args(&args)
            .env_clear()
            .env("PATH", sb.root().join("bin"))
            .env("HOME", sb.root().join("home"))
            .env("DISPLAY", ":99")
            .env("EASY_NOTE_DIR", sb.notes())
            .env("STUB_LOG", sb.root().join("log"))
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
        assert!(began.elapsed() < std::time::Duration::from_secs(2));
        let got = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(got, want);
    }
    assert_eq!(sb.log("open.calls"), "", "no Obsidian, no second window");
}
