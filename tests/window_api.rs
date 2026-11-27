//! HUN-278: the dashboard window's pure parts (FR-042, FR-034, NFR-001, NFR-002, DDR 0009).
//!
//! `window::api::handle` answers the page's IPC requests over a `Store`; it needs no window and
//! no display, so every test here runs in the default build. Each test uses its own temp notes
//! dir and a store with a frozen clock and zone (2026-11-19 10:00 at +07:00). The asset scan
//! reads `src/surfaces/window/assets/`. No literal whole secret token lives here: the values come
//! from `tests/fixtures/secrets/planted.json`, split in parts.

use std::path::{Path, PathBuf};

use easy_note_mcp::core::ai;
use easy_note_mcp::core::config::{after_save_on, AiSettings};
use easy_note_mcp::core::context::Context;
use easy_note_mcp::core::store::Store;
use easy_note_mcp::surfaces::dashboard::use_window;
use easy_note_mcp::surfaces::enrich;
use easy_note_mcp::surfaces::window::{
    allowed_url,
    api::{handle, handle_with},
    asset, capture_from, capture_ipc, dashboard_sizes, initial_note_script, ipc, ipc_from,
    ipc_from_with, ipc_threaded, ipc_with, reply_script, show_script, single, Capture, Running,
    CSP, ORIGIN, QUICK_START, REFRESH,
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
    // HUN-282: the board no longer carries the full body; `detail` does (below).
    assert!(first.get("text").is_none(), "{first}");
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
    // HUN-284 AC 1: no 'unsafe-inline' (it was `style-src easy-note: 'unsafe-inline'`); the
    // header and the page's own <meta> policy are the same string.
    assert_eq!(CSP, "default-src easy-note:");
    assert!(!CSP.contains("unsafe"), "{CSP}");
    let (mime, body) = asset("/").expect("the page");
    assert!(
        std::str::from_utf8(body).unwrap().contains(&format!(
            r#"http-equiv="Content-Security-Policy" content="{CSP}""#
        )),
        "index.html <meta> CSP differs from the header CSP"
    );
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
    assert!(allowed_url("easy-note://localhost"));
    // HUN-284 AC 3: only the exact origin `easy-note://localhost`, not the whole scheme.
    for url in [
        "easy-note://other/",
        "easy-note://localhost.evil/",
        "easy-note://localhost.evil",
        "easy-note://localhost:8080/",
        "easy-note://localhost@evil/",
        "easy-note://evil/localhost/",
        "easy-note:///index.html",
        "easy-note://",
        "https://x/",
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
    let inline_style = regex::Regex::new(
        r#"(?i)\sstyle\s*=\s*["']|<style\b|setAttribute\(\s*['"]style|\.cssText\b|\.style\s*=[^=]|<script>"#,
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
        // HUN-284 AC 1: nothing the CSP `default-src easy-note:` blocks as inline style. A CSSOM
        // property write (`el.style.width = ...`) is not blocked and stays allowed.
        assert!(
            inline_style.find(&text).is_none(),
            "inline style in {rel}: {:?}",
            inline_style.find(&text)
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
        page.contains("window.__easyNote.refresh = refreshBoard;"),
        "app.js must set window.__easyNote.refresh to the focus refresh (HUN-282: it joins the start-up load)"
    );
    let html = std::str::from_utf8(asset("/").unwrap().1).unwrap();
    let ipc_at = html.find(r#"src="ipc.js""#).expect("ipc.js script");
    let app_at = html.find(r#"src="app.js""#).expect("app.js script");
    assert!(
        ipc_at < app_at,
        "ipc.js defines window.__easyNote before app.js"
    );
}

#[test]
fn ipc_is_answered_only_from_the_app_origin() {
    // HUN-284 AC 4: wry passes the source page URL as the IPC request URI (WebKitGTK: the main
    // frame URL); a message from any other origin gets no answer and reaches no command.
    let sb = Sandbox::new("ipc-origin");
    sb.four();
    let store = sb.store();
    let body = r#"{"req": 1, "cmd": "board"}"#;
    for source in ["easy-note://localhost/", "easy-note://localhost/index.html"] {
        let out: Value =
            serde_json::from_str(&ipc_from(&store, source, body).expect(source)).unwrap();
        assert_eq!(out["total"], 4, "{source}");
    }
    let before = snapshot(&sb.notes());
    for source in [
        "easy-note://localhost.evil/",
        "easy-note://other/",
        "https://x/",
        "about:blank",
        "",
    ] {
        assert_eq!(ipc_from(&store, source, body), None, "{source}");
        let trash = r#"{"req": 2, "cmd": "trash", "id": "20261112-080000-cccc"}"#;
        assert_eq!(ipc_from(&store, source, trash), None, "{source}");
    }
    assert_eq!(
        snapshot(&sb.notes()),
        before,
        "a refused source changed the notes dir"
    );
}

#[test]
fn a_row_click_brings_the_detail_into_view_only_when_it_wraps() {
    // HUN-284 AC 5: at 800 px the detail panel sits below the list; a row click scrolls its top
    // into view. Two columns (1280 px): the panel is beside the list, nothing scrolls.
    let app = std::str::from_utf8(asset("/app.js").unwrap().1).unwrap();
    assert!(
        app.contains("b.addEventListener('click', function () { select(n.id, true); });"),
        "the row click does not ask select to reveal the panel"
    );
    // After the detail (or its error) is rendered, not before: the panel's height is known then.
    let at = app
        .find("function select(id, reveal)")
        .expect("select(id, reveal)");
    let select = &app[at..at + app[at..].find("\n  }\n").expect("end of select")];
    assert_eq!(
        select
            .matches("renderDetail();\n      if (reveal) revealDetail();")
            .count(),
        2,
        "select reveals after both the detail and the error render"
    );
    let at = app.find("function revealDetail(").expect("revealDetail");
    let body = &app[at..at + app[at..].find("\n  }\n").expect("end of revealDetail")];
    for part in [
        "offsetTop",
        "scrollIntoView(",
        "block: 'start'",
        "(prefers-reduced-motion: reduce)",
    ] {
        assert!(body.contains(part), "revealDetail lacks {part}");
    }
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
    // A trashed note is refused by accept_ai; a second trash too (edit: `edit_errors_write_nothing`).
    for cmd in ["accept_ai", "trash", "detail"] {
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
    for cmd in ["accept_ai", "trash", "restore", "edit"] {
        for (id, want) in [
            (json!(null), "missing id"),
            (json!(7), "missing id"),
            (json!("../../etc/passwd"), "bad id"),
            (json!("2026/11/20261119-090000-aaaa"), "bad id"),
            (json!("20261119-090000-ffff"), "no such note"),
        ] {
            let mut request =
                json!({"cmd": cmd, "text": "x", "tags": [], "base": "x", "base_tags": []});
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
        "edit",
        "context",
        "add",
        "enrich",
        "close",
        "settings_get",
        "settings_save",
    ];
    assert_eq!(cmds, all.into_iter().collect(), "commands in ipc.js");
    // `settings_get` reads config.toml: a temp HOME.
    let env = AiEnv::ok("page-cmds");
    let store = env.sb.store();
    for cmd in &cmds {
        // HUN-289: `close` is the quick window's own command (`capture_ipc`), not the dashboard's.
        if *cmd == "close" {
            assert_eq!(
                capture_ipc(r#"{"cmd": "close"}"#, &Context::default()),
                Capture::Close
            );
        } else {
            let out = call(&store, &json!({"cmd": cmd}));
            assert_ne!(out["error"], "unknown command", "{cmd}: {out}");
        }
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
    for label in ["'Add as tags'", "'Copy id'", "'Move to trash'", "'Copied'"] {
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
// HUN-282: the board's payload, the page's single start-up load and its debounced search
// ---------------------------------------------------------------------------------------------

#[test]
fn board_items_carry_a_search_field_not_the_full_text() {
    let sb = Sandbox::new("payload");
    sb.four();
    let store = sb.store();
    let board = call(&store, &json!({"cmd": "board"}));
    for n in board["notes"].as_array().unwrap() {
        assert!(
            n.get("text").is_none(),
            "the board carries no full body: {n}"
        );
        assert!(n["search"].is_string(), "{n}");
    }
    // Search still finds every word of the body: the field holds each distinct word once.
    let first = &board["notes"][0];
    let words: std::collections::BTreeSet<&str> =
        first["search"].as_str().unwrap().split(' ').collect();
    for w in [
        "Retry",
        "bug",
        "only",
        "on",
        "Firefox",
        "ui_wait_for",
        "times",
        "out.",
    ] {
        assert!(words.contains(w), "{w} missing from {first}");
    }
    // The detail panel still gets the whole text.
    let id = first["id"].as_str().unwrap();
    let d = call(&store, &json!({"cmd": "detail", "id": id}));
    assert_eq!(
        d["text"],
        "Retry bug only on Firefox\n\nui_wait_for times out.\n"
    );
}

#[test]
fn a_long_repetitive_body_gives_a_smaller_board_reply() {
    let sb = Sandbox::new("payload-size");
    let body = "the quick brown fox jumps over the lazy dog\n".repeat(200);
    sb.note(
        "20261119-090000-0001",
        "2026-11-19T09:00:00+07:00",
        "tags: []\n",
        &body,
    );
    let board = handle(&sb.store(), r#"{"cmd": "board"}"#);
    assert!(
        board.len() < body.len() / 4,
        "{} bytes for a {} byte body",
        board.len(),
        body.len()
    );
}

#[test]
fn the_search_field_is_masked() {
    let sb = Sandbox::new("payload-mask");
    let cases = planted();
    let body: Vec<&str> = cases.iter().map(|(t, _)| t.as_str()).collect();
    sb.note(
        "20261119-090000-5ec8",
        "2026-11-19T09:00:00+07:00",
        "tags: []\n",
        &body.join("\n"),
    );
    let raw = handle(&sb.store(), r#"{"cmd": "board"}"#);
    let board: Value = serde_json::from_str(&raw).unwrap();
    assert!(board["notes"][0]["search"]
        .as_str()
        .unwrap()
        .contains("[masked:"));
    for (_, secret) in &cases {
        assert!(!raw.contains(secret.as_str()), "secret in {raw}");
    }
}

/// The page contract, run for real: `ipc.js` and `app.js` in node over a fake DOM and a stub
/// `window.ipc` that counts requests (`tests/fixtures/window/page_harness.js`). Skipped, with a
/// printed note, when `node` is not installed.
fn run_page(board: &Value) -> Option<Value> {
    run_page_with(board, None)
}

/// As `run_page`; with `details` (note id -> its `detail` answer) the HUN-286 edit scenarios run too.
fn run_page_with(board: &Value, details: Option<&Value>) -> Option<Value> {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("board.json");
    std::fs::write(&file, board.to_string()).unwrap();
    let harness =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/window/page_harness.js");
    let mut command = std::process::Command::new("node");
    command.arg(harness).arg(assets_dir()).arg(&file);
    if let Some(details) = details {
        let more = dir.path().join("details.json");
        std::fs::write(&more, details.to_string()).unwrap();
        command.arg(more);
    }
    let out = match command.output() {
        Ok(out) => out,
        Err(e) => {
            eprintln!("skipped: no node to run the page ({e})");
            return None;
        }
    };
    assert!(
        out.status.success(),
        "page harness failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Some(serde_json::from_slice(&out.stdout).unwrap())
}

#[test]
fn the_page_asks_for_the_board_once_when_the_window_opens() {
    let sb = Sandbox::new("page-once");
    sb.four();
    let board = call(&sb.store(), &json!({"cmd": "board"}));
    let Some(run) = run_page(&board) else { return };
    // AC 1: the shell's focus refresh and the page's own start-up load are one request ...
    assert_eq!(run["focus_before_answer"]["boards"], 1, "{run}");
    assert_eq!(run["focus_after_answer"]["boards"], 1, "{run}");
    // ... and a later focus still refreshes.
    assert_eq!(run["refresh_much_later"]["boards"], 2, "{run}");
}

#[test]
fn typing_in_search_renders_once_and_shows_the_same_rows() {
    let sb = Sandbox::new("page-search");
    sb.four();
    let board = call(&sb.store(), &json!({"cmd": "board"}));
    let Some(run) = run_page(&board) else { return };
    let s = &run["search_typing"];
    // AC 2: ten key presses in a row are one render of the one matching row.
    assert_eq!(s["rebuilt"], 1, "{s}");
    assert_eq!(
        s["typed"]["rows"],
        json!(["Retry bug only on Firefox"]),
        "{s}"
    );
    assert_eq!(
        s["bugs"]["rows"],
        json!(["Retry bug only on Firefox"]),
        "{s}"
    );
    assert_eq!(s["cleared"]["rows"].as_array().unwrap().len(), 4, "{s}");
    assert_eq!(
        s["all"]["filters"],
        json!(["4", "1", "0", "0", "1", "2", "1"]),
        "{s}"
    );
    // A word that is only in the body still matches (the search field), a missing one does not.
    let b = &run["search_body_word"];
    assert_eq!(
        b["body"]["rows"],
        json!(["Retry bug only on Firefox"]),
        "{b}"
    );
    assert_eq!(b["none"]["rows"], json!([]), "{b}");
    assert_eq!(
        b["none"]["noneText"], "No note matches “zzzz-nothing”.",
        "{b}"
    );
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

// ---------------------------------------------------------------------------------------------
// HUN-280 frontend slice: the capture dialog of docs/design/Main.dc.html in the page
// ---------------------------------------------------------------------------------------------

/// The body of the named `function <name>(` in app.js, up to the next two-space `}` line.
fn js_function<'a>(app: &'a str, name: &str) -> &'a str {
    let at = app
        .find(&format!("function {name}("))
        .unwrap_or_else(|| panic!("app.js lacks function {name}"));
    let rest = &app[at..];
    &rest[..rest.find("\n  }\n").unwrap_or(rest.len())]
}

#[test]
fn the_page_has_the_design_capture_dialog() {
    let html = std::str::from_utf8(asset("/").unwrap().1).unwrap();
    // Labels and texts from the board "Dashboard (clickable)": header button, dialog, key hints.
    for must in [
        r#"id="new-note">+ New note</button>"#,
        r#"role="dialog" aria-modal="true" aria-label="New note""#,
        r#"id="cap-chip-text""#,
        r#"aria-label="Drop context""#,
        "from your last note",
        r#"placeholder="Note…" aria-label="Note text""#,
        "<kbd>Enter</kbd> save",
        "<kbd>Shift+Enter</kbd> new line",
        "<kbd>Esc</kbd> cancel",
        "<kbd>Backspace</kbd> on empty: drop context",
    ] {
        assert!(html.contains(must), "index.html lacks {must}");
    }
    // The dialog starts closed.
    assert!(
        html.contains(r#"id="cap" hidden"#),
        "dialog not hidden at start"
    );
    let css = std::str::from_utf8(asset("/app.css").unwrap().1).unwrap();
    // Dialog values copied from the design.
    for must in [
        "width: 560px; max-width: calc(100% - 32px)",
        "padding-top: 14vh",
        "background: rgba(8, 10, 9, 0.6)",
        "font: 400 18px/1.5 'IBM Plex Sans', sans-serif",
    ] {
        assert!(css.contains(must), "app.css lacks {must}");
    }
}

#[test]
fn the_capture_keys_and_save_follow_the_guide() {
    // HUN-289: the dialog code is capture.js, shared by the dashboard and the quick window.
    let app = std::str::from_utf8(asset("/app.js").unwrap().1).unwrap();
    let cap = std::str::from_utf8(asset("/capture.js").unwrap().1).unwrap();
    let keys = js_function(cap, "capKey");
    // AC 3: Enter saves, Shift+Enter is a new line, Esc closes, Backspace on empty drops the chip.
    // IME composition: Enter while composing does not save.
    for must in [
        "e.isComposing",
        "e.key === 'Enter' && !e.shiftKey",
        "e.key === 'Escape'",
        "e.key === 'Backspace' && box.value === '' && keepContext",
    ] {
        assert!(keys.contains(must), "capKey lacks {must}");
    }
    let save = js_function(cap, "saveCapture");
    // AC 4: empty or whitespace-only text sends no request.
    let guard = save.find("if (!text.trim()) {").expect("no empty guard");
    let ask = save
        .find("easyNoteIpc.add(text, keepContext)")
        .expect("no add request");
    assert!(guard < ask, "the empty guard comes after the request");
    // AC 5: close, then the page's own reaction; an error keeps the dialog.
    for must in [
        "closeCapture()",
        "call('onSaved', d)",
        "call('onError', err.message)",
    ] {
        assert!(save.contains(must), "saveCapture lacks {must}");
    }
    // The dashboard's reaction: saved toast, board reload, select the new note.
    for must in [
        "'✓ noted'",
        "loadBoard()",
        "select(d.id)",
        "'could not save: ' + message",
        "window.easyNoteCapture.attach(",
    ] {
        assert!(app.contains(must), "app.js lacks {must}");
    }
    let open = js_function(cap, "openCapture");
    assert!(open.contains("easyNoteIpc.context()"), "{open}");
    assert!(open.contains(".focus()"), "{open}");
    // A click on the backdrop keeps focus in the text box, so the keys above still reach capKey.
    assert!(cap.contains("$('cap').addEventListener('mousedown'"));
    // The focus refresh rebuilds the list only; it never touches the dialog.
    assert!(!js_function(app, "loadBoard").contains("Capture"));
}

// ---------------------------------------------------------------------------------------------
// HUN-280 follow-up: a window `add` starts the AI enrich step like `easy-note add` (owner, 2026-11-20)
// The hook is the seam: the shell passes `enrich::after_save` when `[ai] enrich` and `after_save`
// are on. A test cannot call that itself: it would start the test binary as `ai enrich <id>`.
// ---------------------------------------------------------------------------------------------

/// Run one request with a recording hook; returns the reply and the ids the hook got.
fn with_hook(store: &Store, request: &Value) -> (Value, Vec<String>) {
    let seen = std::cell::RefCell::new(vec![]);
    let out = handle_with(store, &request.to_string(), &|id| {
        seen.borrow_mut().push(id.to_string())
    });
    (serde_json::from_str(&out).unwrap(), seen.into_inner())
}

#[test]
fn add_hands_the_new_id_to_the_hook_once() {
    let sb = Sandbox::new("hook-on");
    let store = sb.store();
    let (reply, seen) = with_hook(&store, &json!({"cmd": "add", "text": "buy a hub"}));
    assert_eq!(seen, vec![reply["id"].as_str().unwrap().to_string()]);
    assert_eq!(written(&sb, &seen[0]).matches("buy a hub").count(), 1);
}

#[test]
fn a_failed_add_and_every_other_command_start_nothing() {
    let sb = Sandbox::new("hook-none");
    let store = sb.store();
    last_cli(&sb);
    for request in [
        json!({"cmd": "add", "text": "   "}),
        json!({"cmd": "add"}),
        json!({"cmd": "board"}),
        json!({"cmd": "context"}),
        json!({"cmd": "nope"}),
    ] {
        let (_, seen) = with_hook(&store, &request);
        assert!(seen.is_empty(), "{request}: {seen:?}");
    }
}

#[test]
fn without_a_hook_add_saves_the_same_reply() {
    // `handle` and `ipc` are the flag-off path: the same note, no hook to call.
    let sb = Sandbox::new("hook-off");
    let store = sb.store();
    let plain = call(&store, &json!({"cmd": "add", "text": "plain"}));
    assert!(plain["id"].is_string());
    let via_ipc: Value =
        serde_json::from_str(&ipc(&store, r#"{"req": 1, "cmd": "add", "text": "again"}"#)).unwrap();
    assert_eq!(via_ipc["req"], 1);
    assert!(via_ipc["id"].is_string());
}

#[test]
fn ipc_hands_the_new_id_to_the_hook_and_keeps_the_req() {
    let sb = Sandbox::new("hook-ipc");
    let store = sb.store();
    let seen = std::cell::RefCell::new(vec![]);
    let out = ipc_with(
        &store,
        r#"{"req": 5, "cmd": "add", "text": "via ipc"}"#,
        &|id| seen.borrow_mut().push(id.to_string()),
    );
    let reply: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(reply["req"], 5);
    assert_eq!(seen.into_inner(), vec![reply["id"].as_str().unwrap()]);
}

/// The shell's path (merge of HUN-280 with HUN-284): the origin check runs first, so a page
/// outside `easy-note://localhost` gets no answer, saves nothing and starts no hook.
#[test]
fn ipc_from_with_runs_the_hook_only_for_the_apps_own_page() {
    let sb = Sandbox::new("hook-origin");
    let store = sb.store();
    let body = r#"{"req": 7, "cmd": "add", "text": "from a page"}"#;
    let seen = std::cell::RefCell::new(vec![]);
    let hook = |id: &str| seen.borrow_mut().push(id.to_string());
    assert_eq!(
        ipc_from_with(&store, "https://evil.example/", body, &hook),
        None
    );
    assert!(seen.borrow().is_empty());
    let out = ipc_from_with(&store, "easy-note://localhost/", body, &hook).expect("own page");
    let reply: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(reply["req"], 7);
    assert_eq!(seen.into_inner(), vec![reply["id"].as_str().unwrap()]);
}

// ---------------------------------------------------------------------------------------------
// HUN-286: `edit` saves the text and tags through `Store::update`, as `easy-note edit` does
// ---------------------------------------------------------------------------------------------

/// The `edit` request for `id` with the `detail` text as the base.
fn edit_request(store: &Store, id: &str, text: &str, tags: Value) -> Value {
    let shown = call(store, &json!({"cmd": "detail", "id": id}));
    json!({"cmd": "edit", "id": id, "text": text, "tags": tags, "base": shown["text"], "base_tags": shown["tags"]})
}

/// `acted`, with its stored tags replaced by `tags` (a hand edit in another app).
fn acted_with_tags(sb: &Sandbox, tags: &str) -> PathBuf {
    let path = acted(sb);
    let file = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        file.replace("tags: [\"y\"]", &format!("tags: [{tags}]")),
    )
    .unwrap();
    path
}

#[test]
fn edit_keeps_existing_tags_the_page_sends_with_a_hash() {
    // The page sends every tag as `#tag` (app.js `startEdit`); a kept tag stays as stored, as in
    // `easy-note edit`, even when `easy-note tag` would refuse or lowercase it.
    let sb = Sandbox::new("edit-hash-tags");
    let path = acted_with_tags(&sb, "\"café\", \"Project\"");
    let store = sb.store();
    let request = edit_request(&store, ACTED, "new text", json!(["#café", "#Project"]));
    let out = call(&store, &request);
    assert_eq!(out["tags"], json!(["café", "Project"]), "{out}");
    let file = std::fs::read_to_string(&path).unwrap();
    assert!(file.contains("tags: [\"café\", \"Project\"]"), "{file}");
}

#[test]
fn edit_refuses_when_only_the_tags_changed_outside_the_window() {
    // AC 2 for the tags: a tag change made in another app is not overwritten without a word.
    let sb = Sandbox::new("edit-stale-tags");
    let path = acted(&sb);
    let store = sb.store();
    let request = edit_request(&store, ACTED, "my draft", json!(["#y"]));
    let file = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        file.replace("tags: [\"y\"]", "tags: [\"y\", \"obsidian\"]"),
    )
    .unwrap();
    let before = snapshot(&sb.notes());
    let out = call(&store, &request);
    assert_eq!(out, json!({"error": "the note changed outside the window"}));
    assert_eq!(snapshot(&sb.notes()), before, "byte for byte");
}

#[test]
fn edit_saves_the_text_and_tags_and_keeps_the_rest() {
    // AC 1, Success Criterion 1.
    let sb = Sandbox::new("edit");
    let path = acted(&sb);
    let store = sb.store();
    let request = edit_request(
        &store,
        ACTED,
        "Retry bug only on Chrome #Web",
        json!(["#Z", "y", "z"]),
    );
    let out = call(&store, &request);
    assert_eq!(out["id"], ACTED, "the updated detail: {out}");
    assert_eq!(out["text"], "Retry bug only on Chrome #Web\n", "{out}");
    // `easy-note tag` rule: no `#`, each once; the text's `#words` join as in `easy-note edit`.
    assert_eq!(out["tags"], json!(["z", "y", "web"]), "{out}");
    assert_eq!(out["masked"], false, "{out}");
    assert_eq!(out["ai"]["stale"], true, "the text changed: {out}");
    let file = std::fs::read_to_string(&path).unwrap();
    assert_eq!(body_of(&file), "Retry bug only on Chrome #Web\n");
    for kept in [
        "created: \"2026-11-19T09:00:00+07:00\"",
        "repo: \"easy-ui-mcp\"",
        "source: \"cli\"",
        "tags: [\"z\", \"y\", \"web\"]",
        "updated: \"2026-11-19T10:00:00+07:00\"",
    ] {
        assert!(file.contains(kept), "{kept} in {file}");
    }
}

#[test]
fn edit_of_the_tags_only_keeps_the_text_bytes() {
    let sb = Sandbox::new("edit-tags");
    let path = acted(&sb);
    let store = sb.store();
    let before = std::fs::read_to_string(&path).unwrap();
    // The page may drop the last newline of the text it loaded: still the same text.
    let mut request = edit_request(&store, ACTED, "", json!(["y", "new"]));
    let text = request["base"]
        .as_str()
        .unwrap()
        .trim_end_matches('\n')
        .to_string();
    request["text"] = json!(text);
    request["base"] = json!(text);
    let out = call(&store, &request);
    assert_eq!(out["tags"], json!(["y", "new"]), "{out}");
    assert_eq!(out["ai"]["stale"], false, "{out}");
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(body_of(&after), body_of(&before), "the text is unchanged");
}

#[test]
fn edit_with_nothing_changed_writes_nothing() {
    let sb = Sandbox::new("edit-same");
    acted(&sb);
    let store = sb.store();
    let base = call(&store, &json!({"cmd": "detail", "id": ACTED}))["text"].clone();
    let before = snapshot(&sb.notes());
    let out = call(
        &store,
        &json!({"cmd": "edit", "id": ACTED, "text": base, "tags": ["y"], "base": base, "base_tags": ["y"]}),
    );
    assert_eq!(out["id"], ACTED, "{out}");
    assert_eq!(
        snapshot(&sb.notes()),
        before,
        "no `updated` stamp, no write"
    );
}

#[test]
fn edit_refuses_when_the_note_changed_outside_the_window() {
    // AC 2, Success Criterion 2.
    let sb = Sandbox::new("edit-stale");
    let path = acted(&sb);
    let store = sb.store();
    let request = edit_request(&store, ACTED, "my draft", json!(["y"]));
    let mut file = std::fs::read_to_string(&path).unwrap();
    file.push_str("added in Obsidian\n");
    std::fs::write(&path, &file).unwrap();
    let before = snapshot(&sb.notes());
    let out = call(&store, &request);
    assert_eq!(out, json!({"error": "the note changed outside the window"}));
    assert_eq!(snapshot(&sb.notes()), before, "byte for byte");
}

#[test]
fn edit_masks_a_planted_secret_on_write() {
    // NFR-002, Success Criterion 3.
    let sb = Sandbox::new("edit-leak");
    acted(&sb);
    let store = sb.store();
    let cases = planted();
    let text: Vec<&str> = cases.iter().map(|(t, _)| t.as_str()).collect();
    let request = edit_request(&store, ACTED, &text.join("\n"), json!([]));
    let raw = handle(&store, &request.to_string());
    let out: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(out["masked"], true, "{out}");
    let file = std::fs::read_to_string(sb.notes().join(format!("2026/11/{ACTED}.md"))).unwrap();
    for (_, secret) in &cases {
        assert!(!file.contains(secret.as_str()), "secret in the file");
        assert!(!raw.contains(secret.as_str()), "secret in the reply");
    }
    // Saving the masked text again keeps it masked and changes nothing more.
    let again = edit_request(&store, ACTED, out["text"].as_str().unwrap(), json!([]));
    let before = std::fs::read_to_string(sb.notes().join(format!("2026/11/{ACTED}.md"))).unwrap();
    let out = call(&store, &again);
    assert_eq!(out["masked"], true, "{out}");
    let after = std::fs::read_to_string(sb.notes().join(format!("2026/11/{ACTED}.md"))).unwrap();
    assert_eq!(body_of(&after), body_of(&before));
}

#[test]
fn edit_errors_write_nothing() {
    // AC 3, Success Criterion 4.
    let sb = Sandbox::new("edit-errors");
    sb.four();
    acted(&sb);
    let inbox = sb.notes().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::write(inbox.join("p.md"), "phone note\n").unwrap();
    let store = sb.store();
    let base = call(&store, &json!({"cmd": "detail", "id": ACTED}))["text"].clone();
    let inbox_base = call(&store, &json!({"cmd": "detail", "id": "inbox/p"}))["text"].clone();
    let ok = |id: &str, b: &Value| json!({"cmd": "edit", "id": id, "text": "x", "tags": [], "base": b, "base_tags": if id == ACTED { json!(["y"]) } else { json!([]) }});
    let before = snapshot(&sb.notes());
    let mut cases = vec![];
    for empty in ["", "   ", " \n\t\n"] {
        let mut r = ok(ACTED, &base);
        r["text"] = json!(empty);
        cases.push((r, "empty"));
    }
    cases.push((ok("inbox/p", &inbox_base), "inbox notes are read-only"));
    cases.push((ok("20261119-090000-ffff", &base), "no such note"));
    cases.push((ok("../../etc/passwd", &base), "bad id"));
    let mut bad_tag = ok(ACTED, &base);
    bad_tag["tags"] = json!(["a b"]);
    cases.push((bad_tag, "not a tag"));
    for (key, want) in [
        ("id", "missing id"),
        ("text", "missing text"),
        ("tags", "missing tags"),
        ("base", "missing base"),
        ("base_tags", "missing base"),
    ] {
        let mut r = ok(ACTED, &base);
        r.as_object_mut().unwrap().remove(key);
        cases.push((r, want));
    }
    let mut not_list = ok(ACTED, &base);
    not_list["tags"] = json!("y");
    cases.push((not_list, "missing tags"));
    let mut not_strings = ok(ACTED, &base);
    not_strings["tags"] = json!([1]);
    cases.push((not_strings, "missing tags"));
    for (request, want) in &cases {
        let out = call(&store, request);
        assert_eq!(out, json!({"error": want}), "{request}");
    }
    assert_eq!(snapshot(&sb.notes()), before, "no change on disk");
    // A trashed note is no note.
    call(&store, &json!({"cmd": "trash", "id": ACTED}));
    let out = call(&store, &ok(ACTED, &base));
    assert_eq!(out, json!({"error": "no such note"}));
}

#[test]
fn the_window_api_runs_no_xdg_open() {
    // AC 9: `open_in_editor` is gone; no window command opens a note in another app.
    let sb = Sandbox::new("no-open");
    acted(&sb);
    let fake = FakeOpener::new(0);
    let out = call(&sb.store(), &json!({"cmd": "open_in_editor", "id": ACTED}));
    assert_eq!(out, json!({"error": "unknown command"}));
    assert!(fake.calls().is_empty());
    let api = include_str!("../src/surfaces/window/api.rs");
    assert!(!api.contains("xdg-open\")"), "api.rs starts no xdg-open");
    for name in ["/ipc.js", "/app.js"] {
        let page = std::str::from_utf8(asset(name).unwrap().1).unwrap();
        for gone in ["open_in_editor", "openInEditor", "Open in editor"] {
            assert!(!page.contains(gone), "{name} still has {gone}");
        }
    }
}

#[test]
fn the_initial_note_goes_to_the_page_as_one_json_string() {
    // AC 7: never string-built into JS; ipc.js keeps the value when it sets `window.__easyNote`.
    let id = "inbox/a\"b'</script>\u{2028}\\";
    let script = initial_note_script(id);
    let json = script
        .strip_prefix("window.__easyNote = ")
        .and_then(|s| s.strip_suffix(';'))
        .unwrap_or_else(|| panic!("{script}"));
    let value: Value = serde_json::from_str(json).unwrap();
    assert_eq!(value, json!({"initialNote": id}));
    let ipc_js = std::str::from_utf8(asset("/ipc.js").unwrap().1).unwrap();
    assert!(ipc_js.contains("typeof prior.initialNote === 'string'"));
    assert!(ipc_js.contains("initialNote: initialNote"));
}

// ---------------------------------------------------------------------------------------------
// HUN-286 frontend slice: Edit mode in the detail panel, the initial note, the page side of
// "the note changed outside the window". The real ipc.js and app.js run in node over a stub IPC
// fed real `handle` answers (`tests/fixtures/window/page_harness.js`).
// ---------------------------------------------------------------------------------------------

/// The board and every note's `detail` of a store with the four sample notes, the AI-enriched
/// `ACTED` note and one inbox note; then the harness run with the edit scenarios.
fn run_edit_page() -> Option<Value> {
    let sb = Sandbox::new("page-edit");
    sb.four();
    acted(&sb);
    let inbox = sb.notes().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::write(inbox.join("p.md"), "phone note\n").unwrap();
    let store = sb.store();
    let board = call(&store, &json!({"cmd": "board"}));
    let other = *ids(&board)
        .iter()
        .find(|id| **id != ACTED && **id != "inbox/p")
        .unwrap();
    let mut notes = serde_json::Map::new();
    for id in ids(&board) {
        notes.insert(
            id.to_string(),
            call(&store, &json!({"cmd": "detail", "id": id})),
        );
    }
    // A note with no AI detail, for the HUN-288 scenarios.
    let plain = *ids(&board)
        .iter()
        .find(|id| notes[**id]["ai"].is_null() && !id.starts_with("inbox/"))
        .unwrap();
    let other = *ids(&board)
        .iter()
        .find(|id| **id != ACTED && **id != "inbox/p" && **id != plain)
        .unwrap_or(&other);
    let more = json!({"roles": {"editable": ACTED, "inbox": "inbox/p", "other": other, "plain": plain},
                      "notes": notes});
    run_page_with(&board, Some(&more))
}

#[test]
fn the_initial_note_is_selected_after_the_first_board_load() {
    let Some(run) = run_edit_page() else { return };
    // AC 7 (page side): the board first, then the note's detail, shown in the panel.
    let r = &run["initial_note"];
    assert_eq!(r["posts"], json!(["board", "detail"]), "{r}");
    assert_eq!(r["detail"], json!([ACTED]), "{r}");
    assert_eq!(r["shown"]["id"], ACTED, "{r}");
    assert_eq!(
        r["current"], "true",
        "the row is marked as the selection: {r}"
    );
    // An unknown id shows the panel's existing text.
    let r = &run["initial_unknown"];
    assert_eq!(
        r["shown"]["text"], "Could not read the note: no such note",
        "{r}"
    );
    // Without `--note` nothing is selected: no detail asked, the panel keeps index.html's
    // "Pick a note to read it." (the fake DOM holds no static HTML, so the panel reads empty).
    let r = &run["initial_none"];
    assert_eq!(r["posts"], json!(["board"]), "{r}");
    assert_eq!(r["shown"]["text"], "", "{r}");
}

#[test]
fn edit_replaces_open_in_editor_and_opens_the_text_and_tags() {
    let Some(run) = run_edit_page() else { return };
    // AC 4: `Edit` in the actions row; edit mode shows the note text and its tags.
    let r = &run["initial_note"]["shown"];
    assert_eq!(
        r["buttons"],
        json!(["Add as tags", "Edit", "Re-run", "Copy id", "Move to trash"]),
        "{r}"
    );
    let r = &run["edit_open"]["shown"];
    assert_eq!(r["editing"], true, "{r}");
    assert_eq!(
        r["draft"], "Retry bug only on Firefox\n\n  keep   these bytes  ",
        "the text without its last line end: {r}"
    );
    assert_eq!(r["tags"], "#y", "{r}");
    assert_eq!(r["buttons"], json!(["Save", "Cancel"]), "{r}");
    assert_eq!(r["focusIsBox"], true, "the caret is in the text box: {r}");
}

#[test]
fn ctrl_enter_saves_through_edit_and_shows_the_new_note() {
    let Some(run) = run_edit_page() else { return };
    let r = &run["edit_save"];
    // AC 4: one `edit` request with the whole text, the tags as words and the loaded text as base.
    assert_eq!(
        r["sent"],
        json!([{"req": r["sent"][0]["req"], "cmd": "edit", "id": ACTED,
                "text": "New text #Web\nsecond line", "tags": ["#y", "new"], "base": r["base"],
                "base_tags": ["y"]}]),
        "{r}"
    );
    assert_eq!(r["shown"]["editing"], false, "{r}");
    assert_eq!(
        r["shown"]["words"],
        json!(["New text #Web\nsecond line\n"]),
        "{r}"
    );
    assert_eq!(r["shown"]["stale"], true, "the AI detail shows stale: {r}");
    assert_eq!(r["shown"]["toast"], "saved", "{r}");
    // The list row updates: the board is asked again after the save.
    assert_eq!(r["boards"], 1, "{r}");
    let r = &run["edit_save_button"];
    assert_eq!(r["sent"], 1, "{r}");
    assert_eq!(r["shown"]["words"], json!(["by the button\n"]), "{r}");
}

#[test]
fn esc_and_cancel_leave_edit_mode_without_writing() {
    let Some(run) = run_edit_page() else { return };
    for (how, r) in [
        ("Esc", &run["edit_cancel"]["esc"]),
        ("Cancel", &run["edit_cancel"]["button"]),
    ] {
        assert_eq!(r["sent"], 0, "{how}: {r}");
        assert_eq!(r["shown"]["editing"], false, "{how}: {r}");
        assert_eq!(
            r["shown"]["words"],
            json!(["Retry bug only on Firefox\n\n  keep   these bytes  \n"]),
            "{how}: the old text: {r}"
        );
        assert_eq!(
            r["shown"]["toast"], "cancelled · nothing saved",
            "{how}: {r}"
        );
    }
    // IME composition: Ctrl+Enter and Esc while composing do nothing.
    let r = &run["edit_composing"];
    assert_eq!(r["sent"], 0, "{r}");
    assert_eq!(r["shown"]["editing"], true, "{r}");
    assert_eq!(r["shown"]["draft"], "typing with an IME", "{r}");
}

#[test]
fn the_focus_refresh_keeps_edit_mode_and_the_draft() {
    let Some(run) = run_edit_page() else { return };
    // AC 6: the board is reloaded, the same text box stays with the draft in it.
    let r = &run["edit_refresh"];
    assert_eq!(r["boards"], 1, "{r}");
    assert_eq!(r["sameBox"], true, "{r}");
    assert_eq!(r["shown"]["editing"], true, "{r}");
    assert_eq!(r["shown"]["draft"], "a draft kept over a refresh", "{r}");
    assert_eq!(r["shown"]["tags"], "#kept", "{r}");
    assert_eq!(r["sent"], 0, "{r}");
}

#[test]
fn a_save_refused_as_changed_outside_reloads_the_note_and_keeps_the_draft() {
    let Some(run) = run_edit_page() else { return };
    // AC 2 (page side): the message, the note asked again, the draft still in the box.
    let r = &run["edit_changed_outside"];
    let refused = &r["refused"]["shown"];
    assert_eq!(refused["editing"], true, "{r}");
    assert_eq!(refused["draft"], "my draft", "{r}");
    assert!(
        refused["error"]
            .as_str()
            .unwrap()
            .contains("the note changed outside the window"),
        "{r}"
    );
    assert_eq!(
        refused["words"],
        json!(["changed in Obsidian\n"]),
        "the text in the file now is shown: {r}"
    );
    assert_eq!(r["refused"]["detailsAfter"], 2, "detail asked again: {r}");
    // A second save sends the reloaded text as its base and goes through.
    assert_eq!(r["second"]["base"], "changed in Obsidian\n", "{r}");
    assert_eq!(r["second"]["text"], "my draft", "{r}");
    assert_eq!(
        r["second"]["base_tags"],
        json!(["y", "obsidian"]),
        "and the reloaded tags: {r}"
    );
    assert_eq!(r["shown"]["editing"], false, "{r}");
    assert_eq!(r["shown"]["words"], json!(["my draft\n"]), "{r}");
}

#[test]
fn a_refused_or_empty_save_keeps_edit_mode() {
    let Some(run) = run_edit_page() else { return };
    let r = &run["edit_error"]["shown"];
    assert_eq!(r["editing"], true, "{r}");
    assert_eq!(r["draft"], "xxxxxxxxxx", "{r}");
    assert_eq!(r["error"], "could not save: the text is too long", "{r}");
    // Empty or white-space text sends no request (the backend answers `empty`).
    let r = &run["edit_empty"];
    assert_eq!(r["sent"], 0, "{r}");
    assert_eq!(r["shown"]["editing"], true, "{r}");
    assert!(
        r["shown"]["error"].as_str().unwrap().contains("empty"),
        "{r}"
    );
}

#[test]
fn another_note_is_not_selected_while_editing() {
    let Some(run) = run_edit_page() else { return };
    // Questions Log 2026-11-21: a row click while editing is blocked; the draft stays.
    let r = &run["edit_switch"];
    let b = &r["blocked"];
    assert_eq!(b["detail"], 0, "no detail asked for the other note: {r}");
    assert_eq!(b["current"], "false", "{r}");
    assert_eq!(b["shown"]["editing"], true, "{r}");
    assert_eq!(b["shown"]["draft"], "a draft I want to keep", "{r}");
    assert_eq!(b["shown"]["toast"], "save or cancel the edit first", "{r}");
    assert_eq!(b["shown"]["focusIsBox"], true, "{r}");
    // After Esc the row click selects the other note.
    assert_eq!(r["after"]["editing"], false, "{r}");
    assert_ne!(r["after"]["id"], ACTED, "{r}");
}

#[test]
fn an_inbox_note_has_no_edit_button_and_says_read_only() {
    let Some(run) = run_edit_page() else { return };
    // AC 5.
    let r = &run["inbox"]["shown"];
    assert_eq!(r["id"], "inbox/p", "{r}");
    // HUN-288: `Run AI` is there for an inbox note too (the AI file is not the note).
    assert_eq!(
        r["buttons"],
        json!(["Run AI", "Copy id", "Move to trash"]),
        "{r}"
    );
    assert_eq!(r["readOnly"], "Inbox notes are read-only.", "{r}");
}

// ---------------------------------------------------------------------------------------------
// HUN-288: `Run AI`, the `enrich` command. A stub `claude` is first on PATH (a script that
// records what it was given); HOME is a temp dir; no test starts the real `claude`.
// ---------------------------------------------------------------------------------------------

const STUB_OK: &str = "printf '%s' \"$STUB_ANSWER\"";

/// A temp HOME and a stub `claude` first on `PATH` (the old `PATH` stays after it, so `node` and
/// `sh` tools resolve); both come back on drop. Holds `PATH_LOCK`: the environment is shared.
struct AiEnv {
    sb: Sandbox,
    rec: PathBuf,
    old: Vec<(&'static str, Option<std::ffi::OsString>)>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

fn answer_json(summary: &str) -> String {
    json!({
        "type": "result", "subtype": "success", "is_error": false,
        "result": "ignored",
        "structured_output": {
            "title": "Buy a USB-C hub", "kind": "todo", "summary": summary,
            "tags_suggested": ["hardware", "shopping"],
        },
        "modelUsage": {"claude-haiku-5-5": {"inputTokens": 10}},
    })
    .to_string()
}

impl AiEnv {
    /// `script` is the stub's last step; it runs after argv, env and stdin are recorded.
    fn new(name: &str, script: &str, answer: &str) -> AiEnv {
        use std::os::unix::fs::PermissionsExt;
        let lock = PATH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let sb = Sandbox::new(name);
        let home = sb.dir.path().join("home");
        let bin = sb.dir.path().join("bin");
        let rec = sb.dir.path().join("rec");
        for d in [&home, &bin, &rec] {
            std::fs::create_dir_all(d).unwrap();
        }
        let stub = bin.join("claude");
        std::fs::write(
            &stub,
            format!(
                "#!/bin/sh\nfor a in \"$@\"; do printf '[%s]\\n' \"$a\"; done > '{r}/argv'\nenv > '{r}/env'\npwd > '{r}/pwd'\ncat > '{r}/stdin'\necho call >> '{r}/calls'\n{script}\n",
                r = rec.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
        let path = format!(
            "{}:{}",
            bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let mut old = vec![];
        for (key, value) in [
            ("HOME", home.as_os_str().to_owned()),
            ("PATH", path.into()),
            ("STUB_ANSWER", answer.into()),
            ("ANTHROPIC_API_KEY", "must-not-reach-the-child".into()),
        ] {
            old.push((key, std::env::var_os(key)));
            std::env::set_var(key, value);
        }
        AiEnv {
            sb,
            rec,
            old,
            _lock: lock,
        }
    }
    fn ok(name: &str) -> AiEnv {
        AiEnv::new(
            name,
            STUB_OK,
            &answer_json("A reminder to buy a USB-C hub."),
        )
    }
    fn config(&self, text: &str) {
        let path = self
            .sb
            .dir
            .path()
            .join("home/.config/easy-note/config.toml");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    fn calls(&self) -> usize {
        std::fs::read_to_string(self.rec.join("calls")).map_or(0, |c| c.lines().count())
    }
    fn recorded(&self, name: &str) -> String {
        std::fs::read_to_string(self.rec.join(name)).unwrap_or_default()
    }
    fn ai_files(&self) -> Vec<PathBuf> {
        let dir = self.sb.notes().join(".easy-note/ai");
        if dir.exists() {
            files(&dir)
        } else {
            vec![]
        }
    }
}

impl Drop for AiEnv {
    fn drop(&mut self) {
        for (key, value) in &self.old {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }
}

const PLAIN: &str = "20261119-090000-0a11";

fn plain_note(env: &AiEnv) -> PathBuf {
    env.sb.note(
        PLAIN,
        "2026-11-19T09:00:00+07:00",
        "tags: []\n",
        "Buy a USB-C hub",
    );
    env.sb.notes().join("2026/11").join(format!("{PLAIN}.md"))
}

#[test]
fn enrich_runs_the_per_note_step_and_answers_the_detail_with_the_ai_filled_in() {
    let env = AiEnv::ok("enrich-ok");
    let file = plain_note(&env);
    // The Obsidian view `<notes dir>/ai/<id>.md` is written when a vault holds the notes dir.
    std::fs::create_dir_all(env.sb.dir.path().join(".obsidian")).unwrap();
    let before = std::fs::read(&file).unwrap();
    // AC 1 + Q1 option A: no config.toml at all, and the click still runs with the defaults.
    let out = call(&env.sb.store(), &json!({"cmd": "enrich", "id": PLAIN}));
    assert!(out.get("error").is_none(), "{out}");
    assert_eq!(out["id"], PLAIN, "{out}");
    assert_eq!(out["ai"]["title"], "Buy a USB-C hub", "{out}");
    assert_eq!(out["ai"]["kind"], "to-do", "{out}");
    assert_eq!(
        out["ai"]["summary"], "A reminder to buy a USB-C hub.",
        "{out}"
    );
    assert_eq!(out["ai"]["tags_suggested"], json!(["hardware", "shopping"]));
    assert_eq!(out["ai"]["model"], "claude-haiku-5-5", "{out}");
    assert_eq!(out["ai"]["stale"], false, "{out}");
    assert_eq!(
        out["ai"],
        call(&env.sb.store(), &json!({"cmd": "detail", "id": PLAIN}))["ai"]
    );
    // The same AI file and the same view as `ai enrich`.
    let names: Vec<String> = env
        .ai_files()
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, [format!("{PLAIN}.json")]);
    assert!(env.sb.notes().join(format!("ai/{PLAIN}.md")).is_file());
    // The same `claude -p` argv, env and stdin as `ai enrich`.
    let settings = AiSettings::default();
    let want: String = enrich::argv(&settings, &enrich::schema().to_string())[1..]
        .iter()
        .map(|a| format!("[{a}]\n"))
        .collect();
    assert_eq!(env.recorded("argv"), want);
    assert!(!env.recorded("env").contains("ANTHROPIC_API_KEY"));
    let stdin = env.recorded("stdin");
    assert!(stdin.starts_with(enrich::PROMPT), "{stdin}");
    assert!(
        stdin.contains("<note>\nBuy a USB-C hub\n</note>"),
        "{stdin}"
    );
    // AC 4 (NFR-012): the note file is byte for byte what it was.
    assert_eq!(std::fs::read(&file).unwrap(), before);
}

#[test]
fn enrich_runs_again_over_old_detail_and_the_stale_mark_goes() {
    let env = AiEnv::ok("enrich-again");
    plain_note(&env);
    env.sb.enrich(PLAIN, "bug");
    let path = env.sb.notes().join("2026/11").join(format!("{PLAIN}.md"));
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str("one more line\n");
    std::fs::write(&path, text).unwrap();
    let store = env.sb.store();
    assert_eq!(
        call(&store, &json!({"cmd": "detail", "id": PLAIN}))["ai"]["stale"],
        true
    );
    let out = call(&store, &json!({"cmd": "enrich", "id": PLAIN}));
    assert_eq!(out["ai"]["title"], "Buy a USB-C hub", "{out}");
    assert_eq!(out["ai"]["stale"], false, "{out}");
}

#[test]
fn enrich_errors_are_words_and_write_nothing() {
    // AC 3: each failure is `{"error": "<words>"}`; no AI file, the note file unchanged.
    let cases: [(&str, &str, &str, &str, &str); 6] = [
        // (stub script, [ai] section, id, what the error holds, why)
        ("exit 1", "", PLAIN, "log in once: claude", "not logged in"),
        (
            "echo not json",
            "",
            PLAIN,
            "the answer is not JSON",
            "bad answer",
        ),
        (
            "exec sleep 30",
            "[ai]\ntimeout_secs = 1\n",
            PLAIN,
            "timeout",
            "timed out",
        ),
        (
            STUB_OK,
            "[ai]\ncommand = \"easy-note-no-such-claude\"\n",
            PLAIN,
            "[ai] command",
            "claude not found",
        ),
        (
            STUB_OK,
            "[ai]\nmodel = \"-x\"\n",
            PLAIN,
            "[ai] model",
            "bad config",
        ),
        (
            STUB_OK,
            "",
            "20261119-090000-ffff",
            "no such note",
            "unknown id",
        ),
    ];
    for (script, config, id, holds, why) in cases {
        let env = AiEnv::new("enrich-err", script, &answer_json("s"));
        let file = plain_note(&env);
        let before = std::fs::read(&file).unwrap();
        env.config(config);
        let out = call(&env.sb.store(), &json!({"cmd": "enrich", "id": id}));
        let error = out["error"]
            .as_str()
            .unwrap_or_else(|| panic!("{why}: {out}"));
        assert!(error.contains(holds), "{why}: {out}");
        assert_eq!(
            out.as_object().unwrap().len(),
            1,
            "{why}: only the error: {out}"
        );
        assert!(env.ai_files().is_empty(), "{why}: an AI file was written");
        assert_eq!(std::fs::read(&file).unwrap(), before, "{why}");
        if why == "unknown id" || why == "claude not found" || why == "bad config" {
            assert_eq!(env.calls(), 0, "{why}: claude must not run");
        }
    }
}

#[test]
fn enrich_refuses_a_trashed_a_bad_and_a_missing_id_without_running_claude() {
    let env = AiEnv::ok("enrich-ids");
    plain_note(&env);
    let store = env.sb.store();
    assert_eq!(
        call(&store, &json!({"cmd": "trash", "id": PLAIN})),
        json!({"trashed": PLAIN})
    );
    for (request, error) in [
        (json!({"cmd": "enrich", "id": PLAIN}), "no such note"),
        (
            json!({"cmd": "enrich", "id": "../../etc/passwd"}),
            "no such note",
        ),
        (json!({"cmd": "enrich", "id": ""}), "no such note"),
        (json!({"cmd": "enrich"}), "missing id"),
        (json!({"cmd": "enrich", "id": 7}), "missing id"),
    ] {
        assert_eq!(call(&store, &request), json!({"error": error}), "{request}");
    }
    assert_eq!(env.calls(), 0);
    assert!(env.ai_files().is_empty());
}

#[test]
fn enrich_masks_every_string_of_the_answer() {
    // AC 7: a secret the AI wrote is masked in the file (`ai::enrich`) and again on output.
    let cases = planted();
    let (_, secret) = &cases[0];
    let env = AiEnv::new(
        "enrich-mask",
        STUB_OK,
        &answer_json(&format!("the key is {secret}")),
    );
    plain_note(&env);
    let raw = handle(
        &env.sb.store(),
        &json!({"cmd": "enrich", "id": PLAIN}).to_string(),
    );
    assert!(!raw.contains(secret.as_str()), "secret in {raw}");
    assert!(raw.contains("[masked:"), "{raw}");
}

#[test]
fn a_note_edited_during_the_run_is_refused_and_writes_nothing() {
    // The text sent to claude is the text the AI file is stamped for: an edit saved while the
    // child runs must not get a fresh `based_on` for words the AI never saw.
    let script = format!(
        "echo 'edited while AI ran' >> \"$(dirname \"$0\")/../notes/2026/11/{PLAIN}.md\"\n{STUB_OK}"
    );
    let env = AiEnv::new("enrich-race", &script, &answer_json("raced"));
    let file = plain_note(&env);
    let out = call(&env.sb.store(), &json!({"cmd": "enrich", "id": PLAIN}));
    assert_eq!(env.calls(), 1, "claude ran once");
    assert_eq!(out["error"], "the note changed while AI ran", "{out}");
    assert!(out.get("ai").is_none(), "{out}");
    assert!(env.ai_files().is_empty(), "an AI file was written");
    assert!(std::fs::read_to_string(&file)
        .unwrap()
        .contains("edited while AI ran"));
}

#[test]
fn a_slow_claude_does_not_hold_the_other_requests() {
    use std::sync::mpsc;
    use std::time::Duration;
    // AC 2: the stub sleeps 1 s before it answers; meanwhile `board` and `detail` are answered.
    let env = AiEnv::new(
        "enrich-slow",
        &format!("sleep 1\n{STUB_OK}"),
        &answer_json("slow"),
    );
    plain_note(&env);
    let store = env.sb.store();
    let running = Running::default();
    let (tx, rx) = mpsc::channel::<String>();
    let sent = tx.clone();
    let body = json!({"req": 41, "cmd": "enrich", "id": PLAIN}).to_string();
    let started = std::time::Instant::now();
    ipc_threaded(&store, &running, ORIGIN, &body, &|_| {}, move |a| {
        let _ = sent.send(a);
    });
    assert!(
        started.elapsed() < Duration::from_millis(500),
        "the call waited for claude"
    );
    // A second click on the same note while it runs: refused at once, claude is not started twice.
    let again = tx.clone();
    ipc_threaded(&store, &running, ORIGIN, &body, &|_| {}, move |a| {
        let _ = again.send(a);
    });
    let refused: Value =
        serde_json::from_str(&rx.recv_timeout(Duration::from_millis(500)).unwrap()).unwrap();
    assert_eq!(refused["req"], 41, "{refused}");
    assert!(
        refused["error"]
            .as_str()
            .unwrap()
            .contains("already running"),
        "{refused}"
    );
    // Other requests (the page's `board`, `detail`) are answered while the child runs; so is
    // an `enrich` of another note.
    for request in [
        json!({"req": 42, "cmd": "board"}),
        json!({"req": 43, "cmd": "detail", "id": PLAIN}),
    ] {
        let sent = tx.clone();
        ipc_threaded(
            &store,
            &running,
            ORIGIN,
            &request.to_string(),
            &|_| {},
            move |a| {
                let _ = sent.send(a);
            },
        );
        let got: Value =
            serde_json::from_str(&rx.recv_timeout(Duration::from_millis(500)).unwrap()).unwrap();
        assert_eq!(got["req"], request["req"], "{got}");
        assert!(got.get("error").is_none(), "{got}");
    }
    assert!(
        rx.try_recv().is_err(),
        "the enrich answer came before the child ended"
    );
    // The answer reaches the page when the child ends, with the request's `req`.
    let done: Value =
        serde_json::from_str(&rx.recv_timeout(Duration::from_secs(10)).unwrap()).unwrap();
    assert_eq!(done["req"], 41, "{done}");
    assert_eq!(done["ai"]["summary"], "slow", "{done}");
    assert_eq!(env.calls(), 1);
    // The note may be run again once the first run ended.
    let sent = tx.clone();
    ipc_threaded(&store, &running, ORIGIN, &body, &|_| {}, move |a| {
        let _ = sent.send(a);
    });
    let second: Value =
        serde_json::from_str(&rx.recv_timeout(Duration::from_secs(10)).unwrap()).unwrap();
    assert!(second.get("error").is_none(), "{second}");
}

#[test]
fn a_request_from_another_page_is_dropped_before_any_child_starts() {
    use std::sync::mpsc;
    use std::time::Duration;
    let env = AiEnv::ok("enrich-origin");
    plain_note(&env);
    let (tx, rx) = mpsc::channel::<String>();
    let body = json!({"req": 1, "cmd": "enrich", "id": PLAIN}).to_string();
    ipc_threaded(
        &env.sb.store(),
        &Running::default(),
        "https://evil.example/",
        &body,
        &|_| {},
        move |a| {
            let _ = tx.send(a);
        },
    );
    assert!(rx.recv_timeout(Duration::from_millis(300)).is_err());
    assert_eq!(env.calls(), 0);
}

// HUN-288 page side: the `Run AI` button and its states (AC 5, 6), over the stub IPC.

#[test]
fn run_ai_sits_in_the_actions_row_and_reads_again_when_the_note_has_ai_detail() {
    let Some(run) = run_edit_page() else { return };
    let none = &run["ai_idle"]["none"];
    assert_eq!(
        none["buttons"],
        json!(["Edit", "Run AI", "Copy id", "Move to trash"]),
        "{none}"
    );
    assert_eq!(none["disabled"], json!([]), "{none}");
    let with = &run["ai_idle_with_ai"]["shown"];
    assert_eq!(
        with["buttons"],
        json!(["Add as tags", "Edit", "Re-run", "Copy id", "Move to trash"]),
        "{with}"
    );
}

#[test]
fn run_ai_shows_running_once_then_the_new_ai_detail() {
    let Some(run) = run_edit_page() else { return };
    let r = &run["ai_run"];
    assert_eq!(r["markBefore"], "", "{r}");
    // While it runs: the button is disabled and reads `AI running…`.
    assert_eq!(r["running"]["disabled"], json!(["AI running…"]), "{r}");
    assert!(
        r["running"]["buttons"]
            .as_array()
            .unwrap()
            .contains(&json!("AI running…")),
        "{r}"
    );
    // Two more clicks while it runs send nothing: one `enrich` request for the note.
    let sent = r["sent"].as_array().unwrap();
    assert_eq!(sent.len(), 1, "{r}");
    assert_eq!(sent[0]["cmd"], "enrich", "{r}");
    // The answer fills the AI detail section; the button comes back as `Re-run`.
    let shown = &r["shown"];
    assert_eq!(shown["aiTitle"], "AI title new", "{r}");
    assert_eq!(shown["aiKind"], "type: idea", "{r}");
    assert_eq!(shown["aiSummary"], "AI summary new", "{r}");
    assert_eq!(shown["aiTags"], json!(["#alpha", "#beta"]), "{r}");
    assert_eq!(shown["disabled"], json!([]), "{r}");
    assert!(
        shown["buttons"]
            .as_array()
            .unwrap()
            .contains(&json!("Re-run")),
        "{r}"
    );
    // The board is asked again, so the row gets its AI mark.
    assert_eq!(r["boards"], 1, "{r}");
    assert_eq!(r["markAfter"], "AI", "{r}");
    let again = &run["ai_again"];
    assert_eq!(
        again["running"]["disabled"],
        json!(["AI running…"]),
        "{again}"
    );
    assert_eq!(again["shown"]["aiTitle"], "AI title new", "{again}");
    assert_eq!(again["shown"]["stale"], false, "{again}");
}

#[test]
fn run_ai_error_is_a_toast_and_the_button_comes_back() {
    let Some(run) = run_edit_page() else { return };
    let r = &run["ai_error"];
    let toast = r["first"]["toast"].as_str().unwrap();
    assert!(toast.starts_with("Run AI failed: claude not found"), "{r}");
    assert!(toast.contains("[ai] command"), "{r}");
    assert_eq!(r["first"]["disabled"], json!([]), "{r}");
    assert!(
        r["first"]["buttons"]
            .as_array()
            .unwrap()
            .contains(&json!("Run AI")),
        "{r}"
    );
    assert_eq!(r["first"]["aiTitle"], "", "no AI detail was invented: {r}");
    // The button works again: a second click sends a second request, and it succeeds.
    assert_eq!(r["sent"], 2, "{r}");
    assert_eq!(r["second"]["aiTitle"], "AI title new", "{r}");
}

#[test]
fn an_answer_for_a_note_no_longer_selected_is_not_lost() {
    let Some(run) = run_edit_page() else { return };
    // AC 6: select another note while AI runs.
    let r = &run["ai_switch"];
    assert_eq!(r["sent"], 1, "{r}");
    assert_ne!(r["away"]["id"], r["back"]["id"], "{r}");
    // Back on the note before the answer: it still shows it is running (one run per note).
    assert_eq!(r["back"]["disabled"], json!(["AI running…"]), "{r}");
    assert_eq!(r["back"]["aiTitle"], "", "{r}");
    // The answer then fills the panel of the note that is selected again; the other panel is untouched.
    assert_eq!(r["done"]["aiTitle"], "AI title new", "{r}");
    assert_ne!(r["elsewhere"]["aiTitle"], "AI title new", "{r}");
    assert_eq!(r["mark"], "AI", "{r}");
    let r = &run["ai_answer_elsewhere"];
    assert_ne!(
        r["shownOther"]["aiTitle"], "AI title new",
        "the panel of the selected note is not touched: {r}"
    );
    assert_eq!(r["mark"], "AI", "the row has its AI mark: {r}");
    assert_eq!(
        r["back"]["aiTitle"], "AI title new",
        "coming back shows the AI detail: {r}"
    );
}

#[test]
fn an_answer_while_the_note_is_in_edit_mode_keeps_the_draft() {
    let Some(run) = run_edit_page() else { return };
    let r = &run["ai_while_editing"];
    assert_eq!(r["editing"]["editing"], true, "{r}");
    assert_eq!(r["editing"]["draft"], "my draft kept", "{r}");
    assert_eq!(r["after"]["editing"], false, "{r}");
    assert_eq!(r["after"]["aiTitle"], "AI title new", "{r}");
}

// ---------------------------------------------------------------------------------------------
// HUN-289: `easy-note quick` in a `window` build opens the capture dialog alone, in a small window.
// The page talks `context`, `add` and `close` only; Rust keeps the answer and `quick` saves it.
// ---------------------------------------------------------------------------------------------

fn chip_context() -> Context {
    Context {
        repo: Some("easy-note-mcp".into()),
        task: Some("HUN-289".into()),
        branch: Some("feat/HUN-289-quick".into()),
        ..Context::default()
    }
}

fn reply_of(got: Capture) -> Value {
    match got {
        Capture::Reply(answer) => serde_json::from_str(&answer).unwrap(),
        other => panic!("not a reply: {other:?}"),
    }
}

#[test]
fn the_quick_window_answers_context_from_the_chip_quick_computed() {
    // AC 5: the chip is the one `quick` computed (`--cwd` or the last context), not a store read.
    let out = reply_of(capture_ipc(
        r#"{"req": 1, "cmd": "context"}"#,
        &chip_context(),
    ));
    assert_eq!(
        out,
        json!({"req": 1, "context": {"repo": "easy-note-mcp", "task": "HUN-289"}})
    );
    let none = reply_of(capture_ipc(
        r#"{"req": 2, "cmd": "context"}"#,
        &Context {
            branch: Some("main".into()),
            ..Context::default()
        },
    ));
    assert_eq!(none, json!({"req": 2, "context": null}));
}

#[test]
fn the_quick_window_add_hands_the_text_back_and_close_closes() {
    // AC 2 and 3: the text and the chip choice go back to `quick`, which saves them as it always did.
    assert_eq!(
        capture_ipc(
            r#"{"req": 3, "cmd": "add", "text": "fix the tray\nsecond line", "keep_context": true}"#,
            &chip_context()
        ),
        Capture::Save {
            text: "fix the tray\nsecond line".into(),
            keep_context: true
        }
    );
    assert_eq!(
        capture_ipc(r#"{"cmd": "add", "text": "x"}"#, &chip_context()),
        Capture::Save {
            text: "x".into(),
            keep_context: false
        }
    );
    assert_eq!(
        capture_ipc(r#"{"req": 4, "cmd": "close"}"#, &chip_context()),
        Capture::Close
    );
    // Empty text is never a save: the page closes on it, and a bad page gets the `add` error.
    let empty = reply_of(capture_ipc(
        r#"{"req": 5, "cmd": "add", "text": " \n "}"#,
        &chip_context(),
    ));
    assert_eq!(empty, json!({"req": 5, "error": "empty"}));
    let missing = reply_of(capture_ipc(r#"{"req": 6, "cmd": "add"}"#, &chip_context()));
    assert_eq!(missing, json!({"req": 6, "error": "missing text"}));
}

#[test]
fn the_quick_window_refuses_text_over_the_limit_and_keeps_the_dialog() {
    // As the dashboard's `add`: the page gets "the text is too long" (dialog and text stay), the
    // window is not closed on it, and text at the limit is still a save.
    let at_limit = "a".repeat(20_000);
    assert_eq!(
        capture_ipc(
            &json!({"req": 7, "cmd": "add", "text": at_limit}).to_string(),
            &chip_context()
        ),
        Capture::Save {
            text: at_limit,
            keep_context: false
        }
    );
    let too_long = "é".repeat(20_001);
    let got = reply_of(capture_ipc(
        &json!({"req": 8, "cmd": "add", "text": too_long}).to_string(),
        &chip_context(),
    ));
    assert_eq!(got, json!({"req": 8, "error": "the text is too long"}));
}

#[test]
fn the_quick_window_limit_counts_the_trimmed_text() {
    // Review P3: `quick::run` saves the trimmed text, so padding does not count against the limit.
    let padded = format!("{}\n\n\n", "a".repeat(19_999));
    assert_eq!(
        capture_ipc(
            &json!({"req": 9, "cmd": "add", "text": padded}).to_string(),
            &chip_context()
        ),
        Capture::Save {
            text: padded,
            keep_context: false
        }
    );
    let padded_over = format!("  {}\n\n", "a".repeat(20_001));
    let got = reply_of(capture_ipc(
        &json!({"req": 10, "cmd": "add", "text": padded_over}).to_string(),
        &chip_context(),
    ));
    assert_eq!(got, json!({"req": 10, "error": "the text is too long"}));
}

#[test]
fn the_quick_window_refuses_every_other_command() {
    // AC 7: no board, detail, edit or trash in the quick window.
    for cmd in [
        "board",
        "detail",
        "edit",
        "trash",
        "restore",
        "accept_ai",
        "x",
    ] {
        let body = json!({"req": 7, "cmd": cmd, "id": "20261119-090000-aaaa"}).to_string();
        let out = reply_of(capture_ipc(&body, &chip_context()));
        assert_eq!(out, json!({"req": 7, "error": "unknown command"}), "{cmd}");
    }
    assert_eq!(
        reply_of(capture_ipc("not json", &chip_context())),
        json!({"error": "bad request"})
    );
    assert_eq!(
        reply_of(capture_ipc(r#"{"req": 8}"#, &chip_context())),
        json!({"req": 8, "error": "missing cmd"})
    );
    // The same origin check as the dashboard's IPC.
    let body = r#"{"req": 1, "cmd": "close"}"#;
    assert_eq!(
        capture_from("easy-note://localhost/quick.html", body, &chip_context()),
        Some(Capture::Close)
    );
    for source in ["easy-note://localhost.evil/quick.html", "https://x/", ""] {
        assert_eq!(
            capture_from(source, body, &chip_context()),
            None,
            "{source}"
        );
    }
}

#[test]
fn the_quick_window_masks_a_secret_in_the_chip() {
    let (_, secret) = planted().into_iter().next().expect("a planted secret");
    let ctx = Context {
        repo: Some(secret.clone()),
        ..Context::default()
    };
    let raw = match capture_ipc(r#"{"req": 1, "cmd": "context"}"#, &ctx) {
        Capture::Reply(raw) => raw,
        other => panic!("{other:?}"),
    };
    assert!(!raw.contains(&secret), "secret in {raw}");
}

#[test]
fn the_quick_page_is_served_and_loads_only_the_dialog() {
    let (mime, body) = asset("/quick.html").expect("the quick page");
    assert_eq!(mime, "text/html; charset=utf-8");
    let html = std::str::from_utf8(body).unwrap();
    assert!(html.contains(&format!(
        r#"http-equiv="Content-Security-Policy" content="{CSP}""#
    )));
    assert!(html.contains(r#"role="dialog" aria-modal="true" aria-label="New note""#));
    assert!(
        html.contains(r#"id="cap" hidden"#),
        "open() must see a closed dialog"
    );
    for gone in ["id=\"groups\"", "id=\"search\"", "app.js", "id=\"detail\""] {
        assert!(!html.contains(gone), "quick.html has {gone}");
    }
    // The shared dialog code, then the quick page's own few lines: one implementation (AC 4).
    let order: Vec<usize> = ["ipc.js", "capture.js", "quick.js"]
        .iter()
        .map(|s| {
            html.find(&format!(r#"src="{s}""#))
                .unwrap_or_else(|| panic!("{s}"))
        })
        .collect();
    assert!(order.windows(2).all(|w| w[0] < w[1]), "{order:?}");
    let index = std::str::from_utf8(asset("/").unwrap().1).unwrap();
    let cap_at = index
        .find(r#"src="capture.js""#)
        .expect("dashboard loads capture.js");
    assert!(cap_at < index.find(r#"src="app.js""#).unwrap());
    // The quick page asks for nothing but `context`, `add` and `close`.
    let quick = std::str::from_utf8(asset("/quick.js").unwrap().1).unwrap();
    for gone in ["board", "detail", ".edit(", "trash"] {
        assert!(!quick.contains(gone), "quick.js uses {gone}");
    }
    assert_eq!(asset("/quick.css").unwrap().0, "text/css; charset=utf-8");
    assert_eq!(
        asset("/capture.js").unwrap().0,
        "text/javascript; charset=utf-8"
    );
    assert!(QUICK_START.ends_with("/quick.html") && allowed_url(QUICK_START));
}

/// The quick page, run for real: `ipc.js`, `capture.js` and `quick.js` in node over a fake DOM
/// (`tests/fixtures/window/quick_harness.js`). Skipped, with a printed note, without `node`.
fn run_quick_page() -> Option<Value> {
    let harness =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/window/quick_harness.js");
    let out = match std::process::Command::new("node")
        .arg(harness)
        .arg(assets_dir())
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            eprintln!("skipped: no node to run the page ({e})");
            return None;
        }
    };
    assert!(
        out.status.success(),
        "quick harness failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Some(serde_json::from_slice(&out.stdout).unwrap())
}

#[test]
fn the_quick_page_keys_follow_the_dashboard_dialog() {
    let Some(run) = run_quick_page() else { return };
    let ctx = json!({"cmd": "context"});
    // AC 1: open at start with the chip and the caret in the box; it asks for the context only.
    assert_eq!(
        run["start"],
        json!({"open": true, "chip": "easy-note-mcp · HUN-289", "focusIsBox": true, "textLen": 0, "error": null, "sent": [ctx]}),
        "{run}"
    );
    // AC 2: Enter sends one `add` with the text and the chip kept; a second Enter sends nothing more.
    assert_eq!(
        run["enter"]["sent"],
        json!([ctx, {"cmd": "add", "text": "fix the tray", "keep_context": true}]),
        "{run}"
    );
    // AC 4: Shift+Enter and Enter while composing are not a save.
    assert_eq!(run["shift_enter"]["sent"], json!([ctx]), "{run}");
    assert_eq!(run["composing"]["sent"], json!([ctx]), "{run}");
    // AC 3: Esc and an empty Enter ask to close; nothing is added.
    assert_eq!(run["esc"]["sent"], json!([ctx, {"cmd": "close"}]), "{run}");
    assert_eq!(
        run["empty_enter"]["sent"],
        json!([ctx, {"cmd": "close"}]),
        "{run}"
    );
    // AC 4: Backspace on the empty box drops the chip; the save then keeps no context.
    assert_eq!(
        run["drop_chip"]["sent"],
        json!([ctx, {"cmd": "add", "text": "no context", "keep_context": false}]),
        "{run}"
    );
    assert_eq!(run["drop_chip"]["chip"], Value::Null, "{run}");
    // Review P3: Enter before the `context` reply waits for it, so the chip's context is kept.
    assert_eq!(
        run["fast_enter"]["sent"],
        json!([ctx, {"cmd": "add", "text": "quick one", "keep_context": true}]),
        "{run}"
    );
    assert_eq!(run["no_chip"]["chip"], Value::Null, "{run}");
    // Review P2: an `add` error reply keeps the dialog and the text and shows the message.
    let err = &run["add_error"];
    assert_eq!(err["open"], json!(true), "{run}");
    assert_eq!(err["error"], json!("the text is too long"), "{run}");
    assert_eq!(err["textLen"], json!(20_001), "{run}");
    assert_eq!(run["enter"]["error"], Value::Null, "{run}");
}

// ---------------------------------------------------------------------------------------------
// HUN-290: one dashboard window per notes dir (FR-045). The lock and the request socket are plain
// files and sockets over a temp notes dir; no window opens.
// ---------------------------------------------------------------------------------------------

/// A child process that takes the window lock and waits to be killed (the test binary run again,
/// filtered to `lock_holder_child`).
fn spawn_lock_holder(root: &Path) -> std::process::Child {
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "lock_holder_child", "--nocapture", "--ignored"])
        .env("EASY_NOTE_TEST_LOCK_DIR", root)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::BufRead;
    let mut out = std::io::BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    while !line.contains("LOCKED") {
        line.clear();
        assert!(
            out.read_line(&mut line).unwrap() > 0,
            "the child never locked"
        );
    }
    child
}

#[test]
#[ignore = "run only by spawn_lock_holder"]
fn lock_holder_child() {
    let Some(root) = std::env::var_os("EASY_NOTE_TEST_LOCK_DIR") else {
        return;
    };
    let _owner = single::acquire(Path::new(&root)).unwrap().expect("free");
    println!("LOCKED");
    std::thread::sleep(std::time::Duration::from_secs(60));
}

#[test]
fn hun290_ac4_the_lock_is_a_file_in_the_notes_dir_and_only_one_owner_holds_it() {
    let sb = Sandbox::new("single-lock");
    let root = sb.dir.path().join("fresh notes");
    let owner = single::acquire(&root).unwrap().expect("first call owns");
    assert!(root.join(".easy-note/window.lock").is_file());
    assert!(
        single::acquire(&root).unwrap().is_none(),
        "second call: held"
    );
    drop(owner);
    assert!(single::acquire(&root).unwrap().is_some(), "free again");
}

#[test]
fn hun290_serving_requests_keeps_the_lock_held() {
    let sb = Sandbox::new("single-serve");
    let root = sb.dir.path().to_path_buf();
    let owner = single::acquire(&root).unwrap().expect("first call owns");
    owner.serve(|_| {});
    assert!(
        single::acquire(&root).unwrap().is_none(),
        "a serving window still holds the lock"
    );
}

#[test]
fn hun290_ac4_a_killed_window_leaves_a_lock_the_next_start_takes_over() {
    let sb = Sandbox::new("single-stale");
    let root = sb.dir.path().to_path_buf();
    let mut child = spawn_lock_holder(&root);
    assert!(
        single::acquire(&root).unwrap().is_none(),
        "held by the child"
    );
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(root.join(".easy-note/window.lock").exists(), "left behind");
    assert!(
        single::acquire(&root).unwrap().is_some(),
        "the next start takes it over"
    );
}

#[test]
fn hun290_ac1_ac3_a_second_call_sends_a_focus_request_with_the_note_id() {
    let sb = Sandbox::new("single-focus");
    let root = sb.dir.path().to_path_buf();
    let owner = single::acquire(&root).unwrap().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    owner.serve(move |note| {
        let _ = tx.send(note);
    });
    let began = std::time::Instant::now();
    single::request_focus(&root, None).unwrap();
    single::request_focus(&root, Some("20261119-090000-abcd")).unwrap();
    assert!(began.elapsed() < std::time::Duration::from_secs(2));
    let wait = std::time::Duration::from_secs(2);
    assert_eq!(rx.recv_timeout(wait).unwrap(), None);
    assert_eq!(
        rx.recv_timeout(wait).unwrap().as_deref(),
        Some("20261119-090000-abcd")
    );
}

#[test]
fn hun290_a_request_with_a_bad_line_or_id_is_dropped() {
    assert_eq!(single::parse("focus\n"), Some(None));
    assert_eq!(single::parse("focus abc\n"), Some(Some("abc".to_string())));
    assert_eq!(single::parse("quit\n"), None);
    assert_eq!(single::parse("focus a b\n"), None);
    assert_eq!(single::parse(""), None);
}

#[test]
fn hun290_a_second_call_waits_then_gives_up_when_nothing_listens() {
    let sb = Sandbox::new("single-nolistener");
    let root = sb.dir.path().to_path_buf();
    let _owner = single::acquire(&root).unwrap().unwrap(); // lock held, socket never served
    let began = std::time::Instant::now();
    let r = single::request_focus_within(&root, None, std::time::Duration::from_millis(300));
    // The socket is bound by `acquire`, so the request queues; it must not hang or crash.
    assert!(r.is_ok() || r.is_err());
    assert!(began.elapsed() < std::time::Duration::from_secs(2));
    // No socket at all: an error after the wait.
    let none = sb.dir.path().join("none");
    let began = std::time::Instant::now();
    assert!(
        single::request_focus_within(&none, None, std::time::Duration::from_millis(300)).is_err()
    );
    assert!(began.elapsed() >= std::time::Duration::from_millis(250));
}

#[test]
fn hun290_a_long_notes_dir_path_uses_a_short_socket_path() {
    let mut deep = PathBuf::from("/tmp");
    for _ in 0..12 {
        deep.push("a-long-directory-name");
    }
    let path = single::socket_path(&deep);
    assert!(path.as_os_str().len() < 100, "{}", path.display());
    assert_ne!(path, single::socket_path(&deep.join("other")));
    assert_eq!(path, single::socket_path(&deep));
    // And it works end to end: a real deep notes dir.
    let sb = Sandbox::new("single-deep");
    let mut root = sb.dir.path().to_path_buf();
    for _ in 0..10 {
        root.push("a-long-directory-name");
    }
    assert!(root.join(".easy-note/window.sock").as_os_str().len() > 108);
    let owner = single::acquire(&root).unwrap().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    owner.serve(move |n| {
        let _ = tx.send(n);
    });
    single::request_focus(&root, Some("x1")).unwrap();
    assert_eq!(
        rx.recv_timeout(std::time::Duration::from_secs(2))
            .unwrap()
            .as_deref(),
        Some("x1")
    );
}

#[test]
fn hun290_ac6_a_different_notes_dir_has_its_own_owner() {
    let a = Sandbox::new("single-a");
    let b = Sandbox::new("single-b");
    let _one = single::acquire(a.dir.path()).unwrap().unwrap();
    assert!(single::acquire(b.dir.path()).unwrap().is_some());
}

#[test]
fn hun290_ac3_the_show_script_passes_the_id_as_one_json_string() {
    let id = "a\"b'</script>\u{2028}\\";
    let script = show_script(id);
    let arg = script
        .strip_prefix("window.__easyNote && window.__easyNote.show && window.__easyNote.show(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or_else(|| panic!("{script}"));
    assert_eq!(serde_json::from_str::<Value>(arg).unwrap(), json!(id));
}

#[test]
fn hun290_ac3_the_page_shows_a_note_when_the_shell_asks() {
    let page = std::str::from_utf8(asset("/app.js").unwrap().1).unwrap();
    assert!(
        page.contains("window.__easyNote.show = "),
        "app.js defines show"
    );
    let Some(run) = run_edit_page() else { return };
    let r = &run["show_request"];
    assert_eq!(r["detail"], json!([ACTED]), "{r}");
    assert_eq!(r["shown"]["id"], ACTED, "{r}");
    assert_eq!(r["current"], "true", "{r}");
}

// ---------------------------------------------------------------------------------------------
// HUN-300: the Settings screen, `settings_get` and `settings_save` (FR-048, FR-052, NFR-014).
// A temp HOME and a stub `claude` (`AiEnv`); no test starts the real one or opens `~/.config`.
// ---------------------------------------------------------------------------------------------

const NOTICE_HAIKU: &str = "Masked note text is sent to your own claude CLI (account folder: ~/.claude, model: haiku). Nothing runs until you confirm.";

/// `AiEnv` with no `CLAUDE_CONFIG_DIR`, so the default account folder is the one named.
fn settings_env(name: &str) -> AiEnv {
    let mut env = AiEnv::ok(name);
    env.old
        .push(("CLAUDE_CONFIG_DIR", std::env::var_os("CLAUDE_CONFIG_DIR")));
    std::env::remove_var("CLAUDE_CONFIG_DIR");
    env
}

impl AiEnv {
    fn config_file(&self) -> PathBuf {
        self.sb
            .dir
            .path()
            .join("home/.config/easy-note/config.toml")
    }
    fn config_text(&self) -> String {
        std::fs::read_to_string(self.config_file()).unwrap_or_default()
    }
    fn config_table(&self) -> toml::Table {
        self.config_text().parse().unwrap()
    }
}

fn settings_get(store: &Store) -> Value {
    call(store, &json!({"cmd": "settings_get"}))
}

/// The page's save: the shown settings with `changes` over them, on the base it loaded.
fn save_request(loaded: &Value, changes: Value) -> Value {
    let mut settings = loaded["settings"].clone();
    for (key, value) in changes.as_object().unwrap() {
        settings[key] = value.clone();
    }
    json!({"cmd": "settings_save", "base": loaded["base"], "settings": settings})
}

fn confirmed(mut request: Value) -> Value {
    request["confirm_notice"] = json!(true);
    request
}

fn organize() -> Value {
    json!({"enrich": "claude", "schedule": "tray"})
}

#[test]
fn settings_get_answers_the_values_the_read_only_keys_and_the_notice() {
    let env = settings_env("set-get");
    let store = env.sb.store();
    let got = settings_get(&store);
    assert_eq!(
        got["settings"],
        json!({"enrich": "off", "schedule": "off", "interval_min": 60, "max_per_run": 20,
               "batch_size": 10, "jobs": ["organize", "drafts", "digest"], "model": "haiku",
               "after_save": false}),
        "{got}"
    );
    assert_eq!(got["command"], "claude");
    assert_eq!(got["account"], "~/.claude");
    assert_eq!(got["path"], env.config_file().to_str().unwrap());
    assert!(got["base"].is_string(), "{got}");
    assert_eq!(got["notice"]["confirmed_model"], Value::Null);
    let template = got["notice"]["template"].as_str().unwrap();
    assert_eq!(
        template
            .replace("{dir}", "~/.claude")
            .replace("{model}", "haiku"),
        NOTICE_HAIKU
    );
    assert!(!env.config_file().exists(), "a read writes nothing");

    env.config("[ai]\nenrich = \"claude\"\ncommand = \"/opt/claude\"\nclaude_config_dir = \"/acct\"\nmodel = \"sonnet\"\ninterval_min = 15\nmax_per_run = 40\nbatch_size = 8\njobs = [\"drafts\"]\nafter_save = true\n");
    let got = settings_get(&store);
    assert_eq!(
        got["settings"],
        json!({"enrich": "claude", "schedule": "off", "interval_min": 15, "max_per_run": 40,
               "batch_size": 8, "jobs": ["drafts"], "model": "sonnet", "after_save": true})
    );
    assert_eq!(got["command"], "/opt/claude");
    assert_eq!(got["account"], "/acct");
}

#[test]
fn settings_get_of_a_file_it_cannot_use_names_the_file() {
    let env = settings_env("set-broken");
    let store = env.sb.store();
    for (text, part) in [
        ("[ai\n", "cannot read"),
        ("[ai]\ninterval_min = 1\n", "[ai] interval_min"),
    ] {
        env.config(text);
        let got = settings_get(&store);
        let error = got["error"].as_str().unwrap();
        assert!(error.contains(part), "{error}");
        assert!(error.contains("config.toml"), "{error}");
        // A save over it is refused too, and the file is as it was.
        let save = call(
            &store,
            &json!({"cmd": "settings_save", "base": "x", "settings": {}}),
        );
        assert!(save["error"].is_string(), "{save}");
        assert_eq!(env.config_text(), text);
    }
}

#[test]
fn organize_turns_on_only_with_the_confirmed_notice_and_records_it() {
    let env = settings_env("set-confirm");
    let store = env.sb.store();
    env.config("# mine\n[ai]\ncommand = \"claude\"\n");
    let loaded = settings_get(&store);
    let refused = call(&store, &save_request(&loaded, organize()));
    assert!(
        refused["error"].as_str().unwrap().contains("notice"),
        "{refused}"
    );
    assert_eq!(env.config_text(), "# mine\n[ai]\ncommand = \"claude\"\n");
    // Only a true `confirm_notice` counts.
    let mut odd = save_request(&loaded, organize());
    odd["confirm_notice"] = json!("yes");
    assert!(call(&store, &odd)["error"].is_string());

    let saved = call(&store, &confirmed(save_request(&loaded, organize())));
    assert_eq!(saved["error"], Value::Null, "{saved}");
    assert_eq!(saved["settings"]["schedule"], "tray");
    assert_eq!(saved["notice"]["confirmed_model"], "haiku");
    assert_ne!(saved["base"], loaded["base"]);
    let file = env.config_table();
    assert_eq!(file["ai"]["schedule"].as_str(), Some("tray"));
    assert_eq!(file["ai"]["enrich"].as_str(), Some("claude"));
    let version = easy_note_mcp::core::config::notice_version(NOTICE_HAIKU);
    assert_eq!(file["ai"]["notice_seen"].as_str(), Some(version.as_str()));
    assert!(env.config_text().starts_with("# mine\n"));
    assert_eq!(env.calls(), 0, "saving starts no claude");
}

#[test]
fn a_seen_notice_is_not_asked_again_until_its_text_changes() {
    let env = settings_env("set-seen");
    let store = env.sb.store();
    let loaded = settings_get(&store);
    let on = call(&store, &confirmed(save_request(&loaded, organize())));
    // A change while it is on needs no new confirm.
    let faster = call(&store, &save_request(&on, json!({"interval_min": 30})));
    assert_eq!(faster["settings"]["interval_min"], 30, "{faster}");
    let off = call(&store, &save_request(&faster, json!({"schedule": "off"})));
    assert_eq!(off["settings"]["schedule"], "off", "{off}");
    assert_eq!(off["notice"]["confirmed_model"], "haiku");
    // The same text: on again at once.
    let again = call(&store, &save_request(&off, organize()));
    assert_eq!(again["settings"]["schedule"], "tray", "{again}");
    // Another model is another text.
    let off = call(&store, &save_request(&again, json!({"schedule": "off"})));
    let mut sonnet = organize();
    sonnet["model"] = json!("sonnet");
    let refused = call(&store, &save_request(&off, sonnet.clone()));
    assert!(
        refused["error"].as_str().unwrap().contains("notice"),
        "{refused}"
    );
    let on = call(&store, &confirmed(save_request(&off, sonnet.clone())));
    assert_eq!(on["notice"]["confirmed_model"], "sonnet", "{on}");
    // So is another account folder, set in the file.
    let off = call(&store, &save_request(&on, json!({"schedule": "off"})));
    assert!(off["error"].is_null(), "{off}");
    let text = env.config_text();
    env.config(&format!("{text}claude_config_dir = \"/work\"\n"));
    let loaded = settings_get(&store);
    assert_eq!(loaded["notice"]["confirmed_model"], Value::Null, "{loaded}");
    assert!(call(&store, &save_request(&loaded, sonnet))["error"].is_string());
}

#[test]
fn organize_cannot_be_on_while_the_ai_is_off() {
    let env = settings_env("set-ai-off");
    let store = env.sb.store();
    let loaded = settings_get(&store);
    let request = save_request(&loaded, json!({"schedule": "tray"}));
    for request in [request.clone(), confirmed(request)] {
        let out = call(&store, &request);
        let error = out["error"].as_str().unwrap();
        assert!(error.contains("[ai] schedule"), "{error}");
        assert!(!env.config_file().exists());
    }
}

#[test]
fn settings_save_refuses_a_bad_value_naming_the_key_and_writes_nothing() {
    let env = settings_env("set-bad");
    let store = env.sb.store();
    env.config("[ai]\nmodel = \"haiku\"\n");
    let loaded = settings_get(&store);
    for (key, value) in [
        ("interval_min", json!(0)),
        ("interval_min", json!("abc")),
        ("interval_min", json!("30")),
        ("interval_min", json!("")),
        ("interval_min", json!(1.5)),
        ("interval_min", json!(-1)),
        ("interval_min", Value::Null),
        ("interval_min", json!(u64::MAX)),
        ("max_per_run", json!(201)),
        ("batch_size", json!(21)),
        ("model", json!("")),
        ("model", json!("--x")),
        ("jobs", json!(["sweep"])),
        ("jobs", json!("organize")),
        ("enrich", json!("api")),
        ("schedule", json!("cron")),
        ("after_save", json!("yes")),
    ] {
        let mut changes = json!({});
        changes[key] = value.clone();
        let out = call(&store, &save_request(&loaded, changes));
        let error = out["error"].as_str().unwrap_or_else(|| panic!("{out}"));
        assert!(error.contains(&format!("[ai] {key}")), "{value}: {error}");
        assert_eq!(env.config_text(), "[ai]\nmodel = \"haiku\"\n", "{key}");
        // A key left out is not silently reset.
        let mut short = save_request(&loaded, json!({}));
        short["settings"].as_object_mut().unwrap().remove(key);
        let out = call(&store, &short);
        assert!(out["error"].as_str().unwrap().contains(key), "{out}");
        assert_eq!(env.config_text(), "[ai]\nmodel = \"haiku\"\n", "{key}");
    }
    for request in [
        json!({"cmd": "settings_save"}),
        json!({"cmd": "settings_save", "base": loaded["base"]}),
        json!({"cmd": "settings_save", "base": loaded["base"], "settings": []}),
        json!({"cmd": "settings_save", "settings": loaded["settings"]}),
    ] {
        let out = call(&store, &request);
        assert!(out["error"].is_string(), "{request}: {out}");
        assert_eq!(env.config_text(), "[ai]\nmodel = \"haiku\"\n");
    }
}

#[test]
fn settings_save_refuses_a_file_changed_after_settings_get() {
    let env = settings_env("set-outside");
    let store = env.sb.store();
    let loaded = settings_get(&store);
    env.config("[ai]\nmodel = \"opus\"\n");
    for request in [
        save_request(&loaded, json!({"interval_min": 30})),
        confirmed(save_request(&loaded, organize())),
        save_request(&loaded, organize()),
    ] {
        let out = call(&store, &request);
        assert!(
            out["error"].as_str().unwrap().contains("changed outside"),
            "{out}"
        );
        assert_eq!(env.config_text(), "[ai]\nmodel = \"opus\"\n");
    }
}

#[test]
fn settings_save_never_writes_the_command_or_the_account_folder() {
    let env = settings_env("set-readonly");
    let store = env.sb.store();
    env.config("[ai]\ncommand = \"claude\"\nclaude_config_dir = \"~/.claude-work\"\n");
    let loaded = settings_get(&store);
    let mut request = save_request(
        &loaded,
        json!({"interval_min": 30, "command": "/tmp/evil", "claude_config_dir": "/tmp/evil",
               "notice_seen": "forged", "timeout_secs": 1, "dir": "/tmp/evil"}),
    );
    request["command"] = json!("/tmp/evil");
    let out = call(&store, &request);
    assert!(out["error"].is_null(), "{out}");
    let text = env.config_text();
    assert!(
        text.starts_with("[ai]\ncommand = \"claude\"\nclaude_config_dir = \"~/.claude-work\"\n"),
        "{text}"
    );
    assert!(text.contains("interval_min = 30\n"), "{text}");
    for gone in ["evil", "forged", "timeout_secs", "dir = \"/"] {
        assert!(!text.contains(gone), "{gone}: {text}");
    }
}

#[test]
fn doctor_shows_the_values_saved_from_the_window() {
    let env = settings_env("set-doctor");
    let store = env.sb.store();
    let loaded = settings_get(&store);
    let mut changes = organize();
    changes["interval_min"] = json!(30);
    changes["max_per_run"] = json!(40);
    changes["batch_size"] = json!(8);
    changes["jobs"] = json!(["organize"]);
    changes["model"] = json!("sonnet");
    changes["after_save"] = json!(true);
    let saved = call(&store, &confirmed(save_request(&loaded, changes)));
    assert!(saved["error"].is_null(), "{saved}");
    let out = std::process::Command::new(BIN)
        .arg("doctor")
        .env("EASY_NOTE_DIR", env.sb.notes())
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("model sonnet, after_save true"),
        "ai line: {text}"
    );
    assert!(
        text.contains("ai schedule: tray, every 30 min, up to 40 notes per run, batch 8, jobs organize, digest at 3h\n"),
        "schedule line: {text}"
    );
}

#[test]
fn a_save_after_a_settings_change_follows_the_new_run_after_each_save() {
    // The open window asks this at each new note, so it needs no restart.
    let env = settings_env("set-after-save");
    let store = env.sb.store();
    assert!(!after_save_on());
    let loaded = settings_get(&store);
    let on = call(
        &store,
        &save_request(&loaded, json!({"enrich": "claude", "after_save": true})),
    );
    assert!(after_save_on(), "{on}");
    call(&store, &save_request(&on, json!({"after_save": false})));
    assert!(!after_save_on());
    let shell = include_str!("../src/surfaces/window/mod.rs");
    assert!(
        shell.contains("if config::after_save_on() {"),
        "the shell reads the setting at each added note"
    );
}

#[test]
fn the_settings_answers_mask_secrets_and_stay_on_the_app_page() {
    let env = settings_env("set-mask");
    let store = env.sb.store();
    let (text, secret) = planted()
        .into_iter()
        .find(|(text, _)| !text.contains('\n') && !text.contains('"') && !text.contains('\\'))
        .unwrap();
    env.config(&format!("[ai]\ncommand = \"{text}\"\n"));
    let out = handle(&store, r#"{"cmd": "settings_get"}"#);
    assert!(!out.contains(&secret), "{out}");
    assert!(out.contains("[masked:"), "{out}");
    let body = r#"{"req": 1, "cmd": "settings_get"}"#;
    assert!(ipc_from(&store, "https://x/", body).is_none());
    let save = save_request(&settings_get(&store), json!({"interval_min": 30})).to_string();
    let before = env.config_text();
    assert!(ipc_from(&store, "easy-note://other/", &save).is_none());
    assert_eq!(env.config_text(), before);
}

#[test]
fn settings_save_without_write_permission_says_which_file() {
    use std::os::unix::fs::PermissionsExt;
    let env = settings_env("set-denied");
    let store = env.sb.store();
    env.config("[ai]\n");
    let loaded = settings_get(&store);
    let folder = env.config_file().parent().unwrap().to_path_buf();
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o500)).unwrap();
    let out = call(&store, &save_request(&loaded, json!({"interval_min": 30})));
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o700)).unwrap();
    let error = out["error"].as_str().unwrap();
    assert!(
        error.starts_with("cannot write ") && error.contains("config.toml"),
        "{error}"
    );
    assert_eq!(env.config_text(), "[ai]\n");
}

// HUN-300 page side: the gear, the Settings dialog and the notice, over the stub IPC fed one real
// `settings_get` answer.

/// The harness run with the Settings scenarios, on the answer of a fresh config.
fn run_settings_page() -> Option<Value> {
    let (board, settings) = {
        let env = settings_env("set-page");
        let store = env.sb.store();
        (call(&store, &json!({"cmd": "board"})), settings_get(&store))
    };
    run_page_with(&board, Some(&json!({ "settings": settings })))
}

#[test]
fn the_gear_opens_settings_with_the_values_of_the_file() {
    let Some(run) = run_settings_page() else {
        return;
    };
    let r = &run["set_open"];
    assert_eq!(r["loading"]["open"], true, "{r}");
    assert_eq!(r["loading"]["loading"], true, "{r}");
    assert_eq!(r["loading"]["form"], false, "{r}");
    assert_eq!(r["posts"], json!(["board", "settings_get"]), "{r}");
    let shown = &r["shown"];
    assert_eq!(shown["form"], true, "{r}");
    assert_eq!(shown["loading"], false, "{r}");
    assert_eq!(shown["asking"], false, "{r}");
    assert_eq!(shown["values"], json!(["60", "20", "10", "haiku"]), "{r}");
    assert_eq!(shown["jobs"], json!([true, true, true]), "{r}");
    assert_eq!(shown["afterSave"], false, "{r}");
    assert_eq!(shown["enrich"], "false", "{r}");
    assert!(
        shown["readOnly"][0]
            .as_str()
            .unwrap()
            .ends_with(".config/easy-note/config.toml"),
        "{r}"
    );
    assert_eq!(shown["readOnly"][1], "claude", "{r}");
    assert_eq!(shown["readOnly"][2], "~/.claude", "{r}");
    // AC 6: the switch is off and disabled with the reason while the AI is off.
    let r = &run["set_disabled"];
    assert_eq!(r["shown"]["scheduleDisabled"], true, "{r}");
    assert_eq!(r["shown"]["reason"], true, "{r}");
    assert_eq!(r["shown"]["schedule"], "false", "{r}");
    assert_eq!(r["shown"]["asking"], false, "{r}");
    assert_eq!(r["saves"], 0, "{r}");
}

#[test]
fn cancel_on_the_notice_saves_nothing_and_leaves_the_switch_off() {
    let Some(run) = run_settings_page() else {
        return;
    };
    let r = &run["set_cancel"];
    assert_eq!(r["asked"]["asking"], true, "{r}");
    assert_eq!(r["asked"]["ask"], NOTICE_HAIKU, "{r}");
    assert_eq!(r["asked"]["schedule"], "false", "off until Confirm: {r}");
    assert_eq!(r["asked"]["inert"], true, "{r}");
    for after in ["cancelled", "escaped"] {
        assert_eq!(r[after]["asking"], false, "{after}: {r}");
        assert_eq!(r[after]["schedule"], "false", "{after}: {r}");
        assert_eq!(r[after]["open"], true, "{after}: {r}");
        assert_eq!(r[after]["inert"], false, "{after}: {r}");
    }
    assert_eq!(r["saves"], 0, "no settings_save call: {r}");
}

#[test]
fn confirm_on_the_notice_saves_once_and_shows_saved() {
    let Some(run) = run_settings_page() else {
        return;
    };
    let r = &run["set_confirm"];
    assert_eq!(r["saves"].as_array().unwrap().len(), 1, "{r}");
    let save = &r["saves"][0];
    assert_eq!(save["confirm_notice"], true, "{r}");
    assert_eq!(save["settings"]["schedule"], "tray", "{r}");
    assert_eq!(save["settings"]["enrich"], "claude", "{r}");
    assert!(save["base"].is_string(), "{r}");
    assert_eq!(r["shown"]["schedule"], "true", "{r}");
    assert_eq!(r["shown"]["open"], true, "{r}");
    assert_eq!(r["shown"]["asking"], false, "{r}");
    assert_eq!(r["shown"]["toast"], "Saved", "{r}");
    assert_eq!(
        r["posts"],
        json!(["board", "settings_get", "settings_save"]),
        "nothing runs: {r}"
    );
}

#[test]
fn a_seen_notice_is_not_shown_again_and_a_new_text_is() {
    let Some(run) = run_settings_page() else {
        return;
    };
    let r = &run["set_seen"];
    assert_eq!(r["flipped"]["shown"]["asking"], false, "{r}");
    assert_eq!(r["flipped"]["shown"]["schedule"], "true", "{r}");
    assert_eq!(r["flipped"]["saves"], 0, "{r}");
    assert_eq!(r["saves"].as_array().unwrap().len(), 1, "{r}");
    assert_eq!(r["saves"][0]["confirm_notice"], false, "{r}");
    assert_eq!(r["saves"][0]["settings"]["schedule"], "tray", "{r}");
    assert_eq!(r["shown"]["open"], false, "Save closes: {r}");
    assert_eq!(r["shown"]["toast"], "Saved", "{r}");
    // The model changed after the switch went on: the new text is confirmed before the save.
    let r = &run["set_model_changed"];
    assert_eq!(r["asked"]["shown"]["asking"], true, "{r}");
    assert_eq!(
        r["asked"]["shown"]["ask"],
        NOTICE_HAIKU.replace("haiku", "sonnet"),
        "{r}"
    );
    assert_eq!(r["asked"]["saves"], 0, "{r}");
    assert_eq!(r["saves"].as_array().unwrap().len(), 1, "{r}");
    assert_eq!(r["saves"][0]["confirm_notice"], true, "{r}");
    assert_eq!(r["saves"][0]["settings"]["model"], "sonnet", "{r}");
    assert_eq!(r["shown"]["open"], false, "{r}");
    // Already on: no notice for another change.
    let r = &run["set_already_on"];
    assert_eq!(r["first"]["schedule"], "true", "{r}");
    assert_eq!(r["saves"].as_array().unwrap().len(), 1, "{r}");
    assert_eq!(r["shown"]["asking"], false, "{r}");
}

#[test]
fn save_sends_the_typed_values_and_a_failed_save_keeps_the_dialog() {
    let Some(run) = run_settings_page() else {
        return;
    };
    let r = &run["set_values"];
    assert_eq!(
        r["saves"][0]["settings"],
        json!({"enrich": "off", "schedule": "off", "interval_min": 30, "max_per_run": 40,
               "batch_size": 8, "jobs": ["organize", "digest"], "model": "haiku",
               "after_save": true}),
        "{r}"
    );
    assert_eq!(r["shown"]["open"], false, "{r}");
    assert_eq!(r["shown"]["toast"], "Saved", "{r}");
    // AI off turns scheduled runs off in the same save.
    let r = &run["set_off_while_on"];
    assert_eq!(r["off"]["schedule"], "false", "{r}");
    assert_eq!(r["off"]["scheduleDisabled"], true, "{r}");
    assert_eq!(r["saves"][0]["settings"]["schedule"], "off", "{r}");
    // Text goes to the core as typed; its error names the key.
    let r = &run["set_error"];
    assert_eq!(r["saves"][0]["settings"]["interval_min"], "abc", "{r}");
    assert_eq!(r["shown"]["open"], true, "{r}");
    assert_eq!(r["shown"]["values"][0], "abc", "the typed value stays: {r}");
    for place in ["error", "toast"] {
        let text = r["shown"][place].as_str().unwrap();
        assert!(text.starts_with("Not saved: "), "{r}");
        assert!(text.contains("[ai] interval_min"), "{r}");
    }
    // A refused save after Confirm leaves the switch off.
    let r = &run["set_confirm_error"];
    assert_eq!(r["saves"], 1, "{r}");
    assert_eq!(r["shown"]["schedule"], "false", "{r}");
    // The toast names the file shortly; the dialog keeps the whole path.
    assert_eq!(
        r["shown"]["toast"], "Not saved: cannot write config.toml: Permission denied (os error 13)",
        "{r}"
    );
    let path = r["shown"]["readOnly"][0].as_str().unwrap();
    assert_eq!(
        r["shown"]["error"],
        format!("Not saved: cannot write {path}: Permission denied (os error 13)"),
        "{r}"
    );
}

#[test]
fn a_changed_or_unreadable_file_is_read_again() {
    let Some(run) = run_settings_page() else {
        return;
    };
    let r = &run["set_outside"];
    assert_eq!(r["gets"], 2, "{r}");
    assert_eq!(r["refused"]["values"][0], "15", "the file's value: {r}");
    assert!(
        r["refused"]["error"]
            .as_str()
            .unwrap()
            .contains("changed outside"),
        "{r}"
    );
    assert_eq!(r["saves"][1]["base"], "newer", "{r}");
    let r = &run["set_unreadable"];
    assert_eq!(r["failed"]["form"], false, "{r}");
    assert_eq!(r["failed"]["body"], false, "no empty band: {r}");
    assert_eq!(r["retried"]["body"], true, "{r}");
    assert_eq!(r["failed"]["save"], false, "{r}");
    assert_eq!(r["failed"]["retry"], true, "{r}");
    let error = r["failed"]["error"].as_str().unwrap();
    assert!(error.contains("cannot read /x/config.toml"), "{r}");
    assert!(error.contains("Try again"), "{r}");
    assert_eq!(r["retried"]["form"], true, "{r}");
    assert_eq!(r["retried"]["retry"], false, "{r}");
    assert_eq!(r["retried"]["error"], "", "{r}");
    assert_eq!(r["gets"], 2, "{r}");
    let r = &run["set_close"];
    assert_eq!(r["escaped"]["open"], false, "{r}");
    assert_eq!(
        r["reopened"]["values"][0], "60",
        "the draft is dropped: {r}"
    );
    assert_eq!(r["closed"]["open"], false, "{r}");
    assert_eq!(r["saves"], 0, "{r}");
}

#[test]
fn the_page_has_the_settings_gear_dialog_and_notice() {
    let html = std::str::from_utf8(asset("/").unwrap().1).unwrap();
    for part in [
        r#"id="settings-open" aria-label="Settings""#,
        r#"id="set-dialog" role="dialog" aria-modal="true""#,
        r#"id="ask-dialog" role="alertdialog" aria-modal="true""#,
        ">Organize automatically<",
        "Turn on AI first.",
        // The tray timer is a later task: the caption claims no run.
        "Saves your choice for timed AI runs. The timer is not built yet.",
        "Run after each save",
        ">Confirm</button>",
        ">Cancel</button>",
        r#"<script src="settings.js"></script>"#,
    ] {
        assert!(html.contains(part), "index.html lacks {part}");
    }
    // `command` and the account folder are text, never inputs.
    for id in ["set-command", "set-account"] {
        assert!(html.contains(&format!(r#"<dd id="{id}">"#)), "{id}");
    }
}

// ---------------------------------------------------------------------------------------------
// HUN-291: `updated` beside `created` (FR-046)
// ---------------------------------------------------------------------------------------------

const CHANGED: &str = "20261119-070000-ed01";
const UNTOUCHED: &str = "20261119-060000-ed02";

/// One note edited two hours before the frozen clock (its stamp written in UTC), one never edited.
fn edited_pair(sb: &Sandbox) {
    sb.note(
        CHANGED,
        "2026-11-19T07:00:00+07:00",
        "updated: \"2026-11-19T01:00:00+00:00\"\ntags: []\n",
        "changed note",
    );
    sb.note(
        UNTOUCHED,
        "2026-11-19T06:00:00+07:00",
        "tags: []\n",
        "kept note",
    );
}

#[test]
fn hun291_board_and_detail_carry_updated_only_on_the_edited_note() {
    let sb = Sandbox::new("edited-keys");
    edited_pair(&sb);
    let store = sb.store();
    let board = call(&store, &json!({"cmd": "board"}));
    assert_eq!(ids(&board), [CHANGED, UNTOUCHED], "newest `created` first");
    let detail = |id: &str| call(&store, &json!({"cmd": "detail", "id": id}));
    for note in [&board["notes"][0], &detail(CHANGED)] {
        assert_eq!(note["updated"], "2026-11-19T01:00:00+00:00", "{note}");
        assert_eq!(note["edited"], "2h ago", "{note}");
        assert_eq!(note["created"], "2026-11-19T07:00:00+07:00", "{note}");
    }
    // The full stamp, in the store's zone.
    assert_eq!(detail(CHANGED)["edited_at"], "2026-11-19 08:00:00 +07:00");
    for note in [&board["notes"][1], &detail(UNTOUCHED)] {
        for key in ["updated", "edited", "edited_at"] {
            assert!(note.get(key).is_none(), "{key} in {note}");
        }
    }
}

#[test]
fn hun291_an_updated_before_created_shows_the_stamp_in_the_window() {
    let sb = Sandbox::new("edited-clock");
    sb.note(
        CHANGED,
        "2026-11-19T07:00:00+07:00",
        "updated: \"2026-11-19T06:00:00+07:00\"\ntags: []\n",
        "clock set back",
    );
    let store = sb.store();
    let board = call(&store, &json!({"cmd": "board"}));
    assert_eq!(board["notes"][0]["edited"], "2026-11-19 06:00", "{board}");
    let detail = call(&store, &json!({"cmd": "detail", "id": CHANGED}));
    assert_eq!(detail["edited_at"], "2026-11-19 06:00:00 +07:00");
}

#[test]
fn hun291_the_windows_edit_and_add_as_tags_show_edited_just_now() {
    let sb = Sandbox::new("edited-actions");
    acted(&sb);
    let store = sb.store();
    let before = call(&store, &json!({"cmd": "detail", "id": ACTED}));
    assert!(before.get("edited").is_none(), "{before}");
    let out = call(&store, &json!({"cmd": "accept_ai", "id": ACTED}));
    assert_eq!(out["edited"], "just now", "{out}");

    let sb = Sandbox::new("edited-edit");
    acted(&sb);
    let store = sb.store();
    let request = edit_request(&store, ACTED, "new words", json!(["y"]));
    let out = call(&store, &request);
    assert_eq!(out["edited"], "just now", "{out}");
}

#[test]
fn hun291_the_page_shows_edited_on_the_row_and_the_full_stamp_in_the_detail() {
    let sb = Sandbox::new("edited-page");
    edited_pair(&sb);
    let store = sb.store();
    let board = call(&store, &json!({"cmd": "board"}));
    let mut notes = serde_json::Map::new();
    for id in ids(&board) {
        notes.insert(
            id.to_string(),
            call(&store, &json!({"cmd": "detail", "id": id})),
        );
    }
    let Some(run) = run_page_with(&board, Some(&json!({ "notes": notes }))) else {
        return;
    };
    let r = &run["edited"];
    assert_eq!(
        r["rows"],
        json!(["edited 2h ago · no project", "no project"]),
        "{r}"
    );
    assert_eq!(
        r["context"][CHANGED], "no project · today 07:00 · edited 2026-11-19 08:00:00 +07:00",
        "{r}"
    );
    assert_eq!(r["context"][UNTOUCHED], "no project · today 06:00", "{r}");
}

// ---------------------------------------------------------------------------------------------
// HUN-304: the layout fits every allowed window size (FR-042)
// ---------------------------------------------------------------------------------------------

/// The declarations of the first rule of `css` that starts a line with `selector {`.
fn css_rule<'a>(css: &'a str, selector: &str) -> &'a str {
    let head = format!("\n{selector} {{");
    let at = css
        .find(&head)
        .unwrap_or_else(|| panic!("app.css lacks the rule {selector}"));
    let body = &css[at + head.len()..];
    &body[..body.find('}').expect("end of rule")]
}

#[test]
fn hun304_a_large_screen_shows_a_centred_container() {
    let css = std::str::from_utf8(asset("/app.css").unwrap().1).unwrap();
    let main = css_rule(css, ".main");
    for must in ["max-width: 1440px", "margin: 0 auto"] {
        assert!(main.contains(must), ".main lacks {must}: {main}");
    }
    // The header's contents and the toast stay on the container's edges (720 - 28 = 692).
    let header = css_rule(css, ".header");
    assert!(
        header.contains("padding: 10px max(28px, calc(50% - 692px))"),
        "{header}"
    );
    let toast = css_rule(css, ".toast");
    assert!(
        toast.contains("right: max(24px, calc(50% - 692px))"),
        "{toast}"
    );
}

#[test]
fn hun304_every_allowed_width_shows_the_list_and_the_panel_side_by_side() {
    let css = std::str::from_utf8(asset("/app.css").unwrap().1).unwrap();
    let at = css
        .find("@media (min-width: 760px) {")
        .expect("no two-column block");
    let block = &css[at..at + css[at..].find("\n}\n").expect("end of block")];
    for must in [
        ".columns { flex-wrap: nowrap; }",
        ".notes { flex: 1 1 0; }",
        "flex: 0 0 clamp(320px, calc(100vw - 620px), 380px)",
        "position: sticky",
        "top: 16px",
        "max-height: calc(100vh - 32px)",
        "overflow-y: auto",
    ] {
        assert!(block.contains(must), "the two-column block lacks {must}");
    }
    // The block comes after the base rules, or they would win over it.
    assert!(at > css.find("\n.detail {").expect("base .detail rule"));
    // The smallest window on a large screen is inside the two-column range.
    assert!(dashboard_sizes(None).1 .0 >= 760.0);
    // A long word in the panel wraps instead of scrolling the panel sideways.
    assert!(css.contains(".tag, .ai-tag, .ai-title { overflow-wrap: anywhere; }"));
}

#[test]
fn hun304_a_newly_picked_note_starts_at_the_top_of_the_panel() {
    let app = std::str::from_utf8(asset("/app.js").unwrap().1).unwrap();
    let at = app
        .find("function select(id, reveal)")
        .expect("select(id, reveal)");
    let select = &app[at..at + app[at..].find("\n  }\n").expect("end of select")];
    // Only for another note: a board refresh selects the same one and keeps its place.
    let reset = select
        .find("if (state.sel !== id) $('detail').scrollTop = 0;")
        .expect("select does not reset the panel's scroll");
    assert!(reset < select.find("state.sel = id;").expect("state.sel = id"));
}

#[test]
fn hun304_the_window_is_never_larger_than_the_screen() {
    let usual = ((1280.0, 820.0), (800.0, 560.0));
    for (screen, expected) in [
        (None, usual),
        (Some((3440.0, 1440.0)), usual),
        (Some((1920.0, 1080.0)), usual),
        (Some((1360.0, 900.0)), usual),
        (Some((1366.0, 768.0)), ((1280.0, 688.0), (800.0, 560.0))),
        (Some((1280.0, 720.0)), ((1200.0, 640.0), (800.0, 560.0))),
        (Some((800.0, 600.0)), ((720.0, 520.0), (720.0, 520.0))),
        (Some((640.0, 480.0)), ((560.0, 400.0), (560.0, 400.0))),
        // A size that cannot be a screen keeps the usual sizes.
        (Some((0.0, 0.0)), usual),
        (Some((80.0, 80.0)), usual),
        (Some((f64::NAN, f64::NAN)), usual),
    ] {
        assert_eq!(dashboard_sizes(screen), expected, "screen {screen:?}");
    }
}

#[test]
fn hun304_a_row_click_never_scrolls_the_page_to_a_panel_that_stays_in_view() {
    // A stuck sticky panel has a larger `offsetTop` than the list, so the HUN-284 test alone
    // reads it as wrapped and moves the page on each click.
    let app = std::str::from_utf8(asset("/app.js").unwrap().1).unwrap();
    let at = app.find("function revealDetail(").expect("revealDetail");
    let body = &app[at..at + app[at..].find("\n  }\n").expect("end of revealDetail")];
    let stays = body
        .find("if (getComputedStyle(aside).position === 'sticky') return;")
        .expect("revealDetail does not leave a sticky panel alone");
    assert!(stays < body.find("scrollIntoView(").expect("scrollIntoView"));
}
