//! The window's IPC API (FR-042, DDR 0009 item 3): one JSON request in, one JSON response out,
//! over a `Store`. No window, no display; only the actions and `settings_save` below write,
//! through the core (`enrich` also reads config.toml for `[ai]`, as the CLI does). A page
//! bug can send anything, so every command and id is checked and a bad request is an
//! `{"error": ...}` object, never a panic.
//! Every string in every response passes `guard::mask_for_output` (NFR-002).
//!
//! - `{"cmd": "board"}` → `{notes_dir, today, total, projects, top_projects, notes}`; a note item
//!   has no full text, only `search`: the masked body's distinct words, so the page's search
//!   matches the same words as before (HUN-282)
//! - `{"cmd": "detail", "id": "<note id>"}` → one note with `masked`, `context` and `ai`
//! - a note item and a detail of an edited note also have `updated` and `edited` (`2h ago`), and the
//!   detail `edited_at`, the full local stamp (FR-046); a note never edited has none of the three
//!
//! Actions (FR-043), each through the same core calls as its CLI command, nothing else writes:
//! - `{"cmd": "accept_ai", "id"}` → the updated `detail` (`easy-note tag <id> --accept-ai`)
//! - `{"cmd": "trash", "id"}` → `{"trashed": id}` (`easy-note rm`, the AI file moves too)
//! - `{"cmd": "restore", "id"}` → `{"restored": id}` (`easy-note restore`)
//! - `{"cmd": "edit", "id", "text", "tags": [..], "base", "base_tags": [..]}` → the updated `detail` (`easy-note
//!   edit`, HUN-286): `base` is the `detail` text the page loaded; when the note's text is not
//!   `base` now (a trailing newline aside), nothing is written and the error is `the note changed
//!   outside the window`. Tags follow `easy-note tag` (no `#`, each once; a tag the note already
//!   has is kept as it is); a changed text adds its `#words` as `easy-note edit` does. Empty text
//!   is `empty`; no change writes nothing. `masked` is also true when the guard masked on this save
//!
//! - `{"cmd": "enrich", "id"}` → the updated `detail`, its AI detail filled in (HUN-288, `Run AI`): the
//!   same per-note step as `easy-note ai enrich <id>` (`enrich::for_window`). It waits for the
//!   person's `claude` (seconds), so the shell answers it off the event-loop thread. The click is
//!   the opt-in: no `[ai]` section is needed. Errors are words (`claude` not found, not logged in,
//!   timed out, bad answer, `[ai]` config, no such note) and nothing is written
//!
//! No command opens a note in another app (HUN-286: `open_in_editor` is gone).
//!
//! Settings (FR-048, FR-052, HUN-300): `[ai]` of config.toml, read and written by the core.
//! - `{"cmd": "settings_get"}` → `{path, base, settings: {the `config::SCREEN_KEYS`}, command,
//!   account, notice: {template, confirmed_model}}`: `command` and `account` are for reading only;
//!   `confirmed_model` is the model when the notice for the saved account and model was confirmed
//! - `{"cmd": "settings_save", "base", "settings": {every screen key}, "confirm_notice": bool}` →
//!   the new `settings_get`. Nothing is written when `base` is not the file now (`the settings
//!   changed outside the window`), when a value breaks a rule of `[ai]` (the key is named), or
//!   when the save turns scheduled runs on and the notice for that account and model is neither
//!   confirmed now (`confirm_notice: true`) nor before (NFR-014). `command` and
//!   `claude_config_dir` are never written
//!
//! Capture (FR-044), the same rule as `easy-note quick` (FR-027) and the same core call as
//! `easy-note add`:
//! - `{"cmd": "context"}` → `{"context": {repo, task} | null}`: the context of the newest `cli` or
//!   `hook` note made less than two hours ago; null when there is none or it has no repo and no task
//! - `{"cmd": "add", "text", "keep_context": bool}` → the new note's `detail`. The text is trimmed;
//!   empty text is `{"error": "empty"}` and writes nothing. `source` is `quick`; the context is the
//!   one `context` names (with its remote and branch) when `keep_context` is true, else none. A
//!   missing `keep_context` is false. Tags come from `#words` as in `add`; a secret is masked on write
//!
//! A note's day is the date in the UTC offset written in its `created`, as in `query`; "today"
//! is the store's clock in the store's zone.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use serde_json::{json, Map, Value};

use crate::core::context::Context;
use crate::core::format::{Note, Value as Field};
use crate::core::query::{self, Edited};
use crate::core::store::{Item, Store, StoreError};
use crate::core::{ai, ai_view, config, guard, tags};
use crate::surfaces::enrich;
use crate::surfaces::quick::last_context;

/// How many projects the counts strip shows as bars.
const TOP: usize = 3;
/// Context keys of the detail panel, each a string or null.
const CONTEXT: [&str; 3] = ["branch", "commit", "session"];

/// Answer one IPC request.
pub fn handle(store: &Store, request: &str) -> String {
    handle_with(store, request, &|_| {})
}

/// `handle`, and `on_added` gets the id of each note `add` saved, after the save. The shell passes
/// `enrich::after_save` there when `[ai] enrich` and `after_save` are on.
pub fn handle_with(store: &Store, request: &str, on_added: &dyn Fn(&str)) -> String {
    let mut answer = answer(store, request, on_added).unwrap_or_else(|e| json!({ "error": e }));
    mask_all(&mut answer);
    answer.to_string()
}

fn answer(store: &Store, request: &str, on_added: &dyn Fn(&str)) -> Result<Value, String> {
    let request: Value = serde_json::from_str(request).map_err(|_| "bad request")?;
    let id = || {
        request
            .get("id")
            .and_then(Value::as_str)
            .ok_or("missing id")
    };
    let cmd = request.get("cmd").and_then(Value::as_str);
    match cmd {
        Some("enrich") => return enrich_note(store, id()?),
        Some("settings_get") => return settings_get(),
        Some("settings_save") => return settings_save(&request),
        _ => {}
    }
    let local: Result<Value, &'static str> = match cmd {
        Some("board") => Ok(board(store)),
        Some("detail") => detail(store, id()?),
        Some("accept_ai") => accept_ai(store, id()?),
        Some("trash") => trash(store, id()?),
        Some("restore") => restore(store, id()?),
        Some("edit") => edit(store, id()?, &request),
        Some("context") => Ok(json!({ "context": context(store) })),
        Some("add") => add(store, &request, on_added),
        Some(_) => Err("unknown command"),
        None => Err("missing cmd"),
    };
    Ok(local?)
}

/// The page's short text for a store error; never the id or a path.
fn store_error(e: StoreError) -> &'static str {
    match e {
        StoreError::NotFound(_) => "no such note",
        StoreError::BadId(_) => "bad id",
        StoreError::ReadOnly => "inbox notes are read-only",
        StoreError::Exists(_) => "a file of that name is in the way",
        StoreError::TextTooLong(_) => "the text is too long",
        _ => "cannot use the note",
    }
}

/// `tag <id> --accept-ai` (`cli::write::tag`): the valid suggested tags are added after the
/// note's own, each once; the text is not touched.
fn accept_ai(store: &Store, id: &str) -> Result<Value, &'static str> {
    let note = store.read(id).map_err(store_error)?;
    store.ensure_writable(note.id()).map_err(store_error)?;
    let (ai_detail, _) = ai::read(store, note.id()).ok_or("no AI detail")?;
    let mut now: Vec<String> = match note.fields.get("tags") {
        Some(Field::List(tags)) => tags.clone(),
        _ => vec![],
    };
    for tag in ai_detail
        .tags_suggested
        .iter()
        .filter_map(|t| tags::normalize(t))
    {
        if !now.contains(&tag) {
            now.push(tag);
        }
    }
    store.update(id, None, Some(&now)).map_err(store_error)?;
    ai_view::sync(store, id);
    detail(store, id)
}

/// `ai enrich <id>` (`enrich::for_window`), then the note's `detail`. The error is the words of the
/// failed step, not a fixed `&'static str`.
fn enrich_note(store: &Store, id: &str) -> Result<Value, String> {
    let done = enrich::for_window(store, id)?;
    detail(store, &done).map_err(String::from)
}

/// The account folder as the page shows it, and as the notice names it.
fn shown_account(settings: &config::AiSettings) -> String {
    guard::mask_for_output(&config::account_folder(settings))
}

/// The version of the notice that `settings` would show.
fn notice_of(settings: &config::AiSettings) -> String {
    config::notice_version(&config::schedule_notice(
        &shown_account(settings),
        &settings.model,
    ))
}

/// What the Settings screen shows, with the stamp its save is checked against.
fn settings_get() -> Result<Value, String> {
    let (settings, base) = config::ai_settings_stamped().map_err(|e| e.to_string())?;
    let path = config::config_path().map_err(|e| e.to_string())?;
    let confirmed = settings.notice_seen == Some(notice_of(&settings));
    let jobs: Vec<&str> = settings.jobs.iter().map(|job| job.name()).collect();
    Ok(json!({
        "path": path.to_string_lossy(),
        "base": base,
        "settings": {
            "enrich": if settings.enrich { "claude" } else { "off" },
            "schedule": match settings.schedule {
                config::Schedule::Off => "off",
                config::Schedule::Tray => "tray",
            },
            "interval_min": settings.interval_min,
            "max_per_run": settings.max_per_run,
            "batch_size": settings.batch_size,
            "jobs": jobs,
            "model": settings.model,
            "after_save": settings.after_save,
        },
        "command": settings.command,
        "account": shown_account(&settings),
        "notice": {
            "template": config::SCHEDULE_NOTICE,
            "confirmed_model": confirmed.then_some(&settings.model),
        },
    }))
}

/// Save the screen's values through `config::save_ai_settings_if`. The save that turns scheduled
/// runs on needs the person's confirm of the notice, now or before (NFR-014).
fn settings_save(request: &Value) -> Result<Value, String> {
    let base = request
        .get("base")
        .and_then(Value::as_str)
        .ok_or("missing base")?;
    let given = request
        .get("settings")
        .and_then(Value::as_object)
        .ok_or("missing settings")?;
    let (current, now) = config::ai_settings_stamped().map_err(|e| e.to_string())?;
    if now != base {
        return Err(config::CHANGED_OUTSIDE.to_string());
    }
    // A value TOML has no form for (null, a too large number) goes in as text: the rule of its
    // key then refuses it, naming the key.
    let shown: toml::Table = given
        .iter()
        .map(|(key, value)| {
            let value =
                toml::Value::try_from(value).unwrap_or_else(|_| toml::Value::String(String::new()));
            (key.clone(), value)
        })
        .collect();
    let mut wanted = config::ai_settings_shown(&current, &shown).map_err(|e| e.to_string())?;
    if wanted.effective_schedule() && !current.effective_schedule() {
        let version = notice_of(&wanted);
        if request.get("confirm_notice").and_then(Value::as_bool) == Some(true) {
            wanted.notice_seen = Some(version);
        } else if wanted.notice_seen != Some(version) {
            return Err("confirm the notice before Organize automatically turns on".to_string());
        }
    }
    config::save_ai_settings_if(&wanted, base).map_err(|e| e.to_string())?;
    settings_get()
}

/// `rm <id>` (`cli::write::rm`).
fn trash(store: &Store, id: &str) -> Result<Value, &'static str> {
    let live = store.canonical_id(id).ok();
    store.rm_with_ai(id).map_err(store_error)?;
    if let Some(live) = live {
        ai_view::sync_remove(store, &live);
    }
    Ok(json!({ "trashed": id }))
}

/// `restore <id>` (`cli::write::restore`).
fn restore(store: &Store, id: &str) -> Result<Value, &'static str> {
    store.restore_with_ai(id).map_err(store_error)?;
    if let Ok(live) = store.canonical_id(id) {
        ai_view::sync(store, &live);
    }
    Ok(json!({ "restored": id }))
}

/// `edit <id>` (`cli::write::edit`) with the text and tags from the page, guarded by `base`.
fn edit(store: &Store, id: &str, request: &Value) -> Result<Value, &'static str> {
    let text = request
        .get("text")
        .and_then(Value::as_str)
        .ok_or("missing text")?;
    let wanted: Vec<&str> = request
        .get("tags")
        .and_then(Value::as_array)
        .and_then(|tags| tags.iter().map(Value::as_str).collect())
        .ok_or("missing tags")?;
    let base = request
        .get("base")
        .and_then(Value::as_str)
        .ok_or("missing base")?;
    let base_tags: Vec<&str> = request
        .get("base_tags")
        .and_then(Value::as_array)
        .and_then(|tags| tags.iter().map(Value::as_str).collect())
        .ok_or("missing base")?;
    if text.trim().is_empty() {
        return Err("empty");
    }
    let note = store.read(id).map_err(store_error)?;
    store.ensure_writable(note.id()).map_err(store_error)?;
    let same = |a: &str, b: &str| a.trim_end_matches('\n') == b.trim_end_matches('\n');
    let shown = guard::mask_for_output(&note.body);
    let before: Vec<String> = match note.fields.get("tags") {
        Some(Field::List(tags)) => tags.clone(),
        _ => vec![],
    };
    // The tags count as part of the base: a tag change made in another app is not overwritten.
    if !same(&shown, base) || before != base_tags {
        return Err("the note changed outside the window");
    }
    let mut now: Vec<String> = vec![];
    for tag in wanted {
        // The page sends each tag as `#tag`; a tag the note has is kept as stored (`easy-note edit`).
        let bare = tag.strip_prefix('#').unwrap_or(tag);
        let tag = match before.iter().find(|t| *t == bare) {
            Some(kept) => kept.clone(),
            None => tags::normalize(tag).ok_or("not a tag")?,
        };
        if !now.contains(&tag) {
            now.push(tag);
        }
    }
    let changed = (!same(&shown, text)).then_some(text);
    if changed.is_some() {
        for tag in tags::extract(text) {
            if !now.contains(&tag) {
                now.push(tag);
            }
        }
    }
    let new_tags = (now != before).then_some(now);
    if changed.is_none() && new_tags.is_none() {
        return detail(store, id);
    }
    let updated = store
        .update(id, changed, new_tags.as_deref())
        .map_err(store_error)?;
    ai_view::sync(store, id);
    let mut out = detail(store, id)?;
    if !updated.masked.is_empty() {
        out["masked"] = json!(true);
    }
    Ok(out)
}

/// The last context for the dialog's chip: repo and task, or null when it holds neither.
fn context(store: &Store) -> Value {
    match last_context(&store.root, now(store)) {
        Some(c) if c.repo.is_some() || c.task.is_some() => {
            json!({ "repo": c.repo, "task": c.task })
        }
        _ => Value::Null,
    }
}

/// `easy-note add` with the `quick` context rule: one new note, `source: quick`.
fn add(store: &Store, request: &Value, on_added: &dyn Fn(&str)) -> Result<Value, &'static str> {
    let text = request
        .get("text")
        .and_then(Value::as_str)
        .ok_or("missing text")?
        .trim();
    if text.is_empty() {
        return Err("empty");
    }
    let keep = request.get("keep_context").and_then(Value::as_bool) == Some(true);
    let context = if keep {
        last_context(&store.root, now(store)).unwrap_or_default()
    } else {
        Context::default()
    };
    let added = store
        .add(text, "quick", &tags::extract(text), &context.fields())
        .map_err(store_error)?;
    on_added(&added.id);
    detail(store, &added.id)
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
        let mut row = item(&note, stamp, today, mark);
        add_edited(store, &note, &mut row);
        row.insert("search".into(), json!(search_words(&note.body)));
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
    if let Some(edited) = add_edited(store, &note, &mut out) {
        out.insert("edited_at".into(), json!(edited.stamp));
    }
    out.insert("text".into(), json!(note.body));
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

/// The board's search field: the distinct whitespace-separated words of the masked body, in first
/// seen order. A search word has no whitespace, so it lies inside one word of the body: matching
/// it against this field gives the same answer as matching it against the whole body.
fn search_words(body: &str) -> String {
    let masked = guard::mask_for_output(body);
    let mut seen = HashSet::new();
    let words: Vec<&str> = masked
        .split_whitespace()
        .filter(|w| seen.insert(*w))
        .collect();
    words.join(" ")
}

/// One row of the board and of the detail (the detail adds the text). `mark` is the AI kind and stale flag when the note has AI detail.
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

/// `updated` and `edited` of a note that has `updated` (FR-046).
fn add_edited(store: &Store, note: &Note, out: &mut Map<String, Value>) -> Option<Edited> {
    let updated = note.get_str("updated")?;
    let edited = query::edited(store, note.get_str("created")?, updated)?;
    out.insert("updated".into(), json!(updated));
    out.insert("edited".into(), json!(edited.when));
    Some(edited)
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
    now(store).date_naive()
}

/// The store's clock in the store's zone, to the second.
fn now(store: &Store) -> DateTime<FixedOffset> {
    let secs = i64::try_from((store.now_ns)() / 1_000_000_000).unwrap_or(0);
    let offset = FixedOffset::east_opt((store.utc_offset)(secs))
        .unwrap_or_else(|| FixedOffset::east_opt(0).expect("zero offset"));
    DateTime::<Utc>::from_timestamp(secs, 0)
        .unwrap_or_default()
        .with_timezone(&offset)
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
