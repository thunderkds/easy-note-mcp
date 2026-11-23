//! HUN-260: `easy-note quick`, the small input box, through the real binary (FR-022, FR-027,
//! NFR-011). Every test puts stub `zenity`, `kdialog` and `notify-send` scripts on a temp `PATH`
//! that holds nothing else (plus `git` by symlink when a repo is read), sets `DISPLAY` to a dummy
//! value and uses a temp HOME and EASY_NOTE_DIR. No test opens a real dialog or notification.

use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Datelike, Duration, Local};
use easy_note_mcp::core::format::{parse, Note};

const AWS: &str = concat!("AK", "IAIOSFODNN7EXAMPLE");
/// Timing runs hold the write lock, so no other test competes for the CPU while they run.
static QUIET: RwLock<()> = RwLock::new(());

/// A stub dialog: records its arguments and start time, prints `$STUB_TEXT`, exits `$STUB_EXIT`.
const DIALOG_STUB: &str = r#"#!/bin/sh
printf '%s\n' "$@" > "$STUB_LOG/__NAME__.args"
/usr/bin/date +%s%N > "$STUB_LOG/__NAME__.start"
echo called >> "$STUB_LOG/__NAME__.calls"
if [ -n "$STUB_TEXT" ]; then printf '%s\n' "$STUB_TEXT"; fi
exit "${STUB_EXIT:-0}"
"#;
const NOTIFY_STUB: &str = r#"#!/bin/sh
printf '%s\n' "$@" >> "$STUB_LOG/notify.args"
echo called >> "$STUB_LOG/notify.calls"
"#;

struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let sb = Sandbox {
            dir: tempfile::Builder::new()
                .prefix(&format!("easy-note-quick-{name}-"))
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
    fn home(&self) -> PathBuf {
        self.root().join("home")
    }
    fn notes(&self) -> PathBuf {
        self.root().join("notes")
    }
    fn bin(&self) -> PathBuf {
        self.root().join("bin")
    }
    fn log(&self) -> PathBuf {
        self.root().join("log")
    }
    /// Put a stub script named `name` on the stub PATH.
    fn stub(&self, name: &str, body: &str) {
        let path = self.bin().join(name);
        std::fs::write(&path, body).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    fn zenity(&self) {
        self.stub("zenity", &DIALOG_STUB.replace("__NAME__", "zenity"));
    }
    fn kdialog(&self) {
        self.stub("kdialog", &DIALOG_STUB.replace("__NAME__", "kdialog"));
    }
    fn notify(&self) {
        self.stub("notify-send", NOTIFY_STUB);
    }
    /// `git` on the stub PATH: a symlink to the real one.
    fn git(&self) {
        let real = std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|d| d.join("git"))
            .find(|p| p.is_file())
            .expect("git is installed");
        symlink(real, self.bin().join("git")).unwrap();
    }
    fn logged(&self, file: &str) -> String {
        std::fs::read_to_string(self.log().join(file)).unwrap_or_default()
    }
    fn calls(&self, tool: &str) -> usize {
        self.logged(&format!("{tool}.calls")).lines().count()
    }
}

fn command(sb: &Sandbox, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_easy-note"));
    cmd.arg("quick")
        .args(args)
        .env_clear()
        .env("PATH", sb.bin())
        .env("HOME", sb.home())
        .env("EASY_NOTE_DIR", sb.notes())
        .env("DISPLAY", ":99")
        .env("STUB_LOG", sb.log())
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .current_dir(sb.home())
        .stdin(Stdio::null());
    cmd
}

/// Run `quick` with the stub dialog answering `text`.
fn quick(sb: &Sandbox, text: &str, args: &[&str]) -> Output {
    command(sb, args).env("STUB_TEXT", text).output().unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn note_files(sb: &Sandbox) -> Vec<PathBuf> {
    let mut found = vec![];
    let mut stack = vec![sb.notes()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md")
                && !path.starts_with(sb.notes().join("inbox"))
            {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

fn parsed(path: &Path) -> Note {
    parse(&std::fs::read_to_string(path).unwrap()).expect("the note parses")
}

/// The one note `quick` wrote.
fn the_note(sb: &Sandbox, planted: usize) -> Note {
    let files = note_files(sb);
    assert_eq!(files.len(), planted + 1, "{files:?}");
    files
        .iter()
        .map(|p| parsed(p))
        .find(|n| n.get_str("source") == Some("quick"))
        .expect("a quick note")
}

fn tags_of(note: &Note) -> Vec<String> {
    match note.fields.get("tags") {
        Some(easy_note_mcp::core::format::Value::List(l)) => l.clone(),
        other => panic!("tags: {other:?}"),
    }
}

/// Write a note file `age` old by hand, as another surface would have.
fn plant(sb: &Sandbox, age: Duration, source: &str, repo: Option<&str>, task: Option<&str>) {
    plant_at(&sb.notes(), Local::now() - age, source, repo, task);
}

fn plant_at(
    notes: &Path,
    at: DateTime<Local>,
    source: &str,
    repo: Option<&str>,
    task: Option<&str>,
) {
    plant_in(notes, at, at, source, repo, task);
}

/// The note was created at `at` and sits in the month folder of `folder_at`.
fn plant_in(
    notes: &Path,
    at: DateTime<Local>,
    folder_at: DateTime<Local>,
    source: &str,
    repo: Option<&str>,
    task: Option<&str>,
) {
    let id = format!(
        "{}-{:04x}",
        at.format("%Y%m%d-%H%M%S"),
        at.timestamp_subsec_nanos() % 0xffff
    );
    let dir = notes
        .join(folder_at.format("%Y").to_string())
        .join(folder_at.format("%m").to_string());
    std::fs::create_dir_all(&dir).unwrap();
    let mut text = format!(
        "---\neasy-note: 1\nid: \"{id}\"\ncreated: \"{}\"\n",
        at.to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
    );
    if let Some(repo) = repo {
        text.push_str(&format!(
            "repo: \"{repo}\"\nremote: \"me/{repo}\"\nbranch: \"fix/{}\"\n",
            task.unwrap_or("x").to_lowercase()
        ));
    }
    if let Some(task) = task {
        text.push_str(&format!("task: \"{task}\"\n"));
    }
    text.push_str(&format!(
        "tags: []\nsource: \"{source}\"\n---\nplanted from {source}\n"
    ));
    std::fs::write(dir.join(format!("{id}.md")), text).unwrap();
}

fn git(cwd: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t.invalid"])
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
}

/// A repo `<root>/proj` on branch `fix/t120-probe` with an origin remote.
fn repo(root: &Path) -> PathBuf {
    let dir = root.join("proj");
    std::fs::create_dir_all(&dir).unwrap();
    git(&dir, &["init", "-q", "-b", "fix/t120-probe"]);
    git(&dir, &["commit", "-q", "--allow-empty", "-m", "start"]);
    git(
        &dir,
        &["remote", "add", "origin", "git@github.com:me/proj.git"],
    );
    dir
}

// ---------------------------------------------------------------------------------------------
// AC 1: text saved, cancel and empty save nothing
// ---------------------------------------------------------------------------------------------

#[test]
fn ac1_entered_text_is_saved_as_a_quick_note_with_tags_and_notified() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("ac1");
    sb.zenity();
    sb.notify();
    let o = quick(&sb, "retry bug #ff", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let note = the_note(&sb, 0);
    assert_eq!(note.get_str("source"), Some("quick"));
    assert_eq!(tags_of(&note), ["ff"]);
    assert_eq!(note.body, "retry bug #ff\n");
    // the confirmation line is printed and sent as one notification
    let line = out(&o);
    assert!(line.starts_with("✓ noted "), "{line}");
    assert_eq!(sb.calls("notify"), 1);
    let sent = sb.logged("notify.args");
    assert_eq!(sent, format!("easy-note\n{line}"));
    // the dialog got an argument list, not a shell string
    assert_eq!(
        sb.logged("zenity.args"),
        "--entry\n--title\neasy-note\n--text\nNote\n"
    );
}

#[test]
fn ac1_cancel_empty_and_blank_text_save_nothing_and_notify_nothing() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    for (name, text, exit) in [
        ("cancel", "never saved", "1"),
        ("empty", "", "0"),
        ("blank", "   \t ", "0"),
    ] {
        let sb = Sandbox::new(name);
        sb.zenity();
        sb.notify();
        let o = command(&sb, &[])
            .env("STUB_TEXT", text)
            .env("STUB_EXIT", exit)
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(0), "{name}: {}", err(&o));
        assert!(note_files(&sb).is_empty(), "{name}");
        assert!(!sb.notes().exists(), "{name}: no notes dir for no note");
        assert_eq!(out(&o), "", "{name}");
        assert_eq!(sb.calls("notify"), 0, "{name}");
        assert_eq!(sb.calls("zenity"), 1, "{name}");
    }
}

#[test]
fn a_trailing_newline_is_trimmed_and_inner_lines_are_kept() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("lines");
    sb.zenity();
    let o = quick(&sb, "first\nsecond", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(the_note(&sb, 0).body, "first\nsecond\n");
}

// ---------------------------------------------------------------------------------------------
// AC 4: tool order and display
// ---------------------------------------------------------------------------------------------

#[test]
fn ac4_kdialog_is_used_when_zenity_is_missing() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("kdialog");
    sb.kdialog();
    let o = quick(&sb, "from kdialog", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(
        sb.logged("kdialog.args"),
        "--title\neasy-note\n--inputbox\nNote\n"
    );
    assert_eq!(the_note(&sb, 0).body, "from kdialog\n");
}

#[test]
fn ac4_zenity_comes_before_kdialog() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("order");
    sb.zenity();
    sb.kdialog();
    let o = quick(&sb, "x", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(sb.calls("zenity"), 1);
    assert_eq!(sb.calls("kdialog"), 0);
}

#[test]
fn ac4_a_wayland_display_alone_is_enough_for_a_dialog() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("wayland");
    sb.zenity();
    let o = command(&sb, &[])
        .env_remove("DISPLAY")
        .env("WAYLAND_DISPLAY", "wayland-99")
        .env("STUB_TEXT", "on wayland")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(sb.calls("zenity"), 1);
}

#[test]
fn ac4_no_display_means_no_dialog_tool_and_without_a_terminal_exit_1() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("nodisplay");
    sb.zenity();
    sb.kdialog();
    sb.notify();
    let o = command(&sb, &[])
        .env_remove("DISPLAY")
        .env("STUB_TEXT", "lost?")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(
        err(&o).contains("no dialog tool and no terminal"),
        "{}",
        err(&o)
    );
    assert_eq!(sb.calls("zenity") + sb.calls("kdialog"), 0);
    assert!(note_files(&sb).is_empty());
}

#[test]
fn ac4_a_display_with_no_tool_and_no_terminal_exits_1() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("notool");
    let o = quick(&sb, "x", &[]);
    assert_eq!(o.status.code(), Some(1));
    assert!(
        err(&o).contains("no dialog tool and no terminal"),
        "{}",
        err(&o)
    );
}

// ---------------------------------------------------------------------------------------------
// AC 2: --cwd
// ---------------------------------------------------------------------------------------------

#[test]
fn ac2_cwd_gives_the_context_of_that_folder_and_wins_over_the_last_context() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("cwd");
    sb.zenity();
    sb.git();
    let proj = repo(sb.root());
    plant(
        &sb,
        Duration::minutes(5),
        "hook",
        Some("other-repo"),
        Some("T999"),
    );
    let o = quick(&sb, "in the repo", &["--cwd", proj.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let note = the_note(&sb, 1);
    assert_eq!(note.get_str("repo"), Some("proj"));
    assert_eq!(note.get_str("remote"), Some("me/proj"));
    assert_eq!(note.get_str("branch"), Some("fix/t120-probe"));
    assert_eq!(note.get_str("task"), Some("T120"));
    assert!(out(&o).contains("· proj · T120"), "{}", out(&o));
    // the box shows the chip of the folder, not of the last note
    let text = sb.logged("zenity.args");
    assert!(
        text.contains("Note — proj · T120 (start with - to drop)"),
        "{text}"
    );
    assert!(!text.contains("other-repo"), "{text}");
}

#[test]
fn ac2_cwd_outside_a_repo_gives_no_context() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("cwd-none");
    sb.zenity();
    sb.git();
    let o = quick(&sb, "nowhere", &["--cwd", sb.home().to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let note = the_note(&sb, 0);
    assert_eq!(note.get_str("repo"), None);
    assert_eq!(sb.logged("zenity.args").lines().last(), Some("Note"));
}

// ---------------------------------------------------------------------------------------------
// AC 3: last context
// ---------------------------------------------------------------------------------------------

#[test]
fn ac3_a_recent_hook_note_gives_the_context_shown_in_the_box() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("last");
    sb.zenity();
    plant(&sb, Duration::minutes(10), "hook", Some("R"), Some("T"));
    let o = quick(&sb, "x", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let note = the_note(&sb, 1);
    assert_eq!(note.get_str("repo"), Some("R"));
    assert_eq!(note.get_str("task"), Some("T"));
    assert_eq!(note.get_str("remote"), Some("me/R"));
    let args = sb.logged("zenity.args");
    assert!(args.contains("R · T"), "{args}");
    assert_eq!(
        args.lines().last(),
        Some("Note — R · T (start with - to drop)")
    );
}

#[test]
fn ac3_a_cli_note_counts_and_the_newest_cli_or_hook_note_wins() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("newest");
    sb.zenity();
    plant(&sb, Duration::minutes(40), "cli", Some("old"), Some("T1"));
    plant(&sb, Duration::minutes(20), "hook", Some("mid"), Some("T2"));
    plant(&sb, Duration::minutes(30), "cli", Some("older"), Some("T3"));
    let o = quick(&sb, "x", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(the_note(&sb, 3).get_str("repo"), Some("mid"));
}

#[test]
fn ac3_a_note_two_hours_old_or_more_gives_no_context() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("stale");
    sb.zenity();
    plant(&sb, Duration::minutes(121), "hook", Some("R"), Some("T"));
    let o = quick(&sb, "x", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let note = the_note(&sb, 1);
    assert_eq!(note.get_str("repo"), None);
    assert_eq!(note.get_str("task"), None);
    assert_eq!(sb.logged("zenity.args").lines().last(), Some("Note"));
}

#[test]
fn ac3_a_note_just_under_two_hours_old_still_counts() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("edge");
    sb.zenity();
    plant(&sb, Duration::minutes(119), "hook", Some("R"), Some("T"));
    quick(&sb, "x", &[]);
    assert_eq!(the_note(&sb, 1).get_str("repo"), Some("R"));
}

#[test]
fn ac3_quick_mcp_and_inbox_notes_are_not_a_context_source() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("sources");
    sb.zenity();
    plant(&sb, Duration::minutes(3), "quick", Some("Q"), Some("T1"));
    plant(&sb, Duration::minutes(4), "mcp", Some("M"), Some("T2"));
    let inbox = sb.notes().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::write(
        inbox.join("phone.md"),
        "---\nrepo: \"I\"\ntask: \"T3\"\n---\nfrom the phone\n",
    )
    .unwrap();
    let o = quick(&sb, "x", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let files = note_files(&sb);
    let fresh = files
        .iter()
        .map(|p| parsed(p))
        .filter(|n| n.body == "x\n")
        .collect::<Vec<_>>();
    assert_eq!(fresh.len(), 1);
    assert_eq!(fresh[0].get_str("repo"), None);
    assert_eq!(sb.logged("zenity.args").lines().last(), Some("Note"));
}

#[test]
fn ac3_the_previous_month_folder_is_read_too() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("month");
    sb.zenity();
    // A note made 30 minutes ago lies in last month's folder only just after a month turns, so
    // the file is put there on purpose: the box must read both folders.
    let last_month = Local::now().with_day(1).unwrap() - Duration::days(1);
    plant_in(
        &sb.notes(),
        Local::now() - Duration::minutes(30),
        last_month,
        "cli",
        Some("R"),
        Some("T"),
    );
    let o = quick(&sb, "x", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(the_note(&sb, 1).get_str("repo"), Some("R"));
}

#[test]
fn ac3_a_dash_drops_the_context_and_is_removed_with_one_following_space() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("dash");
    sb.zenity();
    plant(&sb, Duration::minutes(10), "hook", Some("R"), Some("T"));
    let o = quick(&sb, "- x", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let note = the_note(&sb, 1);
    assert_eq!(note.body, "x\n");
    assert_eq!(note.get_str("repo"), None);
    assert_eq!(note.get_str("task"), None);
    assert!(!out(&o).contains('R'), "{}", out(&o));
}

#[test]
fn ac3_a_dash_alone_saves_nothing() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("dash-alone");
    sb.zenity();
    plant(&sb, Duration::minutes(10), "hook", Some("R"), Some("T"));
    let o = quick(&sb, "-", &[]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(note_files(&sb).len(), 1, "only the planted note");
}

#[test]
fn ac3_only_a_lone_dash_drops_the_context_not_dash_words() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    for text in ["-x", "--x", "-1 retry"] {
        let sb = Sandbox::new("dash-word");
        sb.zenity();
        plant(&sb, Duration::minutes(10), "hook", Some("R"), Some("T"));
        let o = quick(&sb, text, &[]);
        assert_eq!(o.status.code(), Some(0), "{text}: {}", err(&o));
        let note = the_note(&sb, 1);
        assert_eq!(note.body, format!("{text}\n"));
        assert_eq!(note.get_str("repo"), Some("R"), "{text}");
    }
}

#[test]
fn ac3_with_no_context_shown_a_leading_dash_is_plain_text() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("dash-bullet");
    sb.zenity();
    let o = quick(&sb, "- a bullet", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(the_note(&sb, 0).body, "- a bullet\n");
}

#[test]
fn ac3_the_dash_drops_a_cwd_context_too() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("dash-cwd");
    sb.zenity();
    sb.git();
    let proj = repo(sb.root());
    let o = quick(&sb, "- x", &["--cwd", proj.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let note = the_note(&sb, 0);
    assert_eq!(note.body, "x\n");
    assert_eq!(note.get_str("repo"), None);
}

#[test]
fn ac3_the_box_label_cannot_carry_a_secret_or_a_control_character() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("label");
    sb.zenity();
    plant(
        &sb,
        Duration::minutes(10),
        "hook",
        Some(&format!("r {AWS}")),
        Some("T"),
    );
    quick(&sb, "x", &[]);
    let label = sb.logged("zenity.args");
    assert!(label.contains("[masked:aws_key]"), "{label}");
    assert!(!label.contains(AWS), "{label}");
}

// ---------------------------------------------------------------------------------------------
// AC 5: notification
// ---------------------------------------------------------------------------------------------

#[test]
fn ac5_without_notify_send_the_line_is_still_printed() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("nonotify");
    sb.zenity();
    let o = quick(&sb, "quiet one", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).starts_with("✓ noted "), "{}", out(&o));
    assert_eq!(note_files(&sb).len(), 1);
}

#[test]
fn a_failed_save_is_reported_on_stderr_and_in_a_notification_and_exits_1() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("failed");
    sb.zenity();
    sb.notify();
    let blocker = sb.root().join("blocker");
    std::fs::write(&blocker, "x").unwrap();
    let o = command(&sb, &[])
        .env("EASY_NOTE_DIR", blocker.join("notes"))
        .env("STUB_TEXT", "will not save")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("note not saved"), "{}", err(&o));
    assert_eq!(sb.calls("notify"), 1);
    assert!(sb.logged("notify.args").contains("note not saved"));
}

// ---------------------------------------------------------------------------------------------
// AC 10: secrets
// ---------------------------------------------------------------------------------------------

#[test]
fn ac10_a_secret_is_masked_in_the_file_the_stdout_line_and_the_notification() {
    let _q = QUIET.read().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("secret");
    sb.zenity();
    sb.notify();
    let o = quick(&sb, &format!("key {AWS} #ops"), &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let note = the_note(&sb, 0);
    assert_eq!(note.body, "key [masked:aws_key] #ops\n");
    assert!(out(&o).contains("1 secret masked: aws_key"), "{}", out(&o));
    let mut all = String::new();
    for file in ["notify.args", "zenity.args"] {
        all.push_str(&sb.logged(file));
    }
    all.push_str(&out(&o));
    all.push_str(&err(&o));
    assert!(!all.contains(AWS), "{all}");
}

// ---------------------------------------------------------------------------------------------
// AC 6: the box shows fast
// ---------------------------------------------------------------------------------------------

/// Nanoseconds since the epoch now.
fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

#[test]
fn ac6_the_dialog_starts_under_300_ms_p95_with_2000_notes() {
    let _q = QUIET.write().unwrap_or_else(|e| e.into_inner());
    let sb = Sandbox::new("timing");
    sb.zenity();
    sb.notify();
    // 2,000 older notes, newest 3 h ago: the scan must stop at the first old note.
    let base = Local::now() - Duration::hours(3);
    for i in 0..2000 {
        plant_at(
            &sb.notes(),
            base - Duration::seconds(i),
            "cli",
            Some("R"),
            Some("T"),
        );
    }
    let mut ms: Vec<f64> = vec![];
    for _ in 0..20 {
        let _ = std::fs::remove_file(sb.log().join("zenity.start"));
        let t0 = now_ns();
        let o = command(&sb, &[]).env("STUB_EXIT", "1").output().unwrap();
        assert_eq!(o.status.code(), Some(0), "{}", err(&o));
        let start: u128 = sb
            .logged("zenity.start")
            .trim()
            .parse()
            .expect("the stub started");
        ms.push((start - t0) as f64 / 1e6);
    }
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let (min, median, p95) = (
        ms[0],
        ms[ms.len() / 2],
        ms[(ms.len() * 95).div_ceil(100) - 1],
    );
    eprintln!("AC6 box start over 20 runs, 2000 notes, debug build: min {min:.1} ms, median {median:.1} ms, p95 {p95:.1} ms");
    assert!(p95 < 300.0, "p95 {p95:.1} ms");
}
