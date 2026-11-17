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

use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, Offset, SecondsFormat, TimeZone, Utc};
use saphyr::{LoadableYamlNode, ScalarOwned, YamlOwned};
use sha2::{Digest, Sha256};

use crate::core::ai;
use crate::core::config::{self, ConfigError};
use crate::core::format::{
    check_text, parse, quote, render, FormatError, Note, TextTooLong, Value, SOURCES, VERSION,
};
use crate::core::fsutil::{create_temp, unlink_quiet, write_all_sync};
use crate::core::guard;
use crate::core::tags;

pub const CONTEXT_KEYS: [&str; 7] = [
    "repo", "remote", "branch", "task", "client", "session", "commit",
];
const MAX_CLASH: u32 = 1000;
/// An inbox id starts with this; the rest is a file name part (FR-023).
const INBOX_PREFIX: &str = "inbox/";
/// The trash name of an inbox file is this, the unix seconds of its mtime, `__`, and its file
/// name: `inbox__1790000000__a.txt`.
const TRASH_INBOX: &str = "inbox__";

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
    /// An inbox note: a sync app owns the file, so easy-note never writes it.
    #[error("inbox notes are read-only")]
    ReadOnly,
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
    /// Kinds the guard masked, in order: body, tags, fields as given. Never a value.
    pub masked: Vec<&'static str>,
}

/// What `update` returns: the note as written (masked) and the kinds masked in that write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Updated {
    pub note: Note,
    /// Kinds masked in what `update` wrote: body, tags, context fields, extra. Never a value.
    pub masked: Vec<&'static str>,
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
        // A secret never reaches the file; the note is still kept (D21).
        let (text, mut masked) = guard::mask(text);
        check_text(&text)?;
        let text = text.as_str();
        if !SOURCES.contains(&source) || source == "inbox" {
            return Err(StoreError::Invalid(format!("unknown source {source:?}")));
        }
        let mut masked_tags = Vec::with_capacity(tags.len());
        for tag in tags {
            let (tag, found) = guard::mask(tag);
            masked_tags.push(tag);
            masked.extend(found);
        }
        let tags = masked_tags.as_slice();
        let mut context = vec![];
        for (key, value) in fields {
            if !CONTEXT_KEYS.contains(key) {
                return Err(StoreError::Invalid(format!(
                    "{key:?} is not a context field"
                )));
            }
            if let Some(value) = value {
                let (value, found) = guard::mask(value);
                masked.extend(found);
                context.push((key.to_string(), value));
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
            masked,
        })
    }

    /// One note, its free text masked for output; the file is never rewritten by a read.
    pub fn read(&self, note_id: &str) -> Result<Note, StoreError> {
        let mut note = self.read_raw(note_id)?;
        mask_note(&mut note);
        Ok(note)
    }

    /// The canonical id of a note and its body exactly as stored, unmasked: only for
    /// `ai::based_on`, whose hash is the one thing derived from it that leaves the core.
    pub(crate) fn raw_body(&self, note_id: &str) -> Result<(String, String), StoreError> {
        let note = self.read_raw(note_id)?;
        Ok((note.id().to_string(), note.body))
    }

    /// The canonical id of a live note (an inbox id as listed); `NotFound` when there is none.
    pub(crate) fn canonical_id(&self, note_id: &str) -> Result<String, StoreError> {
        let (id, path) = self.resolve(note_id)?;
        if !path.is_file() {
            return Err(StoreError::NotFound(id));
        }
        Ok(id)
    }

    /// Now, local time to the second, ISO 8601 with offset (the AI file's `at`).
    pub(crate) fn now_stamp(&self) -> String {
        self.local_time((self.now_ns)())
            .to_rfc3339_opts(SecondsFormat::Secs, false)
    }

    fn read_raw(&self, note_id: &str) -> Result<Note, StoreError> {
        let (note_id, path) = self.resolve(note_id)?;
        let raw = match fs::read(&path) {
            Ok(raw) => raw,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(StoreError::NotFound(note_id.to_string()))
            }
            Err(e) => return Err(e.into()),
        };
        let item = if note_id.starts_with(INBOX_PREFIX) {
            self.load_inbox(&note_id, &path, &raw)
        } else {
            parse_file(&path, &raw)
        };
        match item {
            Item::Note(note) => Ok(note),
            Item::Skipped(skipped) => Err(FormatError {
                reason: skipped.reason,
            }
            .into()),
        }
    }

    /// Inbox notes are read-only (FR-023): `Err(ReadOnly)` for an inbox id. Callers that would
    /// do work first (an editor session) ask before they start.
    pub fn ensure_writable(&self, note_id: &str) -> Result<(), StoreError> {
        if note_id.starts_with(INBOX_PREFIX) {
            return Err(StoreError::ReadOnly);
        }
        Ok(())
    }

    /// Rewrite one note (`edit` / `tag`, FR-008): unknown keys kept, `updated` set, temp + rename.
    pub fn update(
        &self,
        note_id: &str,
        text: Option<&str>,
        tags: Option<&[String]>,
    ) -> Result<Updated, StoreError> {
        self.ensure_writable(note_id)?;
        let mut masked = vec![];
        let mut new_body = None;
        if let Some(text) = text {
            let (text, found) = guard::mask(text);
            check_text(&text)?;
            masked = found;
            new_body = Some(with_newline(&text));
        }
        let mut note = self.read_raw(note_id)?;
        if let Some(body) = new_body {
            note.body = body;
        }
        if let Some(tags) = tags {
            note.fields
                .insert("tags".to_string(), Value::List(tags.to_vec()));
        }
        // Also masks a secret a person typed into the file by hand, in any free-text part.
        masked.extend(mask_note(&mut note));
        // Each chunk is one YAML key and must stay newline-ended: an unclosed private key masks
        // to the chunk end.
        for chunk in &mut note.extra {
            *chunk = format!("{}\n", chunk.trim_end_matches('\n'));
        }
        let stamp = self
            .local_time((self.now_ns)())
            .to_rfc3339_opts(SecondsFormat::Secs, false);
        note.fields.insert("updated".to_string(), Value::Str(stamp));
        let path = note.path.clone().expect("read sets the path");
        let dir = path.parent().expect("a note path has a parent");
        let data = render(&note);
        // Never write a file the reader cannot parse (masking can break a quoted value).
        if parse(&data).is_err() {
            return Err(FormatError {
                reason: "masking would leave the note unreadable; nothing written".to_string(),
            }
            .into());
        }
        let (mut file, tmp) = create_temp(dir)?;
        let result =
            write_all_sync(&mut file, data.as_bytes()).and_then(|()| (self.rename)(&tmp, &path));
        unlink_quiet(&tmp);
        result?;
        Ok(Updated { note, masked })
    }

    /// Move a note to `<dir>/.easy-note/trash/`; its mtime becomes the time it was trashed.
    pub fn rm(&self, note_id: &str) -> Result<PathBuf, StoreError> {
        self.rm_with_ai(note_id).map(|(path, _)| path)
    }

    /// `rm`; the note's AI file follows it to `trash/ai/<note trash file name>.json`. The bool is
    /// `false` when it could not (a file of that name is in the way): both files stay where they
    /// are and the note move still stands.
    pub fn rm_with_ai(&self, note_id: &str) -> Result<(PathBuf, bool), StoreError> {
        let (note_id, src) = self.resolve(note_id)?;
        if !src.is_file() {
            return Err(StoreError::NotFound(note_id));
        }
        // The trash name of an inbox file carries its origin and its mtime, so `restore` can
        // give the file its mtime back (`created` of an undated inbox note is the mtime).
        let name = match note_id.starts_with(INBOX_PREFIX) {
            true => {
                let file = file_name(&src).to_string_lossy().into_owned();
                if self
                    .trashed_inbox()
                    .iter()
                    .any(|(_, _, name)| *name == file)
                {
                    return Err(StoreError::Exists(format!(
                        "{note_id} is already in the trash; nothing moved"
                    )));
                }
                let secs = fs::metadata(&src)?
                    .modified()?
                    .duration_since(UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs());
                format!("{TRASH_INBOX}{secs}__{file}")
            }
            false => file_name(&src).to_string_lossy().into_owned(),
        };
        let dst = self.trash_dir().join(&name);
        fs::create_dir_all(self.trash_dir())?;
        self.move_no_replace(&src, &dst)?;
        // Open read-only: a 0444 note was moved already and a write-open would fail on it.
        fs::File::open(&dst)?.set_modified(SystemTime::now())?;
        let moved = self.carry_ai(
            &ai::path(&self.root, &note_id),
            &ai::trash_path(&self.root, &name),
        );
        Ok((dst, moved))
    }

    pub fn restore(&self, note_id: &str) -> Result<PathBuf, StoreError> {
        self.restore_with_ai(note_id).map(|(path, _)| path)
    }

    /// `restore`; the note's AI file comes back from `trash/ai/` too. The bool is `false` when it
    /// could not (a file of that name is in `.easy-note/ai/`): both files are kept.
    pub fn restore_with_ai(&self, note_id: &str) -> Result<(PathBuf, bool), StoreError> {
        let mut mtime = None;
        let (src, dst) = match inbox_part(note_id)? {
            Some(part) => {
                let trashed = self.trashed_inbox();
                let listed = inbox_ids(trashed.iter().map(|(_, _, n)| n.clone()).collect());
                let name = pick_inbox(&listed, part, note_id)?
                    .ok_or_else(|| StoreError::NotFound(note_id.to_string()))?;
                let (path, secs, _) = trashed
                    .into_iter()
                    .find(|(_, _, n)| *n == name)
                    .expect("pick_inbox answers with a listed name");
                mtime = Some(UNIX_EPOCH + Duration::from_secs(secs));
                (path, self.inbox_dir().join(name))
            }
            None => {
                let dst = self.note_path(note_id)?;
                (self.trash_dir().join(file_name(&dst)), dst)
            }
        };
        if !src.is_file() {
            return Err(StoreError::NotFound(note_id.to_string()));
        }
        fs::create_dir_all(dst.parent().expect("a note path has a parent"))?;
        self.move_no_replace(&src, &dst)?;
        if let Some(mtime) = mtime {
            fs::File::open(&dst)?.set_modified(mtime)?;
        }
        // The live id of the restored file names its AI file (an inbox id as listed now).
        let live_id = match mtime {
            Some(_) => self
                .inbox_files()
                .into_iter()
                .find(|(_, path)| file_name(path) == file_name(&dst))
                .map(|(id, _)| id),
            None => Some(note_id.to_string()),
        };
        let moved = match live_id {
            Some(id) => self.carry_ai(
                &ai::trash_path(&self.root, &file_name(&src).to_string_lossy()),
                &ai::path(&self.root, &id),
            ),
            None => false,
        };
        Ok((dst, moved))
    }

    /// Move an AI file with its note, never replacing a file. `true` when there was none to move.
    fn carry_ai(&self, src: &Path, dst: &Path) -> bool {
        if !src.is_file() {
            return true;
        }
        let made = dst.parent().map_or(Ok(()), fs::create_dir_all);
        made.is_ok() && self.move_no_replace(src, dst).is_ok()
    }

    /// Delete trash files trashed more than `older_than` ago; return them, sorted. A note's AI file
    /// goes with it, by the note's mtime only (a move keeps the AI file's old mtime), and so does
    /// any AI file in `trash/ai/` whose note is no longer in the trash.
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
            let ai_file = ai::trash_path(&self.root, &file_name(path).to_string_lossy());
            match fs::remove_file(ai_file) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e.into()),
                _ => {}
            }
        }
        if let Ok(entries) = fs::read_dir(self.trash_dir().join("ai")) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let orphan = name
                    .strip_suffix(".json")
                    .is_some_and(|note| !note.is_empty() && !self.trash_dir().join(note).exists());
                if orphan && entry.file_type().is_ok_and(|t| t.is_file()) {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        Ok(gone)
    }

    /// Every `<dir>/YYYY/MM/*.md`, in path order: a note, or Skipped with its path and reason
    /// (FR-017). Files are read one by one as the iterator advances.
    pub fn iter_notes(&self) -> impl Iterator<Item = Item> {
        self.items(false).map(|(item, _)| item)
    }

    /// `iter_notes`, each note with `ai::based_on` of its body as stored (before the mask).
    pub(crate) fn iter_hashed(&self) -> impl Iterator<Item = (Item, Option<String>)> {
        self.items(true)
    }

    fn items(&self, hash: bool) -> impl Iterator<Item = (Item, Option<String>)> {
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
        let me = self.clone();
        let dated = paths.into_iter().map(move |path| {
            let path = std::path::absolute(&path).unwrap_or(path);
            let item = match fs::read(&path) {
                Ok(raw) => parse_file(&path, &raw),
                Err(_) => Item::Skipped(Skipped {
                    path,
                    reason: "cannot read file".to_string(),
                }),
            };
            finish(item, hash)
        });
        // Then the inbox notes, parsed views of files that are never written (FR-023).
        let inbox = me.inbox_files().into_iter().map(move |(id, path)| {
            let item = match fs::read(&path) {
                Ok(raw) => me.load_inbox(&id, &path, &raw),
                Err(_) => Item::Skipped(Skipped {
                    path,
                    reason: "cannot read file".to_string(),
                }),
            };
            finish(item, hash)
        });
        dated.chain(inbox)
    }

    /// The ids of the notes in the trash that came from the inbox, and the dated ones, sorted;
    /// an inbox id is `inbox/<stem>` (the file name when two files share a stem).
    pub fn trash_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = match fs::read_dir(self.trash_dir()) {
            Ok(entries) => entries
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|name| !name.starts_with(TRASH_INBOX))
                .filter_map(|name| name.strip_suffix(".md").map(str::to_string))
                .collect(),
            Err(_) => vec![],
        };
        ids.extend(
            inbox_ids(self.trashed_inbox_names())
                .into_iter()
                .map(|(id, _)| id),
        );
        ids.sort();
        ids
    }

    /// `(trash path, mtime seconds, inbox file name)` of each trash file that came from the
    /// inbox, by file name. A trash name that is not `inbox__<digits>__<inbox file name>` is
    /// not listed.
    fn trashed_inbox(&self) -> Vec<(PathBuf, u64, String)> {
        let Ok(entries) = fs::read_dir(self.trash_dir()) else {
            return vec![];
        };
        let mut found: Vec<(PathBuf, u64, String)> = entries
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
            .filter_map(|e| {
                let name = e.file_name().into_string().ok()?;
                let (secs, file) = split_trash_inbox(&name)?;
                Some((e.path(), secs, file.to_string()))
            })
            .collect();
        found.sort_by(|a, b| a.2.cmp(&b.2));
        found
    }

    /// File names of the trash files that came from the inbox, sorted.
    fn trashed_inbox_names(&self) -> Vec<String> {
        self.trashed_inbox()
            .into_iter()
            .map(|(_, _, name)| name)
            .collect()
    }

    fn inbox_dir(&self) -> PathBuf {
        self.root.join("inbox")
    }

    /// `(id, path)` of each note file directly in `inbox/`, by file name. Only regular files
    /// with a `.md` or `.txt` name that does not start with `.`: no subfolder, no symlink.
    fn inbox_files(&self) -> Vec<(String, PathBuf)> {
        let dir = std::path::absolute(self.inbox_dir()).unwrap_or_else(|_| self.inbox_dir());
        let Ok(entries) = fs::read_dir(&dir) else {
            return vec![];
        };
        let mut names: Vec<String> = entries
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| is_inbox_file_name(name))
            .collect();
        names.sort();
        inbox_ids(names)
            .into_iter()
            .map(|(id, name)| (id, dir.join(name)))
            .collect()
    }

    /// The canonical id and the file of one note. An inbox id is checked before the disk is
    /// touched, and only a file `inbox_files` listed can be the answer, so the path never leaves
    /// `inbox/`.
    fn resolve(&self, note_id: &str) -> Result<(String, PathBuf), StoreError> {
        let Some(part) = inbox_part(note_id)? else {
            return Ok((note_id.to_string(), self.note_path(note_id)?));
        };
        let files = self.inbox_files();
        let names: Vec<(String, String)> = files
            .iter()
            .map(|(id, path)| (id.clone(), file_name(path).to_string_lossy().into_owned()))
            .collect();
        let name = pick_inbox(&names, part, note_id)?
            .ok_or_else(|| StoreError::NotFound(note_id.to_string()))?;
        files
            .into_iter()
            .find(|(_, path)| file_name(path).to_string_lossy() == name)
            .ok_or_else(|| StoreError::NotFound(note_id.to_string()))
    }

    /// One inbox file as a note (unmasked): its own `easy-note: 1` front matter when it has a
    /// valid one (the id stays the inbox id), else a view made from the name, the mtime and the
    /// text (NOTE_FORMAT, "Inbox files").
    fn load_inbox(&self, id: &str, path: &Path, raw: &[u8]) -> Item {
        let skip = |reason: &str| {
            Item::Skipped(Skipped {
                path: path.to_path_buf(),
                reason: reason.to_string(),
            })
        };
        let Ok(text) = std::str::from_utf8(raw) else {
            return skip("not UTF-8");
        };
        if text.trim().is_empty() {
            return skip("empty file");
        }
        if let Ok(mut note) = parse(text) {
            note.fields
                .insert("id".to_string(), Value::Str(id.to_string()));
            note.path = Some(path.to_path_buf());
            return Item::Note(note);
        }
        let name = file_name(path).to_string_lossy().into_owned();
        let created = self
            .name_time(&name)
            .unwrap_or_else(|| {
                let mtime = fs::metadata(path).and_then(|m| m.modified()).ok();
                let ns = mtime
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map_or(0, |d| d.as_nanos());
                self.local_time(ns)
            })
            .to_rfc3339_opts(SecondsFormat::Secs, false);
        let fields = [
            ("easy-note", Value::Int(VERSION)),
            ("id", Value::Str(id.to_string())),
            ("created", Value::Str(created)),
            ("tags", Value::List(tags::extract(text))),
            ("source", Value::Str("inbox".to_string())),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
        let mut note = Note::new(fields, text.to_string());
        note.path = Some(path.to_path_buf());
        Item::Note(note)
    }

    /// The local time in a `YYYYMMDD-HHMMSS` file name prefix; `None` when there is none or it
    /// is not a real date.
    fn name_time(&self, name: &str) -> Option<DateTime<FixedOffset>> {
        let prefix = name.get(..15)?;
        if !prefix.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
            return None;
        }
        let naive = NaiveDateTime::parse_from_str(prefix, "%Y%m%d-%H%M%S").ok()?;
        // The zone in force at that local time: guess with the UTC reading, then correct once.
        let guess = naive.and_utc().timestamp();
        let first = (self.utc_offset)(guess);
        let second = (self.utc_offset)(guess - i64::from(first));
        FixedOffset::east_opt(second)?
            .from_local_datetime(&naive)
            .single()
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

/// A note masked for output; with `hash`, first the hash of its body as stored.
fn finish(item: Item, hash: bool) -> (Item, Option<String>) {
    match item {
        Item::Note(mut note) => {
            let based_on = hash.then(|| ai::based_on(&note.body));
            mask_note(&mut note);
            (Item::Note(note), based_on)
        }
        skipped => (skipped, None),
    }
}

/// Mask every free-text part in place: body, tags, context fields, unknown front-matter text.
/// Returns the kinds masked, in that order. `id`, `created`, `updated`, `source` and the version
/// key are never masked.
fn mask_note(note: &mut Note) -> Vec<&'static str> {
    let mut kinds = vec![];
    let mut mask = |value: &str| {
        let (value, found) = guard::mask(value);
        kinds.extend(found);
        value
    };
    note.body = mask(&note.body);
    if let Some(Value::List(tags)) = note.fields.get_mut("tags") {
        for tag in tags.iter_mut() {
            *tag = mask(tag);
        }
    }
    for key in CONTEXT_KEYS {
        if let Some(Value::Str(value)) = note.fields.get_mut(key) {
            *value = mask(value);
        }
    }
    for chunk in &mut note.extra {
        // Pass 1: the raw YAML source. Pass 2: the decoded text, because an escape such as `\n`
        // or `\x41` in a double-quoted value hides a token from the raw patterns.
        let (raw, found) = guard::mask(chunk);
        kinds.extend(found);
        *chunk = raw;
        if let Some((text, found)) = mask_decoded(chunk) {
            kinds.extend(found);
            *chunk = text;
        }
    }
    kinds
}

/// The decoded strings of one unknown key (`chunk` is one YAML key), masked. `None` when they
/// hold no secret, or when the chunk is not one key (the raw pass has masked it already). When a
/// secret is found the key is written again with the masked text, so no secret text is left
/// behind: a string value keeps its masked text; any other value becomes `"[masked:<kind>]"`.
fn mask_decoded(chunk: &str) -> Option<(String, Vec<&'static str>)> {
    let docs = YamlOwned::load_from_str(chunk).ok()?;
    let Some(YamlOwned::Mapping(map)) = docs.first() else {
        return None;
    };
    let mut pairs = map.iter();
    let (key, value) = pairs.next()?;
    if pairs.next().is_some() {
        return None;
    }
    let mut kinds = vec![];
    let key_text = mask_strings(key, &mut kinds);
    let value_text = mask_strings(value, &mut kinds);
    let marker = quote(&format!("[masked:{}]", kinds.first()?));
    Some((
        format!(
            "{}: {}\n",
            key_text.map_or(marker.clone(), |t| quote(&t)),
            value_text.map_or(marker, |t| quote(&t)),
        ),
        kinds,
    ))
}

/// Mask a string node and return its text; for a list or map, mask every string inside it
/// (only to find the kinds) and return `None`.
fn mask_strings(node: &YamlOwned, kinds: &mut Vec<&'static str>) -> Option<String> {
    match node {
        YamlOwned::Value(ScalarOwned::String(s)) => {
            let (text, found) = guard::mask(s);
            kinds.extend(found);
            Some(text)
        }
        YamlOwned::Sequence(items) => {
            for item in items {
                mask_strings(item, kinds);
            }
            None
        }
        YamlOwned::Mapping(map) => {
            for (k, v) in map.iter() {
                mask_strings(k, kinds);
                mask_strings(v, kinds);
            }
            None
        }
        _ => None,
    }
}

fn with_newline(text: &str) -> String {
    if text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{text}\n")
    }
}

/// The name part of an inbox id: `Ok(None)` for any other id, `BadId` for an `inbox/...` id whose
/// name part is empty, holds `/` or NUL, or starts with `.` (so never `.`, `..` or a dot file).
fn inbox_part(note_id: &str) -> Result<Option<&str>, StoreError> {
    let Some(part) = note_id.strip_prefix(INBOX_PREFIX) else {
        return Ok(None);
    };
    if part.is_empty() || part.starts_with('.') || part.contains(['/', '\0']) {
        return Err(StoreError::BadId(note_id.to_string()));
    }
    Ok(Some(part))
}

/// `(mtime seconds, file name)` of a trash name `inbox__<digits>__<inbox file name>`; `None` for
/// any other name, so a file name taken from it never holds a `/`.
fn split_trash_inbox(trash_name: &str) -> Option<(u64, &str)> {
    let (secs, file) = trash_name.strip_prefix(TRASH_INBOX)?.split_once("__")?;
    let secs = match secs.bytes().all(|b| b.is_ascii_digit()) {
        true => secs.parse().ok()?,
        false => return None,
    };
    (is_inbox_file_name(file) && !file.contains(['/', '\0'])).then_some((secs, file))
}

/// A file name that counts as an inbox note: `.md` or `.txt`, not a dot file.
fn is_inbox_file_name(name: &str) -> bool {
    !name.starts_with('.') && (name.ends_with(".md") || name.ends_with(".txt"))
}

/// `(id, file name)` for sorted inbox file names. The id is `inbox/<stem>`; when two files share
/// a stem (`a.md`, `a.txt`) both ids keep the extension.
fn inbox_ids(names: Vec<String>) -> Vec<(String, String)> {
    let stem = |name: &str| {
        name.rsplit_once('.')
            .map_or(name, |(stem, _)| stem)
            .to_string()
    };
    let stems: Vec<String> = names.iter().map(|n| stem(n)).collect();
    names
        .into_iter()
        .zip(&stems)
        .map(|(name, stem)| {
            let shared = stems.iter().filter(|s| *s == stem).count() > 1;
            let id = format!("{INBOX_PREFIX}{}", if shared { &name } else { stem });
            (id, name)
        })
        .collect()
}

/// The file name an inbox id names among `(id, file name)` pairs: the exact id, else the one file
/// whose name or stem is `part`. Two such files are an error that names their ids.
fn pick_inbox(
    listed: &[(String, String)],
    part: &str,
    note_id: &str,
) -> Result<Option<String>, StoreError> {
    let exact = format!("{INBOX_PREFIX}{part}");
    if let Some((_, name)) = listed.iter().find(|(id, _)| *id == exact) {
        return Ok(Some(name.clone()));
    }
    let named: Vec<&(String, String)> = listed
        .iter()
        .filter(|(_, name)| name == part || name.rsplit_once('.').is_some_and(|(s, _)| s == part))
        .collect();
    match named.as_slice() {
        [] => Ok(None),
        [(_, name)] => Ok(Some(name.clone())),
        many => Err(StoreError::Invalid(format!(
            "{note_id} names more than one file; use {}",
            many.iter()
                .map(|(id, _)| id.as_str())
                .collect::<Vec<_>>()
                .join(" or ")
        ))),
    }
}

/// What to call a trash file in a message: its id (`inbox/<stem>` for an inbox file).
pub fn trash_label(file_name: &str) -> String {
    let stem = |name: &str| name.rsplit_once('.').map_or(name, |(s, _)| s).to_string();
    match split_trash_inbox(file_name) {
        Some((_, name)) => format!("{INBOX_PREFIX}{}", stem(name)),
        None => stem(file_name),
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
) -> Result<Updated, StoreError> {
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
