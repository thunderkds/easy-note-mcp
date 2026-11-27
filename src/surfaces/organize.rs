//! One batch of the AI organizer (FR-047, Path B): ONE call to the person's `claude` for up to
//! `batch_size` notes, through the same child as `ai enrich` (FR-037). Each answer item is checked
//! alone and written to AI files only, with `ai::apply_item` (the writers of HUN-292, under one
//! undo step). A bad item leaves only its note pending; a failed call leaves the whole batch
//! pending. No note file is opened for writing (NFR-012, NFR-015).
//!
//! `run` is `easy-note ai run` (FR-047, FR-049): the batches of one run under `ai-run.lock`, and
//! the state file with its back-off.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, Write};
use std::path::Path;

use chrono::{DateTime, FixedOffset, TimeDelta};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::cli::safe;
use super::enrich;
use crate::core::ai::{self, AiError, Input, ItemError, Organized, WriteLock};
use crate::core::config::{self, AiSettings, Job};
use crate::core::fsutil;
use crate::core::guard;
use crate::core::organize::{self, Item};
use crate::core::overrides::{self, Overrides};
use crate::core::store::{self, Store, StoreError};

/// What one batch did. A note in `pending` keeps no new AI detail from this batch, except when its
/// reason says organizing failed (AC 12): then it has the enrich keys but no new `organized_at`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub written: Vec<Written>,
    pub pending: Vec<Pending>,
    /// Whether `claude` was started: not when no note could be sent.
    pub called: bool,
}

/// A note whose AI file was written; `kept` names the fields a person's override kept (FR-050).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    pub id: String,
    pub kept: Vec<&'static str>,
}

/// A note left pending, with a fixed reason that never quotes the model's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    pub id: String,
    pub reason: String,
}

/// A note as sent: its canonical id, repo, the hash of its body at read time, and the body.
struct Sent {
    id: String,
    repo: Option<String>,
    hash: String,
    body: String,
}

fn reason(err: AiError) -> String {
    match err {
        AiError::Refused(why) => why,
        AiError::Store(StoreError::NotFound(_)) => "no such note".to_string(),
        AiError::Store(_) => "cannot write the AI file".to_string(),
    }
}

fn read_note(store: &Store, id: &str) -> Result<Sent, String> {
    let gone = |e: StoreError| match e {
        StoreError::NotFound(_) | StoreError::BadId(_) => "no such note".to_string(),
        _ => "cannot read the note".to_string(),
    };
    let note = store.read(id).map_err(gone)?;
    let (id, body) = store.raw_body(id).map_err(gone)?;
    Ok(Sent {
        id,
        repo: note.get_str("repo").map(str::to_string),
        hash: ai::based_on(&body),
        body,
    })
}

/// Names with their counts, most used first.
type Names = Vec<(String, usize)>;

/// The project and topic lists of the live notes, as a person sees them (overrides applied).
fn lists(store: &Store, all: &Overrides) -> (Names, Names) {
    let (mut projects, mut topics) = (vec![], vec![]);
    for item in store.iter_notes() {
        let store::Item::Note(note) = item else {
            continue;
        };
        let seen = ai::effective_with(store, note.id(), all);
        projects.extend(seen.project);
        topics.extend(seen.topic);
    }
    (organize::top(projects), organize::top(topics))
}

/// The answer object of the schema: `{"items": [...]}` and nothing else.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Answer {
    items: Vec<Item>,
}

/// The items and the model id in the CLI's JSON output: `structured_output.items` when
/// `structured_output` is an object, else `result` parsed as that object. `Err` is a fixed reason:
/// it never quotes the output.
fn parse(raw: &[u8], fallback: &str) -> Result<(Vec<Item>, String), String> {
    let out: Value = serde_json::from_slice(raw).map_err(|_| "the answer is not JSON")?;
    if out.get("is_error").and_then(Value::as_bool) == Some(true) {
        return Err("claude reported an error".into());
    }
    let answer = match out.get("structured_output") {
        Some(found) if found.is_object() => found.clone(),
        _ => out
            .get("result")
            .and_then(Value::as_str)
            .and_then(|text| serde_json::from_str(text).ok())
            .ok_or("the answer holds no JSON")?,
    };
    let answer: Answer =
        serde_json::from_value(answer).map_err(|_| "the answer has the wrong shape")?;
    Ok((answer.items, enrich::model_of(&out, fallback)))
}

/// Write one checked item, under the write lock. The whole item is checked before the first write;
/// the body hash and the holds are checked again just before it. A note with an AI file keeps its old kind through `enrich`, so the
/// kind change is made (and logged) by `organize`; a field with an override is passed as `None`.
fn apply(
    lock: &WriteLock,
    held: &ai::Holds,
    store: &Store,
    sent: &Sent,
    item: &Item,
    model: &str,
    all: &Overrides,
) -> Result<Written, String> {
    let checked = organize::check_item(item, sent.repo.as_deref())?;
    match store.raw_body(&sent.id) {
        Ok((_, now)) if ai::based_on(&now) == sent.hash => {}
        Ok(_) => return Err("the note changed while AI ran".into()),
        Err(_) => return Err("no such note".into()),
    }
    if held.get(&sent.id) == Some(&sent.hash) {
        return Err("undone while AI ran".into());
    }
    let mine = all.notes.get(&sent.id);
    let has = |field: &str| mine.is_some_and(|m| m.contains_key(field));
    let kept: Vec<&'static str> = overrides::FIELDS.into_iter().filter(|f| has(f)).collect();
    // The kind a person sees: their override, else the model's. Not a draft: no draft keys (F2).
    let shown_kind = mine
        .and_then(|m| m.get("kind"))
        .map_or(checked.kind.as_str(), String::as_str);
    // A note with no AI file: a draft goes through `enrich` as `other` (AC 10).
    let enrich_kind = ai::read(store, &sent.id).map_or_else(
        || match checked.kind.as_str() {
            "draft" => "other".to_string(),
            kind => kind.to_string(),
        },
        |(old, _)| old.kind,
    );
    ai::apply_item_locked(
        lock,
        store,
        &sent.id,
        &Input {
            title: &checked.title,
            kind: &enrich_kind,
            summary: &checked.summary,
            tags_suggested: &checked.tags_suggested,
            model,
        },
        &Organized {
            project: checked.project.as_deref().filter(|_| !has("project")),
            topic: checked.topic.as_deref().filter(|_| !has("topic")),
            kind: (!has("kind")).then_some(checked.kind.as_str()),
            draft_reason: checked.draft_reason.as_deref(),
            draft_suggestion: checked.draft_suggestion.as_deref(),
            clear_draft: shown_kind != "draft",
        },
    )
    .map_err(|e| match e {
        ItemError::Enrich(e) => reason(e),
        ItemError::Organize(e) => {
            format!("AI detail written, but organizing failed: {}", reason(e))
        }
    })?;
    Ok(Written {
        id: sent.id.clone(),
        kept,
    })
}

/// Run one batch over `ids` (at most `batch_size`, chosen by the caller). `Err` is the fixed reason
/// of a batch that wrote nothing: enrich off, no command, or a failed, timed-out, non-JSON or
/// wrong-shape answer. Only with `[ai] enrich = "claude"` (FR-037, NFR-014). `all` is the
/// overrides the caller loaded, for the prompt; they are read again just before the writes.
pub fn run_batch(
    store: &Store,
    settings: &AiSettings,
    ids: &[String],
    all: &Overrides,
) -> Result<Outcome, String> {
    if !settings.enrich {
        return Err(off());
    }
    let program = enrich::resolve(&settings.command)
        .ok_or_else(|| format!("claude not found ({})", settings.command))?;
    let mut out = Outcome::default();
    let mut notes = vec![];
    for id in ids {
        match read_note(store, id) {
            Ok(sent) => notes.push(sent),
            Err(reason) => out.pending.push(Pending {
                id: id.clone(),
                reason,
            }),
        }
    }
    let (projects, topics) = lists(store, all);
    let pairs: Vec<(&str, &str)> = notes
        .iter()
        .map(|n| (n.id.as_str(), n.body.as_str()))
        .collect();
    let (prompt, fit) = organize::prompt(&projects, &topics, &pairs);
    out.pending.extend(notes[fit..].iter().map(|n| Pending {
        id: n.id.clone(),
        reason: "did not fit in the prompt".into(),
    }));
    let sent = &notes[..fit];
    if sent.is_empty() {
        return Ok(out);
    }
    out.called = true;
    let raw = enrich::call(
        &program,
        settings,
        &organize::schema().to_string(),
        prompt,
        organize::timeout(settings.timeout_secs, sent.len()),
    )?;
    let (items, model) = parse(&raw, &settings.model)?;
    // The lock covers the writes only, never the call. A fix or an undo made while the call ran
    // is read here (HUN-295).
    let lock = ai::lock(store).map_err(reason)?;
    let now = overrides::reread(store);
    let held = ai::holds(store).map_err(reason)?;
    for note in sent {
        // The prompt carried the masked id, so the answer names that one.
        let key = guard::mask(&note.id).0;
        let found: Vec<&Item> = items.iter().filter(|i| i.id == key).collect();
        let result = match found.as_slice() {
            [] => Err("no answer for this note".to_string()),
            [item] => apply(&lock, &held, store, note, item, &model, &now),
            _ => Err("answered more than once".to_string()),
        };
        match result {
            Ok(written) => out.written.push(written),
            Err(reason) => out.pending.push(Pending {
                id: note.id.clone(),
                reason,
            }),
        }
    }
    Ok(out)
}

/// A failed run waits at most this long, and a `next_after` further ahead is a clock that moved.
const MAX_WAIT_MIN: i64 = 240;
const MAX_ERROR_CHARS: usize = 200;

/// `.easy-note/ai-run-state.json` (FR-049): the last run, its counts, and the back-off.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RunState {
    pub last_run: Option<String>,
    pub organized: usize,
    pub skipped: usize,
    pub left: usize,
    pub calls: usize,
    /// Runs with a failed call since the last good one.
    pub failures: u32,
    /// When an unattended run may try again; `ai run` by hand does not wait for it.
    pub next_after: Option<String>,
    pub last_error: Option<String>,
}

impl RunState {
    /// The state of the notes dir; empty when there is none or it cannot be read.
    pub fn read(root: &Path) -> RunState {
        fs::read(root.join(".easy-note").join("ai-run-state.json"))
            .ok()
            .and_then(|raw| serde_json::from_slice(&raw).ok())
            .unwrap_or_default()
    }

    fn write(&self, store: &Store) -> Result<(), String> {
        let folder = store.root.join(".easy-note");
        let path = folder.join("ai-run-state.json");
        let mut data = serde_json::to_string(self).expect("a state serializes");
        data.push('\n');
        let written = fs::create_dir_all(&folder).and_then(|()| {
            let (mut file, tmp) = fsutil::create_temp(&folder)?;
            let result = fsutil::write_all_sync(&mut file, data.as_bytes())
                .and_then(|()| fs::rename(&tmp, &path));
            fsutil::unlink_quiet(&tmp);
            result
        });
        written.map_err(|e| format!("cannot write {}: {e}", path.display()))
    }

    /// Whether an unattended run should still wait at `now`.
    pub fn backing_off(&self, now: DateTime<FixedOffset>) -> bool {
        self.next_after
            .as_deref()
            .and_then(|text| DateTime::parse_from_rfc3339(text).ok())
            .is_some_and(|next| now < next && next - now <= TimeDelta::minutes(MAX_WAIT_MIN))
    }

    /// Count a failed run: the wait is `interval_min` doubled once per failure, at most 4 h.
    fn fail(&mut self, why: &str, interval_min: u32, now: &str) {
        self.failures = self.failures.saturating_add(1);
        let wait = i64::from(interval_min)
            .saturating_mul(1 << self.failures.min(16))
            .min(MAX_WAIT_MIN);
        self.next_after = DateTime::parse_from_rfc3339(now)
            .ok()
            .map(|at| (at + TimeDelta::minutes(wait)).to_rfc3339());
        let line = why.lines().next().unwrap_or_default();
        self.last_error = Some(guard::mask(line).0.chars().take(MAX_ERROR_CHARS).collect());
    }

    fn succeed(&mut self) {
        self.failures = 0;
        self.next_after = None;
        self.last_error = None;
    }
}

fn off() -> String {
    let file = config::config_path().map_or_else(
        |_| "config.toml".to_string(),
        |path| path.display().to_string(),
    );
    format!("AI enrich is off: set [ai] enrich = \"claude\" in {file}")
}

fn say(text: &str) {
    let mut out = io::stdout().lock();
    let _ = writeln!(out, "{text}");
    let _ = out.flush();
}

/// One run per notes dir: `.easy-note/ai-run.lock`, held while the returned file lives. `Ok(None)`:
/// another run holds it. The system drops it when the process ends, however it ends.
fn acquire_run_lock(root: &Path) -> Result<Option<File>, String> {
    let folder = root.join(".easy-note");
    let path = folder.join("ai-run.lock");
    let file = fs::create_dir_all(&folder)
        .and_then(|()| {
            OpenOptions::new()
                .create(true)
                .truncate(false)
                .write(true)
                .open(&path)
        })
        .map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    match file.try_lock() {
        Ok(()) => Ok(Some(file)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(e)) => Err(format!("cannot lock {}: {e}", path.display())),
    }
}

fn plural(n: usize, word: &str) -> String {
    format!("{n} {word}{}", if n == 1 { "" } else { "s" })
}

/// `easy-note ai run [--limit N] [--dry-run]` (FR-047, FR-049, NFR-014): organize the notes that
/// wait, newest first, at most `min(limit, max_per_run)`, in batches of `batch_size`, up to the
/// first failed call. `Err` is the text for stderr (exit 1): enrich off, a setup error, or the
/// summary of a run whose every call failed.
pub fn run(limit: Option<usize>, dry_run: bool) -> Result<(), String> {
    let settings = config::ai_settings().map_err(|e| e.to_string())?;
    if !settings.enrich {
        return Err(off());
    }
    if limit == Some(0) {
        return Err("--limit must be at least 1".into());
    }
    if !settings
        .jobs
        .iter()
        .any(|job| matches!(job, Job::Organize | Job::Drafts))
    {
        say("nothing to do: [ai] jobs has neither \"organize\" nor \"drafts\"");
        return Ok(());
    }
    let store = Store::open().map_err(|e| e.to_string())?;
    let max = settings.max_per_run as usize;
    let cap = limit.map_or(max, |limit| limit.min(max));
    let size = settings.batch_size as usize;
    let waiting = || ai::to_organize(&store).map_err(|e| e.to_string());
    if dry_run {
        let count = waiting()?.len().min(cap);
        if count == 0 {
            say("nothing to do");
        } else {
            say(&format!(
                "dry run: {} in {} with model {}; nothing was started or written",
                plural(count, "note"),
                plural(count.div_ceil(size), "call"),
                safe(&settings.model)
            ));
        }
        return Ok(());
    }
    let Some(_running) = acquire_run_lock(&store.root)? else {
        say("already running");
        return Ok(());
    };
    // Read once per run (FR-050).
    let all = overrides::read(&store);
    let mut chosen = waiting()?;
    let before = RunState::read(&store.root);
    let now = store.now_stamp();
    let mut state = RunState {
        last_run: Some(now.clone()),
        failures: before.failures,
        next_after: before.next_after,
        last_error: before.last_error,
        ..RunState::default()
    };
    if chosen.is_empty() {
        // Nothing was asked of `claude`, so the back-off stays as it is.
        state.write(&store)?;
        say("nothing to do");
        return Ok(());
    }
    if enrich::resolve(&settings.command).is_none() {
        let why = format!("claude not found ({})", settings.command);
        state.left = chosen.len();
        state.fail(&why, settings.interval_min, &now);
        state.write(&store)?;
        return Err(format!("failed: {why}"));
    }
    chosen.truncate(cap);
    let (mut good_calls, mut failure) = (0usize, None);
    for batch in chosen.chunks(size) {
        match run_batch(&store, &settings, batch, &all) {
            Ok(out) => {
                if out.called {
                    state.calls += 1;
                    good_calls += 1;
                }
                state.organized += out.written.len();
                state.skipped += out.pending.len();
                for note in &out.pending {
                    eprintln!(
                        "easy-note: {}: skipped: {}",
                        safe(&note.id),
                        safe(&note.reason)
                    );
                }
            }
            Err(why) => {
                state.calls += 1;
                state.skipped += batch.len();
                eprintln!("easy-note: failed: {}", safe(&why));
                failure = Some(why);
                break;
            }
        }
    }
    state.left = waiting()?.len();
    match &failure {
        Some(why) => state.fail(why, settings.interval_min, &now),
        None => state.succeed(),
    }
    state.write(&store)?;
    let summary = format!(
        "organized {}, skipped {}, left {} pending, calls {}",
        state.organized, state.skipped, state.left, state.calls
    );
    if failure.is_some() && good_calls == 0 {
        return Err(summary);
    }
    say(&summary);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backing_off_ends_at_next_after_and_ignores_a_time_more_than_four_hours_ahead() {
        let at = |text: &str| DateTime::parse_from_rfc3339(text).unwrap();
        let now = at("2026-11-27T12:00:00+07:00");
        let state = |next: Option<&str>| RunState {
            next_after: next.map(str::to_string),
            ..RunState::default()
        };
        assert!(state(Some("2026-11-27T12:01:00+07:00")).backing_off(now));
        assert!(state(Some("2026-11-27T16:00:00+07:00")).backing_off(now));
        assert!(!state(Some("2026-11-27T16:00:01+07:00")).backing_off(now));
        assert!(!state(Some("2026-11-27T12:00:00+07:00")).backing_off(now));
        assert!(!state(Some("soon")).backing_off(now));
        assert!(!state(None).backing_off(now));
    }

    #[test]
    fn many_failures_still_wait_four_hours_and_the_error_is_one_masked_line() {
        let mut state = RunState {
            failures: u32::MAX,
            ..RunState::default()
        };
        let token = concat!("gh", "p_EXAMPLEexampleEXAMPLEexampleEXAMPLE123456");
        state.fail(
            &format!("bad {token}\nsecond"),
            u32::MAX,
            "2026-11-27T12:00:00+07:00",
        );
        assert_eq!(
            state.next_after.as_deref(),
            Some("2026-11-27T16:00:00+07:00")
        );
        let error = state.last_error.unwrap();
        assert!(error.starts_with("bad [masked:"), "{error}");
        assert!(!error.contains("second"), "{error}");
    }
}
