//! HUN-261 oracle: read-only inbox notes (FR-023, NOTE_FORMAT "Inbox files"). A file a sync app
//! drops in `<notes dir>/inbox/` is a note; easy-note never writes it (D23).
//!
//! Everything runs through the real binary (CLI and `serve` over stdio) with a temp HOME and
//! EASY_NOTE_DIR; the real ones are never touched (NFR-003). The local zone of the binary is
//! pinned with `TZ`, so `created` can be asserted to the second.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

const BIN: &str = env!("CARGO_BIN_EXE_easy-note");
/// POSIX zone string: UTC+07:00 with no tz database needed.
const ZONE: &str = "ICT-7";
const AWS: &str = concat!("AK", "IAIOSFODNN7EXAMPLE");

struct Sandbox {
    root: PathBuf,
    home: PathBuf,
    notes: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let root =
            std::env::temp_dir().join(format!("easy-note-inbox-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let home = root.join("home");
        let inbox = root.join("notes").join("inbox");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&inbox).unwrap();
        Sandbox {
            notes: root.join("notes"),
            home,
            root,
        }
    }

    fn inbox(&self) -> PathBuf {
        self.notes.join("inbox")
    }

    /// A file a sync app dropped in `inbox/`, with a fixed mtime.
    fn drop_file(&self, name: &str, content: &[u8]) -> PathBuf {
        let path = self.inbox().join(name);
        std::fs::write(&path, content).unwrap();
        set_mtime(&path, 1_772_712_000); // 2026-03-05T12:00:00Z
        path
    }

    fn trash(&self) -> PathBuf {
        self.notes.join(".easy-note").join("trash")
    }

    fn cmd(&self, program: &str, args: &[&str]) -> Command {
        let mut cmd = Command::new(program);
        cmd.args(args)
            .env("HOME", &self.home)
            .env("EASY_NOTE_DIR", &self.notes)
            .env("TZ", ZONE)
            .env_remove("EDITOR")
            .env_remove("VISUAL")
            .current_dir(&self.home)
            .stdin(Stdio::null());
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.cmd(BIN, args).output().unwrap()
    }

    /// Exit 0 asserted; returns stdout.
    fn ok(&self, args: &[&str]) -> String {
        let out = self.run(args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{args:?}: {}",
            text(&out.stderr)
        );
        text(&out.stdout)
    }

    /// The exit code asserted, no stdout; returns stderr.
    fn fails(&self, args: &[&str], code: i32) -> String {
        let out = self.run(args);
        assert_eq!(
            out.status.code(),
            Some(code),
            "{args:?}: {} {}",
            text(&out.stdout),
            text(&out.stderr)
        );
        assert!(out.stdout.is_empty(), "no stdout: {args:?}");
        text(&out.stderr)
    }

    /// `easy-note serve` over stdio: one `tools/call` per entry; (is_error, text) of each answer.
    fn mcp(&self, calls: &[(&str, Value)]) -> Vec<(bool, String)> {
        let mut child = self
            .cmd(BIN, &["serve"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let mut send = |id: Option<u64>, method: &str, params: Value| -> Option<Value> {
            let mut request = json!({"jsonrpc": "2.0", "method": method, "params": params});
            if let Some(id) = id {
                request["id"] = id.into();
            }
            writeln!(stdin, "{request}").unwrap();
            stdin.flush().unwrap();
            id.map(|_| serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap())
        };
        send(
            Some(1),
            "initialize",
            json!({"protocolVersion": "2025-03-26", "capabilities": {},
                "clientInfo": {"name": "inbox-test", "version": "0"}}),
        );
        send(None, "notifications/initialized", json!({}));
        let mut answers = vec![];
        for (n, (tool, args)) in calls.iter().enumerate() {
            let answer = send(
                Some(n as u64 + 2),
                "tools/call",
                json!({"name": tool, "arguments": args}),
            )
            .unwrap();
            answers.push(match answer.get("result") {
                Some(result) => (
                    result["isError"].as_bool().unwrap_or(false),
                    result["content"][0]["text"].as_str().unwrap().to_string(),
                ),
                None => (
                    true,
                    answer["error"]["message"].as_str().unwrap().to_string(),
                ),
            });
        }
        drop(stdin);
        assert!(child.wait().unwrap().success());
        answers
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn set_mtime(path: &Path, secs: u64) {
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(UNIX_EPOCH + Duration::from_secs(secs))
        .unwrap();
}

/// Every entry directly in `dir`, with its bytes and mtime, plus the dir's own mtime.
type Snapshot = (SystemTime, BTreeMap<String, (Vec<u8>, SystemTime)>);

fn snapshot(dir: &Path) -> Snapshot {
    let files = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| {
            let path = e.unwrap().path();
            let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
            let bytes = std::fs::read(&path).unwrap_or_default(); // a subfolder has none
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                (bytes, mtime),
            )
        })
        .collect();
    (std::fs::metadata(dir).unwrap().modified().unwrap(), files)
}

fn ids(listing: &str) -> Vec<String> {
    listing
        .lines()
        .map(|l| l.split_whitespace().next().unwrap().to_string())
        .collect()
}

/// A tool answer is JSON text.
fn json_of(answer: &(bool, String)) -> Value {
    assert!(!answer.0, "tool error: {}", answer.1);
    serde_json::from_str(&answer.1).unwrap()
}

// ---------------------------------------------------------------------------------------------
// AC 1-3: what a note looks like
// ---------------------------------------------------------------------------------------------

#[test]
fn ac1_a_timestamp_name_gives_the_id_and_the_local_created_time() {
    let sb = Sandbox::new("ac1");
    sb.drop_file("20261114-183002.txt", b"phone idea #mobile");
    let shown = sb.ok(&["show", "inbox/20261114-183002"]);
    assert!(
        shown.contains("\nid: \"inbox/20261114-183002\"\n"),
        "{shown}"
    );
    assert!(
        shown.contains("\ncreated: \"2026-11-14T18:30:02+07:00\"\n"),
        "{shown}"
    );
    assert!(shown.contains("\nsource: \"inbox\"\n"), "{shown}");
    assert!(shown.ends_with("---\nphone idea #mobile"), "{shown}");
    for key in ["repo", "remote", "branch", "task"] {
        assert!(!shown.contains(&format!("\n{key}:")), "no {key}: {shown}");
    }
}

#[test]
fn ac2_a_name_without_a_time_prefix_or_with_a_bad_date_takes_the_file_mtime() {
    let sb = Sandbox::new("ac2");
    sb.drop_file("ideas.md", b"no prefix\n");
    sb.drop_file("20269999-999999.txt", b"bad date\n");
    // 2026-03-05T12:00:00Z is 19:00 in the pinned zone
    for id in ["inbox/ideas", "inbox/20269999-999999"] {
        let shown = sb.ok(&["show", id]);
        assert!(
            shown.contains("\ncreated: \"2026-03-05T19:00:00+07:00\"\n"),
            "{id}: {shown}"
        );
    }
}

#[test]
fn ac3_hash_words_become_tags_and_valid_front_matter_wins() {
    let sb = Sandbox::new("ac3");
    sb.drop_file("a.txt", b"fix #Flaky test #ui/wait and #flaky again\n");
    let shown = sb.ok(&["show", "inbox/a"]);
    assert!(
        shown.contains("\ntags: [\"flaky\", \"ui/wait\"]\n"),
        "{shown}"
    );

    let fm = "---\neasy-note: 1\nid: \"bogus-id\"\ncreated: \"2026-01-02T03:04:05+00:00\"\n\
              repo: \"easy-ui-mcp\"\ntags: [\"real\"]\nsource: \"inbox\"\n---\nbody with #other\n";
    sb.drop_file("b.md", fm.as_bytes());
    let shown = sb.ok(&["show", "inbox/b"]);
    assert!(
        shown.contains("\nid: \"inbox/b\"\n"),
        "id stays the inbox id: {shown}"
    );
    assert!(
        shown.contains("\ncreated: \"2026-01-02T03:04:05+00:00\"\n"),
        "{shown}"
    );
    assert!(shown.contains("\nrepo: \"easy-ui-mcp\"\n"), "{shown}");
    assert!(shown.contains("\ntags: [\"real\"]\n"), "{shown}");
    assert!(shown.ends_with("---\nbody with #other\n"), "{shown}");

    // front matter that is not `easy-note: 1`: the whole file is the text
    let v2 = "---\neasy-note: 2\nid: x\n---\nhello #t\n";
    sb.drop_file("c.md", v2.as_bytes());
    let shown = sb.ok(&["show", "inbox/c"]);
    assert!(shown.contains("\ntags: [\"t\"]\n"), "{shown}");
    assert!(shown.ends_with(&format!("---\n{v2}")), "{shown}");
}

// ---------------------------------------------------------------------------------------------
// AC 4: the read commands and tools include inbox notes
// ---------------------------------------------------------------------------------------------

fn three_files(sb: &Sandbox) {
    sb.drop_file("20261114-183002.txt", b"phone idea #mobile");
    sb.drop_file("20261115-090000.md", b"second needle #mobile\n");
    sb.drop_file("plain.txt", b"third one\n");
}

#[test]
fn ac4_cli_ls_search_stats_and_show_include_the_inbox_notes() {
    let sb = Sandbox::new("ac4-cli");
    three_files(&sb);
    let listed = ids(&sb.ok(&["ls"]));
    assert_eq!(
        listed,
        [
            "inbox/20261115-090000",
            "inbox/20261114-183002",
            "inbox/plain"
        ]
    );
    // a dated note beside them
    let added = sb.ok(&["add", "dated note"]);
    assert_eq!(ids(&sb.ok(&["ls"])).len(), 4, "{added}");
    assert_eq!(
        ids(&sb.ok(&["search", "needle"])),
        ["inbox/20261115-090000"]
    );
    assert_eq!(ids(&sb.ok(&["ls", "--tag", "mobile"])).len(), 2);
    assert_eq!(ids(&sb.ok(&["ls", "--source", "inbox"])).len(), 3);
    let stats = sb.ok(&["stats"]);
    assert!(stats.starts_with("4 notes"), "{stats}");
    assert!(stats.contains("source: inbox (3), cli (1)"), "{stats}");
    assert!(stats.contains("tag: mobile (2)"), "{stats}");
    assert!(sb.ok(&["show", "inbox/plain"]).contains("third one"));
}

#[test]
fn ac4_mcp_list_search_stats_and_read_include_the_inbox_notes() {
    let sb = Sandbox::new("ac4-mcp");
    three_files(&sb);
    let answers = sb.mcp(&[
        ("note_list", json!({})),
        ("note_search", json!({"query": "needle"})),
        ("note_stats", json!({})),
        ("note_read", json!({"id": "inbox/20261114-183002"})),
    ]);
    let list = json_of(&answers[0]);
    assert_eq!(list["total"], 3);
    assert_eq!(list["items"][0]["id"], "inbox/20261115-090000");
    assert_eq!(
        json_of(&answers[1])["items"][0]["id"],
        "inbox/20261115-090000"
    );
    let stats = json_of(&answers[2]);
    assert_eq!(stats["total"], 3);
    assert_eq!(stats["by_source"]["inbox"], 3);
    let read = json_of(&answers[3]);
    assert!(read.to_string().contains("phone idea"), "{read}");
}

// ---------------------------------------------------------------------------------------------
// AC 5: no read changes the inbox
// ---------------------------------------------------------------------------------------------

#[test]
fn ac5_every_read_command_and_tool_leaves_the_inbox_byte_identical() {
    let sb = Sandbox::new("ac5");
    three_files(&sb);
    sb.drop_file("secret.txt", format!("key {AWS} #leak\n").as_bytes());
    sb.drop_file("front.md", b"---\neasy-note: 1\nid: \"x\"\ncreated: \"2026-01-02T03:04:05+00:00\"\ntags: []\nsource: \"inbox\"\n---\nhi\n");
    sb.drop_file("bad.txt", &[0xff, 0xfe, 0x41]);
    let before = snapshot(&sb.inbox());
    for args in [
        vec!["ls"],
        vec!["ls", "--tag", "mobile"],
        vec!["search", "needle"],
        vec!["search", "leak"],
        vec!["show", "inbox/plain"],
        vec!["show", "inbox/secret"],
        vec!["show", "inbox/front"],
        vec!["stats"],
        vec!["stats", "--json"],
        vec!["doctor"],
        vec!["path"],
        vec!["trash"],
    ] {
        sb.run(&args);
        assert!(
            snapshot(&sb.inbox()) == before,
            "{args:?} changed the inbox"
        );
    }
    sb.mcp(&[
        ("note_list", json!({})),
        ("note_search", json!({"query": "leak"})),
        ("note_stats", json!({})),
        ("note_read", json!({"id": "inbox/secret"})),
        ("note_read", json!({"id": "inbox/plain"})),
    ]);
    assert!(
        snapshot(&sb.inbox()) == before,
        "an MCP read changed the inbox"
    );
}

// ---------------------------------------------------------------------------------------------
// AC 6: rm and restore
// ---------------------------------------------------------------------------------------------

#[test]
fn ac6_rm_moves_the_file_to_the_trash_and_restore_puts_it_back() {
    let sb = Sandbox::new("ac6");
    let path = sb.drop_file("20261114-183002.txt", b"phone idea #mobile");
    let bytes = std::fs::read(&path).unwrap();
    sb.drop_file("keep.md", b"stays\n");

    let said = sb.ok(&["rm", "inbox/20261114-183002"]);
    assert!(
        said.starts_with("✓ removed inbox/20261114-183002"),
        "{said}"
    );
    assert!(!path.exists());
    assert_eq!(
        std::fs::read(sb.trash().join("inbox__20261114-183002.txt")).unwrap(),
        bytes
    );
    assert_eq!(ids(&sb.ok(&["ls"])), ["inbox/keep"]);
    assert_eq!(sb.ok(&["trash"]), "inbox/20261114-183002\n");
    sb.fails(&["show", "inbox/20261114-183002"], 1);

    // a new file under the same name blocks the restore; nothing is replaced
    std::fs::write(&path, b"new file").unwrap();
    sb.fails(&["restore", "inbox/20261114-183002"], 1);
    assert_eq!(std::fs::read(&path).unwrap(), b"new file");
    assert!(sb.trash().join("inbox__20261114-183002.txt").exists());
    std::fs::remove_file(&path).unwrap();

    let said = sb.ok(&["restore", "inbox/20261114-183002"]);
    assert_eq!(said, "✓ restored inbox/20261114-183002\n");
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert!(std::fs::read_dir(sb.trash()).unwrap().next().is_none());
    assert_eq!(ids(&sb.ok(&["ls"])).len(), 2);
    sb.fails(&["restore", "inbox/20261114-183002"], 1);
    sb.fails(&["rm", "inbox/nothing-here"], 1);
}

#[test]
fn trash_empty_prints_the_inbox_id_not_the_trash_name() {
    let sb = Sandbox::new("ac6-empty");
    sb.drop_file("a.txt", b"x\n");
    sb.ok(&["rm", "inbox/a"]);
    let out = sb.ok(&["trash", "--empty", "--older-than", "0d"]);
    assert!(out.contains("deleted inbox/a\n"), "{out}");
    assert!(!out.contains("inbox__"), "{out}");
}

// ---------------------------------------------------------------------------------------------
// AC 7 and the edge cases: which files count, same stem, unreadable files
// ---------------------------------------------------------------------------------------------

#[test]
fn ac7_subfolders_dot_files_symlinks_and_other_extensions_are_ignored() {
    let sb = Sandbox::new("ac7");
    sb.drop_file("ok.txt", b"counted\n");
    sb.drop_file("also.md", b"counted\n");
    for name in [
        "photo.png",
        "notes.docx",
        "UPPER.TXT",
        "x.md.icloud",
        ".hidden.txt",
        ".~lock.a.txt#",
        ".md",
        "txt",
    ] {
        sb.drop_file(name, b"ignored\n");
    }
    std::fs::create_dir_all(sb.inbox().join("sub")).unwrap();
    std::fs::write(sb.inbox().join("sub").join("deep.txt"), b"ignored\n").unwrap();
    std::fs::create_dir_all(sb.inbox().join("dir.md")).unwrap();
    let outside = sb.root.join("outside.txt");
    std::fs::write(&outside, b"outside the inbox\n").unwrap();
    std::os::unix::fs::symlink(&outside, sb.inbox().join("link.txt")).unwrap();

    let mut listed = ids(&sb.ok(&["ls"]));
    listed.sort();
    assert_eq!(listed, ["inbox/also", "inbox/ok"]);
    sb.fails(&["show", "inbox/link"], 1);
    sb.fails(&["show", "inbox/deep"], 1);
    assert!(sb.ok(&["stats"]).starts_with("2 notes"));
    // not counted as skipped either: they are not notes
    assert!(!sb.ok(&["stats"]).contains("skipped"));
}

#[test]
fn an_md_and_a_txt_with_one_stem_get_ids_with_the_extension() {
    let sb = Sandbox::new("same-stem");
    sb.drop_file("same.md", b"from md\n");
    sb.drop_file("same.txt", b"from txt\n");
    sb.drop_file("solo.txt", b"alone\n");
    let mut listed = ids(&sb.ok(&["ls"]));
    listed.sort();
    assert_eq!(listed, ["inbox/same.md", "inbox/same.txt", "inbox/solo"]);
    assert!(sb.ok(&["show", "inbox/same.md"]).contains("from md"));
    assert!(sb.ok(&["show", "inbox/same.txt"]).contains("from txt"));
    // the bare stem names two files: the error names both ids and nothing is shown
    let err = sb.fails(&["show", "inbox/same"], 2);
    assert!(
        err.contains("inbox/same.md") && err.contains("inbox/same.txt"),
        "{err}"
    );
    sb.fails(&["rm", "inbox/same"], 2);
    assert!(sb.inbox().join("same.md").exists() && sb.inbox().join("same.txt").exists());
    // the file name works for a file that has no twin
    assert!(sb.ok(&["show", "inbox/solo.txt"]).contains("alone"));
    // trash and restore keep the extension
    sb.ok(&["rm", "inbox/same.txt"]);
    assert!(sb.trash().join("inbox__same.txt").exists());
    assert_eq!(sb.ok(&["trash"]), "inbox/same\n");
    sb.ok(&["restore", "inbox/same"]);
    assert!(sb.inbox().join("same.txt").exists());
}

#[test]
fn a_non_utf8_or_empty_file_is_skipped_with_its_path_and_never_quoted() {
    let sb = Sandbox::new("skipped");
    sb.drop_file("good.txt", b"fine\n");
    sb.drop_file(
        "binary.txt",
        &[0xff, 0xfe, b'S', b'E', b'C', b'R', b'E', b'T'],
    );
    sb.drop_file("empty.md", b"");
    sb.drop_file("blank.md", b" \n\n");
    let stats = sb.ok(&["stats"]);
    assert!(stats.starts_with("1 note"), "{stats}");
    assert!(stats.contains("skipped: 3"), "{stats}");
    assert!(stats.contains("binary.txt (not UTF-8)"), "{stats}");
    assert!(stats.contains("empty.md (empty file)"), "{stats}");
    assert!(!stats.contains("SECRET"), "{stats}");
    assert_eq!(ids(&sb.ok(&["ls"])), ["inbox/good"]);
}

// ---------------------------------------------------------------------------------------------
// AC 9: an id cannot leave `inbox/`
// ---------------------------------------------------------------------------------------------

const ESCAPES: [&str; 8] = [
    "inbox/../x",
    "inbox/a/b",
    "inbox/.hidden",
    "inbox/",
    "inbox/..",
    "inbox/.",
    "inbox/../.easy-note/trash/inbox__a.txt",
    "inbox//x",
];

#[test]
fn ac9_an_id_that_escapes_the_inbox_is_a_bad_id_in_the_cli() {
    let sb = Sandbox::new("ac9-cli");
    // Files a path join would reach: a dated-looking note and a plain file beside the inbox.
    std::fs::write(sb.notes.join("x.md"), "outside A\n").unwrap();
    std::fs::write(sb.notes.join("x.txt"), "outside B\n").unwrap();
    std::fs::write(sb.inbox().join(".hidden.txt"), "hidden\n").unwrap();
    sb.drop_file("a.txt", b"inside\n");
    let outside = snapshot(&sb.notes);
    for id in ESCAPES {
        for cmd in ["show", "rm", "restore"] {
            let err = sb.fails(&[cmd, id], 2);
            assert!(err.contains("not a note id"), "{cmd} {id}: {err}");
        }
        let err = sb.fails(&["tag", id, "x"], 2);
        assert!(err.contains("not a note id"), "tag {id}: {err}");
    }
    // nothing was read, moved or written, inside or outside the inbox
    assert!(snapshot(&sb.notes).1 == outside.1);
    assert!(!sb.trash().exists());
    assert!(sb.notes.join("x.md").exists() && sb.notes.join("x.txt").exists());
    assert!(sb.inbox().join(".hidden.txt").exists());
}

#[test]
fn ac9_an_id_that_escapes_the_inbox_is_a_bad_id_in_mcp() {
    let sb = Sandbox::new("ac9-mcp");
    std::fs::write(sb.notes.join("x.md"), "outside A\n").unwrap();
    sb.drop_file("a.txt", b"inside\n");
    let calls: Vec<(&str, Value)> = ESCAPES
        .iter()
        .map(|id| ("note_read", json!({"id": id})))
        .collect();
    for (id, (is_error, message)) in ESCAPES.iter().zip(sb.mcp(&calls)) {
        assert!(is_error, "{id}");
        assert!(message.contains("not a note id"), "{id}: {message}");
        assert!(!message.contains("outside"), "{id}: {message}");
    }
}

// ---------------------------------------------------------------------------------------------
// AC 10: inbox notes are read-only
// ---------------------------------------------------------------------------------------------

#[test]
fn ac10_edit_and_tag_on_an_inbox_id_exit_1_before_anything_runs_or_changes() {
    let sb = Sandbox::new("ac10");
    sb.drop_file("a.txt", b"inside #x\n");
    let before = snapshot(&sb.inbox());
    // an editor that leaves a marker if it is ever started
    let marker = sb.home.join("editor-ran");
    let editor = sb.home.join("editor.sh");
    std::fs::write(&editor, format!("#!/bin/sh\ntouch {}\n", marker.display())).unwrap();
    std::fs::set_permissions(&editor, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();

    let out = sb
        .cmd(BIN, &["edit", "inbox/a"])
        .env("EDITOR", &editor)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("inbox notes are read-only"),
        "{}",
        text(&out.stderr)
    );
    assert!(!marker.exists(), "the editor must not start");
    for args in [
        vec!["tag", "inbox/a", "new"],
        vec!["tag", "inbox/a", "--remove", "x"],
    ] {
        let err = sb.fails(&args, 1);
        assert!(err.contains("inbox notes are read-only"), "{args:?}: {err}");
    }
    assert!(snapshot(&sb.inbox()) == before, "the file changed");
    assert!(!sb.trash().exists());
}

#[test]
fn add_never_writes_into_the_inbox() {
    let sb = Sandbox::new("add");
    sb.ok(&["add", "typed note"]);
    assert!(std::fs::read_dir(sb.inbox()).unwrap().next().is_none());
}

#[test]
fn a_secret_in_an_inbox_file_is_masked_on_every_read_and_stays_in_the_file() {
    let sb = Sandbox::new("mask");
    let path = sb.drop_file("s.txt", format!("deploy with {AWS} now #ops\n").as_bytes());
    for args in [
        vec!["show", "inbox/s"],
        vec!["ls"],
        vec!["search", "deploy"],
    ] {
        let out = sb.ok(&args);
        assert!(!out.contains(AWS), "{args:?}: {out}");
    }
    assert!(sb.ok(&["show", "inbox/s"]).contains("[masked:"));
    let answers = sb.mcp(&[
        ("note_read", json!({"id": "inbox/s"})),
        ("note_list", json!({})),
    ]);
    for (is_error, body) in answers {
        assert!(!is_error && !body.contains(AWS), "{body}");
    }
    // easy-note never rewrites the file, even to mask it
    assert!(std::fs::read_to_string(&path).unwrap().contains(AWS));
}
