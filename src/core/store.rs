//! The notes dir is the only store (DDR 0002): new IDs, never-overwrite atomic writes, trash,
//! skipped files.
//!
//! A new note is written to a temp file in its month folder, fsynced, then hard-linked to its
//! final name: the link fails if the name exists, so a write can never replace a file (NFR-005).
//! On a clash the ID gets `-2`, `-3`, ... A kill before the link leaves only a `.*.tmp` file, never
//! a partial `.md` note (FR-008).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, FixedOffset, Local, Offset, SecondsFormat, TimeZone, Utc};
use sha2::{Digest, Sha256};

use crate::core::config::{self, ConfigError};
use crate::core::format::{
    check_text, parse, render, FormatError, Note, TextTooLong, Value, SOURCES, VERSION,
};
use crate::core::fsutil::{create_temp, unlink_quiet, write_all_sync};

pub const CONTEXT_KEYS: [&str; 7] = [
    "repo", "remote", "branch", "task", "client", "session", "commit",
];
const MAX_CLASH: u32 = 1000;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// No note has this ID (the message is the ID).
    #[error("{0}")]
    NotFound(String),
    #[error("not a note id: {0:?}")]
    BadId(String),
    /// A bad argument: unknown source, a key that is not a context field.
    #[error("{0}")]
    Invalid(String),
    /// A move or write would replace a file that already exists.
    #[error("{0}")]
    Exists(String),
    #[error(transparent)]
    Format(#[from] FormatError),
    #[error(transparent)]
    TextTooLong(#[from] TextTooLong),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Added {
    pub id: String,
    pub path: PathBuf,
    /// True only for the add that created the notes dir.
    pub created_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    pub path: PathBuf,
    /// Never quotes file content.
    pub reason: String,
}

/// What `iter_notes` yields: a note, or a file it could not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Note(Note),
    Skipped(Skipped),
}

type FileOp = fn(&Path, &Path) -> io::Result<()>;

/// One notes dir. `Store::system` is the real thing; the other fields are seams that tests replace
/// (frozen clock, hostname and zone; a filesystem without hard links; a failing rename).
#[derive(Clone)]
pub struct Store {
    pub root: PathBuf,
    /// Nanoseconds since the epoch.
    pub now_ns: fn() -> u128,
    pub hostname: fn() -> String,
    /// Seconds east of UTC in force at the given Unix second.
    pub utc_offset: fn(i64) -> i32,
    pub link: FileOp,
    pub rename: FileOp,
}

fn system_now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos())
}

fn system_hostname() -> String {
    gethostname::gethostname().to_string_lossy().into_owned()
}

fn system_utc_offset(secs: i64) -> i32 {
    Local
        .timestamp_opt(secs, 0)
        .single()
        .map_or(0, |t| t.offset().fix().local_minus_utc())
}

impl Store {
    pub fn system(root: PathBuf) -> Store {
        Store {
            root,
            now_ns: system_now_ns,
            hostname: system_hostname,
            utc_offset: system_utc_offset,
            link: |src, dst| fs::hard_link(src, dst),
            rename: |src, dst| fs::rename(src, dst),
        }
    }

    /// The store on the notes dir from `EASY_NOTE_DIR`, config.toml or `~/easy-notes`.
    pub fn open() -> Result<Store, StoreError> {
        Ok(Store::system(config::notes_dir()?))
    }

    /// Local time of `ns`, to the second.
    fn local_time(&self, ns: u128) -> DateTime<FixedOffset> {
        let secs = i64::try_from(ns / 1_000_000_000).unwrap_or(0);
        let offset = FixedOffset::east_opt((self.utc_offset)(secs)).unwrap_or(Utc.fix());
        DateTime::<Utc>::from_timestamp(secs, 0)
            .unwrap_or_default()
            .with_timezone(&offset)
    }

    /// Write one new note. `fields` holds context keys (CONTEXT_KEYS); `None` values are left out.
    pub fn add(
        &self,
        text: &str,
        source: &str,
        tags: &[String],
        fields: &[(&str, Option<&str>)],
    ) -> Result<Added, StoreError> {
        check_text(text)?;
        if !SOURCES.contains(&source) || source == "inbox" {
            return Err(StoreError::Invalid(format!("unknown source {source:?}")));
        }
        let mut context = vec![];
        for (key, value) in fields {
            if !CONTEXT_KEYS.contains(key) {
                return Err(StoreError::Invalid(format!(
                    "{key:?} is not a context field"
                )));
            }
            if let Some(value) = value {
                context.push((key.to_string(), value.to_string()));
            }
        }

        let created_dir = match fs::create_dir(&self.root) {
            Ok(()) => true,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => false,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                fs::create_dir_all(&self.root)?;
                true
            }
            Err(e) => return Err(e.into()),
        };

        let ns = (self.now_ns)();
        let created = self.local_time(ns);
        let digest = Sha256::digest(format!("{ns}\n{}\n{text}", (self.hostname)()).as_bytes());
        let base_id = format!(
            "{}-{:02x}{:02x}",
            created.format("%Y%m%d-%H%M%S"),
            digest[0],
            digest[1]
        );
        let body = with_newline(text);
        let stamp = created.to_rfc3339_opts(SecondsFormat::Secs, false);
        let content = |note_id: &str| -> Vec<u8> {
            let mut meta = vec![
                ("easy-note".to_string(), Value::Int(VERSION)),
                ("id".to_string(), Value::Str(note_id.to_string())),
                ("created".to_string(), Value::Str(stamp.clone())),
            ];
            meta.extend(
                context
                    .iter()
                    .map(|(k, v)| (k.clone(), Value::Str(v.clone()))),
            );
            meta.push(("tags".to_string(), Value::List(tags.to_vec())));
            meta.push(("source".to_string(), Value::Str(source.to_string())));
            render(&Note::new(meta.into_iter().collect(), body.clone())).into_bytes()
        };

        let month = self
            .root
            .join(created.format("%Y").to_string())
            .join(created.format("%m").to_string());
        fs::create_dir_all(&month)?;
        let path = self.write_new(&month, &base_id, &content)?;
        let id = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        Ok(Added {
            id,
            path,
            created_dir,
        })
    }

    pub fn read(&self, note_id: &str) -> Result<Note, StoreError> {
        let path = self.note_path(note_id)?;
        let raw = match fs::read(&path) {
            Ok(raw) => raw,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(StoreError::NotFound(note_id.to_string()))
            }
            Err(e) => return Err(e.into()),
        };
        match parse_file(&path, &raw) {
            Item::Note(note) => Ok(note),
            Item::Skipped(skipped) => Err(FormatError {
                reason: skipped.reason,
            }
            .into()),
        }
    }

    /// Rewrite one note (`edit` / `tag`, FR-008): unknown keys kept, `updated` set, temp + rename.
    pub fn update(
        &self,
        note_id: &str,
        text: Option<&str>,
        tags: Option<&[String]>,
    ) -> Result<Note, StoreError> {
        if let Some(text) = text {
            check_text(text)?;
        }
        let mut note = self.read(note_id)?;
        if let Some(text) = text {
            note.body = with_newline(text);
        }
        if let Some(tags) = tags {
            note.fields
                .insert("tags".to_string(), Value::List(tags.to_vec()));
        }
        let stamp = self
            .local_time((self.now_ns)())
            .to_rfc3339_opts(SecondsFormat::Secs, false);
        note.fields.insert("updated".to_string(), Value::Str(stamp));
        let path = note.path.clone().expect("read sets the path");
        let dir = path.parent().expect("a note path has a parent");
        let (mut file, tmp) = create_temp(dir)?;
        let result = write_all_sync(&mut file, render(&note).as_bytes())
            .and_then(|()| (self.rename)(&tmp, &path));
        unlink_quiet(&tmp);
        result?;
        Ok(note)
    }

    /// Move a note to `<dir>/.easy-note/trash/`; its mtime becomes the time it was trashed.
    pub fn rm(&self, note_id: &str) -> Result<PathBuf, StoreError> {
        let src = self.note_path(note_id)?;
        if !src.is_file() {
            return Err(StoreError::NotFound(note_id.to_string()));
        }
        let dst = self.trash_dir().join(file_name(&src));
        fs::create_dir_all(self.trash_dir())?;
        self.move_no_replace(&src, &dst)?;
        fs::OpenOptions::new()
            .write(true)
            .open(&dst)?
            .set_modified(SystemTime::now())?;
        Ok(dst)
    }

    pub fn restore(&self, note_id: &str) -> Result<PathBuf, StoreError> {
        let dst = self.note_path(note_id)?;
        let src = self.trash_dir().join(file_name(&dst));
        if !src.is_file() {
            return Err(StoreError::NotFound(note_id.to_string()));
        }
        fs::create_dir_all(dst.parent().expect("a note path has a parent"))?;
        self.move_no_replace(&src, &dst)?;
        Ok(dst)
    }

    /// Delete trash files trashed more than `older_than` ago; return them, sorted.
    pub fn empty_trash(&self, older_than: Duration) -> Result<Vec<PathBuf>, StoreError> {
        let Ok(entries) = fs::read_dir(self.trash_dir()) else {
            return Ok(vec![]);
        };
        let cutoff = SystemTime::now()
            .checked_sub(older_than)
            .unwrap_or(UNIX_EPOCH);
        let mut gone = vec![];
        for entry in entries {
            let entry = entry?;
            let meta = entry.metadata()?;
            if meta.is_file() && meta.modified()? < cutoff {
                gone.push(entry.path());
            }
        }
        gone.sort();
        for path in &gone {
            fs::remove_file(path)?;
        }
        Ok(gone)
    }

    /// Every `<dir>/YYYY/MM/*.md`, in path order: a note, or Skipped with its path and reason
    /// (FR-017). Files are read one by one as the iterator advances.
    pub fn iter_notes(&self) -> impl Iterator<Item = Item> {
        let mut paths: Vec<PathBuf> = vec![];
        for year in digit_dirs(&self.root, 4) {
            for month in digit_dirs(&year, 2) {
                let Ok(entries) = fs::read_dir(&month) else {
                    continue;
                };
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.ends_with(".md") && !name.starts_with('.') && path.is_file() {
                        paths.push(path);
                    }
                }
            }
        }
        paths.sort();
        paths.into_iter().map(|path| {
            let path = std::path::absolute(&path).unwrap_or(path);
            match fs::read(&path) {
                Ok(raw) => parse_file(&path, &raw),
                Err(_) => Item::Skipped(Skipped {
                    path,
                    reason: "cannot read file".to_string(),
                }),
            }
        })
    }

    fn note_path(&self, note_id: &str) -> Result<PathBuf, StoreError> {
        if !is_note_id(note_id) {
            return Err(StoreError::BadId(note_id.to_string()));
        }
        Ok(self
            .root
            .join(&note_id[..4])
            .join(&note_id[4..6])
            .join(format!("{note_id}.md")))
    }

    fn trash_dir(&self) -> PathBuf {
        self.root.join(".easy-note").join("trash")
    }

    fn write_new(
        &self,
        folder: &Path,
        base_id: &str,
        content: &dyn Fn(&str) -> Vec<u8>,
    ) -> Result<PathBuf, StoreError> {
        let (mut file, tmp) = create_temp(folder)?; // exclusive create, mode 0600
        let result = self.link_new(&mut file, &tmp, folder, base_id, content);
        unlink_quiet(&tmp);
        result
    }

    fn link_new(
        &self,
        file: &mut fs::File,
        tmp: &Path,
        folder: &Path,
        base_id: &str,
        content: &dyn Fn(&str) -> Vec<u8>,
    ) -> Result<PathBuf, StoreError> {
        for n in 1..=MAX_CLASH {
            let note_id = if n == 1 {
                base_id.to_string()
            } else {
                format!("{base_id}-{n}")
            };
            let final_path = folder.join(format!("{note_id}.md"));
            write_all_sync(file, &content(&note_id))?;
            match (self.link)(tmp, &final_path) {
                Ok(()) => return Ok(final_path),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(_) => {
                    // No hard links here (some FUSE filesystems): reserve the name with an
                    // exclusive create, then swap the full content in. A kill in between leaves
                    // an empty file, which readers report as skipped.
                    match fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&final_path)
                    {
                        Ok(_) => {}
                        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                        Err(e) => return Err(e.into()),
                    }
                    if let Err(e) = (self.rename)(tmp, &final_path) {
                        unlink_quiet(&final_path);
                        return Err(e.into());
                    }
                    return Ok(final_path);
                }
            }
        }
        Err(StoreError::Exists(format!(
            "{MAX_CLASH} notes already use ID {base_id}"
        )))
    }

    fn move_no_replace(&self, src: &Path, dst: &Path) -> Result<(), StoreError> {
        let exists =
            || StoreError::Exists(format!("{} already exists; nothing moved", dst.display()));
        match (self.link)(src, dst) {
            Ok(()) => {
                fs::remove_file(src)?;
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Err(exists()),
            Err(_) => {
                // No hard links here: best-effort check, then rename.
                if dst.exists() {
                    return Err(exists());
                }
                (self.rename)(src, dst)?;
                Ok(())
            }
        }
    }
}

fn with_newline(text: &str) -> String {
    if text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{text}\n")
    }
}

fn file_name(path: &Path) -> std::ffi::OsString {
    path.file_name().unwrap_or_default().to_os_string()
}

/// Subdirectories of `dir` whose name is exactly `digits` ASCII digits.
fn digit_dirs(dir: &Path, digits: usize) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return vec![];
    };
    entries
        .flatten()
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.len() == digits && name.bytes().all(|b| b.is_ascii_digit()) && e.path().is_dir()
        })
        .map(|e| e.path())
        .collect()
}

fn parse_file(path: &Path, raw: &[u8]) -> Item {
    let skip = |reason: &str| {
        Item::Skipped(Skipped {
            path: path.to_path_buf(),
            reason: reason.to_string(),
        })
    };
    let Ok(text) = std::str::from_utf8(raw) else {
        return skip("not UTF-8");
    };
    match parse(text) {
        Err(e) => skip(&e.reason),
        Ok(mut note) => {
            if note.id() != path.file_stem().unwrap_or_default().to_string_lossy() {
                return skip("id does not match file name");
            }
            note.path = Some(path.to_path_buf());
            Item::Note(note)
        }
    }
}

/// `YYYYMMDD-HHMMSS-xxxx` with an optional `-N` clash suffix (N ≥ 1, no leading zero).
fn is_note_id(id: &str) -> bool {
    let b = id.as_bytes();
    if b.len() < 20
        || !b[..8].iter().all(u8::is_ascii_digit)
        || b[8] != b'-'
        || !b[9..15].iter().all(u8::is_ascii_digit)
        || b[15] != b'-'
        || !b[16..20]
            .iter()
            .all(|c| matches!(c, b'0'..=b'9' | b'a'..=b'f'))
    {
        return false;
    }
    match &b[20..] {
        [] => true,
        [b'-', n @ ..] => !n.is_empty() && n[0] != b'0' && n.iter().all(u8::is_ascii_digit),
        _ => false,
    }
}

// The free functions below are the entry points the surfaces use: one call on the notes dir from
// `EASY_NOTE_DIR`, config.toml or `~/easy-notes`.

pub fn add(
    text: &str,
    source: &str,
    tags: &[String],
    fields: &[(&str, Option<&str>)],
) -> Result<Added, StoreError> {
    Store::open()?.add(text, source, tags, fields)
}

pub fn read(note_id: &str) -> Result<Note, StoreError> {
    Store::open()?.read(note_id)
}

pub fn update(
    note_id: &str,
    text: Option<&str>,
    tags: Option<&[String]>,
) -> Result<Note, StoreError> {
    Store::open()?.update(note_id, text, tags)
}

pub fn rm(note_id: &str) -> Result<PathBuf, StoreError> {
    Store::open()?.rm(note_id)
}

pub fn restore(note_id: &str) -> Result<PathBuf, StoreError> {
    Store::open()?.restore(note_id)
}

pub fn empty_trash(older_than: Duration) -> Result<Vec<PathBuf>, StoreError> {
    Store::open()?.empty_trash(older_than)
}

pub fn iter_notes() -> Result<impl Iterator<Item = Item>, StoreError> {
    Ok(Store::open()?.iter_notes())
}
