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
    allowed_url,
    api::{handle, handle_with},
    asset, initial_note_script, ipc, ipc_from, ipc_from_with, ipc_with, reply_script, CSP, REFRESH,
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
            let mut request = json!({"cmd": cmd, "text": "x", "tags": [], "base": "x"});
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
    let app = std::str::from_utf8(asset("/app.js").unwrap().1).unwrap();
    let keys = js_function(app, "capKey");
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
    let save = js_function(app, "saveCapture");
    // AC 4: empty or whitespace-only text sends no request.
    let guard = save
        .find("if (!text.trim()) return;")
        .expect("no empty guard");
    let ask = save
        .find("easyNoteIpc.add(text, keepContext)")
        .expect("no add request");
    assert!(guard < ask, "the empty guard comes after the request");
    // AC 5: close, saved toast, board reload, select the new note. An error keeps the dialog.
    for must in [
        "closeCapture()",
        "'✓ noted'",
        "loadBoard()",
        "select(d.id)",
        "'could not save: ' + err.message",
    ] {
        assert!(save.contains(must), "saveCapture lacks {must}");
    }
    let open = js_function(app, "openCapture");
    assert!(open.contains("easyNoteIpc.context()"), "{open}");
    assert!(open.contains(".focus()"), "{open}");
    // A click on the backdrop keeps focus in the text box, so the keys above still reach capKey.
    assert!(app.contains("$('cap').addEventListener('mousedown'"));
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
    let base = call(store, &json!({"cmd": "detail", "id": id}))["text"].clone();
    json!({"cmd": "edit", "id": id, "text": text, "tags": tags, "base": base})
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
        &json!({"cmd": "edit", "id": ACTED, "text": base, "tags": ["y"], "base": base}),
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
    let ok =
        |id: &str, b: &Value| json!({"cmd": "edit", "id": id, "text": "x", "tags": [], "base": b});
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
    let more =
        json!({"roles": {"editable": ACTED, "inbox": "inbox/p", "other": other}, "notes": notes});
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
        json!(["Add as tags", "Edit", "Copy id", "Move to trash"]),
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
                "text": "New text #Web\nsecond line", "tags": ["#y", "new"], "base": r["base"]}]),
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
    assert_eq!(r["buttons"], json!(["Copy id", "Move to trash"]), "{r}");
    assert_eq!(r["readOnly"], "Inbox notes are read-only.", "{r}");
}
