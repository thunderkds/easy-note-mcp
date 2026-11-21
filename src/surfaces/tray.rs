//! `tray`: an optional icon in the Linux system tray (FR-033, DDR 0005).
//!
//! The menu is New note, Open dashboard (runs `easy-note dashboard`), the 5 newest notes, Open notes folder and Quit. New note runs this binary
//! as `quick`. A note opens the window on it (`dashboard --note <id>`) in a `window` build with a
//! display, else its file with `xdg-open`. The folder opens with `xdg-open`. Every command is an
//! argv list, never a shell. The recent list is read again each time the menu opens.
//!
//! The first part of this file is always compiled and holds no D-Bus: the menu model, the argv of
//! each action, the single-instance lock and the texts. The StatusNotifierItem part (`ksni`) is
//! behind the cargo feature `tray`, so a default build links no tray crate (NFR-010). Without the
//! feature `run` only says how to install a build that has one.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};

use super::cli::safe;
use super::dashboard;
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
    pub id: String,
    pub title: String,
    pub created: DateTime<Utc>,
    pub path: PathBuf,
}

/// What an item does when it is clicked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    NewNote,
    /// A recent note: its id and its file.
    Open(String, PathBuf),
    OpenFolder,
    OpenDashboard,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    Item(String, Action),
    /// An item that cannot be clicked.
    Disabled(String),
    Separator,
}

/// The menu, top to bottom: New note, Open dashboard, the newest notes (at most `RECENT`) or `no notes yet`,
/// Open notes folder, Quit. `recent` is newest first. A label holds only masked text, with
/// control characters shown as `\xNN`.
pub fn menu(recent: &[Recent], now: DateTime<Utc>) -> Vec<Entry> {
    let mut entries = vec![
        Entry::Item("New note".to_string(), Action::NewNote),
        Entry::Item("Open dashboard".to_string(), Action::OpenDashboard),
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
        entries.push(Entry::Item(
            label,
            Action::Open(note.id.clone(), note.path.clone()),
        ));
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
                id: item.id,
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
        Action::Open(id, path) => Some(open_note(id, path, exe, dashboard::window_here())),
        Action::OpenFolder => Some(open(root)),
        Action::OpenDashboard => Some((exe.to_path_buf(), vec![OsString::from("dashboard")])),
        Action::Quit => None,
    }
}

/// A recent note's click: `<exe> dashboard --note <id>` when the window can open (`window`), else
/// `xdg-open <file>`.
pub fn open_note(id: &str, path: &Path, exe: &Path, window: bool) -> (PathBuf, Vec<OsString>) {
    if window {
        let args = ["dashboard", "--note", id].map(OsString::from).to_vec();
        (exe.to_path_buf(), args)
    } else {
        (PathBuf::from("xdg-open"), vec![path.as_os_str().to_owned()])
    }
}

/// The theme icon used when our own icons are not installed.
const FALLBACK_ICON: &str = "accessories-text-editor";

/// The tray icon name: ours when `install launcher` wrote it, the theme icon otherwise (FR-039).
pub fn icon_choice(installed: bool, saved: bool) -> &'static str {
    match (installed, saved) {
        (true, true) => "easy-note-saved-symbolic",
        (true, false) => "easy-note-symbolic",
        (false, _) => FALLBACK_ICON,
    }
}

/// How long the "just saved" badge shows.
const BADGE_SECS: i64 = 3;

/// Does the badge show? Only from the change until `BADGE_SECS` later. A clock set back gives no
/// badge, so it never shows longer than 3 s.
pub fn badge_state(last_change: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    last_change.is_some_and(|at| {
        let secs = now.signed_duration_since(at).num_milliseconds();
        (0..BADGE_SECS * 1000).contains(&secs)
    })
}

/// The newest modified time of this month's folder and the inbox: one or two `stat` calls and no
/// note listing (NFR-004). A missing folder counts as no time.
pub fn watch_stamp(root: &Path, now: DateTime<Utc>) -> Option<SystemTime> {
    let month = root.join(now.format("%Y/%m").to_string());
    [month, root.join("inbox")]
        .iter()
        .filter_map(|dir| fs::metadata(dir).and_then(|m| m.modified()).ok())
        .max()
}

/// The tray's saved-badge state: when the watched stamp last changed.
pub struct Badge {
    stamp: Option<SystemTime>,
    changed: Option<DateTime<Utc>>,
}

impl Badge {
    /// `stamp` is the baseline taken at start, so a note saved before the tray ran shows nothing.
    pub fn new(stamp: Option<SystemTime>) -> Badge {
        Badge {
            stamp,
            changed: None,
        }
    }

    /// Feed the stamp of this poll; `true` when the badge shows. A folder that vanishes is no
    /// change.
    pub fn poll(&mut self, stamp: Option<SystemTime>, now: DateTime<Utc>) -> bool {
        if stamp.is_some() && stamp != self.stamp {
            self.changed = Some(now);
        }
        if stamp.is_some() {
            self.stamp = stamp;
        }
        badge_state(self.changed, now)
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

    /// Where `install launcher` puts the symbolic tray icon, below `$HOME`.
    const ICON_ROOT: &str = ".local/share/icons";
    const SYMBOLIC: &str = "hicolor/symbolic/apps/easy-note-symbolic.svg";
    const WATCHER: &str = "org.kde.StatusNotifierWatcher";
    /// How often the main thread checks that the host did not go away.
    const POLL: Duration = Duration::from_secs(1);

    struct SniTray {
        store: Store,
        recent: Vec<Recent>,
        clicks: mpsc::Sender<Action>,
        /// The icon theme folder, when our symbolic icon is installed there.
        icon_root: Option<PathBuf>,
        saved: bool,
    }

    impl ksni::Tray for SniTray {
        fn id(&self) -> String {
            "easy-note".to_string()
        }

        fn title(&self) -> String {
            "easy-note".to_string()
        }

        fn icon_name(&self) -> String {
            icon_choice(self.icon_root.is_some(), self.saved).to_string()
        }

        fn icon_theme_path(&self) -> String {
            self.icon_root
                .as_ref()
                .map_or(String::new(), |dir| dir.display().to_string())
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
        let icon_root = std::env::var_os("HOME")
            .filter(|home| !home.is_empty())
            .map(|home| PathBuf::from(home).join(ICON_ROOT))
            .filter(|root| root.join(SYMBOLIC).is_file());
        let mut shown = false;
        let mut badge = Badge::new(watch_stamp(&store.root, Utc::now()));
        let tray = SniTray {
            icon_root,
            saved: false,
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
                    let now = Utc::now();
                    let saved = badge.poll(watch_stamp(&store.root, now), now);
                    if saved != shown {
                        shown = saved;
                        handle.update(|tray| tray.saved = saved);
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
            id: format!("id-{title}"),
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
                "Open dashboard",
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
            entries[1],
            Entry::Item("Open dashboard".into(), Action::OpenDashboard)
        );
        assert_eq!(
            entries[3],
            Entry::Item(
                "n0 · just now".into(),
                Action::Open("id-n0".into(), notes[0].path.clone())
            )
        );
        assert_eq!(
            entries[9],
            Entry::Item("Open notes folder".into(), Action::OpenFolder)
        );
        assert_eq!(entries[10], Entry::Item("Quit".into(), Action::Quit));
    }

    #[test]
    fn an_empty_store_shows_one_disabled_item() {
        let entries = menu(&[], at(12, 0, 0));
        assert_eq!(
            labels(&entries),
            [
                "New note",
                "Open dashboard",
                "---",
                "no notes yet",
                "---",
                "Open notes folder",
                "Quit"
            ]
        );
        assert_eq!(entries[3], Entry::Disabled("no notes yet".into()));
    }

    #[test]
    fn a_long_title_is_cut_to_forty_characters_and_a_secret_stays_masked() {
        let long = "x".repeat(60);
        let label = labels(&menu(&[note(&long, at(11, 0, 0))], at(12, 0, 0)))[3].clone();
        let title = label.split(" · ").next().unwrap();
        assert_eq!(title.chars().count(), LABEL_MAX);
        assert!(title.ends_with('…'));
        let exact = "y".repeat(LABEL_MAX);
        assert!(labels(&menu(&[note(&exact, at(11, 0, 0))], at(12, 0, 0)))[3].starts_with(&exact));
        let aws = concat!("AK", "IAIOSFODNN7EXAMPLE");
        let label = labels(&menu(
            &[note(&format!("key {aws}"), at(11, 0, 0))],
            at(12, 0, 0),
        ))[3]
            .clone();
        assert!(!label.contains(aws), "{label}");
        let label = labels(&menu(&[note("a\u{1b}[2Jb", at(11, 0, 0))], at(12, 0, 0)))[3].clone();
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
        let id = "inbox/a b; rm -rf";
        assert_eq!(
            argv(&Action::Open(id.into(), file.clone()), exe, root),
            Some(open_note(id, &file, exe, dashboard::window_here()))
        );
        assert_eq!(
            argv(&Action::OpenFolder, exe, root),
            Some((PathBuf::from("xdg-open"), vec![root.as_os_str().to_owned()]))
        );
        assert_eq!(argv(&Action::Quit, exe, root), None);
        assert_eq!(
            argv(&Action::OpenDashboard, exe, root),
            Some((exe.to_path_buf(), vec![OsString::from("dashboard")]))
        );
    }

    /// HUN-286 AC 8: a window build with a display opens the window on the note, else the file.
    #[test]
    fn a_recent_note_opens_the_window_when_it_can_else_the_file() {
        let exe = Path::new("/opt/bin/easy-note");
        let file = PathBuf::from("/home/me/easy-notes/2026/11/a b; rm -rf.md");
        let id = "20261119-090000-aaaa";
        assert_eq!(
            open_note(id, &file, exe, true),
            (
                exe.to_path_buf(),
                vec![
                    OsString::from("dashboard"),
                    OsString::from("--note"),
                    OsString::from(id)
                ]
            )
        );
        assert_eq!(
            open_note(id, &file, exe, false),
            (PathBuf::from("xdg-open"), vec![file.into_os_string()])
        );
    }

    /// HUN-276 AC 6.
    #[test]
    fn the_tray_icon_is_ours_only_when_it_is_installed() {
        assert_eq!(icon_choice(true, false), "easy-note-symbolic");
        assert_eq!(icon_choice(true, true), "easy-note-saved-symbolic");
        assert_eq!(icon_choice(false, false), "accessories-text-editor");
        assert_eq!(icon_choice(false, true), "accessories-text-editor");
    }

    /// HUN-276 AC 7: the badge shows for 3 seconds from the change.
    #[test]
    fn the_badge_shows_for_three_seconds() {
        let t0 = at(12, 0, 0);
        let after = |ms: i64| t0 + chrono::Duration::milliseconds(ms);
        assert!(badge_state(Some(t0), t0));
        assert!(badge_state(Some(t0), after(2900)));
        assert!(!badge_state(Some(t0), after(3100)));
        assert!(!badge_state(None, t0));
        // the clock went back: never longer than 3 s, so no badge
        assert!(!badge_state(Some(after(5000)), t0));
    }

    #[test]
    fn a_changed_stamp_starts_the_badge_and_the_first_poll_does_not() {
        let t0 = at(12, 0, 0);
        let s = |n: u64| Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(n));
        let mut badge = Badge::new(s(10));
        assert!(!badge.poll(s(10), t0), "same stamp: no badge");
        assert!(badge.poll(s(11), t0), "a new stamp starts it");
        assert!(badge.poll(s(11), t0 + chrono::Duration::seconds(2)));
        assert!(!badge.poll(s(11), t0 + chrono::Duration::seconds(4)));
        // a folder that vanishes is no change
        assert!(!badge.poll(None, t0 + chrono::Duration::seconds(5)));
        // a month folder that appears is a change
        assert!(badge.poll(s(20), t0 + chrono::Duration::seconds(6)));
    }

    #[test]
    fn the_stamp_is_the_newest_of_this_month_and_the_inbox_and_missing_is_none() {
        let dir = tempfile::tempdir().unwrap();
        let now = at(12, 0, 0);
        assert_eq!(watch_stamp(dir.path(), now), None);
        fs::create_dir_all(dir.path().join("2026/11")).unwrap();
        let month = watch_stamp(dir.path(), now);
        assert!(month.is_some());
        // another month is not watched
        let other = tempfile::tempdir().unwrap();
        fs::create_dir_all(other.path().join("2026/10")).unwrap();
        assert_eq!(watch_stamp(other.path(), now), None);
        fs::create_dir_all(dir.path().join("inbox")).unwrap();
        assert!(watch_stamp(dir.path(), now) >= month);
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
