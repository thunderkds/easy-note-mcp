//! HUN-265 (plan T010): the perf check over 10,000 notes (NFR-004, NFR-009, NFR-010, PRD OQ #4).
//!
//! `fixture` writes a Notes dir of valid note files; `fixture_has_10000_valid_notes` runs in
//! `cargo test`. The five timing tests are `#[ignore]`: they run the release binary over a fresh
//! 10,000-note fixture, each with 1 untimed first run and 30 timed runs, and print one line
//! `perf <name>: ...` before they assert the budget. Run them by hand with
//! `cargo test --release --test perf -- --ignored --nocapture --test-threads=1`.
//! Each test uses a temp HOME and EASY_NOTE_DIR (NFR-003) and a temp git repo as cwd.
//! HUN-278 adds `perf_window_board` (NFR-013): the window's `board` handler, in process, over
//! 5,000 notes; the handler gets 300 ms of the 1 s first-paint budget.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use chrono::{Datelike, Local, SecondsFormat, TimeDelta, TimeZone};
use easy_note_mcp::core::format::parse;
use easy_note_mcp::core::store::Store;
use easy_note_mcp::surfaces::window::api::handle;
use rmcp::model::{CallToolRequestParams, ClientCapabilities, ClientConfig, Implementation};
use rmcp::transport::TokioChildProcess;
use rmcp::ServiceExt;
use serde_json::{json, Value};

const BIN: &str = env!("CARGO_BIN_EXE_easy-note");
const NOTES: usize = 10_000;
const RUNS: usize = 30;
const SOURCES: [&str; 4] = ["cli", "mcp", "hook", "quick"];

const BUDGET_SEARCH_MS: u64 = 500;
const BUDGET_NOTE_ADD_MS: u64 = 150;
const BUDGET_HOOK_PASS_MS: u64 = 50;
const BUDGET_HOOK_SAVE_MS: u64 = 150;
const BUDGET_N_COLD_START_MS: u64 = 150;
const WINDOW_NOTES: usize = 5_000;
const BUDGET_WINDOW_BOARD_MS: u64 = 300;

// ---------------------------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------------------------

/// A temp root with a HOME and a notes dir path (the notes dir is created by `fixture`).
struct Sandbox {
    dir: tempfile::TempDir,
    home: PathBuf,
    notes: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let dir = tempfile::Builder::new()
            .prefix(&format!("easy-note-perf-{name}-"))
            .tempdir()
            .unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let notes = dir.path().join("notes");
        Sandbox { dir, home, notes }
    }
    fn root(&self) -> &Path {
        self.dir.path()
    }
}

/// Writes `count` valid note files under `notes`, as `YYYY/MM/<id>.md`. The same on every run.
///
/// Notes spread over 24 month folders ending this month, 5 repos, 20 tasks, 10 tags and the 4
/// surface sources. Every 10th body holds the word `retry`. The id is `YYYYMMDD-HHMMSS-xxxx`
/// (4 hex digits from the index) and matches the file name and `created`.
fn fixture(notes: &Path, count: usize) {
    let now = Local::now();
    let this_month = now.year() * 12 + now.month0() as i32;
    for i in 0..count {
        let months_back = 23 - (i % 24) as i32;
        let month_index = this_month - months_back;
        let (year, month0) = (
            month_index.div_euclid(12),
            month_index.rem_euclid(12) as u32,
        );
        let month_start = Local
            .with_ymd_and_hms(year, month0 + 1, 1, 0, 0, 0)
            .earliest()
            .expect("month start exists");
        let created = month_start + TimeDelta::seconds((i / 24) as i64 * 61);
        let id = format!("{}-{i:04x}", created.format("%Y%m%d-%H%M%S"));
        let stamp = created.to_rfc3339_opts(SecondsFormat::Secs, false);
        let body = if i % 10 == 0 {
            format!("retry bug on firefox, note {i}")
        } else {
            format!("routine note {i}")
        };
        let text = format!(
            "---\neasy-note: 1\nid: \"{id}\"\ncreated: \"{stamp}\"\nrepo: \"repo-{}\"\ntask: \"T{:03}\"\ntags: [\"tag{}\"]\nsource: \"{}\"\n---\n{body}\n",
            i % 5,
            i % 20,
            i % 10,
            SOURCES[i % 4],
        );
        let folder = notes
            .join(format!("{:04}", created.year()))
            .join(format!("{:02}", created.month()));
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join(format!("{id}.md")), text).unwrap();
    }
}

/// Every `.md` file under `dir`, sorted.
fn note_files(dir: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
    }
    let mut out = vec![];
    walk(dir, &mut out);
    out.sort();
    out
}

/// `easy-note <args>` (or any program) with the temp HOME, notes dir and git config, in `cwd`.
fn command(sb: &Sandbox, program: &Path, args: &[&str], cwd: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .env("HOME", &sb.home)
        .env("EASY_NOTE_DIR", &sb.notes)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .current_dir(cwd);
    cmd
}

/// One process, start included, timed to exit.
fn timed_run(sb: &Sandbox, program: &Path, args: &[&str], cwd: &Path) -> (Duration, Output) {
    let start = Instant::now();
    let out = command(sb, program, args, cwd).output().unwrap();
    (start.elapsed(), out)
}

/// One `hook-prompt` process fed the event on stdin, timed to exit.
fn timed_hook(sb: &Sandbox, cwd: &Path, event: &str) -> (Duration, Output) {
    let mut cmd = command(sb, Path::new(BIN), &["hook-prompt"], cwd);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let start = Instant::now();
    let mut child = cmd.spawn().unwrap();
    // The hook may exit before it reads everything; a broken pipe here shows up in the output.
    let _ = child.stdin.take().unwrap().write_all(event.as_bytes());
    let out = child.wait_with_output().unwrap();
    (start.elapsed(), out)
}

/// A recorded hook event with its `cwd` replaced by `cwd`.
fn event(name: &str, cwd: &Path) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/claude_hook")
        .join(name);
    let mut ev: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    ev["cwd"] = json!(cwd.to_string_lossy());
    ev.to_string()
}

fn git(cwd: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t.invalid"])
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A git repo `<root>/repo` on branch `feat/t010-perf`, the cwd for every timed run.
fn repo(root: &Path) -> PathBuf {
    let dir = root.join("repo");
    std::fs::create_dir_all(&dir).unwrap();
    git(&dir, &["init", "-q", "-b", "feat/t010-perf"]);
    git(&dir, &["commit", "-q", "--allow-empty", "-m", "first"]);
    dir
}

fn ms(d: Duration) -> String {
    format!("{:.1}", d.as_secs_f64() * 1000.0)
}

/// Prints the one `perf` line, then asserts p95 is under the budget. `times` holds the 30 timed
/// runs; p95 is the 29th of the 30 sorted times, as in `tests/quick.rs` AC6.
fn report(name: &str, first: Duration, mut times: Vec<Duration>, budget_ms: u64) {
    assert_eq!(times.len(), RUNS);
    times.sort();
    let rank = |pct: usize| times[(RUNS * pct).div_ceil(100) - 1];
    let (p50, p95, max) = (rank(50), rank(95), times[RUNS - 1]);
    eprintln!(
        "perf {name}: n={RUNS} first {} ms p50 {} ms p95 {} ms max {} ms",
        ms(first),
        ms(p50),
        ms(p95),
        ms(max)
    );
    assert!(
        p95 < Duration::from_millis(budget_ms),
        "{name}: p95 {} ms is over the {budget_ms} ms budget",
        ms(p95)
    );
}

// ---------------------------------------------------------------------------------------------
// The fixture is valid (runs in cargo test)
// ---------------------------------------------------------------------------------------------

#[test]
fn fixture_has_10000_valid_notes() {
    let sb = Sandbox::new("fixture");
    fixture(&sb.notes, NOTES);
    let files = note_files(&sb.notes);
    assert_eq!(files.len(), NOTES);
    for path in files.iter().step_by(100) {
        let text = std::fs::read_to_string(path).unwrap();
        let note = parse(&text).unwrap_or_else(|e| panic!("{}: {e:?}", path.display()));
        let id = note.get_str("id").unwrap();
        assert_eq!(path.file_stem().unwrap().to_str().unwrap(), id);
    }
    let out = command(&sb, Path::new(BIN), &["stats", "--json"], sb.root())
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stats: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(stats["total"], NOTES);
}

// ---------------------------------------------------------------------------------------------
// The timings (ignored: run with --release --ignored on an idle machine)
// ---------------------------------------------------------------------------------------------

#[test]
#[ignore]
fn perf_search() {
    let sb = Sandbox::new("search");
    fixture(&sb.notes, NOTES);
    let repo = repo(sb.root());
    let mut runs: Vec<Duration> = vec![];
    for _ in 0..=RUNS {
        let (elapsed, out) = timed_run(&sb, Path::new(BIN), &["search", "retry"], &repo);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!out.stdout.is_empty(), "search printed nothing");
        runs.push(elapsed);
    }
    let first = runs.remove(0);
    report("search", first, runs, BUDGET_SEARCH_MS);
}

#[tokio::test]
#[ignore]
async fn perf_note_add() {
    let sb = Sandbox::new("note_add");
    fixture(&sb.notes, NOTES);
    let repo = repo(sb.root());
    let mut cmd = tokio::process::Command::new(BIN);
    cmd.arg("serve")
        .env("HOME", &sb.home)
        .env("EASY_NOTE_DIR", &sb.notes)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null");
    let transport = TokioChildProcess::new(cmd).unwrap();
    let info = ClientConfig::new(
        ClientCapabilities::default(),
        Implementation::new("perf-client", "0"),
    );
    // The server starts and initialises once, before any timing (its start is outside the budget).
    let client = info.serve(transport).await.expect("initialize");
    let mut runs: Vec<Duration> = vec![];
    for run in 0..=RUNS {
        let before = note_files(&sb.notes).len();
        let args = json!({"text": format!("perf note {run} #perf"), "cwd": repo.to_string_lossy()});
        let params = CallToolRequestParams::new("note_add")
            .with_arguments(args.as_object().unwrap().clone());
        let start = Instant::now();
        let result = client.call_tool(params).await.expect("tool call answers");
        runs.push(start.elapsed());
        assert_ne!(
            result.is_error,
            Some(true),
            "note_add failed: {:?}",
            result.content
        );
        assert_eq!(note_files(&sb.notes).len(), before + 1, "one note per call");
    }
    client.cancel().await.unwrap();
    let first = runs.remove(0);
    report("note_add", first, runs, BUDGET_NOTE_ADD_MS);
}

#[test]
#[ignore]
fn perf_hook_pass() {
    let sb = Sandbox::new("hook_pass");
    fixture(&sb.notes, NOTES);
    let repo = repo(sb.root());
    let ev = event("passthrough_main_repo.json", &repo);
    let mut runs: Vec<Duration> = vec![];
    for _ in 0..=RUNS {
        let (elapsed, out) = timed_hook(&sb, &repo, &ev);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&out.stdout),
            "",
            "the pass path printed"
        );
        runs.push(elapsed);
    }
    let first = runs.remove(0);
    report("hook_pass", first, runs, BUDGET_HOOK_PASS_MS);
}

#[test]
#[ignore]
fn perf_hook_save() {
    let sb = Sandbox::new("hook_save");
    fixture(&sb.notes, NOTES);
    let repo = repo(sb.root());
    let ev = event("note_prefix_from_subdir.json", &repo);
    let mut runs: Vec<Duration> = vec![];
    for _ in 0..=RUNS {
        let before = note_files(&sb.notes).len();
        let (elapsed, out) = timed_hook(&sb, &repo, &ev);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains(r#""decision":"block""#), "{stdout}");
        assert_eq!(note_files(&sb.notes).len(), before + 1, "one note per save");
        runs.push(elapsed);
    }
    let first = runs.remove(0);
    report("hook_save", first, runs, BUDGET_HOOK_SAVE_MS);
}

#[test]
#[ignore]
fn perf_n_cold_start() {
    let sb = Sandbox::new("n_cold_start");
    fixture(&sb.notes, NOTES);
    let repo = repo(sb.root());
    let link = sb.root().join("n");
    std::os::unix::fs::symlink(BIN, &link).unwrap();
    let mut runs: Vec<Duration> = vec![];
    for run in 0..=RUNS {
        let text = run.to_string();
        let before = note_files(&sb.notes).len();
        let (elapsed, out) =
            timed_run(&sb, &link, &["perf", "note", text.as_str(), "#perf"], &repo);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!out.stdout.is_empty(), "n printed no confirmation");
        assert_eq!(note_files(&sb.notes).len(), before + 1, "one note per run");
        runs.push(elapsed);
    }
    let first = runs.remove(0);
    report("n_cold_start", first, runs, BUDGET_N_COLD_START_MS);
}

#[test]
#[ignore]
fn perf_window_board() {
    let sb = Sandbox::new("window_board");
    fixture(&sb.notes, WINDOW_NOTES);
    let store = Store::system(sb.notes.clone());
    let mut runs: Vec<Duration> = vec![];
    for _ in 0..=RUNS {
        let start = Instant::now();
        let out = handle(&store, r#"{"cmd": "board"}"#);
        runs.push(start.elapsed());
        let board: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(board["total"], WINDOW_NOTES, "every note on the board");
    }
    let first = runs.remove(0);
    report("window_board", first, runs, BUDGET_WINDOW_BOARD_MS);
}
