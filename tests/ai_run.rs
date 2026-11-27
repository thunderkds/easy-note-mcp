//! HUN-295: `easy-note ai run` (FR-047, FR-048, FR-049, NFR-012, NFR-014, NFR-015): caps and
//! batches, the run lock, the state file and back-off, `--dry-run`, the hold after an AI undo and
//! the write lock.
//!
//! Every test runs the real binary with a temp HOME, XDG_CONFIG_HOME and EASY_NOTE_DIR, and a fake
//! `claude` shell script (an absolute `command`). The fake counts its calls, answers every note id
//! it was sent, and waits while a `gate` file exists. No test starts the real `claude` or opens the
//! real `~/easy-notes/`, `~/.config/` or `~/.local/share/`.

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

use chrono::DateTime;
use easy_note_mcp::core::ai::{self, Input, Organized};
use easy_note_mcp::core::overrides;
use easy_note_mcp::core::store::Store;
use serde_json::Value;

const FAKE: &str = r#"#!/bin/sh
d=$(dirname "$0")
cat > "$d/stdin.$$"
echo call >> "$d/calls"
n=$(wc -l < "$d/calls")
if [ -f "$d/gate" ]; then
  : > "$d/waiting"
  i=0
  while [ -f "$d/gate" ] && [ "$i" -lt 200 ]; do sleep 0.05; i=$((i+1)); done
fi
case "$(cat "$d/mode" 2>/dev/null)" in
  fail) exit 1;;
  fail_after_first) [ "$n" -gt 1 ] && exit 1;;
esac
if grep -q '^<notes>' "$d/stdin.$$"; then
  items=$(grep '^{"id":' "$d/stdin.$$" | sed 's/^{"id":"\([^"]*\)".*/{"id":"\1","title":"model title","kind":"todo","summary":"model summary","project":"model project","topic":"model topic","tags_suggested":[]}/' | paste -sd, -)
  printf '{"is_error":false,"structured_output":{"items":[%s]}}' "$items"
else
  printf '{"is_error":false,"structured_output":{"title":"one title","kind":"idea","summary":"one summary","tags_suggested":[]}}'
fi
"#;

struct Sb {
    dir: tempfile::TempDir,
}

impl Sb {
    /// A sandbox with AI on, plus `extra` lines in `[ai]`.
    fn new(extra: &str) -> Sb {
        let dir = tempfile::Builder::new()
            .prefix("easy-note-ai-run-")
            .tempdir()
            .unwrap();
        for d in ["home/.config/easy-note", "bin", "notes"] {
            std::fs::create_dir_all(dir.path().join(d)).unwrap();
        }
        let sb = Sb { dir };
        let fake = sb.path("bin/claude");
        std::fs::write(&fake, FAKE).unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        sb.config(extra);
        sb
    }
    fn config(&self, extra: &str) {
        let command = self.path("bin/claude");
        self.raw_config(&format!(
            "[ai]\nenrich = \"claude\"\ncommand = \"{}\"\nmodel = \"haiku\"\n{extra}\n",
            command.display()
        ));
    }
    fn raw_config(&self, text: &str) {
        std::fs::write(self.path("home/.config/easy-note/config.toml"), text).unwrap();
    }
    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }
    fn state_dir(&self) -> PathBuf {
        self.path("notes/.easy-note")
    }
    fn store(&self) -> Store {
        Store::system(self.path("notes"))
    }
    /// A note file made on day `day` of November 2026, so the newest-first order is fixed.
    fn note(&self, day: u32, text: &str) -> String {
        let id = format!("202611{day:02}-120000-abcd");
        let path = self.note_path(&id);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let file = format!(
            "---\neasy-note: 1\nid: \"{id}\"\ncreated: \"2026-11-{day:02}T12:00:00+07:00\"\ntags: []\nsource: \"cli\"\n---\n{text}\n"
        );
        std::fs::write(path, file).unwrap();
        id
    }
    fn note_path(&self, id: &str) -> PathBuf {
        self.path("notes")
            .join(&id[0..4])
            .join(&id[4..6])
            .join(format!("{id}.md"))
    }
    fn edit_note(&self, id: &str, more: &str) {
        let path = self.note_path(id);
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(path, format!("{text}{more}\n")).unwrap();
    }
    fn enrich_only(&self, id: &str) {
        let input = Input {
            title: "old title",
            kind: "other",
            summary: "old summary",
            tags_suggested: &[],
            model: "m",
        };
        ai::enrich(&self.store(), id, &input).unwrap();
    }
    fn ai_file(&self, id: &str) -> Option<Value> {
        let raw = std::fs::read_to_string(self.state_dir().join(format!("ai/{id}.json"))).ok()?;
        Some(serde_json::from_str(&raw).unwrap())
    }
    fn state(&self) -> Value {
        let raw = std::fs::read_to_string(self.state_dir().join("ai-run-state.json")).unwrap();
        serde_json::from_str(&raw).unwrap()
    }
    fn calls(&self) -> usize {
        std::fs::read_to_string(self.path("bin/calls"))
            .unwrap_or_default()
            .lines()
            .count()
    }
    fn mode(&self, mode: &str) {
        std::fs::write(self.path("bin/mode"), mode).unwrap();
    }
    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_easy-note"));
        command
            .args(args)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", self.path("home"))
            .env("XDG_CONFIG_HOME", self.path("home/.config"))
            .env("EASY_NOTE_DIR", self.path("notes"))
            .current_dir(self.path("home"))
            .stdin(Stdio::null());
        command
    }
    fn exec(&self, args: &[&str]) -> Done {
        Done::of(self.command(args).output().unwrap())
    }
    fn run(&self) -> Done {
        self.exec(&["ai", "run"])
    }
    fn start(&self, args: &[&str]) -> Child {
        self.command(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
    }
    /// Start `ai run` and return once its child waits at the gate.
    fn run_to_gate(&self) -> Child {
        std::fs::write(self.path("bin/gate"), "").unwrap();
        let child = self.start(&["ai", "run"]);
        wait_for(|| self.path("bin/waiting").exists());
        child
    }
    fn open_gate(&self, child: Child) -> Done {
        std::fs::remove_file(self.path("bin/gate")).unwrap();
        Done::of(child.wait_with_output().unwrap())
    }
    /// Every file of the notes dir with its bytes.
    fn tree(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(dir: &Path, into: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    into.insert(path.clone(), vec![]);
                    walk(&path, into);
                } else {
                    into.insert(path.clone(), std::fs::read(path).unwrap());
                }
            }
        }
        let mut all = BTreeMap::new();
        walk(&self.path("notes"), &mut all);
        all
    }
}

struct Done {
    code: i32,
    out: String,
    err: String,
}

impl Done {
    fn of(output: Output) -> Done {
        Done {
            code: output.status.code().unwrap_or(-1),
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            err: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
    fn ok(self) -> Done {
        assert_eq!(self.code, 0, "out: {}\nerr: {}", self.out, self.err);
        self
    }
    fn last(&self) -> &str {
        self.out.lines().last().unwrap_or_default()
    }
}

fn wait_for(done: impl Fn() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(start.elapsed() < Duration::from_secs(8), "timed out");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Minutes from the state's `last_run` to its `next_after`.
fn wait_minutes(state: &Value) -> i64 {
    let at = |key: &str| DateTime::parse_from_rfc3339(state[key].as_str().unwrap()).unwrap();
    (at("next_after") - at("last_run")).num_minutes()
}

// AC 1

#[test]
fn with_enrich_off_it_says_how_to_turn_it_on_and_starts_nothing() {
    let sb = Sb::new("");
    sb.note(1, "one");
    sb.raw_config(&format!(
        "[ai]\ncommand = \"{}\"\n",
        sb.path("bin/claude").display()
    ));
    for args in [&["ai", "run"][..], &["ai", "run", "--dry-run"]] {
        let done = sb.exec(args);
        assert_eq!(done.code, 1, "{}", done.out);
        assert!(
            done.err
                .contains("AI enrich is off: set [ai] enrich = \"claude\""),
            "{}",
            done.err
        );
    }
    assert_eq!(sb.calls(), 0);
    assert!(!sb.state_dir().exists(), "nothing written");
}

// AC 2, 5, 7 — success criterion 1

#[test]
fn five_notes_in_batches_of_two_take_three_calls_and_record_the_run() {
    let sb = Sb::new("batch_size = 2");
    let ids: Vec<String> = (1..=5).map(|d| sb.note(d, "a note")).collect();
    let before: Vec<Vec<u8>> = ids
        .iter()
        .map(|id| std::fs::read(sb.note_path(id)).unwrap())
        .collect();
    let done = sb.run().ok();
    assert_eq!(
        done.last(),
        "organized 5, skipped 0, left 0 pending, calls 3"
    );
    assert_eq!(sb.calls(), 3);
    for (id, bytes) in ids.iter().zip(before) {
        assert_eq!(sb.ai_file(id).unwrap()["topic"], "model topic");
        assert_eq!(std::fs::read(sb.note_path(id)).unwrap(), bytes, "NFR-012");
    }
    let state = sb.state();
    assert!(state["last_run"].is_string(), "{state}");
    assert_eq!(state["organized"], 5);
    assert_eq!(state["skipped"], 0);
    assert_eq!(state["left"], 0);
    assert_eq!(state["calls"], 3);
    assert_eq!(state["failures"], 0);
    assert_eq!(state["next_after"], Value::Null);
    assert_eq!(state["last_error"], Value::Null);
}

#[test]
fn it_takes_the_newest_notes_up_to_the_smaller_of_limit_and_max_per_run() {
    let sb = Sb::new("batch_size = 2\nmax_per_run = 3");
    let ids: Vec<String> = (1..=5).map(|d| sb.note(d, "a note")).collect();
    let done = sb.exec(&["ai", "run", "--limit", "1"]).ok();
    assert_eq!(
        done.last(),
        "organized 1, skipped 0, left 4 pending, calls 1"
    );
    assert!(sb.ai_file(&ids[4]).is_some(), "the newest");
    let done = sb.exec(&["ai", "run", "--limit", "50"]).ok();
    assert_eq!(
        done.last(),
        "organized 3, skipped 0, left 1 pending, calls 2"
    );
    assert!(sb.ai_file(&ids[0]).is_none(), "the oldest waits");
    assert!(sb.ai_file(&ids[1]).is_some());
    let done = sb.exec(&["ai", "run", "--limit", "0"]);
    assert_eq!(done.code, 1);
    assert!(
        done.err.contains("--limit must be at least 1"),
        "{}",
        done.err
    );
}

#[test]
fn nothing_to_do_starts_no_child() {
    let sb = Sb::new("");
    let done = sb.run().ok();
    assert_eq!(done.last(), "nothing to do");
    assert_eq!(sb.calls(), 0);
    assert_eq!(sb.state()["calls"], 0);
}

// AC 3 — success criterion 4

#[test]
fn dry_run_prints_the_plan_starts_no_child_and_writes_nothing() {
    let sb = Sb::new("batch_size = 2\nmax_per_run = 3");
    for d in 1..=5 {
        sb.note(d, "a note");
    }
    let before = sb.tree();
    let done = sb.exec(&["ai", "run", "--dry-run"]).ok();
    assert_eq!(
        done.last(),
        "dry run: 3 notes in 2 calls with model haiku; nothing was started or written"
    );
    assert_eq!(sb.calls(), 0);
    assert_eq!(sb.tree(), before);
    let done = sb.exec(&["ai", "run", "--dry-run", "--limit", "1"]).ok();
    assert!(
        done.last().starts_with("dry run: 1 note in 1 call "),
        "{}",
        done.out
    );
}

// AC 4 — success criterion 2

#[test]
fn a_second_run_says_already_running_and_a_dead_runs_lock_never_blocks() {
    let sb = Sb::new("");
    let id = sb.note(1, "one");
    let first = sb.run_to_gate();
    let second = sb.run().ok();
    assert_eq!(second.last(), "already running");
    assert_eq!(sb.calls(), 1, "one child only");
    sb.open_gate(first).ok();
    assert!(sb.ai_file(&id).is_some());
    assert!(sb.state_dir().join("ai-run.lock").exists());

    sb.note(2, "two");
    std::fs::remove_file(sb.path("bin/waiting")).unwrap();
    let mut killed = sb.run_to_gate();
    killed.kill().unwrap();
    killed.wait().unwrap();
    std::fs::remove_file(sb.path("bin/gate")).unwrap();
    let done = sb.run().ok();
    assert_eq!(
        done.last(),
        "organized 1, skipped 0, left 0 pending, calls 1"
    );
}

// AC 5 — success criterion 3

#[test]
fn failed_runs_double_the_wait_and_a_good_run_resets_it() {
    let sb = Sb::new("interval_min = 10");
    sb.note(1, "one");
    sb.mode("fail");
    for (failures, minutes) in [(1, 20), (2, 40)] {
        let done = sb.run();
        assert_eq!(done.code, 1, "{}", done.out);
        assert!(done.err.contains("claude exited 1"), "{}", done.err);
        assert!(
            done.err
                .contains("organized 0, skipped 1, left 1 pending, calls 1"),
            "{}",
            done.err
        );
        let state = sb.state();
        assert_eq!(state["failures"], failures);
        assert_eq!(wait_minutes(&state), minutes);
        let error = state["last_error"].as_str().unwrap();
        assert!(error.starts_with("claude exited 1"), "{error}");
        assert!(!error.contains('\n'), "one line: {error}");
    }
    // By hand, `next_after` is ignored: the child is started again at once.
    assert_eq!(sb.calls(), 2);
    sb.mode("ok");
    sb.run().ok();
    let state = sb.state();
    assert_eq!(state["failures"], 0);
    assert_eq!(state["next_after"], Value::Null);
    assert_eq!(state["last_error"], Value::Null);
}

#[test]
fn the_wait_never_passes_four_hours() {
    let sb = Sb::new("interval_min = 200");
    sb.note(1, "one");
    sb.mode("fail");
    sb.run();
    assert_eq!(wait_minutes(&sb.state()), 240);
}

#[test]
fn a_run_stops_at_the_first_failed_call_and_a_partial_run_exits_zero_and_backs_off() {
    let sb = Sb::new("batch_size = 1\ninterval_min = 10");
    for d in 1..=3 {
        sb.note(d, "a note");
    }
    sb.mode("fail_after_first");
    let done = sb.run().ok();
    assert_eq!(
        done.last(),
        "organized 1, skipped 1, left 2 pending, calls 2"
    );
    assert_eq!(sb.calls(), 2, "no third call");
    assert_eq!(sb.state()["failures"], 1);
}

#[test]
fn a_missing_claude_is_a_failed_run() {
    let sb = Sb::new("");
    sb.note(1, "one");
    sb.raw_config("[ai]\nenrich = \"claude\"\ncommand = \"/no/such/claude\"\n");
    let done = sb.run();
    assert_eq!(done.code, 1);
    assert!(done.err.contains("claude not found"), "{}", done.err);
    assert_eq!(sb.state()["failures"], 1);
}

#[test]
fn a_broken_state_file_never_blocks_a_run() {
    let sb = Sb::new("");
    sb.note(1, "one");
    std::fs::create_dir_all(sb.state_dir()).unwrap();
    std::fs::write(sb.state_dir().join("ai-run-state.json"), "{not json").unwrap();
    sb.run().ok();
    assert_eq!(sb.state()["failures"], 0);
}

#[test]
fn a_notes_dir_it_cannot_write_is_a_clear_error() {
    let sb = Sb::new("");
    sb.note(1, "one");
    std::fs::create_dir_all(sb.state_dir()).unwrap();
    let set = |mode: u32| {
        std::fs::set_permissions(sb.state_dir(), std::fs::Permissions::from_mode(mode)).unwrap()
    };
    set(0o500);
    let done = sb.run();
    set(0o700);
    assert_eq!(done.code, 1, "{}", done.out);
    assert!(done.err.contains("ai-run.lock"), "{}", done.err);
    assert_eq!(sb.calls(), 0);
}

// AC 6

#[test]
fn without_the_organize_and_drafts_jobs_it_does_nothing_and_says_so() {
    let sb = Sb::new("jobs = [\"digest\"]");
    sb.note(1, "one");
    let before = sb.tree();
    for args in [&["ai", "run"][..], &["ai", "run", "--dry-run"]] {
        let done = sb.exec(args).ok();
        assert!(
            done.last()
                .contains("[ai] jobs has neither \"organize\" nor \"drafts\""),
            "{}",
            done.out
        );
    }
    assert_eq!(sb.calls(), 0);
    assert_eq!(sb.tree(), before);
}

// AC 8

#[test]
fn a_note_enriched_but_not_organized_is_picked_again() {
    let sb = Sb::new("");
    let never = sb.note(1, "enriched only");
    let older = sb.note(2, "organized before the last enrich");
    let fine = sb.note(3, "organized");
    sb.enrich_only(&never);
    for (id, at) in [(&older, "2000-01-01T00:00:00+07:00"), (&fine, "")] {
        sb.enrich_only(id);
        let organized = Organized {
            topic: Some("kept topic"),
            ..Organized::default()
        };
        ai::organize(&sb.store(), id, &organized).unwrap();
        if !at.is_empty() {
            let mut file = sb.ai_file(id).unwrap();
            file["organized_at"] = at.into();
            std::fs::write(
                sb.state_dir().join(format!("ai/{id}.json")),
                file.to_string(),
            )
            .unwrap();
        }
    }
    let done = sb.run().ok();
    assert_eq!(
        done.last(),
        "organized 2, skipped 0, left 0 pending, calls 1"
    );
    assert_eq!(sb.ai_file(&fine).unwrap()["topic"], "kept topic");
    assert_eq!(sb.ai_file(&never).unwrap()["topic"], "model topic");
    assert_eq!(sb.ai_file(&older).unwrap()["topic"], "model topic");
}

// AC 9

#[test]
fn a_fix_made_while_the_call_runs_is_kept() {
    let sb = Sb::new("");
    let id = sb.note(1, "one");
    let run = sb.run_to_gate();
    // AC 11: the write lock is free during the call, so this returns at once.
    sb.exec(&["ai", "set", &id, "topic", "mine"]).ok();
    sb.open_gate(run).ok();
    let file = sb.ai_file(&id).unwrap();
    assert!(file.get("topic").is_none(), "the AI wrote no topic: {file}");
    assert_eq!(file["project"], "model project");
    assert_eq!(
        ai::effective(&sb.store(), &id).topic.as_deref(),
        Some("mine")
    );
    // The latest step is the AI's item, logged after the person's fix.
    let undone = overrides::undo(&sb.store(), &id).unwrap();
    assert!(!undone.fields.contains(&"topic".to_string()), "{undone:?}");
}

// AC 10

#[test]
fn an_undone_note_is_held_until_its_text_changes() {
    let sb = Sb::new("");
    let id = sb.note(1, "one");
    sb.run().ok();
    sb.exec(&["ai", "undo", &id]).ok();
    assert!(sb.ai_file(&id).is_none());
    let done = sb.run().ok();
    assert_eq!(done.last(), "nothing to do");
    assert_eq!(sb.calls(), 1);
    sb.edit_note(&id, "and more");
    let done = sb.run().ok();
    assert_eq!(
        done.last(),
        "organized 1, skipped 0, left 0 pending, calls 1"
    );
    assert!(sb.ai_file(&id).is_some());
}

#[test]
fn ai_enrich_by_hand_ends_the_hold() {
    let sb = Sb::new("");
    let id = sb.note(1, "one");
    sb.run().ok();
    sb.exec(&["ai", "undo", &id]).ok();
    sb.exec(&["ai", "enrich", &id]).ok();
    // Enriched, not organized (AC 8), and no longer held.
    let done = sb.run().ok();
    assert_eq!(
        done.last(),
        "organized 1, skipped 0, left 0 pending, calls 1"
    );
    assert_eq!(sb.ai_file(&id).unwrap()["topic"], "model topic");
}

#[test]
fn undoing_your_own_fix_sets_no_hold() {
    let sb = Sb::new("");
    let id = sb.note(1, "one");
    sb.exec(&["ai", "set", &id, "topic", "mine"]).ok();
    sb.exec(&["ai", "undo", &id]).ok();
    let done = sb.run().ok();
    assert_eq!(
        done.last(),
        "organized 1, skipped 0, left 0 pending, calls 1"
    );
}

#[test]
fn an_undo_made_while_the_call_runs_holds_the_note() {
    let sb = Sb::new("");
    let id = sb.note(1, "one");
    // Enriched but not organized, so the run picks it and there is an AI step to undo.
    sb.enrich_only(&id);
    let run = sb.run_to_gate();
    sb.exec(&["ai", "undo", &id]).ok();
    let done = sb.open_gate(run).ok();
    assert_eq!(
        done.last(),
        "organized 0, skipped 1, left 0 pending, calls 1"
    );
    assert!(done.err.contains("undone while AI ran"), "{}", done.err);
    assert!(sb.ai_file(&id).is_none(), "the undo stands");
}

#[test]
fn a_broken_hold_file_stops_the_run_and_the_undo_changes_nothing() {
    let sb = Sb::new("");
    let id = sb.note(1, "one");
    sb.run().ok();
    let hold = sb.state_dir().join("ai-hold.json");
    std::fs::write(&hold, "{not json").unwrap();
    let before = sb.ai_file(&id).unwrap();
    let done = sb.exec(&["ai", "undo", &id]);
    assert_eq!(done.code, 1);
    assert!(done.err.contains("ai-hold.json"), "{}", done.err);
    assert_eq!(sb.ai_file(&id).unwrap(), before);
    sb.note(2, "two");
    let done = sb.run();
    assert_eq!(done.code, 1, "{}", done.out);
    assert!(done.err.contains("ai-hold.json"), "{}", done.err);
    assert_eq!(sb.calls(), 1);
    assert_eq!(std::fs::read_to_string(hold).unwrap(), "{not json");
}

// AC 11

#[test]
fn set_undo_rm_and_a_run_wait_for_the_write_lock() {
    let sb = Sb::new("");
    let id = sb.note(1, "one");
    sb.run().ok();
    sb.note(2, "two");
    let lock: File = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(sb.state_dir().join("ai-write.lock"))
        .unwrap();
    for args in [
        &["ai", "set", &id, "topic", "mine"][..],
        &["ai", "undo", &id],
        &["ai", "rm", &id],
        &["ai", "run"],
    ] {
        lock.lock().unwrap();
        let before = sb.tree();
        let mut child = sb.start(args);
        std::thread::sleep(Duration::from_millis(600));
        assert!(child.try_wait().unwrap().is_none(), "{args:?} waits");
        let ai_files = |tree: &BTreeMap<PathBuf, Vec<u8>>| {
            tree.iter()
                .filter(|(path, _)| {
                    let name = path.to_string_lossy();
                    name.contains("/ai/") || name.ends_with("ai-overrides.json")
                })
                .map(|(path, bytes)| (path.clone(), bytes.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(ai_files(&sb.tree()), ai_files(&before), "{args:?}");
        lock.unlock().unwrap();
        Done::of(child.wait_with_output().unwrap()).ok();
    }
}
