//! HUN-270 part 1 oracle: declared secret patterns, `mask`, `mask_for_output`,
//! no logging, false positives. Ported case by case from the Python HUN-255
//! `tests/test_guard.py` (verified branch at 3c5e40f).
//!
//! GitHub push protection rejected the Python branch on a fake Slack token, so no
//! literal here (or in the fixtures) holds a whole token prefix. Planted values are
//! stored as parts and joined at test time; inline values use `concat!` or `join`.

use easy_note_mcp::core::guard::{mask, mask_for_output, KINDS};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

const DECLARED: [&str; 10] = [
    "aws_key",
    "github_token",
    "slack_token",
    "openai_key",
    "anthropic_key",
    "private_key",
    "jwt",
    "password_assign",
    "secret_assign",
    "bearer",
];

const PLANTED_JSON: &str = include_str!("fixtures/secrets/planted.json");
const DEV_NOTES: &str = include_str!("fixtures/secrets/dev_notes_50.txt");

struct Planted {
    kind: String,
    text: String,
    secret: String,
}

fn join(parts: &Value) -> String {
    parts
        .as_array()
        .expect("parts must be an array")
        .iter()
        .map(|p| p.as_str().expect("part must be a string"))
        .collect()
}

fn planted() -> Vec<Planted> {
    let v: Value = serde_json::from_str(PLANTED_JSON).expect("planted.json parses");
    v.as_array()
        .expect("planted.json is an array")
        .iter()
        .map(|c| Planted {
            kind: c["kind"].as_str().expect("kind").to_string(),
            text: join(&c["text"]),
            secret: join(&c["secret"]),
        })
        .collect()
}

fn dev_notes() -> Vec<&'static str> {
    DEV_NOTES.lines().collect()
}

fn no_kinds() -> Vec<&'static str> {
    Vec::new()
}

/// A private key header, built so the source never holds the whole header.
fn begin(label: &str, tail: &str) -> String {
    ["-----BEG", "IN ", label, "PRIVATE KEY", tail, "-----"].concat()
}

fn end(label: &str, tail: &str) -> String {
    ["-----END ", label, "PRIVATE KEY", tail, "-----"].concat()
}

/// The well-known jwt.io sample token, built from parts.
fn jwt_full() -> String {
    [
        "ey",
        "JhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.",
        "ey",
        "JzdWIiOiIxMjM0NTY3ODkwIn0.",
        "SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
    ]
    .concat()
}

// AC 1: the ten declared kinds, in the declared order

#[test]
fn declared_pattern_names() {
    assert_eq!(KINDS, DECLARED);
    assert_eq!(KINDS.len(), 10);
}

#[test]
fn planted_secret_is_masked() {
    let mut failures = Vec::new();
    for (i, case) in planted().iter().enumerate() {
        let (masked, kinds) = mask(&case.text);
        if masked.contains(&case.secret) {
            failures.push(format!("case {i} {}: secret still in output", case.kind));
        }
        if !masked.contains(&format!("[masked:{}]", case.kind)) {
            failures.push(format!(
                "case {i} {}: marker missing in {masked:?}",
                case.kind
            ));
        }
        if kinds != vec![case.kind.as_str()] {
            failures.push(format!("case {i} {}: kinds were {kinds:?}", case.kind));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn every_declared_kind_has_a_planted_hit() {
    let cases = planted();
    for kind in DECLARED {
        assert!(
            cases.iter().any(|c| c.kind == kind),
            "no planted case for {kind}"
        );
    }
}

// AC 2: each match becomes [masked:<kind>], surrounding text unchanged

#[test]
fn surrounding_text_unchanged() {
    let text = concat!("before AKI", "AIOSFODNN7EXAMPLE after");
    let (masked, kinds) = mask(text);
    assert_eq!(masked, "before [masked:aws_key] after");
    assert_eq!(kinds, vec!["aws_key"]);
}

#[test]
fn assignment_keeps_the_key_name_and_masks_only_the_value() {
    let (masked, kinds) = mask("password=hunter2example, done");
    assert_eq!(masked, "password=[masked:password_assign], done");
    assert_eq!(kinds, vec!["password_assign"]);
    let (masked, _) = mask(r#"{"db_password": "correct horse battery"}"#);
    assert_eq!(masked, r#"{"db_password": "[masked:password_assign]"}"#);
}

#[test]
fn bearer_keeps_the_scheme_word() {
    let (masked, kinds) = mask("Authorization: Bearer Zm9vYmFyEXAMPLEtokenValue98765");
    assert_eq!(masked, "Authorization: Bearer [masked:bearer]");
    assert_eq!(kinds, vec!["bearer"]);
}

#[test]
fn private_key_block_is_masked_whole_and_multiline() {
    let text = format!(
        "a\n{}\nline1EXAMPLE\nline2EXAMPLE\n{}\nb",
        begin("", ""),
        end("", "")
    );
    let (masked, kinds) = mask(&text);
    assert_eq!(masked, "a\n[masked:private_key]\nb");
    assert_eq!(kinds, vec!["private_key"]);
}

#[test]
fn pgp_private_key_block_is_masked_whole() {
    let text = format!(
        "a\n{}\n\nlQOYBEXAMPLE\n=ab12\n{}\nb",
        begin("PGP ", " BLOCK"),
        end("PGP ", " BLOCK")
    );
    let (masked, kinds) = mask(&text);
    assert_eq!(masked, "a\n[masked:private_key]\nb");
    assert_eq!(kinds, vec!["private_key"]);
}

#[test]
fn password_with_a_quote_inside_is_masked_whole() {
    let cases = [
        ("password=Xy'9kLmQ2vRt8 done", "9kLmQ2vRt8"),
        (r#"password=Pa"ss9kLmQ2vRt8 done"#, "ss9kLmQ2vRt8"),
        ("DB_PASSWORD=Tr0ub4dor's&3x done", "Tr0ub4dor's&3x"),
        (r#"password="S3cr3t-unclosed done"#, "S3cr3t-unclosed"),
    ];
    for (text, secret) in cases {
        let (masked, kinds) = mask(text);
        assert!(!masked.contains(secret), "{text:?} -> {masked:?}");
        assert!(!masked.contains("'s&3x"), "{text:?} -> {masked:?}");
        assert_eq!(kinds, vec!["password_assign"], "{text:?}");
        assert!(masked.ends_with(" done"), "{text:?} -> {masked:?}");
    }
}

#[test]
fn partial_overlap_masks_the_union() {
    let text = format!(
        "password=abcd1234{}\nMIIEXAMPLE\n{} tail",
        begin("RSA ", ""),
        end("RSA ", "")
    );
    let (masked, kinds) = mask(&text);
    assert!(!masked.contains("abcd1234"), "{masked:?}");
    assert!(!masked.contains("MIIEXAMPLE"), "{masked:?}");
    assert_eq!(masked, "password=[masked:private_key] tail");
    assert_eq!(kinds, vec!["private_key"]);
}

#[test]
fn marker_does_not_hide_a_later_secret() {
    let (masked, kinds) = mask("password=[masked:password_assign];password=hunter2real");
    assert!(!masked.contains("hunter2real"), "{masked:?}");
    assert_eq!(kinds, vec!["password_assign"]);
}

#[test]
fn truncated_private_key_masks_to_the_end() {
    let text = format!("x {}\nMHcCAQEEIEXAMPLE", begin("EC ", ""));
    let (masked, kinds) = mask(&text);
    assert_eq!(masked, "x [masked:private_key]");
    assert_eq!(kinds, vec!["private_key"]);
}

#[test]
fn several_secrets_report_count_and_kinds_in_text_order() {
    let text = concat!(
        "gh",
        "p_EXAMPLEexampleEXAMPLEexampleEXAMPLE123456 then AKI",
        "AIOSFODNN7EXAMPLE"
    );
    let (masked, kinds) = mask(text);
    assert_eq!(kinds, vec!["github_token", "aws_key"]);
    assert_eq!(masked, "[masked:github_token] then [masked:aws_key]");
}

#[test]
fn long_runs_stay_fast() {
    // No pattern may go quadratic on a long word run (a note can hold a pasted blob).
    let blobs = [
        "a".repeat(200_000),
        "password".repeat(20_000),
        "sk-".repeat(50_000),
        ["ey", "J."].concat().repeat(50_000),
        "x ".repeat(100_000),
    ];
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for blob in &blobs {
            let _ = mask(blob);
        }
        let _ = tx.send(());
    });
    rx.recv_timeout(Duration::from_secs(60))
        .expect("mask did not finish the long runs in 60 s");
}

#[test]
fn no_match_returns_text_unchanged_and_no_kinds() {
    assert_eq!(
        mask("just a normal note"),
        ("just a normal note".to_string(), no_kinds())
    );
    assert_eq!(mask(""), (String::new(), no_kinds()));
}

// Overlap and idempotence (edge case checklist)

#[test]
fn overlap_masks_once_longest_span_wins() {
    let cases = planted();
    let jwt = &cases
        .iter()
        .find(|c| c.kind == "jwt")
        .expect("planted jwt")
        .secret;
    let (masked, kinds) = mask(&format!("token_secret=\"Bearer {jwt}\" tail"));
    assert!(!masked.contains(jwt.as_str()), "{masked:?}");
    assert_eq!(kinds.len(), 1, "{kinds:?}");
    assert_eq!(masked.matches("[masked:").count(), 1, "{masked:?}");
}

#[test]
fn full_jwt_inside_a_bearer_assignment_is_masked_once() {
    let jwt = jwt_full();
    let (masked, kinds) = mask(&format!("token_secret=\"Bearer {jwt}\" tail"));
    assert!(!masked.contains(&jwt), "{masked:?}");
    assert_eq!(kinds.len(), 1, "{kinds:?}");
    assert_eq!(masked.matches("[masked:").count(), 1, "{masked:?}");
}

#[test]
fn jwt_as_password_value_is_masked_once() {
    let jwt = jwt_full();
    let (masked, kinds) = mask(&format!("password={jwt}"));
    assert!(!masked.contains(&jwt), "{masked:?}");
    assert_eq!(kinds.len(), 1, "{kinds:?}");
    assert!(masked.starts_with("password=[masked:"), "{masked:?}");
}

#[test]
fn second_pass_changes_nothing() {
    let mut failures = Vec::new();
    for (i, case) in planted().iter().enumerate() {
        let (once, _) = mask(&case.text);
        let (twice, kinds) = mask(&once);
        if twice != once || !kinds.is_empty() {
            failures.push(format!(
                "case {i} {}: {once:?} -> {twice:?} {kinds:?}",
                case.kind
            ));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

// Near misses: one per pattern, each must stay untouched

macro_rules! near_miss {
    ($name:ident, $text:expr) => {
        #[test]
        fn $name() {
            let text: String = String::from($text);
            assert_eq!(mask(&text), (text.clone(), no_kinds()));
        }
    };
}

near_miss!(
    near_miss_aws_key,
    concat!(
        "AKI",
        "AIOSFODNN7EXAMPL is one char short, and akiaiosfodnn7example is lower case"
    )
);
near_miss!(
    near_miss_github_token,
    "ghp_short and ghost branch ghp-cleanup"
);
near_miss!(
    near_miss_slack_token,
    "xoxb- alone, xoxz-123456789012-abcdefghij is not a slack kind"
);
near_miss!(near_miss_openai_key, "sk-learn and sk-short are not keys");
near_miss!(near_miss_anthropic_key, "sk-ant- is the prefix only");
near_miss!(
    near_miss_private_key,
    "-----BEGIN CERTIFICATE----- is public, not a private key"
);
near_miss!(
    near_miss_jwt,
    concat!(
        "ey",
        "JhbGciOiJIUzI1NiJ9.ey",
        "JzdWIiOiIxMjM0In0 has only two parts"
    )
);
near_miss!(
    near_miss_password_assign,
    "password reset flow, and password: x"
);
near_miss!(
    near_miss_secret_assign,
    "secret santa list, secret to speed, api_key is rotated"
);
near_miss!(
    near_miss_bearer,
    "Bearer authentication is in RFC 6750, Bearer aaaaaaaaaaaaaaaaaaaaaaaaaaaa"
);

#[test]
fn commit_sha_and_sha256_are_not_masked() {
    let text = format!(
        "commit 4fb32dd19c9678a1b2c3d4e5f60718293a4b5c6d and sha256:{}",
        "ab12".repeat(16)
    );
    assert_eq!(mask(&text), (text.clone(), no_kinds()));
}

// mask_for_output (AC 4 helper)

#[test]
fn mask_for_output_returns_masked_text_only() {
    let raw = concat!("hand edited AKI", "AIOSFODNN7EXAMPLE here");
    let out: String = mask_for_output(raw);
    assert_eq!(out, "hand edited [masked:aws_key] here");
}

#[test]
fn mask_for_output_leaves_clean_text_alone() {
    assert_eq!(mask_for_output("clean"), "clean");
}

#[test]
fn mask_for_output_does_not_touch_the_input_file() {
    let dir =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("guard-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("hand.md");
    let raw = format!("---\neasy-note: 1\n---\nJWT {}\n", jwt_full());
    std::fs::write(&file, &raw).unwrap();
    let out = mask_for_output(&std::fs::read_to_string(&file).unwrap());
    assert!(!out.contains("SflKxwRJ"), "{out:?}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), raw);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mask_for_output_equals_the_text_half_of_mask() {
    for case in planted() {
        assert_eq!(
            mask_for_output(&case.text),
            mask(&case.text).0,
            "{}",
            case.kind
        );
    }
}

// AC 6: the guard never logs and never prints note text.
// Python captured log records at DEBUG. The Rust guard has no logger, so the
// mapping is structural: the guard source holds no logging or printing call.

#[test]
fn guard_source_has_no_logging_or_printing() {
    let src = include_str!("../src/core/guard.rs");
    let banned = [
        "println!",
        "eprintln!",
        "print!",
        "eprint!",
        "dbg!",
        "log::",
        "tracing::",
        "io::stdout",
        "io::stderr",
    ];
    for (n, line) in src.lines().enumerate() {
        let code = line.trim_start();
        if code.starts_with("//") {
            continue;
        }
        for b in banned {
            assert!(!code.contains(b), "guard.rs:{} uses {b}: {line}", n + 1);
        }
    }
}

// AC 7: 50 normal dev notes mask nothing

#[test]
fn dev_notes_fixture_has_fifty_notes() {
    assert_eq!(dev_notes().len(), 50);
}

#[test]
fn normal_dev_notes_mask_nothing() {
    let mut failures = Vec::new();
    for (i, note) in dev_notes().iter().enumerate() {
        let (masked, kinds) = mask(note);
        if masked != *note || !kinds.is_empty() {
            failures.push(format!("note {i}: {note:?} -> {masked:?} {kinds:?}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
