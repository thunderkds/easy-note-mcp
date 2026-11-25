//! HUN-269 note format (port of Python HUN-253 tests/test_format.py): exact round-trip, YAML traps,
//! version check, unknown keys, title and size rules.

use std::collections::BTreeMap;
use std::path::PathBuf;

use easy_note_mcp::core::format::{self as fmt, Note, Value, KEY_ORDER, SOURCES};
use saphyr::{LoadableYamlNode, YamlOwned};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/notes/2026/11")
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(name)).unwrap()
}

fn fixture_text(name: &str) -> String {
    String::from_utf8(fixture(name)).unwrap()
}

/// A valid note plus the given field overrides.
fn note(over: Vec<(&str, Value)>) -> Note {
    let mut fields = BTreeMap::new();
    fields.insert("easy-note".to_string(), Value::Int(1));
    fields.insert("id".to_string(), Value::Str("20261114-164713-a1b2".into()));
    fields.insert(
        "created".to_string(),
        Value::Str("2026-11-14T16:47:13+07:00".into()),
    );
    fields.insert("tags".to_string(), Value::List(vec![]));
    fields.insert("source".to_string(), Value::Str("cli".into()));
    for (key, value) in over {
        fields.insert(key.to_string(), value);
    }
    Note::new(fields, "text\n".to_string())
}

/// The front matter between the two `---` lines of rendered text.
fn front(text: &str) -> &str {
    text.split("---\n").nth(1).unwrap()
}

fn yaml_map(front: &str) -> saphyr::MappingOwned {
    match YamlOwned::load_from_str(front).unwrap().remove(0) {
        YamlOwned::Mapping(map) => map,
        other => panic!("not a mapping: {other:?}"),
    }
}

fn yaml_str(map: &saphyr::MappingOwned, key: &str) -> Option<String> {
    map.iter()
        .find(|(k, _)| k.as_str() == Some(key))
        .and_then(|(_, v)| v.as_str().map(str::to_string))
}

// AC1 — render(parse(f)) of a tool-written note is byte-identical to f

#[test]
fn round_trip_of_tool_written_fixture_is_byte_identical() {
    let raw = fixture("20261114-164713-a1b2.md");
    let text = String::from_utf8(raw.clone()).unwrap();
    assert_eq!(fmt::render(&fmt::parse(&text).unwrap()).into_bytes(), raw);
}

#[test]
fn render_quotes_strings_so_yaml_keeps_them_strings() {
    let values = [
        "no",
        "on",
        "yes",
        "null",
        "~",
        "2026-11-14",
        "12",
        "0x1f",
        "a: b",
        "#x",
        "- x",
        "[x]",
        "quote \" and \\ slash",
        "tab\there",
        "é ü 中文",
        "emoji 😀",
        "nel\u{85}",
        "ls\u{2028}",
        "del\u{7f}",
        "",
    ];
    for value in values {
        let tags = if value.is_empty() {
            vec![]
        } else {
            vec![value.to_string()]
        };
        let n = note(vec![
            ("branch", Value::Str(value.into())),
            ("tags", Value::List(tags)),
        ]);
        let text = fmt::render(&n);
        let again = fmt::parse(&text).unwrap();
        assert_eq!(again.get_str("branch"), Some(value), "value {value:?}");
        assert_eq!(fmt::render(&again), text, "value {value:?}");
        // a plain YAML reader agrees
        let map = yaml_map(front(&text));
        assert_eq!(
            yaml_str(&map, "branch").as_deref(),
            Some(value),
            "value {value:?}"
        );
    }
}

#[test]
fn render_uses_fixed_key_order_whatever_the_input_order() {
    let mut fields = BTreeMap::new();
    fields.insert("source".to_string(), Value::Str("cli".into()));
    fields.insert("tags".to_string(), Value::List(vec!["a".into()]));
    fields.insert("repo".to_string(), Value::Str("r".into()));
    fields.insert(
        "created".to_string(),
        Value::Str("2026-11-14T16:47:13+07:00".into()),
    );
    fields.insert("id".to_string(), Value::Str("20261114-164713-a1b2".into()));
    fields.insert("easy-note".to_string(), Value::Int(1));
    let text = fmt::render(&Note::new(fields, "x\n".into()));
    let keys: Vec<&str> = text
        .lines()
        .skip(1)
        .take(5)
        .map(|l| l.split(':').next().unwrap())
        .collect();
    assert_eq!(keys, ["easy-note", "id", "created", "repo", "tags"]);
}

#[test]
fn created_stays_a_string_even_when_hand_written_unquoted() {
    let text = "---\neasy-note: 1\nid: x\ncreated: 2026-11-14T16:47:13+07:00\ntags: []\nsource: cli\n---\nbody\n";
    assert_eq!(
        fmt::parse(text).unwrap().get_str("created"),
        Some("2026-11-14T16:47:13+07:00")
    );
}

#[test]
fn crlf_file_parses() {
    let n = fmt::parse(&fixture_text("20261114-170300-c71f.md")).unwrap();
    assert_eq!(n.id(), "20261114-170300-c71f");
    assert_eq!(n.body, "Edited on Windows\r\nsecond line\r\n");
    assert_eq!(n.title(), "Edited on Windows");
}

// AC6 (parse side) — bad front matter and `easy-note: 2` are refused with a reason, never "fixed"

#[test]
fn bad_fixture_files_return_format_error() {
    for (name, reason) in [
        ("20261114-170000-bad1.md", "not valid YAML"),
        ("20261114-170100-0002.md", "unsupported version 2"),
        ("20261114-170200-0003.md", "no front matter"),
    ] {
        let err = fmt::parse(&fixture_text(name)).unwrap_err();
        assert!(err.to_string().contains(reason), "{name}: {err}");
    }
}

#[test]
fn invalid_front_matter_returns_format_error() {
    let cases = [
        // bool is not 1
        "easy-note: true\nid: x\ncreated: 2026-11-14T16:47:13+07:00\ntags: []\nsource: cli\n",
        // version missing
        "id: x\ncreated: 2026-11-14T16:47:13+07:00\ntags: []\nsource: cli\n",
        // id missing
        "easy-note: 1\ncreated: 2026-11-14T16:47:13+07:00\ntags: []\nsource: cli\n",
        // created not ISO
        "easy-note: 1\nid: x\ncreated: yesterday\ntags: []\nsource: cli\n",
        // no offset
        "easy-note: 1\nid: x\ncreated: 2026-11-14T16:47:13\ntags: []\nsource: cli\n",
        // tags not a list
        "easy-note: 1\nid: x\ncreated: 2026-11-14T16:47:13+07:00\ntags: flaky\nsource: cli\n",
        // unknown source
        "easy-note: 1\nid: x\ncreated: 2026-11-14T16:47:13+07:00\ntags: []\nsource: fax\n",
        // repo is not a string
        "easy-note: 1\nid: x\ncreated: 2026-11-14T16:47:13+07:00\ntags: []\nsource: cli\nrepo: 12\n",
        // not a mapping
        "- just\n- a list\n",
    ];
    for front in cases {
        let text = format!("---\n{front}---\nbody\n");
        assert!(fmt::parse(&text).is_err(), "should be refused: {front}");
    }
}

#[test]
fn format_error_reason_never_quotes_file_content() {
    let text = "---\neasy-note: 1\nid: \"ghp_SECRETVALUE\n---\nbody\n";
    let err = fmt::parse(text).unwrap_err();
    assert!(!err.to_string().contains("ghp_"), "{err}");
}

// AC7 (parse side) — unknown keys are kept verbatim, known keys stay canonical

#[test]
fn unknown_keys_render_verbatim_after_known_keys() {
    let raw = fixture_text("20261114-170400-0b5d.md");
    let out = fmt::render(&fmt::parse(&raw).unwrap());
    assert!(out.contains("aliases:\n  - firefox retry\n"));
    assert!(out.contains("date: 2024-01-01 # Obsidian date property\n"));
    assert!(out.contains("cssclasses: [wide]\n"));
    assert!(out.find("source:").unwrap() < out.find("aliases:").unwrap());
    // a plain YAML reader still reads the date as plain text `2024-01-01`
    let map = yaml_map(front(&out));
    assert_eq!(yaml_str(&map, "date").as_deref(), Some("2024-01-01"));
}

// AC9 — title is the first line cut to 120 chars; text over 20,000 chars is refused

#[test]
fn title_is_first_line_cut_to_120_chars() {
    let long_first = "a".repeat(130);
    let n = Note::new(BTreeMap::new(), format!("{long_first}\nrest\n"));
    assert_eq!(n.title(), "a".repeat(120));
    assert_eq!(
        Note::new(BTreeMap::new(), "short\nrest\n".into()).title(),
        "short"
    );
}

#[test]
fn check_text_refuses_over_20000_chars_with_clear_error() {
    fmt::check_text(&"x".repeat(20_000)).unwrap();
    let err = fmt::check_text(&"x".repeat(20_001)).unwrap_err();
    assert!(
        err.to_string()
            .contains("20,001 chars; the limit is 20,000"),
        "{err}"
    );
}

// AC10 — docs/NOTE_FORMAT.md is Final for v1 and matches the code

fn doc() -> String {
    std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/NOTE_FORMAT.md"))
        .unwrap()
}

#[test]
fn note_format_doc_is_final_and_its_example_round_trips() {
    let doc = doc();
    assert!(doc.contains("**Status**: Final for `easy-note: 1`"));
    // The round 6 kind `draft` is a word in the doc now; only a Draft status is wrong.
    assert!(!doc.contains("**Status**: Draft"));
    let example = doc
        .split("```markdown\n")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap();
    assert_eq!(example, fixture_text("20261114-164713-a1b2.md"));
    assert_eq!(fmt::render(&fmt::parse(example).unwrap()), example);
}

#[test]
fn note_format_doc_lists_every_key_the_code_knows() {
    let doc = doc();
    let table_keys: Vec<&str> = doc
        .lines()
        .filter(|l| l.starts_with("| `"))
        .map(|l| l.split('`').nth(1).unwrap())
        .collect();
    assert_eq!(table_keys, KEY_ORDER);
    for source in SOURCES {
        assert!(doc.contains(&format!("`{source}`")), "{source}");
    }
}
