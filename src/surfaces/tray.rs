//! `tray`: an optional icon in the Linux system tray (FR-033, DDR 0005).
//!
//! The menu is New note, the 5 newest notes, Open notes folder and Quit. New note runs this binary
//! as `quick`; a note and the folder open with `xdg-open`. Every command is an argv list, never a
//! shell. The recent list is read again each time the menu opens.
//!
//! The first part of this file is always compiled and holds no D-Bus: the menu model, the argv of
//! each action, the single-instance lock and the texts. The StatusNotifierItem part (`ksni`) is
//! behind the cargo feature `tray`, so a default build links no tray crate (NFR-010). Without the
//! feature `run` only says how to install a build that has one.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use super::cli::safe;
use crate::core::query::{self, Filters};
use crate::core::store::Store;

/// How many notes the menu lists.
pub const RECENT: usize = 5;
/// The longest note label, in characters (an ellipsis counts).
const LABEL_MAX: usize = 40;

/// What a person can do when there is no tray host: the hotkey is the one-press path (FR-033).
pub const HOTKEY_HINT: &str =
    "GNOME needs the AppIndicator extension; the one-press path is a hotkey: run easy-note hotkey";
/// What `tray` and `install tray` say in a build without the feature.
pub const NEEDS_FEATURE: &str =
    "this build has no tray; install one with: cargo install --path . --locked --features tray";

/// One note in the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recent {
    pub title: String,
    pub created: DateTime<Utc>,
    pub path: PathBuf,
}

/// What an item does when it is clicked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    NewNote,
    Open(PathBuf),
    OpenFolder,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    Item(String, Action),
    /// An item that cannot be clicked.
    Disabled(String),
    Separator,
}

/// The menu, top to bottom: New note, the newest notes (at most `RECENT`) or `no notes yet`,
/// Open notes folder, Quit. `recent` is newest first. A label holds only masked text, with
/// control characters shown as `\xNN`.
pub fn menu(recent: &[Recent], now: DateTime<Utc>) -> Vec<Entry> {
    let mut entries = vec![
        Entry::Item("New note".to_string(), Action::NewNote),
        Entry::Separator,
    ];
    if recent.is_empty() {
        entries.push(Entry::Disabled("no notes yet".to_string()));
    }
    for note in recent.iter().take(RECENT) {
        let label = format!(
            "{} · {}",
            cut(&safe(&note.title)),
            relative(note.created, now)
        );
        entries.push(Entry::Item(label, Action::Open(note.path.clone())));
    }
    entries.push(Entry::Separator);
    entries.push(Entry::Item(
        "Open notes folder".to_string(),
        Action::OpenFolder,
    ));
    entries.push(Entry::Item("Quit".to_string(), Action::Quit));
    entries
}

/// At most `LABEL_MAX` characters; a longer text ends with `…`.
fn cut(text: &str) -> String {
    if text.chars().count() <= LABEL_MAX {
        return text.to_string();
    }
    let mut out: String = text.chars().take(LABEL_MAX - 1).collect();
    out.push('…');
    out
}

/// `just now`, `5 min ago`, `3 h ago`, `2 d ago`. A time after `now` is `just now`.
pub fn relative(created: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let secs = now.signed_duration_since(created).num_seconds();
    match secs {
        ..=59 => "just now".to_string(),
        60..=3599 => format!("{} min ago", secs / 60),
        3600..=86399 => format!("{} h ago", secs / 3600),
        _ => format!("{} d ago", secs / 86400),
    }
}

/// The newest `n` notes with the path of each file. A note that cannot be read is left out, and
/// so is a store that cannot be listed: the menu then shows `no notes yet`.
pub fn recent(store: &Store, n: usize) -> Vec<Recent> {
    let Ok(page) = query::list_notes_in(store, &Filters::default(), n) else {
        return vec![];
    };
    page.items
        .into_iter()
        .filter_map(|item| {
            let path = store.read(&item.id).ok()?.path?;
            let created = DateTime::parse_from_rfc3339(&item.created).ok()?;
            Some(Recent {
                title: item.title,
                created: created.with_timezone(&Utc),
                path,
            })
        })
        .collect()
}

/// The program and arguments of an action, or `None` for Quit. `exe` is this binary and `root`
/// the notes dir.
pub fn argv(action: &Action, exe: &Path, root: &Path) -> Option<(PathBuf, Vec<OsString>)> {
    let open = |target: &Path| {
        (
            PathBuf::from("xdg-open"),
            vec![target.as_os_str().to_owned()],
        )
    };
    match action {
        Action::NewNote => Some((exe.to_path_buf(), vec![OsString::from("quick")])),
        Action::Open(path) => Some(open(path)),
        Action::OpenFolder => Some(open(root)),
        Action::Quit => None,
    }
}

/// One tray per notes dir: an exclusive lock on `<notes dir>/.easy-note/tray.lock`, held while the
/// returned file lives. The system drops it when the process ends, however it ends.
pub fn acquire_lock(root: &Path) -> Result<File, String> {
    let dir = root.join(".easy-note");
    let path = dir.join("tray.lock");
    let file = fs::create_dir_all(&dir)
        .and_then(|()| {
            OpenOptions::new()
                .create(true)
                .truncate(false)
                .write(true)
                .open(&path)
        })
        .map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err("tray already running".to_string()),
        Err(TryLockError::Error(e)) => Err(format!("cannot lock {}: {e}", path.display())),
    }
}

/// The doctor's line about the tray host, or `None` when there is one.
pub fn doctor_line(host_present: bool) -> Option<String> {
    (!host_present).then(|| format!("tray: no tray host ({HOTKEY_HINT})"))
}

#[cfg(not(feature = "tray"))]
pub fn run() -> Result<(), String> {
    Err(NEEDS_FEATURE.to_string())
}

#[cfg(feature = "tray")]
pub use icon::{host_present, run};

#[cfg(feature = "tray")]
mod icon {
    use std::process::{Command, Stdio};
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;

    use ksni::blocking::TrayMethods;
    use ksni::menu::{MenuItem, StandardItem};

    use super::*;

    /// The theme icon until the designer's set exists.
    const ICON: &str = "accessories-text-editor";
    const WATCHER: &str = "org.kde.StatusNotifierWatcher";
    /// How often the main thread checks that the host did not go away.
    const POLL: Duration = Duration::from_secs(1);

    struct SniTray {
        store: Store,
        recent: Vec<Recent>,
        clicks: mpsc::Sender<Action>,
    }

    impl ksni::Tray for SniTray {
        fn id(&self) -> String {
            "easy-note".to_string()
        }

        fn title(&self) -> String {
            "easy-note".to_string()
        }

        fn icon_name(&self) -> String {
            ICON.to_string()
        }

        fn menu(&self) -> Vec<MenuItem<Self>> {
            menu(&self.recent, Utc::now())
                .into_iter()
                .map(|entry| match entry {
                    Entry::Separator => MenuItem::Separator,
                    Entry::Disabled(label) => StandardItem {
                        label: mnemonic_safe(&label),
                        enabled: false,
                        ..Default::default()
                    }
                    .into(),
                    Entry::Item(label, action) => StandardItem {
                        label: mnemonic_safe(&label),
                        activate: Box::new(move |tray: &mut SniTray| {
                            let _ = tray.clicks.send(action.clone());
                        }),
                        ..Default::default()
                    }
                    .into(),
                })
                .collect()
        }

        fn menu_about_to_show(&mut self) {
            self.recent = recent(&self.store, RECENT);
        }
    }

    /// A menu label reads `_` as a key marker; `__` shows one underscore.
    fn mnemonic_safe(label: &str) -> String {
        label.replace('_', "__")
    }

    /// Start the child with no stdio and wait for it on a thread, so none is left a zombie.
    fn start(action: &Action, exe: &Path, root: &Path) {
        let Some((program, args)) = argv(action, exe, root) else {
            return;
        };
        let child = Command::new(&program)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        match child {
            Ok(mut child) => {
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
            }
            Err(e) => eprintln!("easy-note: cannot start {}: {e}", program.display()),
        }
    }

    /// Show the icon until Quit. `Err` is the message for stderr (exit 1): another tray runs, or
    /// there is no tray host.
    pub fn run() -> Result<(), String> {
        let store = Store::open().map_err(|e| e.to_string())?;
        let _lock = acquire_lock(&store.root)?;
        let exe = std::env::current_exe().map_err(|e| format!("cannot find myself: {e}"))?;
        let (clicks, inbox) = mpsc::channel();
        let tray = SniTray {
            recent: recent(&store, RECENT),
            store: store.clone(),
            clicks,
        };
        let handle = tray
            .spawn()
            .map_err(|e| format!("no tray host ({e}); {HOTKEY_HINT}"))?;
        loop {
            match inbox.recv_timeout(POLL) {
                Ok(Action::Quit) | Err(RecvTimeoutError::Disconnected) => break,
                Ok(action) => start(&action, &exe, &store.root),
                Err(RecvTimeoutError::Timeout) => {
                    if handle.is_closed() {
                        return Err(format!("the tray host went away; {HOTKEY_HINT}"));
                    }
                }
            }
        }
        handle.shutdown().wait();
        Ok(())
    }

    /// Is the StatusNotifierWatcher on the session bus? No session bus counts as no.
    pub fn host_present() -> bool {
        let Ok(bus) = zbus::blocking::Connection::session() else {
            return false;
        };
        let Ok(name) = zbus::names::BusName::try_from(WATCHER) else {
            return false;
        };
        let Ok(proxy) = zbus::blocking::fdo::DBusProxy::new(&bus) else {
            return false;
        };
        proxy.name_has_owner(name).unwrap_or(false)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn an_underscore_in_a_label_shows_as_one() {
            assert_eq!(mnemonic_safe("snake_case"), "snake__case");
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn at(h: u32, m: u32, s: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 17, h, m, s).unwrap()
    }

    fn note(title: &str, created: DateTime<Utc>) -> Recent {
        Recent {
            title: title.to_string(),
            created,
            path: PathBuf::from(format!("/notes/{title}.md")),
        }
    }

    fn labels(entries: &[Entry]) -> Vec<String> {
        entries
            .iter()
            .map(|e| match e {
                Entry::Item(label, _) | Entry::Disabled(label) => label.clone(),
                Entry::Separator => "---".to_string(),
            })
            .collect()
    }

    #[test]
    fn the_menu_lists_the_five_newest_notes_between_the_fixed_items() {
        let now = at(12, 0, 0);
        let notes: Vec<Recent> = (0..7_i64)
            .map(|i| note(&format!("n{i}"), now - chrono::Duration::minutes(i)))
            .collect();
        let entries = menu(&notes, now);
        assert_eq!(
            labels(&entries),
            [
                "New note",
                "---",
                "n0 · just now",
                "n1 · 1 min ago",
                "n2 · 2 min ago",
                "n3 · 3 min ago",
                "n4 · 4 min ago",
                "---",
                "Open notes folder",
                "Quit",
            ]
        );
        assert_eq!(entries[0], Entry::Item("New note".into(), Action::NewNote));
        assert_eq!(
            entries[2],
            Entry::Item("n0 · just now".into(), Action::Open(notes[0].path.clone()))
        );
        assert_eq!(
            entries[8],
            Entry::Item("Open notes folder".into(), Action::OpenFolder)
        );
        assert_eq!(entries[9], Entry::Item("Quit".into(), Action::Quit));
    }

    #[test]
    fn an_empty_store_shows_one_disabled_item() {
        let entries = menu(&[], at(12, 0, 0));
        assert_eq!(
            labels(&entries),
            [
                "New note",
                "---",
                "no notes yet",
                "---",
                "Open notes folder",
                "Quit"
            ]
        );
        assert_eq!(entries[2], Entry::Disabled("no notes yet".into()));
    }

    #[test]
    fn a_long_title_is_cut_to_forty_characters_and_a_secret_stays_masked() {
        let long = "x".repeat(60);
        let label = labels(&menu(&[note(&long, at(11, 0, 0))], at(12, 0, 0)))[2].clone();
        let title = label.split(" · ").next().unwrap();
        assert_eq!(title.chars().count(), LABEL_MAX);
        assert!(title.ends_with('…'));
        let exact = "y".repeat(LABEL_MAX);
        assert!(labels(&menu(&[note(&exact, at(11, 0, 0))], at(12, 0, 0)))[2].starts_with(&exact));
        let aws = concat!("AK", "IAIOSFODNN7EXAMPLE");
        let label = labels(&menu(
            &[note(&format!("key {aws}"), at(11, 0, 0))],
            at(12, 0, 0),
        ))[2]
            .clone();
        assert!(!label.contains(aws), "{label}");
        let label = labels(&menu(&[note("a\u{1b}[2Jb", at(11, 0, 0))], at(12, 0, 0)))[2].clone();
        assert!(!label.contains('\u{1b}'), "{label}");
    }

    #[test]
    fn relative_time_steps() {
        let now = at(12, 0, 0);
        let ago = |secs: i64| relative(now - chrono::Duration::seconds(secs), now);
        assert_eq!(ago(0), "just now");
        assert_eq!(ago(59), "just now");
        assert_eq!(ago(60), "1 min ago");
        assert_eq!(ago(3599), "59 min ago");
        assert_eq!(ago(3600), "1 h ago");
        assert_eq!(ago(86399), "23 h ago");
        assert_eq!(ago(86400), "1 d ago");
        assert_eq!(ago(-30), "just now");
    }

    #[test]
    fn each_action_is_an_argv_list() {
        let exe = Path::new("/opt/bin/easy-note");
        let root = Path::new("/home/me/easy-notes");
        assert_eq!(
            argv(&Action::NewNote, exe, root),
            Some((exe.to_path_buf(), vec![OsString::from("quick")]))
        );
        let file = PathBuf::from("/home/me/easy-notes/2026/11/a b; rm -rf.md");
        assert_eq!(
            argv(&Action::Open(file.clone()), exe, root),
            Some((PathBuf::from("xdg-open"), vec![file.into_os_string()]))
        );
        assert_eq!(
            argv(&Action::OpenFolder, exe, root),
            Some((PathBuf::from("xdg-open"), vec![root.as_os_str().to_owned()]))
        );
        assert_eq!(argv(&Action::Quit, exe, root), None);
    }

    #[test]
    fn a_second_lock_on_the_same_notes_dir_is_refused_until_the_first_ends() {
        let dir = tempfile::tempdir().unwrap();
        let first = acquire_lock(dir.path()).unwrap();
        assert_eq!(
            acquire_lock(dir.path()).unwrap_err(),
            "tray already running"
        );
        drop(first);
        acquire_lock(dir.path()).unwrap();
    }

    #[test]
    fn the_doctor_line_names_a_missing_host_only() {
        let line = doctor_line(false).unwrap();
        assert!(line.starts_with("tray: no tray host ("), "{line}");
        assert!(line.contains("easy-note hotkey"), "{line}");
        assert_eq!(doctor_line(true), None);
    }

    fn write_note(root: &Path, id: &str, created: &str, title: &str) {
        let dir = root.join("2026/11");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{id}.md")),
            format!(
                "---\neasy-note: 1\nid: \"{id}\"\ncreated: \"{created}\"\ntags: []\nsource: \"cli\"\n---\n{title}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn recent_reads_the_newest_notes_with_their_paths_and_skips_unreadable_files() {
        let dir = tempfile::tempdir().unwrap();
        for i in 1..=7 {
            write_note(
                dir.path(),
                &format!("20261101-10000{i}-aaa{i}"),
                &format!("2026-11-01T10:00:0{i}+00:00"),
                &format!("note {i}"),
            );
        }
        fs::write(
            dir.path().join("2026/11/20261101-100009-bad0.md"),
            "no front matter",
        )
        .unwrap();
        let store = Store::system(dir.path().to_path_buf());
        let got = recent(&store, RECENT);
        let titles: Vec<&str> = got.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(titles, ["note 7", "note 6", "note 5", "note 4", "note 3"]);
        assert_eq!(
            got[0].path,
            dir.path().join("2026/11/20261101-100007-aaa7.md")
        );
        assert_eq!(
            got[0].created,
            Utc.with_ymd_and_hms(2026, 11, 1, 10, 0, 7).unwrap()
        );
        assert!(recent(&Store::system(dir.path().join("none")), RECENT).is_empty());
    }
}
