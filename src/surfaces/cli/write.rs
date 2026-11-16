//! The commands that change the notes dir: `init`, `add`, `edit`, `tag`, `rm`, `restore`, `trash`.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, IsTerminal, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::{confirmation, count, safe, Fail, Outcome};
use crate::core::format::{parse, render, Note, Value};
use crate::core::store::Store;
use crate::core::{config, context, tags};

/// `init [--dir PATH]`: make the dir and save it in the config file (FR-001).
pub fn init(dir: Option<PathBuf>) -> Outcome {
    let dir = match dir {
        Some(dir) => std::path::absolute(dir)?,
        None => config::notes_dir()?,
    };
    fs::create_dir_all(&dir)?;
    config::save_dir(&dir)?;
    println!("✓ notes dir {}", safe(&dir.display().to_string()));
    Ok(())
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
    let store = Store::open()?;
    let given = words.join(" ");
    let text = if edit {
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
    println!(
        "{}",
        confirmation(
            "noted",
            &added.id,
            &[found.repo.as_deref(), found.task.as_deref()],
            &added.masked
        )
    );
    Ok(())
}

/// A temp file in `<notes dir>/.easy-note/edit/` for the editor (the only place easy-note writes
/// besides notes and config). Removed when dropped, with the folders this run made.
struct Scratch {
    path: PathBuf,
    root: PathBuf,
    root_existed: bool,
}

impl Scratch {
    fn create(store: &Store, name: &str, content: &str) -> io::Result<Scratch> {
        let root_existed = store.root.exists();
        let dir = store.root.join(".easy-note").join("edit");
        fs::create_dir_all(&dir)?;
        let scratch = Scratch {
            path: dir.join(format!("{name}-{}.md", std::process::id())),
            root: store.root.clone(),
            root_existed,
        };
        let mut file: File = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&scratch.path)?;
        file.write_all(content.as_bytes())?;
        Ok(scratch)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
        let Some(edit_dir) = self.path.parent().filter(|p| p.ends_with("edit")) else {
            return;
        };
        // `remove_dir` fails on a folder that holds anything, so a person's files are safe.
        let _ = fs::remove_dir(edit_dir);
        if let Some(hidden) = edit_dir.parent() {
            let _ = fs::remove_dir(hidden);
        }
        if !self.root_existed {
            let _ = fs::remove_dir(&self.root);
        }
    }
}

/// Open `$EDITOR` (else `vi`) on `path`; it may hold arguments, like `code -w`.
fn run_editor(path: &Path) -> Outcome {
    let var = std::env::var("EDITOR").unwrap_or_default();
    let mut parts = var.split_whitespace();
    let program = parts.next().unwrap_or("vi");
    let status = Command::new(program)
        .args(parts)
        .arg(path)
        .status()
        .map_err(|e| Fail::runtime(format!("cannot run the editor {program:?}: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(Fail::runtime(format!(
            "the editor {program:?} ended with {status}; nothing saved"
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

/// The first front-matter key, other than `tags` and `updated`, whose value differs.
fn changed_key(old: &Note, new: &Note) -> Option<String> {
    let skip = |key: &&String| !matches!(key.as_str(), "tags" | "updated");
    old.fields
        .keys()
        .chain(new.fields.keys())
        .filter(skip)
        .find(|key| old.fields.get(*key) != new.fields.get(*key))
        .cloned()
}

/// `edit <id>`: the whole file in `$EDITOR`, then parse it again, mask, save (FR-013, FR-014).
/// Only the text and the tags are saved; any other change refuses the edit.
pub fn edit(id: &str) -> Outcome {
    let store = Store::open()?;
    let note = store.read(id)?;
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
            "only the text and tags can change, not {key:?}"
        )));
    }
    let text = (new.body != note.body).then_some(new.body.as_str());
    if text.is_some_and(|t| t.trim().is_empty()) {
        return Err(refuse(
            "the text is empty (use rm to delete a note)".to_string(),
        ));
    }
    let new_tags = tags_of(&new);
    let tags = (new_tags != tags_of(&note)).then_some(new_tags);
    if text.is_none() && tags.is_none() {
        eprintln!("easy-note: no changes");
        return Ok(());
    }
    let updated = store.update(id, text, tags.as_deref())?;
    println!("{}", confirmation("edited", id, &[], &updated.masked));
    Ok(())
}

/// `tag <id> <tags...>`, or `--remove`.
pub fn tag(id: &str, remove: bool, wanted: &[String]) -> Outcome {
    let wanted: Vec<String> = wanted
        .iter()
        .map(|t| {
            tags::normalize(t)
                .ok_or_else(|| Fail::usage(format!("not a tag: {t} (use a-z, 0-9, _, / and -)")))
        })
        .collect::<Result<_, _>>()?;
    let store = Store::open()?;
    let mut now = tags_of(&store.read(id)?);
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
    println!(
        "{}",
        confirmation("tagged", id, &[Some(&shown)], &updated.masked)
    );
    Ok(())
}

pub fn rm(id: &str) -> Outcome {
    Store::open()?.rm(id)?;
    println!("✓ removed {id} (easy-note restore {id} brings it back)");
    Ok(())
}

pub fn restore(id: &str) -> Outcome {
    Store::open()?.restore(id)?;
    println!("✓ restored {id}");
    Ok(())
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

fn stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// `trash` lists the trash; `trash --empty --older-than 30d` deletes old files for good (FR-009).
pub fn trash(empty: bool, older_than: Option<&str>) -> Outcome {
    let store = Store::open()?;
    if empty {
        let age = parse_days(older_than.unwrap_or_default())?;
        let gone = store.empty_trash(age)?;
        for path in &gone {
            println!("deleted {}", stem(path));
        }
        println!("✓ deleted {} from the trash", count(gone.len(), "note"));
        return Ok(());
    }
    // The layout is the store's: `<dir>/.easy-note/trash/<id>.md`.
    let mut ids: Vec<String> = match fs::read_dir(store.root.join(".easy-note").join("trash")) {
        Ok(entries) => entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "md"))
            .map(|p| stem(&p))
            .collect(),
        Err(_) => vec![],
    };
    ids.sort();
    for id in ids {
        println!("{}", safe(&id));
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
