//! HUN-269 store (port of Python HUN-253 tests/test_store.py): dir resolution, IDs and clashes,
//! never-overwrite, atomic writes, trash, skipped files, update.
//!
//! Every store test gets its own temp dir as the notes dir and a frozen clock, hostname and zone
//! (UTC+7) through the `Store` seams. Only the AC 8 tests use `HOME` and `EASY_NOTE_DIR`; they
//! take `ENV_LOCK` and run on a temp `HOME`. No test opens the real `~/easy-notes` or `~/.config`.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use easy_note_mcp::core::format as fmt;
use easy_note_mcp::core::store::{self, Item, Skipped, Store, StoreError};
use easy_note_mcp::core::{config, store::CONTEXT_KEYS};
use sha2::{Digest, Sha256};

const FROZEN_NS: u128 = 1_794_649_633_123_456_789; // 2026-11-14T16:47:13.123456789+07:00
const HOST: &str = "test-host";

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// A temp dir that is removed when the test ends.
struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let dir =
            std::env::temp_dir().join(format!("easy-note-store-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Sandbox(dir)
    }
    fn notes(&self) -> PathBuf {
        self.0.join("notes")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn no_link(_: &Path, _: &Path) -> io::Result<()> {
    Err(io::Error::from(io::ErrorKind::PermissionDenied))
}

fn broken_rename(_: &Path, _: &Path) -> io::Result<()> {
    Err(io::Error::from_raw_os_error(5)) // Input/output error
}

/// A store on `root` with a frozen clock (FROZEN_NS), fixed hostname and zone UTC+7.
fn store_at(root: &Path) -> Store {
    Store {
        now_ns: || FROZEN_NS,
        hostname: || HOST.to_string(),
        utc_offset: |_| 7 * 3600,
        ..Store::system(root.to_path_buf())
    }
}

fn t(tags: &[&str]) -> Vec<String> {
    tags.iter().map(|s| s.to_string()).collect()
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// Every `.md` file outside `.easy-note`, sorted.
fn md_files(root: &Path) -> Vec<PathBuf> {
    let mut all = vec![];
    walk(root, &mut all);
    all.retain(|p| {
        p.extension().is_some_and(|e| e == "md")
            && !p.strip_prefix(root).unwrap().starts_with(".easy-note")
    });
    all.sort();
    all
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut all = vec![];
    walk(root, &mut all);
    all.into_iter()
        .map(|p| {
            let bytes = std::fs::read(&p).unwrap();
            (p.strip_prefix(root).unwrap().to_path_buf(), bytes)
        })
        .collect()
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn copy_fixtures(to: &Path) {
    copy_dir(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/notes"),
        to,
    );
}

fn digest4(text: &str) -> String {
    let hex = Sha256::digest(format!("{FROZEN_NS}\n{HOST}\n{text}").as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    hex[..4].to_string()
}

fn is_id(id: &str) -> bool {
    let b = id.as_bytes();
    b.len() == 20
        && b[..8].iter().all(u8::is_ascii_digit)
        && b[8] == b'-'
        && b[9..15].iter().all(u8::is_ascii_digit)
        && b[15] == b'-'
        && b[16..]
            .iter()
            .all(|c| matches!(c, b'0'..=b'9' | b'a'..=b'f'))
}

// AC1 — a note written by the store round-trips byte for byte

#[test]
fn added_note_round_trips_byte_identical() {
    let sb = Sandbox::new("round-trip");
    let s = store_at(&sb.notes());
    let fields = [
        ("repo", Some("easy-ui-mcp")),
        ("branch", Some("no")),
        ("task", Some("T120")),
        ("client", None),
    ];
    let added = s
        .add(
            "retry bug only on Firefox\n\nsecond line",
            "mcp",
            &t(&["flaky"]),
            &fields,
        )
        .unwrap();
    let raw = std::fs::read(&added.path).unwrap();
    let text = String::from_utf8(raw.clone()).unwrap();
    assert_eq!(fmt::render(&fmt::parse(&text).unwrap()).into_bytes(), raw);
    let note = s.read(&added.id).unwrap();
    assert_eq!(note.get_str("branch"), Some("no"));
    assert_eq!(note.get_str("client"), None);
    assert_eq!(note.body, "retry bug only on Firefox\n\nsecond line\n");
}

// AC2 — ID format and clash suffix

#[test]
fn id_is_local_time_plus_sha256_of_ns_hostname_text() {
    let sb = Sandbox::new("id");
    let s = store_at(&sb.notes());
    let added = s.add("x", "cli", &[], &[]).unwrap();
    assert_eq!(added.id, format!("20261114-164713-{}", digest4("x")));
    assert!(is_id(&added.id), "{}", added.id);
    assert_eq!(
        added.path,
        sb.notes().join("2026/11").join(format!("{}.md", added.id))
    );
    assert_eq!(
        s.read(&added.id).unwrap().get_str("created"),
        Some("2026-11-14T16:47:13+07:00")
    );
}

#[test]
fn two_writes_same_second_same_text_get_clash_suffix() {
    let sb = Sandbox::new("clash");
    let s = store_at(&sb.notes());
    let first = s.add("x", "cli", &[], &[]).unwrap();
    let second = s.add("x", "cli", &[], &[]).unwrap();
    assert_eq!(second.id, format!("{}-2", first.id));
    assert_eq!(
        s.add("x", "cli", &[], &[]).unwrap().id,
        format!("{}-3", first.id)
    );
    let stems: Vec<String> = md_files(&sb.notes())
        .iter()
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        stems,
        [
            first.id.clone(),
            format!("{}-2", first.id),
            format!("{}-3", first.id)
        ]
    );
    assert_eq!(s.read(&second.id).unwrap().id(), second.id); // front matter id equals the file stem
}

#[test]
fn month_folder_uses_local_date_not_utc() {
    // 2026-11-30T20:00:00Z is already 2026-12-01 03:00 in UTC+7
    let sb = Sandbox::new("month");
    let s = Store {
        now_ns: || 1_796_068_800 * 1_000_000_000,
        ..store_at(&sb.notes())
    };
    let added = s.add("x", "cli", &[], &[]).unwrap();
    assert_eq!(added.path.parent().unwrap(), sb.notes().join("2026/12"));
    assert!(added.id.starts_with("20261201-030000-"), "{}", added.id);
}

// AC3 — a write never replaces an existing file

fn write_never_replaces_existing_file(hard_links: bool, name: &str) {
    let sb = Sandbox::new(name);
    let mut s = store_at(&sb.notes());
    if !hard_links {
        s.link = no_link;
    }
    let target = sb
        .notes()
        .join("2026/11")
        .join(format!("20261114-164713-{}.md", digest4("x")));
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, b"synced from the other machine\n").unwrap();

    let added = s.add("x", "cli", &[], &[]).unwrap();

    assert_ne!(added.path, target);
    assert!(added.id.ends_with("-2"));
    assert_eq!(
        std::fs::read(&target).unwrap(),
        b"synced from the other machine\n"
    );
    assert!(std::fs::read(&added.path)
        .unwrap()
        .starts_with(b"---\neasy-note: 1\n"));
    let hidden = std::fs::read_dir(target.parent().unwrap())
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with('.')
        })
        .count();
    assert_eq!(hidden, 0, "temp file removed");
}

#[test]
fn write_never_replaces_existing_file_with_hard_links() {
    write_never_replaces_existing_file(true, "no-replace-link");
}

#[test]
fn write_never_replaces_existing_file_without_hard_links() {
    write_never_replaces_existing_file(false, "no-replace-nolink");
}

// AC4 — temp file in the same dir + rename; a killed write leaves no partial .md

/// Child half of `killed_write_leaves_no_md_file`: runs only when the parent sets the env var.
#[test]
fn killed_write_child() {
    let Some(root) = std::env::var_os("EASY_NOTE_KILL_ROOT") else {
        return;
    };
    let s = Store {
        link: |_, _| -> io::Result<()> { std::process::exit(9) },
        ..store_at(Path::new(&root))
    };
    let _ = s.add("half written", "cli", &[], &[]);
}

#[test]
fn killed_write_leaves_no_md_file() {
    // A real kill: the child process exits inside the link step, so no cleanup code runs.
    let sb = Sandbox::new("kill");
    let done = Command::new(std::env::current_exe().unwrap())
        .args([
            "killed_write_child",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("EASY_NOTE_KILL_ROOT", sb.notes())
        .output()
        .unwrap();
    assert_eq!(done.status.code(), Some(9), "{done:?}");
    assert_eq!(md_files(&sb.notes()), Vec::<PathBuf>::new());
}

#[test]
fn failing_rename_on_no_link_filesystem_leaves_no_md() {
    let sb = Sandbox::new("rename-add");
    let s = Store {
        link: no_link,
        rename: broken_rename,
        ..store_at(&sb.notes())
    };
    let err = s.add("x", "cli", &[], &[]).unwrap_err();
    assert!(err.to_string().contains("Input/output error"), "{err}");
    assert_eq!(md_files(&sb.notes()), Vec::<PathBuf>::new());
    let left: Vec<_> = std::fs::read_dir(sb.notes().join("2026/11"))
        .unwrap()
        .collect();
    assert!(left.is_empty(), "temp file removed too: {left:?}");
}

#[test]
fn failing_rename_during_update_keeps_old_bytes() {
    let sb = Sandbox::new("rename-update");
    let s = store_at(&sb.notes());
    let added = s.add("x", "cli", &[], &[]).unwrap();
    let before = snapshot(&sb.notes());
    let broken = Store {
        rename: broken_rename,
        ..store_at(&sb.notes())
    };
    assert!(broken.update(&added.id, Some("new text"), None).is_err());
    assert_eq!(snapshot(&sb.notes()), before);
}

// AC5 — rm → trash, restore → back with identical bytes, empty_trash by age

fn set_mtime(path: &Path, when: SystemTime) {
    let file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    file.set_modified(when).unwrap();
}

fn days_ago(days: u64) -> SystemTime {
    SystemTime::now() - Duration::from_secs(days * 86_400)
}

#[test]
fn rm_moves_to_trash_and_restore_brings_back_same_bytes() {
    let sb = Sandbox::new("trash");
    let s = store_at(&sb.notes());
    let added = s.add("keep me", "cli", &[], &[]).unwrap();
    let original = std::fs::read(&added.path).unwrap();
    let trashed = s.rm(&added.id).unwrap();
    assert_eq!(
        trashed,
        sb.notes()
            .join(".easy-note/trash")
            .join(format!("{}.md", added.id))
    );
    assert!(!added.path.exists());
    assert_eq!(std::fs::read(&trashed).unwrap(), original);
    assert!(matches!(s.read(&added.id), Err(StoreError::NotFound(_))));
    assert_eq!(s.restore(&added.id).unwrap(), added.path);
    assert_eq!(std::fs::read(&added.path).unwrap(), original);
    assert!(!trashed.exists());
}

#[test]
fn rm_and_restore_unknown_id_return_not_found() {
    let sb = Sandbox::new("trash-unknown");
    let s = store_at(&sb.notes());
    assert!(matches!(
        s.rm("20261114-164713-0000"),
        Err(StoreError::NotFound(_))
    ));
    assert!(matches!(
        s.restore("20261114-164713-0000"),
        Err(StoreError::NotFound(_))
    ));
}

#[test]
fn restore_never_replaces_a_note_at_the_target() {
    let sb = Sandbox::new("restore-clash");
    let s = store_at(&sb.notes());
    let added = s.add("x", "cli", &[], &[]).unwrap();
    let trashed = s.rm(&added.id).unwrap();
    std::fs::write(&added.path, b"newer copy from sync\n").unwrap();
    assert!(matches!(s.restore(&added.id), Err(StoreError::Exists(_))));
    assert_eq!(
        std::fs::read(&added.path).unwrap(),
        b"newer copy from sync\n"
    );
    assert!(trashed.exists());
}

#[test]
fn empty_trash_deletes_only_files_older_than_cutoff() {
    let sb = Sandbox::new("empty-trash");
    let s = store_at(&sb.notes());
    let old = s.rm(&s.add("old", "cli", &[], &[]).unwrap().id).unwrap();
    let new = s.rm(&s.add("new", "cli", &[], &[]).unwrap().id).unwrap();
    set_mtime(&old, days_ago(40));
    assert_eq!(
        s.empty_trash(Duration::from_secs(30 * 86_400)).unwrap(),
        vec![old.clone()]
    );
    assert!(!old.exists() && new.exists());
}

#[test]
fn rm_stamps_trash_time_so_an_old_note_is_not_emptied_at_once() {
    let sb = Sandbox::new("trash-stamp");
    let s = store_at(&sb.notes());
    let added = s.add("written long ago", "cli", &[], &[]).unwrap();
    set_mtime(&added.path, days_ago(400));
    let trashed = s.rm(&added.id).unwrap();
    assert_eq!(
        s.empty_trash(Duration::from_secs(30 * 86_400)).unwrap(),
        Vec::<PathBuf>::new()
    );
    assert!(trashed.exists());
}

// AC6 — bad front matter and `easy-note: 2` are skipped with their path, bytes unchanged

#[test]
fn bad_files_are_skipped_with_path_and_left_unchanged() {
    let sb = Sandbox::new("skipped");
    let root = sb.notes();
    copy_fixtures(&root);
    let month = root.join("2026/11");
    std::fs::write(
        month.join("20261114-170500-0bad.md"),
        b"---\xff\xfe not utf-8\n",
    )
    .unwrap();
    std::fs::copy(
        month.join("20261114-164713-a1b2.md"),
        month.join("20261114-170600-0f0f.md"),
    )
    .unwrap(); // id differs from file name
    std::fs::write(root.join("README.md"), "not a note, outside YYYY/MM\n").unwrap();
    let before = snapshot(&root);

    let found: Vec<Item> = store_at(&root).iter_notes().collect();

    let skipped: BTreeMap<String, String> = found
        .iter()
        .filter_map(|item| match item {
            Item::Skipped(s) => Some((
                s.path.file_name().unwrap().to_string_lossy().into_owned(),
                s.reason.clone(),
            )),
            Item::Note(_) => None,
        })
        .collect();
    let expected: BTreeMap<String, String> = [
        // The line is where the YAML reader reports the unclosed quote (the quote opens on line 3).
        (
            "20261114-170000-bad1.md",
            "front matter is not valid YAML (line 3)",
        ),
        ("20261114-170100-0002.md", "unsupported version 2"),
        ("20261114-170200-0003.md", "no front matter"),
        ("20261114-170500-0bad.md", "not UTF-8"),
        ("20261114-170600-0f0f.md", "id does not match file name"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    assert_eq!(skipped, expected);
    for item in &found {
        match item {
            Item::Skipped(s) => assert!(s.path.is_absolute()),
            Item::Note(n) => assert!(n.path.as_ref().unwrap().is_absolute()),
        }
    }
    let ids: Vec<&str> = found
        .iter()
        .filter_map(|i| match i {
            Item::Note(n) => Some(n.id()),
            Item::Skipped(_) => None,
        })
        .collect();
    assert_eq!(
        ids,
        [
            "20261114-164713-a1b2",
            "20261114-170300-c71f",
            "20261114-170400-0b5d"
        ]
    );
    assert_eq!(snapshot(&root), before);
}

// AC7 — update keeps unknown keys and sets `updated`

#[test]
fn update_keeps_unknown_keys_and_sets_updated() {
    let sb = Sandbox::new("update-keys");
    copy_fixtures(&sb.notes());
    let note = store_at(&sb.notes())
        .update(
            "20261114-170400-0b5d",
            None,
            Some(&t(&["flaky", "obsidian"])),
        )
        .unwrap();
    let out = std::fs::read_to_string(note.path.unwrap()).unwrap();
    assert!(out.contains("aliases:\n  - firefox retry\n"));
    assert!(out.contains("date: 2024-01-01 # Obsidian date property\n"));
    assert!(out.contains("cssclasses: [wide]\n"));
    assert!(out.contains("updated: \"2026-11-14T16:47:13+07:00\"\n"));
    assert!(out.contains("tags: [\"flaky\", \"obsidian\"]\n"));
    assert!(out.ends_with("---\nHand-edited in Obsidian\n"));
}

#[test]
fn update_text_replaces_body_and_checks_size() {
    let sb = Sandbox::new("update-text");
    let s = store_at(&sb.notes());
    let added = s.add("first", "cli", &[], &[]).unwrap();
    assert_eq!(
        s.update(&added.id, Some("second"), None).unwrap().body,
        "second\n"
    );
    assert_eq!(s.read(&added.id).unwrap().body, "second\n");
    let err = s
        .update(&added.id, Some(&"x".repeat(20_001)), None)
        .unwrap_err();
    assert!(matches!(err, StoreError::TextTooLong(_)), "{err}");
}

#[test]
fn read_refuses_a_value_that_is_not_an_id() {
    let sb = Sandbox::new("bad-id");
    let s = store_at(&sb.notes());
    for bad in [
        "../../etc/passwd",
        "20261114-164713-a1b2/../x",
        "20261114-164713-A1B2",
        "",
    ] {
        let err = s.read(bad).unwrap_err();
        assert!(err.to_string().contains("not a note id"), "{bad}: {err}");
    }
}

// AC8 — zero setup: ~/easy-notes is created by the first add; EASY_NOTE_DIR beats the config.
// These tests change HOME and EASY_NOTE_DIR, so they hold ENV_LOCK and restore both at the end.

struct EnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

impl EnvGuard {
    fn new(home: &Path, notes_dir: Option<&Path>) -> EnvGuard {
        let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let saved = ["HOME", "EASY_NOTE_DIR"]
            .into_iter()
            .map(|k| (k, std::env::var_os(k)))
            .collect();
        std::env::set_var("HOME", home);
        match notes_dir {
            Some(dir) => std::env::set_var("EASY_NOTE_DIR", dir),
            None => std::env::remove_var("EASY_NOTE_DIR"),
        }
        EnvGuard { _lock: lock, saved }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.saved {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }
}

#[test]
fn first_add_creates_default_dir_once() {
    let sb = Sandbox::new("default-dir");
    let home = sb.0.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let _env = EnvGuard::new(&home, None);
    let first = store::add("x", "cli", &[], &[]).unwrap();
    assert!(first.created_dir);
    assert!(
        first.path.starts_with(home.join("easy-notes")),
        "{:?}",
        first.path
    );
    assert!(!store::add("y", "cli", &[], &[]).unwrap().created_dir);
}

#[test]
fn env_dir_beats_config_and_config_beats_default() {
    let sb = Sandbox::new("env-config");
    let home = sb.0.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let from_env = sb.notes();
    let from_config = sb.0.join("from config");
    {
        let _env = EnvGuard::new(&home, Some(&from_env));
        config::save_dir(&from_config).unwrap();
        assert!(store::add("x", "cli", &[], &[])
            .unwrap()
            .path
            .starts_with(&from_env));
    }
    let _env = EnvGuard::new(&home, None);
    assert!(store::add("x", "cli", &[], &[])
        .unwrap()
        .path
        .starts_with(&from_config));
}

#[test]
fn config_round_trips_awkward_paths() {
    let sb = Sandbox::new("config-odd");
    let home = sb.0.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let _env = EnvGuard::new(&home, None);
    let odd = sb.0.join("quo\"te \\ back é 😀 tab\t del\u{7f}");
    config::save_dir(&odd).unwrap();
    assert_eq!(config::notes_dir().unwrap(), odd);
    assert_eq!(
        config::config_path().unwrap(),
        home.join(".config/easy-note/config.toml")
    );
}

#[test]
fn broken_config_returns_clear_error() {
    let sb = Sandbox::new("config-broken");
    let home = sb.0.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let _env = EnvGuard::new(&home, None);
    let path = config::config_path().unwrap();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "dir = [unclosed\n").unwrap();
    let err = config::notes_dir().unwrap_err();
    assert!(err.to_string().contains("config.toml"), "{err}");
}

// AC9 (store side) — the full text is kept; only over-size text is refused, before anything is written

#[test]
fn long_first_line_is_kept_whole_in_the_file() {
    let sb = Sandbox::new("long-title");
    let s = store_at(&sb.notes());
    let added = s.add(&"t".repeat(300), "cli", &[], &[]).unwrap();
    let note = s.read(&added.id).unwrap();
    assert_eq!(note.body, format!("{}\n", "t".repeat(300)));
    assert_eq!(note.title(), "t".repeat(120));
}

#[test]
fn oversize_text_is_refused_before_any_write() {
    let sb = Sandbox::new("oversize");
    let s = store_at(&sb.notes());
    let err = s.add(&"x".repeat(20_001), "cli", &[], &[]).unwrap_err();
    assert!(err.to_string().contains("limit is 20,000"), "{err}");
    assert!(!sb.notes().exists());
}

#[test]
fn add_rejects_bad_arguments() {
    let sb = Sandbox::new("bad-args");
    let s = store_at(&sb.notes());
    let err = s.add("x", "fax", &[], &[]).unwrap_err();
    assert!(err.to_string().contains("unknown source"), "{err}");
    let err = s.add("x", "inbox", &[], &[]).unwrap_err();
    assert!(err.to_string().contains("unknown source"), "{err}");
    let err = s.add("x", "cli", &[], &[("id", Some("x"))]).unwrap_err();
    assert!(err.to_string().contains("not a context field"), "{err}");
    assert!(!sb.notes().exists());
    // Rust types make a non-string tag, a bare-string `tags` and a non-string field a compile error.
    assert_eq!(
        CONTEXT_KEYS,
        ["repo", "remote", "branch", "task", "client", "session", "commit"]
    );
}

#[test]
fn tags_are_stored_in_order() {
    let sb = Sandbox::new("tags");
    let s = store_at(&sb.notes());
    let added = s.add("x", "cli", &t(&["a", "b"]), &[]).unwrap();
    assert_eq!(
        s.read(&added.id).unwrap().fields.get("tags"),
        Some(&fmt::Value::List(t(&["a", "b"])))
    );
}

#[test]
fn unreadable_file_is_skipped_not_fatal() {
    use std::os::unix::fs::PermissionsExt;
    let sb = Sandbox::new("unreadable");
    let s = store_at(&sb.notes());
    let added = s.add("x", "cli", &[], &[]).unwrap();
    std::fs::set_permissions(&added.path, std::fs::Permissions::from_mode(0)).unwrap();
    let found: Vec<Item> = s.iter_notes().collect();
    std::fs::set_permissions(&added.path, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(
        found,
        vec![Item::Skipped(Skipped {
            path: added.path.clone(),
            reason: "cannot read file".to_string()
        })]
    );
}
