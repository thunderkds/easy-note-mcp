//! One batch of the AI organizer (FR-047, Path B): ONE call to the person's `claude` for up to
//! `batch_size` notes, through the same child as `ai enrich` (FR-037). Each answer item is checked
//! alone and written to AI files only, with the writers of HUN-292: `ai::enrich`, then
//! `ai::organize`. A bad item leaves only its note pending; a failed call leaves the whole batch
//! pending. No note file is opened for writing (NFR-012, NFR-015).

use serde_json::Value;

use super::enrich;
use crate::core::ai::{self, AiError, Input, Organized};
use crate::core::config::{self, AiSettings};
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

/// Write one checked item. The whole item is checked before the first write; the body hash is
/// checked again just before it. A note with an AI file keeps its old kind through `enrich`, so the
/// kind change is made (and logged) by `organize`; a field with an override is passed as `None`.
fn apply(
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
    let mine = all.notes.get(&sent.id);
    let has = |field: &str| mine.is_some_and(|m| m.contains_key(field));
    let kept: Vec<&'static str> = overrides::FIELDS.into_iter().filter(|f| has(f)).collect();
    // A note with no AI file: a draft goes through `enrich` as `other` (AC 10).
    let enrich_kind = ai::read(store, &sent.id).map_or_else(
        || match checked.kind.as_str() {
            "draft" => "other".to_string(),
            kind => kind.to_string(),
        },
        |(old, _)| old.kind,
    );
    ai::enrich(
        store,
        &sent.id,
        &Input {
            title: &checked.title,
            kind: &enrich_kind,
            summary: &checked.summary,
            tags_suggested: &checked.tags_suggested,
            model,
        },
    )
    .map_err(reason)?;
    ai::organize(
        store,
        &sent.id,
        &Organized {
            project: checked.project.as_deref().filter(|_| !has("project")),
            topic: checked.topic.as_deref().filter(|_| !has("topic")),
            kind: (!has("kind")).then_some(checked.kind.as_str()),
            draft_reason: checked.draft_reason.as_deref(),
            draft_suggestion: checked.draft_suggestion.as_deref(),
        },
    )
    .map_err(|e| format!("AI detail written, but organizing failed: {}", reason(e)))?;
    Ok(Written {
        id: sent.id.clone(),
        kept,
    })
}

/// Run one batch over `ids` (at most `batch_size`, chosen by the caller). `Err` is the fixed reason
/// of a batch that wrote nothing: enrich off, no command, or a failed, timed-out, non-JSON or
/// wrong-shape answer. Only with `[ai] enrich = "claude"` (FR-037, NFR-014).
pub fn run_batch(store: &Store, settings: &AiSettings, ids: &[String]) -> Result<Outcome, String> {
    if !settings.enrich {
        let file = config::config_path().map_or_else(
            |_| "config.toml".to_string(),
            |path| path.display().to_string(),
        );
        return Err(format!(
            "AI enrich is off: set [ai] enrich = \"claude\" in {file}"
        ));
    }
    let program = enrich::resolve(&settings.command)
        .ok_or_else(|| format!("claude not found ({})", settings.command))?;
    // Read once (AC 11): one warning for a bad file, not one per note.
    let all = overrides::read(store);
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
    let (projects, topics) = lists(store, &all);
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
    let raw = enrich::call(
        &program,
        settings,
        &organize::schema().to_string(),
        prompt,
        organize::timeout(settings.timeout_secs, sent.len()),
    )?;
    let (items, model) = parse(&raw, &settings.model)?;
    for note in sent {
        // The prompt carried the masked id, so the answer names that one.
        let key = guard::mask(&note.id).0;
        let found: Vec<&Item> = items.iter().filter(|i| i.id == key).collect();
        let result = match found.as_slice() {
            [] => Err("no answer for this note".to_string()),
            [item] => apply(store, note, item, &model, &all),
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
