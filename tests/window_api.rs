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
use easy_note_mcp::surfaces::window::{allowed_url, api::handle, asset, ipc, reply_script, CSP};
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
    assert!(scanned >= 3, "index.html, app.js, app.css");
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
