//! HUN-277: `install obsidian` registers the notes dir as an Obsidian vault (FR-040, DDR 0008) and
//! `easy-note setup` runs the person-level installs in one command (FR-041).
//!
//! Every run clears the environment and uses a temp HOME, XDG_CONFIG_HOME, EASY_NOTE_DIR and
//! CLAUDE_CONFIG_DIR, and a temp `PATH` that holds only stub `xdg-mime` / `xdg-open` scripts. The
//! running-Obsidian check reads `$EASY_NOTE_PROC_DIR/<pid>/comm` (default `/proc`): every test points
//! it at a temp folder, so no test depends on a real Obsidian. The real `~/.config/obsidian` is never
//! read or written.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::Value;

const MIME_STUB: &str =
    "#!/bin/sh\nif [ -n \"$STUB_HANDLER\" ]; then printf '%s\\n' \"$STUB_HANDLER\"; fi\n";
const OPEN_STUB: &str = "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"$STUB_LOG/open.args\"\n";
const HANDLER: &str = "obsidian.desktop";

/// An Obsidian vault list with one other vault and an extra top-level key, as Obsidian writes it.
const OTHER: &str = r#"{"vaults":{"0123456789abcdef":{"path":"/elsewhere/vault","ts":1700000000000,"open":true}},"frame":"native","updateDisabled":true}"#;

struct Sb {
    dir: tempfile::TempDir,
}

impl Sb {
    fn new(name: &str) -> Sb {
        let sb = Sb {
            dir: tempfile::Builder::new()
                .prefix(&format!("easy-note-setup-{name}-"))
                .tempdir()
                .unwrap(),
        };
        for d in ["home", "bin", "log", "proc/1", "xdg"] {
            std::fs::create_dir_all(sb.root().join(d)).unwrap();
        }
        std::fs::write(sb.root().join("proc/1/comm"), "bash\n").unwrap();
        for (name, body) in [("xdg-mime", MIME_STUB), ("xdg-open", OPEN_STUB)] {
            let path = sb.root().join("bin").join(name);
            std::fs::write(&path, body).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        sb
    }
    fn root(&self) -> &Path {
        self.dir.path()
    }
    fn notes(&self) -> PathBuf {
        self.root().join("notes dir é")
    }
    fn vaults_file(&self) -> PathBuf {
        self.root().join("xdg/obsidian/obsidian.json")
    }
    fn write_vaults(&self, text: &str) {
        std::fs::create_dir_all(self.vaults_file().parent().unwrap()).unwrap();
        std::fs::write(self.vaults_file(), text).unwrap();
    }
    fn obsidian_runs(&self) {
        std::fs::create_dir_all(self.root().join("proc/4242")).unwrap();
        std::fs::write(self.root().join("proc/4242/comm"), "obsidian\n").unwrap();
    }
    fn run(&self, args: &[&str], handler: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_easy-note"))
            .args(args)
            .env_clear()
            .env("PATH", self.root().join("bin"))
            .env("HOME", self.root().join("home"))
            .env("XDG_CONFIG_HOME", self.root().join("xdg"))
            .env("EASY_NOTE_DIR", self.notes())
            .env("EASY_NOTE_PROC_DIR", self.root().join("proc"))
            .env("CLAUDE_CONFIG_DIR", self.root().join("claude"))
            .env("STUB_LOG", self.root().join("log"))
            .env("STUB_HANDLER", handler)
            .current_dir(self.root().join("home"))
            .stdin(Stdio::null())
            .output()
            .unwrap()
    }
    fn install(&self, handler: &str) -> Output {
        self.run(&["install", "obsidian"], handler)
    }
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

fn json_of(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Every file below `root` (not `log/`) with its bytes, and every directory.
fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut all = vec![];
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd {
            let p = e.unwrap().path();
            if p.ends_with("log") {
                continue;
            }
            if p.is_dir() && !p.is_symlink() {
                all.push((p.clone(), b"<dir>".to_vec()));
                stack.push(p);
            } else {
                all.push((p.clone(), std::fs::read(&p).unwrap_or_default()));
            }
        }
    }
    all.sort();
    all
}

fn backups(sb: &Sb) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(sb.root().join("xdg/obsidian"))
        .map(|rd| rd.map(|e| e.unwrap().path()).collect())
        .unwrap_or_default();
    found.retain(|p| {
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        name.starts_with("obsidian.json.easy-note-") && name.ends_with(".bak")
    });
    found
}

/// The vault entries whose `path` is the notes dir.
fn ours(v: &Value, notes: &Path) -> Vec<(String, Value)> {
    v["vaults"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, e)| e["path"] == notes.to_str().unwrap())
        .map(|(k, e)| (k.clone(), e.clone()))
        .collect()
}

// ---------------------------------------------------------------------------------------------
// AC 2: add the vault, keep the rest, back up, make `.obsidian/`; a second run writes nothing
// ---------------------------------------------------------------------------------------------

#[test]
fn ac2_install_adds_one_vault_keeps_the_rest_backs_up_and_is_idempotent() {
    let sb = Sb::new("add");
    sb.write_vaults(OTHER);
    std::fs::set_permissions(sb.vaults_file(), std::fs::Permissions::from_mode(0o600)).unwrap();
    let o = sb.install(HANDLER);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let out = text(&o.stdout);
    assert!(out.contains("easy-note.base"), "{out}");
    assert!(
        out.contains(&format!(
            "✓ added {} to Obsidian's vaults",
            sb.notes().display()
        )),
        "{out}"
    );

    let old: Value = serde_json::from_str(OTHER).unwrap();
    let new = json_of(&sb.vaults_file());
    let keys: Vec<&String> = new.as_object().unwrap().keys().collect();
    assert_eq!(
        keys,
        ["vaults", "frame", "updateDisabled"],
        "key order kept"
    );
    assert_eq!(new["frame"], old["frame"]);
    assert_eq!(new["updateDisabled"], old["updateDisabled"]);
    assert_eq!(
        new["vaults"]["0123456789abcdef"],
        old["vaults"]["0123456789abcdef"]
    );
    assert_eq!(new["vaults"].as_object().unwrap().len(), 2);
    let added = ours(&new, &sb.notes());
    assert_eq!(added.len(), 1, "{new}");
    let (key, entry) = &added[0];
    assert!(
        key.len() == 16
            && key
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
        "{key}"
    );
    assert_eq!(entry["open"], true);
    assert!(entry["ts"].as_u64().unwrap() > 1_700_000_000_000, "{entry}");
    assert_eq!(entry.as_object().unwrap().len(), 3, "{entry}");

    let mode = std::fs::metadata(sb.vaults_file())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600, "the old mode is kept");
    let b = backups(&sb);
    assert_eq!(b.len(), 1, "{b:?}");
    assert_eq!(std::fs::read_to_string(&b[0]).unwrap(), OTHER);
    let dot = sb.notes().join(".obsidian");
    assert!(dot.is_dir());
    assert_eq!(std::fs::read_dir(&dot).unwrap().count(), 0, "empty");

    let before = snapshot(sb.root());
    let o = sb.install(HANDLER);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(
        text(&o.stdout).contains(&format!(
            "already an Obsidian vault ({})",
            sb.notes().display()
        )),
        "{}",
        text(&o.stdout)
    );
    assert_eq!(snapshot(sb.root()), before, "the second run writes nothing");
}

/// Edge cases: a symlinked vault list stays a symlink; a missing `vaults` key is added.
#[test]
fn ac2_a_symlinked_vault_list_stays_a_link_and_a_missing_vaults_key_is_added() {
    let sb = Sb::new("link");
    let real = sb.root().join("xdg/real.json");
    std::fs::write(&real, r#"{"frame":"native"}"#).unwrap();
    std::fs::create_dir_all(sb.vaults_file().parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&real, sb.vaults_file()).unwrap();
    let o = sb.install(HANDLER);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(std::fs::symlink_metadata(sb.vaults_file())
        .unwrap()
        .is_symlink());
    let new = json_of(&real);
    assert_eq!(new["frame"], "native");
    assert_eq!(ours(&new, &sb.notes()).len(), 1, "{new}");
}

// ---------------------------------------------------------------------------------------------
// AC 3: a vault that is the notes dir or a parent covers it; a child does not
// ---------------------------------------------------------------------------------------------

#[test]
fn ac3_a_parent_vault_covers_the_notes_dir_and_nothing_is_written() {
    let sb = Sb::new("parent");
    let parent = format!("{}/", sb.root().display());
    let list = format!(r#"{{"vaults":{{"aaaa":{{"path":"{parent}","ts":1,"open":true}}}}}}"#);
    sb.write_vaults(&list);
    let o = sb.install(HANDLER);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(
        text(&o.stdout).contains(&format!("already an Obsidian vault ({parent})")),
        "{}",
        text(&o.stdout)
    );
    assert_eq!(std::fs::read_to_string(sb.vaults_file()).unwrap(), list);
    assert!(backups(&sb).is_empty());
    assert!(!sb.notes().join(".obsidian").exists());
}

#[test]
fn ac3_a_vault_inside_the_notes_dir_does_not_count() {
    let sb = Sb::new("child");
    let child = sb.notes().join("notes");
    sb.write_vaults(&format!(
        r#"{{"vaults":{{"aaaa":{{"path":"{}","ts":1,"open":true}}}}}}"#,
        child.display()
    ));
    let o = sb.install(HANDLER);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let new = json_of(&sb.vaults_file());
    assert_eq!(new["vaults"]["aaaa"]["path"], child.to_str().unwrap());
    assert_eq!(ours(&new, &sb.notes()).len(), 1, "{new}");
}

// ---------------------------------------------------------------------------------------------
// AC 4: Obsidian is open
// ---------------------------------------------------------------------------------------------

#[test]
fn ac4_when_obsidian_runs_nothing_is_written_to_the_list_and_it_says_to_close_it() {
    let sb = Sb::new("running");
    sb.write_vaults(OTHER);
    sb.obsidian_runs();
    let o = sb.install(HANDLER);
    assert_eq!(o.status.code(), Some(1), "{}", text(&o.stdout));
    assert!(
        text(&o.stderr).contains("Close Obsidian, then run easy-note setup again"),
        "{}",
        text(&o.stderr)
    );
    assert!(
        text(&o.stdout).contains("easy-note.base"),
        "the base step ran first: {}",
        text(&o.stdout)
    );
    assert!(sb.notes().join("easy-note.base").is_file());
    assert_eq!(std::fs::read_to_string(sb.vaults_file()).unwrap(), OTHER);
    assert!(backups(&sb).is_empty());
    assert!(!sb.notes().join(".obsidian").exists());
}

#[test]
fn ac4_when_obsidian_runs_but_the_dir_is_covered_it_is_already_done() {
    let sb = Sb::new("running-covered");
    sb.write_vaults(&format!(
        r#"{{"vaults":{{"aaaa":{{"path":"{}","ts":1,"open":true}}}}}}"#,
        sb.notes().display()
    ));
    sb.obsidian_runs();
    let o = sb.install(HANDLER);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(text(&o.stdout).contains("already an Obsidian vault"));
}

// ---------------------------------------------------------------------------------------------
// AC 5: Obsidian never started
// ---------------------------------------------------------------------------------------------

#[test]
fn ac5_no_handler_and_no_vault_list_skips_the_vault_step() {
    let sb = Sb::new("absent");
    let o = sb.install("");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(
        text(&o.stdout).contains("Obsidian is not installed: the vault step is skipped"),
        "{}",
        text(&o.stdout)
    );
    assert!(!sb.root().join("xdg/obsidian").exists());
    assert!(!sb.notes().join(".obsidian").exists());
}

#[test]
fn ac5_a_handler_and_no_vault_list_creates_the_list_with_only_our_vault() {
    let sb = Sb::new("fresh");
    let o = sb.install(HANDLER);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let new = json_of(&sb.vaults_file());
    assert_eq!(new.as_object().unwrap().len(), 1, "{new}");
    assert_eq!(new["vaults"].as_object().unwrap().len(), 1, "{new}");
    assert_eq!(ours(&new, &sb.notes()).len(), 1, "{new}");
    let mode = std::fs::metadata(sb.root().join("xdg/obsidian"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o700);
    assert!(backups(&sb).is_empty(), "no old file, no backup");
}

#[test]
fn ac5_a_vault_list_without_a_handler_still_gets_our_vault() {
    let sb = Sb::new("nohandler-list");
    sb.write_vaults(OTHER);
    let o = sb.install("");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert_eq!(ours(&json_of(&sb.vaults_file()), &sb.notes()).len(), 1);
}

#[test]
fn edge_xdg_config_home_unset_means_home_dot_config() {
    let sb = Sb::new("noxdg");
    let o = Command::new(env!("CARGO_BIN_EXE_easy-note"))
        .args(["install", "obsidian"])
        .env_clear()
        .env("PATH", sb.root().join("bin"))
        .env("HOME", sb.root().join("home"))
        .env("EASY_NOTE_DIR", sb.notes())
        .env("EASY_NOTE_PROC_DIR", sb.root().join("proc"))
        .env("STUB_HANDLER", HANDLER)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let list = sb.root().join("home/.config/obsidian/obsidian.json");
    assert_eq!(ours(&json_of(&list), &sb.notes()).len(), 1);
    assert!(!sb.vaults_file().exists());
}

// ---------------------------------------------------------------------------------------------
// AC 6: a vault list we cannot edit safely
// ---------------------------------------------------------------------------------------------

#[test]
fn ac6_a_bad_vault_list_is_left_as_it_is_and_named() {
    for bad in ["not json at all SECRETWORD", r#"{"vaults":[]}"#, "[1]"] {
        let sb = Sb::new("bad");
        sb.write_vaults(bad);
        let o = sb.install(HANDLER);
        let err = text(&o.stderr);
        assert_eq!(o.status.code(), Some(1), "{bad}: {}", text(&o.stdout));
        assert!(
            err.contains(sb.vaults_file().to_str().unwrap()),
            "{bad}: {err}"
        );
        assert!(err.contains("left as it is"), "{bad}: {err}");
        assert!(!err.contains("SECRETWORD"), "never quotes content: {err}");
        assert_eq!(std::fs::read_to_string(sb.vaults_file()).unwrap(), bad);
        assert!(backups(&sb).is_empty());
        assert!(!sb.notes().join(".obsidian").exists());
    }
}

// ---------------------------------------------------------------------------------------------
// AC 7: uninstall never edits the vault list
// ---------------------------------------------------------------------------------------------

#[test]
fn ac7_uninstall_obsidian_keeps_the_vault_and_its_folder() {
    let sb = Sb::new("uninstall");
    sb.write_vaults(OTHER);
    assert_eq!(sb.install(HANDLER).status.code(), Some(0));
    let list = std::fs::read(sb.vaults_file()).unwrap();
    let o = sb.run(&["uninstall", "obsidian"], HANDLER);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(!sb.notes().join("easy-note.base").exists());
    assert_eq!(std::fs::read(sb.vaults_file()).unwrap(), list);
    assert!(sb.notes().join(".obsidian").is_dir());
}

#[test]
fn ac7_the_readme_says_uninstall_keeps_the_vault_and_setup_is_the_way_in() {
    let readme =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")).unwrap();
    assert!(readme.contains("easy-note setup"), "README names setup");
    assert!(
        !readme.contains("Open the notes folder as a vault in Obsidian once"),
        "the manual vault step is gone"
    );
    assert!(
        readme.contains("never edits Obsidian's vault list"),
        "README says uninstall keeps the vault"
    );
}

// ---------------------------------------------------------------------------------------------
// AC 8: `easy-note setup`
// ---------------------------------------------------------------------------------------------

#[test]
fn ac8_setup_runs_each_install_under_a_heading_and_a_second_run_writes_nothing() {
    let sb = Sb::new("setup");
    sb.write_vaults(OTHER);
    let o = sb.run(&["setup"], HANDLER);
    let out = text(&o.stdout);
    assert_eq!(o.status.code(), Some(0), "{out}{}", text(&o.stderr));
    let at = |h: &str| {
        out.lines()
            .position(|l| l == h)
            .unwrap_or_else(|| panic!("{h} in {out}"))
    };
    assert!(at("launcher:") < at("obsidian:") && at("obsidian:") < at("tray:"));
    let home = sb.root().join("home");
    assert!(home
        .join(".local/share/applications/easy-note.desktop")
        .is_file());
    assert!(home
        .join(".local/share/icons/hicolor/scalable/apps/easy-note.svg")
        .is_file());
    assert!(sb.notes().join("easy-note.base").is_file());
    assert_eq!(ours(&json_of(&sb.vaults_file()), &sb.notes()).len(), 1);
    let autostart = home.join(".config/autostart/easy-note-tray.desktop");
    if cfg!(feature = "tray") {
        assert!(autostart.is_file());
    } else {
        assert!(!autostart.exists());
        assert!(out.contains("tray not built in: skipped"), "{out}");
    }
    assert!(
        out.trim_end()
            .lines()
            .last()
            .unwrap()
            .contains("easy-note install claude"),
        "{out}"
    );
    assert!(
        !sb.root().join("claude").exists(),
        "setup never runs install claude"
    );

    let before = snapshot(sb.root());
    let o = sb.run(&["setup"], HANDLER);
    let out = text(&o.stdout);
    assert_eq!(o.status.code(), Some(0), "{out}{}", text(&o.stderr));
    assert_eq!(snapshot(sb.root()), before, "the second run writes nothing");
    assert!(out.matches("already").count() >= 3, "{out}");
}

#[test]
fn ac8_a_failing_step_does_not_stop_the_next_and_setup_exits_1() {
    let sb = Sb::new("setup-fail");
    // A launcher file that is not ours makes the launcher step fail.
    let desktop = sb
        .root()
        .join("home/.local/share/applications/easy-note.desktop");
    std::fs::create_dir_all(desktop.parent().unwrap()).unwrap();
    std::fs::write(&desktop, "[Desktop Entry]\nName=mine\n").unwrap();
    let o = sb.run(&["setup"], HANDLER);
    assert_eq!(o.status.code(), Some(1), "{}", text(&o.stdout));
    assert!(text(&o.stderr).contains("easy-note.desktop"));
    assert!(sb.notes().join("easy-note.base").is_file(), "obsidian ran");
    assert_eq!(ours(&json_of(&sb.vaults_file()), &sb.notes()).len(), 1);
    assert!(text(&o.stdout).contains("tray:"), "{}", text(&o.stdout));
}

#[test]
fn ac8_setup_while_obsidian_runs_exits_1_and_leaves_the_list() {
    let sb = Sb::new("setup-running");
    sb.write_vaults(OTHER);
    sb.obsidian_runs();
    let o = sb.run(&["setup"], HANDLER);
    assert_eq!(o.status.code(), Some(1));
    assert!(text(&o.stderr).contains("Close Obsidian, then run easy-note setup again"));
    assert_eq!(std::fs::read_to_string(sb.vaults_file()).unwrap(), OTHER);
}

// ---------------------------------------------------------------------------------------------
// AC 10: help order; no other command writes the vault list or `.obsidian/`
// ---------------------------------------------------------------------------------------------

#[test]
fn ac10_help_lists_setup_before_install() {
    let sb = Sb::new("help");
    let help = text(&sb.run(&["--help"], "").stdout);
    let at = |cmd: &str| {
        help.lines()
            .position(|l| l.trim_start().starts_with(&format!("{cmd} ")))
            .unwrap_or_else(|| panic!("{cmd} in {help}"))
    };
    assert!(at("setup") < at("install"), "{help}");
    assert!(at("setup") < at("uninstall"), "{help}");
}

#[test]
fn ac10_no_other_command_writes_the_vault_list_or_dot_obsidian() {
    let sb = Sb::new("others");
    sb.write_vaults(OTHER);
    let add = sb.run(&["add", "first", "#t"], HANDLER);
    assert_eq!(add.status.code(), Some(0), "{}", text(&add.stderr));
    let id = text(&add.stdout)
        .split_whitespace()
        .nth(2)
        .unwrap()
        .to_string();
    for args in [
        vec!["ls"],
        vec!["search", "first"],
        vec!["show", &id],
        vec!["stats"],
        vec!["doctor"],
        vec!["path"],
        vec!["hotkey"],
        vec!["tag", &id, "more"],
        vec!["init"],
        vec!["ai", "render"],
        vec!["dashboard"],
        vec!["install", "claude"],
        vec!["uninstall", "claude"],
        vec!["install", "launcher"],
        vec!["uninstall", "launcher"],
        vec!["install", "tray"],
        vec!["uninstall", "tray"],
        vec!["uninstall", "obsidian"],
        vec!["rm", &id],
        vec!["restore", &id],
    ] {
        sb.run(&args, HANDLER);
        assert_eq!(
            std::fs::read_to_string(sb.vaults_file()).unwrap(),
            OTHER,
            "{args:?} wrote the vault list"
        );
        assert!(
            !sb.notes().join(".obsidian").exists(),
            "{args:?} made .obsidian"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// AC 11: the store ignores `.obsidian/`
// ---------------------------------------------------------------------------------------------

#[test]
fn ac11_the_store_is_unchanged_with_dot_obsidian_present() {
    let sb = Sb::new("store");
    sb.run(&["add", "first", "#t"], HANDLER);
    let ls = || text(&sb.run(&["ls"], HANDLER).stdout);
    let stats = || text(&sb.run(&["stats", "--json"], HANDLER).stdout);
    let (ls0, stats0) = (ls(), stats());
    assert_eq!(sb.install(HANDLER).status.code(), Some(0));
    let dot = sb.notes().join(".obsidian");
    assert!(dot.is_dir());
    // Obsidian fills the folder; none of it is a note or a skipped file.
    std::fs::write(dot.join("app.json"), "{}").unwrap();
    std::fs::write(dot.join("workspace.md"), "not a note").unwrap();
    assert_eq!(ls(), ls0);
    assert_eq!(stats(), stats0);
}
