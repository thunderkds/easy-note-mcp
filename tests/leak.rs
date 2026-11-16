//! HUN-270 part 2 oracle (red): the guard wired into the store, and the leak harness.
//! Ported from the Python HUN-255 `tests/test_leak.py` (verified branch at 3c5e40f) and its
//! `test_store.py` (`test_update_never_writes_a_file_that_does_not_parse`).
//!
//! Part-2 contract under test (AC 3, 4, 5, 8, 9, 10):
//! * `store::add` returns `Added { id, path, created_dir, masked }`.
//! * `store::update` returns `Updated { note, masked }`.
//! * `masked` lists the kinds in order: body, tags, context fields (CONTEXT_KEYS order for
//!   update, the order given for add), unknown front-matter text (`extra`); repeats kept; a clean
//!   write reports nothing. It never holds a value.
//! * `store::read` and `store::iter_notes` return notes masked in every free-text part
//!   (body, tags, context fields, `extra`); the file bytes stay unchanged.
//! * `store::update` masks what it writes, parses the rendered text back, and on a parse failure
//!   returns `StoreError::Format` (no note text in the message) and writes nothing.
//! * The size check (`MAX_TEXT`) runs on the masked text.
//!
//! The leak harness is a registry of surfaces (`SURFACES`) x planted secrets
//! (`fixtures/secrets/planted.json`). Each surface runs its full write/read path on one planted
//! text and returns everything it got back. The harness greps every returned object, captured
//! stdout and stderr, and every file under the temp HOME and notes dir (trash and temp files
//! included) for the raw value. Later tasks (CLI, MCP, hook, quick, inbox) add their surface by
//! registering one function.
//!
//! GitHub push protection rejected the Python branch on a fake Slack token, so no literal here
//! holds a whole token prefix. Planted values come from the fixture (parts joined at run time)
//! and every inline value is built with `concat!` or `join`.
//!
//! Tests use a temp HOME and EASY_NOTE_DIR only (NFR-003), never the real `~/easy-notes`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use easy_note_mcp::core::format as fmt;
use easy_note_mcp::core::guard;
use easy_note_mcp::core::store::{self, Item, StoreError, Updated, CONTEXT_KEYS};
use serde_json::Value;

const PLANTED_JSON: &str = include_str!("fixtures/secrets/planted.json");
const SOURCE: &str = "cli";
const NOTE_ID: &str = "20261114-164713-ab12";
const CLEAN_BODY: &str = "clean body\n";

// Fake values, never a whole real-looking token in the source.
const AWS: &str = concat!("AK", "IAIOSFODNN7EXAMPLE");
const GITHUB: &str = concat!("gh", "p_EXAMPLEexampleEXAMPLEexampleEXAMPLE123456");
const SLACK: &str = concat!(
    "xo",
    "xb-123456789012-1234567890123-EXAMPLEexampleEXAMPLE12"
);
const OPENAI: &str = concat!("s", "k-EXAMPLEexampleEXAMPLEexample1234");

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct Planted {
    kind: String,
    text: String,
    secret: String,
}

fn join(parts: &Value) -> String {
    parts
        .as_array()
        .expect("parts must be an array")
        .iter()
        .map(|p| p.as_str().expect("part must be a string"))
        .collect()
}

fn planted() -> Vec<Planted> {
    let v: Value = serde_json::from_str(PLANTED_JSON).expect("planted.json parses");
    v.as_array()
        .expect("planted.json is an array")
        .iter()
        .map(|c| Planted {
            kind: c["kind"].as_str().expect("kind").to_string(),
            text: join(&c["text"]),
            secret: join(&c["secret"]),
        })
        .collect()
}

fn case_id(c: &Planted) -> String {
    format!(
        "{}:{}",
        c.kind,
        c.secret.chars().take(12).collect::<String>()
    )
}

/// A temp HOME and notes dir. Holds the env lock, so tests that set `HOME` / `EASY_NOTE_DIR`
/// never overlap. Removed when the test ends.
struct Env {
    root: PathBuf,
    notes: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl Env {
    fn new(name: &str) -> Env {
        let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let root =
            std::env::temp_dir().join(format!("easy-note-leak-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let home = root.join("home");
        std::fs::create_dir_all(&home).unwrap();
        let notes = root.join("notes");
        std::env::set_var("HOME", &home);
        std::env::set_var("EASY_NOTE_DIR", &notes);
        Env {
            root,
            notes,
            _lock: lock,
        }
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn all_files(root: &Path) -> Vec<PathBuf> {
    let mut all = vec![];
    walk(root, &mut all);
    all.sort();
    all
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    all_files(root)
        .into_iter()
        .map(|p| {
            let bytes = std::fs::read(&p).unwrap();
            (p.strip_prefix(root).unwrap().to_path_buf(), bytes)
        })
        .collect()
}

fn contains(haystack: &[u8], needle: &str) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|w| w == needle.as_bytes())
}

/// Kinds as owned strings, so a test holds for `Vec<String>` and `Vec<&str>` alike.
fn strs<S: AsRef<str>>(kinds: &[S]) -> Vec<String> {
    kinds.iter().map(|k| k.as_ref().to_string()).collect()
}

fn tags(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn dbg<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}")
}

fn read_parsed(path: &Path) -> fmt::Note {
    fmt::parse(&std::fs::read_to_string(path).unwrap()).expect("the stored file parses")
}

fn tag_list(note: &fmt::Note) -> Vec<String> {
    match note.fields.get("tags") {
        Some(fmt::Value::List(list)) => list.clone(),
        other => panic!("tags is not a list: {other:?}"),
    }
}

fn field(note: &fmt::Note, key: &str) -> String {
    note.get_str(key)
        .unwrap_or_else(|| panic!("no string field {key}"))
        .to_string()
}

/// Only the notes: iter_notes also yields Skipped files.
fn listed() -> Vec<fmt::Note> {
    store::iter_notes()
        .unwrap()
        .filter_map(|item| match item {
            Item::Note(n) => Some(n),
            Item::Skipped(_) => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Surface registry. A surface takes the planted text and returns everything it got back.
// ---------------------------------------------------------------------------------------------

/// What one surface run produced: Debug text of every returned object, and captured output.
struct Captured {
    objects: Vec<String>,
    stdout: String,
    stderr: String,
}

type Surface = fn(&str) -> Captured;

/// add -> read -> update (text, then tags) -> iter_notes -> rm (moves the file to trash).
/// The core is a library: it prints nothing, which `store_source_has_no_logging_or_printing`
/// pins; a process surface (CLI, hook) fills `stdout` and `stderr` with the real capture.
fn surface_core(text: &str) -> Captured {
    let mut objects = vec![];
    let added = store::add(text, SOURCE, &tags(&["leak"]), &[]).unwrap();
    objects.push(dbg(&added));
    objects.push(dbg(&store::read(&added.id).unwrap()));
    objects.push(dbg(&store::update(
        &added.id,
        Some(&format!("edited: {text}")),
        None,
    )
    .unwrap()));
    objects.push(dbg(&store::update(
        &added.id,
        None,
        Some(&tags(&["leak", "again"])),
    )
    .unwrap()));
    for item in store::iter_notes().unwrap() {
        objects.push(dbg(&item));
    }
    objects.push(dbg(&store::rm(&added.id).unwrap()));
    Captured {
        objects,
        stdout: String::new(),
        stderr: String::new(),
    }
}

/// The CLI: the real binary, every command that takes note text or prints it, and the error
/// paths that echo an argument. All stdout and stderr are captured and grepped.
fn surface_cli(text: &str) -> Captured {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::process::{Command, Stdio};

    let home = PathBuf::from(std::env::var("HOME").unwrap());
    // An $EDITOR stub that copies the file it was given into $HOME (so the harness greps what the
    // editor saw), then appends the planted text from its environment (never from a file).
    let stub = home.join("append.sh");
    std::fs::write(
        &stub,
        "#!/bin/sh\ncp \"$1\" \"$HOME/seen-$$.txt\"\nprintf '\\nedited %s\\n' \"$PLANT\" >> \"$1\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();

    let mut got = Captured {
        objects: vec![],
        stdout: String::new(),
        stderr: String::new(),
    };
    let mut run = |args: &[&str], stdin: Option<&str>| -> (String, Option<i32>) {
        let mut child = Command::new(env!("CARGO_BIN_EXE_easy-note"))
            .args(args)
            .env("PLANT", text)
            .env("EDITOR", &stub)
            .current_dir(&home)
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if let Some(input) = stdin {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(input.as_bytes())
                .unwrap();
        }
        let out = child.wait_with_output().unwrap();
        let (o, e) = (
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        );
        // The arguments are left out: they hold the planted text this test is hiding from the output.
        let n = got.objects.len();
        got.objects
            .push(format!("run {n} -> {:?}\n{o}\n{e}", out.status.code()));
        got.stdout.push_str(&o);
        got.stderr.push_str(&e);
        (o, out.status.code())
    };
    // A happy-path run must exit 0: a run that failed early would leave nothing to leak.
    let mut ok = |args: &[&str], stdin: Option<&str>| -> String {
        let (out, code) = run(args, stdin);
        assert_eq!(code, Some(0), "`{}` failed: {out}", args[0]);
        out
    };
    let id_of = |line: &str| {
        let id = line
            .split_whitespace()
            .nth(2)
            .unwrap_or_default()
            .to_string();
        assert!(id.starts_with("20"), "a note id, got {id:?}");
        id
    };

    let by_arg = id_of(&ok(&["add", text], None));
    let by_pipe = id_of(&ok(&["add"], Some(text)));
    let by_editor = id_of(&ok(&["add", "-e", "start"], None));
    // the given text goes through the editor file: it must be masked before the editor sees it
    id_of(&ok(&["add", "-e", text], None));
    ok(&["ls"], None);
    ok(&["search", text], None);
    ok(&["show", &by_arg], None);
    ok(&["edit", &by_pipe], None);
    ok(&["show", &by_pipe], None);
    ok(&["tag", &by_arg, "leak", "again"], None);
    ok(&["stats"], None);
    ok(&["stats", "--json"], None);
    ok(&["doctor"], None);
    ok(&["rm", &by_editor], None);
    ok(&["trash"], None);
    ok(&["restore", &by_editor], None);
    ok(&["path"], None);
    // the text where an id, a filter, a tag or a command should be: every error echoes or not
    for args in [
        vec!["show", text],
        vec!["rm", text],
        vec!["restore", text],
        vec!["edit", text],
        vec!["tag", text, "x"],
        vec!["tag", &by_arg, text],
        vec!["ls", text],
        vec!["ls", "--since", text],
        vec!["ls", "--tag", text],
        vec!["search", "--bogus", text],
        vec!["trash", "--empty", "--older-than", text],
        vec![text],
    ] {
        run(&args, None);
    }
    got
}

/// Every string in a JSON value, and inside any string that is itself JSON (a tool answer is
/// JSON text in a JSON line), so a secret JSON escaping would hide is still grepped.
fn json_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            out.push(s.clone());
            if let Ok(inner @ (Value::Object(_) | Value::Array(_))) = serde_json::from_str(s) {
                json_strings(&inner, out);
            }
        }
        Value::Array(list) => list.iter().for_each(|v| json_strings(v, out)),
        Value::Object(map) => map.values().for_each(|v| json_strings(v, out)),
        _ => {}
    }
}

/// The real `easy-note serve` over stdio, driven line by line: add, read, a read of an id that
/// is the secret (a tool error), an add with a wrongly typed argument (a protocol-level error
/// that quotes the value), search, list, stats. Every answer line, its decoded strings, the whole
/// of stdout and stderr are captured. The child inherits the temp HOME and EASY_NOTE_DIR.
fn surface_mcp(text: &str) -> Captured {
    use std::io::{BufRead, Read, Write};
    use std::process::{Command, Stdio};

    let mut child = Command::new(env!("CARGO_BIN_EXE_easy-note"))
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut lines = std::io::BufReader::new(child.stdout.take().unwrap()).lines();
    let home = std::env::var("HOME").unwrap();
    let mut stdout = String::new();
    let mut objects = vec![];
    let mut next_id = 0u32;
    let mut send = |method: &str, params: Value, expect_answer: bool| -> Option<Value> {
        next_id += 1;
        let mut request = serde_json::json!({"jsonrpc": "2.0", "method": method, "params": params});
        if expect_answer {
            request["id"] = next_id.into();
        }
        writeln!(stdin, "{request}").unwrap();
        stdin.flush().unwrap();
        if !expect_answer {
            return None;
        }
        let line = lines.next().expect("the server answers").unwrap();
        stdout.push_str(&line);
        stdout.push('\n');
        let answer: Value = serde_json::from_str(&line).expect("one JSON-RPC line");
        assert_eq!(answer["id"], next_id, "{line}");
        objects.push(line);
        let mut decoded = vec![];
        json_strings(&answer, &mut decoded);
        objects.extend(decoded);
        Some(answer)
    };
    let call = |name: &str, args: Value| serde_json::json!({"name": name, "arguments": args});

    send(
        "initialize",
        serde_json::json!({"protocolVersion": "2025-03-26", "capabilities": {},
            "clientInfo": {"name": "leak-test", "version": "0"}}),
        true,
    );
    send("notifications/initialized", serde_json::json!({}), false);
    let added = send(
        "tools/call",
        call(
            "note_add",
            serde_json::json!({"text": text, "tags": ["leak"], "cwd": home}),
        ),
        true,
    )
    .unwrap();
    let reply = added["result"]["content"][0]["text"].as_str().unwrap();
    let id = serde_json::from_str::<Value>(reply).expect("note_add answers JSON")["id"]
        .as_str()
        .unwrap()
        .to_string();
    send(
        "tools/call",
        call("note_read", serde_json::json!({"id": id})),
        true,
    );
    send(
        "tools/call",
        call("note_read", serde_json::json!({"id": text})),
        true,
    );
    send(
        "tools/call",
        call("note_add", serde_json::json!({"text": "x", "tags": text})),
        true,
    );
    send(
        "tools/call",
        call("note_search", serde_json::json!({"query": "leak"})),
        true,
    );
    send(
        "tools/call",
        call("note_search", serde_json::json!({"query": text})),
        true,
    );
    send("tools/call", call("note_list", serde_json::json!({})), true);
    send(
        "tools/call",
        call("note_stats", serde_json::json!({})),
        true,
    );
    drop(stdin);

    for line in lines {
        let line = line.unwrap();
        stdout.push_str(&line);
        stdout.push('\n');
    }
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert!(child.wait().unwrap().success());
    Captured {
        objects,
        stdout,
        stderr,
    }
}

const SURFACES: &[(&str, Surface)] = &[
    ("core", surface_core),
    ("cli", surface_cli),
    ("mcp", surface_mcp),
];

fn nothing_leaked(secret: &str, got: &Captured, root: &Path) -> Result<(), String> {
    if got.objects.is_empty() {
        return Err("a surface must return what it got, or the grep proves nothing".into());
    }
    if got.stdout.contains(secret) {
        return Err("secret reached stdout".into());
    }
    if got.stderr.contains(secret) {
        return Err("secret reached stderr".into());
    }
    for (i, object) in got.objects.iter().enumerate() {
        if object.contains(secret) {
            return Err(format!("secret in returned object {i}"));
        }
    }
    let files = all_files(root); // temp HOME, notes dir, trash and temp files
    if files.is_empty() {
        return Err("the surface wrote nothing, so the file grep proves nothing".into());
    }
    for path in files {
        if contains(&std::fs::read(&path).unwrap(), secret) {
            return Err(format!(
                "secret on disk: {}",
                path.strip_prefix(root).unwrap().display()
            ));
        }
    }
    Ok(())
}

#[test]
fn planted_secret_never_leaks() {
    let mut failures = vec![];
    for (name, surface) in SURFACES {
        for case in planted() {
            let env = Env::new("leak");
            let got = surface(&case.text);
            if let Err(why) = nothing_leaked(&case.secret, &got, &env.root) {
                failures.push(format!("{name} x {}: {why}", case_id(&case)));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn the_registry_holds_the_core_and_the_cli_surface() {
    for wanted in ["core", "cli"] {
        assert!(SURFACES.iter().any(|(name, _)| *name == wanted), "{wanted}");
    }
}

/// Pin of the harness itself: it must catch a leak, or "finds none" proves nothing.
#[test]
fn harness_catches_a_planted_value_in_a_file_and_in_an_object() {
    let env = Env::new("harness");
    let path = env.root.join("home").join("stray.txt");
    std::fs::write(&path, format!("oops {AWS}")).unwrap();
    let clean = Captured {
        objects: vec!["fine".into()],
        stdout: String::new(),
        stderr: String::new(),
    };
    assert!(nothing_leaked(AWS, &clean, &env.root)
        .unwrap_err()
        .contains("secret on disk"));
    std::fs::remove_file(&path).unwrap();
    std::fs::write(env.root.join("home").join("ok.txt"), "x").unwrap();
    let dirty = Captured {
        objects: vec![format!("holds {AWS}")],
        stdout: String::new(),
        stderr: String::new(),
    };
    assert!(nothing_leaked(AWS, &dirty, &env.root)
        .unwrap_err()
        .contains("returned object"));
}

/// AC 5/6: the core never prints or logs. No logger exists in the crate, so the check is
/// structural (as for the guard): the store source holds no printing or logging call.
#[test]
fn store_source_has_no_logging_or_printing() {
    let src = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/core/store.rs"),
    )
    .unwrap();
    for needle in [
        "println!",
        "eprintln!",
        "print!(",
        "eprint!(",
        "dbg!",
        "log::",
        "tracing::",
        "io::stdout",
        "io::stderr",
    ] {
        assert!(!src.contains(needle), "store.rs holds {needle}");
    }
}

#[test]
fn core_keeps_the_note_and_marks_the_kind() {
    for case in planted() {
        let _env = Env::new("keep");
        let added = store::add(&case.text, SOURCE, &[], &[]).unwrap();
        let (expected, kinds) = guard::mask(&case.text);
        let id = case_id(&case);
        assert_eq!(strs(&added.masked), strs(&kinds), "{id}");
        assert_eq!(strs(&added.masked), vec![case.kind.clone()], "{id}");
        assert_eq!(added.masked.len(), 1, "{id}");
        let stored = read_parsed(&added.path);
        assert_eq!(stored.body, format!("{expected}\n"), "{id}");
        assert!(
            stored.body.contains(&format!("[masked:{}]", case.kind)),
            "{id}"
        );
        assert!(!dbg(&added).contains(&case.secret), "{id}");
    }
}

// ---------------------------------------------------------------------------------------------
// Success criterion 1: store::add masks before the write and reports count and kinds
// ---------------------------------------------------------------------------------------------

#[test]
fn add_masks_two_secrets_and_reports_count_and_kinds() {
    let _env = Env::new("add2");
    let added = store::add(&format!("key {AWS} and {GITHUB}"), SOURCE, &[], &[]).unwrap();
    assert_eq!(strs(&added.masked), ["aws_key", "github_token"]);
    assert_eq!(added.masked.len(), 2);
    let raw = std::fs::read_to_string(&added.path).unwrap();
    assert!(raw.contains("[masked:aws_key]") && raw.contains("[masked:github_token]"));
    assert!(!raw.contains(AWS) && !raw.contains(GITHUB));
    assert_eq!(
        fmt::parse(&raw).unwrap().body,
        "key [masked:aws_key] and [masked:github_token]\n"
    );
    assert!(!dbg(&added).contains(AWS) && !dbg(&added).contains(GITHUB));
}

#[test]
fn add_reports_kinds_in_text_order() {
    let _env = Env::new("add-order");
    let added = store::add(&format!("first {GITHUB} then {AWS}"), SOURCE, &[], &[]).unwrap();
    assert_eq!(strs(&added.masked), ["github_token", "aws_key"]);
}

#[test]
fn add_of_a_note_with_no_secret_reports_nothing_and_changes_nothing() {
    let _env = Env::new("add-clean");
    let added = store::add(
        "retry bug only on Firefox, commit 3f2a9c1e0b",
        SOURCE,
        &[],
        &[],
    )
    .unwrap();
    assert!(added.masked.is_empty());
    assert_eq!(
        read_parsed(&added.path).body,
        "retry bug only on Firefox, commit 3f2a9c1e0b\n"
    );
}

#[test]
fn add_keeps_text_around_the_secret() {
    let _env = Env::new("add-around");
    let added = store::add(
        &format!("line one\ndeploy key {AWS} rotate it\n\nlast line"),
        SOURCE,
        &tags(&["ops"]),
        &[],
    )
    .unwrap();
    let note = read_parsed(&added.path);
    assert_eq!(
        note.body,
        "line one\ndeploy key [masked:aws_key] rotate it\n\nlast line\n"
    );
    assert_eq!(tag_list(&note), ["ops"]);
    assert_eq!(field(&note, "source"), SOURCE);
}

// ---------------------------------------------------------------------------------------------
// store::update masks before the write and returns Updated { note, masked }
// ---------------------------------------------------------------------------------------------

#[test]
fn update_masks_text_before_the_write() {
    let _env = Env::new("upd-text");
    let added = store::add("clean note", SOURCE, &[], &[]).unwrap();
    let result = store::update(
        &added.id,
        Some(&format!("now holds {AWS} and {GITHUB} plus words")),
        None,
    )
    .unwrap();
    let raw = std::fs::read_to_string(&added.path).unwrap();
    assert!(!raw.contains(AWS) && !raw.contains(GITHUB));
    assert_eq!(
        fmt::parse(&raw).unwrap().body,
        "now holds [masked:aws_key] and [masked:github_token] plus words\n"
    );
    assert_eq!(result.note.body, fmt::parse(&raw).unwrap().body);
    assert!(!dbg(&result).contains(AWS) && !dbg(&result).contains(GITHUB));
}

#[test]
fn update_of_tags_alone_keeps_a_masked_body_masked() {
    let _env = Env::new("upd-tags-body");
    let added = store::add(&format!("key {AWS}"), SOURCE, &[], &[]).unwrap();
    store::update(&added.id, None, Some(&tags(&["a"]))).unwrap();
    let raw = std::fs::read_to_string(&added.path).unwrap();
    assert!(!raw.contains(AWS) && raw.contains("[masked:aws_key]"));
}

// ---------------------------------------------------------------------------------------------
// Success criterion 2: a hand-edited file is masked on every read; the file bytes stay unchanged
// ---------------------------------------------------------------------------------------------

/// A valid file a person typed or synced in: it never went through `store::add`.
fn hand_write_front(
    notes: &Path,
    tag_values: &[&str],
    context: &[(&str, &str)],
    extra_text: Option<&str>,
    body: &str,
) -> (PathBuf, Vec<u8>) {
    let mut fields = BTreeMap::new();
    fields.insert("easy-note".to_string(), fmt::Value::Int(fmt::VERSION));
    fields.insert("id".to_string(), fmt::Value::Str(NOTE_ID.to_string()));
    fields.insert(
        "created".to_string(),
        fmt::Value::Str("2026-11-14T16:47:13+07:00".to_string()),
    );
    fields.insert("tags".to_string(), fmt::Value::List(tags(tag_values)));
    fields.insert("source".to_string(), fmt::Value::Str(SOURCE.to_string()));
    for (key, value) in context {
        fields.insert(key.to_string(), fmt::Value::Str(value.to_string()));
    }
    let mut note = fmt::Note::new(fields, body.to_string());
    if let Some(text) = extra_text {
        // An Obsidian-style key; a block scalar keeps any text verbatim.
        let lines: String = text.split('\n').map(|l| format!("  {l}\n")).collect();
        note.extra = vec![format!("x-obsidian: |\n{lines}")];
    }
    let path = notes.join("2026").join("11").join(format!("{NOTE_ID}.md"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let data = fmt::render(&note).into_bytes();
    std::fs::write(&path, &data).unwrap();
    (path, data)
}

fn hand_write(notes: &Path, body: &str) -> (PathBuf, Vec<u8>) {
    hand_write_front(notes, &[], &[], None, body)
}

#[test]
fn hand_edited_file_is_masked_on_read_and_iter_notes_and_bytes_unchanged() {
    for case in planted() {
        let env = Env::new("hand-body");
        let id = case_id(&case);
        let (path, before) = hand_write(&env.notes, &format!("{}\n", case.text));
        assert!(
            contains(&before, &case.secret),
            "{id}: the planted file holds the secret"
        );

        let read_note = store::read(NOTE_ID).unwrap();
        let listed = listed();
        assert_eq!(
            listed
                .iter()
                .map(|n| n.id().to_string())
                .collect::<Vec<_>>(),
            [NOTE_ID],
            "{id}"
        );
        let expected = guard::mask_for_output(&format!("{}\n", case.text));
        for note in [&read_note, &listed[0]] {
            assert!(!note.body.contains(&case.secret), "{id}");
            assert!(!note.title().contains(&case.secret), "{id}");
            assert!(!dbg(note).contains(&case.secret), "{id}");
            assert!(
                note.body.contains(&format!("[masked:{}]", case.kind)),
                "{id}"
            );
            assert_eq!(note.body, expected, "{id}");
        }
        assert_eq!(
            std::fs::read(&path).unwrap(),
            before,
            "{id}: a read never rewrites the file"
        );
    }
}

#[test]
fn hand_edited_jwt_example() {
    let env = Env::new("hand-jwt");
    // the fixture's `secret` is only the JWT signature; the whole token is the word that starts with the JWT header marker
    let jwt = planted()
        .into_iter()
        .filter(|c| c.kind == "jwt")
        .flat_map(|c| {
            c.text
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .find(|w| w.starts_with(concat!("ey", "J")))
        .expect("a planted jwt");
    let (path, before) = hand_write(
        &env.notes,
        &format!("token for staging {jwt} expires today\n"),
    );
    let note = store::read(NOTE_ID).unwrap();
    assert_eq!(note.body, "token for staging [masked:jwt] expires today\n");
    assert_eq!(note.title(), "token for staging [masked:jwt] expires today");
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

// ---------------------------------------------------------------------------------------------
// Stage 4 security finding: the guard covered only the body. A secret in a tag, in a context
// field, or (hand-edited) in an unknown front-matter key must not reach a file or a response.
// ---------------------------------------------------------------------------------------------

/// Where a planted text goes: a tag on add, a tag on update, or one context field on add.
fn meta_places() -> Vec<(&'static str, &'static str)> {
    let mut places = vec![("add", "tags"), ("update", "tags")];
    places.extend(CONTEXT_KEYS.iter().map(|k| ("add", *k)));
    places
}

/// Plant `text` in a tag or a context field, then every read path; return everything got back.
fn surface_meta(op: &str, place: &str, text: &str) -> Captured {
    let mut objects = vec![];
    let added = if op == "add" {
        if place == "tags" {
            store::add("clean body", SOURCE, &[text.to_string()], &[]).unwrap()
        } else {
            store::add("clean body", SOURCE, &[], &[(place, Some(text))]).unwrap()
        }
    } else {
        let added = store::add("clean body", SOURCE, &tags(&["ok"]), &[]).unwrap();
        objects.push(dbg(&store::update(
            &added.id,
            None,
            Some(&tags(&["ok", text])),
        )
        .unwrap()));
        objects.push(dbg(&store::update(
            &added.id,
            Some("edited body"),
            Some(&[text.to_string()]),
        )
        .unwrap()));
        added
    };
    objects.push(dbg(&added));
    objects.push(dbg(&store::read(&added.id).unwrap()));
    for item in store::iter_notes().unwrap() {
        objects.push(dbg(&item));
    }
    Captured {
        objects,
        stdout: String::new(),
        stderr: String::new(),
    }
}

#[test]
fn secret_in_a_tag_or_context_field_never_leaks() {
    let mut failures = vec![];
    for (op, place) in meta_places() {
        for case in planted() {
            let env = Env::new("meta");
            let got = surface_meta(op, place, &case.text);
            if let Err(why) = nothing_leaked(&case.secret, &got, &env.root) {
                failures.push(format!("{op}-{place} x {}: {why}", case_id(&case)));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn add_keeps_a_masked_tag_and_reports_its_kind() {
    for case in planted() {
        let _env = Env::new("tag-keep");
        let id = case_id(&case);
        let added = store::add(
            "clean body",
            SOURCE,
            &tags(&["keep", &case.text, "after"]),
            &[],
        )
        .unwrap();
        let (expected, kinds) = guard::mask(&case.text);
        assert_eq!(strs(&added.masked), strs(&kinds), "{id}");
        assert_eq!(strs(&added.masked), vec![case.kind.clone()], "{id}");
        let stored = read_parsed(&added.path);
        let stored_tags = tag_list(&stored);
        assert_eq!(stored_tags, ["keep", expected.as_str(), "after"], "{id}");
        assert!(
            stored_tags[1].contains(&format!("[masked:{}]", case.kind)),
            "{id}"
        );
        assert!(!dbg(&added).contains(&case.secret), "{id}");
        assert!(
            !dbg(&store::read(&added.id).unwrap()).contains(&case.secret),
            "{id}"
        );
    }
}

#[test]
fn add_keeps_a_masked_context_field_and_reports_its_kind() {
    for key in CONTEXT_KEYS {
        for case in planted() {
            let _env = Env::new("field-keep");
            let id = format!("{key} x {}", case_id(&case));
            let added = store::add("clean body", SOURCE, &[], &[(key, Some(&case.text))]).unwrap();
            let (expected, kinds) = guard::mask(&case.text);
            assert_eq!(strs(&added.masked), strs(&kinds), "{id}");
            assert_eq!(strs(&added.masked), vec![case.kind.clone()], "{id}");
            assert_eq!(field(&read_parsed(&added.path), key), expected, "{id}");
            assert!(!dbg(&added).contains(&case.secret), "{id}");
            assert_eq!(
                field(&store::read(&added.id).unwrap(), key),
                expected,
                "{id}"
            );
        }
    }
}

#[test]
fn update_keeps_a_masked_tag() {
    for case in planted() {
        let _env = Env::new("upd-tag-keep");
        let id = case_id(&case);
        let added = store::add("clean body", SOURCE, &tags(&["ok"]), &[]).unwrap();
        let note = store::update(&added.id, None, Some(&tags(&["ok", &case.text])))
            .unwrap()
            .note;
        let expected = guard::mask_for_output(&case.text);
        assert_eq!(tag_list(&note), ["ok", expected.as_str()], "{id}");
        assert_eq!(
            tag_list(&read_parsed(&added.path)),
            ["ok", expected.as_str()],
            "{id}"
        );
        assert!(
            !contains(&std::fs::read(&added.path).unwrap(), &case.secret),
            "{id}"
        );
    }
}

#[test]
fn non_secret_tags_and_context_fields_are_unchanged() {
    let _env = Env::new("clean-meta");
    let sha = format!("sha256:{}", "ab".repeat(32));
    let clean_tags = tags(&["flaky", "retry-bug", &sha, "https://example.com/a?b=c"]);
    let context = [
        ("repo", "easy-ui-mcp"),
        ("remote", "thunderkds/easy-ui-mcp"),
        ("branch", "fix/t120-firefox"),
        ("task", "T120"),
        ("client", "claude-code"),
        ("session", "0b6a3f6e-9c1d-4f6e-8a37-2f5f2d0b9a11"),
        ("commit", "3f2a9c1e0b7d4a5c8e9f00112233445566778899"),
    ];
    let fields: Vec<(&str, Option<&str>)> = context.iter().map(|(k, v)| (*k, Some(*v))).collect();
    let added = store::add("clean body", SOURCE, &clean_tags, &fields).unwrap();
    assert!(added.masked.is_empty());
    let check = |note: &fmt::Note, expect_tags: &[String]| {
        assert_eq!(tag_list(note), expect_tags);
        for (k, v) in context {
            assert_eq!(field(note, k), v);
        }
    };
    check(&read_parsed(&added.path), &clean_tags);
    check(&store::read(&added.id).unwrap(), &clean_tags);
    let mut more = clean_tags.clone();
    more.push("more".to_string());
    let result = store::update(&added.id, None, Some(&more)).unwrap();
    assert!(result.masked.is_empty());
    check(&result.note, &more);
    check(&read_parsed(&added.path), &more);
}

// ---------------------------------------------------------------------------------------------
// Added.masked order: body kinds, then tag kinds, then field kinds in the order the fields were given
// ---------------------------------------------------------------------------------------------

#[test]
fn added_masked_orders_body_then_tags_then_fields_as_given() {
    let _env = Env::new("add-order-all");
    let task = "password=hunter2example";
    let repo = format!("ref {SLACK}");
    let added = store::add(
        &format!("first {GITHUB} then {AWS}"),
        SOURCE,
        &tags(&[
            "plain",
            &format!("t {OPENAI}"),
            &format!("{SLACK} and {AWS}"),
        ]),
        // given order is task before repo; CONTEXT_KEYS order is repo before task
        &[
            ("task", Some(task)),
            ("branch", None),
            ("repo", Some(&repo)),
            ("client", Some("claude-code")),
        ],
    )
    .unwrap();
    assert_eq!(
        strs(&added.masked),
        [
            "github_token",
            "aws_key", // body, in text order
            "openai_key",
            "slack_token",
            "aws_key", // tags, in tag order then text order
            "password_assign",
            "slack_token", // fields, in the order given
        ]
    );
    assert_eq!(added.masked.len(), 7); // one entry per secret, repeats kept
    let bytes = std::fs::read(&added.path).unwrap();
    for raw in [GITHUB, AWS, OPENAI, SLACK, "hunter2example"] {
        assert!(
            !dbg(&added).contains(raw) && !contains(&bytes, raw),
            "{raw}"
        );
    }
}

#[test]
fn added_masked_orders_fields_by_the_order_given_not_by_context_keys() {
    let _env = Env::new("add-order-fields");
    let repo = format!("x {AWS}");
    let task = format!("x {SLACK}");
    let forward = store::add(
        "clean",
        SOURCE,
        &[],
        &[("repo", Some(&repo)), ("task", Some(&task))],
    )
    .unwrap();
    let backward = store::add(
        "clean",
        SOURCE,
        &[],
        &[("task", Some(&task)), ("repo", Some(&repo))],
    )
    .unwrap();
    assert_eq!(strs(&forward.masked), ["aws_key", "slack_token"]);
    assert_eq!(strs(&backward.masked), ["slack_token", "aws_key"]);
}

#[test]
fn added_masked_is_empty_for_a_clean_note_with_clean_tags_and_fields() {
    let _env = Env::new("add-clean-all");
    let added = store::add(
        "clean",
        SOURCE,
        &tags(&["a", "b"]),
        &[("repo", Some("r")), ("task", None)],
    )
    .unwrap();
    assert!(added.masked.is_empty());
}

// ---------------------------------------------------------------------------------------------
// Hand-edited front matter: masked on read and iter_notes, file bytes unchanged
// ---------------------------------------------------------------------------------------------

fn hand_places() -> Vec<&'static str> {
    let mut places = vec!["tags"];
    places.extend(CONTEXT_KEYS);
    places.push("extra");
    places
}

fn hand_write_at(notes: &Path, place: &str, text: &str) -> (PathBuf, Vec<u8>) {
    match place {
        "tags" => hand_write_front(notes, &["keep", text], &[], None, CLEAN_BODY),
        "extra" => hand_write_front(notes, &[], &[], Some(text), CLEAN_BODY),
        key => hand_write_front(notes, &[], &[(key, text)], None, CLEAN_BODY),
    }
}

#[test]
fn hand_edited_front_matter_is_masked_on_read_and_iter_notes_and_bytes_unchanged() {
    for place in hand_places() {
        for case in planted() {
            let env = Env::new("hand-front");
            let id = format!("{place} x {}", case_id(&case));
            let (path, before) = hand_write_at(&env.notes, place, &case.text);
            assert!(
                contains(&before, &case.secret),
                "{id}: the planted file holds the secret"
            );
            let raw_extra = fmt::parse(std::str::from_utf8(&before).unwrap())
                .unwrap()
                .extra
                .concat();

            let read_note = store::read(NOTE_ID).unwrap();
            let listed = listed();
            assert_eq!(
                listed
                    .iter()
                    .map(|n| n.id().to_string())
                    .collect::<Vec<_>>(),
                [NOTE_ID],
                "{id}"
            );
            let masked = guard::mask_for_output(&case.text);
            assert!(masked.contains(&format!("[masked:{}]", case.kind)), "{id}");
            for note in [&read_note, &listed[0]] {
                assert!(
                    !dbg(note).contains(&case.secret),
                    "{id}: secret in the returned Note"
                );
                assert!(
                    dbg(note).contains(&format!("[masked:{}]", case.kind)),
                    "{id}"
                );
                assert_eq!(note.body, CLEAN_BODY, "{id}");
                match place {
                    "tags" => assert_eq!(tag_list(note), ["keep", masked.as_str()], "{id}"),
                    "extra" => assert_eq!(
                        note.extra.concat(),
                        guard::mask_for_output(&raw_extra),
                        "{id}"
                    ),
                    key => assert_eq!(field(note, key), masked, "{id}"),
                }
            }
            assert_eq!(
                std::fs::read(&path).unwrap(),
                before,
                "{id}: a read never rewrites the file"
            );
        }
    }
}

#[test]
fn hand_edited_front_matter_with_all_three_places_and_two_notes() {
    let env = Env::new("hand-three");
    // A tag, a context key and an unknown key in one file, read next to a clean sibling note.
    let (hand_path, _) = hand_write_front(
        &env.notes,
        &[&format!("x {AWS}")],
        &[("branch", &format!("b {GITHUB}"))],
        Some(&format!("tok: {SLACK}")),
        CLEAN_BODY,
    );
    let clean = store::add(
        "clean",
        SOURCE,
        &tags(&["fine"]),
        &[("repo", Some("easy-note-mcp"))],
    )
    .unwrap();
    let before = std::fs::read(&hand_path).unwrap();
    let notes_out = listed();
    assert_eq!(notes_out.len(), 2);
    let by_id: BTreeMap<String, &fmt::Note> =
        notes_out.iter().map(|n| (n.id().to_string(), n)).collect();
    let hand = by_id[NOTE_ID];
    assert_eq!(tag_list(hand), ["x [masked:aws_key]"]);
    assert_eq!(field(hand, "branch"), "b [masked:github_token]");
    assert_eq!(
        hand.extra.concat().matches("[masked:slack_token]").count(),
        1
    );
    for raw in [AWS, GITHUB, SLACK] {
        assert!(!dbg(hand).contains(raw) && !dbg(&store::read(NOTE_ID).unwrap()).contains(raw));
    }
    let other = by_id[clean.id.as_str()];
    assert_eq!(tag_list(other), ["fine"]);
    assert_eq!(field(other, "repo"), "easy-note-mcp");
    assert_eq!(std::fs::read(&hand_path).unwrap(), before);
}

#[test]
fn hand_edited_clean_front_matter_is_returned_unchanged() {
    let env = Env::new("hand-clean");
    let sha = format!("sha256:{}", "cd".repeat(32));
    let commit = "3f2a9c1e0b7d4a5c8e9f00112233445566778899";
    let (path, _) = hand_write_front(
        &env.notes,
        &["a", &sha],
        &[("repo", "r"), ("commit", commit)],
        Some("cssclass: wide\nrating: 4"),
        CLEAN_BODY,
    );
    let raw_extra = read_parsed(&path).extra.concat();
    let note = store::read(NOTE_ID).unwrap();
    assert_eq!(tag_list(&note), ["a", sha.as_str()]);
    assert_eq!(field(&note, "repo"), "r");
    assert_eq!(field(&note, "commit"), commit);
    assert_eq!(note.extra.concat(), raw_extra);
}

// ---------------------------------------------------------------------------------------------
// Review P2 gap: a rewrite of a hand-edited file never writes a raw secret back to disk
// ---------------------------------------------------------------------------------------------

#[test]
fn tags_only_update_on_a_hand_edited_file_writes_the_body_masked() {
    for case in planted() {
        let env = Env::new("tags-only-hand");
        let id = case_id(&case);
        let (path, before) = hand_write(&env.notes, &format!("{}\n", case.text));
        assert!(contains(&before, &case.secret), "{id}");
        let result = store::update(NOTE_ID, None, Some(&tags(&["a"]))).unwrap(); // the body is not part of the change
        let after = std::fs::read(&path).unwrap();
        assert!(!contains(&after, &case.secret), "{id}");
        let stored = fmt::parse(std::str::from_utf8(&after).unwrap()).unwrap();
        assert_eq!(
            stored.body,
            guard::mask_for_output(&format!("{}\n", case.text)),
            "{id}"
        );
        assert!(
            stored.body.contains(&format!("[masked:{}]", case.kind)),
            "{id}"
        );
        assert_eq!(tag_list(&stored), ["a"], "{id}");
        assert!(!dbg(&result).contains(&case.secret), "{id}");
        // the raw body secret masked on this write is counted
        assert_eq!(strs(&result.masked), vec![case.kind.clone()], "{id}");
    }
}

#[test]
fn any_update_of_a_hand_edited_file_writes_no_raw_secret_from_its_front_matter() {
    // Not in the Supervisor contract for update; follows from NFR-002 (no secret in any written file).
    for place in hand_places() {
        let env = Env::new("any-update-hand");
        let (path, before) = hand_write_at(&env.notes, place, &format!("leak {AWS} here"));
        assert!(contains(&before, AWS), "{place}");
        let result = store::update(NOTE_ID, Some("new clean text"), None).unwrap();
        let after = std::fs::read(&path).unwrap();
        assert!(
            !contains(&after, AWS),
            "a raw secret from {place} was written back by update"
        );
        assert!(contains(&after, "[masked:aws_key]"), "{place}");
        assert_eq!(
            strs(&result.masked),
            ["aws_key"],
            "the secret masked from {place} must be counted"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Decision (3-Advocate debate): store::update returns Updated { note, masked }, like Added.masked
//   masked = kinds masked before this write: body, then tags, then context fields in CONTEXT_KEYS
//   order, then extra; repeats kept; a raw secret a person typed into the file counts too.
// ---------------------------------------------------------------------------------------------

#[test]
fn updated_holds_the_note_and_the_masked_kinds() {
    let _env = Env::new("updated-shape");
    let id = store::add("clean", SOURCE, &[], &[]).unwrap().id;
    let Updated { note, masked } = store::update(&id, Some("still clean"), None).unwrap();
    assert_eq!(note.id(), id);
    assert!(masked.is_empty());
}

#[test]
fn update_with_a_secret_in_new_text_reports_its_kind() {
    let _env = Env::new("upd-kind");
    let added = store::add("clean note", SOURCE, &[], &[]).unwrap();
    let result = store::update(&added.id, Some(&format!("rotate {AWS} today")), None).unwrap();
    assert_eq!(strs(&result.masked), ["aws_key"]);
    assert!(!dbg(&result).contains(AWS));
    assert!(!contains(&std::fs::read(&added.path).unwrap(), AWS));
}

#[test]
fn update_reports_the_kind_of_each_planted_secret_in_new_text() {
    for case in planted() {
        let _env = Env::new("upd-planted");
        let added = store::add("clean note", SOURCE, &[], &[]).unwrap();
        let result = store::update(&added.id, Some(&case.text), None).unwrap();
        assert_eq!(
            strs(&result.masked),
            vec![case.kind.clone()],
            "{}",
            case_id(&case)
        );
        assert!(!dbg(&result).contains(&case.secret), "{}", case_id(&case));
    }
}

#[test]
fn update_with_secret_tags_reports_the_tag_kinds_in_tag_order() {
    let _env = Env::new("upd-tag-kinds");
    let added = store::add("clean body", SOURCE, &tags(&["ok"]), &[]).unwrap();
    let result = store::update(
        &added.id,
        None,
        Some(&tags(&[
            "plain",
            &format!("t {OPENAI}"),
            &format!("{SLACK} and {AWS}"),
        ])),
    )
    .unwrap();
    assert_eq!(
        strs(&result.masked),
        ["openai_key", "slack_token", "aws_key"]
    );
    let bytes = std::fs::read(&added.path).unwrap();
    for raw in [OPENAI, SLACK, AWS] {
        assert!(
            !dbg(&result).contains(raw) && !contains(&bytes, raw),
            "{raw}"
        );
    }
}

#[test]
fn tags_only_update_of_a_hand_edited_file_with_a_raw_body_secret_reports_that_kind() {
    let env = Env::new("hand-body-kind");
    let (path, before) = hand_write(&env.notes, &format!("typed by hand {AWS}\n"));
    assert!(contains(&before, AWS));
    let result = store::update(NOTE_ID, None, Some(&tags(&["a"]))).unwrap();
    assert_eq!(strs(&result.masked), ["aws_key"]);
    assert!(!contains(&std::fs::read(&path).unwrap(), AWS));
    assert!(!dbg(&result).contains(AWS));
}

/// The new text replaces the hand-typed body, so only what is written counts.
#[test]
fn update_with_new_text_does_not_count_the_replaced_raw_body() {
    let env = Env::new("replace-raw");
    let (path, _) = hand_write(&env.notes, &format!("typed by hand {AWS}\n"));
    let result = store::update(NOTE_ID, Some("a clean replacement"), None).unwrap();
    assert!(result.masked.is_empty());
    assert!(!contains(&std::fs::read(&path).unwrap(), AWS));
}

#[test]
fn a_clean_update_reports_nothing() {
    let _env = Env::new("upd-clean");
    let added = store::add("clean note", SOURCE, &tags(&["a"]), &[]).unwrap();
    assert!(store::update(
        &added.id,
        Some("retry bug only on Firefox, commit 3f2a9c1e0b"),
        None
    )
    .unwrap()
    .masked
    .is_empty());
    let sha = format!("sha256:{}", "ab".repeat(32));
    assert!(store::update(&added.id, None, Some(&tags(&["b", &sha])))
        .unwrap()
        .masked
        .is_empty());
    // nothing to change, still a valid update
    assert!(store::update(&added.id, None, None)
        .unwrap()
        .masked
        .is_empty());
}

#[test]
fn update_does_not_recount_a_marker_already_on_disk() {
    let _env = Env::new("no-recount");
    let added = store::add(&format!("key {AWS}"), SOURCE, &[format!("t {SLACK}")], &[]).unwrap();
    assert_eq!(strs(&added.masked), ["aws_key", "slack_token"]);
    // the file already holds [masked:...] only
    let result = store::update(&added.id, None, Some(&tags(&["fresh"]))).unwrap();
    assert!(result.masked.is_empty());
}

#[test]
fn update_masked_orders_body_then_tags_then_context_keys_then_extra() {
    let env = Env::new("upd-order-all");
    // Hand-typed context fields and an unknown key are masked on this write and counted, in
    // CONTEXT_KEYS order.
    let per_key = |key: &str| -> (String, &'static str) {
        match key {
            "repo" => (format!("v {SLACK}"), "slack_token"), // first key: tells the order from the others
            "task" => (format!("v {OPENAI}"), "openai_key"),
            _ => (format!("v {AWS}"), "aws_key"),
        }
    };
    let values: Vec<(String, String)> = CONTEXT_KEYS
        .iter()
        .map(|k| (k.to_string(), per_key(k).0))
        .collect();
    let context: Vec<(&str, &str)> = values
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let (path, _) = hand_write_front(
        &env.notes,
        &["plain"],
        &context,
        Some(&format!("tok: {GITHUB}")),
        &format!("raw {AWS}\n"),
    );
    let result = store::update(NOTE_ID, None, Some(&[format!("t {GITHUB}")])).unwrap();
    let mut expected = vec!["aws_key".to_string()]; // body
    expected.push("github_token".to_string()); // tags
    expected.extend(CONTEXT_KEYS.iter().map(|k| per_key(k).1.to_string())); // CONTEXT_KEYS order
    expected.push("github_token".to_string()); // extra
    assert_eq!(strs(&result.masked), expected);
    assert_eq!(result.masked.len(), 3 + CONTEXT_KEYS.len()); // repeats kept: one entry per secret
    let after = std::fs::read(&path).unwrap();
    for raw in [AWS, GITHUB, SLACK, OPENAI] {
        assert!(
            !contains(&after, raw) && !dbg(&result).contains(raw),
            "{raw}"
        );
    }
}

#[test]
fn update_masked_new_text_comes_before_tag_kinds() {
    let _env = Env::new("upd-text-first");
    let added = store::add("clean", SOURCE, &[], &[]).unwrap();
    let result = store::update(
        &added.id,
        Some(&format!("body {GITHUB}")),
        Some(&[format!("t {AWS}")]),
    )
    .unwrap();
    assert_eq!(strs(&result.masked), ["github_token", "aws_key"]);
}

#[test]
fn debug_of_updated_never_holds_a_raw_value() {
    for case in planted() {
        let _env = Env::new("upd-debug");
        let id = case_id(&case);
        let added = store::add("clean", SOURCE, &[], &[]).unwrap();
        let result = store::update(
            &added.id,
            Some(&case.text),
            Some(std::slice::from_ref(&case.text)),
        )
        .unwrap();
        assert_eq!(
            strs(&result.masked),
            vec![case.kind.clone(), case.kind.clone()],
            "{id}"
        );
        let shown = dbg(&result);
        assert!(!shown.contains(&case.secret), "{id}");
        assert!(shown.contains(&format!("[masked:{}]", case.kind)), "{id}");
    }
}

// ---------------------------------------------------------------------------------------------
// AC 10: update never writes a file that does not parse; the size check runs on the masked text
// ---------------------------------------------------------------------------------------------

/// An unclosed private key in a quoted hand-edited value masks to the chunk end and eats the
/// closing quote. Update must refuse (quoting no note text) and leave the file and folder as
/// they were. Python: `test_update_never_writes_a_file_that_does_not_parse`.
#[test]
fn update_never_writes_a_file_that_does_not_parse() {
    let env = Env::new("unparseable");
    let added = store::add("clean", SOURCE, &[], &[]).unwrap();
    let key = [
        "-----BEG",
        "IN RSA PRIVATE KEY-----\\nMIIEowIBAAKCAQEAexampleEXAMPLE",
    ]
    .concat();
    let hand = std::fs::read_to_string(&added.path).unwrap().replacen(
        "---\nclean",
        &format!("x-note: \"{key}\"\n---\nclean"),
        1,
    );
    std::fs::write(&added.path, &hand).unwrap();
    assert!(
        !fmt::parse(&hand).unwrap().extra.is_empty(),
        "the hand-edited file is valid before the update"
    );
    let before = snapshot(&env.root);
    let err = store::update(&added.id, None, Some(&tags(&["a"]))).unwrap_err();
    assert!(matches!(err, StoreError::Format(_)), "{err:?}");
    let shown = format!("{err} {err:?}");
    assert!(
        !shown.contains("BEGIN") && !shown.contains("MIIE") && !shown.contains("PRIVATE"),
        "the error quotes note text: {shown}"
    );
    assert_eq!(
        snapshot(&env.root),
        before,
        "bytes unchanged, no temp file left"
    );
}

/// Raw text over the limit that masks to within it is accepted (the size check runs on the
/// masked text), for add and for update.
#[test]
fn size_check_accepts_text_that_is_over_the_limit_only_before_masking() {
    let _env = Env::new("size-ok");
    let text = format!("{} {AWS}", "x".repeat(19_980)); // 20,001 chars raw, 19,997 masked
    assert_eq!(text.chars().count(), 20_001);
    let added = store::add(&text, SOURCE, &[], &[]).unwrap();
    assert_eq!(strs(&added.masked), ["aws_key"]);
    assert!(read_parsed(&added.path)
        .body
        .ends_with(" [masked:aws_key]\n"));
    let result = store::update(&added.id, Some(&text), None).unwrap();
    assert_eq!(strs(&result.masked), ["aws_key"]);
}

/// Raw text within the limit that masks to more than the limit is rejected (the marker is
/// longer than a short value), and nothing is written.
#[test]
fn size_check_rejects_text_that_is_over_the_limit_only_after_masking() {
    let env = Env::new("size-too-long");
    let text = format!("{} password=abcd", "x".repeat(19_986)); // 20,000 chars raw
    assert_eq!(text.chars().count(), 20_000);
    assert!(guard::mask(&text).0.chars().count() > 20_000);
    let err = store::add(&text, SOURCE, &[], &[]).unwrap_err();
    assert!(matches!(err, StoreError::TextTooLong(_)), "{err:?}");
    assert!(!dbg(&err).contains("abcd"));
    assert!(
        !env.notes.exists() || all_files(&env.notes).is_empty(),
        "a rejected add writes nothing"
    );

    let added = store::add("short", SOURCE, &[], &[]).unwrap();
    let before = snapshot(&env.root);
    let err = store::update(&added.id, Some(&text), None).unwrap_err();
    assert!(matches!(err, StoreError::TextTooLong(_)), "{err:?}");
    assert_eq!(snapshot(&env.root), before);
}

// ---------------------------------------------------------------------------------------------
// Part 3 (security-review finding): a YAML escape directly before a token, in an unknown
// front-matter key, hides the token from the anchored patterns in the raw source text
// ---------------------------------------------------------------------------------------------

/// The anchored kinds: their patterns start with a word boundary, so `\n` (two characters in a
/// double-quoted scalar) before the token leaves no boundary in the raw text.
const ANCHORED: [&str; 7] = [
    "aws_key",
    "github_token",
    "slack_token",
    "openai_key",
    "anthropic_key",
    "jwt",
    "bearer",
];

/// The text that must start right after the escape, and the pieces that must not survive.
fn escape_token(case: &Planted) -> (String, Vec<String>) {
    match case.kind.as_str() {
        "jwt" => {
            let start = case.text.find("eyJ").expect("planted jwt");
            let token: String = case.text[start..]
                .chars()
                .take_while(|c| !c.is_whitespace())
                .collect();
            (token.clone(), vec![token, case.secret.clone()])
        }
        "bearer" => (format!("Bearer {}", case.secret), vec![case.secret.clone()]),
        _ => (case.secret.clone(), vec![case.secret.clone()]),
    }
}

/// A valid file with one hand-typed unknown key (`chunk`, YAML source) and a clean body.
fn hand_write_chunk(notes: &Path, chunk: &str) -> (PathBuf, Vec<u8>) {
    let (path, _) = hand_write(notes, CLEAN_BODY);
    let mut note = read_parsed(&path);
    note.extra = vec![format!("{chunk}\n")];
    let data = fmt::render(&note).into_bytes();
    std::fs::write(&path, &data).unwrap();
    (path, data)
}

/// Read, list and update a hand-typed unknown key. Nothing may hold `leaks`; update reports
/// `kinds`; the key text is gone from every file under the temp root.
fn assert_chunk_masked(label: &str, chunk: &str, leaks: &[String], kinds: &[&str]) {
    let env = Env::new("chunk");
    let (path, before) = hand_write_chunk(&env.notes, chunk);
    let read_note = store::read(NOTE_ID).unwrap();
    let listed = listed();
    assert_eq!(listed.len(), 1, "{label}");
    for note in [&read_note, &listed[0]] {
        for leak in leaks {
            assert!(
                !dbg(note).contains(leak.as_str()),
                "{label}: value in a returned Note: {:?}",
                note.extra
            );
        }
    }
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "{label}: a read never rewrites the file"
    );

    let result = store::update(NOTE_ID, Some("new"), None).unwrap();
    assert_eq!(strs(&result.masked), kinds, "{label}: update reports kinds");
    for leak in leaks {
        assert!(
            !dbg(&result).contains(leak.as_str()),
            "{label}: value in Updated"
        );
        for file in all_files(&env.root) {
            assert!(
                !contains(&std::fs::read(&file).unwrap(), leak),
                "{label}: value on disk in {}",
                file.display()
            );
        }
    }
    read_parsed(&path); // the rewritten file still parses
}

#[test]
fn escape_before_an_anchored_token_in_an_unknown_key_never_leaks() {
    for case in planted()
        .into_iter()
        .filter(|c| ANCHORED.contains(&c.kind.as_str()))
    {
        let (token, leaks) = escape_token(&case);
        for esc in ["\\n", "\\t", "\\r"] {
            let chunk = format!("desc: \"creds:{esc}{token}\"");
            let label = format!("{} after {esc}", case_id(&case));
            assert_chunk_masked(&label, &chunk, &leaks, &[case.kind.as_str()]);
        }
    }
}

#[test]
fn a_hex_or_unicode_escape_that_spells_the_token_never_leaks() {
    // `\x41` is `A` once decoded: the raw text never holds the token.
    let hex = AWS.replacen('A', "\\x41", 1);
    let uni = AWS.replacen('A', "\\u0041", 1);
    for chunk in [
        format!("desc: \"key {hex}\""),
        format!("desc: \"key {uni}\""),
    ] {
        assert!(!chunk.contains(AWS));
        assert_chunk_masked(&chunk, &chunk, &[AWS.to_string()], &["aws_key"]);
    }
}

#[test]
fn a_folded_or_next_line_value_in_an_unknown_key_never_leaks() {
    for chunk in [
        format!("k: >-\n  {AWS}"),
        format!("k: |\n  see\n  {GITHUB}"),
        format!("k:\n  \"{SLACK}\""),
    ] {
        let leak = [AWS, GITHUB, SLACK]
            .into_iter()
            .find(|s| chunk.contains(s))
            .unwrap();
        let kind = match leak {
            AWS => "aws_key",
            GITHUB => "github_token",
            _ => "slack_token",
        };
        assert_chunk_masked(&chunk, &chunk, &[leak.to_string()], &[kind]);
    }
}

#[test]
fn escaped_token_inside_a_list_or_map_value_never_leaks() {
    for chunk in [
        format!("k: [\"a\", \"b:\\n{AWS}\"]"),
        format!("k:\n  - \"x\"\n  - \"y:\\t{AWS}\""),
        format!("k:\n  inner: \"z:\\r{AWS}\""),
    ] {
        assert_chunk_masked(&chunk, &chunk, &[AWS.to_string()], &["aws_key"]);
    }
}

#[test]
fn escaped_token_after_a_raw_masked_one_reports_both_kinds_in_order() {
    // A plain token (the raw pass catches it) and an escaped one (only the decoded pass can).
    let chunk = format!("k: \"{GITHUB} then:\\n{AWS}\"");
    assert_chunk_masked(
        &chunk,
        &chunk,
        &[AWS.to_string(), GITHUB.to_string()],
        &["github_token", "aws_key"],
    );
}

#[test]
fn an_unknown_key_with_escapes_and_no_secret_is_kept_as_typed() {
    let env = Env::new("chunk-clean");
    let chunk = "desc: \"line one\\nline two\\ttabbed\"";
    let (path, before) = hand_write_chunk(&env.notes, chunk);
    let read_note = store::read(NOTE_ID).unwrap();
    assert_eq!(read_note.extra, [format!("{chunk}\n")]);
    assert_eq!(listed()[0].extra, [format!("{chunk}\n")]);
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let result = store::update(NOTE_ID, Some("new"), None).unwrap();
    assert!(result.masked.is_empty());
    assert!(std::fs::read_to_string(&path).unwrap().contains(chunk));
}
