//! Parse and render one note in the `easy-note: 1` format (docs/NOTE_FORMAT.md, DDR 0002).
//!
//! `render(parse(f)) == f` for every file this module rendered: known keys are written in a fixed
//! order with every string double-quoted, so YAML never turns `no`, `on` or a date into another
//! type. Unknown keys (for example ones Obsidian adds) are kept as their original YAML source text.

use std::collections::BTreeMap;
use std::path::PathBuf;

use chrono::{DateTime, NaiveDateTime};
use saphyr::{LoadableYamlNode, ScalarOwned, YamlOwned};
use saphyr_parser::{Event, Parser};

pub const VERSION: i64 = 1;
pub const MAX_TEXT: usize = 20_000;
pub const MAX_TITLE: usize = 120;
pub const SOURCES: [&str; 5] = ["cli", "mcp", "hook", "quick", "inbox"];
const STRING_KEYS: [&str; 11] = [
    "id", "created", "updated", "repo", "remote", "branch", "task", "source", "client", "session",
    "commit",
];
pub const KEY_ORDER: [&str; 13] = [
    "easy-note",
    "id",
    "created",
    "updated",
    "repo",
    "remote",
    "branch",
    "task",
    "tags",
    "source",
    "client",
    "session",
    "commit",
];
const REQUIRED: [&str; 5] = ["easy-note", "id", "created", "tags", "source"];

/// The file is not a readable `easy-note: 1` note. The message never quotes file content.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{reason}")]
pub struct FormatError {
    pub reason: String,
}

fn bad(reason: impl Into<String>) -> FormatError {
    FormatError {
        reason: reason.into(),
    }
}

/// Note text over MAX_TEXT chars: the only refusal on write (FR-002).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("note text is {} chars; the limit is {}", thousands(*.chars), thousands(MAX_TEXT))]
pub struct TextTooLong {
    pub chars: usize,
}

fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A value of a known front-matter key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Str(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    /// Known keys only, validated.
    pub fields: BTreeMap<String, Value>,
    /// Everything after the closing `---` line, unchanged.
    pub body: String,
    /// Unknown keys, original YAML text, in file order.
    pub extra: Vec<String>,
    /// Set by the store; never rendered.
    pub path: Option<PathBuf>,
}

impl Note {
    pub fn new(fields: BTreeMap<String, Value>, body: String) -> Note {
        Note {
            fields,
            body,
            extra: vec![],
            path: None,
        }
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        match self.fields.get(key) {
            Some(Value::Str(s)) => Some(s),
            _ => None,
        }
    }

    /// The `id` key, or "" when the note has none.
    pub fn id(&self) -> &str {
        self.get_str("id").unwrap_or("")
    }

    /// The first line of the body, cut to MAX_TITLE chars.
    pub fn title(&self) -> String {
        let first = self.body.split('\n').next().unwrap_or("");
        first
            .trim_end_matches('\r')
            .chars()
            .take(MAX_TITLE)
            .collect()
    }
}

pub fn check_text(text: &str) -> Result<(), TextTooLong> {
    let chars = text.chars().count();
    if chars > MAX_TEXT {
        return Err(TextTooLong { chars });
    }
    Ok(())
}

/// Split `---\n<front>---\n<body>` into (front, body). `\r\n` line ends are accepted.
fn split_front(text: &str) -> Option<(&str, &str)> {
    let rest = text
        .strip_prefix("---\r\n")
        .or_else(|| text.strip_prefix("---\n"))?;
    let start = text.len() - rest.len();
    let mut pos = start;
    while pos <= text.len() {
        let line_end = text[pos..].find('\n').map_or(text.len(), |i| pos + i);
        let line = &text[pos..line_end];
        if line == "---" || line == "---\r" {
            let body_start = (line_end + 1).min(text.len());
            return Some((&text[start..pos], &text[body_start..]));
        }
        if line_end == text.len() {
            return None;
        }
        pos = line_end + 1;
    }
    None
}

pub fn parse(text: &str) -> Result<Note, FormatError> {
    let (front, body) = split_front(text).ok_or_else(|| bad("no front matter"))?;
    let docs = YamlOwned::load_from_str(front).map_err(|e| {
        // The file line: the front matter starts on line 2 of the file.
        bad(format!(
            "front matter is not valid YAML (line {})",
            e.marker().line() + 1
        ))
    })?;
    let map = match docs.first() {
        Some(YamlOwned::Mapping(map)) => map,
        _ => return Err(bad("front matter is not a mapping")),
    };
    let data: Vec<(Option<&str>, &YamlOwned)> = map.iter().map(|(k, v)| (key_str(k), v)).collect();
    let get = |key: &str| data.iter().find(|(k, _)| *k == Some(key)).map(|(_, v)| *v);

    match get("easy-note") {
        Some(YamlOwned::Value(ScalarOwned::Integer(v))) if *v == VERSION => {}
        Some(YamlOwned::Value(ScalarOwned::Integer(v))) => {
            return Err(bad(format!("unsupported version {v}")))
        }
        _ => return Err(bad("missing or non-integer easy-note version")),
    }
    for key in REQUIRED {
        if get(key).is_none() {
            return Err(bad(format!("missing required key '{key}'")));
        }
    }
    let mut fields = BTreeMap::new();
    fields.insert("easy-note".to_string(), Value::Int(VERSION));
    for key in STRING_KEYS {
        if let Some(value) = get(key) {
            match value {
                YamlOwned::Value(ScalarOwned::String(s)) => {
                    fields.insert(key.to_string(), Value::Str(s.clone()));
                }
                _ => return Err(bad(format!("key '{key}' is not a string"))),
            }
        }
    }
    let tags = match get("tags") {
        Some(YamlOwned::Sequence(items)) => items
            .iter()
            .map(|i| match i {
                YamlOwned::Value(ScalarOwned::String(s)) => Some(s.clone()),
                _ => None,
            })
            .collect::<Option<Vec<String>>>(),
        _ => None,
    };
    fields.insert(
        "tags".to_string(),
        Value::List(tags.ok_or_else(|| bad("key 'tags' is not a list of strings"))?),
    );
    if !SOURCES.contains(&note_str(&fields, "source")) {
        return Err(bad("unknown source"));
    }
    for key in ["created", "updated"] {
        if fields.contains_key(key) {
            check_time(key, note_str(&fields, key))?;
        }
    }
    let has_unknown = data
        .iter()
        .any(|(k, _)| !k.is_some_and(|k| KEY_ORDER.contains(&k)));
    let extra = if has_unknown {
        raw_unknown(front)
    } else {
        vec![]
    };
    Ok(Note {
        fields,
        body: body.to_string(),
        extra,
        path: None,
    })
}

pub fn render(note: &Note) -> String {
    let mut out = String::from("---\n");
    for key in KEY_ORDER {
        if let Some(value) = note.fields.get(key) {
            out.push_str(&format!("{key}: {}\n", value_text(value)));
        }
    }
    for chunk in &note.extra {
        out.push_str(chunk);
    }
    out.push_str("---\n");
    out.push_str(&note.body);
    out
}

fn key_str(key: &YamlOwned) -> Option<&str> {
    match key {
        YamlOwned::Value(ScalarOwned::String(s)) => Some(s),
        _ => None,
    }
}

fn note_str<'a>(fields: &'a BTreeMap<String, Value>, key: &str) -> &'a str {
    match fields.get(key) {
        Some(Value::Str(s)) => s,
        _ => "",
    }
}

fn check_time(key: &str, value: &str) -> Result<(), FormatError> {
    if DateTime::parse_from_rfc3339(value).is_ok() {
        return Ok(());
    }
    if NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f").is_ok() {
        return Err(bad(format!("key '{key}' has no UTC offset")));
    }
    Err(bad(format!("key '{key}' is not an ISO 8601 time")))
}

/// Source text of each top-level unknown key, from the key to the start of the next key.
fn raw_unknown(front: &str) -> Vec<String> {
    // Marker indexes count chars; map them to byte offsets.
    let offsets: Vec<usize> = front
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(front.len()))
        .collect();
    // (name if the key is a plain string, start byte) of every top-level key.
    let mut keys: Vec<(Option<String>, usize)> = vec![];
    let (mut depth, mut nodes) = (0usize, 0usize);
    for item in Parser::new_from_str(front) {
        let Ok((event, span)) = item else { break };
        // A node that is a direct child of the top mapping (depth 1): even ones are keys.
        let child = depth == 1
            && matches!(
                event,
                Event::Scalar(..)
                    | Event::Alias(_)
                    | Event::MappingStart(..)
                    | Event::SequenceStart(..)
            );
        if child {
            if nodes % 2 == 0 {
                let name = match &event {
                    Event::Scalar(s, ..) => Some(s.to_string()),
                    _ => None,
                };
                keys.push((name, offsets[span.start.index()]));
            }
            nodes += 1;
        }
        match event {
            Event::MappingStart(..) | Event::SequenceStart(..) => depth += 1,
            Event::MappingEnd | Event::SequenceEnd => depth -= 1,
            _ => {}
        }
    }
    let mut out = vec![];
    for (i, (name, start)) in keys.iter().enumerate() {
        if name.as_deref().is_some_and(|n| KEY_ORDER.contains(&n)) {
            continue;
        }
        let end = keys.get(i + 1).map_or(front.len(), |(_, s)| *s);
        out.push(format!(
            "{}\n",
            front[*start..end].trim_end_matches(['\r', '\n'])
        ));
    }
    out
}

fn value_text(value: &Value) -> String {
    match value {
        Value::Int(n) => n.to_string(),
        Value::Str(s) => quote(s),
        Value::List(items) => {
            let quoted: Vec<String> = items.iter().map(|i| quote(i)).collect();
            format!("[{}]", quoted.join(", "))
        }
    }
}

/// A JSON string is a valid YAML double-quoted scalar; escape what YAML would misread.
pub(crate) fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            // Control chars, plus chars a YAML 1.1 reader rejects or reads as a line break
            // even inside a quoted string.
            c if (c as u32) < 0x20
                || matches!(c as u32, 0x7f..=0x9f | 0x2028 | 0x2029 | 0xfffe | 0xffff) =>
            {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
