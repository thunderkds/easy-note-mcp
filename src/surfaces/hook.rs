//! `hook-prompt`: the Claude Code `UserPromptSubmit` hook (FR-018, DDR 0003).
//!
//! Claude Code runs this on every prompt and writes the event JSON to stdin. A prompt that starts
//! with the capture prefix is saved as a note (`source: hook`) and blocked, so no model turn runs.
//! Every other prompt gets no output and exit 0. The command never fails a prompt: it exits 0 on
//! every path, and bad input just lets the prompt through.
//!
//! The pass path reads stdin and, at most, one small file (`config.toml`, for the prefix). It
//! opens no store, runs no git command and writes nothing, and it starts no async runtime (NFR-009).

use std::io::{Read, Write};
use std::path::Path;

use serde_json::{json, Value};

use super::cli::{confirmation, safe};
use crate::core::config::{self, DEFAULT_CAPTURE_PREFIX};
use crate::core::store::Store;
use crate::core::{context, tags};

/// Read one event from stdin and print the answer, if any. Always returns; the caller exits 0.
pub fn run() {
    let mut input = Vec::new();
    if std::io::stdin().lock().read_to_end(&mut input).is_err() {
        return;
    }
    if let Some(line) = answer(&input) {
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, "{line}");
        let _ = out.flush();
    }
}

/// The one stdout line for this event, or `None` when the prompt passes silently.
fn answer(input: &[u8]) -> Option<String> {
    let event: Value = serde_json::from_slice(input).ok()?;
    let prompt = event.get("prompt")?.as_str()?;
    // Any config problem (unreadable file, bad value) means the default prefix: a prompt must
    // never fail for it.
    let prefix = config::capture_prefix().unwrap_or_else(|_| DEFAULT_CAPTURE_PREFIX.to_string());
    let text = prompt.strip_prefix(prefix.as_str())?.trim();
    if text.is_empty() {
        return None;
    }
    let cwd = event.get("cwd").and_then(Value::as_str).map(Path::new);
    let reply = match save(text, cwd) {
        Ok(reason) => json!({"decision": "block", "reason": reason}),
        // The prompt passes, so there is no `decision`. Claude Code drops `reason` without a
        // block, so `systemMessage` carries the same warning to the person.
        Err(message) => {
            let warning = format!("easy-note: {} — note not saved", safe(&message));
            json!({"reason": warning, "systemMessage": warning})
        }
    };
    Some(reply.to_string())
}

/// Save the note; the reason line is `✓ noted <id> · <repo> · <task>`, with a masked count if any.
fn save(text: &str, cwd: Option<&Path>) -> Result<String, String> {
    let store = Store::open().map_err(|e| e.to_string())?;
    let found = context::capture(None, cwd, &[]);
    let added = store
        .add(text, "hook", &tags::extract(text), &found.fields())
        .map_err(|e| e.to_string())?;
    Ok(confirmation(
        "noted",
        &added.id,
        &[found.repo.as_deref(), found.task.as_deref()],
        &added.masked,
    ))
}
