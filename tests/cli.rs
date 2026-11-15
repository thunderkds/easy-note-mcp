//! Locked oracle for HUN-268 (R1): the binary runs as `easy-note` and as `n`,
//! `add` is a stub, and `serve` answers `initialize` on stdio and exits on EOF.
//! Tests use a temp HOME and EASY_NOTE_DIR; they never touch the real ones.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_easy-note"))
}

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("easy-note-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(program: &Path, args: &[&str], home: &Path) -> Output {
    Command::new(program)
        .args(args)
        .env("HOME", home)
        .env("EASY_NOTE_DIR", home.join("notes"))
        .output()
        .expect("run binary")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn version_prints_name_and_number() {
    let home = sandbox("version");
    let out = run(&bin(), &["--version"], &home);
    assert!(out.status.success());
    assert_eq!(text(&out.stdout).trim(), "easy-note 0.1.0");
}

#[test]
fn n_symlink_prints_same_version() {
    let home = sandbox("n-version");
    let link = home.join("n");
    std::os::unix::fs::symlink(bin(), &link).unwrap();
    let out = run(&link, &["--version"], &home);
    assert!(out.status.success());
    assert_eq!(text(&out.stdout).trim(), "easy-note 0.1.0");
}

#[test]
fn add_stub_exits_2_with_not_implemented() {
    let home = sandbox("add");
    let out = run(&bin(), &["add", "hello"], &home);
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("not implemented yet"));
}

#[test]
fn n_hello_behaves_as_add_hello() {
    let home = sandbox("n-hello");
    let link = home.join("n");
    std::os::unix::fs::symlink(bin(), &link).unwrap();
    let via_n = run(&link, &["hello"], &home);
    let via_add = run(&bin(), &["add", "hello"], &home);
    assert_eq!(via_n.status.code(), Some(2));
    assert_eq!(via_n.status.code(), via_add.status.code());
    assert_eq!(text(&via_n.stderr), text(&via_add.stderr));
    assert_eq!(via_n.stdout, via_add.stdout);
}

#[test]
fn serve_answers_initialize_then_exits_on_eof() {
    let home = sandbox("serve");
    let mut child = Command::new(bin())
        .arg("serve")
        .env("HOME", &home)
        .env("EASY_NOTE_DIR", home.join("notes"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn serve");
    let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"cli-test","version":"0"}}}"#;
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "{request}").unwrap();
    drop(stdin); // EOF
    let out = child.wait_with_output().expect("wait");
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    let first = text(&out.stdout).lines().next().unwrap_or("").to_owned();
    let reply: serde_json::Value = serde_json::from_str(&first).expect("one JSON-RPC line");
    assert_eq!(reply["id"], 1);
    assert!(reply["result"].is_object(), "reply: {first}");
    assert_eq!(reply["result"]["serverInfo"]["name"], "easy-note");
}
