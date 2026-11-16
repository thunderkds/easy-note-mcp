//! HUN-260: `easy-note hotkey` prints the commands that bind a key to `easy-note quick` for the
//! desktop in `$XDG_CURRENT_DESKTOP` (FR-025). It writes nothing and runs no command. Tests use
//! a temp HOME, EASY_NOTE_DIR and a stub-only PATH.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A stub that would leave a marker file if `hotkey` ever ran it.
const TRIPWIRE: &str = "#!/bin/sh\necho ran >> \"$HOME/tripwire\"\n";

type Snapshot = Vec<(PathBuf, Vec<u8>)>;

fn snapshot(root: &Path) -> Snapshot {
    let mut all = vec![];
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                all.push((path.clone(), vec![]));
                stack.push(path);
            } else {
                all.push((path.clone(), std::fs::read(&path).unwrap()));
            }
        }
    }
    all.sort();
    all
}

fn hotkey(desktop: Option<&str>) -> (Output, Snapshot, Snapshot) {
    let dir = tempfile::tempdir().unwrap();
    let (home, bin) = (dir.path().join("home"), dir.path().join("bin"));
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&bin).unwrap();
    for name in [
        "gsettings",
        "kwriteconfig5",
        "kwriteconfig6",
        "easy-note",
        "notify-send",
    ] {
        let path = bin.join(name);
        std::fs::write(&path, TRIPWIRE).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let before = snapshot(dir.path());
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_easy-note"));
    cmd.arg("hotkey")
        .env_clear()
        .env("HOME", &home)
        .env("EASY_NOTE_DIR", dir.path().join("notes"))
        .env("PATH", &bin)
        .current_dir(&home);
    if let Some(desktop) = desktop {
        cmd.env("XDG_CURRENT_DESKTOP", desktop);
    }
    let out = cmd.output().unwrap();
    let after = snapshot(dir.path());
    (out, before, after)
}

fn text(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn ac7_gnome_gets_gsettings_commands_for_a_custom_keybinding() {
    for desktop in ["GNOME", "ubuntu:GNOME", "Unity:gnome"] {
        let (o, _, _) = hotkey(Some(desktop));
        assert_eq!(o.status.code(), Some(0), "{desktop}");
        let t = text(&o);
        assert!(
            t.contains(
                "gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings"
            ),
            "{t}"
        );
        assert!(t.contains("command 'easy-note quick'"), "{t}");
        assert!(t.contains("binding '<Control><Alt>n'"), "{t}");
        assert!(t.contains("name 'easy-note quick'"), "{t}");
        // existing shortcuts must not be wiped without a warning
        assert!(
            t.contains(
                "gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings"
            ),
            "{t}"
        );
    }
}

#[test]
fn ac7_kde_gets_the_system_settings_path_and_no_gsettings() {
    let (o, _, _) = hotkey(Some("KDE"));
    assert_eq!(o.status.code(), Some(0));
    let t = text(&o);
    assert!(t.contains("System Settings"), "{t}");
    assert!(t.contains("easy-note quick"), "{t}");
    assert!(!t.contains("gsettings"), "{t}");
    assert!(!t.contains("kwriteconfig"), "{t}");
}

#[test]
fn ac7_any_other_desktop_or_none_gets_the_generic_text() {
    for desktop in [Some("sway"), Some("X-Cinnamon"), Some(""), None] {
        let (o, _, _) = hotkey(desktop);
        assert_eq!(o.status.code(), Some(0), "{desktop:?}");
        let t = text(&o);
        assert!(t.contains("easy-note quick"), "{t}");
        assert!(!t.contains("gsettings set"), "{t}");
        assert!(!t.contains("System Settings"), "{t}");
    }
}

#[test]
fn ac7_hotkey_writes_nothing_and_runs_no_command() {
    for desktop in [Some("GNOME"), Some("KDE"), None] {
        let (o, before, after) = hotkey(desktop);
        assert_eq!(o.status.code(), Some(0));
        assert_eq!(before, after, "{desktop:?}: the file system is unchanged");
        assert!(
            !after.iter().any(|(p, _)| p.ends_with("tripwire")),
            "{desktop:?}: a command ran"
        );
    }
}
