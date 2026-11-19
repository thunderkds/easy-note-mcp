//! Obsidian views of AI detail (FR-036, DDR 0006): when the notes dir sits in an Obsidian vault
//! (a `.obsidian` folder in it or a parent), each AI file also shows as `<notes dir>/ai/<id>.md`,
//! a labelled, read-only page linked to the note. A view is output only: nothing reads it back,
//! the store does not list it as a note, and no note file is ever opened for writing here.
//!
//! The view name is the AI file name with `.md`: an inbox id `inbox/<stem>` gives
//! `inbox__<stem>.md`, so a name never holds `/`. Only a file that starts with the
//! `easy-note-ai-view: 1` marker is replaced or removed, so the person's own files in `ai/` stay.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::core::ai::{self, Detail, LABEL};
use crate::core::format::quote;
use crate::core::fsutil::{create_temp, unlink_quiet, write_all_sync};
use crate::core::guard;
use crate::core::store::Store;

const MARKER: &str = "---\neasy-note-ai-view: 1\n";
const DIR: &str = "ai";

/// The nearest of the notes dir and its parents that holds a `.obsidian` directory.
pub fn vault_root(notes_dir: &Path) -> Option<PathBuf> {
    let start = std::path::absolute(notes_dir).unwrap_or_else(|_| notes_dir.to_path_buf());
    start
        .ancestors()
        .find(|dir| dir.join(".obsidian").is_dir())
        .map(Path::to_path_buf)
}

fn view_path(root: &Path, note_id: &str) -> PathBuf {
    let name = ai::file_name(note_id);
    let stem = name.strip_suffix(".json").unwrap_or(&name);
    root.join(DIR).join(format!("{stem}.md"))
}

fn mask(text: &str) -> String {
    guard::mask(text).0
}

/// One line of masked text: line breaks become spaces.
fn one_line(text: &str) -> String {
    mask(text).split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The Markdown of one view. Pure; every string is masked, front matter strings are quoted.
pub fn render(detail: &Detail, note_id: &str, stale: bool) -> String {
    let link = note_id.strip_prefix("inbox/").unwrap_or(note_id);
    let tags: Vec<String> = detail.tags_suggested.iter().map(|t| mask(t)).collect();
    let quoted: Vec<String> = tags.iter().map(|t| quote(t)).collect();
    let mut out = String::from(MARKER);
    out.push_str(&format!(
        "note: {}\n",
        quote(&format!("[[{}]]", mask(link)))
    ));
    out.push_str(&format!("model: {}\n", quote(&mask(&detail.model))));
    out.push_str(&format!("at: {}\n", quote(&mask(&detail.at))));
    out.push_str(&format!("kind: {}\n", quote(&mask(&detail.kind))));
    out.push_str(&format!("stale: {stale}\n"));
    out.push_str(&format!("tags_suggested: [{}]\n---\n\n", quoted.join(", ")));
    out.push_str(&format!("# {LABEL}\n\n"));
    out.push_str(&format!("## {}\n\n", one_line(&detail.title)));
    out.push_str(&format!("{}\n", mask(&detail.summary)));
    if !tags.is_empty() {
        let shown: Vec<String> = tags.iter().map(|t| format!("#{t}")).collect();
        out.push_str(&format!("\nSuggested tags: {}\n", shown.join(" ")));
        out.push_str(&format!(
            "\nAccept with: easy-note tag {} --accept-ai\n",
            one_line(note_id)
        ));
    }
    out
}

/// Whether `path` is a view of ours (or absent).
fn ours_or_absent(path: &Path) -> bool {
    match fs::read(path) {
        Ok(raw) => raw.starts_with(MARKER.as_bytes()),
        Err(e) => e.kind() == io::ErrorKind::NotFound,
    }
}

/// Write the view of one note from its AI file (temp file, fsync, rename). Nothing outside a
/// vault; no AI file removes a view of ours. Never replaces a file that is not a view of ours.
pub fn write_view(store: &Store, note_id: &str) -> io::Result<()> {
    if vault_root(&store.root).is_none() {
        return Ok(());
    }
    let Ok(id) = store.canonical_id(note_id) else {
        return Ok(());
    };
    let Some((detail, stale)) = ai::read(store, &id) else {
        return remove_at(&store.root, &id);
    };
    let target = view_path(&store.root, &id);
    if !ours_or_absent(&target) {
        return Ok(());
    }
    let folder = store.root.join(DIR);
    fs::create_dir_all(&folder)?;
    let (mut file, tmp) = create_temp(&folder)?;
    let result = write_all_sync(&mut file, render(&detail, &id, stale).as_bytes())
        .and_then(|()| (store.rename)(&tmp, &target));
    unlink_quiet(&tmp);
    result
}

fn remove_at(root: &Path, note_id: &str) -> io::Result<()> {
    let target = view_path(root, note_id);
    if !target.is_file() || !ours_or_absent(&target) {
        return Ok(());
    }
    fs::remove_file(target)
}

/// Delete the view of a note (a resolved id, which may no longer be live); only a view of ours.
pub fn remove_view(store: &Store, note_id: &str) -> io::Result<()> {
    if vault_root(&store.root).is_none() {
        return Ok(());
    }
    remove_at(&store.root, note_id)
}

/// `write_view` that never fails its caller: a failure goes to stderr once.
pub fn sync(store: &Store, note_id: &str) {
    if let Err(e) = write_view(store, note_id) {
        eprintln!("easy-note: AI view not written: {e}");
    }
}

/// `remove_view` that never fails its caller.
pub fn sync_remove(store: &Store, note_id: &str) {
    if let Err(e) = remove_view(store, note_id) {
        eprintln!("easy-note: AI view not removed: {e}");
    }
}

/// Rewrite every view from the AI files: the notes that have one. `None` outside a vault.
pub fn render_all(store: &Store) -> io::Result<Option<usize>> {
    if vault_root(&store.root).is_none() {
        return Ok(None);
    }
    let mut written = 0;
    for item in store.iter_notes() {
        if let crate::core::store::Item::Note(note) = item {
            if ai::read(store, note.id()).is_some() {
                write_view(store, note.id())?;
                written += 1;
            }
        }
    }
    Ok(Some(written))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Detail {
        Detail {
            version: 1,
            id: "i".into(),
            title: "T\nitle".into(),
            kind: "bug".into(),
            summary: "s".into(),
            tags_suggested: vec!["a".into(), "b".into()],
            based_on: "sha256:x".into(),
            model: "m".into(),
            at: "2026-11-19T10:00:00+07:00".into(),
        }
    }

    #[test]
    fn render_has_front_matter_label_tags_and_accept_line() {
        let text = render(&sample(), "20261119-100000-ab12", true);
        assert!(text.starts_with("---\neasy-note-ai-view: 1\nnote: \"[[20261119-100000-ab12]]\"\n"));
        assert!(text.contains("stale: true\n"));
        assert!(text.contains("tags_suggested: [\"a\", \"b\"]\n"));
        assert!(text.contains("# AI detail (not your words)\n\n## T itle\n\ns\n"));
        assert!(text.contains("Suggested tags: #a #b\n"));
        assert!(text.contains("Accept with: easy-note tag 20261119-100000-ab12 --accept-ai\n"));
    }

    #[test]
    fn inbox_link_has_no_slash_and_the_name_is_safe() {
        let text = render(&sample(), "inbox/phone", false);
        assert!(text.contains("note: \"[[phone]]\"\n"));
        assert_eq!(
            view_path(Path::new("/n"), "inbox/phone"),
            Path::new("/n/ai/inbox__phone.md")
        );
    }

    #[test]
    fn vault_root_finds_a_dir_not_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("v");
        let notes = vault.join("a/notes");
        fs::create_dir_all(&notes).unwrap();
        fs::write(vault.join(".obsidian"), "x").unwrap();
        assert_ne!(vault_root(&notes), Some(vault.clone()));
        fs::remove_file(vault.join(".obsidian")).unwrap();
        fs::create_dir(vault.join(".obsidian")).unwrap();
        assert_eq!(vault_root(&notes), Some(vault));
    }
}
