//! A person's fixes of project, topic and kind, and the change log (HUN-292, FR-050).
//!
//! Two files under `<notes dir>/.easy-note/`, neither a note:
//! - `ai-overrides.json`: `{"easy-note-overrides": 1, "notes": {"<id>": {"project": "…", "topic": "…",
//!   "kind": "…"}}, "topics": {"<from>": "<to>"}}`. Written whole (temp file + rename). Values are
//!   masked and capped. An override wins over the AI file; the AI file is never changed by it.
//! - `ai-log.jsonl`: one JSON line per change, append only: `id`, `field`, `old`, `new` (`null` is
//!   "no value"), `who` (`ai` or `person`), `at`, `undo: true` on the line an undo writes, and `op`
//!   on the lines of one AI item (HUN-302): `undo` takes the lines of one `op` back together; a
//!   line without `op` is one step. Each write of an AI file first logs an `ai_file` line whose
//!   `old` is the whole earlier file (`null`: there was none), then one line per key that changes,
//!   all in one write; the AI file is not replaced when that write fails. A person's values are cut
//!   to 200 chars; the `old` value of an AI line is whole, so such a line can hold a few KiB.
//!   Each append is one write call on an append-mode file, so a line lands whole.
//!
//! Public API (other tasks build on these names): `read`, `set`, `undo`, `log_change`, `problems`,
//! `Overrides`, `Undone`, `Back`, `FIELDS`; the merged view is `ai::effective`. No function here
//! opens a note file for writing.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::core::ai::{self, AiError, MAX_PROJECT, MAX_TOPIC};
use crate::core::fsutil::{create_temp, unlink_quiet, write_all_sync};
use crate::core::guard;
use crate::core::store::Store;

/// The keys a person can override.
pub const FIELDS: [&str; 3] = ["project", "topic", "kind"];
const VERSION: i64 = 1;
/// A value in a log line is cut to this many chars (not the `old` value of an AI line).
const LOG_VALUE: usize = 200;
/// The field of the line that holds the whole AI file as it was before a write.
const AI_FILE: &str = "ai_file";
/// The keys of an AI file that a log line can name; `undo` takes each of them back.
const AI_FIELDS: [&str; 8] = [
    "title",
    "summary",
    "tags_suggested",
    "kind",
    "project",
    "topic",
    "draft_reason",
    "draft_suggestion",
];

/// The overrides file as read: per-note values and the topic rename map.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Overrides {
    #[serde(rename = "easy-note-overrides", default)]
    pub version: i64,
    #[serde(default)]
    pub notes: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(default)]
    pub topics: BTreeMap<String, String>,
}

/// What `undo` did: the note, the keys it took back and what is in force now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Undone {
    pub id: String,
    pub fields: Vec<String>,
    pub back: Back,
}

/// The state after an undo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Back {
    /// A person's change: the override now in force (`None`: no override).
    Value(Option<String>),
    /// An AI change: the AI file holds the earlier values again.
    Restored,
    /// The AI change had created the AI file: it is removed.
    Removed,
    /// The note has no AI file any more, so nothing was restored.
    Gone,
}

#[derive(Debug, Serialize, Deserialize)]
struct Line {
    id: String,
    field: String,
    old: Option<String>,
    new: Option<String>,
    who: String,
    at: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    undo: bool,
    /// Shared by the lines of one AI item (HUN-302).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    op: Option<String>,
}

fn dir(store: &Store) -> PathBuf {
    store.root.join(".easy-note")
}

fn overrides_path(store: &Store) -> PathBuf {
    dir(store).join("ai-overrides.json")
}

fn log_path(store: &Store) -> PathBuf {
    dir(store).join("ai-log.jsonl")
}

fn mask(text: &str) -> String {
    guard::mask(text).0
}

/// The file, parsed, and the `<id>.<key>` names of the entries dropped as invalid; `Err` when the
/// whole file is not valid. A missing file is no overrides.
fn parse(store: &Store) -> Result<(Overrides, Vec<String>), ()> {
    let raw = match fs::read(overrides_path(store)) {
        Ok(raw) => raw,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok((Overrides::default(), vec![])),
        Err(_) => return Err(()),
    };
    let mut found: Overrides = serde_json::from_slice(&raw).map_err(|_| ())?;
    if found.version != VERSION {
        return Err(());
    }
    // A hand-edited file is masked on read too; an entry out of its caps or lists is dropped.
    let mut dropped = vec![];
    for (id, fields) in found.notes.iter_mut() {
        fields.retain(|key, value| {
            *value = mask(value);
            let ok = check_value(key, value).is_ok();
            if !ok {
                dropped.push(mask(&format!("{id}.{key}")));
            }
            ok
        });
    }
    found.notes.retain(|_, fields| !fields.is_empty());
    for to in found.topics.values_mut() {
        *to = mask(to);
    }
    Ok((found, dropped))
}

/// The overrides, masked. A missing file is no overrides; an invalid one is no overrides too, with
/// one warning line on stderr (`doctor` names it). Never writes.
pub fn read(store: &Store) -> Overrides {
    match parse(store) {
        Ok((found, dropped)) => {
            if !dropped.is_empty() {
                eprintln!("easy-note: ai-overrides.json has invalid entries; ignoring them (run `easy-note doctor`)");
            }
            found
        }
        Err(()) => {
            eprintln!(
                "easy-note: ai-overrides.json is not valid; ignoring it (run `easy-note doctor`)"
            );
            Overrides::default()
        }
    }
}

fn check_value(field: &str, value: &str) -> Result<(), AiError> {
    if value.trim().is_empty() {
        return Err(AiError::Refused(
            "value is empty; give a non-empty value".into(),
        ));
    }
    match field {
        "project" => ai::check_len("project", value, MAX_PROJECT),
        "topic" => ai::check_len("topic", value, MAX_TOPIC),
        "kind" => ai::check_kind(value),
        other => Err(not_a_field(other)),
    }
}

fn not_a_field(field: &str) -> AiError {
    AiError::Refused(format!(
        "field must be one of {} (got {})",
        FIELDS.join(", "),
        mask(field)
    ))
}

fn save(store: &Store, data: &Overrides) -> Result<(), AiError> {
    let folder = dir(store);
    fs::create_dir_all(&folder)?;
    let mut text = serde_json::to_string_pretty(data).expect("overrides serialize");
    text.push('\n');
    let (mut file, tmp) = create_temp(&folder)?;
    let result = write_all_sync(&mut file, text.as_bytes())
        .and_then(|()| (store.rename)(&tmp, &overrides_path(store)));
    unlink_quiet(&tmp);
    result?;
    Ok(())
}

/// The file to change: never an invalid one (it is not overwritten silently).
fn load_for_write(store: &Store) -> Result<Overrides, AiError> {
    let (mut data, dropped) = parse(store).map_err(|()| {
        AiError::Refused(
            "ai-overrides.json is not valid; fix or remove it first (see `easy-note doctor`)"
                .into(),
        )
    })?;
    if !dropped.is_empty() {
        return Err(AiError::Refused(format!(
            "ai-overrides.json has invalid entries ({}); fix or remove them first (see `easy-note doctor`)",
            dropped.join(", ")
        )));
    }
    data.version = VERSION;
    Ok(data)
}

fn cut(text: &str) -> String {
    match text.char_indices().nth(LOG_VALUE) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text.to_string(),
    }
}

fn append(store: &Store, line: &Line) -> io::Result<()> {
    append_all(store, std::slice::from_ref(line))
}

fn append_all(store: &Store, lines: &[Line]) -> io::Result<()> {
    let mut text = String::new();
    for line in lines {
        text.push_str(&serde_json::to_string(line).expect("a log line serializes"));
        text.push('\n');
    }
    fs::create_dir_all(dir(store))?;
    // One write call on an append-mode file: the line lands whole or not at all.
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path(store))
        .and_then(|mut f| f.write_all(text.as_bytes()))
}

fn log_failed(err: &io::Error) -> AiError {
    AiError::Refused(format!(
        "the change was saved, but ai-log.jsonl could not be written: {}",
        err.kind()
    ))
}

/// Append one change to the log. `old` / `new` are masked and cut; `who` is `ai` or `person`.
pub fn log_change(
    store: &Store,
    id: &str,
    field: &str,
    old: Option<&str>,
    new: Option<&str>,
    who: &str,
) {
    let _ = log_line(store, id, field, old, new, who, false);
}

fn log_line(
    store: &Store,
    id: &str,
    field: &str,
    old: Option<&str>,
    new: Option<&str>,
    who: &str,
    undo: bool,
) -> io::Result<()> {
    append(
        store,
        &Line {
            id: mask(id),
            field: field.to_string(),
            old: old.map(|v| cut(&mask(v))),
            new: new.map(|v| cut(&mask(v))),
            who: who.to_string(),
            at: store.now_stamp(),
            undo,
            op: None,
        },
    )
}

/// A new id for the lines of one AI item: unique over processes and under a frozen clock.
pub(crate) fn new_op(store: &Store) -> String {
    static COUNT: AtomicU64 = AtomicU64::new(0);
    format!(
        "{:x}-{:x}-{:x}",
        (store.now_ns)(),
        std::process::id(),
        COUNT.fetch_add(1, Ordering::Relaxed)
    )
}

/// The value of one `AI_FIELDS` key in a detail, as a log line holds it.
fn value_of(detail: &ai::Detail, field: &str) -> Option<String> {
    match field {
        "title" => Some(detail.title.clone()),
        "summary" => Some(detail.summary.clone()),
        "tags_suggested" => {
            Some(serde_json::to_string(&detail.tags_suggested).expect("tags serialize"))
        }
        "kind" => Some(detail.kind.clone()),
        "project" => detail.project.clone(),
        "topic" => detail.topic.clone(),
        "draft_reason" => detail.draft_reason.clone(),
        "draft_suggestion" => detail.draft_suggestion.clone(),
        _ => None,
    }
}

/// Log one write of an AI file before it happens, `who: ai`, in one append: the `ai_file` line with
/// the whole earlier file, then one line per key that differs (`old` whole, `new` cut). `before`
/// comes from a read, so it is masked already. `Err` when the log cannot be written, so the caller
/// writes no AI file.
pub(crate) fn log_ai_write(
    store: &Store,
    id: &str,
    op: &str,
    before: Option<&ai::Detail>,
    after: &ai::Detail,
) -> Result<(), AiError> {
    let line = |field: &str, old: Option<String>, new: Option<String>| Line {
        id: mask(id),
        field: field.to_string(),
        old,
        new,
        who: "ai".to_string(),
        at: store.now_stamp(),
        undo: false,
        op: Some(op.to_string()),
    };
    let whole = before.map(|d| serde_json::to_string(d).expect("a detail serializes"));
    let mut lines = vec![line(AI_FILE, whole, None)];
    if let Some(before) = before {
        for field in AI_FIELDS {
            let (old, new) = (value_of(before, field), value_of(after, field));
            if old != new {
                lines.push(line(
                    field,
                    old.map(|v| mask(&v)),
                    new.map(|v| cut(&mask(&v))),
                ));
            }
        }
    }
    append_all(store, &lines).map_err(|e| not_logged(&e))
}

fn not_logged(err: &io::Error) -> AiError {
    AiError::Refused(format!(
        "ai-log.jsonl could not be written: {}; the AI file was not changed",
        err.kind()
    ))
}

/// A person fixes `project`, `topic` or `kind` of a live note. The value is masked, then capped;
/// the override is written whole and one log line is appended. The AI file is not changed.
pub fn set(store: &Store, note_id: &str, field: &str, value: &str) -> Result<(), AiError> {
    if !FIELDS.contains(&field) {
        return Err(not_a_field(field));
    }
    let value = mask(value.trim());
    check_value(field, &value)?;
    let id = store.canonical_id(note_id)?;
    let mut data = load_for_write(store)?;
    let entry = data.notes.entry(id.clone()).or_default();
    let old = entry.insert(field.to_string(), value.clone());
    save(store, &data)?;
    log_line(
        store,
        &id,
        field,
        old.as_deref(),
        Some(&value),
        "person",
        false,
    )
    .map_err(|e| log_failed(&e))?;
    Ok(())
}

fn undoable(line: &Line) -> bool {
    line.field == AI_FILE || AI_FIELDS.contains(&line.field.as_str())
}

/// The latest step of a note that no later undo took back, oldest line first: all lines of one
/// `op`, else one line. The log is replayed; an undo line with an `op` takes the lines of that `op`
/// and field, one without (a person's undo, and every undo before HUN-302) the latest of its field.
fn latest(store: &Store, id: &str) -> Vec<Line> {
    let Ok(text) = fs::read_to_string(log_path(store)) else {
        return vec![];
    };
    let mut stack: Vec<Line> = vec![];
    for line in text.lines() {
        let Ok(line) = serde_json::from_str::<Line>(line) else {
            continue;
        };
        if line.id != id || !undoable(&line) {
            continue;
        }
        if !line.undo {
            stack.push(line);
        } else if line.op.is_some() {
            stack.retain(|l| l.op != line.op || l.field != line.field);
        } else if let Some(at) = stack.iter().rposition(|l| l.field == line.field) {
            stack.remove(at);
        }
    }
    match stack.pop() {
        Some(top) if top.op.is_some() => {
            stack.retain(|l| l.op == top.op);
            stack.push(top);
            stack
        }
        Some(top) => vec![top],
        None => vec![],
    }
}

/// Undo the latest change of a live note. A person's change puts the override back (removed, or its
/// old value). An AI change puts the AI file back as it was before that item, or removes it when
/// the item had created it; overrides stay. The undo is logged. Refused with `nothing to undo` when
/// the log has no change left for the note.
pub fn undo(store: &Store, note_id: &str) -> Result<Undone, AiError> {
    let id = store.canonical_id(note_id)?;
    let mut step = latest(store, &id);
    match step.last() {
        None => Err(AiError::Refused("nothing to undo".into())),
        Some(top) if top.who == "ai" => undo_ai(store, id, &step),
        Some(_) => {
            let line = step.pop().expect("a step has a line");
            undo_person(store, id, line)
        }
    }
}

fn undo_person(store: &Store, id: String, line: Line) -> Result<Undone, AiError> {
    let back = line.old.as_deref();
    let mut data = load_for_write(store)?;
    let entry = data.notes.entry(id.clone()).or_default();
    match back {
        Some(old) => {
            entry.insert(line.field.clone(), old.to_string());
        }
        None => {
            entry.remove(&line.field);
        }
    }
    if entry.is_empty() {
        data.notes.remove(&id);
    }
    save(store, &data)?;
    log_line(
        store,
        &id,
        &line.field,
        line.new.as_deref(),
        back,
        "person",
        true,
    )
    .map_err(|e| log_failed(&e))?;
    Ok(Undone {
        id,
        fields: vec![line.field],
        back: Back::Value(line.old),
    })
}

/// Take one AI step back. The undo lines are written first, and the `ai_file` one holds the file
/// that is about to be replaced, so no AI text is lost when a write fails.
fn undo_ai(store: &Store, id: String, step: &[Line]) -> Result<Undone, AiError> {
    // The first `ai_file` line of the item holds the state before all of it.
    let whole = step.iter().find(|l| l.field == AI_FILE);
    let plan = match whole {
        Some(line) => ai::plan_restore(store, &id, line.old.as_deref())?,
        // A line from before HUN-302 names one key. Newest first, so the oldest value wins.
        None => ai::plan_fields(
            store,
            &id,
            step.iter()
                .rev()
                .map(|l| (l.field.as_str(), l.old.as_deref())),
        )?,
    };
    let mut replaced = plan.replaced();
    let lines: Vec<Line> = step
        .iter()
        .rev()
        .map(|line| Line {
            id: mask(&id),
            field: line.field.clone(),
            old: if line.field == AI_FILE {
                replaced.take()
            } else {
                line.new.clone()
            },
            new: if line.field == AI_FILE {
                None
            } else {
                line.old.as_deref().map(|v| cut(&mask(v)))
            },
            who: "person".to_string(),
            at: store.now_stamp(),
            undo: true,
            op: line.op.clone(),
        })
        .collect();
    append_all(store, &lines).map_err(|e| not_logged(&e))?;
    let back = ai::apply_restore(store, &id, plan)?;
    let mut fields: Vec<String> = vec![];
    for line in step {
        if line.field != AI_FILE && !fields.contains(&line.field) {
            fields.push(line.field.clone());
        }
    }
    Ok(Undone { id, fields, back })
}

/// Lines for `doctor`: an invalid overrides file, and overrides of notes that are neither live nor
/// in the trash.
pub fn problems(store: &Store) -> Vec<String> {
    let (data, dropped) = match parse(store) {
        Ok(found) => found,
        Err(()) => {
            return vec!["ai-overrides.json is not valid; it is ignored until you fix it".into()]
        }
    };
    let trashed = store.trash_ids();
    let mut lines: Vec<String> = dropped
        .iter()
        .map(|name| format!("ai-overrides.json: invalid entry {name} (ignored; fix or remove it)"))
        .collect();
    lines.extend(
        data.notes
            .keys()
            .filter(|id| store.canonical_id(id).is_err() && !trashed.contains(&id.to_string()))
            .map(|id| {
                format!(
                    "orphan override: no note {} (the override is kept)",
                    mask(id)
                )
            }),
    );
    lines
}
