//! HUN-259: `easy-note install claude` and `uninstall claude` on temp settings files (FR-019,
//! DDR 0003). Every run uses a temp HOME and, where it matters, a temp `CLAUDE_CONFIG_DIR`;
//! the real `~/.claude` is never read or written.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};

const OWN_COMMAND: &str = "easy-note hook-prompt";

/// An easy-memory style entry, exactly as `serde_json` prints it inside `hooks.UserPromptSubmit`
/// with 2-space indent.
const MEMORY_ENTRY: &str = r#"      {
        "hooks": [
          {
            "type": "command",
            "command": "python3 /opt/easy-memory/easy_memory.py hook-prompt",
            "timeout": 5
          }
        ]
      }"#;

/// A user settings file with other keys, an easy-memory entry and a `Stop` hook.
fn settings_with_memory() -> String {
    format!(
        r#"{{
  "model": "opus",
  "env": {{
    "A": "1"
  }},
  "hooks": {{
    "UserPromptSubmit": [
{MEMORY_ENTRY}
    ],
    "Stop": [
      {{
        "hooks": [
          {{
            "type": "command",
            "command": "echo stopped"
          }}
        ]
      }}
    ]
  }}
}}
"#
    )
}

struct Sb {
    dir: tempfile::TempDir,
    /// Stub programs, outside `dir` so the file snapshots never see them.
    bin: tempfile::TempDir,
}

impl Sb {
    fn new(name: &str) -> Sb {
        let sb = Sb {
            dir: tempfile::Builder::new()
                .prefix(&format!("easy-note-install-{name}-"))
                .tempdir()
                .unwrap(),
            bin: tempfile::Builder::new()
                .prefix("easy-note-install-bin-")
                .tempdir()
                .unwrap(),
        };
        std::fs::create_dir_all(sb.home()).unwrap();
        // HUN-277: `install obsidian` asks `xdg-mime` for the Obsidian handler and reads Obsidian's
        // vault list and the process list. A stub with no handler, a temp XDG_CONFIG_HOME and an
        // empty process dir keep every run off the real machine.
        let bin = sb.bin.path();
        std::fs::write(bin.join("xdg-mime"), "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(
            bin.join("xdg-mime"),
            std::os::unix::fs::PermissionsExt::from_mode(0o755),
        )
        .unwrap();
        sb
    }
    fn root(&self) -> &Path {
        self.dir.path()
    }
    fn home(&self) -> PathBuf {
        self.root().join("home")
    }
    /// The config dir when `CLAUDE_CONFIG_DIR` is not set.
    fn default_cfg(&self) -> PathBuf {
        self.home().join(".claude")
    }
    fn cfg(&self) -> PathBuf {
        self.root().join("cfg")
    }
}

fn easy(sb: &Sb, args: &[&str], cfg: Option<&Path>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_easy-note"));
    let path = format!(
        "{}:{}",
        sb.bin.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    cmd.args(args)
        .env("HOME", sb.home())
        .env("PATH", path)
        .env("XDG_CONFIG_HOME", sb.root().join("xdg"))
        .env("EASY_NOTE_PROC_DIR", sb.root().join("proc"))
        .env("EASY_NOTE_DIR", sb.root().join("notes"))
        .current_dir(sb.home());
    match cfg {
        Some(dir) => cmd.env("CLAUDE_CONFIG_DIR", dir),
        None => cmd.env_remove("CLAUDE_CONFIG_DIR"),
    };
    cmd.output().unwrap()
}

fn install(sb: &Sb, cfg: Option<&Path>) -> Output {
    easy(sb, &["install", "claude"], cfg)
}

fn uninstall(sb: &Sb, cfg: Option<&Path>) -> Output {
    easy(sb, &["uninstall", "claude"], cfg)
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn json_of(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn backups(cfg: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(cfg)
        .map(|rd| rd.map(|e| e.unwrap().path()).collect())
        .unwrap_or_default();
    found.retain(|p| {
        p.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("settings.json.easy-note-")
    });
    found.sort();
    found
}

/// `settings.json.easy-note-YYYYMMDD-HHMMSS.bak`, with an optional `-N` before `.bak` when a
/// second run in the same second would have collided.
fn is_backup_name(name: &str) -> bool {
    let Some(rest) = name
        .strip_prefix("settings.json.easy-note-")
        .and_then(|r| r.strip_suffix(".bak"))
    else {
        return false;
    };
    let (stamp, extra) = rest.split_at(rest.len().min(15));
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    stamp.len() == 15
        && digits(&stamp[..8])
        && &stamp[8..9] == "-"
        && digits(&stamp[9..])
        && (extra.is_empty() || (extra.starts_with('-') && digits(&extra[1..])))
}

fn own_entries(settings: &Value) -> usize {
    settings["hooks"]["UserPromptSubmit"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter(|e| {
                    e["hooks"]
                        .as_array()
                        .is_some_and(|h| h.iter().any(|x| x["command"] == OWN_COMMAND))
                })
                .count()
        })
        .unwrap_or(0)
}

fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut all = vec![];
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd {
            let p = e.unwrap().path();
            if p.is_dir() && !p.is_symlink() {
                stack.push(p);
            } else {
                all.push((p.clone(), std::fs::read(&p).unwrap_or_default()));
            }
        }
    }
    all.sort();
    all
}

// ---------------------------------------------------------------------------------------------
// AC 6: install
// ---------------------------------------------------------------------------------------------

#[test]
fn ac6_install_prints_a_diff_backs_up_and_adds_exactly_one_entry_and_note_md() {
    let sb = Sb::new("ac6");
    let cfg = sb.cfg();
    let original = settings_with_memory();
    write(&cfg.join("settings.json"), &original);

    let o = install(&sb, Some(&cfg));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let stdout = text(&o.stdout);
    let settings_path = cfg.join("settings.json");
    assert!(
        stdout.contains(&format!("--- {}", settings_path.display())),
        "{stdout}"
    );
    assert!(
        stdout.contains(&format!("+++ {}", settings_path.display())),
        "{stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.starts_with('+') && l.contains(OWN_COMMAND)),
        "the diff shows the added command: {stdout}"
    );

    // a timestamped backup with the old bytes
    let baks = backups(&cfg);
    assert_eq!(baks.len(), 1, "{baks:?}");
    assert!(
        is_backup_name(&baks[0].file_name().unwrap().to_string_lossy()),
        "{baks:?}"
    );
    assert_eq!(std::fs::read_to_string(&baks[0]).unwrap(), original);

    // exactly one entry: command, timeout 5
    let new = json_of(&settings_path);
    assert_eq!(own_entries(&new), 1);
    let list = new["hooks"]["UserPromptSubmit"].as_array().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(
        list[1],
        json!({"hooks":[{"type":"command","command":OWN_COMMAND,"timeout":5}]})
    );
    // the command file
    let note_md = cfg.join("commands/note.md");
    let md = std::fs::read_to_string(&note_md).unwrap();
    assert!(md.contains("/note"), "{md}");
    assert!(md.contains("hook"), "{md}");
}

#[test]
fn ac6_running_install_twice_changes_nothing() {
    let sb = Sb::new("ac6-twice");
    let cfg = sb.cfg();
    write(&cfg.join("settings.json"), &settings_with_memory());
    assert_eq!(install(&sb, Some(&cfg)).status.code(), Some(0));
    let after_first = snapshot(sb.root());
    assert_eq!(backups(&cfg).len(), 1);

    let o = install(&sb, Some(&cfg));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(
        text(&o.stdout).contains("already installed"),
        "{}",
        text(&o.stdout)
    );
    assert_eq!(
        snapshot(sb.root()),
        after_first,
        "the second run changed a file"
    );
    assert_eq!(backups(&cfg).len(), 1, "the second run wrote a backup");
}

#[test]
fn ac6_a_backup_is_never_overwritten() {
    let sb = Sb::new("ac6-bak");
    let cfg = sb.cfg();
    let original = settings_with_memory();
    write(&cfg.join("settings.json"), &original);
    assert_eq!(install(&sb, Some(&cfg)).status.code(), Some(0));
    let first = backups(&cfg);
    assert_eq!(first.len(), 1);
    // install and uninstall in the same second would pick the same stamp
    assert_eq!(uninstall(&sb, Some(&cfg)).status.code(), Some(0));
    let all = backups(&cfg);
    assert_eq!(all.len(), 2, "{all:?}");
    assert_eq!(
        std::fs::read_to_string(&first[0]).unwrap(),
        original,
        "the first backup changed"
    );
    for b in &all {
        assert!(
            is_backup_name(&b.file_name().unwrap().to_string_lossy()),
            "{b:?}"
        );
    }
}

#[test]
fn ac6_a_missing_settings_file_is_created_with_only_our_entry_and_needs_no_backup() {
    let sb = Sb::new("ac6-missing");
    let cfg = sb.cfg();
    let o = install(&sb, Some(&cfg));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert_eq!(
        json_of(&cfg.join("settings.json")),
        json!({"hooks":{"UserPromptSubmit":[
            {"hooks":[{"type":"command","command":OWN_COMMAND,"timeout":5}]}
        ]}})
    );
    assert!(backups(&cfg).is_empty());
    assert!(cfg.join("commands/note.md").is_file());
}

#[test]
fn ac6_a_note_md_the_person_wrote_is_kept() {
    let sb = Sb::new("ac6-md");
    let cfg = sb.cfg();
    write(&cfg.join("commands/note.md"), "my own /note command\n");
    let o = install(&sb, Some(&cfg));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert_eq!(
        std::fs::read_to_string(cfg.join("commands/note.md")).unwrap(),
        "my own /note command\n"
    );
    assert_eq!(own_entries(&json_of(&cfg.join("settings.json"))), 1);
    // and uninstall leaves it too
    assert_eq!(uninstall(&sb, Some(&cfg)).status.code(), Some(0));
    assert_eq!(
        std::fs::read_to_string(cfg.join("commands/note.md")).unwrap(),
        "my own /note command\n"
    );
}

#[test]
fn ac6_install_restores_a_deleted_note_md_without_touching_settings() {
    let sb = Sb::new("ac6-md-restore");
    let cfg = sb.cfg();
    assert_eq!(install(&sb, Some(&cfg)).status.code(), Some(0));
    std::fs::remove_file(cfg.join("commands/note.md")).unwrap();
    let settings = std::fs::read(cfg.join("settings.json")).unwrap();
    let o = install(&sb, Some(&cfg));
    assert_eq!(o.status.code(), Some(0));
    assert!(cfg.join("commands/note.md").is_file());
    assert_eq!(std::fs::read(cfg.join("settings.json")).unwrap(), settings);
    assert!(
        backups(&cfg).is_empty(),
        "settings did not change, so no backup"
    );
}

// ---------------------------------------------------------------------------------------------
// AC 6b: the config dir
// ---------------------------------------------------------------------------------------------

#[test]
fn ac6b_claude_config_dir_is_the_only_place_touched_and_the_diff_names_it() {
    let sb = Sb::new("ac6b");
    let cfg = sb.cfg();
    write(&cfg.join("settings.json"), &settings_with_memory());
    let o = install(&sb, Some(&cfg));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(
        text(&o.stdout).contains(&cfg.display().to_string()),
        "{}",
        text(&o.stdout)
    );
    assert!(!sb.default_cfg().exists(), "~/.claude must not be created");

    let mut touched: Vec<PathBuf> = snapshot(sb.root())
        .into_iter()
        .map(|(p, _)| p.strip_prefix(sb.root()).unwrap().to_path_buf())
        .collect();
    touched.retain(|p| !p.starts_with("home") || p.starts_with("home/.claude"));
    assert!(
        touched.iter().all(|p| p.starts_with("cfg")),
        "files outside CLAUDE_CONFIG_DIR: {touched:?}"
    );
    let names: Vec<String> = touched
        .iter()
        .map(|p| p.strip_prefix("cfg").unwrap().display().to_string())
        .filter(|n| !n.starts_with("settings.json.easy-note-"))
        .collect();
    assert_eq!(names, ["commands/note.md", "settings.json"]);

    let o = uninstall(&sb, Some(&cfg));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(
        text(&o.stdout).contains(&cfg.display().to_string()),
        "{}",
        text(&o.stdout)
    );
    assert!(!sb.default_cfg().exists());
}

#[test]
fn ac6b_without_claude_config_dir_the_dir_is_home_dot_claude() {
    for set_empty in [false, true] {
        let sb = Sb::new("ac6b-home");
        let empty = PathBuf::new();
        let cfg_arg = set_empty.then_some(empty.as_path());
        let o = install(&sb, cfg_arg);
        assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
        let dir = sb.default_cfg();
        assert_eq!(own_entries(&json_of(&dir.join("settings.json"))), 1);
        assert!(dir.join("commands/note.md").is_file());
        assert!(
            text(&o.stdout).contains(&dir.display().to_string()),
            "{}",
            text(&o.stdout)
        );
        assert!(!sb.cfg().exists());
        let o = uninstall(&sb, cfg_arg);
        assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
        assert_eq!(own_entries(&json_of(&dir.join("settings.json"))), 0);
        assert!(!dir.join("commands/note.md").exists());
    }
}

// ---------------------------------------------------------------------------------------------
// AC 7: other entries are untouched; a symlink stays a symlink
// ---------------------------------------------------------------------------------------------

#[test]
fn ac7_other_hooks_and_keys_are_value_and_byte_identical_after_install_and_uninstall() {
    let sb = Sb::new("ac7");
    let cfg = sb.cfg();
    let original = settings_with_memory();
    let before: Value = serde_json::from_str(&original).unwrap();
    write(&cfg.join("settings.json"), &original);

    assert_eq!(install(&sb, Some(&cfg)).status.code(), Some(0));
    let installed = std::fs::read_to_string(cfg.join("settings.json")).unwrap();
    assert!(
        installed.contains(MEMORY_ENTRY),
        "easy-memory's entry text changed:\n{installed}"
    );
    let mid: Value = serde_json::from_str(&installed).unwrap();
    assert_eq!(mid["model"], before["model"]);
    assert_eq!(mid["env"], before["env"]);
    assert_eq!(mid["hooks"]["Stop"], before["hooks"]["Stop"]);
    assert_eq!(
        mid["hooks"]["UserPromptSubmit"][0],
        before["hooks"]["UserPromptSubmit"][0]
    );
    // key order is kept
    let keys: Vec<&String> = mid.as_object().unwrap().keys().collect();
    assert_eq!(keys, ["model", "env", "hooks"]);

    assert_eq!(uninstall(&sb, Some(&cfg)).status.code(), Some(0));
    let after = std::fs::read_to_string(cfg.join("settings.json")).unwrap();
    assert_eq!(
        after, original,
        "install then uninstall is not byte-identical"
    );
}

#[test]
fn ac7_a_symlinked_settings_file_stays_a_symlink() {
    let sb = Sb::new("ac7-link");
    let cfg = sb.cfg();
    let dotfiles = sb.root().join("dotfiles");
    let target = dotfiles.join("claude-settings.json");
    write(&target, &settings_with_memory());
    std::fs::create_dir_all(&cfg).unwrap();
    let link = cfg.join("settings.json");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let o = install(&sb, Some(&cfg));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(std::fs::symlink_metadata(&link).unwrap().is_symlink());
    assert_eq!(std::fs::read_link(&link).unwrap(), target);
    assert_eq!(own_entries(&json_of(&target)), 1);
    assert!(!std::fs::symlink_metadata(&target).unwrap().is_symlink());
    assert_eq!(
        files_in(&dotfiles),
        ["claude-settings.json"],
        "no stray temp file"
    );

    assert_eq!(uninstall(&sb, Some(&cfg)).status.code(), Some(0));
    assert!(std::fs::symlink_metadata(&link).unwrap().is_symlink());
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        settings_with_memory()
    );
}

fn files_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn ac7_the_file_mode_is_kept() {
    use std::os::unix::fs::PermissionsExt;
    let sb = Sb::new("ac7-mode");
    let cfg = sb.cfg();
    let path = cfg.join("settings.json");
    write(&path, &settings_with_memory());
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
    assert_eq!(install(&sb, Some(&cfg)).status.code(), Some(0));
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o640);
}

// ---------------------------------------------------------------------------------------------
// AC 8: uninstall
// ---------------------------------------------------------------------------------------------

#[test]
fn ac8_uninstall_removes_exactly_its_entry_and_note_md() {
    let sb = Sb::new("ac8");
    let cfg = sb.cfg();
    write(&cfg.join("settings.json"), &settings_with_memory());
    assert_eq!(install(&sb, Some(&cfg)).status.code(), Some(0));
    assert!(cfg.join("commands/note.md").is_file());

    let o = uninstall(&sb, Some(&cfg));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let stdout = text(&o.stdout);
    assert!(
        stdout
            .lines()
            .any(|l| l.starts_with('-') && l.contains(OWN_COMMAND)),
        "the diff shows the removed command: {stdout}"
    );
    assert_eq!(own_entries(&json_of(&cfg.join("settings.json"))), 0);
    assert_eq!(
        json_of(&cfg.join("settings.json")),
        serde_json::from_str::<Value>(&settings_with_memory()).unwrap()
    );
    assert!(!cfg.join("commands/note.md").exists());
}

#[test]
fn ac8_uninstall_on_a_file_made_by_install_leaves_no_empty_hooks_and_nothing_to_do_says_so() {
    let sb = Sb::new("ac8-empty");
    let cfg = sb.cfg();
    assert_eq!(install(&sb, Some(&cfg)).status.code(), Some(0));
    assert_eq!(uninstall(&sb, Some(&cfg)).status.code(), Some(0));
    assert_eq!(json_of(&cfg.join("settings.json")), json!({}));
    assert!(!cfg.join("commands/note.md").exists());
    let before = snapshot(sb.root());
    let o = uninstall(&sb, Some(&cfg));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(
        text(&o.stdout).contains("not installed"),
        "{}",
        text(&o.stdout)
    );
    assert_eq!(
        snapshot(sb.root()),
        before,
        "a no-op uninstall changed something"
    );
}

#[test]
fn ac8_uninstall_with_no_settings_file_creates_nothing() {
    let sb = Sb::new("ac8-none");
    let o = uninstall(&sb, Some(&sb.cfg()));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(
        text(&o.stdout).contains("not installed"),
        "{}",
        text(&o.stdout)
    );
    assert!(!sb.cfg().exists());
}

#[test]
fn ac8_a_hook_entry_that_holds_other_commands_loses_only_ours() {
    let sb = Sb::new("ac8-shared");
    let cfg = sb.cfg();
    let shared = json!({"hooks":{"UserPromptSubmit":[
        {"matcher":"","hooks":[
            {"type":"command","command":"keep-me"},
            {"type":"command","command":OWN_COMMAND,"timeout":5}
        ]}
    ]}});
    let shared_text = serde_json::to_string_pretty(&shared).unwrap();
    write(&cfg.join("settings.json"), &shared_text);
    let o = install(&sb, Some(&cfg));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    // our command is already in an entry: settings stay as they are, no backup
    assert_eq!(
        std::fs::read_to_string(cfg.join("settings.json")).unwrap(),
        shared_text
    );
    assert!(backups(&cfg).is_empty());
    assert_eq!(uninstall(&sb, Some(&cfg)).status.code(), Some(0));
    assert_eq!(
        json_of(&cfg.join("settings.json")),
        json!({"hooks":{"UserPromptSubmit":[
            {"matcher":"","hooks":[{"type":"command","command":"keep-me"}]}
        ]}})
    );
}

// ---------------------------------------------------------------------------------------------
// AC 11: a settings file we cannot edit safely
// ---------------------------------------------------------------------------------------------

#[test]
fn ac11_settings_we_cannot_edit_safely_exit_1_name_the_file_and_change_nothing() {
    let bad = [
        ("comment", "// my hooks\n{}\n"),
        ("block comment", "{ /* x */ \"a\": 1 }"),
        ("invalid json", "{\"hooks\": "),
        ("empty file", ""),
        ("array", "[]"),
        ("string", "\"settings\""),
        ("hooks is a list", "{\"hooks\": []}"),
        (
            "event is an object",
            "{\"hooks\":{\"UserPromptSubmit\":{}}}",
        ),
        (
            "event is a string",
            "{\"hooks\":{\"UserPromptSubmit\":\"x\"}}",
        ),
    ];
    for (what, content) in bad {
        for op in ["install", "uninstall"] {
            let sb = Sb::new("ac11");
            let cfg = sb.cfg();
            let path = cfg.join("settings.json");
            write(&path, content);
            let before = snapshot(sb.root());
            let o = easy(&sb, &[op, "claude"], Some(&cfg));
            let stderr = text(&o.stderr);
            assert_eq!(o.status.code(), Some(1), "{what} {op}: {stderr}");
            assert!(
                stderr.contains(&path.display().to_string()),
                "{what} {op}: {stderr}"
            );
            assert_eq!(
                snapshot(sb.root()),
                before,
                "{what} {op}: something changed"
            );
            assert!(backups(&cfg).is_empty(), "{what} {op}");
            assert!(!cfg.join("commands").exists(), "{what} {op}");
        }
    }
}

// ---------------------------------------------------------------------------------------------
// usage
// ---------------------------------------------------------------------------------------------

#[test]
fn an_unknown_install_target_is_a_usage_error() {
    let sb = Sb::new("usage");
    for args in [
        vec!["install"],
        vec!["install", "vim"],
        vec!["uninstall", "vim"],
    ] {
        let o = easy(&sb, &args, Some(&sb.cfg()));
        assert_eq!(o.status.code(), Some(2), "{args:?}");
    }
    assert!(!sb.cfg().exists());
}

// ---------------------------------------------------------------------------------------------
// HUN-260 AC 8: `install launcher` / `uninstall launcher`
// ---------------------------------------------------------------------------------------------

const LAUNCHER: &str = "[Desktop Entry]\nType=Application\nName=easy-note\nComment=Your notes, one press away\nExec=easy-note dashboard\nIcon=easy-note\nTerminal=false\nCategories=Utility;\nActions=new-note;\n\n[Desktop Action new-note]\nName=New note\nExec=easy-note quick\n";
/// The launcher text before HUN-276 (an upgrade target).
const LAUNCHER_V1: &str = "[Desktop Entry]\nType=Application\nName=easy-note\nComment=Save a one-line note\nExec=easy-note quick\nTerminal=false\nCategories=Utility;\n";

fn launcher_path(sb: &Sb) -> PathBuf {
    sb.home()
        .join(".local/share/applications/easy-note.desktop")
}

/// HUN-276 AC 2: the five files `install launcher` owns, relative to HOME.
const ICON_FILES: [(&str, &str); 4] = [
    (
        ".local/share/icons/hicolor/scalable/apps/easy-note.svg",
        "easy-note.svg",
    ),
    (
        ".local/share/icons/hicolor/16x16/apps/easy-note.svg",
        "easy-note-16.svg",
    ),
    (
        ".local/share/icons/hicolor/symbolic/apps/easy-note-symbolic.svg",
        "easy-note-symbolic.svg",
    ),
    (
        ".local/share/icons/hicolor/symbolic/apps/easy-note-saved-symbolic.svg",
        "easy-note-saved-symbolic.svg",
    ),
];

fn asset(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/assets/icons/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn launcher(sb: &Sb, verb: &str) -> Output {
    easy(sb, &[verb, "launcher"], Some(&sb.cfg()))
}

#[test]
fn ac8_install_launcher_writes_exactly_the_launcher_and_its_icons() {
    let sb = Sb::new("launcher");
    let o = launcher(&sb, "install");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let files: Vec<PathBuf> = snapshot(sb.root()).into_iter().map(|(p, _)| p).collect();
    let mut want: Vec<PathBuf> = ICON_FILES
        .iter()
        .map(|(rel, _)| sb.home().join(rel))
        .collect();
    want.push(launcher_path(&sb));
    want.sort();
    assert_eq!(files, want, "no other file anywhere");
    let body = std::fs::read_to_string(launcher_path(&sb)).unwrap();
    assert_eq!(body, LAUNCHER);
    for (rel, name) in ICON_FILES {
        assert_eq!(
            std::fs::read_to_string(sb.home().join(rel)).unwrap(),
            asset(name),
            "{rel}"
        );
    }
    assert!(
        text(&o.stdout).contains("easy-note.desktop"),
        "{}",
        text(&o.stdout)
    );
    assert!(
        !sb.cfg().exists(),
        "the Claude Code config dir is not touched"
    );
    assert!(!sb.default_cfg().exists());
}

/// HUN-276 AC 3: the launcher passes the freedesktop validator when it is installed.
#[test]
fn ac3_the_launcher_text_passes_desktop_file_validate() {
    let sb = Sb::new("launcher-validate");
    launcher(&sb, "install");
    match Command::new("desktop-file-validate")
        .arg(launcher_path(&sb))
        .output()
    {
        Ok(o) => assert!(o.status.success(), "{}{}", text(&o.stdout), text(&o.stderr)),
        Err(_) => eprintln!("skipped: desktop-file-validate is not installed"),
    }
}

/// HUN-276 AC 4: an install made by the old launcher text is upgraded and the change is shown.
#[test]
fn ac4_the_old_launcher_text_is_upgraded_and_the_change_is_shown() {
    let sb = Sb::new("launcher-v1");
    write(&launcher_path(&sb), LAUNCHER_V1);
    let o = launcher(&sb, "install");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert_eq!(
        std::fs::read_to_string(launcher_path(&sb)).unwrap(),
        LAUNCHER
    );
    let said = text(&o.stdout);
    assert!(said.contains("-Exec=easy-note quick"), "{said}");
    assert!(said.contains("+Exec=easy-note dashboard"), "{said}");
    for (rel, _) in ICON_FILES {
        assert!(sb.home().join(rel).exists(), "{rel}");
    }
    // the old text, uninstalled, is also ours
    let sb = Sb::new("launcher-v1-uninstall");
    write(&launcher_path(&sb), LAUNCHER_V1);
    let o = launcher(&sb, "uninstall");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(!launcher_path(&sb).exists());
}

#[test]
fn ac2_a_second_install_says_already_installed_and_writes_nothing() {
    let sb = Sb::new("launcher-twice");
    launcher(&sb, "install");
    let before = snapshot(sb.root());
    let o = launcher(&sb, "install");
    assert_eq!(o.status.code(), Some(0));
    assert!(
        text(&o.stdout).contains("already installed"),
        "{}",
        text(&o.stdout)
    );
    assert_eq!(snapshot(sb.root()), before);
}

/// A person deleted one icon by hand: the next install writes only that one.
#[test]
fn ac2_a_deleted_icon_is_written_again_and_the_rest_stay() {
    let sb = Sb::new("launcher-missing-icon");
    launcher(&sb, "install");
    let gone = sb.home().join(ICON_FILES[1].0);
    std::fs::remove_file(&gone).unwrap();
    let o = launcher(&sb, "install");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert_eq!(
        std::fs::read_to_string(&gone).unwrap(),
        asset("easy-note-16.svg")
    );
    let said = text(&o.stdout);
    assert_eq!(said.matches("wrote").count(), 1, "{said}");
}

#[test]
fn ac8_a_desktop_file_that_is_not_ours_is_kept_and_named() {
    let sb = Sb::new("launcher-foreign");
    let path = launcher_path(&sb);
    write(&path, "[Desktop Entry]\nName=mine\n");
    let before = snapshot(sb.root());
    let o = launcher(&sb, "install");
    assert_eq!(o.status.code(), Some(1), "{}", text(&o.stdout));
    let said = text(&o.stderr);
    assert!(said.contains("kept"), "{said}");
    assert!(said.contains(path.to_str().unwrap()), "{said}");
    assert_eq!(snapshot(sb.root()), before, "nothing written");
    let o = launcher(&sb, "uninstall");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let said = text(&o.stdout);
    assert!(said.contains("kept"), "{said}");
    assert!(said.contains(path.to_str().unwrap()), "{said}");
    assert_eq!(snapshot(sb.root()), before, "uninstall");
}

/// HUN-276 AC 2: a foreign icon file stops the install and nothing else is written.
#[test]
fn ac2_a_foreign_icon_file_is_kept_named_and_nothing_else_is_written() {
    let sb = Sb::new("launcher-foreign-icon");
    let icon = sb.home().join(ICON_FILES[0].0);
    write(&icon, "<svg>mine</svg>");
    let before = snapshot(sb.root());
    let o = launcher(&sb, "install");
    assert_eq!(o.status.code(), Some(1), "{}", text(&o.stdout));
    let said = text(&o.stderr);
    assert!(said.contains("kept"), "{said}");
    assert!(said.contains(icon.to_str().unwrap()), "{said}");
    assert_eq!(snapshot(sb.root()), before, "nothing else written");
    // uninstall removes our files and keeps the foreign one
    let sb = Sb::new("launcher-foreign-icon-uninstall");
    launcher(&sb, "install");
    let icon = sb.home().join(ICON_FILES[0].0);
    write(&icon, "<svg>mine</svg>");
    let o = launcher(&sb, "uninstall");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let left: Vec<PathBuf> = snapshot(sb.root()).into_iter().map(|(p, _)| p).collect();
    assert_eq!(left, [icon]);
}

#[test]
fn ac8_uninstall_removes_the_files_only_when_they_are_byte_identical_to_ours() {
    let sb = Sb::new("launcher-uninstall");
    launcher(&sb, "install");
    let o = launcher(&sb, "uninstall");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(!launcher_path(&sb).exists());
    assert!(snapshot(sb.root()).is_empty(), "all five files are gone");
    assert!(text(&o.stdout).contains("removed"), "{}", text(&o.stdout));
    // a file that differs by one byte stays
    write(&launcher_path(&sb), &format!("{LAUNCHER}\n"));
    let o = launcher(&sb, "uninstall");
    assert_eq!(o.status.code(), Some(0));
    assert!(launcher_path(&sb).exists());
}

#[test]
fn ac8_uninstall_with_nothing_installed_says_so_and_creates_nothing() {
    let sb = Sb::new("launcher-none");
    let o = launcher(&sb, "uninstall");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(
        text(&o.stdout).contains("not installed"),
        "{}",
        text(&o.stdout)
    );
    assert!(snapshot(sb.root()).is_empty());
}

#[test]
fn ac8_install_launcher_needs_a_home() {
    let sb = Sb::new("launcher-nohome");
    let o = Command::new(env!("CARGO_BIN_EXE_easy-note"))
        .args(["install", "launcher"])
        .env_remove("HOME")
        .current_dir(sb.root())
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(
        text(&o.stderr).contains("HOME is not set"),
        "{}",
        text(&o.stderr)
    );
}

// ---------------------------------------------------------------------------------------------
// HUN-263 AC 5: `install tray` / `uninstall tray`
// ---------------------------------------------------------------------------------------------

const TRAY: &str =
    "[Desktop Entry]\nType=Application\nName=easy-note tray\nExec=easy-note tray\nTerminal=false\n";

fn tray_path(sb: &Sb) -> PathBuf {
    sb.home().join(".config/autostart/easy-note-tray.desktop")
}

fn tray(sb: &Sb, verb: &str) -> Output {
    easy(sb, &[verb, "tray"], Some(&sb.cfg()))
}

#[cfg(feature = "tray")]
#[test]
fn ac5_install_tray_writes_exactly_one_autostart_file() {
    let sb = Sb::new("tray");
    let o = tray(&sb, "install");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let files: Vec<PathBuf> = snapshot(sb.root()).into_iter().map(|(p, _)| p).collect();
    assert_eq!(files, [tray_path(&sb)], "no other file anywhere");
    assert_eq!(std::fs::read_to_string(tray_path(&sb)).unwrap(), TRAY);
    assert!(
        text(&o.stdout).contains("easy-note-tray.desktop"),
        "{}",
        text(&o.stdout)
    );
    assert!(
        !sb.cfg().exists(),
        "the Claude Code config dir is not touched"
    );
}

#[cfg(feature = "tray")]
#[test]
fn ac5_a_second_tray_install_says_already_installed_and_writes_nothing() {
    let sb = Sb::new("tray-twice");
    tray(&sb, "install");
    let before = snapshot(sb.root());
    let o = tray(&sb, "install");
    assert_eq!(o.status.code(), Some(0));
    assert!(
        text(&o.stdout).contains("already installed"),
        "{}",
        text(&o.stdout)
    );
    assert_eq!(snapshot(sb.root()), before);
}

#[cfg(feature = "tray")]
#[test]
fn ac5_an_autostart_file_that_is_not_ours_is_kept_and_named() {
    let sb = Sb::new("tray-foreign");
    let path = tray_path(&sb);
    write(&path, "[Desktop Entry]\nName=mine\n");
    let before = snapshot(sb.root());
    for verb in ["install", "uninstall"] {
        let o = tray(&sb, verb);
        assert_eq!(o.status.code(), Some(0), "{verb}: {}", text(&o.stderr));
        let said = text(&o.stdout);
        assert!(said.contains("kept"), "{verb}: {said}");
        assert!(said.contains(path.to_str().unwrap()), "{verb}: {said}");
        assert_eq!(snapshot(sb.root()), before, "{verb}");
    }
}

#[cfg(feature = "tray")]
#[test]
fn ac5_uninstall_removes_the_tray_file_only_when_it_is_byte_identical_to_ours() {
    let sb = Sb::new("tray-uninstall");
    tray(&sb, "install");
    let o = tray(&sb, "uninstall");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(!tray_path(&sb).exists());
    assert!(text(&o.stdout).contains("removed"), "{}", text(&o.stdout));
    write(&tray_path(&sb), &format!("{TRAY}\n"));
    let o = tray(&sb, "uninstall");
    assert_eq!(o.status.code(), Some(0));
    assert!(tray_path(&sb).exists());
}

/// A default build never installs a login item that cannot show a tray (and says how to get one).
#[cfg(not(feature = "tray"))]
#[test]
fn ac5_a_default_build_refuses_install_tray_and_writes_nothing() {
    let sb = Sb::new("tray-refused");
    let o = tray(&sb, "install");
    assert_eq!(o.status.code(), Some(1));
    assert!(
        text(&o.stderr).contains("cargo install --path . --locked --features tray"),
        "{}",
        text(&o.stderr)
    );
    assert!(snapshot(sb.root()).is_empty());
}

/// Either build: with nothing installed, uninstall says so and creates nothing; a file left by
/// a tray build is still removed by a default build.
#[test]
fn ac5_uninstall_tray_works_in_every_build() {
    let sb = Sb::new("tray-none");
    let o = tray(&sb, "uninstall");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(
        text(&o.stdout).contains("not installed"),
        "{}",
        text(&o.stdout)
    );
    assert!(snapshot(sb.root()).is_empty());
    write(&tray_path(&sb), TRAY);
    let o = tray(&sb, "uninstall");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    assert!(!tray_path(&sb).exists());
}

// ---------------------------------------------------------------------------------------------
// HUN-273 AC 1, 6, 7: `install obsidian` / `uninstall obsidian`
// ---------------------------------------------------------------------------------------------

fn base_path(sb: &Sb) -> PathBuf {
    sb.root().join("notes/easy-note.base")
}

fn obsidian(sb: &Sb, verb: &str) -> Output {
    easy(sb, &[verb, "obsidian"], Some(&sb.cfg()))
}

#[test]
fn hun273_install_obsidian_writes_exactly_the_base_file_and_is_idempotent() {
    let sb = Sb::new("obsidian");
    let o = obsidian(&sb, "install");
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let files: Vec<PathBuf> = snapshot(sb.root()).into_iter().map(|(p, _)| p).collect();
    assert_eq!(
        files,
        [base_path(&sb)],
        "the notes dir was created; nothing else written"
    );
    assert!(
        text(&o.stdout).contains("easy-note.base"),
        "{}",
        text(&o.stdout)
    );
    let before = snapshot(sb.root());
    let o = obsidian(&sb, "install");
    assert_eq!(o.status.code(), Some(0));
    assert!(
        text(&o.stdout).contains("already installed"),
        "{}",
        text(&o.stdout)
    );
    assert_eq!(snapshot(sb.root()), before);
    let o = obsidian(&sb, "uninstall");
    assert_eq!(o.status.code(), Some(0));
    assert!(!base_path(&sb).exists());
    assert!(text(&o.stdout).contains("removed"), "{}", text(&o.stdout));
}

#[test]
fn hun273_a_foreign_base_file_is_kept_and_named() {
    let sb = Sb::new("obsidian-foreign");
    write(&base_path(&sb), "views: []\n");
    let o = obsidian(&sb, "install");
    assert_eq!(o.status.code(), Some(1), "{}", text(&o.stdout));
    assert!(
        text(&o.stderr).contains("easy-note.base"),
        "{}",
        text(&o.stderr)
    );
    let o = obsidian(&sb, "uninstall");
    assert_eq!(o.status.code(), Some(1));
    assert!(
        text(&o.stderr).contains("easy-note.base"),
        "{}",
        text(&o.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(base_path(&sb)).unwrap(),
        "views: []\n"
    );
}

#[test]
fn hun273_no_other_command_writes_the_base_file() {
    let sb = Sb::new("obsidian-others");
    let id_out = easy(&sb, &["add", "first", "#t"], Some(&sb.cfg()));
    assert_eq!(id_out.status.code(), Some(0), "{}", text(&id_out.stderr));
    for args in [
        vec!["ls"],
        vec!["stats"],
        vec!["doctor"],
        vec!["init"],
        vec!["install", "claude"],
        vec!["install", "launcher"],
        vec!["uninstall", "obsidian"],
    ] {
        easy(&sb, &args, Some(&sb.cfg()));
        assert!(!base_path(&sb).exists(), "{args:?} wrote the base file");
    }
}

/// AC 7: the base file is not a note and not a skipped file.
#[test]
fn hun273_the_store_ignores_the_base_file() {
    let sb = Sb::new("obsidian-store");
    easy(&sb, &["add", "first", "#t"], Some(&sb.cfg()));
    let ls = |sb: &Sb| text(&easy(sb, &["ls"], Some(&sb.cfg())).stdout);
    let stats = |sb: &Sb| text(&easy(sb, &["stats", "--json"], Some(&sb.cfg())).stdout);
    let (ls0, stats0) = (ls(&sb), stats(&sb));
    assert_eq!(obsidian(&sb, "install").status.code(), Some(0));
    assert_eq!(ls(&sb), ls0);
    assert_eq!(stats(&sb), stats0);
}
