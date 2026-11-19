//! The commands that change the notes dir: `init`, `add`, `edit`, `tag`, `rm`, `restore`, `trash`,
//! `ai rm`.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::scratch::Scratch;
use super::{confirmation, count, safe, say, Fail, Outcome};
use crate::core::format::{parse, render, Note, Value};
use crate::core::store::{trash_label, AiCarry, Store};
use crate::core::{ai, config, context, guard, tags};

/// `init [--dir PATH]`: make the dir and save it in the config file (FR-001).
pub fn init(dir: Option<PathBuf>) -> Outcome {
    let dir = match dir {
        Some(dir) => std::path::absolute(dir)?,
        None => config::notes_dir()?,
    };
    fs::create_dir_all(&dir)?;
    config::save_dir(&dir)?;
    if std::env::var_os("EASY_NOTE_DIR").is_some_and(|v| !v.is_empty()) {
        eprintln!("easy-note: EASY_NOTE_DIR is set and overrides the config file; unset it to use the saved dir");
    }
    say(&format!("✓ notes dir {}", safe(&dir.display().to_string())))
}

/// Read the note text when none is on the command line: one line at a terminal, else all of it.
fn read_input(tty: bool, mut input: impl BufRead, mut prompt: impl Write) -> io::Result<String> {
    let mut text = String::new();
    if tty {
        write!(prompt, "note> ")?;
        prompt.flush()?;
        input.read_line(&mut text)?;
    } else {
        input.read_to_string(&mut text)?;
    }
    Ok(text.trim_end().to_string())
}

/// `add` and `n`: save the words as one note, print one line (FR-020, FR-021). Text that is
/// empty or only white space saves nothing and is not an error.
pub fn add(edit: bool, words: &[String]) -> Outcome {
    let (dir, after_save) = config::notes_dir_and_after_save()?;
    let store = Store::system(dir);
    let given = words.join(" ");
    // The editor sees a file, so a secret in the given words is masked before the file exists.
    let mut masked_first: Vec<&'static str> = vec![];
    let text = if edit {
        let (given, kinds) = guard::mask(&given);
        masked_first = kinds;
        edit_text(&store, "new", &given)?
    } else if !words.is_empty() {
        given
    } else {
        let stdin = io::stdin();
        read_input(stdin.is_terminal(), stdin.lock(), io::stderr())?
    };
    if text.trim().is_empty() {
        return Ok(());
    }
    let found = context::capture(None, None, &[]);
    let added = store.add(&text, "cli", &tags::extract(&text), &found.fields())?;
    if added.created_dir {
        eprintln!(
            "easy-note: created notes dir {}",
            safe(&store.root.display().to_string())
        );
    }
    masked_first.extend(&added.masked);
    let shown = say(&confirmation(
        "noted",
        &added.id,
        &[found.repo.as_deref(), found.task.as_deref()],
        &masked_first,
    ));
    if after_save {
        crate::surfaces::enrich::after_save(&store.root, &added.id);
    }
    shown
}

/// Open the editor on `path`, the way git does: `$VISUAL`, else `$EDITOR`, else `vi`, run by
/// `sh` so the command may hold quotes and arguments (`"/my editors/ed" --wait`).
fn run_editor(path: &Path) -> Outcome {
    let pick = |var: &str| std::env::var(var).ok().filter(|v| !v.trim().is_empty());
    let editor = pick("VISUAL")
        .or_else(|| pick("EDITOR"))
        .unwrap_or_else(|| "vi".to_string());
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("{editor} \"$@\""))
        .arg("sh")
        .arg(path)
        .status()
        .map_err(|e| Fail::runtime(format!("cannot run the editor {editor}: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(Fail::runtime(format!(
            "the editor {editor} ended with {status}; nothing saved"
        )))
    }
}

/// Text of a new note typed in the editor.
fn edit_text(store: &Store, name: &str, start: &str) -> Result<String, Fail> {
    let scratch = Scratch::create(store, name, start)?;
    run_editor(&scratch.path)?;
    Ok(fs::read_to_string(&scratch.path)?.trim_end().to_string())
}

fn tags_of(note: &Note) -> Vec<String> {
    match note.fields.get("tags") {
        Some(Value::List(list)) => list.clone(),
        _ => vec![],
    }
}

/// Unknown front-matter keys of a note, each with its YAML text.
fn extra_by_key(note: &Note) -> BTreeMap<&str, &str> {
    note.extra
        .iter()
        .map(|chunk| {
            let key = chunk.split(':').next().unwrap_or_default().trim();
            (key, chunk.trim_end_matches('\n'))
        })
        .collect()
}

/// The first front-matter key, other than `tags` and `updated`, that was added, removed or changed;
/// known keys first, then unknown keys.
fn changed_key(old: &Note, new: &Note) -> Option<String> {
    let skip = |key: &&String| !matches!(key.as_str(), "tags" | "updated");
    let known = old
        .fields
        .keys()
        .chain(new.fields.keys())
        .filter(skip)
        .find(|key| old.fields.get(*key) != new.fields.get(*key))
        .cloned();
    known.or_else(|| {
        let (before, after) = (extra_by_key(old), extra_by_key(new));
        before
            .keys()
            .chain(after.keys())
            .find(|key| before.get(*key) != after.get(*key))
            .map(|key| key.to_string())
    })
}

/// The tags to save after an edit: those in the edited front matter (a new one must be a tag),
/// plus the `#words` of the new text when the text changed (FR-006). `None`: no change.
fn edited_tags(old: &Note, new: &Note, text_changed: bool) -> Result<Option<Vec<String>>, String> {
    let before = tags_of(old);
    let mut wanted: Vec<String> = vec![];
    for tag in tags_of(new) {
        let tag = if before.contains(&tag) {
            tag
        } else {
            tags::normalize(&tag).ok_or_else(|| format!("not a tag: {tag}"))?
        };
        if !wanted.contains(&tag) {
            wanted.push(tag);
        }
    }
    if text_changed {
        for tag in tags::extract(&new.body) {
            if !wanted.contains(&tag) {
                wanted.push(tag);
            }
        }
    }
    Ok((wanted != before).then_some(wanted))
}

/// `edit <id>`: the whole file in the editor, then parse it again, mask, save (FR-013, FR-014).
/// Only the text and the tags are saved; any other change refuses the edit.
pub fn edit(id: &str) -> Outcome {
    let store = Store::open()?;
    let note = store.read(id)?;
    store.ensure_writable(id)?;
    let scratch = Scratch::create(&store, id, &render(&note))?;
    run_editor(&scratch.path)?;
    let edited = fs::read_to_string(&scratch.path)?;
    let refuse = |why: String| Fail::runtime(format!("edit refused: {why}; the note is unchanged"));
    let new = parse(&edited).map_err(|e| refuse(e.reason))?;
    if new.id() != note.id() {
        return Err(refuse("the id was changed".to_string()));
    }
    if let Some(key) = changed_key(&note, &new) {
        return Err(refuse(format!(
            "only the text and tags can change, not {key}"
        )));
    }
    let text = (new.body != note.body).then_some(new.body.as_str());
    if text.is_some_and(|t| t.trim().is_empty()) {
        return Err(refuse(
            "the text is empty (use rm to delete a note)".to_string(),
        ));
    }
    let tags = edited_tags(&note, &new, text.is_some()).map_err(refuse)?;
    if text.is_none() && tags.is_none() {
        eprintln!("easy-note: no changes");
        return Ok(());
    }
    let updated = store.update(id, text, tags.as_deref())?;
    say(&confirmation("edited", id, &[], &updated.masked))
}

/// `tag <id> <tags...>`, or `--remove`; `--accept-ai` adds the tags the AI detail suggests, the
/// only way AI text reaches a note, by the person (DDR 0004).
pub fn tag(id: &str, remove: bool, accept_ai: bool, wanted: &[String]) -> Outcome {
    let mut wanted: Vec<String> = wanted
        .iter()
        .map(|t| {
            tags::normalize(t)
                .ok_or_else(|| Fail::usage(format!("not a tag: {t} (use a-z, 0-9, _, / and -)")))
        })
        .collect::<Result<_, _>>()?;
    let store = Store::open()?;
    let note = store.read(id)?;
    if accept_ai {
        store.ensure_writable(note.id())?;
        let (detail, _) = ai::read(&store, note.id())
            .ok_or_else(|| Fail::runtime(format!("no AI detail for {id}")))?;
        // A masked tag is not a tag: only valid ones are taken.
        wanted.extend(
            detail
                .tags_suggested
                .iter()
                .filter_map(|t| tags::normalize(t)),
        );
    }
    let mut now = tags_of(&note);
    if remove {
        now.retain(|t| !wanted.contains(t));
    } else {
        for tag in wanted {
            if !now.contains(&tag) {
                now.push(tag);
            }
        }
    }
    let updated = store.update(id, None, Some(&now))?;
    let shown = if now.is_empty() {
        "no tags".to_string()
    } else {
        now.iter()
            .map(|t| format!("#{t}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    say(&confirmation(
        "tagged",
        id,
        &[Some(&shown)],
        &updated.masked,
    ))
}

pub fn rm(id: &str) -> Outcome {
    match Store::open()?.rm_with_ai(id)?.1 {
        AiCarry::Done => {}
        AiCarry::InTheWay => eprintln!("easy-note: its AI detail stays in .easy-note/ai/: a file of that name is already in .easy-note/trash/ai/"),
        AiCarry::Failed => eprintln!("easy-note: its AI detail could not be moved to the trash; it stays in .easy-note/ai/"),
    }
    let id = safe(id);
    say(&format!(
        "✓ removed {id} (easy-note restore {id} brings it back)"
    ))
}

pub fn restore(id: &str) -> Outcome {
    // Either way the trashed copy has no note in the trash now: the next empty deletes it.
    match Store::open()?.restore_with_ai(id)?.1 {
        AiCarry::Done => {}
        AiCarry::InTheWay => eprintln!("easy-note: its AI detail in .easy-note/ai/ is kept; the trashed AI detail stays in .easy-note/trash/ai/ and is deleted at the next `trash --empty`"),
        AiCarry::Failed => eprintln!("easy-note: its trashed AI detail could not be moved back; it stays in .easy-note/trash/ai/ and is deleted at the next `trash --empty`"),
    }
    say(&format!("✓ restored {}", safe(id)))
}

/// `ai rm <id>`: delete the note's AI detail file only (exit 1 when it has none).
pub fn ai_rm(id: &str) -> Outcome {
    let store = Store::open()?;
    if !ai::remove(&store, id)? {
        return Err(Fail::runtime(format!("no AI detail for {id}")));
    }
    say(&format!("✓ removed AI detail of {}", safe(id)))
}

/// `30d` is 30 days.
fn parse_days(text: &str) -> Result<Duration, Fail> {
    // No `{:?}` below: it escapes quotes, and the guard cannot match an escaped secret.
    text.strip_suffix('d')
        .and_then(|n| n.parse::<u64>().ok())
        .and_then(|n| n.checked_mul(86_400))
        .map(Duration::from_secs)
        .ok_or_else(|| Fail::usage(format!("--older-than must be days, like 30d, not {text}")))
}

/// `trash` lists the trash; `trash --empty --older-than 30d` deletes old files for good (FR-009).
pub fn trash(empty: bool, older_than: Option<&str>) -> Outcome {
    let store = Store::open()?;
    if empty {
        let age = parse_days(older_than.unwrap_or_default())?;
        let gone = store.empty_trash(age)?;
        for path in &gone {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            say(&format!("deleted {}", safe(&trash_label(&name))))?;
        }
        return say(&format!(
            "✓ deleted {} from the trash",
            count(gone.len(), "note")
        ));
    }
    for id in store.trash_ids() {
        say(&safe(&id))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_a_terminal_one_line_is_read_and_the_prompt_goes_to_the_prompt_stream() {
        let mut prompt = vec![];
        let got = read_input(true, "first line\nsecond\n".as_bytes(), &mut prompt).unwrap();
        assert_eq!(got, "first line");
        assert_eq!(String::from_utf8(prompt).unwrap(), "note> ");
    }

    #[test]
    fn at_a_terminal_an_empty_line_is_empty_text() {
        assert_eq!(read_input(true, "\n".as_bytes(), vec![]).unwrap(), "");
    }

    #[test]
    fn from_a_pipe_all_of_the_input_is_read_with_no_prompt() {
        let mut prompt = vec![];
        let got = read_input(false, "a\nb\n\n".as_bytes(), &mut prompt).unwrap();
        assert_eq!(got, "a\nb");
        assert!(prompt.is_empty());
    }

    #[test]
    fn days_parse() {
        assert_eq!(parse_days("30d").unwrap(), Duration::from_secs(30 * 86_400));
        assert_eq!(parse_days("0d").unwrap(), Duration::ZERO);
        for bad in ["", "30", "d", "-1d", "1.5d", "30h"] {
            assert!(parse_days(bad).is_err(), "{bad:?}");
        }
    }
}
