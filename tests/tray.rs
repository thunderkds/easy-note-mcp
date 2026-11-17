//! HUN-263 (FR-033): `easy-note tray` in a build with the `tray` feature. No test shows a real
//! tray: the session bus points nowhere, so the run ends at once. Temp HOME and notes dir only.
#![cfg(feature = "tray")]

use std::path::Path;
use std::process::{Command, Output};

fn easy(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_easy-note"))
        .args(args)
        .env("HOME", home)
        .env("EASY_NOTE_DIR", home.join("notes"))
        // a bus address that points nowhere: no session bus, so no tray host
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path={}", home.join("none").display()),
        )
        .env_remove("XDG_RUNTIME_DIR")
        .current_dir(home)
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// AC 3: with no tray host the command exits 1 and says what to do.
#[test]
fn ac3_no_tray_host_exits_1_with_the_hotkey_hint() {
    let home = tempfile::tempdir().unwrap();
    let out = easy(home.path(), &["tray"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
    let said = text(&out.stderr);
    assert!(said.contains("no tray host"), "{said}");
    assert!(
        said.contains("GNOME needs the AppIndicator extension; the one-press path is a hotkey: run easy-note hotkey"),
        "{said}"
    );
    assert!(text(&out.stdout).is_empty(), "{}", text(&out.stdout));
    assert!(
        !home.path().join(".config/autostart").exists(),
        "no autostart file"
    );
}

/// AC 3: `doctor` in a tray build reports the missing host, and it is no problem with the notes.
#[test]
fn ac3_doctor_reports_a_missing_tray_host() {
    let home = tempfile::tempdir().unwrap();
    let out = easy(home.path(), &["doctor"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let said = text(&out.stdout);
    assert!(said.contains("tray: no tray host ("), "{said}");
    assert!(said.contains("no problems found"), "{said}");
}
