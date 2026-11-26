//! HUN-292: AI file keys, the overrides file, the change log, `ai set` / `ai undo` (FR-047,
//! FR-050, NFR-015). Library calls run on a `Store` over a temp dir; CLI calls run the real
//! binary with a temp HOME, XDG_CONFIG_HOME and EASY_NOTE_DIR. No test touches the real ones.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use easy_note_mcp::core::ai::{self, Input, Organized};
use easy_note_mcp::core::overrides;
use easy_note_mcp::core::store::Store;
use serde_json::{json, Value};

const GITHUB: &str = concat!("gh", "p_EXAMPLEexampleEXAMPLEexampleEXAMPLE123456");

struct Sb {
    dir: tempfile::TempDir,
}

impl Sb {
    fn new() -> Sb {
        Sb {
            dir: tempfile::Builder::new()
                .prefix("easy-note-ovr-")
                .tempdir()
                .unwrap(),
        }
    }
    fn notes(&self) -> PathBuf {
        self.dir.path().join("notes")
    }
    fn store(&self) -> Store {
        Store::system(self.notes())
    }
    fn add(&self, text: &str) -> String {
        self.store().add(text, "cli", &[], &[]).unwrap().id
    }
    fn enrich(&self, id: &str, kind: &str) {
        let input = Input {
            title: "t",
            kind,
            summary: "s",
            tags_suggested: &[],
            model: "m",
        };
        ai::enrich(&self.store(), id, &input).unwrap();
    }
    fn topic(&self, id: &str, topic: &str) {
        let o = Organized {
            topic: Some(topic),
            ..Organized::default()
        };
        ai::organize(&self.store(), id, &o).unwrap();
    }
    fn cli(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_easy-note"))
            .args(args)
            .env("HOME", self.dir.path())
            .env("XDG_CONFIG_HOME", self.dir.path().join("cfg"))
            .env("EASY_NOTE_DIR", self.notes())
            .current_dir(self.dir.path())
            .output()
            .unwrap()
    }
    fn log(&self) -> Vec<Value> {
        let path = self.notes().join(".easy-note/ai-log.jsonl");
        std::fs::read_to_string(path)
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }
    fn ai_file(&self, id: &str) -> PathBuf {
        self.notes().join(format!(".easy-note/ai/{id}.json"))
    }
    fn overrides_file(&self) -> PathBuf {
        self.notes().join(".easy-note/ai-overrides.json")
    }
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Every file under `root` except `.easy-note/` (and the AI file dir when `with_ai` is false).
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut todo = vec![root.to_path_buf()];
    while let Some(dir) = todo.pop() {
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                todo.push(p);
            } else if !p.starts_with(root.join(".easy-note")) {
                out.insert(p.clone(), std::fs::read(&p).unwrap());
            }
        }
    }
    out
}

// AC 1 -----------------------------------------------------------------------------------------

#[test]
fn detail_reads_with_and_without_the_new_keys_and_masks_and_caps_them() {
    let sb = Sb::new();
    let id = sb.add("a note");
    sb.enrich(&id, "bug");
    let path = sb.ai_file(&id);
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(
        v.get("project").is_none(),
        "absent keys stay out of the file"
    );
    assert!(ai::read(&sb.store(), &id).is_some());

    v["project"] = json!("p");
    v["topic"] = json!(format!("see {GITHUB}"));
    v["draft_reason"] = json!("r");
    v["draft_suggestion"] = json!("s");
    v["organized_at"] = json!("2026-11-25T10:00:00+07:00");
    v["kind"] = json!("draft");
    std::fs::write(&path, v.to_string()).unwrap();
    let (d, _) = ai::read(&sb.store(), &id).unwrap();
    assert_eq!(d.kind, "draft");
    assert_eq!(d.project.as_deref(), Some("p"));
    assert!(d.topic.unwrap().starts_with("see [masked:"));

    for (key, len) in [
        ("project", 61),
        ("topic", 61),
        ("draft_reason", 201),
        ("draft_suggestion", 601),
    ] {
        let mut bad = v.clone();
        bad[key] = json!("x".repeat(len));
        std::fs::write(&path, bad.to_string()).unwrap();
        assert!(ai::read(&sb.store(), &id).is_none(), "{key} over its cap");
    }
    let mut ok = v.clone();
    ok["project"] = json!("x".repeat(60));
    ok["draft_suggestion"] = json!("x".repeat(600));
    std::fs::write(&path, ok.to_string()).unwrap();
    assert!(ai::read(&sb.store(), &id).is_some());
}

#[test]
fn organize_masks_caps_stamps_and_enrich_keeps_the_keys() {
    let sb = Sb::new();
    let id = sb.add("a note");
    sb.enrich(&id, "idea");
    let o = Organized {
        project: Some(&format!("p {GITHUB}")),
        kind: Some("draft"),
        draft_reason: Some("too short"),
        ..Organized::default()
    };
    let d = ai::organize(&sb.store(), &id, &o).unwrap();
    assert!(d.project.as_deref().unwrap().starts_with("p [masked:"));
    assert!(d.organized_at.is_some());
    assert_eq!(d.kind, "draft");
    let too_long = "x".repeat(61);
    let bad = Organized {
        topic: Some(&too_long),
        ..Organized::default()
    };
    assert!(ai::organize(&sb.store(), &id, &bad).is_err());
    sb.enrich(&id, "bug");
    let (d, _) = ai::read(&sb.store(), &id).unwrap();
    assert_eq!(d.draft_reason.as_deref(), Some("too short"));
    let raw = std::fs::read_to_string(sb.ai_file(&id)).unwrap();
    assert!(!raw.contains(GITHUB));
    // no AI file: refused
    let other = sb.add("another");
    assert!(ai::organize(&sb.store(), &other, &o).is_err());
}

// AC 2, 3 --------------------------------------------------------------------------------------

#[test]
fn set_writes_the_overrides_file_masked_and_capped_and_refuses_unknown_ids() {
    let sb = Sb::new();
    let id = sb.add("a note");
    overrides::set(&sb.store(), &id, "project", &format!("p {GITHUB}")).unwrap();
    let raw = std::fs::read_to_string(sb.overrides_file()).unwrap();
    assert!(!raw.contains(GITHUB));
    assert!(raw.contains("[masked:"));
    assert!(overrides::set(&sb.store(), &id, "topic", &"x".repeat(61)).is_err());
    assert!(overrides::set(&sb.store(), &id, "kind", "rant").is_err());
    assert!(overrides::set(&sb.store(), &id, "title", "x").is_err());
    assert!(overrides::set(&sb.store(), &id, "topic", "  ").is_err());
    assert!(overrides::set(&sb.store(), "20200101-000000-abcd", "topic", "x").is_err());
    let tmp: Vec<_> = std::fs::read_dir(sb.notes().join(".easy-note"))
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
        .collect();
    assert!(tmp.is_empty(), "no temp file is left");
}

#[test]
fn effective_applies_the_override_and_the_rename_map_and_leaves_the_ai_file() {
    let sb = Sb::new();
    let id = sb.add("a note");
    sb.enrich(&id, "bug");
    sb.topic(&id, "a");
    assert_eq!(ai::effective(&sb.store(), &id).topic.as_deref(), Some("a"));
    let ai_before = std::fs::read(sb.ai_file(&id)).unwrap();
    overrides::set(&sb.store(), &id, "topic", "b").unwrap();
    overrides::set(&sb.store(), &id, "kind", "todo").unwrap();
    let e = ai::effective(&sb.store(), &id);
    assert_eq!(e.topic.as_deref(), Some("b"));
    assert_eq!(e.kind.as_deref(), Some("todo"));
    assert_eq!(std::fs::read(sb.ai_file(&id)).unwrap(), ai_before);

    // The rename map applies to the AI topic and to an override.
    let mut file: Value =
        serde_json::from_str(&std::fs::read_to_string(sb.overrides_file()).unwrap()).unwrap();
    file["topics"] = json!({"a": "alpha", "b": "beta"});
    std::fs::write(sb.overrides_file(), file.to_string()).unwrap();
    assert_eq!(
        ai::effective(&sb.store(), &id).topic.as_deref(),
        Some("beta")
    );
    overrides::undo(&sb.store(), &id).unwrap(); // kind
    overrides::undo(&sb.store(), &id).unwrap(); // topic
    assert_eq!(
        ai::effective(&sb.store(), &id).topic.as_deref(),
        Some("alpha")
    );
    assert_eq!(ai::effective(&sb.store(), &id).kind.as_deref(), Some("bug"));
}

#[test]
fn an_override_alone_applies_without_an_ai_file() {
    let sb = Sb::new();
    let id = sb.add("a note");
    overrides::set(&sb.store(), &id, "project", "web").unwrap();
    assert_eq!(
        ai::effective(&sb.store(), &id).project.as_deref(),
        Some("web")
    );
    assert_eq!(ai::effective(&sb.store(), "nope").project, None);
}

// AC 4 -----------------------------------------------------------------------------------------

#[test]
fn every_change_is_one_log_line() {
    let sb = Sb::new();
    let id = sb.add("a note");
    sb.enrich(&id, "bug");
    sb.topic(&id, "a");
    overrides::set(&sb.store(), &id, "topic", "b").unwrap();
    let log = sb.log();
    // The creation, then the organize: the earlier file and the one key it changed.
    assert_eq!(log.len(), 4, "{log:?}");
    assert_eq!(log[0]["field"], "ai_file");
    assert_eq!(log[0]["old"], Value::Null);
    assert_eq!(log[1]["field"], "ai_file");
    assert!(log[1]["old"].is_string());
    assert_eq!(log[2]["who"], "ai");
    assert_eq!(log[2]["field"], "topic");
    assert_eq!(log[2]["old"], Value::Null);
    assert_eq!(log[2]["new"], "a");
    assert_ne!(log[0]["op"], log[1]["op"]);
    assert_eq!(log[1]["op"], log[2]["op"]);
    assert_eq!(log[3]["who"], "person");
    assert_eq!(log[3]["id"], id.as_str());
    assert_eq!(log[3]["old"], Value::Null);
    assert_eq!(log[3]["new"], "b");
    assert!(log[3].get("op").is_none(), "a person's change is one step");
    assert!(log[3]["at"].as_str().unwrap().contains('T'));
    overrides::set(&sb.store(), &id, "topic", "c").unwrap();
    assert_eq!(sb.log()[4]["old"], "b");
}

#[test]
fn a_long_value_keeps_the_line_short_and_secrets_stay_out_of_the_log() {
    let sb = Sb::new();
    let id = sb.add("a note");
    overrides::log_change(
        &sb.store(),
        &id,
        "draft_suggestion",
        None,
        Some(&"é".repeat(600)),
        "ai",
    );
    overrides::log_change(&sb.store(), &id, "topic", None, Some(GITHUB), "ai");
    let raw = std::fs::read_to_string(sb.notes().join(".easy-note/ai-log.jsonl")).unwrap();
    assert!(raw.lines().all(|l| l.len() < 4096));
    assert!(!raw.contains(GITHUB));
}

// AC 5, 6 --------------------------------------------------------------------------------------

#[test]
fn cli_set_writes_the_override_and_a_bad_kind_or_empty_value_exits_1() {
    let sb = Sb::new();
    let id = sb.add("a note");
    let out = sb.cli(&["ai", "set", &id, "topic", "firefox flaky"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(std::fs::read_to_string(sb.overrides_file())
        .unwrap()
        .contains("firefox flaky"));
    assert_eq!(sb.log().len(), 1);
    let out = sb.cli(&["ai", "set", &id, "kind", "rant"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stderr).contains("bug, idea, question, todo, draft, other"));
    let out = sb.cli(&["ai", "set", &id, "topic", ""]);
    assert_eq!(out.status.code(), Some(1));
    let out = sb.cli(&["ai", "set", &id, "color", "red"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stderr).contains("project, topic, kind"));
    assert_eq!(sb.log().len(), 1, "a refusal logs nothing");
}

#[test]
fn undo_restores_walks_back_and_says_nothing_to_undo() {
    let sb = Sb::new();
    let id = sb.add("a note");
    sb.enrich(&id, "bug");
    sb.topic(&id, "a");
    let out = sb.cli(&["ai", "undo", "nope-id"]);
    assert_ne!(out.status.code(), Some(0));
    sb.cli(&["ai", "set", &id, "topic", "b"]);
    sb.cli(&["ai", "set", &id, "topic", "c"]);
    let e = |sb: &Sb| ai::effective(&sb.store(), &id).topic;
    assert_eq!(e(&sb).as_deref(), Some("c"));
    let out = sb.cli(&["ai", "undo", &id]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(e(&sb).as_deref(), Some("b"));
    sb.cli(&["ai", "undo", &id]);
    assert_eq!(
        e(&sb).as_deref(),
        Some("a"),
        "override removed, the AI value shows"
    );
    assert!(!std::fs::read_to_string(sb.overrides_file())
        .unwrap()
        .contains("\"topic\""));
    // The AI change itself is the next step back: the AI file topic goes away.
    sb.cli(&["ai", "undo", &id]);
    assert_eq!(e(&sb), None);
    // Then the AI write that created the file.
    sb.cli(&["ai", "undo", &id]);
    assert!(!sb.ai_file(&id).exists());
    let out = sb.cli(&["ai", "undo", &id]);
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stderr).contains("nothing to undo"));
    assert!(sb.log().iter().filter(|l| l.get("undo").is_some()).count() >= 3);
}

// AC 7 -----------------------------------------------------------------------------------------

#[test]
fn an_override_of_a_trashed_note_applies_again_after_restore() {
    let sb = Sb::new();
    let id = sb.add("a note");
    overrides::set(&sb.store(), &id, "project", "web").unwrap();
    sb.store().rm(&id).unwrap();
    assert_eq!(ai::effective(&sb.store(), &id).project, None);
    assert!(
        overrides::problems(&sb.store()).is_empty(),
        "trashed is no orphan"
    );
    sb.store().restore(&id).unwrap();
    assert_eq!(
        ai::effective(&sb.store(), &id).project.as_deref(),
        Some("web")
    );
}

// AC 8 -----------------------------------------------------------------------------------------

#[test]
fn note_files_stay_byte_identical_around_set_undo_and_an_ai_write() {
    let sb = Sb::new();
    let id = sb.add("a note");
    let inbox = sb.notes().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::write(inbox.join("20261125-010101.txt"), "phone note").unwrap();
    let before = snapshot(&sb.notes());
    assert!(before.len() >= 2);
    sb.enrich(&id, "bug");
    sb.topic(&id, "a");
    overrides::set(&sb.store(), &id, "topic", "b").unwrap();
    overrides::undo(&sb.store(), &id).unwrap();
    sb.cli(&["ai", "set", &id, "kind", "draft"]);
    sb.cli(&["ai", "undo", &id]);
    sb.cli(&["ai", "undo", "inbox/20261125-010101"]);
    sb.cli(&["ai", "set", "inbox/20261125-010101", "project", "x"]);
    assert_eq!(snapshot(&sb.notes()), before);
}

// Edge cases -----------------------------------------------------------------------------------

#[test]
fn doctor_names_an_invalid_file_and_an_orphan_and_set_never_overwrites_an_invalid_file() {
    let sb = Sb::new();
    let id = sb.add("a note");
    overrides::set(&sb.store(), &id, "project", "web").unwrap();
    let mut file: Value =
        serde_json::from_str(&std::fs::read_to_string(sb.overrides_file()).unwrap()).unwrap();
    file["notes"]["20200101-000000-abcd"] = json!({"topic": "x"});
    std::fs::write(sb.overrides_file(), file.to_string()).unwrap();
    let out = text(&sb.cli(&["doctor"]).stdout);
    assert!(
        out.contains("orphan override: no note 20200101-000000-abcd"),
        "{out}"
    );
    assert_eq!(
        ai::effective(&sb.store(), &id).project.as_deref(),
        Some("web")
    );

    std::fs::write(sb.overrides_file(), "{ not json").unwrap();
    let out = text(&sb.cli(&["doctor"]).stdout);
    assert!(out.contains("ai-overrides.json is not valid"), "{out}");
    assert!(overrides::set(&sb.store(), &id, "topic", "x").is_err());
    assert_eq!(
        std::fs::read_to_string(sb.overrides_file()).unwrap(),
        "{ not json"
    );
    assert_eq!(ai::effective(&sb.store(), &id).project, None);
}

// Fixes after review -----------------------------------------------------------------------------

#[test]
fn an_invalid_entry_blocks_set_and_undo_and_the_file_bytes_stay() {
    let sb = Sb::new();
    let id = sb.add("a note");
    let other = sb.add("another");
    overrides::set(&sb.store(), &id, "project", "web").unwrap();
    let mut file: Value =
        serde_json::from_str(&std::fs::read_to_string(sb.overrides_file()).unwrap()).unwrap();
    file["notes"][id.as_str()]["topic"] = json!("x".repeat(61));
    std::fs::write(sb.overrides_file(), file.to_string()).unwrap();
    let before = std::fs::read(sb.overrides_file()).unwrap();
    let log_before = sb.log().len();

    let out = sb.cli(&["ai", "set", &other, "topic", "t"]);
    assert_eq!(out.status.code(), Some(1));
    let err = text(&out.stderr);
    assert!(
        err.contains("ai-overrides.json") && err.contains(&format!("{id}.topic")),
        "{err}"
    );
    assert!(!err.contains(&"x".repeat(61)));
    let out = sb.cli(&["ai", "undo", &id]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(std::fs::read(sb.overrides_file()).unwrap(), before);
    assert_eq!(sb.log().len(), log_before);
    // Reads still use the good entry; doctor names the bad one.
    assert_eq!(
        ai::effective(&sb.store(), &id).project.as_deref(),
        Some("web")
    );
    let out = text(&sb.cli(&["doctor"]).stdout);
    assert!(out.contains(&format!("invalid entry {id}.topic")), "{out}");
}

/// Child half of the next test: one load, two `effective_with` calls.
#[test]
fn child_two_effective_with_calls_over_one_load() {
    let Ok(dir) = std::env::var("OVR_CHILD_NOTES") else {
        return;
    };
    let store = Store::system(PathBuf::from(dir));
    let all = overrides::read(&store);
    ai::effective_with(&store, "a", &all);
    ai::effective_with(&store, "b", &all);
}

#[test]
fn two_effective_with_calls_over_one_load_warn_once() {
    let sb = Sb::new();
    sb.add("a note");
    std::fs::create_dir_all(sb.notes().join(".easy-note")).unwrap();
    std::fs::write(sb.overrides_file(), "{ not json").unwrap();
    let out = Command::new(std::env::current_exe().unwrap())
        .args([
            "child_two_effective_with_calls_over_one_load",
            "--exact",
            "--nocapture",
        ])
        .env("OVR_CHILD_NOTES", sb.notes())
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert_eq!(
        err.matches("ai-overrides.json is not valid").count(),
        1,
        "{err}"
    );
}

#[test]
fn a_failed_log_append_after_a_saved_override_is_an_error() {
    let sb = Sb::new();
    let id = sb.add("a note");
    // A directory where the log file goes makes the append fail.
    std::fs::create_dir_all(sb.notes().join(".easy-note/ai-log.jsonl")).unwrap();
    let out = sb.cli(&["ai", "set", &id, "topic", "t"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stderr).contains("ai-log.jsonl could not be written"));
}

// HUN-302: undo of the AI title, summary, tags and draft keys -----------------------------------

impl Sb {
    /// One applied item, as the batch writes it: `enrich`, then `organize`, under one id.
    fn item(&self, id: &str, input: &Input, o: &Organized) {
        ai::apply_item(&self.store(), id, input, o).unwrap();
    }
    fn detail(&self, id: &str) -> Value {
        serde_json::from_str(&std::fs::read_to_string(self.ai_file(id)).unwrap()).unwrap()
    }
    fn note_path(&self, id: &str) -> PathBuf {
        snapshot(&self.notes())
            .into_keys()
            .find(|p| p.file_name().unwrap().to_string_lossy().starts_with(id))
            .unwrap()
    }
}

fn input<'a>(title: &'a str, kind: &'a str, summary: &'a str, tags: &'a [String]) -> Input<'a> {
    Input {
        title,
        kind,
        summary,
        tags_suggested: tags,
        model: "m",
    }
}

#[test]
fn hun302_one_undo_returns_the_whole_earlier_ai_file_after_an_item() {
    let sb = Sb::new();
    let id = sb.add("a note");
    let long = "é".repeat(600);
    let tags = vec!["tray".to_string()];
    let first = input("old title", "other", &long, &tags);
    let organized = Organized {
        project: Some("home"),
        topic: Some("tray"),
        kind: Some("bug"),
        ..Organized::default()
    };
    sb.item(&id, &first, &organized);
    let earlier = sb.detail(&id);
    // The person edits the note: the AI file is stale, and a run replaces it.
    let path = sb.note_path(&id);
    let text = std::fs::read_to_string(&path).unwrap() + "more\n";
    std::fs::write(&path, text).unwrap();
    let notes = snapshot(&sb.notes());
    let new_tags = vec!["usb".to_string()];
    let second = input("new title", "bug", "new summary", &new_tags);
    let moved = Organized {
        project: Some("work"),
        topic: Some("usb"),
        kind: Some("idea"),
        ..Organized::default()
    };
    sb.item(&id, &second, &moved);
    assert_ne!(sb.detail(&id)["based_on"], earlier["based_on"]);

    let done = overrides::undo(&sb.store(), &id).unwrap();
    assert_eq!(
        sb.detail(&id),
        earlier,
        "every key is back, based_on and organized_at too"
    );
    assert_eq!(done.back, overrides::Back::Restored);
    for field in [
        "title",
        "summary",
        "tags_suggested",
        "kind",
        "project",
        "topic",
    ] {
        assert!(done.fields.iter().any(|f| f == field), "{field}: {done:?}");
    }
    assert_eq!(snapshot(&sb.notes()), notes, "no note file changed");
    // The replaced AI file stays readable on the undo line.
    let undo_line = sb
        .log()
        .into_iter()
        .find(|l| l["field"] == "ai_file" && l.get("undo").is_some())
        .unwrap();
    assert!(undo_line["old"].as_str().unwrap().contains("new summary"));
}

#[test]
fn hun302_one_undo_returns_replaced_and_cleared_draft_keys() {
    let sb = Sb::new();
    let id = sb.add("maybe a thing");
    let suggestion = "s".repeat(600);
    let draft = Organized {
        kind: Some("draft"),
        draft_reason: Some("too short"),
        draft_suggestion: Some(&suggestion),
        ..Organized::default()
    };
    sb.item(&id, &input("t", "other", "s", &[]), &draft);
    let earlier = sb.detail(&id);
    let done_draft = Organized {
        kind: Some("todo"),
        clear_draft: true,
        ..Organized::default()
    };
    sb.item(&id, &input("t", "other", "s", &[]), &done_draft);
    assert!(sb.detail(&id).get("draft_reason").is_none());
    overrides::undo(&sb.store(), &id).unwrap();
    assert_eq!(sb.detail(&id), earlier);
    assert_eq!(sb.detail(&id)["draft_suggestion"], suggestion.as_str());
}

#[test]
fn hun302_undo_of_a_new_ai_file_removes_it_and_keeps_the_persons_overrides() {
    let sb = Sb::new();
    std::fs::create_dir_all(sb.notes().join(".obsidian")).unwrap();
    let id = sb.add("a note");
    overrides::set(&sb.store(), &id, "project", "mine").unwrap();
    overrides::set(&sb.store(), &id, "topic", "my topic").unwrap();
    let mine = std::fs::read(sb.overrides_file()).unwrap();
    // Both fields have an override, so the item sets neither.
    let o = Organized::default();
    sb.item(&id, &input("t", "bug", "s", &[]), &o);
    let view = sb.notes().join(format!("ai/{id}.md"));
    assert!(sb.ai_file(&id).exists() && view.exists());

    let done = overrides::undo(&sb.store(), &id).unwrap();
    assert_eq!(done.back, overrides::Back::Removed);
    assert!(!sb.ai_file(&id).exists(), "the AI file is gone");
    assert!(!view.exists(), "its view is gone");
    assert_eq!(std::fs::read(sb.overrides_file()).unwrap(), mine);
    assert_eq!(
        ai::effective(&sb.store(), &id).topic.as_deref(),
        Some("my topic")
    );
    // The next step back is the person's own change.
    let done = overrides::undo(&sb.store(), &id).unwrap();
    assert_eq!(done.fields, ["topic"]);
    assert_eq!(done.back, overrides::Back::Value(None));
}

#[test]
fn hun302_a_second_enrich_that_changes_only_the_kind_is_undone() {
    let sb = Sb::new();
    let id = sb.add("a note");
    sb.enrich(&id, "bug");
    let earlier = sb.detail(&id);
    sb.enrich(&id, "todo");
    overrides::undo(&sb.store(), &id).unwrap();
    assert_eq!(sb.detail(&id), earlier);
    overrides::undo(&sb.store(), &id).unwrap();
    assert!(!sb.ai_file(&id).exists());
    assert!(overrides::undo(&sb.store(), &id).is_err(), "nothing left");
}

#[test]
fn hun302_lines_without_an_op_are_undone_one_at_a_time_and_an_old_undo_line_takes_its_field() {
    let sb = Sb::new();
    let id = sb.add("a note");
    sb.enrich(&id, "bug");
    let line = |field: &str, old: &str, new: &str, undo: bool| {
        let mut l = json!({"id": id, "field": field, "old": old, "new": new,
            "who": if undo { "person" } else { "ai" }, "at": "2026-11-26T10:00:00+07:00"});
        if undo {
            l["undo"] = json!(true);
        }
        l.to_string() + "\n"
    };
    // As written before this change: `ai undo` then took back `kind` only and skipped the rest.
    let log = line("kind", "idea", "bug", false)
        + &line("title", "old title", "t", false)
        + &line("tags_suggested", r#"["tray"]"#, "[]", false)
        + &line("draft_reason", "why", "", false)
        + &line("kind", "bug", "idea", true);
    std::fs::write(sb.notes().join(".easy-note/ai-log.jsonl"), log).unwrap();
    let back: Vec<Vec<String>> = (0..3)
        .map(|_| overrides::undo(&sb.store(), &id).unwrap().fields)
        .collect();
    assert_eq!(
        back,
        [["draft_reason"], ["tags_suggested"], ["title"]].map(|f| f.map(String::from).to_vec())
    );
    let d = sb.detail(&id);
    assert_eq!(d["title"], "old title");
    assert_eq!(d["tags_suggested"], json!(["tray"]));
    assert_eq!(d["draft_reason"], "why");
    assert_eq!(
        d["kind"], "bug",
        "the old undo line already took `kind` back"
    );
    assert!(overrides::undo(&sb.store(), &id).is_err());
}

#[test]
fn hun302_undo_does_not_bring_back_a_removed_ai_file() {
    let sb = Sb::new();
    let id = sb.add("a note");
    sb.enrich(&id, "bug");
    sb.enrich(&id, "todo");
    ai::remove(&sb.store(), &id).unwrap();
    let done = overrides::undo(&sb.store(), &id).unwrap();
    assert_eq!(done.back, overrides::Back::Gone);
    assert!(!sb.ai_file(&id).exists());
}

#[test]
fn hun302_no_ai_file_is_written_when_its_log_line_cannot_be() {
    let sb = Sb::new();
    let id = sb.add("a note");
    std::fs::create_dir_all(sb.notes().join(".easy-note/ai-log.jsonl")).unwrap();
    let err = ai::enrich(&sb.store(), &id, &input("t", "bug", "s", &[]))
        .unwrap_err()
        .to_string();
    assert!(err.contains("ai-log.jsonl could not be written"), "{err}");
    assert!(!sb.ai_file(&id).exists());
}

#[test]
fn hun302_the_cli_says_what_undo_did() {
    let sb = Sb::new();
    let id = sb.add("a note");
    sb.enrich(&id, "bug");
    sb.enrich(&id, "todo");
    let out = text(&sb.cli(&["ai", "undo", &id]).stdout);
    assert!(
        out.contains(&format!("AI detail of {id} is back")) && out.contains("kind"),
        "{out}"
    );
    let out = text(&sb.cli(&["ai", "undo", &id]).stdout);
    assert!(out.contains(&format!("AI detail of {id} removed")), "{out}");
}

#[test]
fn hun302_an_item_whose_ai_file_line_is_unreadable_still_takes_back_all_its_keys() {
    let sb = Sb::new();
    let id = sb.add("a note");
    sb.enrich(&id, "bug");
    let tags = vec!["usb".to_string()];
    sb.item(
        &id,
        &input("new", "bug", "new summary", &tags),
        &Organized::default(),
    );
    // A torn append can break the first line of an item.
    let path = sb.notes().join(".easy-note/ai-log.jsonl");
    let log: String = std::fs::read_to_string(&path)
        .unwrap()
        .lines()
        .filter(|l| !l.contains("\"ai_file\""))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&path, log).unwrap();
    overrides::undo(&sb.store(), &id).unwrap();
    let d = sb.detail(&id);
    assert_eq!(d["title"], "t");
    assert_eq!(d["summary"], "s");
    assert_eq!(d["tags_suggested"], json!([]));
}
