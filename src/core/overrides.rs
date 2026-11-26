//! A person's fixes of project, topic and kind, and the change log (HUN-292, FR-050).
//!
//! Two files under `<notes dir>/.easy-note/`, neither a note:
//! - `ai-overrides.json`: `{"easy-note-overrides": 1, "notes": {"<id>": {"project": "…", "topic": "…",
//!   "kind": "…"}}, "topics": {"<from>": "<to>"}}`. Written whole (temp file + rename). Values are
//!   masked and capped. An override wins over the AI file; the AI file is never changed by it.
//! - `ai-log.jsonl`: one JSON line per change, append only: `id`, `field`, `old`, `new` (`null` is
//!   "no value"), `who` (`ai` or `person`), `at`, and `undo: true` on the line an undo writes.
//!   Values in a line are cut to 200 chars, except the `old` value of `title`, `summary`,
//!   `tags_suggested` (logged by `ai::enrich`) and `draft_reason`, `draft_suggestion` (logged by
//!   `ai::organize`): kept whole for undo (HUN-302) and written before the AI file, which is not
//!   replaced when the line cannot be written. Such a line can hold a 600-char text, a few KiB.
//!   Each append is one write call on an append-mode file, so a line lands whole. `undo` skips
//!   those five fields (they are not in `FIELDS`).
//!
//! Public API (other tasks build on these names): `read`, `set`, `undo`, `log_change`,
//! `log_ai_diff`, `problems`, `Overrides`, `Undone`, `FIELDS`; the merged view is
//! `ai::effective`. No function here opens a note file for writing.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::ai::{self, AiError, MAX_PROJECT, MAX_TOPIC};
use crate::core::fsutil::{create_temp, unlink_quiet, write_all_sync};
use crate::core::guard;
use crate::core::store::Store;

/// The keys a person can override.
pub const FIELDS: [&str; 3] = ["project", "topic", "kind"];
const VERSION: i64 = 1;
/// A value in a log line is cut to this many chars.
const LOG_VALUE: usize = 200;

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

/// What `undo` did: the note, the key and the value now in force (`None`: no override).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Undone {
    pub id: String,
    pub field: String,
    pub now: Option<String>,
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
        },
    )
}

/// Log each of `project`, `topic` and `kind` that differs between two AI files, `who: ai`. The
/// draft keys are logged before the write by `log_draft_diff`.
pub(crate) fn log_ai_diff(store: &Store, id: &str, before: &ai::Detail, after: &ai::Detail) {
    let pairs = [
        (
            "project",
            before.project.as_deref(),
            after.project.as_deref(),
        ),
        ("topic", before.topic.as_deref(), after.topic.as_deref()),
        (
            "kind",
            Some(before.kind.as_str()),
            Some(after.kind.as_str()),
        ),
    ];
    for (field, old, new) in pairs {
        if old != new {
            log_change(store, id, field, old, new, "ai");
        }
    }
}

/// Log each of `title`, `summary` and `tags_suggested` (as JSON text) that differs between two AI
/// files, `who: ai`, with the old value whole and the new one cut. All lines go in one write; `Err`
/// when it fails, so the caller writes no AI file.
pub(crate) fn log_enrich_diff(
    store: &Store,
    id: &str,
    before: &ai::Detail,
    after: &ai::Detail,
) -> Result<(), AiError> {
    let tags = |d: &ai::Detail| serde_json::to_string(&d.tags_suggested).expect("tags serialize");
    log_whole(
        store,
        id,
        [
            (
                "title",
                Some(before.title.clone()),
                Some(after.title.clone()),
            ),
            (
                "summary",
                Some(before.summary.clone()),
                Some(after.summary.clone()),
            ),
            ("tags_suggested", Some(tags(before)), Some(tags(after))),
        ],
    )
}

/// Log each of `draft_reason` and `draft_suggestion` that differs between two AI files (set,
/// replaced or removed), `who: ai`, the old value whole, in one write before the AI file is
/// written; `Err` when it fails, so the caller writes no AI file.
pub(crate) fn log_draft_diff(
    store: &Store,
    id: &str,
    before: &ai::Detail,
    after: &ai::Detail,
) -> Result<(), AiError> {
    log_whole(
        store,
        id,
        [
            (
                "draft_reason",
                before.draft_reason.clone(),
                after.draft_reason.clone(),
            ),
            (
                "draft_suggestion",
                before.draft_suggestion.clone(),
                after.draft_suggestion.clone(),
            ),
        ],
    )
}

/// One line per pair that differs: `old` masked and whole, `new` masked and cut, all in one write.
fn log_whole<const N: usize>(
    store: &Store,
    id: &str,
    pairs: [(&str, Option<String>, Option<String>); N],
) -> Result<(), AiError> {
    let lines: Vec<Line> = pairs
        .into_iter()
        .filter(|(_, old, new)| old != new)
        .map(|(field, old, new)| Line {
            id: mask(id),
            field: field.to_string(),
            old: old.map(|v| mask(&v)),
            new: new.map(|v| cut(&mask(&v))),
            who: "ai".to_string(),
            at: store.now_stamp(),
            undo: false,
        })
        .collect();
    if lines.is_empty() {
        return Ok(());
    }
    append_all(store, &lines).map_err(|e| {
        AiError::Refused(format!(
            "ai-log.jsonl could not be written: {}; the AI file was not changed",
            e.kind()
        ))
    })
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
    FIELDS.contains(&line.field.as_str())
}

/// The latest change of a note that no later undo took back: replay the log, an undo line pops.
fn latest(store: &Store, id: &str) -> Option<Line> {
    let text = fs::read_to_string(log_path(store)).ok()?;
    let mut stack: Vec<Line> = vec![];
    for line in text.lines() {
        let Ok(line) = serde_json::from_str::<Line>(line) else {
            continue;
        };
        if line.id != id || !undoable(&line) {
            continue;
        }
        if line.undo {
            stack.pop();
        } else {
            stack.push(line);
        }
    }
    stack.pop()
}

/// Undo the latest change of a live note: a person's change puts the override back (removed, or
/// its old value); an AI change puts the old value back in the AI file. The undo is logged.
/// Refused with `nothing to undo` when the log has no change left for the note.
pub fn undo(store: &Store, note_id: &str) -> Result<Undone, AiError> {
    let id = store.canonical_id(note_id)?;
    let Some(line) = latest(store, &id) else {
        return Err(AiError::Refused("nothing to undo".into()));
    };
    let back = line.old.as_deref();
    if line.who == "ai" {
        ai::restore_field(store, &id, &line.field, back)?;
    } else {
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
    }
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
        field: line.field,
        now: line.old,
    })
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
