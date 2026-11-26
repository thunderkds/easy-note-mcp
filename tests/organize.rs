//! HUN-294: the batch engine of `ai run` (FR-047, FR-050, NFR-012, NFR-015). One call to the
//! person's `claude` per batch; each item is checked alone and written to AI files only.
//!
//! Every test runs `run_batch` in process on a `Store` over a temp dir, with temp HOME and
//! XDG_CONFIG_HOME, and a fake `claude` shell script (an absolute `command`). The fake records its
//! argv, env and stdin next to itself and prints the canned answer in its dir. No test starts the
//! real `claude` or opens the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use easy_note_mcp::core::ai::{self, Input, Organized};
use easy_note_mcp::core::config::AiSettings;
use easy_note_mcp::core::overrides;
use easy_note_mcp::core::store::Store;
use easy_note_mcp::surfaces::organize::{run_batch, Outcome};
use serde_json::{json, Value};

const GITHUB: &str = concat!("gh", "p_EXAMPLEexampleEXAMPLEexampleEXAMPLE123456");
/// Every test holds it: some set process env, and the fake reads files by fixed names.
static LOCK: Mutex<()> = Mutex::new(());

const FAKE: &str = r#"#!/bin/sh
d=$(dirname "$0")
for a in "$@"; do printf '[%s]\n' "$a"; done > "$d/argv"
env > "$d/env"
cat > "$d/stdin"
echo call >> "$d/calls"
case "$(cat "$d/mode" 2>/dev/null)" in
  junk) echo "this is not json";;
  exit1) exit 1;;
  sleep) echo $$ > "$d/pid"; exec sleep 30;;
  edit) printf 'edited\n' >> "$(cat "$d/edit")"; cat "$d/answer";;
  *) cat "$d/answer";;
esac
"#;

struct Sb {
    dir: tempfile::TempDir,
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
    _lock: MutexGuard<'static, ()>,
}

impl Sb {
    fn new() -> Sb {
        let lock = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::Builder::new()
            .prefix("easy-note-organize-")
            .tempdir()
            .unwrap();
        for d in ["home", "cfg", "bin", "notes"] {
            std::fs::create_dir_all(dir.path().join(d)).unwrap();
        }
        let fake = dir.path().join("bin/claude");
        std::fs::write(&fake, FAKE).unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut sb = Sb {
            dir,
            saved: vec![],
            _lock: lock,
        };
        sb.set_env("HOME", sb.path("home"));
        sb.set_env("XDG_CONFIG_HOME", sb.path("cfg"));
        sb.set_env("EASY_NOTE_DIR", sb.path("notes"));
        sb
    }
    fn set_env(&mut self, key: &'static str, value: impl AsRef<std::ffi::OsStr>) {
        self.saved.push((key, std::env::var_os(key)));
        std::env::set_var(key, value);
    }
    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }
    fn bin(&self, name: &str) -> PathBuf {
        self.path("bin").join(name)
    }
    fn store(&self) -> Store {
        Store::system(self.path("notes"))
    }
    fn settings(&self) -> AiSettings {
        AiSettings {
            enrich: true,
            command: self.bin("claude").display().to_string(),
            claude_config_dir: Some(self.path("acct")),
            model: "haiku".into(),
            ..AiSettings::default()
        }
    }
    fn add(&self, text: &str) -> String {
        self.store().add(text, "cli", &[], &[]).unwrap().id
    }
    fn add_in_repo(&self, text: &str, repo: &str) -> String {
        self.store()
            .add(text, "cli", &[], &[("repo", Some(repo))])
            .unwrap()
            .id
    }
    /// A hand-written (or synced) note file: its body is not masked, as `add` would.
    fn plant(&self, id: &str, body: &str) -> String {
        let path = self.note_path(id);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text = format!(
            "---\neasy-note: 1\nid: \"{id}\"\ncreated: \"2026-11-14T16:47:13+07:00\"\ntags: []\nsource: \"cli\"\n---\n{body}\n"
        );
        std::fs::write(path, text).unwrap();
        id.to_string()
    }
    fn note_path(&self, id: &str) -> PathBuf {
        self.path("notes")
            .join(&id[0..4])
            .join(&id[4..6])
            .join(format!("{id}.md"))
    }
    fn ai_path(&self, id: &str) -> PathBuf {
        self.path("notes/.easy-note/ai").join(format!("{id}.json"))
    }
    fn ai_file(&self, id: &str) -> Option<Value> {
        let raw = std::fs::read_to_string(self.ai_path(id)).ok()?;
        Some(serde_json::from_str(&raw).unwrap())
    }
    fn log(&self) -> Vec<Value> {
        std::fs::read_to_string(self.path("notes/.easy-note/ai-log.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }
    fn answer(&self, items: &Value) {
        let out = json!({
            "type": "result",
            "subtype": "success",
            "is_error": false,
            "result": items.to_string(),
            "structured_output": items,
            "modelUsage": {"claude-haiku-5-5": {"inputTokens": 10}},
        });
        std::fs::write(self.bin("answer"), out.to_string()).unwrap();
    }
    fn mode(&self, mode: &str) {
        std::fs::write(self.bin("mode"), mode).unwrap();
    }
    fn recorded(&self, name: &str) -> String {
        std::fs::read_to_string(self.bin(name)).unwrap_or_default()
    }
    fn calls(&self) -> usize {
        self.recorded("calls").lines().count()
    }
    fn run(&self, ids: &[&String]) -> Result<Outcome, String> {
        let ids: Vec<String> = ids.iter().map(|s| s.to_string()).collect();
        run_batch(&self.store(), &self.settings(), &ids)
    }
    /// A valid AI file through the existing writers, with a kind, project and topic.
    fn seed(&self, id: &str, kind: &str, project: &str, topic: &str) {
        let store = self.store();
        let input = Input {
            title: "old title",
            kind,
            summary: "old summary",
            tags_suggested: &[],
            model: "m",
        };
        ai::enrich(&store, id, &input).unwrap();
        let o = Organized {
            project: Some(project),
            topic: Some(topic),
            ..Organized::default()
        };
        ai::organize(&store, id, &o).unwrap();
    }
}

impl Drop for Sb {
    fn drop(&mut self) {
        for (key, value) in self.saved.drain(..).rev() {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }
}

fn item(id: &str, kind: &str) -> Value {
    json!({
        "id": id,
        "title": format!("title of {kind}"),
        "kind": kind,
        "summary": "what it is about",
        "project": "model-project",
        "topic": "usb hubs",
        "tags_suggested": ["hardware"],
    })
}

fn pending_reason<'a>(out: &'a Outcome, id: &str) -> &'a str {
    out.pending
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.reason.as_str())
        .unwrap_or_else(|| panic!("{id} not pending: {out:?}"))
}

// AC 1, 2, 5, 7, 10 — success criterion 1

#[test]
fn a_good_array_for_three_notes_writes_three_ai_files_and_log_lines() {
    let mut sb = Sb::new();
    sb.set_env("ANTHROPIC_API_KEY", "x-api");
    sb.set_env("ANTHROPIC_AUTH_TOKEN", "x-token");
    let a = sb.add("buy a usb-c hub");
    let b = sb.add_in_repo("retry bug only on firefox", "easy-ui-mcp");
    let c = sb.add("half a thought about");
    let mut draft = item(&c, "draft");
    draft["draft_reason"] = json!("it stops mid-sentence");
    draft["draft_suggestion"] = json!("half a thought about the tray icon");
    sb.answer(&json!([item(&a, "todo"), item(&b, "bug"), draft]));

    let out = sb.run(&[&a, &b, &c]).unwrap();
    assert_eq!(out.written.len(), 3, "{out:?}");
    assert!(out.pending.is_empty(), "{out:?}");
    assert_eq!(sb.calls(), 1, "one call for the batch");

    // AC 2: the argv, exactly; the array schema
    let argv = sb.recorded("argv");
    let lines: Vec<&str> = argv.lines().collect();
    assert_eq!(lines.len(), 11, "{argv}");
    assert_eq!(
        &lines[0..4],
        ["[-p]", "[--output-format]", "[json]", "[--json-schema]"]
    );
    let schema: Value = serde_json::from_str(&lines[4][1..lines[4].len() - 1]).unwrap();
    assert_eq!(schema["type"], "array");
    assert_eq!(
        schema["items"]["properties"]["kind"]["enum"],
        json!(["bug", "idea", "question", "todo", "draft", "other"])
    );
    assert_eq!(
        &lines[5..],
        [
            "[--tools]",
            "[]",
            "[--strict-mcp-config]",
            "[--no-session-persistence]",
            "[--model]",
            "[haiku]"
        ]
    );
    // AC 2: the env
    let env = sb.recorded("env");
    assert!(
        !env.lines().any(|l| l.starts_with("ANTHROPIC_API_KEY=")),
        "{env}"
    );
    assert!(
        !env.lines().any(|l| l.starts_with("ANTHROPIC_AUTH_TOKEN=")),
        "{env}"
    );
    let account = format!("CLAUDE_CONFIG_DIR={}", sb.path("acct").display());
    assert!(env.lines().any(|l| l == account), "{env}");

    // AC 10: a normal AI file with the new keys; the model id the CLI reports
    let fa = sb.ai_file(&a).unwrap();
    assert_eq!(fa["kind"], "todo");
    assert_eq!(fa["title"], "title of todo");
    assert_eq!(fa["project"], "model-project");
    assert_eq!(fa["topic"], "usb hubs");
    assert_eq!(fa["tags_suggested"], json!(["hardware"]));
    assert_eq!(fa["model"], "claude-haiku-5-5");
    assert!(fa["organized_at"].is_string());
    // AC 5: the repo wins over the model's project
    assert_eq!(sb.ai_file(&b).unwrap()["project"], "easy-ui-mcp");
    // AC 10: a draft goes through enrich as other, then organize sets draft
    let fc = sb.ai_file(&c).unwrap();
    assert_eq!(fc["kind"], "draft");
    assert_eq!(fc["draft_reason"], "it stops mid-sentence");
    assert_eq!(fc["draft_suggestion"], "half a thought about the tray icon");
    // NFR-012: the notes count as enriched
    assert_eq!(ai::pending(&sb.store(), 20).unwrap().total, 0);

    // AC 7: one log line per applied field change, who ai
    let log = sb.log();
    assert!(log.len() >= 3, "{log:?}");
    assert!(log.iter().all(|l| l["who"] == "ai"), "{log:?}");
    assert!(log.iter().any(|l| l["id"] == c.as_str()
        && l["field"] == "kind"
        && l["old"] == "other"
        && l["new"] == "draft"));
    assert!(log
        .iter()
        .any(|l| l["id"] == b.as_str() && l["field"] == "project" && l["new"] == "easy-ui-mcp"));
}

// AC 3 — success criterion 2 and the item edge cases

#[test]
fn each_item_is_checked_alone_and_a_bad_one_leaves_only_its_note_pending() {
    let sb = Sb::new();
    let good = sb.add("good note");
    let lunch = sb.add("lunch note");
    let twice = sb.add("answered twice");
    let missing = sb.add("no answer for me");
    let long = sb.add("long topic");
    let mut long_item = item(&long, "idea");
    long_item["topic"] = json!("t".repeat(61));
    sb.answer(&json!([
        item(&good, "idea"),
        item(&lunch, "lunch"),
        item(&twice, "todo"),
        item(&twice, "bug"),
        item("20990101-000000-zzzz", "todo"),
        long_item,
    ]));
    let out = sb.run(&[&good, &lunch, &twice, &missing, &long]).unwrap();
    assert_eq!(
        out.written
            .iter()
            .map(|w| w.id.as_str())
            .collect::<Vec<_>>(),
        [good.as_str()]
    );
    assert!(
        pending_reason(&out, &lunch).contains("kind must be one of"),
        "{out:?}"
    );
    assert!(
        pending_reason(&out, &twice).contains("more than once"),
        "{out:?}"
    );
    assert!(
        pending_reason(&out, &missing).contains("no answer"),
        "{out:?}"
    );
    assert!(
        pending_reason(&out, &long).contains("topic is 61 chars"),
        "{out:?}"
    );
    for id in [&lunch, &twice, &missing, &long] {
        assert!(sb.ai_file(id).is_none(), "{id} has no AI file");
    }
    assert!(sb.ai_file(&good).is_some());
    assert!(
        !sb.path("notes/.easy-note/ai/20990101-000000-zzzz.json")
            .exists(),
        "an id not in the batch is dropped"
    );
}

// AC 4 — success criterion 3

fn fails_whole(sb: &Sb, ids: &[&String]) -> String {
    let err = sb.run(ids).unwrap_err();
    for id in ids {
        assert!(sb.ai_file(id).is_none(), "nothing written: {err}");
    }
    assert!(sb.log().is_empty(), "no log line: {err}");
    err
}

#[test]
fn a_failed_call_junk_or_a_timeout_writes_nothing() {
    let sb = Sb::new();
    let a = sb.add("one");
    let b = sb.add("two");
    sb.answer(&json!([item(&a, "todo"), item(&b, "todo")]));

    sb.mode("exit1");
    let err = fails_whole(&sb, &[&a, &b]);
    assert!(err.contains("claude exited 1"), "{err}");

    sb.mode("junk");
    let err = fails_whole(&sb, &[&a, &b]);
    assert_eq!(err, "the answer is not JSON");

    sb.mode("sleep");
    let settings = AiSettings {
        timeout_secs: 1,
        ..sb.settings()
    };
    let start = Instant::now();
    let err = run_batch(&sb.store(), &settings, &[a.clone(), b.clone()]).unwrap_err();
    assert_eq!(err, "timeout");
    assert!(
        start.elapsed() < Duration::from_secs(10),
        "{:?}",
        start.elapsed()
    );
    let pid = sb.recorded("pid");
    assert!(
        !Path::new(&format!("/proc/{}", pid.trim())).exists(),
        "killed and reaped"
    );
    assert!(sb.ai_file(&a).is_none() && sb.ai_file(&b).is_none());
}

#[test]
fn an_answer_that_fails_the_array_schema_writes_nothing() {
    let sb = Sb::new();
    let a = sb.add("one");
    let b = sb.add("two");
    let mut no_topic = item(&b, "todo");
    no_topic.as_object_mut().unwrap().remove("topic");
    let mut extra = item(&b, "todo");
    extra["x"] = json!(1);
    for bad in [
        json!(item(&a, "todo")),
        json!([item(&a, "todo"), no_topic]),
        json!([item(&a, "todo"), extra]),
        json!([item(&a, "todo"), {"id": b, "title": 3}]),
    ] {
        sb.answer(&bad);
        let err = fails_whole(&sb, &[&a, &b]);
        assert!(err.contains("shape"), "{bad}: {err}");
    }
}

#[test]
fn a_huge_answer_is_read_to_1_mib_and_fails() {
    let sb = Sb::new();
    let a = sb.add("one");
    let pad = " ".repeat(2 << 20);
    let raw = format!(
        "{{\"is_error\": false,{pad}\"structured_output\": {}}}",
        json!([item(&a, "todo")])
    );
    std::fs::write(sb.bin("answer"), raw).unwrap();
    let err = fails_whole(&sb, &[&a]);
    assert_eq!(err, "the answer is not JSON");
}

#[test]
fn with_enrich_off_nothing_runs() {
    let sb = Sb::new();
    let a = sb.add("one");
    let settings = AiSettings {
        enrich: false,
        ..sb.settings()
    };
    let err = run_batch(&sb.store(), &settings, std::slice::from_ref(&a)).unwrap_err();
    assert!(err.contains("AI enrich is off"), "{err}");
    assert_eq!(sb.calls(), 0);
}

// AC 8 — success criterion 4

#[test]
fn a_note_edited_while_the_call_ran_is_skipped() {
    let sb = Sb::new();
    let a = sb.add("stays the same");
    let b = sb.add("edited during the call");
    sb.answer(&json!([item(&a, "todo"), item(&b, "todo")]));
    std::fs::write(sb.bin("edit"), sb.note_path(&b).display().to_string()).unwrap();
    sb.mode("edit");
    let out = sb.run(&[&a, &b]).unwrap();
    assert!(sb.ai_file(&a).is_some());
    assert!(pending_reason(&out, &b).contains("changed"), "{out:?}");
    assert!(sb.ai_file(&b).is_none());
}

// AC 6 — overrides win

#[test]
fn a_field_with_an_override_is_kept_and_reported() {
    let sb = Sb::new();
    let a = sb.add("hub for the desk");
    sb.seed(&a, "idea", "home", "desk setup");
    overrides::set(&sb.store(), &a, "topic", "my topic").unwrap();
    sb.answer(&json!([item(&a, "todo")]));
    let out = sb.run(&[&a]).unwrap();
    assert_eq!(out.written.len(), 1, "{out:?}");
    assert_eq!(out.written[0].kept, ["topic"]);
    let f = sb.ai_file(&a).unwrap();
    assert_eq!(f["topic"], "desk setup", "AI file topic unchanged");
    assert_eq!(f["project"], "model-project");
    assert_eq!(f["kind"], "todo");
    assert_eq!(
        ai::effective(&sb.store(), &a).topic.as_deref(),
        Some("my topic")
    );
}

// Settled decision (2026-11-26): an existing AI file — enrich with the old kind, then organize

#[test]
fn a_stale_file_kind_change_is_logged_and_undone() {
    let sb = Sb::new();
    let a = sb.add("fix the tray icon");
    sb.seed(&a, "bug", "home", "tray");
    // Stale: the note changed after its AI file was written.
    let note = sb.note_path(&a);
    let mut text = std::fs::read_to_string(&note).unwrap();
    text.push_str("more\n");
    std::fs::write(&note, &text).unwrap();
    assert!(
        ai::read(&sb.store(), &a).unwrap().1,
        "stale before the batch"
    );
    let log_before = sb.log().len();

    let mut todo = item(&a, "todo");
    todo["project"] = json!("home");
    todo["topic"] = json!("tray");
    sb.answer(&json!([todo]));
    let out = sb.run(&[&a]).unwrap();
    assert_eq!(out.written.len(), 1, "{out:?}");
    let f = sb.ai_file(&a).unwrap();
    assert_eq!(f["kind"], "todo");
    assert_eq!(f["title"], "title of todo");
    assert!(
        !ai::read(&sb.store(), &a).unwrap().1,
        "fresh after the batch"
    );
    let new_lines = &sb.log()[log_before..];
    assert_eq!(new_lines.len(), 1, "{new_lines:?}");
    assert_eq!(new_lines[0]["field"], "kind");
    assert_eq!(new_lines[0]["old"], "bug");
    assert_eq!(new_lines[0]["new"], "todo");
    let undone = overrides::undo(&sb.store(), &a).unwrap();
    assert_eq!(undone.now.as_deref(), Some("bug"));
    assert_eq!(sb.ai_file(&a).unwrap()["kind"], "bug");
}

#[test]
fn with_a_kind_override_the_ai_kind_and_the_overrides_file_stay() {
    let sb = Sb::new();
    let a = sb.add("fix the tray icon");
    sb.seed(&a, "bug", "home", "tray");
    overrides::set(&sb.store(), &a, "kind", "idea").unwrap();
    let ovr = sb.path("notes/.easy-note/ai-overrides.json");
    let ovr_before = std::fs::read(&ovr).unwrap();
    let log_before = sb.log().len();
    let mut todo = item(&a, "todo");
    todo["project"] = json!("home");
    todo["topic"] = json!("tray");
    sb.answer(&json!([todo]));
    let out = sb.run(&[&a]).unwrap();
    assert_eq!(out.written[0].kept, ["kind"]);
    assert_eq!(sb.ai_file(&a).unwrap()["kind"], "bug");
    assert!(
        !sb.log()[log_before..].iter().any(|l| l["field"] == "kind"),
        "no kind line"
    );
    assert_eq!(std::fs::read(&ovr).unwrap(), ovr_before);
}

// AC 12 — enrich written, organize failed

static RENAMES: AtomicUsize = AtomicUsize::new(0);

#[test]
fn when_organize_fails_after_enrich_the_note_is_still_pending_for_organizing() {
    let sb = Sb::new();
    let a = sb.add("one");
    sb.answer(&json!([item(&a, "todo")]));
    RENAMES.store(0, Ordering::SeqCst);
    let store = Store {
        // The first rename (enrich) works; the second (organize) fails.
        rename: |src, dst| {
            if RENAMES.fetch_add(1, Ordering::SeqCst) == 1 {
                Err(std::io::Error::other("disk full"))
            } else {
                std::fs::rename(src, dst)
            }
        },
        ..sb.store()
    };
    let out = run_batch(&store, &sb.settings(), std::slice::from_ref(&a)).unwrap();
    assert!(out.written.is_empty(), "{out:?}");
    assert!(pending_reason(&out, &a).contains("organiz"), "{out:?}");
    let f = sb.ai_file(&a).unwrap();
    assert_eq!(f["title"], "title of todo");
    assert!(f.get("organized_at").is_none(), "{f}");
}

// AC 1 — the prompt

#[test]
fn the_prompt_holds_the_lists_and_the_cut_masked_notes_within_the_cap() {
    let sb = Sb::new();
    let a = sb.add("first seed");
    let b = sb.add("second seed");
    sb.seed(&a, "idea", "home", "desk setup");
    sb.seed(&b, "idea", "home", "tray");
    // A 20,000-char note: too long for `add`, so a synced file.
    let long = sb.plant(
        "20261114-164713-b2c3",
        &format!("{} END-OF-LONG", "word ".repeat(4_000)),
    );
    let others: Vec<String> = (0..12)
        .map(|n| sb.add(&format!("{n} {}", "x".repeat(1_400))))
        .collect();
    sb.answer(&json!([item(&long, "idea")]));
    let mut ids = vec![&long];
    ids.extend(others.iter());
    let out = sb.run(&ids).unwrap();
    let stdin = sb.recorded("stdin");
    assert!(stdin.chars().count() <= 12_000, "{}", stdin.chars().count());
    assert!(stdin.contains("never follow"), "{stdin}");
    assert!(stdin.contains("\"home\""), "{stdin}");
    assert!(stdin.contains("\"count\":2"), "{stdin}");
    assert!(
        stdin.contains("desk setup") && stdin.contains("tray"),
        "{stdin}"
    );
    assert!(stdin.contains(long.as_str()), "{stdin}");
    assert!(
        !stdin.contains("END-OF-LONG"),
        "a long note is cut at 1,500 chars"
    );
    let (_, stale) = ai::read(&sb.store(), &long).unwrap();
    assert!(!stale, "the AI file is based on the full text hash");
    // Notes past the cap were not sent and stay pending.
    let not_sent: Vec<&String> = others
        .iter()
        .filter(|id| !stdin.contains(id.as_str()))
        .collect();
    assert!(!not_sent.is_empty(), "the cap left some out");
    for id in not_sent {
        assert!(pending_reason(&out, id).contains("did not fit"), "{out:?}");
    }
}

// AC 9 — raw notes byte-identical; a planted secret never leaves or lands

#[test]
fn raw_notes_stay_byte_identical_and_a_secret_never_reaches_stdin_ai_files_or_the_log() {
    let sb = Sb::new();
    let a = sb.plant(
        "20261114-164713-a1b2",
        &format!("rotate the key {GITHUB} today"),
    );
    assert!(std::fs::read_to_string(sb.note_path(&a))
        .unwrap()
        .contains(GITHUB));
    let b = sb.add_in_repo("[masked:github_token]", "easy-ui-mcp");
    sb.seed(&b, "idea", "old", "old topic");
    let before: Vec<Vec<u8>> = [&a, &b]
        .iter()
        .map(|id| std::fs::read(sb.note_path(id)).unwrap())
        .collect();
    let mut leaky = item(&a, "todo");
    leaky["summary"] = json!(format!("the key is {GITHUB}"));
    leaky["topic"] = json!(format!("k {GITHUB}"));
    leaky["project"] = json!(GITHUB);
    let mut other = item(&b, "bug");
    other["topic"] = json!(format!("/home/me/{GITHUB}"));
    sb.answer(&json!([leaky, other]));
    let out = sb.run(&[&a, &b]).unwrap();
    assert_eq!(out.written.len(), 2, "{out:?}");
    let after: Vec<Vec<u8>> = [&a, &b]
        .iter()
        .map(|id| std::fs::read(sb.note_path(id)).unwrap())
        .collect();
    assert_eq!(before, after, "raw notes byte-identical");
    assert!(!sb.recorded("stdin").contains(GITHUB));
    assert!(sb.recorded("stdin").contains("[masked:github_token]"));
    for id in [&a, &b] {
        let raw = std::fs::read_to_string(sb.ai_path(id)).unwrap();
        assert!(!raw.contains(GITHUB), "{raw}");
    }
    let log = std::fs::read_to_string(sb.path("notes/.easy-note/ai-log.jsonl")).unwrap();
    assert!(!log.contains(GITHUB), "{log}");
}
