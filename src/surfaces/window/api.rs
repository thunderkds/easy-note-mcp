//! The window's IPC API (FR-042, DDR 0009 item 3): one JSON request in, one JSON response out,
//! over a `Store`. Pure: no window, no display, no write. A page bug can send anything, so every
//! command and id is checked and a bad request is an `{"error": ...}` object, never a panic.
//! Every string in every response passes `guard::mask_for_output` (NFR-002).
//!
//! - `{"cmd": "board"}` → `{notes_dir, today, total, projects, top_projects, notes}`
//! - `{"cmd": "detail", "id": "<note id>"}` → one note with `masked`, `context` and `ai`
//!
//! A note's day is the date in the UTC offset written in its `created`, as in `query`; "today"
//! is the store's clock in the store's zone.

use std::cmp::Reverse;
use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use serde_json::{json, Map, Value};

use crate::core::format::{Note, Value as Field};
use crate::core::store::{Item, Store, StoreError};
use crate::core::{ai, guard};

/// How many projects the counts strip shows as bars.
const TOP: usize = 3;
/// Context keys of the detail panel, each a string or null.
const CONTEXT: [&str; 3] = ["branch", "commit", "session"];

/// Answer one IPC request.
pub fn handle(store: &Store, request: &str) -> String {
    let mut answer = answer(store, request).unwrap_or_else(|e| json!({ "error": e }));
    mask_all(&mut answer);
    answer.to_string()
}

fn answer(store: &Store, request: &str) -> Result<Value, &'static str> {
    let request: Value = serde_json::from_str(request).map_err(|_| "bad request")?;
    match request.get("cmd").and_then(Value::as_str) {
        Some("board") => Ok(board(store)),
        Some("detail") => {
            let id = request
                .get("id")
                .and_then(Value::as_str)
                .ok_or("missing id")?;
            detail(store, id)
        }
        Some(_) => Err("unknown command"),
        None => Err("missing cmd"),
    }
}

fn board(store: &Store) -> Value {
    let today = today(store);
    let details = ai::all(&store.root);
    let mut rows = vec![];
    for (entry, hash) in store.iter_hashed() {
        let Item::Note(note) = entry else { continue };
        let Some(stamp) = created(&note) else {
            continue;
        };
        let mark = ai::of(&details, note.id())
            .map(|d| (d.kind.as_str(), Some(&d.based_on) != hash.as_ref()));
        let row = item(&note, stamp, today, mark);
        rows.push((stamp, note.id().to_string(), row));
    }
    rows.sort_by(|a, b| {
        b.0.timestamp()
            .cmp(&a.0.timestamp())
            .then_with(|| {
                b.0.timestamp_subsec_nanos()
                    .cmp(&a.0.timestamp_subsec_nanos())
            })
            .then_with(|| a.1.cmp(&b.1))
    });

    let mut repos: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, _, row) in &rows {
        if let Some(repo) = row["repo"].as_str().filter(|r| !r.is_empty()) {
            *repos.entry(repo).or_default() += 1;
        }
    }
    let mut top: Vec<(&str, usize)> = repos.iter().map(|(r, n)| (*r, *n)).collect();
    top.sort_by_key(|&(name, n)| (Reverse(n), name));
    let top_projects: Vec<Value> = top
        .iter()
        .take(TOP)
        .map(|(name, n)| json!({ "name": name, "n": n }))
        .collect();
    let projects = repos.len();
    let today_count = rows.iter().filter(|r| r.2["day"] == "today").count();
    let total = rows.len();
    let notes: Vec<Value> = rows.into_iter().map(|r| Value::Object(r.2)).collect();
    json!({
        "notes_dir": store.root.to_string_lossy(),
        "today": today_count,
        "total": total,
        "projects": projects,
        "top_projects": top_projects,
        "notes": notes,
    })
}

fn detail(store: &Store, id: &str) -> Result<Value, &'static str> {
    let note = store.read(id).map_err(|e| match e {
        StoreError::NotFound(_) => "no such note",
        StoreError::BadId(_) => "bad id",
        _ => "cannot read the note",
    })?;
    let stamp = created(&note).ok_or("cannot read the note")?;
    let detail = ai::read(store, id);
    let mark = detail.as_ref().map(|(d, stale)| (d.kind.as_str(), *stale));
    let mut out = item(&note, stamp, today(store), mark);
    out.insert("masked".into(), json!(note.body.contains("[masked:")));
    let context: Map<String, Value> = CONTEXT
        .iter()
        .map(|&k| {
            let v = note.get_str(k).filter(|v| !v.is_empty());
            (k.to_string(), json!(v))
        })
        .collect();
    out.insert("context".into(), Value::Object(context));
    let ai = match detail {
        Some((d, stale)) => json!({
            "title": d.title,
            "kind": design_kind(&d.kind),
            "summary": d.summary,
            "tags_suggested": d.tags_suggested,
            "model": d.model,
            "at": d.at,
            "stale": stale,
        }),
        None => Value::Null,
    };
    out.insert("ai".into(), ai);
    Ok(Value::Object(out))
}

/// One row of the board. `mark` is the AI kind and stale flag when the note has AI detail.
fn item(
    note: &Note,
    stamp: DateTime<FixedOffset>,
    today: NaiveDate,
    mark: Option<(&str, bool)>,
) -> Map<String, Value> {
    let date = stamp.date_naive();
    let day = if date == today {
        "today"
    } else if today.pred_opt() == Some(date) {
        "yesterday"
    } else {
        "earlier"
    };
    let tags: &[String] = match note.fields.get("tags") {
        Some(Field::List(tags)) => tags,
        _ => &[],
    };
    let json = json!({
        "id": note.id(),
        "created": note.get_str("created"),
        "time": stamp.format("%H:%M").to_string(),
        "day": day,
        "first": note.title(),
        "text": note.body,
        "repo": note.get_str("repo"),
        "task": note.get_str("task"),
        "tags": tags,
        "source": note.get_str("source"),
        "kind": mark.map(|(kind, _)| design_kind(kind)),
        "ai": mark.is_some(),
        "stale": mark.is_some_and(|(_, stale)| stale),
    });
    match json {
        Value::Object(map) => map,
        _ => Map::new(),
    }
}

/// The design spells the core kind `todo` as `to-do`.
fn design_kind(kind: &str) -> &str {
    match kind {
        "todo" => "to-do",
        other => other,
    }
}

fn created(note: &Note) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(note.get_str("created")?).ok()
}

/// The store's "now" as a local date in the store's zone.
fn today(store: &Store) -> NaiveDate {
    let secs = i64::try_from((store.now_ns)() / 1_000_000_000).unwrap_or(0);
    let offset = FixedOffset::east_opt((store.utc_offset)(secs))
        .unwrap_or_else(|| FixedOffset::east_opt(0).expect("zero offset"));
    DateTime::<Utc>::from_timestamp(secs, 0)
        .unwrap_or_default()
        .with_timezone(&offset)
        .date_naive()
}

/// Mask every string in the answer: ids, paths and errors as well as the note text.
fn mask_all(value: &mut Value) {
    match value {
        Value::String(s) => *s = guard::mask_for_output(s),
        Value::Array(items) => items.iter_mut().for_each(mask_all),
        Value::Object(map) => map.values_mut().for_each(mask_all),
        _ => {}
    }
}
