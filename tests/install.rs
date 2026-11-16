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
}

impl Sb {
    fn new(name: &str) -> Sb {
        let sb = Sb {
            dir: tempfile::Builder::new()
                .prefix(&format!("easy-note-install-{name}-"))
                .tempdir()
                .unwrap(),
        };
        std::fs::create_dir_all(sb.home()).unwrap();
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
    cmd.args(args)
        .env("HOME", sb.home())
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
