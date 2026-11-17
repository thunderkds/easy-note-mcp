//! `quick`: one key press or one click opens a one-line box; Enter saves the note (FR-022).
//!
//! The box is a native tool, never a window of ours: `zenity`, else `kdialog`, each only when
//! `$DISPLAY` or `$WAYLAND_DISPLAY` is set; else a prompt on the terminal when stdin is one; else
//! exit 1. Arguments go as an argv list, never through a shell. Exit 0 with text saves it; any other
//! exit is a cancel. A save prints the confirmation line and sends it with `notify-send` when found.
//!
//! The context is the `--cwd` folder's, else that of the newest `cli` or `hook` note made less than
//! two hours ago (FR-027). It is read before the box shows, from the notes of this and the last
//! month only, newest first, and the scan stops at the first note that is too old (NFR-011). It is
//! shown in the box text, and a leading `-` in the answer drops it.

use std::io::{self, BufRead, IsTerminal, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use chrono::{DateTime, Datelike, Local};
use wait_timeout::ChildExt;

use super::cli::{confirmation, safe};
use crate::core::context::{self, Context};
use crate::core::format::parse;
use crate::core::store::Store;
use crate::core::tags;

/// A note older than this is not a context source (FR-027).
const CONTEXT_WINDOW_SECS: i64 = 2 * 60 * 60;
/// Wait this long for `notify-send`; a stuck notification daemon must not hold the command.
const NOTIFY_TIMEOUT: Duration = Duration::from_secs(2);

/// Run `quick`. `Err` is the message for stderr (exit 1); a cancel or an empty answer is `Ok`.
pub fn run(cwd: Option<&Path>) -> Result<(), String> {
    let store = Store::open().map_err(|e| e.to_string())?;
    let context = match cwd.filter(|c| !c.as_os_str().is_empty()) {
        Some(dir) => context::capture(None, Some(dir), &[]),
        None => last_context(&store.root, Local::now()).unwrap_or_default(),
    };
    let chip = chip(&context);
    let label = match &chip {
        Some(chip) => format!("Note — {} (start with - to drop)", safe(chip)),
        None => "Note".to_string(),
    };
    let Some(answer) = ask(&label)? else {
        return Ok(());
    };
    let mut text = answer.trim();
    let mut context = context;
    if chip.is_some() {
        if let Some(rest) = drop_marker(text) {
            text = rest;
            context = Context::default();
        }
    }
    if text.is_empty() {
        return Ok(());
    }
    let saved = store
        .add(text, "quick", &tags::extract(text), &context.fields())
        .map_err(|e| e.to_string());
    match saved {
        Ok(added) => {
            let line = confirmation(
                "noted",
                &added.id,
                &[context.repo.as_deref(), context.task.as_deref()],
                &added.masked,
            );
            let mut out = io::stdout().lock();
            let _ = writeln!(out, "{line}");
            let _ = out.flush();
            notify(&line);
            Ok(())
        }
        Err(message) => {
            // The person looks at the screen, not at stderr, so the failure is a notification too.
            let warning = format!("easy-note: {} — note not saved", safe(&message));
            notify(&warning);
            Err(format!("{} — note not saved", safe(&message)))
        }
    }
}

/// `<repo> · <task>`, or `None` when the context holds neither.
fn chip(context: &Context) -> Option<String> {
    let parts: Vec<&str> = [context.repo.as_deref(), context.task.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    (!parts.is_empty()).then(|| parts.join(" · "))
}

/// The text after a leading `-` that stands alone (`-`, `- x`); `None` for `-x`, `--x` and the rest.
/// One white-space character after the dash goes with it.
fn drop_marker(text: &str) -> Option<&str> {
    let rest = text.strip_prefix('-')?;
    let mut chars = rest.chars();
    match chars.next() {
        None => Some(""),
        Some(c) if c.is_whitespace() => Some(chars.as_str().trim()),
        Some(_) => None,
    }
}

/// The context of the newest `cli` or `hook` note made less than two hours before `now`.
/// Notes of other sources and the inbox are not read as a source. Notes are read newest first
/// (the id starts with the local time), so the first one that is too old ends the search.
fn last_context(root: &Path, now: DateTime<Local>) -> Option<Context> {
    let (year, month) = (now.year(), now.month());
    let before = if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    };
    for (year, month) in [(year, month), before] {
        let dir = root.join(format!("{year:04}")).join(format!("{month:02}"));
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with(".md") && !n.starts_with('.'))
            })
            .collect();
        paths.sort_by(|a, b| b.cmp(a));
        for path in paths {
            let Some(note) = std::fs::read_to_string(&path)
                .ok()
                .and_then(|raw| parse(&raw).ok())
            else {
                continue;
            };
            let Some(created) = note
                .get_str("created")
                .and_then(|c| DateTime::parse_from_rfc3339(c).ok())
            else {
                continue;
            };
            if now.signed_duration_since(created).num_seconds() >= CONTEXT_WINDOW_SECS {
                return None;
            }
            if matches!(note.get_str("source"), Some("cli" | "hook")) {
                let field = |key: &str| note.get_str(key).map(str::to_string);
                return Some(Context {
                    repo: field("repo"),
                    remote: field("remote"),
                    branch: field("branch"),
                    task: field("task"),
                    // A commit belongs to the moment of the old note, not to this one.
                    commit: None,
                });
            }
        }
    }
    None
}

/// Show the box and return the answer; `None` when the person cancelled.
fn ask(label: &str) -> Result<Option<String>, String> {
    if has_display() {
        if let Some(tool) = find_in_path("zenity") {
            return dialog(&tool, &["--entry", "--title", "easy-note", "--text", label]);
        }
        if let Some(tool) = find_in_path("kdialog") {
            return dialog(&tool, &["--title", "easy-note", "--inputbox", label]);
        }
    }
    let stdin = io::stdin();
    if stdin.is_terminal() {
        return prompt(stdin.lock(), io::stderr(), label).map_err(|e| e.to_string());
    }
    Err("no dialog tool and no terminal".to_string())
}

fn has_display() -> bool {
    ["DISPLAY", "WAYLAND_DISPLAY"]
        .iter()
        .any(|name| std::env::var_os(name).is_some_and(|v| !v.is_empty()))
}

/// The first executable file called `name` in `$PATH`.
fn find_in_path(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .filter(|dir| !dir.as_os_str().is_empty())
        .map(|dir| dir.join(name))
        .find(|path| {
            path.metadata()
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
}

/// Run one dialog tool. Exit 0 gives its output; any other exit is a cancel.
fn dialog(tool: &Path, args: &[&str]) -> Result<Option<String>, String> {
    let out = Command::new(tool)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| format!("cannot run {}: {e}", tool.display()))?;
    if !out.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&out.stdout).into_owned()))
}

/// The terminal tier: show the label, read one line; end of input is a cancel.
fn prompt(
    mut input: impl BufRead,
    mut shown: impl Write,
    label: &str,
) -> io::Result<Option<String>> {
    write!(shown, "{label}: ")?;
    shown.flush()?;
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line))
}

/// `notify-send "easy-note" <text>`, when found. A failure is never an error: the line is printed.
fn notify(text: &str) {
    let Some(tool) = find_in_path("notify-send") else {
        return;
    };
    let Ok(mut child) = Command::new(tool)
        .args(["easy-note", text])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return;
    };
    if !matches!(child.wait_timeout(NOTIFY_TIMEOUT), Ok(Some(_))) {
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// The text `hotkey` prints for `$XDG_CURRENT_DESKTOP` (FR-025). It only prints; the person runs
/// the commands.
pub fn hotkey(desktop: &str) -> String {
    let tokens: Vec<String> = desktop.split(':').map(str::to_lowercase).collect();
    let is = |name: &str| tokens.iter().any(|t| t == name);
    let steps = if is("gnome") {
        GNOME
    } else if is("kde") {
        KDE
    } else {
        OTHER
    };
    format!("{steps}{PATH_NOTE}\n")
}

const PATH_NOTE: &str = "If your desktop does not find `easy-note` (for example it sits in ~/.cargo/bin), use its full path from `command -v easy-note` in place of `easy-note`.";

const GNOME: &str = "Bind Ctrl+Alt+N to `easy-note quick` on GNOME. Run these commands yourself; easy-note changes nothing.

1. See your custom shortcuts. Step 2 replaces this list, so keep what it holds:
gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings

2. Add ours (add the path to the list from step 1 if it was not empty):
gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings \"['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/easy-note/']\"
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/easy-note/ name 'easy-note quick'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/easy-note/ command 'easy-note quick'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/easy-note/ binding '<Control><Alt>n'

";

const KDE: &str =
    "Bind a key to `easy-note quick` on KDE. Do these steps yourself; easy-note changes nothing.

1. Open System Settings, then Keyboard, then Shortcuts.
2. Choose Add New, then Command or Script.
3. Enter the command: easy-note quick
4. Click the new entry, press the key you want (for example Ctrl+Alt+N) and apply.

";

const OTHER: &str = "Bind a key to the command `easy-note quick` in your desktop or window manager. easy-note changes nothing.

Examples:
  sway or i3 config:  bindsym $mod+n exec easy-note quick
  any desktop:        use its keyboard-shortcut settings and enter the command: easy-note quick

";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_marker_takes_a_lone_dash_only() {
        assert_eq!(drop_marker("-"), Some(""));
        assert_eq!(drop_marker("- x"), Some("x"));
        assert_eq!(drop_marker("-\tx"), Some("x"));
        assert_eq!(drop_marker("-  x"), Some("x"));
        assert_eq!(drop_marker("-x"), None);
        assert_eq!(drop_marker("--x"), None);
        assert_eq!(drop_marker("x"), None);
        assert_eq!(drop_marker(""), None);
    }

    #[test]
    fn the_terminal_prompt_reads_one_line_and_end_of_input_cancels() {
        let mut shown = vec![];
        let got = prompt("hello\nmore\n".as_bytes(), &mut shown, "Note").unwrap();
        assert_eq!(got.as_deref(), Some("hello\n"));
        assert_eq!(String::from_utf8(shown).unwrap(), "Note: ");
        assert_eq!(prompt("".as_bytes(), vec![], "Note").unwrap(), None);
    }

    #[test]
    fn the_chip_names_repo_and_task_when_there_are_any() {
        let full = Context {
            repo: Some("R".into()),
            task: Some("T".into()),
            ..Context::default()
        };
        assert_eq!(chip(&full).as_deref(), Some("R · T"));
        let repo_only = Context {
            repo: Some("R".into()),
            ..Context::default()
        };
        assert_eq!(chip(&repo_only).as_deref(), Some("R"));
        assert_eq!(chip(&Context::default()), None);
    }
}
