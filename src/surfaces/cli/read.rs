//! The commands that only read: `ls`, `search`, `show`, `stats`, `doctor`, `path`. Every note
//! text they print is masked again (FR-014), whatever the core already did.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use super::scratch;
use super::{count, safe, say, say_raw, unique_kinds, Fail, Outcome};
use crate::core::query::{self, Counts, Filters, Page, Stats};
use crate::core::store::{Item, Store};
use crate::core::{ai, config, format, guard};
#[cfg(feature = "tray")]
use crate::surfaces::tray;

fn print_page(page: &Page) -> Outcome {
    for item in &page.items {
        let mut line = format!("{}  {}", item.id, item.title);
        for tag in &item.tags {
            line.push_str(&format!("  #{tag}"));
        }
        if item.matched_in == Some("ai") {
            line.push_str("  [ai]");
        }
        say(&safe(&line))?;
    }
    if page.truncated {
        eprintln!(
            "easy-note: showing {} of {} (use -n for more, up to {})",
            page.items.len(),
            page.total,
            query::MAX_LIMIT
        );
    }
    Ok(())
}

pub fn ls(filters: Filters, limit: usize) -> Outcome {
    print_page(&query::list_notes(&filters, limit)?)
}

pub fn search(words: &str, filters: Filters, limit: usize) -> Outcome {
    print_page(&query::search(words, &filters, limit)?)
}

/// The note as its file reads, front matter and text, with secrets masked; then its AI detail,
/// if any, under its label (FR-032).
pub fn show(id: &str) -> Outcome {
    let store = Store::open()?;
    let note = store.read(id)?;
    let mut text = format::render(&note);
    if let Some((detail, stale)) = ai::read(&store, note.id()) {
        text.push_str(&ai_block(&detail, stale));
    }
    say_raw(&safe(&text))
}

/// The AI detail block of `show`: label, model and time, `(stale)` when the note changed since.
fn ai_block(detail: &ai::Detail, stale: bool) -> String {
    let mut block = format!(
        "\n--- {} · {} · {}{}\n",
        ai::LABEL,
        detail.model,
        detail.at,
        if stale { " (stale)" } else { "" }
    );
    block.push_str(&format!("title: {}\n", detail.title));
    block.push_str(&format!("kind: {}\n", detail.kind));
    block.push_str(&format!("summary: {}\n", detail.summary));
    if !detail.tags_suggested.is_empty() {
        let tags: Vec<String> = detail
            .tags_suggested
            .iter()
            .map(|t| format!("#{t}"))
            .collect();
        block.push_str(&format!("suggested tags: {}\n", tags.join(" ")));
    }
    block
}

pub fn path() -> Outcome {
    say(&safe(&config::notes_dir()?.display().to_string()))
}

fn counts_line(label: &str, counts: &Counts) -> Option<String> {
    let pairs = counts.as_pairs();
    if pairs.is_empty() {
        return None;
    }
    let list: Vec<String> = pairs.iter().map(|(k, n)| format!("{k} ({n})")).collect();
    Some(format!("{label}: {}", list.join(", ")))
}

fn stats_text(stats: &Stats) -> String {
    let mut lines = vec![count(stats.total, "note")];
    if let (Some(oldest), Some(newest)) = (&stats.oldest, &stats.newest) {
        lines[0].push_str(&format!(", {oldest} to {newest}"));
    }
    lines.extend(
        [
            ("repo", &stats.by_repo),
            ("task", &stats.by_task),
            ("tag", &stats.by_tag),
            ("source", &stats.by_source),
            ("month", &stats.by_month),
        ]
        .iter()
        .filter_map(|(label, counts)| counts_line(label, counts)),
    );
    lines.push(format!("enriched: {}", stats.enriched));
    if !stats.skipped.is_empty() {
        lines.push(format!("skipped: {}", stats.skipped.len()));
        for file in &stats.skipped {
            lines.push(format!("  {} ({})", file.path, file.reason));
        }
    }
    lines.join("\n")
}

pub fn stats(json: bool, filters: Filters) -> Outcome {
    let stats = query::stats(&filters)?;
    if json {
        let text = serde_json::to_string_pretty(&stats)
            .map_err(|e| Fail::runtime(format!("cannot write JSON: {e}")))?;
        say(&safe(&text))
    } else {
        say(&safe(&stats_text(&stats)))
    }
}

/// The `n` the shell runs, when it is not this program (FR-020).
fn n_clash() -> Option<PathBuf> {
    let me = std::env::current_exe().ok()?.canonicalize().ok()?;
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .filter(|dir| !dir.as_os_str().is_empty())
        .map(|dir| dir.join("n"))
        .find(|candidate| {
            fs::metadata(candidate)
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
        .filter(|first| first.canonicalize().ok().as_ref() != Some(&me))
}

/// The doctor's finding about one note: the kinds of secret its file holds, if any. The store
/// masks what it yields, so this reads the file itself.
fn secrets_line(note: &format::Note) -> Option<String> {
    let path = note.path.as_ref()?;
    let (_, kinds) = guard::mask(&fs::read_to_string(path).ok()?);
    (!kinds.is_empty()).then(|| {
        format!(
            "secrets in {}: {}",
            path.display(),
            unique_kinds(&kinds).join(", ")
        )
    })
}

/// The doctor's findings about the notes dir, one line each, and how many notes it checked.
fn findings(store: &Store) -> (Vec<String>, usize) {
    let (mut lines, mut notes) = (vec![], 0);
    for item in store.iter_notes() {
        match item {
            Item::Skipped(file) => lines.push(format!(
                "skipped: {} ({})",
                file.path.display(),
                file.reason
            )),
            Item::Note(note) => {
                notes += 1;
                lines.extend(secrets_line(&note));
            }
        }
    }
    for path in scratch::leftovers(&store.root) {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        lines.push(format!(
            "leftover editor file: {name} (the next edit removes it)"
        ));
    }
    if let Some(other) = n_clash() {
        lines.push(format!(
            "clash: `n` on your PATH is {}, not easy-note",
            other.display()
        ));
    }
    (lines, notes)
}

/// `doctor`: report, never fix. Skipped files, kinds of secret in hand-edited files, leftover
/// editor files, an `n` clash.
pub fn doctor() -> Outcome {
    let store = Store::open()?;
    say(&format!(
        "notes dir: {}",
        safe(&store.root.display().to_string())
    ))?;
    let (lines, notes) = findings(&store);
    for line in &lines {
        say(&safe(line))?;
    }
    say(&format!("{} checked", count(notes, "note")))?;
    if lines.is_empty() {
        say("no problems found")?;
    } else {
        say(&format!("{} found", count(lines.len(), "problem")))?;
    }
    // Only a tray build has a tray to miss, and a missing host is no problem with the notes.
    #[cfg(feature = "tray")]
    if let Some(line) = tray::doctor_line(tray::host_present()) {
        say(&safe(&line))?;
    }
    Ok(())
}
