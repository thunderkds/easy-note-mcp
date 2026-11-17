//! HUN-262 oracle: Claude detail. AI files beside the notes (`.easy-note/ai/`), the two MCP tools
//! `note_pending_enrich` / `note_enrich`, the `enrich_notes` prompt, the AI detail on read,
//! search and stats, and the CLI `ai rm` / `tag --accept-ai` (FR-028–FR-032, NFR-012, DDR 0004).
//!
//! The locked oracle is `note_files_stay_byte_identical_around_every_ai_operation` (AC 4): the
//! sha256 of every file under the notes dir, except `.easy-note/ai/`, is the same before and after
//! every AI read and write.
//!
//! MCP calls run in process (an rmcp client and `EasyNote::with_store` over a duplex pipe); CLI
//! calls run the real binary with a temp HOME and EASY_NOTE_DIR. No test touches the real ones.
//! No literal whole secret token lives here: the value is built with `concat!`.

use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use easy_note_mcp::core::format::parse;
use easy_note_mcp::core::store::Store;
use easy_note_mcp::surfaces::mcp::EasyNote;
use rmcp::model::{
    CallToolRequestParams, ClientCapabilities, ClientConfig, GetPromptRequestParams, Implementation,
};
use rmcp::service::RunningService;
use rmcp::{RoleClient, ServiceExt};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const BIN: &str = env!("CARGO_BIN_EXE_easy-note");
const GITHUB: &str = concat!("gh", "p_EXAMPLEexampleEXAMPLEexampleEXAMPLE123456");
const LABEL: &str = "AI detail (not your words)";

// ---------------------------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------------------------

struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let sb = Sandbox {
            dir: tempfile::Builder::new()
                .prefix(&format!("easy-note-ai-{name}-"))
                .tempdir()
                .unwrap(),
        };
        std::fs::create_dir_all(sb.home()).unwrap();
        sb
    }
    fn root(&self) -> &Path {
        self.dir.path()
    }
    fn home(&self) -> PathBuf {
        self.root().join("home")
    }
    fn notes(&self) -> PathBuf {
        self.root().join("notes")
    }
    fn store(&self) -> Store {
        Store::system(self.notes())
    }
    fn ai_dir(&self) -> PathBuf {
        self.notes().join(".easy-note/ai")
    }
    fn trash_ai(&self) -> PathBuf {
        self.notes().join(".easy-note/trash/ai")
    }
    /// One note written by the store, as the CLI would.
    fn add(&self, text: &str) -> String {
        self.store().add(text, "cli", &[], &[]).unwrap().id
    }
    fn note_path(&self, id: &str) -> PathBuf {
        self.notes()
            .join(&id[..4])
            .join(&id[4..6])
            .join(format!("{id}.md"))
    }
}

fn git(cwd: &Path, args: &[&str]) -> String {
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
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// A repo with one commit; returns its folder and `git rev-parse --short HEAD`.
fn repo_with_commit(root: &Path) -> (PathBuf, String) {
    let repo = root.join("proj");
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "first"]);
    let short = git(&repo, &["rev-parse", "--short", "HEAD"]);
    (repo, short)
}

fn cli_command(sb: &Sandbox, args: &[&str]) -> Command {
    let mut cmd = Command::new(BIN);
    cmd.args(args)
        .env("HOME", sb.home())
        .env("EASY_NOTE_DIR", sb.notes())
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .current_dir(sb.home())
        .stdin(Stdio::null());
    cmd
}

fn cli(sb: &Sandbox, args: &[&str]) -> Output {
    cli_command(sb, args).output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Stdout of a CLI run that must exit 0.
fn ok_cli(sb: &Sandbox, args: &[&str]) -> String {
    let out = cli(sb, args);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{args:?}: {}",
        text(&out.stderr)
    );
    text(&out.stdout)
}

/// sha256 of every file under the notes dir, except `.easy-note/ai/`, by relative path.
fn note_tree(notes: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(dir: &Path, root: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            let rel = path.strip_prefix(root).unwrap().to_path_buf();
            if rel.starts_with(".easy-note/ai") {
                continue;
            }
            if path.is_dir() {
                walk(&path, root, out);
            } else {
                out.insert(rel, Sha256::digest(std::fs::read(&path).unwrap()).to_vec());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(notes, notes, &mut out);
    out
}

/// The `.json` file names in a folder, sorted.
fn json_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.ends_with(".json"))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// `sha256:<hex>` of the body of the note file as stored.
fn body_hash(path: &Path) -> String {
    let raw = std::fs::read_to_string(path).unwrap();
    let body = match parse(&raw) {
        Ok(note) => note.body,
        Err(_) => raw,
    };
    let hex: String = Sha256::digest(body.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("sha256:{hex}")
}

type Client = RunningService<RoleClient, ClientConfig>;

async fn connect(sb: &Sandbox) -> Client {
    let (server_io, client_io) = tokio::io::duplex(1 << 20);
    let server = EasyNote::with_store(sb.store());
    tokio::spawn(async move {
        let running = server.serve(server_io).await.expect("server starts");
        let _ = running.waiting().await;
    });
    let info = ClientConfig::new(
        ClientCapabilities::default(),
        Implementation::new("ai-test", "0"),
    );
    info.serve(client_io).await.expect("client connects")
}

/// `(is_error, first text block)` of one tool call.
async fn call(client: &Client, tool: &'static str, args: Value) -> (bool, String) {
    let params = CallToolRequestParams::new(tool).with_arguments(args.as_object().unwrap().clone());
    let result = client.call_tool(params).await.expect("tool call answers");
    let content = serde_json::to_value(&result.content).unwrap();
    let text = content[0]["text"].as_str().unwrap_or("").to_string();
    (result.is_error == Some(true), text)
}

async fn ok(client: &Client, tool: &'static str, args: Value) -> Value {
    let (is_error, text) = call(client, tool, args).await;
    assert!(!is_error, "{tool} failed: {text}");
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{tool} gave non-JSON ({e}): {text}"))
}

fn enrich_args(id: &str) -> Value {
    json!({
        "id": id,
        "title": "Firefox-only retry failure",
        "kind": "bug",
        "summary": "The retry helper times out after the second navigation, only on Firefox.",
        "tags_suggested": ["firefox", "flaky"],
        "model": "claude-test",
    })
}

async fn pending_ids(client: &Client) -> Vec<String> {
    let page = ok(client, "note_pending_enrich", json!({})).await;
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_string())
        .collect()
}

/// A stub `$EDITOR` that appends one line to the file it is given.
fn append_editor(sb: &Sandbox) -> PathBuf {
    let stub = sb.home().join("append.sh");
    std::fs::write(&stub, "#!/bin/sh\nprintf 'one more line\\n' >> \"$1\"\n").unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    stub
}

// ---------------------------------------------------------------------------------------------
// AC 4: the locked oracle
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn note_files_stay_byte_identical_around_every_ai_operation() {
    let sb = Sandbox::new("oracle");
    let mut ids: Vec<String> = ["retry bug on firefox", "idea: dark mode", "why is ci slow?"]
        .iter()
        .map(|t| sb.add(t))
        .collect();
    // A hand-edited token in a note file: the hash must be over the body as stored, not masked.
    let hand = sb.add("hand edited");
    let path = sb.note_path(&hand);
    let raw = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, format!("{raw}token {GITHUB}\n")).unwrap();
    ids.push(hand);
    // An inbox note: its AI file is ours, the inbox file is never touched.
    std::fs::create_dir_all(sb.notes().join("inbox")).unwrap();
    std::fs::write(
        sb.notes().join("inbox/20261101-090000.txt"),
        "phone note #mobile\n",
    )
    .unwrap();
    ids.push("inbox/20261101-090000".to_string());

    let before = note_tree(&sb.notes());
    let client = connect(&sb).await;

    let mut listed = pending_ids(&client).await;
    listed.sort();
    let mut want = ids.clone();
    want.sort();
    assert_eq!(listed, want);
    for id in &ids {
        let answer = ok(&client, "note_enrich", enrich_args(id)).await;
        assert_eq!(answer["id"], json!(id));
        assert!(answer["based_on"].as_str().unwrap().starts_with("sha256:"));
    }
    assert_eq!(json_names(&sb.ai_dir()).len(), 5);
    assert!(pending_ids(&client).await.is_empty());
    for id in &ids {
        ok(&client, "note_read", json!({"id": id})).await;
    }
    ok(&client, "note_search", json!({"query": "firefox"})).await;
    ok(&client, "note_search", json!({"query": "navigation"})).await;
    ok(&client, "note_stats", json!({})).await;
    ok(&client, "note_list", json!({})).await;
    for id in &ids {
        ok_cli(&sb, &["show", id]);
    }
    ok_cli(&sb, &["search", "navigation"]);
    ok_cli(&sb, &["stats"]);
    ok_cli(&sb, &["stats", "--json"]);
    ok_cli(&sb, &["ai", "rm", &ids[0]]);
    // Failing calls write nothing either.
    call(
        &client,
        "note_enrich",
        json!({"id": "nope", "title": "t", "kind": "bug",
        "summary": "s", "tags_suggested": [], "model": "m"}),
    )
    .await;
    call(&client, "note_pending_enrich", json!({"limit": 21})).await;

    assert_eq!(note_tree(&sb.notes()), before);
    assert_eq!(json_names(&sb.ai_dir()).len(), 4);
}

// ---------------------------------------------------------------------------------------------
// AC 2: note_pending_enrich
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn pending_lists_newest_first_with_the_fields_and_masks_every_string() {
    let sb = Sandbox::new("pending");
    let (repo, _) = repo_with_commit(sb.root());
    let old = sb.add("older note");
    // Another second, so the second id sorts after the first.
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let found = easy_note_mcp::core::context::capture(Some("T7"), Some(&repo), &[]);
    let mut fields = found.fields();
    fields.push(("session", Some("sess-1")));
    let new = sb
        .store()
        .add(&format!("newer note {GITHUB}"), "hook", &[], &fields)
        .unwrap()
        .id;

    let client = connect(&sb).await;
    let page = ok(&client, "note_pending_enrich", json!({})).await;
    let items = page["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"], json!(new));
    assert_eq!(items[1]["id"], json!(old));
    let first = items[0].as_object().unwrap();
    let mut keys: Vec<&str> = first.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(keys, ["created", "id", "repo", "session", "task", "text"]);
    assert_eq!(first["repo"], "proj");
    assert_eq!(first["task"], "T7");
    assert_eq!(first["session"], "sess-1");
    assert!(first["text"].as_str().unwrap().starts_with("newer note"));
    // absent keys are left out
    let second = items[1].as_object().unwrap();
    for key in ["repo", "task", "session"] {
        assert!(!second.contains_key(key), "{key}");
    }
    assert!(!page.to_string().contains(GITHUB));
}

#[tokio::test]
async fn pending_limit_defaults_to_20_and_more_is_refused() {
    let sb = Sandbox::new("limit");
    for n in 0..22 {
        sb.add(&format!("note {n}"));
    }
    let client = connect(&sb).await;
    let page = ok(&client, "note_pending_enrich", json!({})).await;
    assert_eq!(page["items"].as_array().unwrap().len(), 20);
    let page = ok(&client, "note_pending_enrich", json!({"limit": 3})).await;
    assert_eq!(page["items"].as_array().unwrap().len(), 3);
    for bad in [21, 0] {
        let (is_error, message) = call(&client, "note_pending_enrich", json!({"limit": bad})).await;
        assert!(is_error, "{bad}");
        assert!(
            message.contains("limit") && message.contains("20"),
            "{message}"
        );
    }
}

#[tokio::test]
async fn a_trashed_note_is_not_pending_and_cannot_be_enriched() {
    let sb = Sandbox::new("trashed");
    let id = sb.add("going away");
    sb.store().rm(&id).unwrap();
    let client = connect(&sb).await;
    assert!(pending_ids(&client).await.is_empty());
    let (is_error, message) = call(&client, "note_enrich", enrich_args(&id)).await;
    assert!(is_error, "{message}");
    assert!(!sb.ai_dir().exists());
}

// ---------------------------------------------------------------------------------------------
// AC 3: note_enrich
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn enrich_writes_the_ai_file_shape_with_server_set_hash_and_time() {
    let sb = Sandbox::new("shape");
    let id = sb.add("retry bug on firefox");
    let client = connect(&sb).await;
    // based_on and at are not arguments: values a caller sends are ignored.
    let mut args = enrich_args(&id);
    args["based_on"] = json!("sha256:fake");
    args["at"] = json!("1999-01-01T00:00:00+00:00");
    args["tags_suggested"] = json!(["#Firefox", "flaky"]);
    let answer = ok(&client, "note_enrich", args).await;
    let hash = body_hash(&sb.note_path(&id));
    assert_eq!(answer, json!({"id": id, "based_on": hash}));

    let file = sb.ai_dir().join(format!("{id}.json"));
    let saved: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    let mut keys: Vec<&str> = saved
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "at",
            "based_on",
            "easy-note-ai",
            "id",
            "kind",
            "model",
            "summary",
            "tags_suggested",
            "title"
        ]
    );
    assert_eq!(saved["easy-note-ai"], 1);
    assert_eq!(saved["id"], json!(id));
    assert_eq!(saved["kind"], "bug");
    assert_eq!(saved["model"], "claude-test");
    assert_eq!(saved["tags_suggested"], json!(["firefox", "flaky"]));
    assert_eq!(saved["based_on"], json!(hash));
    let at = saved["at"].as_str().unwrap();
    assert!(chrono::DateTime::parse_from_rfc3339(at).is_ok(), "{at}");
    assert!(!at.starts_with("1999"), "{at}");
}

#[tokio::test]
async fn over_cap_fields_are_refused_naming_the_field_and_limit_never_the_value() {
    let sb = Sandbox::new("caps");
    let id = sb.add("a note");
    let client = connect(&sb).await;
    let cases = [
        ("title", json!("Q".repeat(121)), "120"),
        ("summary", json!("Z".repeat(601)), "600"),
        (
            "tags_suggested",
            json!(["aa", "bb", "cc", "dd", "ee", "ff"]),
            "5",
        ),
        ("kind", json!("note"), "bug"),
    ];
    for (field, value, limit) in cases {
        let mut args = enrich_args(&id);
        args[field] = value.clone();
        let (is_error, message) = call(&client, "note_enrich", args).await;
        assert!(is_error, "{field}: {message}");
        assert!(message.contains(field), "{field}: {message}");
        assert!(message.contains(limit), "{field}: {message}");
        let shown = match &value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        assert!(!message.contains(&shown), "{field} value echoed: {message}");
        assert!(!message.contains("QQQQ") && !message.contains("ZZZZ") && !message.contains("ff"));
    }
    // A tag that is not a tag is refused by position, not by value.
    let mut args = enrich_args(&id);
    args["tags_suggested"] = json!(["ok", "not a tag!"]);
    let (is_error, message) = call(&client, "note_enrich", args).await;
    assert!(is_error && message.contains("tags_suggested"), "{message}");
    assert!(!message.contains("not a tag!"), "{message}");
    assert!(json_names(&sb.ai_dir()).is_empty());
}

#[tokio::test]
async fn unknown_malformed_and_path_ids_are_refused_and_nothing_is_written() {
    let sb = Sandbox::new("ids");
    sb.add("a note");
    let client = connect(&sb).await;
    for id in [
        "20261101-000000-dead",
        "nope",
        "../x",
        "inbox/../../y",
        "inbox/missing",
        "20261101-000000-dead/../../z",
    ] {
        let (is_error, message) = call(&client, "note_enrich", enrich_args(id)).await;
        assert!(is_error, "{id}: {message}");
    }
    assert!(json_names(&sb.ai_dir()).is_empty());
    let mut all = vec![];
    for entry in walkdir(sb.root()) {
        all.push(entry);
    }
    assert!(all
        .iter()
        .all(|p| !p.ends_with("x.json") && !p.ends_with("y.json")));
}

fn walkdir(dir: &Path) -> Vec<PathBuf> {
    let mut out = vec![];
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walkdir(&path));
        } else {
            out.push(path);
        }
    }
    out
}

#[tokio::test]
async fn a_second_enrich_replaces_the_ai_file_and_strings_are_masked() {
    let sb = Sandbox::new("replace");
    let id = sb.add("a note");
    let client = connect(&sb).await;
    ok(&client, "note_enrich", enrich_args(&id)).await;
    let mut args = enrich_args(&id);
    args["title"] = json!(format!("second {GITHUB}"));
    args["summary"] = json!(format!("quotes {GITHUB}"));
    args["model"] = json!(format!("m {GITHUB}"));
    ok(&client, "note_enrich", args).await;
    assert_eq!(json_names(&sb.ai_dir()), [format!("{id}.json")]);
    let saved = std::fs::read_to_string(sb.ai_dir().join(format!("{id}.json"))).unwrap();
    assert!(saved.contains("second "), "{saved}");
    assert!(!saved.contains(GITHUB), "{saved}");
    // no temp file is left behind
    let left: Vec<PathBuf> = walkdir(&sb.ai_dir());
    assert_eq!(left.len(), 1, "{left:?}");
}

#[tokio::test]
async fn an_inbox_note_gets_an_inbox_ai_file_name() {
    let sb = Sandbox::new("inbox");
    std::fs::create_dir_all(sb.notes().join("inbox")).unwrap();
    std::fs::write(sb.notes().join("inbox/phone.md"), "from the phone\n").unwrap();
    let client = connect(&sb).await;
    let answer = ok(&client, "note_enrich", enrich_args("inbox/phone")).await;
    assert_eq!(answer["id"], "inbox/phone");
    assert_eq!(json_names(&sb.ai_dir()), ["inbox__phone.json"]);
}

// ---------------------------------------------------------------------------------------------
// AC 5: the enrich_notes prompt
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn the_one_prompt_is_enrich_notes_with_fixed_text() {
    let sb = Sandbox::new("prompt");
    let client = connect(&sb).await;
    let prompts = client.list_all_prompts().await.unwrap();
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].name, "enrich_notes");
    assert!(prompts[0].arguments.as_ref().is_none_or(|a| a.is_empty()));

    let get = || async {
        let got = client
            .get_prompt(GetPromptRequestParams::new("enrich_notes"))
            .await
            .unwrap();
        serde_json::to_value(&got.messages).unwrap().to_string()
    };
    let first = get().await;
    assert_eq!(first, get().await);
    for needle in [
        "one subagent",
        "note_pending_enrich",
        "session",
        "10 minutes before",
        "40",
        "repo",
        "exactly one",
        "no context claim",
        "never quote",
        "note_enrich",
    ] {
        assert!(first.contains(needle), "{needle} missing: {first}");
    }
}

// ---------------------------------------------------------------------------------------------
// AC 6: AI detail on read, search and stats
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn show_and_note_read_carry_the_labelled_ai_detail() {
    let sb = Sandbox::new("read");
    let id = sb.add("retry bug on firefox");
    let plain = sb.add("no detail here");
    let client = connect(&sb).await;
    ok(&client, "note_enrich", enrich_args(&id)).await;

    let shown = ok_cli(&sb, &["show", &id]);
    assert!(shown.contains(LABEL), "{shown}");
    assert!(shown.contains("claude-test"), "{shown}");
    assert!(shown.contains("Firefox-only retry failure"), "{shown}");
    assert!(!shown.contains("(stale)"), "{shown}");
    assert!(!ok_cli(&sb, &["show", &plain]).contains(LABEL));

    let read = ok(&client, "note_read", json!({"id": id})).await;
    assert_eq!(read["ai"]["label"], LABEL);
    assert_eq!(read["ai"]["stale"], false);
    assert_eq!(read["ai"]["model"], "claude-test");
    assert_eq!(read["ai"]["kind"], "bug");
    assert!(read["ai"]["at"].is_string());
    let read = ok(&client, "note_read", json!({"id": plain})).await;
    assert!(read.get("ai").is_none(), "{read}");
}

#[tokio::test]
async fn search_matches_ai_text_and_marks_ai_only_hits() {
    let sb = Sandbox::new("search");
    let enriched = sb.add("retry bug");
    let other = sb.add("navigation in the note itself");
    let client = connect(&sb).await;
    ok(&client, "note_enrich", enrich_args(&enriched)).await;

    let page = ok(&client, "note_search", json!({"query": "navigation"})).await;
    let items = page["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    let by_id = |id: &str| items.iter().find(|i| i["id"] == json!(id)).unwrap().clone();
    assert_eq!(by_id(&enriched)["matched_in"], "ai");
    assert!(by_id(&other).get("matched_in").is_none());
    // tags_suggested and title match too
    let page = ok(&client, "note_search", json!({"query": "flaky"})).await;
    assert_eq!(page["items"][0]["id"], json!(enriched));
    let page = ok(&client, "note_search", json!({"query": "firefox-only"})).await;
    assert_eq!(page["items"][0]["matched_in"], "ai");

    let out = ok_cli(&sb, &["search", "navigation"]);
    let line = out.lines().find(|l| l.starts_with(&enriched)).unwrap();
    assert!(line.contains("[ai]"), "{out}");
    let line = out.lines().find(|l| l.starts_with(&other)).unwrap();
    assert!(!line.contains("[ai]"), "{out}");
}

#[tokio::test]
async fn stats_are_unchanged_except_the_enriched_count() {
    let sb = Sandbox::new("stats");
    let a = sb.add("one #x");
    sb.add("two #x");
    let client = connect(&sb).await;
    let before = ok(&client, "note_stats", json!({})).await;
    assert_eq!(before["enriched"], 0);
    let mut args = enrich_args(&a);
    args["tags_suggested"] = json!(["other"]);
    ok(&client, "note_enrich", args).await;
    let mut after = ok(&client, "note_stats", json!({})).await;
    assert_eq!(after["enriched"], 1);
    after["enriched"] = json!(0);
    assert_eq!(after, before, "AI fields fed a count");
    let out = ok_cli(&sb, &["stats"]);
    assert!(out.contains("enriched: 1"), "{out}");
    // a stale AI file is not enriched
    let path = sb.note_path(&a);
    let raw = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, format!("{raw}changed\n")).unwrap();
    assert_eq!(ok(&client, "note_stats", json!({})).await["enriched"], 0);
}

// ---------------------------------------------------------------------------------------------
// AC 7: stale after edit; rm / restore / trash; ai rm; tag --accept-ai
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn edit_makes_the_detail_stale_and_the_note_pending_again() {
    let sb = Sandbox::new("stale");
    let id = sb.add("retry bug");
    let client = connect(&sb).await;
    ok(&client, "note_enrich", enrich_args(&id)).await;
    assert!(pending_ids(&client).await.is_empty());

    let editor = append_editor(&sb);
    let out = cli_command(&sb, &["edit", &id])
        .env("EDITOR", &editor)
        .env_remove("VISUAL")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));

    assert_eq!(pending_ids(&client).await, std::slice::from_ref(&id));
    let shown = ok_cli(&sb, &["show", &id]);
    assert!(shown.contains("(stale)"), "{shown}");
    let read = ok(&client, "note_read", json!({"id": id})).await;
    assert_eq!(read["ai"]["stale"], true);
}

#[tokio::test]
async fn rm_and_restore_carry_the_ai_file() {
    let sb = Sandbox::new("trash");
    let id = sb.add("a note");
    let client = connect(&sb).await;
    ok(&client, "note_enrich", enrich_args(&id)).await;

    ok_cli(&sb, &["rm", &id]);
    assert!(json_names(&sb.ai_dir()).is_empty());
    assert_eq!(json_names(&sb.trash_ai()), [format!("{id}.md.json")]);
    ok_cli(&sb, &["restore", &id]);
    assert_eq!(json_names(&sb.ai_dir()), [format!("{id}.json")]);
    assert!(json_names(&sb.trash_ai()).is_empty());
    assert!(
        pending_ids(&client).await.is_empty(),
        "restored detail is fresh"
    );

    // an inbox note: the trash name of the AI file is the note's trash name plus .json
    std::fs::create_dir_all(sb.notes().join("inbox")).unwrap();
    std::fs::write(sb.notes().join("inbox/phone.txt"), "phone\n").unwrap();
    ok(&client, "note_enrich", enrich_args("inbox/phone")).await;
    ok_cli(&sb, &["rm", "inbox/phone"]);
    let trashed: Vec<String> = std::fs::read_dir(sb.notes().join(".easy-note/trash"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("inbox__"))
        .collect();
    assert_eq!(trashed.len(), 1);
    assert_eq!(json_names(&sb.trash_ai()), [format!("{}.json", trashed[0])]);
    ok_cli(&sb, &["restore", "inbox/phone"]);
    assert!(json_names(&sb.ai_dir()).contains(&"inbox__phone.json".to_string()));
}

#[tokio::test]
async fn rm_keeps_both_when_a_trashed_ai_file_is_in_the_way() {
    let sb = Sandbox::new("clash");
    let id = sb.add("a note");
    let client = connect(&sb).await;
    ok(&client, "note_enrich", enrich_args(&id)).await;
    std::fs::create_dir_all(sb.trash_ai()).unwrap();
    let blocker = sb.trash_ai().join(format!("{id}.md.json"));
    std::fs::write(&blocker, "{}").unwrap();

    let out = cli(&sb, &["rm", &id]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(!sb.note_path(&id).exists(), "the note still moves");
    assert_eq!(std::fs::read_to_string(&blocker).unwrap(), "{}");
    assert_eq!(json_names(&sb.ai_dir()), [format!("{id}.json")]);
    assert!(
        text(&out.stderr).contains("AI detail"),
        "{}",
        text(&out.stderr)
    );
}

#[tokio::test]
async fn trash_empty_deletes_the_pair_by_the_note_mtime_and_orphans() {
    let sb = Sandbox::new("empty");
    let id = sb.add("a note");
    let client = connect(&sb).await;
    ok(&client, "note_enrich", enrich_args(&id)).await;
    ok_cli(&sb, &["rm", &id]);
    // the AI file is old, the note is new: nothing goes
    let ai = sb.trash_ai().join(format!("{id}.md.json"));
    let old = std::time::SystemTime::now() - std::time::Duration::from_secs(90 * 86_400);
    std::fs::File::options()
        .write(true)
        .open(&ai)
        .unwrap()
        .set_modified(old)
        .unwrap();
    let orphan = sb.trash_ai().join("20200101-000000-abcd.md.json");
    std::fs::write(&orphan, "{}").unwrap();
    let out = ok_cli(&sb, &["trash", "--empty", "--older-than", "30d"]);
    assert!(out.contains("deleted 0 notes"), "{out}");
    assert!(ai.exists());
    assert!(
        !orphan.exists(),
        "an AI file with no note in the trash goes"
    );
    // now the note is old: the pair goes, with one line
    let note = sb.notes().join(format!(".easy-note/trash/{id}.md"));
    std::fs::File::open(&note)
        .unwrap()
        .set_modified(old)
        .unwrap();
    let out = ok_cli(&sb, &["trash", "--empty", "--older-than", "30d"]);
    assert_eq!(out.matches("deleted ").count(), 2, "{out}");
    assert!(out.contains(&format!("deleted {id}")), "{out}");
    assert!(!note.exists() && !ai.exists());
}

#[tokio::test]
async fn ai_rm_deletes_only_the_ai_file() {
    let sb = Sandbox::new("airm");
    let id = sb.add("a note");
    let out = cli(&sb, &["ai", "rm", &id]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("no AI detail"),
        "{}",
        text(&out.stderr)
    );
    let client = connect(&sb).await;
    ok(&client, "note_enrich", enrich_args(&id)).await;
    let before = note_tree(&sb.notes());
    ok_cli(&sb, &["ai", "rm", &id]);
    assert!(json_names(&sb.ai_dir()).is_empty());
    assert_eq!(note_tree(&sb.notes()), before);
    assert_eq!(pending_ids(&client).await, [id]);
}

#[tokio::test]
async fn tag_accept_ai_adds_the_suggested_tags() {
    let sb = Sandbox::new("accept");
    let id = sb
        .store()
        .add("retry bug #old", "cli", &["old".to_string()], &[])
        .unwrap()
        .id;
    let out = cli(&sb, &["tag", &id, "--accept-ai"]);
    assert_eq!(out.status.code(), Some(1), "no AI detail yet");
    let client = connect(&sb).await;
    ok(&client, "note_enrich", enrich_args(&id)).await;
    ok_cli(&sb, &["tag", &id, "--accept-ai"]);
    let note = parse(&std::fs::read_to_string(sb.note_path(&id)).unwrap()).unwrap();
    assert_eq!(
        note.fields.get("tags"),
        Some(&easy_note_mcp::core::format::Value::List(vec![
            "old".into(),
            "firefox".into(),
            "flaky".into()
        ]))
    );
    // the body is unchanged, so the detail stays fresh
    assert!(pending_ids(&client).await.is_empty());
    // an inbox note stays read-only
    std::fs::create_dir_all(sb.notes().join("inbox")).unwrap();
    std::fs::write(sb.notes().join("inbox/p.md"), "phone\n").unwrap();
    ok(&client, "note_enrich", enrich_args("inbox/p")).await;
    let out = cli(&sb, &["tag", "inbox/p", "--accept-ai"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("read-only"),
        "{}",
        text(&out.stderr)
    );
}

// ---------------------------------------------------------------------------------------------
// AC 8: note_add(detail)
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn note_add_with_detail_writes_a_capture_ai_file() {
    let sb = Sandbox::new("detail");
    let client = connect(&sb).await;
    let added = ok(
        &client,
        "note_add",
        json!({"text": "retry bug on firefox", "cwd": sb.home(),
            "detail": format!("found while testing {GITHUB}")}),
    )
    .await;
    let id = added["id"].as_str().unwrap();
    assert_eq!(added["detail_saved"], true);
    let saved: Value = serde_json::from_str(
        &std::fs::read_to_string(sb.ai_dir().join(format!("{id}.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(saved["model"], "capture");
    assert_eq!(saved["kind"], "other");
    assert_eq!(saved["tags_suggested"], json!([]));
    assert_eq!(saved["title"], "retry bug on firefox");
    assert!(saved["summary"]
        .as_str()
        .unwrap()
        .starts_with("found while testing"));
    assert!(!saved.to_string().contains(GITHUB));
    assert_eq!(saved["based_on"], json!(body_hash(&sb.note_path(id))));
    // a fresh capture file is not pending
    assert!(pending_ids(&client).await.is_empty());
}

#[tokio::test]
async fn note_add_with_a_too_long_detail_writes_nothing() {
    let sb = Sandbox::new("toolong");
    let client = connect(&sb).await;
    let (is_error, message) = call(
        &client,
        "note_add",
        json!({"text": "x", "cwd": sb.home(), "detail": "D".repeat(601)}),
    )
    .await;
    assert!(is_error);
    assert!(
        message.contains("detail") && message.contains("600"),
        "{message}"
    );
    assert!(!message.contains("DDDD"), "{message}");
    assert!(!sb.notes().exists());
}

// ---------------------------------------------------------------------------------------------
// Hand-written AI files: masked decoded text (SC 5); bad files are no AI file
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn a_token_hidden_as_a_json_escape_is_masked_on_show_and_note_read() {
    let sb = Sandbox::new("escape");
    let id = sb.add("a note");
    let hash = body_hash(&sb.note_path(&id));
    std::fs::create_dir_all(sb.ai_dir()).unwrap();
    let escaped = format!("\\u0067{}", &GITHUB[1..]);
    std::fs::write(
        sb.ai_dir().join(format!("{id}.json")),
        format!(
            r#"{{"easy-note-ai": 1, "id": "{id}", "title": "t", "kind": "bug",
            "summary": "hidden {escaped}", "tags_suggested": [], "based_on": "{hash}",
            "model": "m", "at": "2026-11-15T09:00:00+07:00", "extra": "ignored"}}"#
        ),
    )
    .unwrap();
    let shown = ok_cli(&sb, &["show", &id]);
    assert!(shown.contains(LABEL) && shown.contains("hidden"), "{shown}");
    assert!(!shown.contains(GITHUB), "{shown}");
    assert!(!shown.contains("ignored"), "{shown}");
    let client = connect(&sb).await;
    let read = ok(&client, "note_read", json!({"id": id})).await;
    assert!(read["ai"]["summary"]
        .as_str()
        .unwrap()
        .starts_with("hidden"));
    assert!(!read.to_string().contains(GITHUB), "{read}");
    // search over the masked AI text: the token is not a hit, and never shown
    let page = ok(&client, "note_search", json!({"query": "hidden"})).await;
    assert!(!page.to_string().contains(GITHUB));
}

#[tokio::test]
async fn a_bad_ai_file_is_no_ai_file_on_every_read() {
    let sb = Sandbox::new("bad");
    let ids: Vec<String> = (0..4).map(|n| sb.add(&format!("note {n}"))).collect();
    let hash = |id: &str| body_hash(&sb.note_path(id));
    std::fs::create_dir_all(sb.ai_dir()).unwrap();
    let write = |id: &str, body: String| {
        std::fs::write(sb.ai_dir().join(format!("{id}.json")), body).unwrap()
    };
    write(&ids[0], "not json".into());
    write(
        &ids[1],
        format!(
            r#"{{"easy-note-ai": 1, "id": "x", "title": 5, "kind": "bug", "summary": "s",
            "tags_suggested": [], "based_on": "{}", "model": "m", "at": "a"}}"#,
            hash(&ids[1])
        ),
    );
    write(
        &ids[2],
        format!(
            r#"{{"easy-note-ai": 1, "id": "x", "title": "t", "kind": "bug", "summary": "{}",
            "tags_suggested": [], "based_on": "{}", "model": "m", "at": "a"}}"#,
            "s".repeat(700),
            hash(&ids[2])
        ),
    );
    write(
        &ids[3],
        format!(
            r#"{{"easy-note-ai": 1, "id": "x", "title": "t", "kind": "rant", "summary": "s",
            "tags_suggested": [], "based_on": "{}", "model": "m", "at": "a"}}"#,
            hash(&ids[3])
        ),
    );
    let client = connect(&sb).await;
    assert_eq!(pending_ids(&client).await.len(), 4);
    for id in &ids {
        assert!(!ok_cli(&sb, &["show", id]).contains(LABEL));
        assert!(ok(&client, "note_read", json!({"id": id}))
            .await
            .get("ai")
            .is_none());
    }
    assert_eq!(ok(&client, "note_stats", json!({})).await["enriched"], 0);
}

// ---------------------------------------------------------------------------------------------
// AC 1: commit on every capture path in a repo with a commit
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn add_note_add_and_quick_cwd_store_the_short_commit() {
    let sb = Sandbox::new("commit");
    let (repo, short) = repo_with_commit(sb.root());

    let out = cli_command(&sb, &["add", "from", "the", "cli"])
        .current_dir(&repo)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));

    let client = connect(&sb).await;
    ok(
        &client,
        "note_add",
        json!({"text": "from mcp", "cwd": repo}),
    )
    .await;

    // quick with a stub zenity on a PATH of stubs and git only
    let bin = sb.root().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let zenity = bin.join("zenity");
    std::fs::write(&zenity, "#!/bin/sh\nprintf 'from quick\\n'\n").unwrap();
    std::fs::set_permissions(&zenity, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(bin.join("notify-send"), "#!/bin/sh\n").unwrap();
    std::fs::set_permissions(
        bin.join("notify-send"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let real_git = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|d| d.join("git"))
        .find(|p| p.is_file())
        .unwrap();
    std::os::unix::fs::symlink(real_git, bin.join("git")).unwrap();
    let out = cli_command(&sb, &["quick", "--cwd", repo.to_str().unwrap()])
        .env("PATH", &bin)
        .env("DISPLAY", ":99")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));

    let notes: Vec<PathBuf> = walkdir(&sb.notes())
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    assert_eq!(notes.len(), 3, "{notes:?}");
    for path in notes {
        let note = parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(note.get_str("commit"), Some(short.as_str()), "{path:?}");
    }
}

#[test]
fn outside_a_repo_or_before_the_first_commit_there_is_no_commit() {
    let sb = Sandbox::new("nocommit");
    let empty = sb.root().join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    git(&empty, &["init", "-q", "-b", "main"]);
    for cwd in [sb.home(), empty] {
        let out = cli_command(&sb, &["add", "no", "commit"])
            .current_dir(&cwd)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    }
    let notes: Vec<PathBuf> = walkdir(&sb.notes())
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    assert_eq!(notes.len(), 2);
    let mut repos = 0;
    for path in notes {
        let note = parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(note.get_str("commit"), None);
        repos += usize::from(note.get_str("repo") == Some("empty"));
    }
    assert_eq!(repos, 1, "the empty repo keeps its repo field");
}
