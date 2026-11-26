//! List, search and stats over the notes dir (FR-010, FR-011, FR-015, FR-016, FR-017).
//!
//! Every call scans the files through `Store::iter_notes`: no index, no ranking, no model (D9,
//! NFR-007). The CLI and the MCP tools call the three entry points and serialize what they return.
//! The same files and the same arguments give the same output: newest `created` instant first,
//! ties by ID.
//!
//! A note's local date is the date in the UTC offset written in its `created` (D10): the same date
//! as its `YYYY/MM` folder and its ID, whatever zone the reader is in.
//!
//! Each entry point has an `_in` twin that takes the `Store`; the plain one opens the store from
//! `EASY_NOTE_DIR`, config.toml or `~/easy-notes`.

use std::collections::{BTreeMap, HashSet};

use caseless::default_case_fold_str as fold;
use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;

use crate::core::ai;
use crate::core::format::{Note, Value};
use crate::core::store::{Item, Skipped, Store, StoreError};

pub const MAX_LIMIT: usize = 50;
pub const DEFAULT_LIMIT: usize = 20;
pub const SNIPPET_MAX: usize = 200;
const ELLIPSIS: char = '…';
/// Chars of context kept before the first hit.
const LEAD: usize = 40;

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("query is empty")]
    EmptyQuery,
    #[error("limit must be a whole number of 1 or more")]
    BadLimit,
    #[error("{name} must be a date like 2026-11-14")]
    BadDate { name: &'static str },
    #[error(transparent)]
    Store(#[from] StoreError),
}

/// Narrowing filters. `None` means no filter. Dates are `YYYY-MM-DD` text, local dates of `created`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filters {
    pub repo: Option<String>,
    pub task: Option<String>,
    /// Matched case-insensitively against whole tags.
    pub tag: Option<String>,
    pub source: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub day: Option<String>,
}

/// One note in a result, without its full text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NoteItem {
    pub id: String,
    pub title: String,
    pub created: String,
    /// When the person last edited the note (FR-046); no key for a note never edited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    pub repo: Option<String>,
    pub task: Option<String>,
    pub tags: Vec<String>,
    pub snippet: String,
    /// `Some("ai")` when the note matched a search only with its AI detail (FR-032).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_in: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Page {
    pub items: Vec<NoteItem>,
    /// Every match, not only the page.
    pub total: usize,
    pub truncated: bool,
}

/// Counts ordered by count (high first), then by name. Serializes as a JSON object in that order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Counts(Vec<(String, usize)>);

impl Counts {
    pub fn as_pairs(&self) -> &[(String, usize)] {
        &self.0
    }

    fn of<'a>(values: impl Iterator<Item = &'a str>) -> Counts {
        let mut map: BTreeMap<&str, usize> = BTreeMap::new();
        for value in values {
            *map.entry(value).or_default() += 1;
        }
        let mut pairs: Vec<(String, usize)> =
            map.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        Counts(pairs)
    }
}

impl Serialize for Counts {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (name, count) in &self.0 {
            map.serialize_entry(name, count)?;
        }
        map.end()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkippedFile {
    pub path: String,
    pub reason: String,
}

/// Counts by repo, task, tag, source and month. Holds no note text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Stats {
    pub total: usize,
    pub by_repo: Counts,
    pub by_task: Counts,
    pub by_tag: Counts,
    pub by_source: Counts,
    pub by_month: Counts,
    pub oldest: Option<String>,
    pub newest: Option<String>,
    pub skipped: Vec<SkippedFile>,
    /// Notes with a fresh AI file, the one count that reads AI files (DDR 0004).
    pub enriched: usize,
}

/// Notes that pass every filter, newest first.
pub fn list_notes(filters: &Filters, limit: usize) -> Result<Page, QueryError> {
    list_notes_in(&Store::open()?, filters, limit)
}

pub fn list_notes_in(store: &Store, filters: &Filters, limit: usize) -> Result<Page, QueryError> {
    check_limit(limit)?;
    let (hits, _) = select(store, &[], filters)?;
    Ok(page(&hits, &[], limit))
}

/// Notes holding every word of `query` (case-insensitive) in title, body or tags, or with the AI
/// detail's title, summary and suggested tags (then `matched_in: "ai"`).
pub fn search(query: &str, filters: &Filters, limit: usize) -> Result<Page, QueryError> {
    search_in(&Store::open()?, query, filters, limit)
}

pub fn search_in(
    store: &Store,
    query: &str,
    filters: &Filters,
    limit: usize,
) -> Result<Page, QueryError> {
    let words: Vec<String> = split_words(query).map(fold).collect();
    if words.is_empty() {
        return Err(QueryError::EmptyQuery);
    }
    check_limit(limit)?;
    let (hits, _) = select(store, &words, filters)?;
    Ok(page(&hits, &words, limit))
}

/// Counts over the notes that pass the filters. A note without a repo (or task) is left out of
/// that count. `skipped` ignores the filters: an unreadable file has no fields to filter on.
pub fn stats(filters: &Filters) -> Result<Stats, QueryError> {
    stats_in(&Store::open()?, filters)
}

pub fn stats_in(store: &Store, filters: &Filters) -> Result<Stats, QueryError> {
    let (hits, skipped) = select(store, &[], filters)?;
    let months: Vec<String> = hits
        .iter()
        .map(|h| h.stamp.date_naive().format("%Y-%m").to_string())
        .collect();
    let dates: Vec<NaiveDate> = hits.iter().map(|h| h.stamp.date_naive()).collect();
    let tags: Vec<&str> = hits
        .iter()
        .flat_map(|h| unique(h.note.fields.get("tags")))
        .collect();
    Ok(Stats {
        total: hits.len(),
        by_repo: Counts::of(hits.iter().filter_map(|h| h.note.get_str("repo"))),
        by_task: Counts::of(hits.iter().filter_map(|h| h.note.get_str("task"))),
        by_tag: Counts::of(tags.into_iter()),
        by_source: Counts::of(hits.iter().filter_map(|h| h.note.get_str("source"))),
        by_month: Counts::of(months.iter().map(String::as_str)),
        oldest: dates.iter().min().map(NaiveDate::to_string),
        newest: dates.iter().max().map(NaiveDate::to_string),
        enriched: hits.iter().filter(|h| h.fresh).count(),
        skipped: skipped
            .into_iter()
            .map(|s| SkippedFile {
                path: s.path.to_string_lossy().into_owned(),
                reason: s.reason,
            })
            .collect(),
    })
}

struct Hit {
    stamp: DateTime<FixedOffset>,
    note: Note,
    /// It has an AI file whose `based_on` is the hash of its body.
    fresh: bool,
    matched_in: Option<&'static str>,
}

fn tags_of(note: &Note) -> &[String] {
    match note.fields.get("tags") {
        Some(Value::List(tags)) => tags,
        _ => &[],
    }
}

/// The tags of one note, each once, in file order.
fn unique(value: Option<&Value>) -> Vec<&str> {
    let mut seen = HashSet::new();
    match value {
        Some(Value::List(tags)) => tags
            .iter()
            .map(String::as_str)
            .filter(|t| seen.insert(*t))
            .collect(),
        _ => vec![],
    }
}

fn select(
    store: &Store,
    words: &[String],
    filters: &Filters,
) -> Result<(Vec<Hit>, Vec<Skipped>), QueryError> {
    let since = as_date(filters.since.as_deref(), "since")?;
    let until = as_date(filters.until.as_deref(), "until")?;
    let day = as_date(filters.day.as_deref(), "day")?;
    let tag = filters.tag.as_deref().map(fold);
    let mut hits = vec![];
    let mut skipped = vec![];
    let details = ai::all(&store.root);
    for (entry, hash) in store.iter_hashed() {
        let note = match entry {
            Item::Skipped(s) => {
                skipped.push(s);
                continue;
            }
            Item::Note(note) => note,
        };
        let Some(stamp) = note
            .get_str("created")
            .and_then(|c| DateTime::parse_from_rfc3339(c).ok())
        else {
            continue;
        };
        let local = stamp.date_naive();
        let differs = |want: &Option<String>, key: &str| {
            want.as_ref().is_some_and(|w| note.get_str(key) != Some(w))
        };
        if differs(&filters.repo, "repo")
            || differs(&filters.task, "task")
            || differs(&filters.source, "source")
            || tag
                .as_ref()
                .is_some_and(|t| !tags_of(&note).iter().any(|x| fold(x) == *t))
            || since.is_some_and(|d| local < d)
            || until.is_some_and(|d| local > d)
            || day.is_some_and(|d| local != d)
        {
            continue;
        }
        let detail = ai::of(&details, note.id());
        let mut matched_in = None;
        if !words.is_empty() {
            let mut haystack = fold(&format!("{}\n{}", note.body, tags_of(&note).join(" ")));
            if !words.iter().all(|w| haystack.contains(w.as_str())) {
                let Some(d) = detail else { continue };
                let extra = format!(
                    "\n{}\n{}\n{}",
                    d.title,
                    d.summary,
                    d.tags_suggested.join(" ")
                );
                haystack.push_str(&fold(&extra));
                if !words.iter().all(|w| haystack.contains(w.as_str())) {
                    continue;
                }
                matched_in = Some("ai");
            }
        }
        let fresh = detail.is_some_and(|d| Some(&d.based_on) == hash.as_ref());
        hits.push(Hit {
            stamp,
            note,
            fresh,
            matched_in,
        });
    }
    hits.sort_by(|a, b| {
        b.stamp
            .timestamp()
            .cmp(&a.stamp.timestamp())
            .then_with(|| {
                b.stamp
                    .timestamp_subsec_nanos()
                    .cmp(&a.stamp.timestamp_subsec_nanos())
            })
            .then_with(|| a.note.id().cmp(b.note.id()))
    });
    Ok((hits, skipped))
}

fn check_limit(limit: usize) -> Result<(), QueryError> {
    if limit < 1 {
        return Err(QueryError::BadLimit);
    }
    Ok(())
}

fn page(hits: &[Hit], words: &[String], limit: usize) -> Page {
    let shown = &hits[..hits.len().min(limit.min(MAX_LIMIT))];
    Page {
        items: shown
            .iter()
            .map(|h| item(&h.note, words, h.matched_in))
            .collect(),
        total: hits.len(),
        truncated: hits.len() > shown.len(),
    }
}

fn item(note: &Note, words: &[String], matched_in: Option<&'static str>) -> NoteItem {
    NoteItem {
        id: note.id().to_string(),
        title: note.title(),
        created: note.get_str("created").unwrap_or("").to_string(),
        updated: note.get_str("updated").map(str::to_string),
        repo: note.get_str("repo").map(str::to_string),
        task: note.get_str("task").map(str::to_string),
        tags: tags_of(note).to_vec(),
        snippet: snippet(&note.body, words),
        matched_in,
    }
}

/// A note's `updated` as a person reads it, in the store's local time (FR-046).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edited {
    /// `just now`, `5m ago`, `2h ago`, `3d ago`; the date and time (`2026-11-19 09:12`) from 30
    /// days on, and when `updated` is later than now or earlier than `created` (a clock set back).
    pub when: String,
    /// The full stamp, `2026-11-19 09:12:00 +07:00`.
    pub stamp: String,
}

/// `None` when `updated` is not a time (the parser never yields such a note).
pub fn edited(store: &Store, created: &str, updated: &str) -> Option<Edited> {
    let updated = DateTime::parse_from_rfc3339(updated).ok()?;
    let secs = updated.timestamp();
    let offset = FixedOffset::east_opt((store.utc_offset)(secs))?;
    let local = updated.with_timezone(&offset);
    let now = i64::try_from((store.now_ns)() / 1_000_000_000).ok()?;
    let set_back = DateTime::parse_from_rfc3339(created).is_ok_and(|created| updated < created);
    let when = ago(now - secs)
        .filter(|_| !set_back)
        .unwrap_or_else(|| local.format("%Y-%m-%d %H:%M").to_string());
    Some(Edited {
        when,
        stamp: local.format("%Y-%m-%d %H:%M:%S %:z").to_string(),
    })
}

/// An age in seconds as words; `None` for a negative age and from 30 days on.
fn ago(secs: i64) -> Option<String> {
    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    if !(0..30 * DAY).contains(&secs) {
        return None;
    }
    Some(match secs {
        s if s < MINUTE => "just now".to_string(),
        s if s < HOUR => format!("{}m ago", s / MINUTE),
        s if s < DAY => format!("{}h ago", s / HOUR),
        s => format!("{}d ago", s / DAY),
    })
}

/// Python's `str.split()` whitespace: Unicode whitespace plus the four ASCII separators.
fn is_space(c: char) -> bool {
    c.is_whitespace() || ('\x1c'..='\x1f').contains(&c)
}

fn split_words(text: &str) -> impl Iterator<Item = &str> {
    text.split(is_space).filter(|w| !w.is_empty())
}

/// One line of at most SNIPPET_MAX chars around the first hit in the body (start of body if none).
fn snippet(body: &str, words: &[String]) -> String {
    let text = split_words(body).collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = text.chars().collect();
    let start = first_hit(&text, &chars, words).map_or(0, |hit| hit.saturating_sub(LEAD));
    let budget = SNIPPET_MAX - usize::from(start > 0);
    let mut out = String::new();
    if start > 0 {
        out.push(ELLIPSIS);
    }
    if chars.len() - start > budget {
        out.extend(&chars[start..start + budget - 1]);
        out.push(ELLIPSIS);
    } else {
        out.extend(&chars[start..]);
    }
    out
}

/// Char index in `text` of the earliest casefolded match of any word (case folding can change a
/// string's length, so the match position is mapped back to the original chars).
fn first_hit(text: &str, chars: &[char], words: &[String]) -> Option<usize> {
    if words.is_empty() {
        return None;
    }
    let folded = fold(text);
    // folded_before[i] = chars of `folded` that come from chars[..i]
    let mut folded_before = Vec::with_capacity(chars.len());
    let mut seen = 0;
    for c in chars {
        folded_before.push(seen);
        seen += fold(&c.to_string()).chars().count();
    }
    words
        .iter()
        .filter_map(|word| {
            let pos = folded[..folded.find(word.as_str())?].chars().count();
            Some(folded_before.partition_point(|&before| before < pos))
        })
        .min()
}

fn as_date(value: Option<&str>, name: &'static str) -> Result<Option<NaiveDate>, QueryError> {
    let Some(text) = value else { return Ok(None) };
    if text.len() != 10 {
        return Err(QueryError::BadDate { name });
    }
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| QueryError::BadDate { name })
}

#[cfg(test)]
mod tests {
    use super::ago;

    #[test]
    fn ago_changes_unit_at_each_boundary() {
        const DAY: i64 = 86_400;
        let table: [(i64, Option<&str>); 12] = [
            (-1, None),
            (0, Some("just now")),
            (59, Some("just now")),
            (60, Some("1m ago")),
            (59 * 60 + 59, Some("59m ago")),
            (3_600, Some("1h ago")),
            (23 * 3_600 + 3_599, Some("23h ago")),
            (DAY, Some("1d ago")),
            (29 * DAY, Some("29d ago")),
            (30 * DAY - 1, Some("29d ago")),
            (30 * DAY, None),
            (400 * DAY, None),
        ];
        for (secs, want) in table {
            assert_eq!(ago(secs).as_deref(), want, "{secs} s");
        }
    }
}
