//! HUN-278: the dashboard window's pure parts (FR-042, FR-034, NFR-001, NFR-002, DDR 0009).
//!
//! `window::api::handle` answers the page's IPC requests over a `Store`; it needs no window and
//! no display, so every test here runs in the default build. Each test uses its own temp notes
//! dir and a store with a frozen clock and zone (2026-11-19 10:00 at +07:00). The asset scan
//! reads `src/surfaces/window/assets/`. No literal whole secret token lives here: the values come
//! from `tests/fixtures/secrets/planted.json`, split in parts.

use std::path::{Path, PathBuf};

use easy_note_mcp::core::ai;
use easy_note_mcp::core::store::Store;
use easy_note_mcp::surfaces::dashboard::use_window;
use easy_note_mcp::surfaces::window::{
    allowed_url, api::handle, asset, ipc, reply_script, CSP, REFRESH,
};
use serde_json::{json, Value};

const PLANTED_JSON: &str = include_str!("fixtures/secrets/planted.json");

// ---------------------------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------------------------

/// 2026-11-19T10:00:00+07:00 in nanoseconds since the epoch.
fn frozen_now() -> u128 {
    let secs = chrono::DateTime::parse_from_rfc3339("2026-11-19T10:00:00+07:00")
        .unwrap()
        .timestamp();
    u128::try_from(secs).unwrap() * 1_000_000_000
}

fn plus7(_: i64) -> i32 {
    7 * 3600
}

struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        Sandbox {
            dir: tempfile::Builder::new()
                .prefix(&format!("easy-note-window-{name}-"))
                .tempdir()
                .unwrap(),
        }
    }
    fn notes(&self) -> PathBuf {
        self.dir.path().join("notes")
    }
    fn store(&self) -> Store {
        Store {
            now_ns: frozen_now,
            utc_offset: plus7,
            ..Store::system(self.notes())
        }
    }
    /// One note file written by hand; `extra` holds front matter lines (each ending in `\n`).
    fn note(&self, id: &str, created: &str, extra: &str, body: &str) {
        let folder = self.notes().join(&id[..4]).join(&id[4..6]);
        std::fs::create_dir_all(&folder).unwrap();
        let text = format!(
            "---\neasy-note: 1\nid: \"{id}\"\ncreated: \"{created}\"\n{extra}source: \"cli\"\n---\n{body}\n"
        );
        std::fs::write(folder.join(format!("{id}.md")), text).unwrap();
    }
    fn enrich(&self, id: &str, kind: &str) {
        let tags = vec!["firefox".to_string()];
        let input = ai::Input {
            title: "Firefox-only timeout",
            kind,
            summary: "Times out after a second navigation.",
            tags_suggested: &tags,
            model: "claude-sonnet-4-5",
        };
        ai::enrich(&self.store(), id, &input).unwrap();
    }
    fn ai_file(&self, id: &str, detail: &Value) {
        let dir = self.notes().join(".easy-note/ai");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), detail.to_string()).unwrap();
    }
    /// Four notes: today (AI, fresh), yesterday (AI, stale), last week, and no project; plus one
    /// file the parser cannot read.
    fn four(&self) {
        self.note(
            "20261119-090000-aaaa",
            "2026-11-19T09:00:00+07:00",
            "repo: \"easy-ui-mcp\"\nbranch: \"fix/t120-firefox\"\ntask: \"T120\"\ntags: [\"flaky\", \"firefox\"]\n",
            "Retry bug only on Firefox\n\nui_wait_for times out.",
        );
        self.note(
            "20261118-200000-bbbb",
            "2026-11-18T20:00:00+07:00",
            "repo: \"easy-ui-mcp\"\ntags: []\n",
            "Test /note during a running turn",
        );
        self.note(
            "20261112-080000-cccc",
            "2026-11-12T08:00:00+07:00",
            "repo: \"easy-kit\"\ntask: \"T7\"\ntags: [\"wake\"]\n",
            "Explain movement in the Wake brief",
        );
        self.note(
            "20261101-080000-dddd",
            "2026-11-01T08:00:00+07:00",
            "tags: []\n",
            "Buy a USB-C hub",
        );
        let bad = self.notes().join("2026/11/20261119-080000-bad0.md");
        std::fs::write(
            bad,
            "---\neasy-note: 1\nid: \"20261119-080000-bad0\ntags: [oops\n---\nx\n",
        )
        .unwrap();
        self.enrich("20261119-090000-aaaa", "bug");
        self.enrich("20261118-200000-bbbb", "todo");
        // The note changes after the AI wrote its detail: stale.
        let path = self.notes().join("2026/11/20261118-200000-bbbb.md");
        let mut text = std::fs::read_to_string(&path).unwrap();
        text.push_str("one more line\n");
        std::fs::write(path, text).unwrap();
    }
}

fn call(store: &Store, request: &Value) -> Value {
    let out = handle(store, &request.to_string());
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out}"))
}

fn ids(board: &Value) -> Vec<&str> {
    board["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect()
}

fn column<'a>(board: &'a Value, key: &str) -> Vec<&'a Value> {
    board["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| &n[key])
        .collect()
}

/// Every string value in a JSON document.
fn strings(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| strings(x, out)),
        Value::Object(o) => o.values().for_each(|x| strings(x, out)),
        _ => {}
    }
}

/// `(text, secret)` of every planted case.
fn planted() -> Vec<(String, String)> {
    let join = |v: &Value| -> String {
        v.as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap())
            .collect()
    };
    let cases: Value = serde_json::from_str(PLANTED_JSON).unwrap();
    cases
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (join(&c["text"]), join(&c["secret"])))
        .collect()
}

fn assets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/surfaces/window/assets")
}

/// Every file under `dir`, sorted.
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut out = vec![];
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(files(&path));
        } else {
            out.push(path);
        }
    }
    out.sort();
    out
}

// ---------------------------------------------------------------------------------------------
// board (AC 3, 4, 7)
// ---------------------------------------------------------------------------------------------

#[test]
fn board_counts_orders_and_marks_the_notes() {
    let sb = Sandbox::new("board");
    sb.four();
    let board = call(&sb.store(), &json!({"cmd": "board"}));
    assert!(board.get("error").is_none(), "{board}");
    assert_eq!(
        board["notes_dir"].as_str().unwrap(),
        sb.notes().to_string_lossy()
    );
    assert_eq!(board["today"], 1, "{board}");
    assert_eq!(board["total"], 4, "skipped file not counted: {board}");
    assert_eq!(board["projects"], 2);
    assert_eq!(
        board["top_projects"],
        json!([{"name": "easy-ui-mcp", "n": 2}, {"name": "easy-kit", "n": 1}])
    );
    assert_eq!(
        ids(&board),
        [
            "20261119-090000-aaaa",
            "20261118-200000-bbbb",
            "20261112-080000-cccc",
            "20261101-080000-dddd"
        ]
    );
    let day: Vec<&Value> = column(&board, "day");
    assert_eq!(
        day,
        [
            &json!("today"),
            &json!("yesterday"),
            &json!("earlier"),
            &json!("earlier")
        ]
    );
    assert_eq!(
        column(&board, "kind"),
        [&json!("bug"), &json!("to-do"), &Value::Null, &Value::Null]
    );
    assert_eq!(
        column(&board, "ai"),
        [&json!(true), &json!(true), &json!(false), &json!(false)]
    );
    assert_eq!(
        column(&board, "stale"),
        [&json!(false), &json!(true), &json!(false), &json!(false)]
    );

    let first = &board["notes"][0];
    assert_eq!(first["time"], "09:00");
    assert_eq!(first["created"], "2026-11-19T09:00:00+07:00");
    assert_eq!(first["first"], "Retry bug only on Firefox");
    assert_eq!(
        first["text"],
        "Retry bug only on Firefox\n\nui_wait_for times out.\n"
    );
    assert_eq!(first["repo"], "easy-ui-mcp");
    assert_eq!(first["task"], "T120");
    assert_eq!(first["tags"], json!(["flaky", "firefox"]));
    assert_eq!(first["source"], "cli");
    let last = &board["notes"][3];
    assert_eq!(last["repo"], Value::Null);
    assert_eq!(last["task"], Value::Null);
    assert_eq!(last["tags"], json!([]));
}

#[test]
fn top_projects_keep_three_ties_by_name() {
    let sb = Sandbox::new("top");
    for (i, repo) in ["d", "c", "b", "a", "a"].iter().enumerate() {
        sb.note(
            &format!("20261110-08000{i}-000{i}"),
            &format!("2026-11-10T08:00:0{i}+07:00"),
            &format!("repo: \"{repo}\"\ntags: []\n"),
            "x",
        );
    }
    sb.note(
        "20261110-080009-0009",
        "2026-11-10T08:00:09+07:00",
        "repo: \"\"\ntags: []\n",
        "empty repo",
    );
    let board = call(&sb.store(), &json!({"cmd": "board"}));
    assert_eq!(board["projects"], 4, "{board}");
    assert_eq!(
        board["top_projects"],
        json!([{"name": "a", "n": 2}, {"name": "b", "n": 1}, {"name": "c", "n": 1}])
    );
}

#[test]
fn an_empty_or_missing_notes_dir_gives_zeros() {
    let sb = Sandbox::new("empty");
    let board = call(&sb.store(), &json!({"cmd": "board"}));
    assert_eq!(board["today"], 0, "{board}");
    assert_eq!(board["total"], 0);
    assert_eq!(board["projects"], 0);
    assert_eq!(board["top_projects"], json!([]));
    assert_eq!(board["notes"], json!([]));
}

#[test]
fn inbox_notes_are_on_the_board_and_have_a_detail() {
    let sb = Sandbox::new("inbox");
    std::fs::create_dir_all(sb.notes().join("inbox")).unwrap();
    std::fs::write(sb.notes().join("inbox/phone.md"), "from the phone\n").unwrap();
    let board = call(&sb.store(), &json!({"cmd": "board"}));
    assert_eq!(ids(&board), ["inbox/phone"], "{board}");
    let detail = call(&sb.store(), &json!({"cmd": "detail", "id": "inbox/phone"}));
    assert_eq!(detail["first"], "from the phone", "{detail}");
}

// ---------------------------------------------------------------------------------------------
// detail (AC 5)
// ---------------------------------------------------------------------------------------------

#[test]
fn detail_has_the_ai_detail_and_the_context() {
    let sb = Sandbox::new("detail");
    sb.four();
    let d = call(
        &sb.store(),
        &json!({"cmd": "detail", "id": "20261119-090000-aaaa"}),
    );
    assert!(d.get("error").is_none(), "{d}");
    assert_eq!(d["id"], "20261119-090000-aaaa");
    assert_eq!(d["day"], "today");
    assert_eq!(d["kind"], "bug");
    assert_eq!(d["masked"], false);
    assert_eq!(
        d["context"],
        json!({"branch": "fix/t120-firefox", "commit": null, "session": null})
    );
    let a = &d["ai"];
    assert_eq!(a["title"], "Firefox-only timeout");
    assert_eq!(a["kind"], "bug");
    assert_eq!(a["summary"], "Times out after a second navigation.");
    assert_eq!(a["tags_suggested"], json!(["firefox"]));
    assert_eq!(a["model"], "claude-sonnet-4-5");
    assert_eq!(a["at"], "2026-11-19T10:00:00+07:00");
    assert_eq!(a["stale"], false);

    let stale = call(
        &sb.store(),
        &json!({"cmd": "detail", "id": "20261118-200000-bbbb"}),
    );
    assert_eq!(stale["ai"]["stale"], true, "{stale}");
    assert_eq!(stale["ai"]["kind"], "to-do");
    assert_eq!(stale["stale"], true);

    let plain = call(
        &sb.store(),
        &json!({"cmd": "detail", "id": "20261112-080000-cccc"}),
    );
    assert_eq!(plain["ai"], Value::Null, "{plain}");
    assert_eq!(
        plain["context"],
        json!({"branch": null, "commit": null, "session": null})
    );
    assert_eq!(plain["kind"], Value::Null);
}

// ---------------------------------------------------------------------------------------------
// errors (AC 3)
// ---------------------------------------------------------------------------------------------

#[test]
fn bad_requests_answer_an_error_object() {
    let sb = Sandbox::new("errors");
    sb.four();
    let store = sb.store();
    for request in [
        "",
        "not json",
        "[]",
        "{}",
        r#"{"cmd": 5}"#,
        r#"{"cmd": "nope"}"#,
        r#"{"cmd": "detail"}"#,
        r#"{"cmd": "detail", "id": 7}"#,
        r#"{"cmd": "detail", "id": "../../etc/passwd"}"#,
        r#"{"cmd": "detail", "id": "inbox/../../etc/passwd"}"#,
        r#"{"cmd": "detail", "id": "2026/11/20261119-090000-aaaa"}"#,
        r#"{"cmd": "detail", "id": "20261119-090000-ffff"}"#,
    ] {
        let out: Value = serde_json::from_str(&handle(&store, request))
            .unwrap_or_else(|e| panic!("{request}: not JSON: {e}"));
        let obj = out
            .as_object()
            .unwrap_or_else(|| panic!("{request}: {out}"));
        assert_eq!(obj.len(), 1, "{request}: {out}");
        let text = obj["error"]
            .as_str()
            .unwrap_or_else(|| panic!("{request}: {out}"));
        assert!(!text.is_empty() && text.len() < 80, "{request}: {text}");
        assert!(!text.contains("passwd"), "the id is not echoed: {text}");
    }
}

// ---------------------------------------------------------------------------------------------
// masking (AC 6, NFR-002)
// ---------------------------------------------------------------------------------------------

#[test]
fn no_planted_secret_leaves_through_board_or_detail() {
    let sb = Sandbox::new("leak");
    let cases = planted();
    let body: Vec<&str> = cases.iter().map(|(t, _)| t.as_str()).collect();
    let id = "20261119-090000-5ec7";
    let first_secret = &cases[0].1;
    sb.note(
        id,
        "2026-11-19T09:00:00+07:00",
        &format!(
            "repo: \"r {first_secret}\"\nbranch: \"b {first_secret}\"\ntask: \"T1\"\ntags: [\"t\"]\n"
        ),
        &body.join("\n"),
    );
    sb.ai_file(
        id,
        &json!({
            "easy-note-ai": 1,
            "id": id,
            "title": format!("title {first_secret}"),
            "kind": "bug",
            "summary": format!("summary {} and {}", cases[1].0, cases[2].0),
            "tags_suggested": ["ok"],
            "based_on": "sha256:0",
            "model": "claude-sonnet-4-5",
            "at": "2026-11-19T09:30:00+07:00"
        }),
    );
    let store = sb.store();
    let board_raw = handle(&store, r#"{"cmd": "board"}"#);
    let detail_raw = handle(&store, &json!({"cmd": "detail", "id": id}).to_string());
    let detail: Value = serde_json::from_str(&detail_raw).unwrap();
    assert!(detail["ai"].is_object(), "the AI file was read: {detail}");
    assert_eq!(detail["masked"], true, "{detail}");
    for raw in [&board_raw, &detail_raw] {
        let mut all = vec![];
        strings(&serde_json::from_str(raw).unwrap(), &mut all);
        for (_, secret) in &cases {
            assert!(!raw.contains(secret.as_str()), "secret in {raw}");
            assert!(
                all.iter().all(|s| !s.contains(secret.as_str())),
                "secret in a string of {raw}"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// choice of window (AC 2)
// ---------------------------------------------------------------------------------------------

#[test]
fn the_window_opens_only_when_built_and_a_display_is_set() {
    assert!(use_window(true, Some("wayland-0"), None));
    assert!(use_window(true, None, Some(":0")));
    assert!(use_window(true, Some("wayland-0"), Some(":0")));
    assert!(use_window(true, Some(""), Some(":0")));
    assert!(!use_window(true, None, None));
    assert!(!use_window(true, Some(""), Some("")));
    assert!(!use_window(false, Some("wayland-0"), Some(":0")));
    assert!(!use_window(false, None, None));
}

// ---------------------------------------------------------------------------------------------
// the page and its shell (AC 9)
// ---------------------------------------------------------------------------------------------

#[test]
fn the_csp_allows_only_the_app_scheme() {
    assert_eq!(
        CSP,
        "default-src easy-note:; style-src easy-note: 'unsafe-inline'"
    );
    let (mime, body) = asset("/").expect("the page");
    assert_eq!(mime, "text/html; charset=utf-8");
    assert!(std::str::from_utf8(body).unwrap().contains("app.js"));
    assert_eq!(asset("/index.html").unwrap().1, body);
    assert_eq!(
        asset("/app.js").unwrap().0,
        "text/javascript; charset=utf-8"
    );
    assert_eq!(asset("/app.css").unwrap().0, "text/css; charset=utf-8");
    assert_eq!(
        asset("/ipc.js").unwrap().0,
        "text/javascript; charset=utf-8"
    );
    for path in [
        "/nope",
        "/../Cargo.toml",
        "//etc/passwd",
        "/assets/app.js",
        "",
    ] {
        assert!(asset(path).is_none(), "{path}");
    }
}

#[test]
fn navigation_stays_on_the_app_scheme() {
    assert!(allowed_url("easy-note://localhost/"));
    assert!(allowed_url("easy-note://localhost/index.html"));
    for url in [
        "https://example.com/",
        "http://127.0.0.1:8769/",
        "file:///etc/passwd",
        "about:blank",
        "javascript:alert(1)",
        "data:text/html,x",
        "easy-note-evil://x",
        "",
    ] {
        assert!(!allowed_url(url), "{url}");
    }
}

#[test]
fn the_reply_is_a_quoted_js_string() {
    let script = reply_script(r#"{"a":"</script>\"); alert(1); (\"","b":"x y"}"#);
    let arg = script
        .strip_prefix("window.__easyNote.reply(")
        .and_then(|s| s.strip_suffix(')'))
        .expect(&script);
    let back: String = serde_json::from_str(arg).expect("one JSON string literal");
    assert_eq!(back, r#"{"a":"</script>\"); alert(1); (\"","b":"x y"}"#);
}

#[test]
fn every_served_asset_is_scanned_and_safe() {
    let dir = assets_dir();
    let sinks = regex::Regex::new(
        r"innerHTML|outerHTML|insertAdjacentHTML|document\.write|\beval\s*\(|new\s+Function\b|setTimeout\s*\(\s*['`]|setInterval\s*\(\s*['`]",
    )
    .unwrap();
    let mut scanned = 0;
    for path in files(&dir) {
        let rel = path
            .strip_prefix(&dir)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        if let Some(font) = rel.strip_prefix("fonts/") {
            // Bundled fonts, each served; the verbatim SIL licence sits beside them, not served.
            if font.ends_with(".woff2") {
                let (mime, served) =
                    asset(&format!("/{rel}")).unwrap_or_else(|| panic!("{rel} is not served"));
                assert_eq!(mime, "font/woff2", "{rel}");
                assert_eq!(served, std::fs::read(&path).unwrap(), "{rel}");
            } else {
                assert_eq!(font, "OFL.txt", "unexpected file {rel}");
            }
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        assert!(
            ["html", "css", "js"].contains(&ext),
            "unexpected file {rel}"
        );
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(
            !text.contains("http:") && !text.contains("https:") && !text.contains("//cdn"),
            "remote URL in {rel}"
        );
        if ext != "css" {
            assert!(
                sinks.find(&text).is_none(),
                "unsafe DOM sink in {rel}: {:?}",
                sinks.find(&text)
            );
        }
        let (_, served) =
            asset(&format!("/{rel}")).unwrap_or_else(|| panic!("{rel} is not served"));
        assert_eq!(served, text.as_bytes(), "{rel} is served as built");
        scanned += 1;
    }
    assert!(scanned >= 4, "index.html, ipc.js, app.js, app.css");
    assert!(
        asset("/fonts/OFL.txt").is_none(),
        "the licence is shipped, not served"
    );
}

#[test]
fn every_reference_in_the_page_is_served() {
    // index.html's src/href and app.css's url(...) must each resolve through `asset`, so the page
    // never asks for a file the binary does not hold (and never for one outside the app scheme).
    let refs = regex::Regex::new(r#"(?:src|href)="([^"]+)"|url\(([^)]+)\)"#).unwrap();
    let mut found = vec![];
    for page in ["/", "/app.css"] {
        let text = std::str::from_utf8(asset(page).unwrap().1).unwrap();
        for c in refs.captures_iter(text) {
            let r = c
                .get(1)
                .or_else(|| c.get(2))
                .unwrap()
                .as_str()
                .trim_matches('\'');
            assert!(
                asset(&format!("/{r}")).is_some(),
                "{page} refers to {r}, not served"
            );
            found.push(r.to_string());
        }
    }
    for r in ["app.css", "ipc.js", "app.js"] {
        assert!(found.iter().any(|f| f == r), "{r} not referenced");
    }
    let fonts = found.iter().filter(|f| f.ends_with(".woff2")).count();
    assert_eq!(fonts, 5, "five Plex faces: {found:?}");
}

#[test]
fn the_focus_refresh_calls_a_function_the_page_defines() {
    // AC 10: the shell runs REFRESH on each window focus; the page must define what it calls.
    assert_eq!(REFRESH, "window.__easyNote && window.__easyNote.refresh()");
    let page = std::str::from_utf8(asset("/app.js").unwrap().1).unwrap();
    assert!(
        page.contains("window.__easyNote.refresh = loadBoard;"),
        "app.js must set window.__easyNote.refresh to the board load"
    );
    let html = std::str::from_utf8(asset("/").unwrap().1).unwrap();
    let ipc_at = html.find(r#"src="ipc.js""#).expect("ipc.js script");
    let app_at = html.find(r#"src="app.js""#).expect("app.js script");
    assert!(
        ipc_at < app_at,
        "ipc.js defines window.__easyNote before app.js"
    );
}

// ---------------------------------------------------------------------------------------------
// the IPC envelope: `req` correlates a reply with its request
// ---------------------------------------------------------------------------------------------

#[test]
fn ipc_echoes_req_on_answers_and_errors() {
    let sb = Sandbox::new("ipc");
    sb.four();
    let store = sb.store();
    let board: Value = serde_json::from_str(&ipc(&store, r#"{"req": 7, "cmd": "board"}"#)).unwrap();
    assert_eq!(board["req"], 7, "{board}");
    assert_eq!(board["total"], 4);
    let detail: Value = serde_json::from_str(&ipc(
        &store,
        r#"{"req": 8, "cmd": "detail", "id": "20261112-080000-cccc"}"#,
    ))
    .unwrap();
    assert_eq!(detail["req"], 8);
    assert_eq!(
        detail["id"], "20261112-080000-cccc",
        "the note id is not the req"
    );
    let error: Value = serde_json::from_str(&ipc(&store, r#"{"req": 9, "cmd": "nope"}"#)).unwrap();
    assert_eq!(error, json!({"error": "unknown command", "req": 9}));
    // A req that is not a number is dropped, never echoed.
    let odd: Value =
        serde_json::from_str(&ipc(&store, r#"{"req": "<b>x</b>", "cmd": "nope"}"#)).unwrap();
    assert_eq!(odd, json!({"error": "unknown command"}));
    for body in ["", "not json", "[1]"] {
        let out: Value = serde_json::from_str(&ipc(&store, body)).unwrap();
        assert!(out["error"].is_string(), "{body}: {out}");
    }
}

// ---------------------------------------------------------------------------------------------
// HUN-279: actions on the selected note (FR-043, FR-032, NFR-001, DDR 0009 item 5)
// ---------------------------------------------------------------------------------------------

const BIN: &str = env!("CARGO_BIN_EXE_easy-note");
const ACTED: &str = "20261119-090000-acdc";

/// Serializes the tests that put a fake `xdg-open` on `PATH` (the process environment is shared).
static PATH_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// `PATH` holds only a temp dir with a fake `xdg-open` that logs `<argc>:<arg 1>` per call and
/// exits with `code`; the old `PATH` comes back on drop.
struct FakeOpener {
    log: PathBuf,
    old: Option<std::ffi::OsString>,
    _bin: tempfile::TempDir,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl FakeOpener {
    fn new(code: i32) -> FakeOpener {
        use std::os::unix::fs::PermissionsExt;
        let lock = PATH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let bin = tempfile::Builder::new()
            .prefix("easy-note-window-bin-")
            .tempdir()
            .unwrap();
        let log = bin.path().join("calls.log");
        let script = bin.path().join("xdg-open");
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\nprintf '%s:%s\\n' \"$#\" \"$1\" >> '{}'\nexit {code}\n",
                log.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let old = std::env::var_os("PATH");
        std::env::set_var("PATH", bin.path());
        FakeOpener {
            log,
            old,
            _bin: bin,
            _lock: lock,
        }
    }
    fn calls(&self) -> Vec<String> {
        std::fs::read_to_string(&self.log)
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }
}

impl Drop for FakeOpener {
    fn drop(&mut self) {
        match &self.old {
            Some(v) => std::env::set_var("PATH", v),
            None => std::env::remove_var("PATH"),
        }
    }
}

/// Every file under `dir` with its bytes, by path relative to `dir`.
fn snapshot(dir: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    if !dir.exists() {
        return Default::default();
    }
    files(dir)
        .into_iter()
        .map(|p| {
            let bytes = std::fs::read(&p).unwrap();
            (p.strip_prefix(dir).unwrap().to_path_buf(), bytes)
        })
        .collect()
}

/// The text after the front matter, byte for byte.
fn body_of(file: &str) -> &str {
    let end = file[4..].find("\n---\n").expect("front matter end") + 4;
    &file[end + 5..]
}

/// One note `ACTED` with tags `["y"]` and an AI file suggesting `["x", "y"]`.
fn acted(sb: &Sandbox) -> PathBuf {
    sb.note(
        ACTED,
        "2026-11-19T09:00:00+07:00",
        "repo: \"easy-ui-mcp\"\ntags: [\"y\"]\n",
        "Retry bug only on Firefox\n\n  keep   these bytes  ",
    );
    let tags = vec!["x".to_string(), "y".to_string()];
    let input = ai::Input {
        title: "Firefox-only timeout",
        kind: "bug",
        summary: "Times out after a second navigation.",
        tags_suggested: &tags,
        model: "claude-sonnet-4-5",
    };
    ai::enrich(&sb.store(), ACTED, &input).unwrap();
    sb.notes().join(format!("2026/11/{ACTED}.md"))
}

fn ai_path(sb: &Sandbox, id: &str) -> PathBuf {
    sb.notes().join(format!(".easy-note/ai/{id}.json"))
}

#[test]
fn accept_ai_adds_the_suggested_tags_like_the_cli() {
    // AC 1, Success Criterion 1.
    let sb = Sandbox::new("accept");
    let path = acted(&sb);
    let before = std::fs::read_to_string(&path).unwrap();
    let out = call(&sb.store(), &json!({"cmd": "accept_ai", "id": ACTED}));
    assert_eq!(out["id"], ACTED, "the updated detail: {out}");
    assert_eq!(out["tags"], json!(["y", "x"]), "{out}");
    assert_eq!(out["ai"]["tags_suggested"], json!(["x", "y"]), "{out}");
    assert_eq!(out["ai"]["stale"], false, "the text did not change: {out}");
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(body_of(&after), body_of(&before), "the text is unchanged");
    assert!(after.contains("tags: [\"y\", \"x\"]"), "{after}");
    // A second press adds nothing (no duplicate tags).
    let again = call(&sb.store(), &json!({"cmd": "accept_ai", "id": ACTED}));
    assert_eq!(again["tags"], json!(["y", "x"]), "{again}");

    // The CLI on the same start state writes the same note (but for its `updated` stamp).
    let twin = Sandbox::new("accept-cli");
    let twin_path = acted(&twin);
    let home = twin.dir.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let o = std::process::Command::new(BIN)
        .args(["tag", ACTED, "--accept-ai"])
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("EASY_NOTE_DIR", twin.notes())
        .current_dir(&home)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let strip = |t: &str| -> String {
        t.lines()
            .filter(|l| !l.starts_with("updated:"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let window_file = std::fs::read_to_string(&path).unwrap();
    let cli_file = std::fs::read_to_string(&twin_path).unwrap();
    assert_eq!(strip(&window_file), strip(&cli_file));
}

#[test]
fn accept_ai_without_ai_detail_is_an_error_and_writes_nothing() {
    let sb = Sandbox::new("accept-none");
    sb.four();
    let store = sb.store();
    let id = "20261112-080000-cccc";
    let before = snapshot(&sb.notes());
    let out = call(&store, &json!({"cmd": "accept_ai", "id": id}));
    assert_eq!(out, json!({"error": "no AI detail"}));
    assert_eq!(snapshot(&sb.notes()), before);
}

#[test]
fn trash_then_restore_brings_back_the_note_and_its_ai_file() {
    // AC 2, AC 3, AC 7, Success Criterion 2.
    let sb = Sandbox::new("trash");
    sb.four();
    let path = acted(&sb);
    let store = sb.store();
    let note = std::fs::read(&path).unwrap();
    let detail = std::fs::read(ai_path(&sb, ACTED)).unwrap();

    let out = call(&store, &json!({"cmd": "trash", "id": ACTED}));
    assert_eq!(out, json!({"trashed": ACTED}));
    assert!(!path.exists() && !ai_path(&sb, ACTED).exists());
    let board = call(&store, &json!({"cmd": "board"}));
    assert!(!ids(&board).contains(&ACTED), "the row is gone");
    assert_eq!(board["total"], 4);
    // A trashed note is refused by accept_ai and open_in_editor; a second trash too.
    for cmd in ["accept_ai", "open_in_editor", "trash", "detail"] {
        let out = call(&store, &json!({"cmd": cmd, "id": ACTED}));
        assert_eq!(out, json!({"error": "no such note"}), "{cmd}");
    }

    let out = call(&store, &json!({"cmd": "restore", "id": ACTED}));
    assert_eq!(out, json!({"restored": ACTED}));
    assert_eq!(
        std::fs::read(&path).unwrap(),
        note,
        "note file back, same bytes"
    );
    assert_eq!(
        std::fs::read(ai_path(&sb, ACTED)).unwrap(),
        detail,
        "AI file back, same bytes"
    );
    let board = call(&store, &json!({"cmd": "board"}));
    assert!(ids(&board).contains(&ACTED), "the row is back");
    assert_eq!(board["total"], 5);

    // Undo after the note came back another way: an error, nothing changes.
    let before = snapshot(&sb.notes());
    let out = call(&store, &json!({"cmd": "restore", "id": ACTED}));
    assert_eq!(out, json!({"error": "no such note"}));
    assert_eq!(snapshot(&sb.notes()), before);
}

#[test]
fn open_in_editor_runs_xdg_open_once_with_the_note_path() {
    // AC 4, Success Criterion 3.
    let sb = Sandbox::new("open");
    let path = acted(&sb);
    let store = sb.store();
    let before = snapshot(&sb.notes());
    {
        let fake = FakeOpener::new(0);
        let out = call(&store, &json!({"cmd": "open_in_editor", "id": ACTED}));
        assert_eq!(out, json!({"opened": ACTED}));
        assert_eq!(fake.calls(), vec![format!("1:{}", path.display())]);
    }
    {
        let fake = FakeOpener::new(4);
        let out = call(&store, &json!({"cmd": "open_in_editor", "id": ACTED}));
        assert_eq!(out, json!({"error": "xdg-open failed"}));
        assert_eq!(fake.calls().len(), 1);
    }
    {
        let fake = FakeOpener::new(0);
        std::fs::remove_file(fake._bin.path().join("xdg-open")).unwrap();
        let out = call(&store, &json!({"cmd": "open_in_editor", "id": ACTED}));
        assert_eq!(out, json!({"error": "cannot run xdg-open"}));
    }
    assert_eq!(snapshot(&sb.notes()), before, "opening writes nothing");
}

#[test]
fn actions_refuse_bad_and_unknown_ids_and_change_nothing() {
    // AC 6, Success Criterion 4.
    let sb = Sandbox::new("action-errors");
    sb.four();
    acted(&sb);
    let inbox = sb.notes().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::write(inbox.join("p.md"), "phone note\n").unwrap();
    let store = sb.store();
    let fake = FakeOpener::new(0);
    let before = snapshot(&sb.notes());
    for cmd in ["accept_ai", "trash", "restore", "open_in_editor"] {
        for (id, want) in [
            (json!(null), "missing id"),
            (json!(7), "missing id"),
            (json!("../../etc/passwd"), "bad id"),
            (json!("2026/11/20261119-090000-aaaa"), "bad id"),
            (json!("20261119-090000-ffff"), "no such note"),
        ] {
            let mut request = json!({"cmd": cmd});
            if !id.is_null() {
                request["id"] = id.clone();
            }
            let out = call(&store, &request);
            assert_eq!(out, json!({"error": want}), "{request}");
        }
    }
    // A live note is not in the trash.
    let out = call(&store, &json!({"cmd": "restore", "id": ACTED}));
    assert_eq!(out, json!({"error": "no such note"}));
    // Inbox notes are read-only, as in the CLI.
    let out = call(&store, &json!({"cmd": "accept_ai", "id": "inbox/p"}));
    assert_eq!(out, json!({"error": "inbox notes are read-only"}));
    assert_eq!(snapshot(&sb.notes()), before, "no change on disk");
    assert!(fake.calls().is_empty(), "xdg-open never ran");
}

#[test]
fn no_planted_secret_leaves_through_the_actions() {
    let sb = Sandbox::new("action-leak");
    let cases = planted();
    let body: Vec<&str> = cases.iter().map(|(t, _)| t.as_str()).collect();
    let id = "20261119-090000-5ec8";
    sb.note(
        id,
        "2026-11-19T09:00:00+07:00",
        "tags: [\"t\"]\n",
        &body.join("\n"),
    );
    sb.ai_file(
        id,
        &json!({
            "easy-note-ai": 1,
            "id": id,
            "title": format!("title {}", cases[0].1),
            "kind": "bug",
            "summary": format!("summary {}", cases[1].0),
            "tags_suggested": ["ok"],
            "based_on": "sha256:0",
            "model": "claude-sonnet-4-5",
            "at": "2026-11-19T09:30:00+07:00"
        }),
    );
    let store = sb.store();
    let mut raws = vec![];
    for cmd in ["accept_ai", "trash", "restore"] {
        raws.push(handle(&store, &json!({"cmd": cmd, "id": id}).to_string()));
    }
    let accepted: Value = serde_json::from_str(&raws[0]).unwrap();
    assert!(accepted["ai"].is_object(), "{accepted}");
    for raw in &raws {
        for (_, secret) in &cases {
            assert!(!raw.contains(secret.as_str()), "secret in {raw}");
        }
    }
}

// ---------------------------------------------------------------------------------------------
// HUN-279 frontend slice: the page calls only commands `handle` knows, and shows the actions
// ---------------------------------------------------------------------------------------------

#[test]
fn the_page_asks_only_for_commands_the_api_answers() {
    let ipc_js = std::str::from_utf8(asset("/ipc.js").unwrap().1).unwrap();
    let asked = regex::Regex::new(r"request\('([a-z_]+)'").unwrap();
    let cmds: std::collections::BTreeSet<&str> = asked
        .captures_iter(ipc_js)
        .map(|c| c.get(1).unwrap().as_str())
        .collect();
    let all = [
        "board",
        "detail",
        "accept_ai",
        "trash",
        "restore",
        "open_in_editor",
    ];
    assert_eq!(cmds, all.into_iter().collect(), "commands in ipc.js");
    let sb = Sandbox::new("page-cmds");
    let store = sb.store();
    for cmd in &cmds {
        let out = call(&store, &json!({"cmd": cmd}));
        assert_ne!(out["error"], "unknown command", "{cmd}: {out}");
        // The contract comment at the top of ipc.js names each request and its result.
        assert!(
            ipc_js.contains(&format!(r#""cmd": "{cmd}""#)),
            "ipc.js contract comment lacks {cmd}"
        );
    }
}

#[test]
fn the_page_has_the_design_actions_and_the_undo_window() {
    let app = std::str::from_utf8(asset("/app.js").unwrap().1).unwrap();
    // Labels from docs/design/Main.dc.html, board "Dashboard (clickable)", and the AC 5 toast.
    for label in [
        "'Add as tags'",
        "'Open in editor'",
        "'Copy id'",
        "'Move to trash'",
        "'Copied'",
    ] {
        assert!(app.contains(label), "app.js lacks {label}");
    }
    let html = std::str::from_utf8(asset("/").unwrap().1).unwrap();
    assert!(html.contains(r#"id="toast-undo" hidden>Undo</button>"#));
    // AC 5: Copy id uses the page clipboard API, not IPC. AC 2/3: Undo is offered for 6 s.
    assert!(app.contains("navigator.clipboard.writeText("));
    assert!(app.contains("var UNDO_MS = 6000;"));
    // AC 7: each IPC action asks for the board again.
    for call in [
        "easyNoteIpc.acceptAi(",
        "easyNoteIpc.trash(",
        "easyNoteIpc.restore(",
        "easyNoteIpc.openInEditor(",
    ] {
        let at = app
            .find(call)
            .unwrap_or_else(|| panic!("app.js lacks {call}"));
        let rest = &app[at..];
        let end = rest.find("\n  }\n").unwrap_or(rest.len());
        assert!(
            rest[..end].contains("afterAction"),
            "{call} is not followed by afterAction (board reload)"
        );
    }
    assert!(app.contains("function afterAction("));
}

// ---------------------------------------------------------------------------------------------
// HUN-280: capture from the window (`context`, `add`; FR-044, FR-027, FR-001, FR-014, NFR-002)
// ---------------------------------------------------------------------------------------------

/// One note file of any `source`, written by hand; `extra` holds front matter lines.
fn sourced(sb: &Sandbox, id: &str, created: &str, source: &str, extra: &str) {
    let folder = sb.notes().join(&id[..4]).join(&id[4..6]);
    std::fs::create_dir_all(&folder).unwrap();
    let text = format!(
        "---\neasy-note: 1\nid: \"{id}\"\ncreated: \"{created}\"\n{extra}tags: []\nsource: \"{source}\"\n---\nx\n"
    );
    std::fs::write(folder.join(format!("{id}.md")), text).unwrap();
}

/// A `cli` note 30 min before the frozen now, in repo `r` on task `T1`, with remote and branch.
fn last_cli(sb: &Sandbox) {
    sourced(
        sb,
        "20261119-093000-c1c1",
        "2026-11-19T09:30:00+07:00",
        "cli",
        "repo: \"r\"\nremote: \"github.com/o/r\"\nbranch: \"fix/t1-x\"\ntask: \"T1\"\ncommit: \"abc1234\"\n",
    );
}

/// The note file `add` wrote for `id`.
fn written(sb: &Sandbox, id: &str) -> String {
    std::fs::read_to_string(
        sb.notes()
            .join(format!("{}/{}/{id}.md", &id[..4], &id[4..6])),
    )
    .unwrap()
}

#[test]
fn context_is_the_newest_cli_or_hook_note_under_two_hours() {
    let sb = Sandbox::new("ctx");
    let store = sb.store();
    assert_eq!(
        call(&store, &json!({"cmd": "context"})),
        json!({"context": null})
    );
    last_cli(&sb);
    // A newer note of another source is not a context source (FR-027).
    sourced(
        &sb,
        "20261119-094500-q1q1",
        "2026-11-19T09:45:00+07:00",
        "quick",
        "repo: \"other\"\n",
    );
    assert_eq!(
        call(&store, &json!({"cmd": "context"})),
        json!({"context": {"repo": "r", "task": "T1"}})
    );
    // A hook note is a source too, and the newest one wins.
    sourced(
        &sb,
        "20261119-095000-h1h1",
        "2026-11-19T09:50:00+07:00",
        "hook",
        "repo: \"h\"\n",
    );
    assert_eq!(
        call(&store, &json!({"cmd": "context"})),
        json!({"context": {"repo": "h", "task": null}})
    );
}

#[test]
fn context_is_null_when_the_last_note_is_two_hours_old() {
    let sb = Sandbox::new("ctx-old");
    sourced(
        &sb,
        "20261119-080000-o1o1",
        "2026-11-19T08:00:00+07:00",
        "cli",
        "repo: \"r\"\ntask: \"T1\"\n",
    );
    assert_eq!(
        call(&sb.store(), &json!({"cmd": "context"})),
        json!({"context": null})
    );
}

#[test]
fn add_keeps_the_last_context_like_quick_and_returns_the_new_item() {
    let sb = Sandbox::new("add-ctx");
    sb.four();
    last_cli(&sb);
    let store = sb.store();
    let out = call(
        &store,
        &json!({"cmd": "add", "text": "  retry on firefox #flaky\nsecond line  ", "keep_context": true}),
    );
    let id = out["id"].as_str().unwrap_or_else(|| panic!("{out}"));
    assert!(id.starts_with("20261119-100000-"), "{out}");
    assert_eq!(out["source"], "quick");
    assert_eq!(out["repo"], "r");
    assert_eq!(out["task"], "T1");
    assert_eq!(out["tags"], json!(["flaky"]));
    assert_eq!(out["day"], "today");
    // Trimmed on save; the store ends the body with one line end, as for every note.
    assert_eq!(out["text"], "retry on firefox #flaky\nsecond line\n");
    assert_eq!(out["masked"], false);
    let file = written(&sb, id);
    for line in [
        "easy-note: 1\n",
        "repo: \"r\"\n",
        "remote: \"github.com/o/r\"\n",
        "branch: \"fix/t1-x\"\n",
        "task: \"T1\"\n",
        "source: \"quick\"\n",
    ] {
        assert!(file.contains(line), "{line:?} not in {file}");
    }
    // A commit belongs to the moment of the old note, not to this one (as in `quick`).
    assert!(!file.contains("commit:"), "{file}");
    // The new note is the first one on the board, in `today`.
    let board = call(&store, &json!({"cmd": "board"}));
    assert_eq!(ids(&board)[0], id);
    assert_eq!(board["notes"][0]["day"], "today");
}

#[test]
fn add_without_keep_context_writes_no_context() {
    let sb = Sandbox::new("add-bare");
    last_cli(&sb);
    let store = sb.store();
    for request in [
        json!({"cmd": "add", "text": "buy a hub", "keep_context": false}),
        json!({"cmd": "add", "text": "buy a hub"}),
    ] {
        let out = call(&store, &request);
        let id = out["id"].as_str().unwrap_or_else(|| panic!("{out}"));
        assert_eq!(out["repo"], Value::Null, "{out}");
        assert_eq!(out["task"], Value::Null, "{out}");
        assert_eq!(out["source"], "quick");
        let file = written(&sb, id);
        for key in ["repo:", "remote:", "branch:", "task:", "commit:"] {
            assert!(!file.contains(key), "{key} in {file}");
        }
    }
}

#[test]
fn add_with_keep_context_and_no_last_context_saves_the_note() {
    let sb = Sandbox::new("add-none");
    let out = call(
        &sb.store(),
        &json!({"cmd": "add", "text": "first note ever", "keep_context": true}),
    );
    let id = out["id"].as_str().unwrap_or_else(|| panic!("{out}"));
    assert_eq!(out["repo"], Value::Null);
    assert!(written(&sb, id).contains("first note ever"));
}

#[test]
fn add_of_empty_text_is_an_error_and_writes_nothing() {
    let sb = Sandbox::new("add-empty");
    last_cli(&sb);
    let store = sb.store();
    let before = snapshot(&sb.notes());
    for text in ["", "   ", "\n\t \n"] {
        let out = call(
            &store,
            &json!({"cmd": "add", "text": text, "keep_context": true}),
        );
        assert_eq!(out, json!({"error": "empty"}), "{text:?}");
    }
    for request in [
        json!({"cmd": "add"}),
        json!({"cmd": "add", "text": 5}),
        json!({"cmd": "add", "text": null}),
    ] {
        assert_eq!(
            call(&store, &request),
            json!({"error": "missing text"}),
            "{request}"
        );
    }
    assert_eq!(snapshot(&sb.notes()), before, "no change on disk");
}

#[test]
fn add_masks_a_planted_secret_on_write() {
    let sb = Sandbox::new("add-leak");
    let store = sb.store();
    for (text, secret) in planted() {
        let raw = handle(
            &store,
            &json!({"cmd": "add", "text": format!("note {text}"), "keep_context": false})
                .to_string(),
        );
        assert!(!raw.contains(secret.as_str()), "secret in {raw}");
        let out: Value = serde_json::from_str(&raw).unwrap();
        let id = out["id"].as_str().unwrap_or_else(|| panic!("{raw}"));
        assert_eq!(out["masked"], true, "{raw}");
        let file = written(&sb, id);
        assert!(!file.contains(secret.as_str()), "secret in the file {id}");
        assert!(file.contains("[masked:"), "{file}");
    }
}
