//! HUN-258 oracle: the MCP surface. Five tools, no write on the read tools, FR-016 bounds,
//! cwd order, `127.0.0.1` only, and a smoke run of the real binary over stdio.
//!
//! Tool logic runs in process (an `rmcp` client and the server over a duplex pipe, the store
//! injected, so no env var is touched). The binary tests spawn `easy-note serve` with a temp HOME
//! and EASY_NOTE_DIR; they never touch the real ones (NFR-003).
//!
//! No literal whole secret token lives here (push protection): the value is built with `concat!`.

// MCP roots are deprecated in the SDK, but FR-004 still names them in the cwd order.
#![allow(deprecated)]

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use easy_note_mcp::core::context;
use easy_note_mcp::core::format::parse;
use easy_note_mcp::core::store::Store;
use easy_note_mcp::surfaces::mcp::EasyNote;
use rmcp::model::{
    CallToolRequestParams, ClientCapabilities, ClientConfig, Implementation, ListRootsResult, Root,
};
use rmcp::service::{RequestContext, RunningService};
use rmcp::transport::TokioChildProcess;
use rmcp::{ClientHandler, RoleClient, ServiceExt};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const BIN: &str = env!("CARGO_BIN_EXE_easy-note");
const TOOLS: [&str; 5] = [
    "note_add",
    "note_list",
    "note_read",
    "note_search",
    "note_stats",
];
const GITHUB: &str = concat!("gh", "p_EXAMPLEexampleEXAMPLEexampleEXAMPLE123456");

// ---------------------------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------------------------

struct Sandbox {
    root: PathBuf,
    home: PathBuf,
    notes: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let root =
            std::env::temp_dir().join(format!("easy-note-mcp-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let home = root.join("home");
        std::fs::create_dir_all(&home).unwrap();
        Sandbox {
            notes: root.join("notes"),
            home,
            root,
        }
    }

    fn store(&self) -> Store {
        Store::system(self.notes.clone())
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn git(cwd: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t.invalid"])
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn make_repo(root: &Path, name: &str, branch: &str) -> PathBuf {
    let repo = root.join(name);
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q", "-b", branch]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "first"]);
    repo
}

/// Every entry under `root`: a directory has no digest, a file has its SHA-256.
fn tree(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn walk(dir: &Path, root: &Path, out: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            let rel = path.strip_prefix(root).unwrap().to_path_buf();
            if path.is_dir() {
                out.insert(rel, None);
                walk(&path, root, out);
            } else {
                let digest = Sha256::digest(std::fs::read(&path).unwrap()).to_vec();
                out.insert(rel, Some(digest));
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

fn note_files(notes: &Path) -> Vec<PathBuf> {
    tree(notes)
        .into_iter()
        .filter(|(path, digest)| {
            digest.is_some()
                && path.extension().is_some_and(|e| e == "md")
                && !path.to_string_lossy().contains("/.")
        })
        .map(|(path, _)| notes.join(path))
        .collect()
}

struct TestClient {
    name: &'static str,
    /// `None`: the client does not advertise roots. `Some`: it does, and answers with these.
    roots: Option<Vec<String>>,
}

impl TestClient {
    fn plain(name: &'static str) -> TestClient {
        TestClient { name, roots: None }
    }
}

impl ClientHandler for TestClient {
    fn get_info(&self) -> ClientConfig {
        let caps = if self.roots.is_some() {
            ClientCapabilities::builder().enable_roots().build()
        } else {
            ClientCapabilities::default()
        };
        ClientConfig::new(caps, Implementation::new(self.name, "0"))
    }

    async fn list_roots(
        &self,
        _context: RequestContext<RoleClient>,
    ) -> Result<ListRootsResult, rmcp::ErrorData> {
        let roots = self.roots.clone().unwrap_or_default();
        Ok(ListRootsResult::new(
            roots.into_iter().map(Root::new).collect(),
        ))
    }
}

type Client = RunningService<RoleClient, TestClient>;

async fn connect(sb: &Sandbox, client: TestClient) -> Client {
    let (server_io, client_io) = tokio::io::duplex(1 << 20);
    let server = EasyNote::with_store(sb.store());
    tokio::spawn(async move {
        let running = server.serve(server_io).await.expect("server starts");
        let _ = running.waiting().await;
    });
    client.serve(client_io).await.expect("client connects")
}

/// `(is_error, first text block)` of one tool call.
async fn call<S: rmcp::service::Service<RoleClient>>(
    client: &RunningService<RoleClient, S>,
    tool: &'static str,
    args: Value,
) -> (bool, String) {
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

fn seed(sb: &Sandbox, texts: &[&str]) -> Vec<String> {
    texts
        .iter()
        .map(|t| {
            sb.store()
                .add(t, "cli", &["seed".to_string()], &[])
                .unwrap()
                .id
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// AC 1, 6: the tool list and the descriptions
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn exactly_the_five_tools_are_registered() {
    let sb = Sandbox::new("list");
    let client = connect(&sb, TestClient::plain("t")).await;
    let mut names: Vec<String> = client
        .list_all_tools()
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.name.to_string())
        .collect();
    names.sort();
    assert_eq!(names, TOOLS);
}

#[tokio::test]
async fn note_add_description_carries_the_consent_and_note_prefix_rules() {
    let sb = Sandbox::new("desc");
    let client = connect(&sb, TestClient::plain("t")).await;
    let tools = client.list_all_tools().await.unwrap();
    let add = tools.iter().find(|t| t.name == "note_add").unwrap();
    let text = add.description.as_deref().unwrap_or("");
    assert!(text.contains("only when the person asks"), "{text}");
    assert!(text.contains("note:") && text.contains("/note"), "{text}");
    for tool in tools.iter().filter(|t| t.name != "note_add") {
        let read_only = tool.annotations.as_ref().and_then(|a| a.read_only_hint);
        assert_eq!(
            read_only,
            Some(true),
            "{} is not marked read-only",
            tool.name
        );
    }
}

// ---------------------------------------------------------------------------------------------
// AC 2: the read tools leave the notes dir byte-identical
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn read_tools_leave_the_notes_dir_byte_identical() {
    let sb = Sandbox::new("noop");
    let ids = seed(&sb, &["alpha note", "beta note", "gamma note"]);
    // A hand-edited file with a token, and a file the parser cannot read.
    let path = note_files(&sb.notes).remove(0);
    let raw = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, format!("{raw}\nhand edit {GITHUB}\n")).unwrap();
    std::fs::create_dir_all(sb.notes.join("2026/11")).unwrap();
    std::fs::write(sb.notes.join("2026/11/20261101-000000-dead.md"), "garbage").unwrap();

    let before = tree(&sb.notes);
    let client = connect(&sb, TestClient::plain("t")).await;
    ok(&client, "note_list", json!({})).await;
    ok(&client, "note_list", json!({"limit": 1, "tag": "seed"})).await;
    ok(&client, "note_search", json!({"query": "note"})).await;
    ok(&client, "note_search", json!({"query": "hand edit"})).await;
    ok(&client, "note_stats", json!({})).await;
    for id in &ids {
        ok(&client, "note_read", json!({"id": id})).await;
    }
    // Failing reads write nothing either.
    call(&client, "note_read", json!({"id": "nope"})).await;
    call(&client, "note_search", json!({"query": "   "})).await;
    assert_eq!(tree(&sb.notes), before);
}

#[tokio::test]
async fn read_tools_do_not_create_a_missing_notes_dir() {
    let sb = Sandbox::new("nodir");
    assert!(!sb.notes.exists());
    let client = connect(&sb, TestClient::plain("t")).await;
    let list = ok(&client, "note_list", json!({})).await;
    assert_eq!(list["total"], 0);
    ok(&client, "note_search", json!({"query": "x"})).await;
    ok(&client, "note_stats", json!({})).await;
    call(&client, "note_read", json!({"id": "20261101-000000-dead"})).await;
    assert!(!sb.notes.exists(), "a read tool created the notes dir");
}

#[tokio::test]
async fn note_read_masks_a_hand_edited_token_and_leaves_the_file_alone() {
    let sb = Sandbox::new("mask");
    let ids = seed(&sb, &["plain text"]);
    let path = note_files(&sb.notes).remove(0);
    let raw = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        raw.replace("plain text", &format!("key {GITHUB} end")),
    )
    .unwrap();
    let before = tree(&sb.notes);

    let client = connect(&sb, TestClient::plain("t")).await;
    let (is_error, text) = call(&client, "note_read", json!({"id": ids[0]})).await;
    assert!(!is_error, "{text}");
    assert!(!text.contains(GITHUB), "token reached the client: {text}");
    assert!(text.contains("[masked:github_token]"), "{text}");
    let note: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(note["id"], ids[0]);
    assert!(note["body"].as_str().unwrap().contains("key [masked:"));
    assert_eq!(tree(&sb.notes), before);
}

// ---------------------------------------------------------------------------------------------
// AC 3: note_add writes exactly one file
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn note_add_writes_one_file_with_source_mcp_and_the_client_name() {
    let sb = Sandbox::new("add");
    let repo = make_repo(&sb.root, "proj", "fix/t120-firefox");
    let client = connect(&sb, TestClient::plain("codex-test")).await;
    assert!(note_files(&sb.notes).is_empty());

    let out = ok(
        &client,
        "note_add",
        json!({"text": "retry bug only on Firefox", "tags": ["flaky"], "cwd": repo}),
    )
    .await;

    let files = note_files(&sb.notes);
    assert_eq!(files.len(), 1, "{files:?}");
    let written = std::fs::read_to_string(&files[0]).unwrap();
    let note = parse(&written).unwrap();
    assert_eq!(note.get_str("source"), Some("mcp"));
    assert_eq!(note.get_str("client"), Some("codex-test"));
    assert_eq!(note.body, "retry bug only on Firefox\n");
    assert_eq!(out["repo"], "proj");
    assert_eq!(out["branch"], "fix/t120-firefox");
    assert_eq!(out["task"], "T120");
    assert_eq!(out["tags"], json!(["flaky"]));
    let id = out["id"].as_str().unwrap();
    assert!(files[0]
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with(id));
    assert_eq!(out["path"].as_str().unwrap(), files[0].to_str().unwrap());
    // Never a path inside the note (FR-004).
    assert!(!written.contains(sb.root.to_str().unwrap()));
}

#[tokio::test]
async fn note_add_masks_a_secret_in_the_text_the_tags_and_the_answer() {
    let sb = Sandbox::new("addmask");
    let client = connect(&sb, TestClient::plain("t")).await;
    let (is_error, text) = call(
        &client,
        "note_add",
        json!({"text": format!("deploy {GITHUB}"), "tags": [GITHUB]}),
    )
    .await;
    assert!(!is_error, "{text}");
    assert!(!text.contains(GITHUB), "{text}");
    for path in note_files(&sb.notes) {
        assert!(!std::fs::read_to_string(path).unwrap().contains(GITHUB));
    }
}

#[tokio::test]
async fn note_add_refuses_empty_whitespace_and_oversized_text_and_writes_nothing() {
    let sb = Sandbox::new("addbad");
    let client = connect(&sb, TestClient::plain("t")).await;
    for text in ["", "   \n\t ", &"x".repeat(20_001)] {
        let (is_error, message) = call(&client, "note_add", json!({"text": text})).await;
        assert!(
            is_error,
            "accepted {:?}",
            text.chars().take(5).collect::<String>()
        );
        assert!(!message.is_empty());
        assert!(note_files(&sb.notes).is_empty());
    }
    assert!(!sb.notes.exists(), "a refused add created the notes dir");
    // The limit itself is fine.
    ok(&client, "note_add", json!({"text": "x".repeat(20_000)})).await;
    assert_eq!(note_files(&sb.notes).len(), 1);
}

// ---------------------------------------------------------------------------------------------
// AC 4: cwd arg, else roots, else process cwd
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn cwd_arg_wins_over_roots() {
    let sb = Sandbox::new("cwd-arg");
    let a = make_repo(&sb.root, "repo-a", "main");
    let b = make_repo(&sb.root, "repo-b", "main");
    let client = connect(
        &sb,
        TestClient {
            name: "t",
            roots: Some(vec![format!("file://{}", b.display())]),
        },
    )
    .await;
    let out = ok(&client, "note_add", json!({"text": "x", "cwd": a})).await;
    assert_eq!(out["repo"], "repo-a");
}

#[tokio::test]
async fn roots_are_used_when_there_is_no_cwd_arg() {
    let sb = Sandbox::new("roots");
    let b = make_repo(&sb.root, "repo-b", "main");
    let client = connect(
        &sb,
        TestClient {
            name: "t",
            roots: Some(vec![format!("file://{}", b.display())]),
        },
    )
    .await;
    let out = ok(&client, "note_add", json!({"text": "x"})).await;
    assert_eq!(out["repo"], "repo-b");
}

#[tokio::test]
async fn no_roots_support_or_no_roots_falls_back_to_the_process_cwd() {
    let expected = context::capture(None, None, &[]).repo;
    for (name, roots) in [("none", None), ("empty", Some(vec![]))] {
        let sb = Sandbox::new(&format!("fallback-{name}"));
        let client = connect(&sb, TestClient { name: "t", roots }).await;
        let out = ok(&client, "note_add", json!({"text": "x"})).await;
        assert_eq!(out["repo"], json!(expected), "{name}");
    }
}

// ---------------------------------------------------------------------------------------------
// AC 5: FR-016 bounds
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn list_and_search_hold_at_most_50_items_with_short_snippets() {
    let sb = Sandbox::new("bounds");
    let long = format!("needle {}", "word ".repeat(300));
    let store = sb.store();
    for _ in 0..55 {
        store.add(&long, "cli", &[], &[]).unwrap();
    }
    let client = connect(&sb, TestClient::plain("t")).await;
    for (tool, args) in [
        ("note_list", json!({"limit": 500})),
        ("note_search", json!({"query": "needle", "limit": 500})),
        ("note_search", json!({"query": "word"})),
    ] {
        let page = ok(&client, tool, args.clone()).await;
        let items = page["items"].as_array().unwrap();
        assert!(items.len() <= 50, "{tool} {args}: {} items", items.len());
        assert_eq!(page["total"], 55, "{tool} {args}");
        for item in items {
            assert!(item["snippet"].as_str().unwrap().chars().count() <= 200);
            for key in ["id", "title", "created", "repo", "task", "tags"] {
                assert!(item.get(key).is_some(), "{tool}: no {key}");
            }
        }
    }
    let page = ok(&client, "note_list", json!({"limit": 500})).await;
    assert_eq!(page["items"].as_array().unwrap().len(), 50);
    assert_eq!(page["truncated"], true);
    let page = ok(&client, "note_list", json!({"limit": 3})).await;
    assert_eq!(page["items"].as_array().unwrap().len(), 3);
    assert_eq!(page["truncated"], true);
}

#[tokio::test]
async fn tool_filters_and_stats_work() {
    let sb = Sandbox::new("filters");
    let store = sb.store();
    store
        .add("one", "cli", &["red".into()], &[("repo", Some("r1"))])
        .unwrap();
    store
        .add("two", "cli", &["blue".into()], &[("repo", Some("r2"))])
        .unwrap();
    let client = connect(&sb, TestClient::plain("t")).await;
    let page = ok(&client, "note_list", json!({"repo": "r1"})).await;
    assert_eq!(page["total"], 1);
    let page = ok(
        &client,
        "note_search",
        json!({"query": "two", "tag": "blue"}),
    )
    .await;
    assert_eq!(page["total"], 1);
    let stats = ok(&client, "note_stats", json!({"repo": "r2"})).await;
    assert_eq!(stats["total"], 1);
    assert!(stats.get("skipped").is_some());
    let (is_error, _) = call(&client, "note_list", json!({"limit": 0})).await;
    assert!(is_error);
}

// ---------------------------------------------------------------------------------------------
// AC 9 (error text): tool errors are masked
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn tool_error_text_never_echoes_a_secret() {
    let sb = Sandbox::new("errmask");
    let client = connect(&sb, TestClient::plain("t")).await;
    // The store's "not a note id" message would quote the id it was given.
    let (is_error, text) = call(&client, "note_read", json!({"id": GITHUB})).await;
    assert!(is_error);
    assert!(!text.contains(GITHUB), "{text}");
    assert!(!text.is_empty());
}

/// A wrongly typed argument makes a protocol error, and serde quotes the value it choked on.
#[tokio::test]
async fn protocol_error_text_never_echoes_a_secret() {
    let sb = Sandbox::new("proto");
    let client = connect(&sb, TestClient::plain("t")).await;
    // `tags` must be a list; a string is not.
    let params = CallToolRequestParams::new("note_add").with_arguments(
        json!({"text": "x", "tags": GITHUB})
            .as_object()
            .unwrap()
            .clone(),
    );
    let shown = match client.call_tool(params).await {
        Ok(result) => format!("{:?}", result.content),
        Err(error) => error.to_string(),
    };
    assert!(!shown.contains(GITHUB), "{shown}");
    assert!(note_files(&sb.notes).is_empty());
}

// ---------------------------------------------------------------------------------------------
// AC 7: HTTP binds 127.0.0.1 only
// ---------------------------------------------------------------------------------------------

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn serve_cmd(sb: &Sandbox, args: &[&str]) -> Command {
    let mut cmd = Command::new(BIN);
    cmd.arg("serve")
        .args(args)
        .env("HOME", &sb.home)
        .env("EASY_NOTE_DIR", &sb.notes)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

/// Run `serve` to its exit; a server that is still running after 10 s fails the test, not hangs it.
fn exits(mut cmd: Command) -> std::process::Output {
    use wait_timeout::ChildExt;
    let mut child = cmd.spawn().unwrap();
    let status = child.wait_timeout(Duration::from_secs(10)).unwrap();
    if status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("the server kept running; it should have exited");
    }
    child.wait_with_output().unwrap()
}

/// Start `serve --http --port 0`; the port comes from the line it prints on stderr.
fn spawn_http(sb: &Sandbox, extra: &[&str]) -> (Server, u16) {
    let mut args = vec!["--http", "--port", "0"];
    args.extend(extra);
    let mut child = serve_cmd(sb, &args).spawn().unwrap();
    let stderr = child.stderr.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            let _ = tx.send(line);
        }
    });
    let server = Server(child);
    let line = rx
        .recv_timeout(Duration::from_secs(20))
        .expect("serve --http prints its address");
    let port = line
        .split("http://127.0.0.1:")
        .nth(1)
        .unwrap_or_else(|| panic!("no 127.0.0.1 address in {line:?}"))
        .split('/')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    (server, port)
}

fn post(port: u16, host: &str, body: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(
        stream,
        "POST /mcp HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut got = Vec::new();
    let mut buf = [0u8; 4096];
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            break;
        }
        got.extend_from_slice(&buf[..n]);
        if String::from_utf8_lossy(&got).contains("easy-note") {
            break;
        }
    }
    String::from_utf8_lossy(&got).into_owned()
}

const INIT: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"http-test","version":"0"}}}"#;

#[test]
fn http_mode_binds_loopback_and_answers_initialize() {
    let sb = Sandbox::new("http");
    let (_server, port) = spawn_http(&sb, &[]);
    let reply = post(port, &format!("127.0.0.1:{port}"), INIT);
    assert!(reply.starts_with("HTTP/1.1 200"), "{reply}");
    assert!(reply.contains("\"name\":\"easy-note\""), "{reply}");
    // A page on another host name (DNS rebinding) is refused.
    let reply = post(port, "evil.example", INIT);
    assert!(!reply.starts_with("HTTP/1.1 200"), "{reply}");
}

#[test]
fn http_host_other_than_127_0_0_1_exits_2_before_binding() {
    let sb = Sandbox::new("badhost");
    for host in ["0.0.0.0", "localhost", "::", "192.168.1.5", ""] {
        let out = exits(serve_cmd(&sb, &["--http", "--host", host, "--port", "0"]));
        assert_eq!(out.status.code(), Some(2), "--host {host:?}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("127.0.0.1"), "{stderr}");
        assert!(
            !stderr.contains("http://"),
            "it reported listening: {stderr}"
        );
        assert!(out.stdout.is_empty());
    }
    // `--host` and `--port` mean nothing without `--http`.
    let out = exits(serve_cmd(&sb, &["--host", "127.0.0.1"]));
    assert_eq!(out.status.code(), Some(2));
    // 127.0.0.1 itself is accepted: the server starts and prints its address.
    let (_server, _port) = spawn_http(&sb, &["--host", "127.0.0.1"]);
}

#[test]
fn http_port_in_use_exits_1_with_a_clear_message() {
    let sb = Sandbox::new("inuse");
    let (_first, port) = spawn_http(&sb, &[]);
    let out = exits(serve_cmd(&sb, &["--http", "--port", &port.to_string()]));
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot bind 127.0.0.1"), "{stderr}");
}

// ---------------------------------------------------------------------------------------------
// AC 8: smoke, the real binary over stdio with an rmcp client
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn smoke_real_binary_over_stdio_calls_every_tool() {
    let sb = Sandbox::new("smoke");
    let repo = make_repo(&sb.root, "smoke-repo", "feat/t107-smoke");
    let mut cmd = tokio::process::Command::new(BIN);
    cmd.arg("serve")
        .env("HOME", &sb.home)
        .env("EASY_NOTE_DIR", &sb.notes);
    let transport = TokioChildProcess::new(cmd).unwrap();
    let info = ClientConfig::new(
        ClientCapabilities::default(),
        Implementation::new("smoke-client", "0"),
    );
    let client = info.serve(transport).await.expect("initialize");

    let mut names: Vec<String> = client
        .list_all_tools()
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.name.to_string())
        .collect();
    names.sort();
    assert_eq!(names, TOOLS);

    let added = call(
        &client,
        "note_add",
        json!({"text": "smoke note #smoke", "tags": ["smoke"], "cwd": repo}),
    )
    .await;
    assert!(!added.0, "{}", added.1);
    let added: Value = serde_json::from_str(&added.1).unwrap();
    let id = added["id"].as_str().unwrap();
    assert_eq!(added["repo"], "smoke-repo");
    assert_eq!(added["task"], "T107");
    assert_eq!(note_files(&sb.notes).len(), 1);

    let read = call(&client, "note_read", json!({"id": id})).await;
    assert!(!read.0 && read.1.contains("smoke note"), "{}", read.1);
    let list = call(&client, "note_list", json!({})).await;
    assert!(!list.0 && list.1.contains(id), "{}", list.1);
    let search = call(&client, "note_search", json!({"query": "smoke"})).await;
    assert!(!search.0 && search.1.contains(id), "{}", search.1);
    let stats = call(&client, "note_stats", json!({})).await;
    assert!(!stats.0, "{}", stats.1);
    let stats: Value = serde_json::from_str(&stats.1).unwrap();
    assert_eq!(stats["total"], 1);
    assert_eq!(stats["by_source"]["mcp"], 1);

    let written = std::fs::read_to_string(&note_files(&sb.notes)[0]).unwrap();
    assert_eq!(
        parse(&written).unwrap().get_str("client"),
        Some("smoke-client")
    );
    client.cancel().await.unwrap();
}
