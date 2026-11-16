//! The commands that only read: `ls`, `search`, `show`, `stats`, `doctor`, `path`. Every note
//! text they print is masked again (FR-014), whatever the core already did.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use super::{count, safe, Fail, Outcome};
use crate::core::query::{self, Counts, Filters, Page, Stats};
use crate::core::store::{Item, Store};
use crate::core::{config, format, guard};

fn print_page(page: &Page) {
    for item in &page.items {
        let mut line = format!("{}  {}", item.id, item.title);
        for tag in &item.tags {
            line.push_str(&format!("  #{tag}"));
        }
        println!("{}", safe(&line));
    }
    if page.truncated {
        eprintln!(
            "easy-note: showing {} of {} (use -n for more, up to {})",
            page.items.len(),
            page.total,
            query::MAX_LIMIT
        );
    }
}

pub fn ls(filters: Filters, limit: usize) -> Outcome {
    print_page(&query::list_notes(&filters, limit)?);
    Ok(())
}

pub fn search(words: &str, filters: Filters, limit: usize) -> Outcome {
    print_page(&query::search(words, &filters, limit)?);
    Ok(())
}

/// The note as its file reads, front matter and text, with secrets masked.
pub fn show(id: &str) -> Outcome {
    let note = Store::open()?.read(id)?;
    print!("{}", safe(&format::render(&note)));
    Ok(())
}

pub fn path() -> Outcome {
    println!("{}", safe(&config::notes_dir()?.display().to_string()));
    Ok(())
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
        println!("{}", safe(&text));
    } else {
        println!("{}", safe(&stats_text(&stats)));
    }
    Ok(())
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

/// `doctor`: report, never fix. Skipped files, kinds of secret in hand-edited files, an `n` clash.
pub fn doctor() -> Outcome {
    let store = Store::open()?;
    println!("notes dir: {}", safe(&store.root.display().to_string()));
    let (mut notes, mut problems) = (0, 0);
    for item in store.iter_notes() {
        match item {
            Item::Skipped(file) => {
                problems += 1;
                let shown = format!("skipped: {} ({})", file.path.display(), file.reason);
                println!("{}", safe(&shown));
            }
            Item::Note(note) => {
                notes += 1;
                // The store masks what it yields, so read the file itself for what is in it.
                let Some(path) = &note.path else { continue };
                let Ok(raw) = fs::read_to_string(path) else {
                    continue;
                };
                let (_, kinds) = guard::mask(&raw);
                if !kinds.is_empty() {
                    problems += 1;
                    let mut unique: Vec<&str> = vec![];
                    for kind in kinds {
                        if !unique.contains(&kind) {
                            unique.push(kind);
                        }
                    }
                    let shown = format!("secrets in {}: {}", path.display(), unique.join(", "));
                    println!("{}", safe(&shown));
                }
            }
        }
    }
    if let Some(other) = n_clash() {
        problems += 1;
        println!(
            "clash: `n` on your PATH is {}, not easy-note",
            safe(&other.display().to_string())
        );
    }
    println!("{} checked", count(notes, "note"));
    if problems == 0 {
        println!("no problems found");
    } else {
        println!("{} found", count(problems, "problem"));
    }
    Ok(())
}
