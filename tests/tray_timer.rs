//! HUN-296 (FR-049, NFR-014): the tray's AI timer. The clock, the config and the spawner are
//! inputs, so no test needs a tray host or starts a child. Temp notes dir only.

use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use easy_note_mcp::core::config::{AiSettings, Schedule};
use easy_note_mcp::surfaces::tray::{ai_run_argv, AiTimer};

fn start() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 27, 10, 0, 0).unwrap()
}

/// `minutes` and `seconds` after the tray started.
fn at(minutes: i64, seconds: i64) -> DateTime<Utc> {
    start() + TimeDelta::minutes(minutes) + TimeDelta::seconds(seconds)
}

fn on(interval_min: u32) -> AiSettings {
    AiSettings {
        enrich: true,
        schedule: Schedule::Tray,
        interval_min,
        ..AiSettings::default()
    }
}

struct Tray {
    timer: AiTimer,
    notes: tempfile::TempDir,
    runs: u32,
}

impl Tray {
    fn new() -> Tray {
        Tray {
            timer: AiTimer::new(start()),
            notes: tempfile::tempdir().unwrap(),
            runs: 0,
        }
    }

    /// One poll of the tray loop; `true` when it started a run.
    fn tick(&mut self, now: DateTime<Utc>, settings: AiSettings) -> bool {
        let before = self.runs;
        let said = self
            .timer
            .tick(now, self.notes.path(), || Ok(settings), || self.runs += 1);
        assert_eq!(said, None);
        self.runs > before
    }

    fn state(&self, json: &str) {
        let folder = self.notes.path().join(".easy-note");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("ai-run-state.json"), json).unwrap();
    }
}

/// AC 1, AC 3: nothing at start, nothing before the interval, one run when it ends, and the next
/// one a full interval later.
#[test]
fn a_run_starts_each_time_the_interval_ends_and_never_at_start() {
    let mut tray = Tray::new();
    assert!(!tray.tick(at(0, 0), on(60)), "not at once");
    assert!(!tray.tick(at(30, 0), on(60)));
    assert!(!tray.tick(at(58, 59), on(60)));
    assert!(tray.tick(at(60, 0), on(60)));
    assert!(!tray.tick(at(61, 0), on(60)));
    assert!(!tray.tick(at(119, 0), on(60)));
    assert!(tray.tick(at(120, 0), on(60)));
}

/// AC 2, NFR-014: with the schedule off or the AI off nothing ever starts.
#[test]
fn nothing_starts_while_the_schedule_or_the_ai_is_off() {
    let mut tray = Tray::new();
    let schedule_off = AiSettings {
        schedule: Schedule::Off,
        ..on(5)
    };
    let ai_off = AiSettings {
        enrich: false,
        ..on(5)
    };
    for minute in [5, 60, 600] {
        assert!(!tray.tick(at(minute, 0), schedule_off.clone()));
        assert!(!tray.tick(at(minute + 2, 0), ai_off.clone()));
    }
}

/// AC 1: settings changed after the start apply on the next look, with no tray restart.
#[test]
fn a_changed_config_applies_without_a_restart() {
    let mut tray = Tray::new();
    assert!(!tray.tick(at(10, 0), on(60)));
    assert!(
        tray.tick(at(11, 0), on(5)),
        "the shorter interval has ended"
    );
    assert!(!tray.tick(at(17, 0), on(60)), "the longer one has not");
    assert!(tray.tick(at(71, 0), on(60)));
}

/// AC 2: a run that holds `ai-run.lock` blocks the timer; the next look after it ends starts one.
#[test]
fn a_held_run_lock_starts_nothing() {
    let mut tray = Tray::new();
    let folder = tray.notes.path().join(".easy-note");
    fs::create_dir_all(&folder).unwrap();
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(folder.join("ai-run.lock"))
        .unwrap();
    lock.lock().unwrap();
    assert!(!tray.tick(at(60, 0), on(60)));
    drop(lock);
    assert!(tray.tick(at(61, 0), on(60)));
}

/// AC 1, FR-049: a failed run's `next_after` holds the timer back until it has passed.
#[test]
fn the_back_off_of_a_failed_run_holds_the_timer() {
    let mut tray = Tray::new();
    tray.state(r#"{"failures":1,"next_after":"2026-11-27T12:30:00+00:00"}"#);
    assert!(!tray.tick(at(60, 0), on(60)));
    assert!(!tray.tick(at(149, 0), on(60)));
    assert!(tray.tick(at(150, 0), on(60)));
}

#[test]
fn sleeping_for_hours_gives_one_run_not_a_burst() {
    let mut tray = Tray::new();
    assert!(tray.tick(at(600, 0), on(60)));
    assert!(!tray.tick(at(601, 0), on(60)));
    assert!(!tray.tick(at(602, 0), on(60)));
}

/// A clock set back must not stop the timer until the old time comes again.
#[test]
fn a_clock_set_back_starts_the_wait_again() {
    let mut tray = Tray::new();
    assert!(!tray.tick(at(-600, 0), on(60)));
    assert!(!tray.tick(at(-541, 0), on(60)));
    assert!(tray.tick(at(-540, 0), on(60)));
}

/// The tray loop polls every second; config.toml is read once a minute.
#[test]
fn the_config_is_read_at_most_once_a_minute() {
    let notes = tempfile::tempdir().unwrap();
    let mut timer = AiTimer::new(start());
    let mut reads = 0;
    for second in 1..=180 {
        timer.tick(
            at(0, second),
            notes.path(),
            || {
                reads += 1;
                Ok(on(60))
            },
            || panic!("no run is due"),
        );
    }
    assert_eq!(reads, 3);
}

/// A TOML parse error has several lines and quotes the bad line: only its first line is said.
#[test]
fn a_bad_config_says_one_line_once_and_the_timer_goes_on() {
    let notes = tempfile::tempdir().unwrap();
    let mut timer = AiTimer::new(start());
    let broken = || {
        Err("config.toml: TOML parse error at line 2, column 8\n  |\n2 | model = oops\n  |         ^\nexpected a value".to_string())
    };
    let nothing = || panic!("a bad config starts nothing");

    let line = timer
        .tick(at(60, 0), notes.path(), broken, nothing)
        .unwrap();
    assert!(line.contains("TOML parse error at line 2"), "{line}");
    assert!(!line.contains('\n') && !line.contains("oops"), "{line}");
    let repeat = timer.tick(at(61, 0), notes.path(), broken, nothing);
    assert_eq!(repeat, None, "said once");

    let mut runs = 0;
    let said = timer.tick(at(62, 0), notes.path(), || Ok(on(60)), || runs += 1);
    assert_eq!((said, runs), (None, 1));
    // broken again after a good read: said again
    assert!(timer
        .tick(at(63, 0), notes.path(), broken, nothing)
        .is_some());
}

#[test]
fn a_notes_dir_that_is_not_there_yet_is_waited_for() {
    let parent = tempfile::tempdir().unwrap();
    let root: PathBuf = parent.path().join("mounted-later");
    let mut timer = AiTimer::new(start());
    let mut runs = 0;
    timer.tick(at(60, 0), &root, || Ok(on(60)), || runs += 1);
    assert_eq!(runs, 0);
    assert!(!root.exists(), "the timer creates nothing");
    fs::create_dir(&root).unwrap();
    timer.tick(at(61, 0), &root, || Ok(on(60)), || runs += 1);
    assert_eq!(runs, 1);
}

#[test]
fn a_timed_run_is_this_binary_with_ai_run() {
    let exe = Path::new("/opt/bin/easy-note");
    let (program, args) = ai_run_argv(exe);
    assert_eq!(program, exe);
    assert_eq!(args, ["ai", "run"]);
}
