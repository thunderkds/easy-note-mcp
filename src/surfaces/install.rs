//! `install claude` and `uninstall claude` (FR-019, DDR 0003), `install launcher` and
//! `uninstall launcher` (FR-026), and `install tray` and `uninstall tray` (FR-033).
//!
//! Install adds exactly one `UserPromptSubmit` entry (command `easy-note hook-prompt`, timeout 5)
//! to the user-level `settings.json` and writes `commands/note.md`, both in the Claude Code config
//! dir: `$CLAUDE_CONFIG_DIR` when set, else `~/.claude`. Uninstall removes exactly those two.
//!
//! The edit is safe: the file is parsed first and any problem (comments, bad JSON, a shape we do
//! not expect) stops the command with nothing written. A change shows a diff and keeps a
//! timestamped backup that is never overwritten. The write is a temp file plus a rename on the
//! symlink's target, so a symlinked settings file stays a symlink. Other entries are re-printed
//! with 2-space indent and the same values and key order; nothing else is changed.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use crate::core::fsutil;

const OWN_COMMAND: &str = "easy-note hook-prompt";
const HOOK_TIMEOUT: u64 = 5;

/// The fallback slash command, used only when the hook does not see `/note` (T001, DDR 0003).
const NOTE_MD: &str = "---
description: Save a note with easy-note (fallback: the /note hook normally does this)
---
Call the `note_add` tool of the easy-note MCP server with `text` set to the text below. Then reply with one line: the note id it returns. Do nothing else.

$ARGUMENTS
";

/// The launcher: the desktop menu and dock open the dashboard; a right-click offers New note
/// (FR-026, FR-039).
const LAUNCHER_DESKTOP: &str = "[Desktop Entry]
Type=Application
Name=easy-note
Comment=Your notes, one press away
Exec=easy-note dashboard
Icon=easy-note
Terminal=false
Categories=Utility;
Actions=new-note;

[Desktop Action new-note]
Name=New note
Exec=easy-note quick
";

/// The launcher text before the icons (HUN-260). It is ours only to upgrade or remove it.
const LAUNCHER_DESKTOP_V1: &str = "[Desktop Entry]
Type=Application
Name=easy-note
Comment=Save a one-line note
Exec=easy-note quick
Terminal=false
Categories=Utility;
";

/// The icon files `install launcher` owns, below `$HOME` (FR-039).
const LAUNCHER_ICONS: [(&str, &str); 4] = [
    (
        ".local/share/icons/hicolor/scalable/apps/easy-note.svg",
        include_str!("../../assets/icons/easy-note.svg"),
    ),
    (
        ".local/share/icons/hicolor/16x16/apps/easy-note.svg",
        include_str!("../../assets/icons/easy-note-16.svg"),
    ),
    (
        ".local/share/icons/hicolor/symbolic/apps/easy-note-symbolic.svg",
        include_str!("../../assets/icons/easy-note-symbolic.svg"),
    ),
    (
        ".local/share/icons/hicolor/symbolic/apps/easy-note-saved-symbolic.svg",
        include_str!("../../assets/icons/easy-note-saved-symbolic.svg"),
    ),
];

/// The tray's login item: one file, so the tray starts with the session (FR-033).
const TRAY_DESKTOP: &str = "[Desktop Entry]
Type=Application
Name=easy-note tray
Exec=easy-note tray
Terminal=false
";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Install,
    Uninstall,
}

/// Run the action on the Claude Code config dir. `Ok` is the text for stdout; `Err` is the
/// message for stderr (exit 1).
pub fn claude(action: Action) -> Result<String, String> {
    let dir = config_dir()?;
    let settings = dir.join("settings.json");
    let note_md = dir.join("commands/note.md");

    // Everything is read and checked before anything is written.
    let state = Settings::read(&settings)?;
    let mut value = state.value.clone();
    let changed =
        edit(&mut value, action).map_err(|why| format!("{}: {why}", settings.display()))?;
    let md = plan_own_file(&note_md, action, NOTE_MD)?;

    let mut out = String::new();
    if changed {
        let new_text = format!(
            "{}\n",
            serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
        );
        out.push_str(&unified_diff(
            state.text.as_deref().unwrap_or(""),
            &new_text,
            &state
                .text
                .as_ref()
                .map_or("/dev/null".to_string(), |_| settings.display().to_string()),
            &settings.display().to_string(),
        ));
        if let Some(old) = &state.text {
            let backup = write_backup(&settings, old, state.mode)?;
            out.push_str(&format!("backup: {}\n", backup.display()));
        }
        write_atomic(
            &state.target,
            new_text.as_bytes(),
            state.mode.unwrap_or(0o644),
        )
        .map_err(|e| format!("cannot write {}: {e}", state.target.display()))?;
        out.push_str(&match action {
            Action::Install => format!("✓ added the easy-note hook to {}\n", settings.display()),
            Action::Uninstall => {
                format!("✓ removed the easy-note hook from {}\n", settings.display())
            }
        });
    }
    out.push_str(&md.apply(&note_md, NOTE_MD)?);
    if out.is_empty() {
        out = match action {
            Action::Install => format!("already installed ({})\n", dir.display()),
            Action::Uninstall => format!("not installed ({})\n", dir.display()),
        };
    }
    Ok(out)
}

/// Run the action on the launcher and its icons: `$HOME/.local/share/applications/easy-note.desktop`
/// and the four icon files. A file there that is not byte-identical to ours is never changed or
/// removed. An install checks all five first: one foreign file means nothing is written. The old
/// launcher text counts as ours and is replaced.
pub fn launcher(action: Action) -> Result<String, String> {
    let home = home_dir()?;
    let mut files: Vec<(PathBuf, &str, &[&str])> = vec![(
        home.join(".local/share/applications/easy-note.desktop"),
        LAUNCHER_DESKTOP,
        &[LAUNCHER_DESKTOP_V1],
    )];
    files.extend(
        LAUNCHER_ICONS
            .iter()
            .map(|(rel, svg)| (home.join(rel), *svg, &[] as &[&str])),
    );
    let mut plans = vec![];
    for (path, ours, old) in &files {
        plans.push(plan_own_file_from(path, action, ours, old)?);
    }
    if action == Action::Install {
        let kept: Vec<String> = files
            .iter()
            .zip(&plans)
            .filter(|(_, plan)| matches!(plan, OwnFile::Keep))
            .map(|((path, ..), _)| path.display().to_string())
            .collect();
        if !kept.is_empty() {
            return Err(format!(
                "kept {}: not the file easy-note writes. Nothing was written\n",
                kept.join(", ")
            ));
        }
    }
    let mut out = String::new();
    for ((path, ours, _), plan) in files.iter().zip(&plans) {
        out.push_str(&plan.apply(path, ours)?);
    }
    Ok(if out.is_empty() {
        let name = files[0].0.display();
        match action {
            Action::Install => format!("already installed ({name})\n"),
            Action::Uninstall => format!("not installed ({name})\n"),
        }
    } else {
        out
    })
}

/// Run the action on `$HOME/.config/autostart/easy-note-tray.desktop`, like `launcher`. A build
/// without the tray feature does not install it: the login item would fail at every login.
pub fn tray(action: Action) -> Result<String, String> {
    if action == Action::Install && !cfg!(feature = "tray") {
        return Err(super::tray::NEEDS_FEATURE.to_string());
    }
    own_desktop_file(
        ".config/autostart/easy-note-tray.desktop",
        TRAY_DESKTOP,
        action,
    )
}

/// `content` at `$HOME/<rel>`, installed or removed only when the file is missing or ours.
fn own_desktop_file(rel: &str, content: &str, action: Action) -> Result<String, String> {
    own_file(&home_dir()?.join(rel), content, action)
}

/// `content` at `path`, installed or removed only when the file is missing or ours.
fn own_file(path: &Path, content: &str, action: Action) -> Result<String, String> {
    let out = plan_own_file(path, action, content)?.apply(path, content)?;
    Ok(if out.is_empty() {
        match action {
            Action::Install => format!("already installed ({})\n", path.display()),
            Action::Uninstall => format!("not installed ({})\n", path.display()),
        }
    } else {
        out
    })
}

/// Run the action on `<notes dir>/easy-note.base`, like `launcher`, but a foreign file is exit 1. The notes dir is created if
/// it is missing, as the first write does.
pub fn obsidian(action: Action) -> Result<String, String> {
    let dir = crate::core::config::notes_dir().map_err(|e| e.to_string())?;
    let path = std::path::absolute(dir.join(super::dashboard::BASE_NAME))
        .map_err(|e| format!("{}: {e}", dir.display()))?;
    let out = own_file(&path, super::dashboard::BASE_FILE, action)?;
    // A file that is not ours is an error here (exit 1), unlike the launcher (AC 1).
    match out.strip_suffix('\n').filter(|_| out.starts_with("kept ")) {
        Some(kept) => Err(kept.to_string()),
        None => Ok(out),
    }
}

fn home_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME")
        .filter(|v| !v.is_empty())
        .ok_or("HOME is not set")?;
    Ok(PathBuf::from(home))
}

/// `$CLAUDE_CONFIG_DIR` when set and not empty, else `$HOME/.claude` (DDR 0003 item 7).
fn config_dir() -> Result<PathBuf, String> {
    let from_env = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    let dir = match from_env("CLAUDE_CONFIG_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(from_env("HOME").ok_or("HOME is not set")?).join(".claude"),
    };
    std::path::absolute(&dir).map_err(|e| format!("{}: {e}", dir.display()))
}

/// The settings file as read: its text (None when missing), its parsed value, the path to write
/// (the symlink's target, so a link stays a link) and its mode.
struct Settings {
    text: Option<String>,
    value: Value,
    target: PathBuf,
    mode: Option<u32>,
}

impl Settings {
    fn read(path: &Path) -> Result<Settings, String> {
        let name = path.display();
        let is_link = fs::symlink_metadata(path).is_ok_and(|m| m.is_symlink());
        let target = if is_link {
            fs::canonicalize(path)
                .map_err(|e| format!("{name}: the symlink points nowhere ({e})"))?
        } else {
            path.to_path_buf()
        };
        let text = match fs::read_to_string(&target) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Settings {
                    text: None,
                    value: json!({}),
                    target,
                    mode: None,
                })
            }
            Err(e) => return Err(format!("{name}: cannot read the file ({e})")),
        };
        let value: Value = serde_json::from_str(&text).map_err(|e| {
            format!("{name}: not plain JSON ({e}); comments are not allowed. Nothing was changed")
        })?;
        let mode = fs::metadata(&target)
            .ok()
            .map(|m| m.permissions().mode() & 0o777);
        Ok(Settings {
            text: Some(text),
            value,
            target,
            mode,
        })
    }
}

fn is_own(hook: &Value) -> bool {
    hook.get("command").and_then(Value::as_str) == Some(OWN_COMMAND)
}

/// Apply the action to the parsed settings. `Ok(true)` when the value changed.
fn edit(root: &mut Value, action: Action) -> Result<bool, String> {
    let Some(root) = root.as_object_mut() else {
        return Err("the top-level value is not an object".to_string());
    };
    match action {
        Action::Install => install_entry(root),
        Action::Uninstall => remove_entry(root),
    }
}

/// `hooks.UserPromptSubmit`, if both exist; an error when either has the wrong type.
fn prompt_hooks(root: &mut Map<String, Value>) -> Result<Option<&mut Vec<Value>>, String> {
    let Some(hooks) = root.get_mut("hooks") else {
        return Ok(None);
    };
    let Some(hooks) = hooks.as_object_mut() else {
        return Err("\"hooks\" is not an object".to_string());
    };
    match hooks.get_mut("UserPromptSubmit") {
        None => Ok(None),
        Some(Value::Array(list)) => Ok(Some(list)),
        Some(_) => Err("\"hooks.UserPromptSubmit\" is not an array".to_string()),
    }
}

fn install_entry(root: &mut Map<String, Value>) -> Result<bool, String> {
    let own =
        json!({"hooks": [{"type": "command", "command": OWN_COMMAND, "timeout": HOOK_TIMEOUT}]});
    if let Some(list) = prompt_hooks(root)? {
        let present = list.iter().any(|entry| {
            entry
                .get("hooks")
                .and_then(Value::as_array)
                .is_some_and(|hooks| hooks.iter().any(is_own))
        });
        if present {
            return Ok(false);
        }
        list.push(own);
        return Ok(true);
    }
    let hooks = root.entry("hooks").or_insert_with(|| json!({}));
    hooks
        .as_object_mut()
        .expect("prompt_hooks checked that hooks is an object")
        .insert("UserPromptSubmit".to_string(), json!([own]));
    Ok(true)
}

/// Take our command out of every entry that holds it. An entry left with no command, an event
/// left with no entry and `hooks` left with no event are dropped too, because the removal emptied
/// them; a list that was already empty is left as it was.
fn remove_entry(root: &mut Map<String, Value>) -> Result<bool, String> {
    let Some(list) = prompt_hooks(root)? else {
        return Ok(false);
    };
    let mut removed = false;
    let mut emptied = vec![false; list.len()];
    for (entry, emptied) in list.iter_mut().zip(emptied.iter_mut()) {
        if let Some(hooks) = entry.get_mut("hooks").and_then(Value::as_array_mut) {
            let before = hooks.len();
            hooks.retain(|hook| !is_own(hook));
            if hooks.len() != before {
                removed = true;
                *emptied = hooks.is_empty();
            }
        }
    }
    if !removed {
        return Ok(false);
    }
    let mut at = 0;
    list.retain(|_| {
        at += 1;
        !emptied[at - 1]
    });
    let hooks = root
        .get_mut("hooks")
        .and_then(Value::as_object_mut)
        .expect("prompt_hooks found the hooks object");
    if hooks
        .get("UserPromptSubmit")
        .and_then(Value::as_array)
        .is_some_and(Vec::is_empty)
    {
        hooks.shift_remove("UserPromptSubmit");
    }
    if hooks.is_empty() {
        root.shift_remove("hooks");
    }
    Ok(true)
}

/// What to do with a file that only easy-note writes (`commands/note.md`, the launcher, the tray
/// login item).
enum OwnFile {
    Nothing,
    Write,
    /// An older text of ours, replaced by the new one; holds the old text for the diff.
    Replace(String),
    Remove,
    /// A file that is not ours: left alone.
    Keep,
}

fn plan_own_file(path: &Path, action: Action, ours: &str) -> Result<OwnFile, String> {
    plan_own_file_from(path, action, ours, &[])
}

/// Like `plan_own_file`; a file that equals one of `previous` is also ours.
fn plan_own_file_from(
    path: &Path,
    action: Action,
    ours: &str,
    previous: &[&str],
) -> Result<OwnFile, String> {
    let current = match fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(format!("{}: cannot read the file ({e})", path.display())),
    };
    let old = previous
        .iter()
        .find(|old| current.as_deref() == Some(old.as_bytes()));
    if let (Action::Install, Some(old)) = (action, old) {
        return Ok(OwnFile::Replace((*old).to_string()));
    }
    let ours = old.is_some() || current.as_deref() == Some(ours.as_bytes());
    Ok(match (action, current.is_some(), ours) {
        (Action::Install, false, _) => OwnFile::Write,
        (Action::Install, true, true) | (Action::Uninstall, false, _) => OwnFile::Nothing,
        (Action::Install, true, false) | (Action::Uninstall, true, false) => OwnFile::Keep,
        (Action::Uninstall, true, true) => OwnFile::Remove,
    })
}

impl OwnFile {
    fn apply(&self, path: &Path, content: &str) -> Result<String, String> {
        match self {
            OwnFile::Nothing => Ok(String::new()),
            OwnFile::Keep => Ok(format!(
                "kept {}: it is not the file easy-note writes\n",
                path.display()
            )),
            OwnFile::Write => {
                fs::create_dir_all(path.parent().expect("an own file has a parent"))
                    .and_then(|()| write_atomic(path, content.as_bytes(), 0o644))
                    .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
                Ok(format!("✓ wrote {}\n", path.display()))
            }
            OwnFile::Replace(old) => {
                write_atomic(path, content.as_bytes(), 0o644)
                    .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
                let shown = path.display().to_string();
                Ok(format!(
                    "{}✓ updated {shown}\n",
                    unified_diff(old, content, &shown, &shown)
                ))
            }
            OwnFile::Remove => {
                fs::remove_file(path)
                    .map_err(|e| format!("cannot remove {}: {e}", path.display()))?;
                Ok(format!("✓ removed {}\n", path.display()))
            }
        }
    }
}

/// Write `data` to a temp file in the target's folder, then rename it over the target.
fn write_atomic(target: &Path, data: &[u8], mode: u32) -> std::io::Result<()> {
    let parent = target.parent().expect("a file path has a parent");
    fs::create_dir_all(parent)?;
    let (mut file, tmp) = fsutil::create_temp(parent)?;
    let result = fsutil::write_all_sync(&mut file, data)
        .and_then(|()| fs::set_permissions(&tmp, fs::Permissions::from_mode(mode)))
        .and_then(|()| fs::rename(&tmp, target));
    fsutil::unlink_quiet(&tmp);
    result
}

/// `settings.json.easy-note-<YYYYMMDD-HHMMSS>.bak` beside the settings file. A name that is
/// taken (two runs in one second) gets `-2`, `-3` ... before `.bak`; an old backup is never
/// overwritten.
fn write_backup(settings: &Path, old: &str, mode: Option<u32>) -> Result<PathBuf, String> {
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let dir = settings.parent().expect("settings has a parent");
    for n in 1..1000 {
        let suffix = if n == 1 {
            String::new()
        } else {
            format!("-{n}")
        };
        let path = dir.join(format!("settings.json.easy-note-{stamp}{suffix}.bak"));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode.unwrap_or(0o600))
            .open(&path)
        {
            Ok(mut file) => {
                return file
                    .write_all(old.as_bytes())
                    .and_then(|()| file.sync_all())
                    .map(|()| path.clone())
                    .map_err(|e| format!("cannot write the backup {}: {e}", path.display()));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("cannot write the backup {}: {e}", path.display())),
        }
    }
    Err("cannot find a free backup name".to_string())
}

/// A unified-style diff with one hunk and 3 lines of context. The files are small, so the middle
/// (after the common head and tail) is matched with a plain longest-common-subsequence table.
fn unified_diff(old: &str, new: &str, old_name: &str, new_name: &str) -> String {
    const CONTEXT: usize = 3;
    const MAX_CELLS: usize = 4_000_000;
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let head = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let tail = a[head..]
        .iter()
        .rev()
        .zip(b[head..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (mid_a, mid_b) = (&a[head..a.len() - tail], &b[head..b.len() - tail]);

    let mut ops: Vec<(char, &str)> = vec![];
    if mid_a.len() * mid_b.len() > MAX_CELLS {
        ops.extend(mid_a.iter().map(|l| ('-', *l)));
        ops.extend(mid_b.iter().map(|l| ('+', *l)));
    } else {
        let width = mid_b.len() + 1;
        let mut lcs = vec![0u32; (mid_a.len() + 1) * width];
        for i in (0..mid_a.len()).rev() {
            for j in (0..mid_b.len()).rev() {
                lcs[i * width + j] = if mid_a[i] == mid_b[j] {
                    lcs[(i + 1) * width + j + 1] + 1
                } else {
                    lcs[(i + 1) * width + j].max(lcs[i * width + j + 1])
                };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < mid_a.len() || j < mid_b.len() {
            if i < mid_a.len() && j < mid_b.len() && mid_a[i] == mid_b[j] {
                ops.push((' ', mid_a[i]));
                i += 1;
                j += 1;
            } else if j == mid_b.len()
                || (i < mid_a.len() && lcs[(i + 1) * width + j] >= lcs[i * width + j + 1])
            {
                ops.push(('-', mid_a[i]));
                i += 1;
            } else {
                ops.push(('+', mid_b[j]));
                j += 1;
            }
        }
    }

    let before = head.min(CONTEXT);
    let after = tail.min(CONTEXT);
    let start = head - before;
    let old_len = before + mid_a.len() + after;
    let new_len = before + mid_b.len() + after;
    let mut out = format!(
        "--- {old_name}\n+++ {new_name}\n@@ -{},{old_len} +{},{new_len} @@\n",
        start + 1,
        start + 1
    );
    for line in &a[start..head] {
        out.push_str(&format!(" {line}\n"));
    }
    for (mark, line) in ops {
        out.push_str(&format!("{mark}{line}\n"));
    }
    for line in &a[a.len() - tail..a.len() - tail + after] {
        out.push_str(&format!(" {line}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_of_an_appended_entry() {
        let old = "{\n  \"a\": [\n    1\n  ]\n}\n";
        let new = "{\n  \"a\": [\n    1,\n    2\n  ]\n}\n";
        assert_eq!(
            unified_diff(old, new, "f", "f"),
            "--- f\n+++ f\n@@ -1,5 +1,6 @@\n {\n   \"a\": [\n-    1\n+    1,\n+    2\n   ]\n }\n"
        );
    }

    #[test]
    fn diff_of_a_new_file_is_all_additions() {
        assert_eq!(
            unified_diff("", "x\ny\n", "/dev/null", "f"),
            "--- /dev/null\n+++ f\n@@ -1,0 +1,2 @@\n+x\n+y\n"
        );
    }

    #[test]
    fn a_taken_backup_name_is_never_reused() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        let first = write_backup(&settings, "one", None).unwrap();
        let second = write_backup(&settings, "two", None).unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read_to_string(&first).unwrap(), "one");
        assert_eq!(fs::read_to_string(&second).unwrap(), "two");
    }

    /// A small well-formedness check: every tag closes in order. No XML crate is needed.
    fn well_formed(xml: &str) -> bool {
        let mut stack: Vec<&str> = vec![];
        let mut rest = xml;
        while let Some(start) = rest.find('<') {
            let Some(end) = rest[start..].find('>') else {
                return false;
            };
            let tag = &rest[start + 1..start + end];
            rest = &rest[start + end + 1..];
            if tag.ends_with('/') {
                continue;
            }
            let name = tag
                .trim_start_matches('/')
                .split_whitespace()
                .next()
                .unwrap_or("");
            if tag.starts_with('/') {
                if stack.pop() != Some(name) {
                    return false;
                }
            } else {
                stack.push(name);
            }
        }
        stack.is_empty()
    }

    #[test]
    fn each_icon_is_an_svg_with_a_view_box_and_the_design_colours() {
        for (rel, svg) in LAUNCHER_ICONS {
            assert!(svg.starts_with("<svg "), "{rel}");
            assert!(svg.contains("viewBox=\""), "{rel}");
            assert!(well_formed(svg), "{rel}");
            if rel.contains("symbolic") {
                assert!(svg.contains("#bebebe") && !svg.contains("#3CF5A8"), "{rel}");
            } else {
                assert!(svg.contains("#181B1A") && svg.contains("#3CF5A8"), "{rel}");
            }
        }
        assert!(
            !LAUNCHER_ICONS[1].1.contains("M24 32"),
            "16 px has no text lines"
        );
    }

    #[test]
    fn remove_keeps_an_empty_list_that_was_already_empty() {
        let mut v: Value = serde_json::from_str(r#"{"hooks":{"UserPromptSubmit":[]}}"#).unwrap();
        assert!(!edit(&mut v, Action::Uninstall).unwrap());
        assert_eq!(v, json!({"hooks":{"UserPromptSubmit":[]}}));
    }
}
