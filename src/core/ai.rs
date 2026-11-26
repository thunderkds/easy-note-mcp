//! AI detail files (FR-029–FR-032, DDR 0004): `<notes dir>/.easy-note/ai/<id>.json`, the
//! `easy-note-ai: 1` shape of `docs/NOTE_FORMAT.md`. Not a note: this module never opens a note
//! file for writing. easy-note runs no model; the text comes from the person's own Claude session
//! through `note_enrich`, from their own `claude` CLI through `ai enrich` (DDR 0007), or from
//! `note_add(detail)` (`model: capture`).
//!
//! Every string is masked on write, and again on read after the JSON is decoded, because a `\u`
//! escape hides a token from the raw patterns. A file that is not readable, not JSON, of the wrong
//! shape or over a cap is no AI file: the note is pending, and no read path fails on it.
//!
//! Writing an AI file is the one overwrite in easy-note: a temp file in `.easy-note/ai/`, fsynced,
//! then renamed over the old file, so a re-run replaces stale detail.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::format::{Note, MAX_TITLE};
use crate::core::fsutil::{create_temp, unlink_quiet, write_all_sync};
use crate::core::guard;
use crate::core::store::{Item, Store, StoreError};
use crate::core::tags;

pub const VERSION: i64 = 1;
/// The kinds `note_enrich` accepts (its tool schema is frozen).
pub const KINDS: [&str; 5] = ["bug", "idea", "question", "todo", "other"];
/// The kinds an AI file may hold and a person may set: `KINDS` plus `draft` (round 6).
pub const ORGANIZER_KINDS: [&str; 6] = ["bug", "idea", "question", "todo", "draft", "other"];
pub const MAX_PROJECT: usize = 60;
pub const MAX_TOPIC: usize = 60;
pub const MAX_DRAFT_REASON: usize = 200;
pub const MAX_DRAFT_SUGGESTION: usize = 600;
pub const MAX_AI_TITLE: usize = MAX_TITLE;
pub const MAX_SUMMARY: usize = 600;
pub const MAX_SUGGESTED: usize = 5;
/// Most notes one `note_pending_enrich` call lists; also its default.
pub const MAX_PENDING: usize = 20;
/// Every read shows the detail under this label (DDR 0004).
pub const LABEL: &str = "AI detail (not your words)";
/// The `model` of the AI file `note_add(detail)` writes.
pub const CAPTURE_MODEL: &str = "capture";
const INBOX_PREFIX: &str = "inbox/";

/// One AI file. Unknown keys are ignored on read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Detail {
    #[serde(rename = "easy-note-ai")]
    pub version: i64,
    pub id: String,
    pub title: String,
    pub kind: String,
    pub summary: String,
    pub tags_suggested: Vec<String>,
    pub based_on: String,
    pub model: String,
    pub at: String,
    /// Round 6 keys, all optional: a file without them still reads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft_suggestion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organized_at: Option<String>,
}

/// What `organize` gives to an AI file: `None` leaves that key as it is. `organized_at` is set by
/// the server. `clear_draft` removes `draft_reason` and `draft_suggestion` (the caller sets it when
/// the effective kind is not `draft`); it wins over the two draft values.
#[derive(Debug, Default, Clone)]
pub struct Organized<'a> {
    pub project: Option<&'a str>,
    pub topic: Option<&'a str>,
    pub kind: Option<&'a str>,
    pub draft_reason: Option<&'a str>,
    pub draft_suggestion: Option<&'a str>,
    pub clear_draft: bool,
}

/// What a caller gives `enrich`. `based_on` and `at` are set by the server, never given.
pub struct Input<'a> {
    pub title: &'a str,
    pub kind: &'a str,
    pub summary: &'a str,
    pub tags_suggested: &'a [String],
    pub model: &'a str,
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    /// A field over its limit or out of its list. Names the field and the limit, never the value.
    #[error("{0}")]
    Refused(String),
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl From<io::Error> for AiError {
    fn from(err: io::Error) -> AiError {
        AiError::Store(err.into())
    }
}

/// One note that needs AI detail, its strings masked. Absent keys are left out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Pending {
    pub id: String,
    pub text: String,
    pub created: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PendingPage {
    pub items: Vec<Pending>,
    pub total: usize,
    pub truncated: bool,
}

/// `sha256:<hex>` of a note body exactly as stored in its file.
pub fn based_on(body: &str) -> String {
    let hex: String = Sha256::digest(body.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("sha256:{hex}")
}

/// The AI file name of a resolved note id: `inbox/<stem>` → `inbox__<stem>.json`, else `<id>.json`.
pub(crate) fn file_name(note_id: &str) -> String {
    match note_id.strip_prefix(INBOX_PREFIX) {
        Some(part) => format!("inbox__{part}.json"),
        None => format!("{note_id}.json"),
    }
}

fn dir(root: &Path) -> PathBuf {
    root.join(".easy-note").join("ai")
}

/// The AI file of a resolved note id.
pub(crate) fn path(root: &Path, note_id: &str) -> PathBuf {
    dir(root).join(file_name(note_id))
}

/// Where the AI file of a trashed note waits: `trash/ai/<note trash file name>.json`.
pub(crate) fn trash_path(root: &Path, note_trash_name: &str) -> PathBuf {
    root.join(".easy-note")
        .join("trash")
        .join("ai")
        .join(format!("{note_trash_name}.json"))
}

/// Refuse a text over `limit` chars with a message that names the field and the limit.
pub fn check_len(field: &str, text: &str, limit: usize) -> Result<(), AiError> {
    let chars = text.chars().count();
    if chars > limit {
        return Err(AiError::Refused(format!(
            "{field} is {chars} chars; the limit is {limit}"
        )));
    }
    Ok(())
}

fn refuse_tag(index: usize) -> AiError {
    AiError::Refused(format!(
        "tags_suggested[{index}] is not a tag (use a-z, 0-9, _, / and -)"
    ))
}

/// Refuse a kind that is not in `ORGANIZER_KINDS`; the message names the allowed values.
pub fn check_kind(kind: &str) -> Result<(), AiError> {
    if !ORGANIZER_KINDS.contains(&kind) {
        return Err(AiError::Refused(format!(
            "kind must be one of {}",
            ORGANIZER_KINDS.join(", ")
        )));
    }
    Ok(())
}

/// The checks on a detail, the same on write and on read: caps, kind, tags.
fn check(detail: &Detail) -> Result<(), AiError> {
    check_len("title", &detail.title, MAX_AI_TITLE)?;
    check_len("summary", &detail.summary, MAX_SUMMARY)?;
    for (field, text, limit) in [
        ("project", &detail.project, MAX_PROJECT),
        ("topic", &detail.topic, MAX_TOPIC),
        ("draft_reason", &detail.draft_reason, MAX_DRAFT_REASON),
        (
            "draft_suggestion",
            &detail.draft_suggestion,
            MAX_DRAFT_SUGGESTION,
        ),
    ] {
        if let Some(text) = text {
            check_len(field, text, limit)?;
        }
    }
    check_kind(&detail.kind)?;
    let count = detail.tags_suggested.len();
    if count > MAX_SUGGESTED {
        return Err(AiError::Refused(format!(
            "tags_suggested has {count} tags; the limit is {MAX_SUGGESTED}"
        )));
    }
    for (index, tag) in detail.tags_suggested.iter().enumerate() {
        if tags::normalize(tag).as_deref() != Some(tag.as_str()) {
            return Err(refuse_tag(index));
        }
    }
    Ok(())
}

fn mask(text: &str) -> String {
    guard::mask(text).0
}

/// Mask every string of a decoded detail.
fn mask_detail(detail: &mut Detail) {
    for text in [
        &mut detail.id,
        &mut detail.title,
        &mut detail.kind,
        &mut detail.summary,
        &mut detail.based_on,
        &mut detail.model,
        &mut detail.at,
    ] {
        *text = mask(text);
    }
    for tag in &mut detail.tags_suggested {
        *tag = mask(tag);
    }
    for text in [
        &mut detail.project,
        &mut detail.topic,
        &mut detail.draft_reason,
        &mut detail.draft_suggestion,
        &mut detail.organized_at,
    ]
    .into_iter()
    .flatten()
    {
        *text = mask(text);
    }
}

/// Write the AI file of one note (`note_enrich`), replacing an old one. Caps are checked on the
/// masked text, as stored; nothing is written on a refusal or an unknown id.
pub fn enrich(store: &Store, note_id: &str, input: &Input) -> Result<Detail, AiError> {
    let count = input.tags_suggested.len();
    if count > MAX_SUGGESTED {
        return Err(AiError::Refused(format!(
            "tags_suggested has {count} tags; the limit is {MAX_SUGGESTED}"
        )));
    }
    let mut suggested: Vec<String> = vec![];
    for (index, tag) in input.tags_suggested.iter().enumerate() {
        let tag = mask(&tags::normalize(tag).ok_or_else(|| refuse_tag(index))?);
        if !suggested.contains(&tag) {
            suggested.push(tag);
        }
    }
    let mut detail = Detail {
        version: VERSION,
        id: String::new(),
        title: mask(input.title),
        kind: input.kind.to_string(),
        summary: mask(input.summary),
        tags_suggested: suggested,
        based_on: String::new(),
        model: mask(input.model),
        at: String::new(),
        project: None,
        topic: None,
        draft_reason: None,
        draft_suggestion: None,
        organized_at: None,
    };
    check(&detail)?;
    let (id, body) = store.raw_body(note_id)?;
    // A re-run of `enrich` keeps what the organizer wrote. It logs each of title, summary and tags
    // it changes first (option A, HUN-294); no AI file is replaced when that log cannot be written.
    if let Some(old) = load(&path(&store.root, &id)) {
        crate::core::overrides::log_enrich_diff(store, &id, &old, &detail)?;
        detail.project = old.project;
        detail.topic = old.topic;
        detail.draft_reason = old.draft_reason;
        detail.draft_suggestion = old.draft_suggestion;
        detail.organized_at = old.organized_at;
    }
    detail.based_on = based_on(&body);
    detail.at = store.now_stamp();
    detail.id = id;
    write(store, &detail)?;
    crate::core::ai_view::sync(store, &detail.id);
    Ok(detail)
}

/// Write the organizer keys into the AI file of a note that already has a valid one (HUN-292).
/// Masks, checks caps and kind, sets `organized_at`, logs each changed key with `who: ai`
/// (`overrides::log_change`), and never touches the note. Refused when the note has no AI file.
pub fn organize(store: &Store, note_id: &str, input: &Organized) -> Result<Detail, AiError> {
    let (id, _) = store.raw_body(note_id)?;
    let Some(mut detail) = load(&path(&store.root, &id)) else {
        return Err(AiError::Refused(format!(
            "no AI detail for {id} to organize"
        )));
    };
    let before = detail.clone();
    let put = |slot: &mut Option<String>, new: Option<&str>| {
        if let Some(new) = new {
            *slot = Some(mask(new));
        }
    };
    put(&mut detail.project, input.project);
    put(&mut detail.topic, input.topic);
    if input.clear_draft {
        detail.draft_reason = None;
        detail.draft_suggestion = None;
    } else {
        put(&mut detail.draft_reason, input.draft_reason);
        put(&mut detail.draft_suggestion, input.draft_suggestion);
    }
    if let Some(kind) = input.kind {
        detail.kind = mask(kind);
    }
    check(&detail)?;
    detail.organized_at = Some(store.now_stamp());
    write(store, &detail)?;
    crate::core::ai_view::sync(store, &detail.id);
    crate::core::overrides::log_ai_diff(store, &id, &before, &detail);
    Ok(detail)
}

/// Put one of `project`, `topic`, `kind` back to `old` in the AI file (undo of an AI change).
/// `None` removes `project` / `topic`; a `kind` has no empty state, so `None` leaves it. A note
/// with no AI file is left as it is.
pub(crate) fn restore_field(
    store: &Store,
    id: &str,
    field: &str,
    old: Option<&str>,
) -> Result<(), AiError> {
    let Some(mut detail) = load(&path(&store.root, id)) else {
        return Ok(());
    };
    match (field, old) {
        ("project", old) => detail.project = old.map(mask),
        ("topic", old) => detail.topic = old.map(mask),
        ("kind", Some(old)) => detail.kind = mask(old),
        _ => return Ok(()),
    }
    check(&detail)?;
    write(store, &detail)?;
    crate::core::ai_view::sync(store, id);
    Ok(())
}

/// What a person sees for a note: the AI value of each key with the person's override applied,
/// and the topic rename map applied to the topic. All `None` for a note that is not live.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Effective {
    pub project: Option<String>,
    pub topic: Option<String>,
    pub kind: Option<String>,
    pub draft_reason: Option<String>,
    pub draft_suggestion: Option<String>,
}

/// The effective values of one note (FR-050). An override of a note that is not live (trashed or
/// gone) is kept in its file and ignored here; it applies again after `restore`.
pub fn effective(store: &Store, note_id: &str) -> Effective {
    effective_with(store, note_id, &crate::core::overrides::read(store))
}

/// `effective` over overrides loaded once with `overrides::read`: a caller that loops over notes
/// reads and parses the file once, and an invalid file warns once.
pub fn effective_with(
    store: &Store,
    note_id: &str,
    all: &crate::core::overrides::Overrides,
) -> Effective {
    let Ok(id) = store.canonical_id(note_id) else {
        return Effective::default();
    };
    let detail = load(&path(&store.root, &id));
    let mine = all.notes.get(&id);
    let pick = |field: &str, ai: Option<&String>| -> Option<String> {
        mine.and_then(|m| m.get(field)).or(ai).cloned()
    };
    let topic = pick("topic", detail.as_ref().and_then(|d| d.topic.as_ref()))
        .map(|t| all.topics.get(&t).cloned().unwrap_or(t));
    Effective {
        project: pick("project", detail.as_ref().and_then(|d| d.project.as_ref())),
        topic,
        kind: pick("kind", detail.as_ref().map(|d| &d.kind)),
        draft_reason: detail.as_ref().and_then(|d| d.draft_reason.clone()),
        draft_suggestion: detail.as_ref().and_then(|d| d.draft_suggestion.clone()),
    }
}

/// The AI file of `note_add(detail)`: the note's title, kind `other`, no tags, model `capture`.
pub fn capture(store: &Store, note_id: &str, title: &str, detail: &str) -> Result<Detail, AiError> {
    let input = Input {
        title,
        kind: "other",
        summary: detail,
        tags_suggested: &[],
        model: CAPTURE_MODEL,
    };
    enrich(store, note_id, &input)
}

fn write(store: &Store, detail: &Detail) -> Result<(), AiError> {
    let folder = dir(&store.root);
    fs::create_dir_all(&folder)?;
    let mut data = serde_json::to_string(detail).expect("a detail serializes");
    data.push('\n');
    let (mut file, tmp) = create_temp(&folder)?;
    let result = write_all_sync(&mut file, data.as_bytes())
        .and_then(|()| (store.rename)(&tmp, &folder.join(file_name(&detail.id))));
    unlink_quiet(&tmp);
    result?;
    Ok(())
}

/// One AI file, checked and masked; `None` when it is missing or not a valid AI file.
fn load(path: &Path) -> Option<Detail> {
    let raw = fs::read(path).ok()?;
    let mut detail: Detail = serde_json::from_slice(&raw).ok()?;
    if detail.version != VERSION || check(&detail).is_err() {
        return None;
    }
    mask_detail(&mut detail);
    Some(detail)
}

/// The AI detail of a note, masked, and whether it is stale (`based_on` is not the hash of the
/// note body as stored). `None`: no valid AI file, or no such note.
pub fn read(store: &Store, note_id: &str) -> Option<(Detail, bool)> {
    let (id, body) = store.raw_body(note_id).ok()?;
    let detail = load(&path(&store.root, &id))?;
    let stale = detail.based_on != based_on(&body);
    Some((detail, stale))
}

/// Every valid AI file in `.easy-note/ai/`, masked, by file name. Read once per query.
pub(crate) fn all(root: &Path) -> HashMap<String, Detail> {
    let Ok(entries) = fs::read_dir(dir(root)) else {
        return HashMap::new();
    };
    entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            if name.starts_with('.') || !name.ends_with(".json") {
                return None;
            }
            Some((name, load(&e.path())?))
        })
        .collect()
}

/// The detail of one note in a map from `all`.
pub(crate) fn of<'a>(details: &'a HashMap<String, Detail>, note_id: &str) -> Option<&'a Detail> {
    details.get(&file_name(note_id))
}

/// Delete the AI file of a note (`ai rm`); `Ok(false)` when it has none. The note is untouched.
pub fn remove(store: &Store, note_id: &str) -> Result<bool, AiError> {
    let id = store.canonical_id(note_id)?;
    match fs::remove_file(path(&store.root, &id)) {
        Ok(()) => {
            crate::core::ai_view::sync_remove(store, &id);
            Ok(true)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

/// The notes (dated and inbox, not trashed) with no AI file or a stale one, newest first, at most
/// `limit` (1 to MAX_PENDING). Writes nothing.
pub fn pending(store: &Store, limit: usize) -> Result<PendingPage, AiError> {
    if !(1..=MAX_PENDING).contains(&limit) {
        return Err(AiError::Refused(format!(
            "limit must be 1 to {MAX_PENDING}"
        )));
    }
    let details = all(&store.root);
    let mut found: Vec<(DateTime<FixedOffset>, Note)> = vec![];
    for (item, hash) in store.iter_hashed() {
        let Item::Note(note) = item else { continue };
        if of(&details, note.id()).is_some_and(|d| Some(&d.based_on) == hash.as_ref()) {
            continue;
        }
        let Some(stamp) = note
            .get_str("created")
            .and_then(|c| DateTime::parse_from_rfc3339(c).ok())
        else {
            continue;
        };
        found.push((stamp, note));
    }
    found.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.id().cmp(b.1.id())));
    let total = found.len();
    let items = found
        .into_iter()
        .take(limit)
        .map(|(_, note)| {
            let field = |key: &str| note.get_str(key).map(str::to_string);
            Pending {
                id: note.id().to_string(),
                created: field("created").unwrap_or_default(),
                repo: field("repo"),
                task: field("task"),
                session: field("session"),
                text: note.body.clone(),
            }
        })
        .collect::<Vec<_>>();
    Ok(PendingPage {
        truncated: total > items.len(),
        items,
        total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = concat!("gh", "p_EXAMPLEexampleEXAMPLEexampleEXAMPLE123456");

    #[test]
    fn based_on_is_sha256_hex_of_the_body() {
        assert_eq!(
            based_on(""),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn file_names_of_dated_and_inbox_ids() {
        assert_eq!(
            file_name("20261114-164713-a1b2"),
            "20261114-164713-a1b2.json"
        );
        assert_eq!(file_name("inbox/phone"), "inbox__phone.json");
        assert_eq!(
            trash_path(Path::new("/n"), "x.md"),
            Path::new("/n/.easy-note/trash/ai/x.md.json")
        );
    }

    #[test]
    fn caps_refuse_with_field_and_limit() {
        let err = check_len("title", &"x".repeat(121), MAX_AI_TITLE).unwrap_err();
        assert_eq!(err.to_string(), "title is 121 chars; the limit is 120");
        assert!(check_len("title", &"é".repeat(120), MAX_AI_TITLE).is_ok());
    }

    fn sample() -> Detail {
        Detail {
            version: 1,
            id: "i".into(),
            title: "t".into(),
            kind: "bug".into(),
            summary: "s".into(),
            tags_suggested: vec!["a".into()],
            based_on: based_on("b"),
            model: "m".into(),
            at: "a".into(),
            project: None,
            topic: None,
            draft_reason: None,
            draft_suggestion: None,
            organized_at: None,
        }
    }

    #[test]
    fn check_takes_a_valid_detail_and_refuses_bad_kind_and_tags() {
        assert!(check(&sample()).is_ok());
        let mut bad = sample();
        bad.kind = "rant".into();
        assert!(check(&bad).is_err());
        let mut bad = sample();
        bad.tags_suggested = vec!["Not A Tag".into()];
        assert_eq!(
            check(&bad).unwrap_err().to_string(),
            "tags_suggested[0] is not a tag (use a-z, 0-9, _, / and -)"
        );
    }

    #[test]
    fn decoded_text_is_masked_on_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.json");
        let mut value = serde_json::to_value(sample()).unwrap();
        value["summary"] = "see TOKEN".into();
        let escaped = format!("\\u0067{}", &TOKEN[1..]);
        let raw = value.to_string().replace("TOKEN", &escaped);
        assert!(!raw.contains(TOKEN));
        fs::write(&path, raw).unwrap();
        let got = load(&path).unwrap();
        assert!(got.summary.starts_with("see [masked:"), "{}", got.summary);
        fs::write(&path, "{\"easy-note-ai\": 1}").unwrap();
        assert!(load(&path).is_none());
    }
}
