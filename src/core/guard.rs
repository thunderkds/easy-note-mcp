//! Secret guard: declared patterns, mask on write and on read, never refuse (D21, FR-014).
//!
//! Ported from the verified Python `guard.py` (HUN-255, 3c5e40f). The patterns are declared
//! here, never imported from a sibling repo. This module never logs or prints: a log line
//! could hold note text (NFR-002).
//!
//! The Python patterns use look-ahead, which the `regex` crate lacks. Two restatements:
//! `sk-` for OpenAI keys skips a match that starts with `sk-ant-`; the value of an
//! assignment (`password=`, `secret=`) is scanned by hand after a regex finds the key.

use regex::Regex;
use std::sync::OnceLock;

/// The ten kinds, in declared order. Declared order is the tie-break for equal spans.
pub const KINDS: [&str; 10] = [
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

const MARKER_START: &str = "[masked:";

const AWS: usize = 0;
const GITHUB: usize = 1;
const SLACK: usize = 2;
const OPENAI: usize = 3;
const ANTHROPIC: usize = 4;
const PRIVATE_KEY: usize = 5;
const JWT: usize = 6;
const PASSWORD: usize = 7;
const SECRET: usize = 8;
const BEARER: usize = 9;

/// A plain regex rule. `group` is the capture that is masked; 0 is the whole match.
/// `check` returns false when the text is not a secret, so it is kept.
struct Rule {
    order: usize,
    regex: Regex,
    group: usize,
    check: Option<fn(&str) -> bool>,
}

struct Rules {
    plain: Vec<Rule>,
    password_key: Regex,
    secret_key: Regex,
}

fn rules() -> &'static Rules {
    static RULES: OnceLock<Rules> = OnceLock::new();
    RULES.get_or_init(|| {
        let rule = |order, pattern: &str, group, check| Rule {
            order,
            regex: Regex::new(pattern).expect("declared pattern compiles"),
            group,
            check,
        };
        // A key word (so `DB_PASSWORD` and `"password"` match), then = or :, then optional blanks.
        let key = |words: &str| {
            Regex::new(&format!(r#"(?i)(?:{words})["']?[ \t]*[=:][ \t]*"#))
                .expect("declared pattern compiles")
        };
        Rules {
            plain: vec![
                rule(AWS, r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b", 0, None),
                rule(
                    GITHUB,
                    r"\b(?:gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{50,})",
                    0,
                    None,
                ),
                rule(SLACK, r"\bxox[abprs]-[A-Za-z0-9-]{10,}", 0, None),
                rule(
                    OPENAI,
                    r"\bsk-(?:proj-)?[A-Za-z0-9_-]{20,}",
                    0,
                    Some(not_anthropic),
                ),
                rule(ANTHROPIC, r"\bsk-ant-[A-Za-z0-9_-]{20,}", 0, None),
                // A block with no END line (a truncated paste) is masked to the end of the text.
                rule(
                    PRIVATE_KEY,
                    r"-----BEGIN [A-Z ]*PRIVATE KEY(?: BLOCK)?-----[\s\S]*?(?:-----END [A-Z ]*PRIVATE KEY(?: BLOCK)?-----|\z)",
                    0,
                    None,
                ),
                rule(JWT, r"\beyJ[\w-]{8,}\.[\w-]{8,}\.[\w-]{8,}", 0, None),
                rule(
                    BEARER,
                    r"(?i)\bBearer[ \t]+([A-Za-z0-9._~+/=-]{20,})",
                    1,
                    Some(high_entropy),
                ),
            ],
            password_key: key("password|passwd"),
            secret_key: key(
                r"secret(?:[_-]?(?:access[_-]?)?key)?|api[_-]?key|(?:auth|access)[_-]?token",
            ),
        }
    })
}

fn not_anthropic(value: &str) -> bool {
    !value.starts_with("sk-ant-")
}

fn entropy(value: &str) -> f64 {
    let mut counts = std::collections::HashMap::new();
    let mut total = 0usize;
    for c in value.chars() {
        *counts.entry(c).or_insert(0usize) += 1;
        total += 1;
    }
    let total = total as f64;
    -counts
        .values()
        .map(|&n| {
            let p = n as f64 / total;
            p * p.log2()
        })
        .sum::<f64>()
}

fn high_entropy(value: &str) -> bool {
    !value.contains(MARKER_START) && value.chars().count() >= 20 && entropy(value) >= 3.5
}

/// End (byte offset in `rest`) of a closed quoted value: the quote, at least 4 characters
/// with no quote and no newline, then the same quote.
fn closed_quote_end(rest: &str, quote: char) -> Option<usize> {
    let body = &rest[1..];
    let stop = body.find([quote, '\n'])?;
    if !body[stop..].starts_with(quote) || body[..stop].chars().count() < 4 {
        return None;
    }
    Some(1 + stop + 1)
}

/// End of an unclosed quoted value: the quote, then a run of at least 3 characters that are
/// not blank, not the quote and do not start a marker.
fn unclosed_quote_end(rest: &str, quote: char) -> Option<usize> {
    let body = &rest[1..];
    let mut count = 0;
    let mut end = body.len();
    for (n, (i, c)) in body.char_indices().enumerate() {
        if c.is_whitespace() || c == quote || body[i..].starts_with(MARKER_START) {
            end = i;
            break;
        }
        count = n + 1;
    }
    (count >= 3).then_some(1 + end)
}

/// End of an unquoted value: a run of non-blank characters (at least 4, quotes allowed after the
/// first) that does not end in sentence punctuation, so `password=x1y2z3, done` keeps its comma.
/// No unquoted value runs into a `[masked:` marker, so a marker never hides a later secret.
fn unquoted_end(rest: &str) -> Option<usize> {
    let mut end = None;
    for (n, (i, c)) in rest.char_indices().enumerate() {
        if c.is_whitespace() || rest[i..].starts_with(MARKER_START) {
            break;
        }
        if n >= 3 && !"\"',;.)".contains(c) {
            end = Some(i + c.len_utf8());
        }
    }
    end
}

/// The value after an assignment key: (end offset in `rest`, whether it is a closed quoted string).
fn value_end(rest: &str) -> Option<(usize, bool)> {
    match rest.chars().next()? {
        q @ ('"' | '\'') => closed_quote_end(rest, q)
            .map(|e| (e, true))
            .or_else(|| unclosed_quote_end(rest, q).map(|e| (e, false))),
        c if c.is_whitespace() => None,
        _ => unquoted_end(rest).map(|e| (e, false)),
    }
}

/// Spans of the values assigned to a key matched by `key_re`. Only the value is masked; a closed
/// quoted value keeps its quotes. A value that already holds a marker is not a secret.
fn assign_spans(text: &str, key_re: &Regex, order: usize, out: &mut Vec<(usize, usize, usize)>) {
    let mut pos = 0;
    while let Some(key) = key_re.find_at(text, pos) {
        let start = key.end();
        let Some((len, closed)) = value_end(&text[start..]) else {
            pos = start;
            continue;
        };
        let end = start + len;
        pos = end;
        if text[start..end].contains(MARKER_START) {
            continue;
        }
        if closed {
            out.push((start + 1, end - 1, order));
        } else {
            out.push((start, end, order));
        }
    }
}

/// Every secret as (start, end, declared index).
fn spans(text: &str) -> Vec<(usize, usize, usize)> {
    let rules = rules();
    let mut found = Vec::new();
    for rule in &rules.plain {
        for caps in rule.regex.captures_iter(text) {
            let Some(m) = caps.get(rule.group) else {
                continue;
            };
            if rule.check.is_some_and(|check| !check(m.as_str())) {
                continue;
            }
            found.push((m.start(), m.end(), rule.order));
        }
    }
    assign_spans(text, &rules.password_key, PASSWORD, &mut found);
    assign_spans(text, &rules.secret_key, SECRET, &mut found);
    found
}

/// Return `(masked_text, kinds)`: each secret becomes `[masked:<kind>]`; kinds are in text order.
///
/// Overlapping matches are masked once, as their union, so no part of either leaks; the kind
/// is the longest span's kind, then the declared order.
pub fn mask(text: &str) -> (String, Vec<&'static str>) {
    let mut all = spans(text);
    all.sort_unstable();
    // (start, end, longest length, kind order)
    let mut kept: Vec<(usize, usize, usize, usize)> = Vec::new();
    for (start, end, order) in all {
        match kept.last_mut() {
            Some(last) if start < last.1 => {
                let len = end - start;
                if len > last.2 || (len == last.2 && order < last.3) {
                    last.2 = len;
                    last.3 = order;
                }
                last.1 = last.1.max(end);
            }
            _ => kept.push((start, end, end - start, order)),
        }
    }
    let mut out = String::with_capacity(text.len());
    let mut pos = 0;
    for &(start, end, _, order) in &kept {
        out.push_str(&text[pos..start]);
        out.push_str(MARKER_START);
        out.push_str(KINDS[order]);
        out.push(']');
        pos = end;
    }
    out.push_str(&text[pos..]);
    (out, kept.iter().map(|k| KINDS[k.3]).collect())
}

/// Mask text read from a file (hand-edited files may hold secrets); the file stays unchanged.
pub fn mask_for_output(text: &str) -> String {
    mask(text).0
}
