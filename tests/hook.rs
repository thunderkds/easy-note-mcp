//! HUN-259: the Claude Code `UserPromptSubmit` hook, `easy-note hook-prompt`, through the real
//! binary (FR-018, NFR-009). Events are the recorded ones in `tests/fixtures/claude_hook/`; only
//! the `cwd` is swapped for a real temp folder, because the fixtures hold scrubbed `/work/...`
//! paths. Tests use a temp HOME, EASY_NOTE_DIR and git repos; they never touch real ones.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

const AWS: &str = concat!("AK", "IAIOSFODNN7EXAMPLE");

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_easy-note"))
}

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/claude_hook")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// The recorded event with its `cwd` replaced.
fn event(name: &str, cwd: &Path) -> String {
    let mut ev = fixture(name);
    ev["cwd"] = json!(cwd.to_string_lossy());
    ev.to_string()
}

/// A temp root with a HOME and a notes dir path (not created).
struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        Sandbox {
            dir: tempfile::Builder::new()
                .prefix(&format!("easy-note-hook-{name}-"))
                .tempdir()
                .unwrap(),
        }
    }
    fn root(&self) -> &Path {
        self.dir.path()
    }
    fn home(&self) -> PathBuf {
        let home = self.root().join("home");
        std::fs::create_dir_all(&home).unwrap();
        home
    }
    fn notes(&self) -> PathBuf {
        self.root().join("notes")
    }
}

fn base_command(sb: &Sandbox) -> Command {
    let mut cmd = Command::new(bin());
    cmd.arg("hook-prompt")
        .env("HOME", sb.home())
        .env("EASY_NOTE_DIR", sb.notes())
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .current_dir(sb.home());
    cmd
}

fn run_with(mut cmd: Command, stdin: &[u8]) -> Output {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // The hook may exit before it reads everything; a broken pipe here is not a failure.
    let _ = child.stdin.take().unwrap().write_all(stdin);
    child.wait_with_output().unwrap()
}

fn hook(sb: &Sandbox, stdin: &str) -> Output {
    run_with(base_command(sb), stdin.as_bytes())
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn git(cwd: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t.invalid"])
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
}

/// A repo `<root>/proj` on branch `main`, a linked worktree `<root>/proj-wt` on `fix/t120-probe`,
/// and `<root>/proj-wt/sub/deep`, as in the recorded fixtures. Returns (worktree, subdir).
fn repo_and_worktree(root: &Path) -> (PathBuf, PathBuf) {
    let main = root.join("proj");
    std::fs::create_dir_all(&main).unwrap();
    git(&main, &["init", "-q", "-b", "main"]);
    git(&main, &["commit", "-q", "--allow-empty", "-m", "start"]);
    let wt = root.join("proj-wt");
    git(
        &main,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "fix/t120-probe",
            wt.to_str().unwrap(),
        ],
    );
    let deep = wt.join("sub/deep");
    std::fs::create_dir_all(&deep).unwrap();
    (wt, deep)
}

fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut all = vec![];
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                all.push(p);
            }
        }
    }
    all.sort();
    all
}

fn note_files(sb: &Sandbox) -> Vec<PathBuf> {
    files_under(&sb.notes())
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect()
}

// ---------------------------------------------------------------------------------------------
// AC 1: the prefix saves one note and blocks the prompt
// ---------------------------------------------------------------------------------------------

#[test]
fn ac1_prefix_prompt_saves_one_hook_note_and_blocks_with_the_confirmation() {
    let cases = [
        (
            "note_prefix_linked_worktree.json",
            false,
            "retry bug on firefox",
        ),
        ("note_prefix_from_subdir.json", true, "from a subdir"),
        (
            "note_prefix_queued_while_working.json",
            false,
            "queued while working",
        ),
    ];
    for (name, deep, text) in cases {
        let sb = Sandbox::new("ac1");
        let (wt, sub) = repo_and_worktree(sb.root());
        let cwd = if deep { &sub } else { &wt };
        let o = hook(&sb, &event(name, cwd));
        assert_eq!(o.status.code(), Some(0), "{name}");
        let stdout = out(&o);
        assert_eq!(stdout.lines().count(), 1, "{name}: {stdout:?}");
        assert!(
            stdout.starts_with(r#"{"decision":"block","reason":"✓ noted "#),
            "{name}: {stdout}"
        );
        assert!(stdout.ends_with('\n'));

        let files = note_files(&sb);
        assert_eq!(files.len(), 1, "{name}: {files:?}");
        let id = files[0].file_stem().unwrap().to_string_lossy().into_owned();
        let want = json!({"decision":"block","reason":format!("✓ noted {id} · proj · T120")});
        assert_eq!(
            serde_json::from_str::<Value>(&stdout).unwrap(),
            want,
            "{name}"
        );

        let saved = std::fs::read_to_string(&files[0]).unwrap();
        assert!(saved.contains("source: \"hook\""), "{saved}");
        assert!(saved.contains("repo: \"proj\""), "{saved}");
        assert!(saved.contains("branch: \"fix/t120-probe\""), "{saved}");
        assert!(saved.contains("task: \"T120\""), "{saved}");
        assert!(saved.contains(text), "{saved}");
        assert!(!saved.contains("/note "), "{saved}");
        // FR-004: no filesystem path in a note
        assert!(!saved.contains(sb.root().to_str().unwrap()), "{saved}");
    }
}

#[test]
fn ac1_the_reason_omits_a_missing_repo_and_task_with_their_separator() {
    let sb = Sandbox::new("ac1-norepo");
    let plain = sb.root().join("plain");
    std::fs::create_dir_all(&plain).unwrap();
    let o = hook(&sb, &event("note_prefix_linked_worktree.json", &plain));
    assert_eq!(o.status.code(), Some(0));
    let files = note_files(&sb);
    assert_eq!(files.len(), 1);
    let id = files[0].file_stem().unwrap().to_string_lossy().into_owned();
    let v: Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(v["reason"], format!("✓ noted {id}"));
}

// ---------------------------------------------------------------------------------------------
// AC 2: any other prompt passes
// ---------------------------------------------------------------------------------------------

#[test]
fn ac2_other_prompts_print_nothing_exit_0_and_write_no_file() {
    for name in [
        "passthrough_main_repo.json",
        "passthrough_linked_worktree.json",
        "unknown_slash_command.json",
    ] {
        let sb = Sandbox::new("ac2");
        let o = hook(&sb, &event(name, &sb.home()));
        assert_eq!(o.status.code(), Some(0), "{name}");
        assert_eq!(out(&o), "", "{name}");
        assert!(!sb.notes().exists(), "{name}: the notes dir was created");
    }
}

#[test]
fn ac2_a_longer_command_a_bare_prefix_and_empty_text_do_not_match() {
    for prompt in [
        "/notes x",
        "/note",
        "/note ",
        "/note    \n\t ",
        " /note x",
        "NOTE x",
    ] {
        let sb = Sandbox::new("ac2-edge");
        let ev = json!({"prompt": prompt, "cwd": sb.home().to_string_lossy()}).to_string();
        let o = hook(&sb, &ev);
        assert_eq!(o.status.code(), Some(0), "{prompt:?}");
        assert_eq!(out(&o), "", "{prompt:?}");
        assert!(!sb.notes().exists(), "{prompt:?}: a note was saved");
    }
}

#[test]
fn ac2_the_note_text_is_the_rest_trimmed() {
    let sb = Sandbox::new("ac2-trim");
    let ev = json!({"prompt": "/note   padded text  \n", "cwd": sb.home().to_string_lossy()});
    let o = hook(&sb, &ev.to_string());
    assert_eq!(o.status.code(), Some(0));
    let saved = std::fs::read_to_string(&note_files(&sb)[0]).unwrap();
    assert!(saved.ends_with("padded text\n"), "{saved:?}");
}

// ---------------------------------------------------------------------------------------------
// AC 3: errors never stop a prompt
// ---------------------------------------------------------------------------------------------

#[test]
fn ac3_malformed_stdin_lets_the_prompt_pass_silently() {
    let bad: [&[u8]; 8] = [
        b"",
        b"not json",
        b"{\"prompt\":\"/note x\"",
        b"[]",
        b"{}",
        b"{\"prompt\":5}",
        b"{\"prompt\":null,\"cwd\":\"/\"}",
        b"\xff\xfe{\"prompt\":\"/note x\"}",
    ];
    for stdin in bad {
        let sb = Sandbox::new("ac3-bad");
        let o = run_with(base_command(&sb), stdin);
        assert_eq!(o.status.code(), Some(0), "{stdin:?}");
        assert_eq!(out(&o), "", "{stdin:?}");
        assert!(!sb.notes().exists(), "{stdin:?}");
    }
}

#[test]
fn ac3_a_core_error_after_a_match_passes_the_prompt_and_shows_the_error() {
    let sb = Sandbox::new("ac3-core");
    // The notes dir sits under a regular file, so the write fails.
    let blocker = sb.root().join("blocker");
    std::fs::write(&blocker, "x").unwrap();
    let mut cmd = base_command(&sb);
    cmd.env("EASY_NOTE_DIR", blocker.join("notes"));
    let o = run_with(
        cmd,
        event("note_prefix_linked_worktree.json", &sb.home()).as_bytes(),
    );
    assert_eq!(o.status.code(), Some(0));
    let stdout = out(&o);
    assert_eq!(stdout.lines().count(), 1, "{stdout:?}");
    let v: Value = serde_json::from_str(&stdout).unwrap();
    assert!(v.get("decision").is_none(), "the prompt must pass: {v}");
    let reason = v["reason"].as_str().unwrap();
    assert!(reason.starts_with("easy-note: "), "{reason}");
    assert!(reason.ends_with(" — note not saved"), "{reason}");
}

#[test]
fn ac3_the_error_reason_is_masked() {
    let sb = Sandbox::new("ac3-mask");
    // The config error names its path; a secret in the path must not reach stdout.
    let home = sb.root().join(AWS).join("home");
    std::fs::create_dir_all(home.join(".config/easy-note/config.toml")).unwrap();
    let mut cmd = base_command(&sb);
    cmd.env("HOME", &home)
        .env_remove("EASY_NOTE_DIR")
        .current_dir(&home);
    let o = run_with(cmd, br#"{"prompt":"/note x","cwd":"/"}"#);
    assert_eq!(o.status.code(), Some(0));
    let stdout = out(&o);
    let v: Value = serde_json::from_str(&stdout).unwrap();
    assert!(v.get("decision").is_none(), "{v}");
    let reason = v["reason"].as_str().unwrap();
    assert!(reason.starts_with("easy-note: "), "{reason}");
    assert!(reason.contains("[masked:aws_key]"), "{reason}");
    assert!(!stdout.contains(AWS), "{stdout}");
    assert!(!String::from_utf8_lossy(&o.stderr).contains(AWS));
}

// ---------------------------------------------------------------------------------------------
// AC 4: the pass path opens no store and touches no file
// ---------------------------------------------------------------------------------------------

/// The pass path with `EASY_NOTE_DIR` pointing at a missing dir and a `config.toml` that cannot
/// be read (it is a directory): still exit 0, empty stdout, nothing created.
fn broken_setup(sb: &Sandbox) -> Command {
    let home = sb.home();
    std::fs::create_dir_all(home.join(".config/easy-note/config.toml")).unwrap();
    let mut cmd = base_command(sb);
    cmd.env("EASY_NOTE_DIR", sb.root().join("missing/notes"));
    cmd
}

fn tree(root: &Path) -> Vec<(PathBuf, bool)> {
    let mut all = vec![];
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            let is_dir = p.is_dir();
            all.push((p.clone(), is_dir));
            if is_dir {
                stack.push(p);
            }
        }
    }
    all.sort();
    all
}

#[test]
fn ac4_pass_path_with_a_missing_notes_dir_and_unreadable_config_creates_nothing() {
    let sb = Sandbox::new("ac4");
    let cmd = broken_setup(&sb);
    let before = tree(sb.root());
    let o = run_with(
        cmd,
        event("passthrough_main_repo.json", &sb.home()).as_bytes(),
    );
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(out(&o), "");
    assert_eq!(String::from_utf8_lossy(&o.stderr), "");
    assert_eq!(
        tree(sb.root()),
        before,
        "the pass path created or removed something"
    );
    assert!(!sb.root().join("missing").exists());
}

#[test]
fn ac4_pass_path_p95_is_under_the_bound() {
    let sb = Sandbox::new("ac4-time");
    let ev = event("passthrough_main_repo.json", &sb.home());
    let mut times: Vec<Duration> = vec![];
    for _ in 0..50 {
        let cmd = broken_setup(&sb);
        let start = Instant::now();
        let o = run_with(cmd, ev.as_bytes());
        times.push(start.elapsed());
        assert_eq!(o.status.code(), Some(0));
        assert_eq!(out(&o), "");
    }
    times.sort();
    let p95 = times[47];
    eprintln!(
        "pass path, 50 runs (debug binary, process start included): min {:?} median {:?} p95 {:?} max {:?}",
        times[0], times[25], p95, times[49]
    );
    // NFR-009 is p95 < 50 ms; the test bound is generous so a loaded CI machine does not flake.
    assert!(p95 < Duration::from_millis(250), "p95 {p95:?}");
}

// ---------------------------------------------------------------------------------------------
// AC 5: the prefix is configurable
// ---------------------------------------------------------------------------------------------

fn write_config(sb: &Sandbox, text: &str) {
    let dir = sb.home().join(".config/easy-note");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("config.toml"), text).unwrap();
}

#[test]
fn ac5_capture_prefix_in_config_replaces_the_default() {
    let sb = Sandbox::new("ac5");
    write_config(&sb, "capture_prefix = \";n \"\n");
    let at =
        |prompt: &str| json!({"prompt": prompt, "cwd": sb.home().to_string_lossy()}).to_string();

    let o = hook(&sb, &at("/note default prefix is off"));
    assert_eq!((o.status.code(), out(&o).as_str()), (Some(0), ""));
    assert!(!sb.notes().exists());

    let o = hook(&sb, &at(";n custom prefix works"));
    assert_eq!(o.status.code(), Some(0));
    assert!(
        out(&o).starts_with(r#"{"decision":"block","reason":"✓ noted "#),
        "{}",
        out(&o)
    );
    let files = note_files(&sb);
    assert_eq!(files.len(), 1);
    let saved = std::fs::read_to_string(&files[0]).unwrap();
    assert!(saved.contains("custom prefix works"), "{saved}");
    assert!(saved.contains("source: \"hook\""), "{saved}");
}

#[test]
fn ac5_default_prefix_is_note_space_and_a_bad_config_value_falls_back_to_it() {
    // an empty prefix would capture every prompt; a non-string or broken file is a config error
    for config in [
        "",
        "capture_prefix = \"\"\n",
        "capture_prefix = 5\n",
        "capture_prefix = [\"/n \"]\n",
        "this is = not toml [\n",
        "dir = \"/somewhere\"\n",
    ] {
        let sb = Sandbox::new("ac5-default");
        write_config(&sb, config);
        let at = |prompt: &str| {
            json!({"prompt": prompt, "cwd": sb.home().to_string_lossy()}).to_string()
        };
        let o = hook(&sb, &at("any other prompt"));
        assert_eq!(
            (o.status.code(), out(&o).as_str()),
            (Some(0), ""),
            "{config:?}"
        );
        assert!(!sb.notes().exists(), "{config:?}: a plain prompt was saved");
        // `dir` in the same file is not the hook's business: the notes dir env still wins
        let o = hook(&sb, &at("/note still the default"));
        assert_eq!(o.status.code(), Some(0), "{config:?}");
        assert!(
            out(&o).contains(r#""decision":"block""#),
            "{config:?}: {}",
            out(&o)
        );
        assert_eq!(note_files(&sb).len(), 1, "{config:?}");
    }
}

// ---------------------------------------------------------------------------------------------
// masking (also in leak.rs, AC 9)
// ---------------------------------------------------------------------------------------------

#[test]
fn a_secret_in_the_prompt_is_masked_in_the_file_and_the_reason_counts_it() {
    let sb = Sandbox::new("mask");
    let ev =
        json!({"prompt": format!("/note key {AWS} leaked"), "cwd": sb.home().to_string_lossy()});
    let o = hook(&sb, &ev.to_string());
    assert_eq!(o.status.code(), Some(0));
    let stdout = out(&o);
    assert!(!stdout.contains(AWS), "{stdout}");
    assert!(stdout.contains("1 secret masked: aws_key"), "{stdout}");
    let saved = std::fs::read_to_string(&note_files(&sb)[0]).unwrap();
    assert!(!saved.contains(AWS), "{saved}");
    assert!(saved.contains("[masked:aws_key]"), "{saved}");
}

#[test]
fn hash_words_in_the_text_become_tags_as_in_the_cli() {
    let sb = Sandbox::new("tags");
    let ev = json!({"prompt": "/note retry bug only on firefox #flaky", "cwd": sb.home().to_string_lossy()});
    let o = hook(&sb, &ev.to_string());
    assert_eq!(o.status.code(), Some(0));
    let saved = std::fs::read_to_string(&note_files(&sb)[0]).unwrap();
    let note = easy_note_mcp::core::format::parse(&saved).unwrap();
    match note.fields.get("tags") {
        Some(easy_note_mcp::core::format::Value::List(list)) => assert_eq!(list, &["flaky"]),
        other => panic!("tags: {other:?}"),
    }
}
