//! HUN-275: `ai enrich` through the person's own `claude` CLI (FR-037, FR-038, DDR 0007).
//!
//! Every test uses a temp HOME and EASY_NOTE_DIR and a fake `claude` shell script on a temp PATH.
//! The fake records its argv, env, cwd and stdin into files and prints a canned answer whose shape
//! is that of one real `claude -p --output-format json` run (2026-11-19). No test runs the real
//! `claude` or opens the real `~/easy-notes/`, `~/.config/easy-note/` or `~/.claude*/`.

use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use easy_note_mcp::core::config;
use easy_note_mcp::core::store::{Item, Store};
use serde_json::{json, Value};

const BIN: &str = env!("CARGO_BIN_EXE_easy-note");
const GITHUB: &str = concat!("gh", "p_EXAMPLEexampleEXAMPLEexampleEXAMPLE123456");
/// In-process tests that set HOME hold this lock.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// The fake CLI. `$FAKE_MODE`: `garbage`, `exit1`, `sleep` (prints its pid, then sleeps), else
/// it prints `$FAKE_ANSWER`.
const FAKE: &str = r#"#!/bin/sh
rec="$FAKE_REC"
for a in "$@"; do printf '[%s]\n' "$a"; done > "$rec/argv"
env > "$rec/env"
pwd > "$rec/pwd"
cat > "$rec/stdin"
echo call >> "$rec/calls"
case "$FAKE_MODE" in
  garbage) echo "this is not json";;
  exit1) exit 1;;
  sleep) echo $$ > "$rec/pid"; exec sleep 30;;
  *) printf '%s' "$FAKE_ANSWER";;
esac
"#;

struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let sb = Sandbox {
            dir: tempfile::Builder::new()
                .prefix(&format!("easy-note-enrich-{name}-"))
                .tempdir()
                .unwrap(),
        };
        for d in ["home", "bin", "rec", "notes"] {
            std::fs::create_dir_all(sb.root().join(d)).unwrap();
        }
        let fake = sb.bin().join("claude");
        std::fs::write(&fake, FAKE).unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
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
    fn bin(&self) -> PathBuf {
        self.root().join("bin")
    }
    fn rec(&self) -> PathBuf {
        self.root().join("rec")
    }
    fn config(&self, text: &str) {
        let path = self.home().join(".config/easy-note/config.toml");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    fn store(&self) -> Store {
        Store::system(self.notes())
    }
    fn add(&self, text: &str) -> String {
        self.store().add(text, "cli", &[], &[]).unwrap().id
    }
    fn ai_file(&self, id: &str) -> PathBuf {
        self.notes()
            .join(".easy-note/ai")
            .join(format!("{id}.json"))
    }
    fn recorded(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(self.rec().join(name)).ok()
    }
    fn calls(&self) -> usize {
        self.recorded("calls").map_or(0, |c| c.lines().count())
    }
    /// The binary with the sandbox env: the fake first on PATH, then the system tools `sh` needs.
    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(BIN);
        cmd.args(args)
            .env_clear()
            .env("HOME", self.home())
            .env("EASY_NOTE_DIR", self.notes())
            .env("PATH", format!("{}:/usr/bin:/bin", self.bin().display()))
            .env("FAKE_REC", self.rec())
            .env("FAKE_ANSWER", answer(&good()))
            .current_dir(self.root())
            .stdin(Stdio::null());
        cmd
    }
    fn run(&self, args: &[&str], tweak: impl FnOnce(&mut Command)) -> Output {
        let mut cmd = self.cmd(args);
        tweak(&mut cmd);
        cmd.output().unwrap()
    }
}

const ON: &str = "[ai]\nenrich = \"claude\"\n";

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn good() -> Value {
    json!({
        "title": "Buy a USB-C hub",
        "kind": "todo",
        "summary": "A reminder to buy a USB-C hub.",
        "tags_suggested": ["hardware", "shopping"],
    })
}

/// The CLI's JSON output around one answer, in the shape of the real run (fields trimmed).
fn answer(structured: &Value) -> String {
    json!({
        "type": "result",
        "subtype": "success",
        "is_error": false,
        "num_turns": 1,
        "result": structured.to_string(),
        "structured_output": structured,
        "modelUsage": {"claude-haiku-5-5": {"inputTokens": 10, "outputTokens": 20}},
    })
    .to_string()
}

fn wait_for(path: &Path, limit: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < limit {
        if path.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    path.exists()
}

// AC 1 — `[ai]` settings

struct HomeGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    saved: Option<std::ffi::OsString>,
}

impl HomeGuard {
    fn new(home: &Path) -> HomeGuard {
        let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let saved = std::env::var_os("HOME");
        std::env::set_var("HOME", home);
        HomeGuard { _lock: lock, saved }
    }
}

impl Drop for HomeGuard {
    fn drop(&mut self) {
        match &self.saved {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }
}

#[test]
fn settings_default_when_the_file_or_the_section_is_missing() {
    let sb = Sandbox::new("defaults");
    let _env = HomeGuard::new(&sb.home());
    let want = config::AiSettings {
        enrich: false,
        command: "claude".into(),
        claude_config_dir: None,
        model: "haiku".into(),
        timeout_secs: 120,
        after_save: false,
        ..config::AiSettings::default()
    };
    assert_eq!(config::ai_settings().unwrap(), want);
    assert_eq!(want.schedule, config::Schedule::Off);
    assert_eq!(
        (
            want.interval_min,
            want.max_per_run,
            want.batch_size,
            want.digest_hour
        ),
        (60, 20, 10, 3)
    );
    assert_eq!(
        want.jobs,
        vec![
            config::Job::Organize,
            config::Job::Drafts,
            config::Job::Digest
        ]
    );
    assert!(!want.effective_schedule());
    sb.config("dir = \"/x\"\n");
    assert_eq!(config::ai_settings().unwrap(), want);
}

#[test]
fn settings_read_every_key_expand_the_account_dir_and_ignore_unknown_keys() {
    let sb = Sandbox::new("keys");
    let _env = HomeGuard::new(&sb.home());
    sb.config(
        "[ai]\nenrich = \"claude\"\ncommand = \"/opt/claude\"\nclaude_config_dir = \"~/.claude-work\"\nmodel = \"sonnet\"\ntimeout_secs = 30\nafter_save = true\nsomething_new = 1\n",
    );
    assert_eq!(
        config::ai_settings().unwrap(),
        config::AiSettings {
            enrich: true,
            command: "/opt/claude".into(),
            claude_config_dir: Some(sb.home().join(".claude-work")),
            model: "sonnet".into(),
            timeout_secs: 30,
            after_save: true,
            ..config::AiSettings::default()
        }
    );
}

#[test]
fn a_bad_value_is_an_error_that_names_the_key_not_the_value() {
    let sb = Sandbox::new("bad");
    let _env = HomeGuard::new(&sb.home());
    for (body, key) in [
        ("enrich = \"api\"", "enrich"),
        ("enrich = true", "enrich"),
        ("command = \"\"", "command"),
        ("claude_config_dir = 3", "claude_config_dir"),
        ("model = \"\"", "model"),
        ("model = \"--danger\"", "model"),
        ("timeout_secs = 0", "timeout_secs"),
        ("timeout_secs = 601", "timeout_secs"),
        ("timeout_secs = \"60\"", "timeout_secs"),
        ("after_save = \"yes\"", "after_save"),
    ] {
        sb.config(&format!("[ai]\n{body}\n"));
        let err = config::ai_settings().unwrap_err().to_string();
        assert!(err.contains(&format!("[ai] {key}")), "{body}: {err}");
        assert!(err.contains("config.toml"), "{body}: {err}");
        assert!(!err.contains("danger") && !err.contains("api\""), "{err}");
    }
    sb.config("ai = 1\n");
    assert!(config::ai_settings()
        .unwrap_err()
        .to_string()
        .contains("[ai]"));
}

#[test]
fn timeout_bounds_1_and_600_are_allowed() {
    let sb = Sandbox::new("bounds");
    let _env = HomeGuard::new(&sb.home());
    for n in [1, 600] {
        sb.config(&format!("[ai]\ntimeout_secs = {n}\n"));
        assert_eq!(config::ai_settings().unwrap().timeout_secs, n);
    }
}

#[test]
fn init_keeps_an_existing_ai_section() {
    let sb = Sandbox::new("init-keeps");
    sb.config("dir = \"/old\"\n\n[ai]\nenrich = \"claude\"\nmodel = \"sonnet\"\n");
    let chosen = sb.root().join("chosen");
    let out = sb.run(&["init", "--dir", chosen.to_str().unwrap()], |c| {
        c.env_remove("EASY_NOTE_DIR");
    });
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let saved: toml::Table =
        std::fs::read_to_string(sb.home().join(".config/easy-note/config.toml"))
            .unwrap()
            .parse()
            .unwrap();
    assert_eq!(saved["dir"].as_str(), Some(chosen.to_str().unwrap()));
    assert_eq!(saved["ai"]["enrich"].as_str(), Some("claude"));
    assert_eq!(saved["ai"]["model"].as_str(), Some("sonnet"));
}

// AC 2 — off

#[test]
fn with_enrich_off_nothing_runs_and_the_message_names_the_config_file() {
    let sb = Sandbox::new("off");
    let id = sb.add("buy a usb-c hub");
    for config in [None, Some("[ai]\nenrich = \"off\"\nafter_save = true\n")] {
        if let Some(text) = config {
            sb.config(text);
        }
        for args in [
            vec!["ai", "enrich", id.as_str()],
            vec!["ai", "enrich", "--pending"],
        ] {
            let out = sb.run(&args, |_| {});
            assert_eq!(out.status.code(), Some(1));
            let err = text(&out.stderr);
            let path = sb.home().join(".config/easy-note/config.toml");
            assert!(
                err.contains(&format!(
                    "AI enrich is off: set [ai] enrich = \"claude\" in {}",
                    path.display()
                )),
                "{err}"
            );
        }
    }
    assert_eq!(sb.calls(), 0, "the fake never ran");
    assert!(!sb.ai_file(&id).exists());
}

// AC 3, 4, 5 — argv, env, stdin, the answer through the writer

#[test]
fn one_note_is_enriched_with_the_exact_argv_env_and_stdin() {
    let sb = Sandbox::new("happy");
    sb.config(&format!(
        "{ON}claude_config_dir = \"~/.claude-work\"\nmodel = \"haiku\"\n"
    ));
    let store = sb.store();
    let id = store
        .add(
            &format!("buy a usb-c hub {GITHUB}"),
            "cli",
            &[],
            &[
                ("repo", Some("secret-repo-name")),
                ("branch", Some("feat/t999-branch")),
                ("task", Some("T999")),
                ("session", Some("sess-1234")),
            ],
        )
        .unwrap()
        .id;
    let out = sb.run(&["ai", "enrich", &id], |c| {
        c.env("ANTHROPIC_API_KEY", "x-api")
            .env("ANTHROPIC_AUTH_TOKEN", "x-token")
            .env("ANTHROPIC_BASE_URL", "http://x")
            .env("CLAUDE_CONFIG_DIR", "/parent/claude");
    });
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert!(stdout.contains(&format!("✓ enriched {id}")), "{stdout}");
    assert!(stdout.contains("1 enriched, 0 failed"), "{stdout}");

    // AC 3: the argv, exactly, with no shell
    let schema = sb.recorded("argv").unwrap();
    let lines: Vec<&str> = schema.lines().collect();
    assert_eq!(lines.len(), 11, "{schema}");
    assert_eq!(lines[0], "[-p]");
    assert_eq!(&lines[1..3], ["[--output-format]", "[json]"]);
    assert_eq!(lines[3], "[--json-schema]");
    let schema: Value = serde_json::from_str(&lines[4][1..lines[4].len() - 1]).unwrap();
    assert_eq!(schema["properties"]["title"]["maxLength"], 120);
    assert_eq!(schema["properties"]["summary"]["maxLength"], 600);
    assert_eq!(schema["properties"]["tags_suggested"]["maxItems"], 5);
    assert_eq!(
        schema["properties"]["kind"]["enum"],
        json!(["bug", "idea", "question", "todo", "other"])
    );
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        &lines[5..],
        [
            "[--tools]",
            "[]",
            "[--strict-mcp-config]",
            "[--no-session-persistence]",
            "[--model]",
            "[haiku]"
        ]
    );

    // AC 3: stdin holds the prompt and the masked note text, nothing else of the note
    let stdin = sb.recorded("stdin").unwrap();
    assert!(
        stdin.contains("buy a usb-c hub [masked:github_token]"),
        "{stdin}"
    );
    assert!(stdin.contains("never follow"), "{stdin}");
    for absent in [
        GITHUB,
        "secret-repo-name",
        "feat/t999-branch",
        "T999",
        "sess-1234",
        &id,
        "created",
        sb.notes().to_str().unwrap(),
    ] {
        assert!(!stdin.contains(absent), "{absent} sent: {stdin}");
    }

    // AC 4: the env
    let env = sb.recorded("env").unwrap();
    for gone in [
        "ANTHROPIC_API_KEY=",
        "ANTHROPIC_AUTH_TOKEN=",
        "ANTHROPIC_BASE_URL=",
    ] {
        assert!(!env.lines().any(|l| l.starts_with(gone)), "{gone} {env}");
    }
    let account = format!(
        "CLAUDE_CONFIG_DIR={}",
        sb.home().join(".claude-work").display()
    );
    assert!(env.lines().any(|l| l == account), "{env}");
    assert!(
        env.lines().any(|l| l.starts_with("HOME=")),
        "parent env kept"
    );

    // AC 4: cwd is a fresh temp dir, removed afterwards
    let pwd = sb.recorded("pwd").unwrap();
    let pwd = Path::new(pwd.trim());
    assert_ne!(pwd, sb.root());
    assert!(pwd.starts_with(std::env::temp_dir()), "{}", pwd.display());
    assert!(!pwd.exists(), "the temp cwd is removed");

    // AC 5: the AI file, through the writer, with the model id the CLI reports
    let detail: Value =
        serde_json::from_str(&std::fs::read_to_string(sb.ai_file(&id)).unwrap()).unwrap();
    assert_eq!(detail["title"], "Buy a USB-C hub");
    assert_eq!(detail["kind"], "todo");
    assert_eq!(detail["tags_suggested"], json!(["hardware", "shopping"]));
    assert_eq!(detail["model"], "claude-haiku-5-5");
    assert!(detail["based_on"].as_str().unwrap().starts_with("sha256:"));
}

#[test]
fn without_an_account_dir_the_parent_claude_config_dir_passes_and_result_text_is_parsed() {
    let sb = Sandbox::new("result-only");
    sb.config(ON);
    let id = sb.add("call the bank");
    // No structured_output and no modelUsage: `result` is parsed, the model is the configured one.
    let raw =
        json!({"type": "result", "is_error": false, "result": good().to_string()}).to_string();
    let out = sb.run(&["ai", "enrich", &id], |c| {
        c.env("FAKE_ANSWER", raw)
            .env("CLAUDE_CONFIG_DIR", "/parent/claude");
    });
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let env = sb.recorded("env").unwrap();
    assert!(
        env.lines().any(|l| l == "CLAUDE_CONFIG_DIR=/parent/claude"),
        "{env}"
    );
    let detail: Value =
        serde_json::from_str(&std::fs::read_to_string(sb.ai_file(&id)).unwrap()).unwrap();
    assert_eq!(detail["model"], "haiku");
}

// AC 5, 6 — every failure: reported, no AI file, other notes go on, exit 1

fn fails_with(sb: &Sandbox, id: &str, tweak: impl FnOnce(&mut Command)) -> String {
    let out = sb.run(&["ai", "enrich", id], tweak);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stdout));
    assert!(!sb.ai_file(id).exists(), "no AI file on a failure");
    let err = text(&out.stderr);
    assert!(err.contains("0 enriched, 1 failed"), "{err}");
    err
}

#[test]
fn garbage_output_fails() {
    let sb = Sandbox::new("garbage");
    sb.config(ON);
    let id = sb.add("x");
    let err = fails_with(&sb, &id, |c| {
        c.env("FAKE_MODE", "garbage");
    });
    assert!(
        err.contains(&format!("{id}: failed: the answer is not JSON")),
        "{err}"
    );
    assert!(
        !err.contains("this is not json"),
        "raw output never shown: {err}"
    );
}

#[test]
fn a_non_zero_exit_fails_with_a_login_hint() {
    let sb = Sandbox::new("exit1");
    sb.config(&format!("{ON}claude_config_dir = \"/acct\"\n"));
    let id = sb.add("x");
    let err = fails_with(&sb, &id, |c| {
        c.env("FAKE_MODE", "exit1");
    });
    assert!(err.contains("failed: claude exited 1"), "{err}");
    assert!(
        err.contains("log in once: CLAUDE_CONFIG_DIR=/acct claude"),
        "{err}"
    );
}

#[test]
fn is_error_wrong_shape_and_over_a_cap_fail() {
    let sb = Sandbox::new("shapes");
    sb.config(ON);
    let id = sb.add("x");
    let is_error = json!({"type": "result", "is_error": true, "result": "boom"}).to_string();
    let err = fails_with(&sb, &id, |c| {
        c.env("FAKE_ANSWER", is_error);
    });
    assert!(err.contains("failed: claude reported an error"), "{err}");
    assert!(!err.contains("boom"), "{err}");

    let extra = json!({"title": "t", "kind": "bug", "summary": "s", "tags_suggested": [], "x": 1});
    let err = fails_with(&sb, &id, |c| {
        c.env("FAKE_ANSWER", answer(&extra));
    });
    assert!(
        err.contains("failed: the answer has the wrong shape"),
        "{err}"
    );

    let mut long = good();
    long["summary"] = json!("s".repeat(601));
    let err = fails_with(&sb, &id, |c| {
        c.env("FAKE_ANSWER", answer(&long));
    });
    assert!(
        err.contains("failed: summary is 601 chars; the limit is 600"),
        "{err}"
    );

    let mut kind = good();
    kind["kind"] = json!("rant");
    let err = fails_with(&sb, &id, |c| {
        c.env("FAKE_ANSWER", answer(&kind));
    });
    assert!(err.contains("failed: kind must be one of"), "{err}");
}

#[test]
fn a_child_past_the_timeout_is_killed_and_reaped() {
    let sb = Sandbox::new("timeout");
    sb.config(&format!("{ON}timeout_secs = 1\n"));
    let id = sb.add("x");
    let start = Instant::now();
    let err = fails_with(&sb, &id, |c| {
        c.env("FAKE_MODE", "sleep");
    });
    assert!(
        start.elapsed() < Duration::from_secs(10),
        "{:?}",
        start.elapsed()
    );
    assert!(err.contains(&format!("{id}: failed: timeout")), "{err}");
    let pid = sb.recorded("pid").unwrap();
    assert!(
        !Path::new(&format!("/proc/{}", pid.trim())).exists(),
        "the child is gone (killed and reaped)"
    );
}

#[test]
fn a_command_not_on_path_fails_before_any_note() {
    let sb = Sandbox::new("not-found");
    sb.config(&format!("{ON}command = \"no-such-claude\"\n"));
    let id = sb.add("x");
    let out = sb.run(&["ai", "enrich", &id], |_| {});
    assert_eq!(out.status.code(), Some(1));
    let err = text(&out.stderr);
    assert!(
        err.contains("failed: claude not found (no-such-claude)"),
        "{err}"
    );
    assert!(!err.contains(&id), "no note was tried: {err}");
}

// AC 7 — ids and --pending

#[test]
fn ids_go_one_by_one_and_an_unknown_id_does_not_stop_the_others() {
    let sb = Sandbox::new("ids");
    sb.config(ON);
    let a = sb.add("first");
    let b = sb.add("second");
    let out = sb.run(&["ai", "enrich", &a, "20990101-000000-zzzz", &b], |_| {});
    assert_eq!(out.status.code(), Some(1));
    let err = text(&out.stderr);
    assert!(
        err.contains("20990101-000000-zzzz: failed: no such note"),
        "{err}"
    );
    assert!(err.contains("2 enriched, 1 failed"), "{err}");
    assert!(sb.ai_file(&a).exists() && sb.ai_file(&b).exists());
    assert_eq!(sb.calls(), 2);
}

#[test]
fn pending_enriches_the_pending_notes_up_to_the_limit() {
    let sb = Sandbox::new("pending");
    sb.config(ON);
    let ids: Vec<String> = (0..3).map(|n| sb.add(&format!("note {n}"))).collect();
    let out = sb.run(&["ai", "enrich", "--pending", "--limit", "2"], |_| {});
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("2 enriched, 0 failed"));
    assert_eq!(ids.iter().filter(|id| sb.ai_file(id).exists()).count(), 2);
    let out = sb.run(&["ai", "enrich", "--pending"], |_| {});
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("1 enriched, 0 failed"));
    assert!(ids.iter().all(|id| sb.ai_file(id).exists()));
    assert_eq!(sb.calls(), 3);
}

#[test]
fn ids_and_pending_are_a_usage_choice() {
    let sb = Sandbox::new("usage");
    assert_eq!(sb.run(&["ai", "enrich"], |_| {}).status.code(), Some(2));
    assert_eq!(
        sb.run(&["ai", "enrich", "--pending", "x"], |_| {})
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        sb.run(&["ai", "enrich", "--limit", "2", "x"], |_| {})
            .status
            .code(),
        Some(2)
    );
}

// AC 8 — after_save

fn newest_id(sb: &Sandbox) -> String {
    let mut ids: Vec<String> = sb
        .store()
        .iter_notes()
        .filter_map(|item| match item {
            Item::Note(note) => Some(note.id().to_string()),
            _ => None,
        })
        .collect();
    ids.sort();
    ids.pop().expect("a note was saved")
}

/// After a save with after_save on, the AI file appears within 5 s; the save itself returned fast,
/// and the started `ai enrich` started nothing more.
fn enriched_after_save(sb: &Sandbox, saved_in: Duration) {
    assert!(
        saved_in < Duration::from_secs(3),
        "the save waited: {saved_in:?}"
    );
    let id = newest_id(sb);
    assert!(
        wait_for(&sb.ai_file(&id), Duration::from_secs(5)),
        "no AI file after the save"
    );
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(sb.calls(), 1, "ai enrich never triggers after_save");
}

const AFTER: &str = "[ai]\nenrich = \"claude\"\nafter_save = true\n";

#[test]
fn after_save_enriches_a_note_saved_by_add() {
    let sb = Sandbox::new("after-add");
    sb.config(AFTER);
    let start = Instant::now();
    let out = sb.run(&["add", "hello", "there"], |_| {});
    let took = start.elapsed();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    enriched_after_save(&sb, took);
}

#[test]
fn after_save_enriches_a_note_saved_by_the_hook() {
    use std::io::Write;
    let sb = Sandbox::new("after-hook");
    sb.config(AFTER);
    let event = json!({"prompt": "/note hello hook", "cwd": sb.root()}).to_string();
    let start = Instant::now();
    let mut child = sb
        .cmd(&["hook-prompt"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(event.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let took = start.elapsed();
    assert!(text(&out.stdout).contains("\"decision\":\"block\""));
    enriched_after_save(&sb, took);
}

#[test]
fn after_save_enriches_a_note_saved_by_quick() {
    let sb = Sandbox::new("after-quick");
    sb.config(AFTER);
    let zenity = sb.bin().join("zenity");
    std::fs::write(&zenity, "#!/bin/sh\necho 'hello quick'\n").unwrap();
    std::fs::set_permissions(&zenity, std::fs::Permissions::from_mode(0o755)).unwrap();
    let start = Instant::now();
    let out = sb.run(&["quick"], |c| {
        c.env("DISPLAY", ":99");
    });
    let took = start.elapsed();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    enriched_after_save(&sb, took);
}

#[tokio::test]
async fn after_save_enriches_a_note_saved_by_mcp_note_add() {
    use rmcp::model::{CallToolRequestParams, ClientCapabilities, ClientConfig, Implementation};
    use rmcp::transport::TokioChildProcess;
    use rmcp::ServiceExt;

    let sb = Sandbox::new("after-mcp");
    sb.config(AFTER);
    let std_cmd = sb.cmd(&["serve"]);
    let mut cmd = tokio::process::Command::from(std_cmd);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    let transport = TokioChildProcess::new(cmd).unwrap();
    let info = ClientConfig::new(ClientCapabilities::default(), Implementation::new("t", "0"));
    let client = info.serve(transport).await.unwrap();
    let start = Instant::now();
    let args = json!({"text": "hello mcp", "cwd": sb.root()});
    let result = client
        .call_tool(
            CallToolRequestParams::new("note_add")
                .with_arguments(args.as_object().unwrap().clone()),
        )
        .await
        .unwrap();
    let took = start.elapsed();
    assert_ne!(result.is_error, Some(true));
    enriched_after_save(&sb, took);
    client.cancel().await.unwrap();
}

#[test]
fn with_after_save_off_a_save_starts_nothing() {
    let sb = Sandbox::new("after-off");
    sb.config(ON);
    let out = sb.run(&["add", "hello"], |_| {});
    assert_eq!(out.status.code(), Some(0));
    sb.config("[ai]\nafter_save = true\n");
    let out = sb.run(&["add", "hello again"], |_| {});
    assert_eq!(out.status.code(), Some(0));
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(sb.calls(), 0, "no process was started");
}

#[test]
fn a_bad_ai_section_never_fails_a_save() {
    let sb = Sandbox::new("after-bad");
    sb.config("[ai]\nenrich = \"nope\"\nafter_save = true\n");
    let out = sb.run(&["add", "kept"], |_| {});
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("✓ noted"));
}

#[test]
fn with_the_env_dir_a_save_needs_no_home_and_no_readable_config_as_today() {
    let sb = Sandbox::new("after-no-home");
    let out = sb.run(&["add", "no home"], |c| {
        c.env_remove("HOME");
    });
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    std::fs::create_dir_all(sb.home().join(".config/easy-note/config.toml")).unwrap();
    let out = sb.run(&["add", "config is a dir"], |_| {});
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(sb.calls(), 0);
}

// AC 9 — doctor

#[test]
fn doctor_prints_the_ai_line() {
    let sb = Sandbox::new("doctor");
    let line = |sb: &Sandbox| {
        let out = sb.run(&["doctor"], |_| {});
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        text(&out.stdout)
            .lines()
            .find(|l| l.starts_with("ai:"))
            .unwrap_or_default()
            .to_string()
    };
    assert_eq!(line(&sb), "ai: off");
    sb.config(ON);
    assert_eq!(
        line(&sb),
        format!(
            "ai: claude via claude ({}), account dir default, model haiku, after_save false",
            sb.bin().join("claude").display()
        )
    );
    sb.config(&format!(
        "{ON}command = \"gone-claude\"\nclaude_config_dir = \"/acct\"\nmodel = \"sonnet\"\nafter_save = true\n"
    ));
    assert_eq!(
        line(&sb),
        "ai: claude via gone-claude (not on PATH), account dir /acct, model sonnet, after_save true"
    );
}

#[test]
fn schedule_keys_are_read_and_the_bounds_hold() {
    let sb = Sandbox::new("sched-ok");
    let _env = HomeGuard::new(&sb.home());
    sb.config(
        "[ai]\nenrich = \"claude\"\nschedule = \"tray\"\ninterval_min = 5\nmax_per_run = 200\nbatch_size = 50\njobs = [\"digest\", \"organize\"]\ndigest_hour = 23\n",
    );
    let ai = config::ai_settings().unwrap();
    assert_eq!(ai.schedule, config::Schedule::Tray);
    assert_eq!(
        (
            ai.interval_min,
            ai.max_per_run,
            ai.batch_size,
            ai.digest_hour
        ),
        (5, 200, 50, 23)
    );
    assert_eq!(ai.jobs, vec![config::Job::Digest, config::Job::Organize]);
    assert!(ai.effective_schedule());
    sb.config("[ai]\njobs = []\nbatch_size = 1\nmax_per_run = 1\ndigest_hour = 0\n");
    let ai = config::ai_settings().unwrap();
    assert!(ai.jobs.is_empty());
    assert_eq!((ai.batch_size, ai.digest_hour), (1, 0));
}

#[test]
fn a_bad_schedule_value_names_the_key_not_the_value() {
    let sb = Sandbox::new("sched-bad");
    let _env = HomeGuard::new(&sb.home());
    for (body, key) in [
        ("schedule = \"cronzz\"", "schedule"),
        ("schedule = true", "schedule"),
        ("interval_min = 1", "interval_min"),
        ("interval_min = \"60\"", "interval_min"),
        ("max_per_run = 0", "max_per_run"),
        ("max_per_run = 201", "max_per_run"),
        ("batch_size = 0", "batch_size"),
        ("batch_size = 51", "batch_size"),
        ("batch_size = 11\nmax_per_run = 10", "batch_size"),
        ("batch_size = 21", "batch_size"),
        ("jobs = \"organize\"", "jobs"),
        ("jobs = [\"sweep77\"]", "jobs"),
        ("jobs = [3]", "jobs"),
        ("digest_hour = 24", "digest_hour"),
        ("digest_hour = -1", "digest_hour"),
        ("digest_hour = \"3\"", "digest_hour"),
    ] {
        sb.config(&format!("[ai]\n{body}\n"));
        let err = config::ai_settings().unwrap_err().to_string();
        assert!(err.contains(&format!("[ai] {key}")), "{body}: {err}");
        assert!(err.contains("config.toml"), "{body}: {err}");
        for leak in ["cronzz", "sweep77", "201", "\"60\"", "-1"] {
            assert!(!err.contains(leak), "{body}: {err}");
        }
    }
}

#[test]
fn tray_schedule_without_enrich_reads_but_is_not_effective() {
    let sb = Sandbox::new("sched-no-enrich");
    let _env = HomeGuard::new(&sb.home());
    sb.config("[ai]\nschedule = \"tray\"\n");
    let ai = config::ai_settings().unwrap();
    assert_eq!(ai.schedule, config::Schedule::Tray);
    assert!(!ai.effective_schedule());
    let line = easy_note_mcp::surfaces::enrich::schedule_line();
    assert!(line.contains("schedule needs enrich = claude"), "{line}");
}

#[test]
fn the_schedule_line_shows_every_key() {
    let sb = Sandbox::new("sched-line");
    let _env = HomeGuard::new(&sb.home());
    sb.config("[ai]\nenrich = \"claude\"\nschedule = \"tray\"\ninterval_min = 30\nmax_per_run = 40\nbatch_size = 8\njobs = [\"drafts\", \"digest\"]\ndigest_hour = 4\n");
    assert_eq!(
        easy_note_mcp::surfaces::enrich::schedule_line(),
        "ai schedule: tray, every 30 min, up to 40 notes per run, batch 8, jobs drafts+digest, digest at 4h"
    );
    sb.config("");
    assert_eq!(
        easy_note_mcp::surfaces::enrich::schedule_line(),
        "ai schedule: off, every 60 min, up to 20 notes per run, batch 10, jobs organize+drafts+digest, digest at 3h"
    );
}

// ---------------------------------------------------------------------------------------------
// HUN-300: `config::save_ai_settings`, the one writer of the Settings screen (FR-048, FR-052)
// ---------------------------------------------------------------------------------------------

/// A config.toml a person wrote by hand: header, same-line and trailing comments, other tables,
/// a key easy-note does not know, a `~` account folder and a literal-quoted string.
const RICH: &str = r#"# easy-note settings (written by hand)
dir = "~/notes"   # where the notes live
capture_prefix = "/n "

[ai] # the AI section
enrich = "claude"   # on since July
command = "/opt/claude/bin/claude" # my build
claude_config_dir = "~/.claude-work"  # the work account
model = 'haiku'      # cheap
timeout_secs = 90
after_save = false
# how often
interval_min = 60 # hourly
max_per_run = 20
future_key = "kept"

[other.tool]
name = "x"   # not ours

# the last line
"#;

fn config_file(sb: &Sandbox) -> PathBuf {
    sb.home().join(".config/easy-note/config.toml")
}

fn config_text(sb: &Sandbox) -> String {
    std::fs::read_to_string(config_file(sb)).unwrap()
}

/// The names in the config folder: a save leaves no temp file behind.
fn config_folder(sb: &Sandbox) -> Vec<String> {
    names(config_file(sb).parent().unwrap())
}

fn names(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn changed(settings: config::AiSettings) -> config::AiSettings {
    config::AiSettings {
        schedule: config::Schedule::Tray,
        interval_min: 30,
        batch_size: 5,
        jobs: vec![config::Job::Organize, config::Job::Drafts],
        model: "sonnet".into(),
        after_save: true,
        notice_seen: Some("abc123".into()),
        ..settings
    }
}

#[test]
fn save_ai_settings_changes_only_its_keys_and_keeps_every_other_byte() {
    let sb = Sandbox::new("save-rich");
    let _env = HomeGuard::new(&sb.home());
    sb.config(RICH);
    let settings = changed(config::ai_settings().unwrap());
    config::save_ai_settings(&settings).unwrap();
    let want = RICH
        .replace("model = 'haiku'      # cheap", "model = \"sonnet\"      # cheap")
        .replace("after_save = false", "after_save = true")
        .replace("interval_min = 60 # hourly", "interval_min = 30 # hourly")
        .replace(
            "future_key = \"kept\"\n",
            "future_key = \"kept\"\nschedule = \"tray\"\nbatch_size = 5\njobs = [\"organize\", \"drafts\"]\nnotice_seen = \"abc123\"\n",
        );
    assert_eq!(config_text(&sb), want);
    // `command` and the unexpanded `~` account folder are the file's own bytes.
    assert!(want.contains("claude_config_dir = \"~/.claude-work\""));
    assert_eq!(config::ai_settings().unwrap(), settings);
    assert_eq!(config_folder(&sb), ["config.toml"]);
}

#[test]
fn save_ai_settings_never_writes_the_command_or_the_account_folder() {
    let sb = Sandbox::new("save-owned");
    let _env = HomeGuard::new(&sb.home());
    sb.config(RICH);
    let mut settings = config::ai_settings().unwrap();
    settings.command = "/tmp/evil".into();
    settings.claude_config_dir = Some("/tmp/other".into());
    settings.timeout_secs = 5;
    settings.digest_hour = 9;
    config::save_ai_settings(&settings).unwrap();
    // Only the keys the Settings screen shows are written; the absent ones are added.
    assert_eq!(
        config_text(&sb),
        RICH.replace(
            "future_key = \"kept\"\n",
            "future_key = \"kept\"\nschedule = \"off\"\nbatch_size = 10\njobs = [\"organize\", \"drafts\", \"digest\"]\n",
        )
    );
    // A file without them gets neither.
    sb.config("[ai]\n");
    config::save_ai_settings(&settings).unwrap();
    let text = config_text(&sb);
    for key in [
        "command",
        "claude_config_dir",
        "timeout_secs",
        "digest_hour",
    ] {
        assert!(!text.contains(key), "{key}: {text}");
    }
}

#[test]
fn save_ai_settings_leaves_an_unchanged_file_byte_for_byte() {
    let sb = Sandbox::new("save-same");
    let _env = HomeGuard::new(&sb.home());
    // Every owned key present, each in a spelling the writer would not choose itself; the jobs
    // in another order over several lines, with a comment inside.
    let text = "[ai]\nenrich = 'off'\nschedule = 'off'\ninterval_min = 1_440\nmax_per_run = 0x14\nbatch_size = 10\njobs = [\n  \"digest\", # nightly\n  \"organize\",\n]\nmodel = '''haiku'''\nafter_save=false\n";
    sb.config(text);
    config::save_ai_settings(&config::ai_settings().unwrap()).unwrap();
    assert_eq!(config_text(&sb), text);
}

#[test]
fn save_ai_settings_replaces_a_changed_jobs_list_and_keeps_its_line_comment() {
    let sb = Sandbox::new("save-jobs");
    let _env = HomeGuard::new(&sb.home());
    let text = "[ai]\nenrich = \"off\"\nschedule = \"off\"\ninterval_min = 60\nmax_per_run = 20\nbatch_size = 10\njobs = [\"organize\"] # just this\nmodel = \"haiku\"\nafter_save = false\n";
    sb.config(text);
    let mut settings = config::ai_settings().unwrap();
    settings.jobs = vec![];
    config::save_ai_settings(&settings).unwrap();
    assert_eq!(
        config_text(&sb),
        text.replace("jobs = [\"organize\"] # just this", "jobs = [] # just this")
    );
}

const ALL_OWNED: &str = "enrich = \"off\"\nschedule = \"off\"\ninterval_min = 60\nmax_per_run = 20\nbatch_size = 10\njobs = [\"organize\", \"drafts\", \"digest\"]\nmodel = \"haiku\"\nafter_save = false\n";

#[test]
fn save_ai_settings_creates_a_missing_file_with_only_the_ai_table() {
    let sb = Sandbox::new("save-new");
    let _env = HomeGuard::new(&sb.home());
    assert!(!config_file(&sb).exists());
    config::save_ai_settings(&config::AiSettings::default()).unwrap();
    assert_eq!(config_text(&sb), format!("[ai]\n{ALL_OWNED}"));
    assert_eq!(
        config::ai_settings().unwrap(),
        config::AiSettings::default()
    );
    assert_eq!(config_folder(&sb), ["config.toml"]);
}

#[test]
fn save_ai_settings_adds_a_real_table_when_the_file_has_none() {
    let sb = Sandbox::new("save-no-table");
    let _env = HomeGuard::new(&sb.home());
    let before = "# header\ndir = \"~/notes\" # mine\n\n[other]\nn = 1\n";
    sb.config(before);
    config::save_ai_settings(&config::AiSettings::default()).unwrap();
    assert_eq!(config_text(&sb), format!("{before}\n[ai]\n{ALL_OWNED}"));
}

#[test]
fn save_ai_settings_edits_an_inline_ai_table_in_place() {
    let sb = Sandbox::new("save-inline");
    let _env = HomeGuard::new(&sb.home());
    sb.config("# header\nai = { enrich = \"off\", command = \"c\" } # inline\ndir = \"~/n\"\n");
    let settings = config::AiSettings {
        enrich: true,
        ..config::ai_settings().unwrap()
    };
    config::save_ai_settings(&settings).unwrap();
    assert_eq!(
        config_text(&sb),
        "# header\nai = { enrich = \"claude\", command = \"c\", schedule = \"off\", interval_min = 60, max_per_run = 20, batch_size = 10, jobs = [\"organize\", \"drafts\", \"digest\"], model = \"haiku\", after_save = false } # inline\ndir = \"~/n\"\n"
    );
    assert_eq!(config::ai_settings().unwrap(), settings);
}

#[test]
fn save_ai_settings_edits_dotted_ai_keys() {
    let sb = Sandbox::new("save-dotted");
    let _env = HomeGuard::new(&sb.home());
    sb.config("ai.enrich = \"off\" # dotted\nai.command = \"c\"\n");
    let settings = config::AiSettings {
        enrich: true,
        ..config::ai_settings().unwrap()
    };
    config::save_ai_settings(&settings).unwrap();
    let text = config_text(&sb);
    assert!(
        text.starts_with("ai.enrich = \"claude\" # dotted\nai.command = \"c\"\nai."),
        "{text}"
    );
    assert_eq!(config::ai_settings().unwrap(), settings);
}

#[test]
fn save_ai_settings_refuses_invalid_input_naming_the_key_and_changes_nothing() {
    let sb = Sandbox::new("save-invalid");
    let _env = HomeGuard::new(&sb.home());
    sb.config(RICH);
    let base = config::ai_settings().unwrap();
    type Change = fn(&mut config::AiSettings);
    let cases: [(Change, &str); 9] = [
        (|s| s.interval_min = 0, "interval_min"),
        (|s| s.interval_min = 4, "interval_min"),
        (|s| s.max_per_run = 0, "max_per_run"),
        (|s| s.max_per_run = 201, "max_per_run"),
        (|s| s.batch_size = 51, "batch_size"),
        (|s| (s.batch_size, s.max_per_run) = (11, 10), "batch_size"),
        (|s| s.model = String::new(), "model"),
        (|s| s.model = "--flag".into(), "model"),
        (
            |s| (s.enrich, s.schedule) = (false, config::Schedule::Tray),
            "schedule",
        ),
    ];
    for (change, key) in cases {
        let mut settings = base.clone();
        change(&mut settings);
        let err = config::save_ai_settings(&settings).unwrap_err().to_string();
        assert!(err.contains(&format!("[ai] {key}")), "{key}: {err}");
        assert!(err.contains("config.toml"), "{key}: {err}");
        assert_eq!(config_text(&sb), RICH, "{key} wrote");
    }
    assert_eq!(config_folder(&sb), ["config.toml"]);
}

#[test]
fn save_ai_settings_refuses_a_file_it_cannot_use_and_changes_nothing() {
    let sb = Sandbox::new("save-broken");
    let _env = HomeGuard::new(&sb.home());
    for (before, part) in [
        ("[ai\nenrich = \"claude\"\n", "cannot read"),
        ("ai = 1\n", "[ai] section"),
        ("[[ai]]\nx = 1\n", "[ai] section"),
        // A bad value in a key the screen does not own is the person's to fix in the file.
        ("[ai]\ntimeout_secs = 0\n", "[ai] timeout_secs"),
    ] {
        sb.config(before);
        let err = config::save_ai_settings(&config::AiSettings::default())
            .unwrap_err()
            .to_string();
        assert!(err.contains(part), "{before}: {err}");
        assert!(err.contains("config.toml"), "{before}: {err}");
        assert!(!err.contains('\n'), "one line: {err:?}");
        assert_eq!(config_text(&sb), before);
    }
}

#[test]
fn save_ai_settings_without_write_permission_is_an_error_that_names_the_file() {
    let sb = Sandbox::new("save-denied");
    let _env = HomeGuard::new(&sb.home());
    sb.config("[ai]\n");
    let folder = config_file(&sb).parent().unwrap().to_path_buf();
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o500)).unwrap();
    let result = config::save_ai_settings(&config::AiSettings::default());
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o700)).unwrap();
    let err = result.unwrap_err().to_string();
    assert!(
        err.starts_with("cannot write ") && err.contains("config.toml"),
        "{err}"
    );
    assert_eq!(config_text(&sb), "[ai]\n");
    assert_eq!(config_folder(&sb), ["config.toml"]);
}

#[test]
fn save_ai_settings_if_refuses_a_file_changed_since_it_was_read() {
    let sb = Sandbox::new("save-outside");
    let _env = HomeGuard::new(&sb.home());
    // No file, a changed file and a removed file each differ from what was read.
    let (settings, absent) = config::ai_settings_stamped().unwrap();
    sb.config("[ai]\nmodel = \"sonnet\"\n");
    let err = config::save_ai_settings_if(&settings, &absent)
        .unwrap_err()
        .to_string();
    assert_eq!(err, config::CHANGED_OUTSIDE);
    assert!(err.contains("changed outside"), "{err}");
    assert_eq!(config_text(&sb), "[ai]\nmodel = \"sonnet\"\n");

    let (settings, base) = config::ai_settings_stamped().unwrap();
    assert_ne!(base, absent);
    sb.config("[ai]\nmodel = \"sonnet\" \n");
    assert!(config::save_ai_settings_if(&settings, &base).is_err());
    assert_eq!(config_text(&sb), "[ai]\nmodel = \"sonnet\" \n");
    std::fs::remove_file(config_file(&sb)).unwrap();
    assert!(config::save_ai_settings_if(&settings, &base).is_err());
    assert!(!config_file(&sb).exists());

    // The stamp of what is there now is accepted, once.
    sb.config("[ai]\nmodel = \"sonnet\"\n");
    let (mut settings, base) = config::ai_settings_stamped().unwrap();
    settings.interval_min = 30;
    config::save_ai_settings_if(&settings, &base).unwrap();
    assert_eq!(config::ai_settings().unwrap().interval_min, 30);
    assert!(config::save_ai_settings_if(&settings, &base).is_err());
}

/// Make config.toml a symlink to `to`, as a dotfiles tool does.
fn link_config(sb: &Sandbox, to: impl AsRef<Path>) {
    let path = config_file(sb);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    symlink(to, path).unwrap();
}

/// A folder under the sandbox root, for the file a link points to.
fn folder(sb: &Sandbox, name: &str) -> PathBuf {
    let folder = sb.root().join(name);
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// Change the interval and save, as the Settings screen does: with the stamp of what was read.
fn save_interval(minutes: u32) -> Result<(), String> {
    let (mut settings, base) = config::ai_settings_stamped().map_err(|e| e.to_string())?;
    settings.interval_min = minutes;
    config::save_ai_settings_if(&settings, &base).map_err(|e| e.to_string())
}

fn links_to(link: &Path) -> PathBuf {
    std::fs::read_link(link).unwrap_or_else(|e| panic!("{} is not a link: {e}", link.display()))
}

#[test]
fn save_ai_settings_writes_through_a_relative_symlink() {
    let sb = Sandbox::new("link-relative");
    let _env = HomeGuard::new(&sb.home());
    let dotfiles = folder(&sb, "home/dotfiles");
    let target = dotfiles.join("note.toml");
    std::fs::write(&target, "# mine\n[ai]\ninterval_min = 60 # hourly\n").unwrap();
    // Relative to the folder of the link, not to the working directory.
    link_config(&sb, "../../dotfiles/note.toml");
    // The link's folder takes no new file: the temp file has to be beside the target.
    let linked = config_file(&sb).parent().unwrap().to_path_buf();
    std::fs::set_permissions(&linked, std::fs::Permissions::from_mode(0o500)).unwrap();
    let result = save_interval(30);
    std::fs::set_permissions(&linked, std::fs::Permissions::from_mode(0o700)).unwrap();
    result.unwrap();
    assert_eq!(
        links_to(&config_file(&sb)),
        Path::new("../../dotfiles/note.toml")
    );
    let text = std::fs::read_to_string(&target).unwrap();
    assert!(
        text.starts_with("# mine\n[ai]\ninterval_min = 30 # hourly\n"),
        "{text}"
    );
    assert_eq!(names(&dotfiles), ["note.toml"]);
    assert_eq!(config_folder(&sb), ["config.toml"]);
    // A later edit of the target is what the settings say.
    std::fs::write(&target, "[ai]\ninterval_min = 90\n").unwrap();
    assert_eq!(config::ai_settings().unwrap().interval_min, 90);
}

#[test]
fn save_ai_settings_writes_through_an_absolute_symlink() {
    let sb = Sandbox::new("link-absolute");
    let _env = HomeGuard::new(&sb.home());
    let dotfiles = folder(&sb, "dotfiles");
    let target = dotfiles.join("note.toml");
    std::fs::write(&target, "[ai]\n").unwrap();
    link_config(&sb, &target);
    save_interval(30).unwrap();
    assert_eq!(links_to(&config_file(&sb)), target);
    assert!(std::fs::symlink_metadata(&target).unwrap().is_file());
    assert_eq!(config::ai_settings().unwrap().interval_min, 30);
    assert!(std::fs::read_to_string(&target)
        .unwrap()
        .contains("interval_min = 30"));
    assert_eq!(names(&dotfiles), ["note.toml"]);
    assert_eq!(config_folder(&sb), ["config.toml"]);
}

#[test]
fn save_ai_settings_follows_a_chain_of_symlinks_to_its_end() {
    let sb = Sandbox::new("link-chain");
    let _env = HomeGuard::new(&sb.home());
    let (hop, end) = (folder(&sb, "hop"), folder(&sb, "end"));
    let target = end.join("note.toml");
    std::fs::write(&target, "[ai]\n").unwrap();
    symlink("../end/note.toml", hop.join("next.toml")).unwrap();
    link_config(&sb, hop.join("next.toml"));
    save_interval(30).unwrap();
    assert_eq!(links_to(&config_file(&sb)), hop.join("next.toml"));
    assert_eq!(
        links_to(&hop.join("next.toml")),
        Path::new("../end/note.toml")
    );
    assert!(std::fs::read_to_string(&target)
        .unwrap()
        .contains("interval_min = 30"));
    assert_eq!(names(&hop), ["next.toml"]);
    assert_eq!(names(&end), ["note.toml"]);
}

#[test]
fn save_ai_settings_creates_the_target_of_a_dangling_symlink() {
    let sb = Sandbox::new("link-dangling");
    let _env = HomeGuard::new(&sb.home());
    let dotfiles = folder(&sb, "dotfiles");
    let target = dotfiles.join("note.toml");
    link_config(&sb, &target);
    save_interval(30).unwrap();
    assert_eq!(links_to(&config_file(&sb)), target);
    assert!(std::fs::read_to_string(&target)
        .unwrap()
        .contains("interval_min = 30"));
    assert_eq!(names(&dotfiles), ["note.toml"]);
    assert_eq!(config_folder(&sb), ["config.toml"]);
}

#[test]
fn save_ai_settings_refuses_a_dangling_symlink_into_a_missing_folder() {
    let sb = Sandbox::new("link-no-folder");
    let _env = HomeGuard::new(&sb.home());
    let target = sb.root().join("absent/note.toml");
    link_config(&sb, &target);
    let err = save_interval(30).unwrap_err();
    assert!(err.starts_with("cannot write "), "{err}");
    for part in [config_file(&sb), target.clone()] {
        assert!(err.contains(&part.display().to_string()), "{err}");
    }
    assert!(!err.contains('\n'), "one line: {err:?}");
    assert_eq!(links_to(&config_file(&sb)), target);
    assert!(!sb.root().join("absent").exists());
    assert_eq!(config_folder(&sb), ["config.toml"]);
}

#[test]
fn save_ai_settings_refuses_a_symlink_loop() {
    let sb = Sandbox::new("link-loop");
    let _env = HomeGuard::new(&sb.home());
    let other = folder(&sb, "other").join("back.toml");
    symlink(config_file(&sb), &other).unwrap();
    link_config(&sb, &other);
    let err = config::save_ai_settings(&config::AiSettings::default())
        .unwrap_err()
        .to_string();
    assert!(err.contains("symbolic links"), "{err}");
    assert!(
        err.contains(&config_file(&sb).display().to_string()),
        "{err}"
    );
    assert_eq!(links_to(&config_file(&sb)), other);
    assert_eq!(links_to(&other), config_file(&sb));
    assert_eq!(config_folder(&sb), ["config.toml"]);
}

#[test]
fn the_notice_is_the_agreed_text_and_its_version_follows_the_text() {
    assert_eq!(
        config::schedule_notice("~/.claude", "haiku"),
        "Masked note text is sent to your own claude CLI (account folder: ~/.claude, model: haiku). Nothing runs until you confirm."
    );
    let version =
        |dir: &str, model: &str| config::notice_version(&config::schedule_notice(dir, model));
    assert_eq!(version("/a", "haiku"), version("/a", "haiku"));
    assert_ne!(version("/a", "haiku"), version("/a", "sonnet"));
    assert_ne!(version("/a", "haiku"), version("/b", "haiku"));
}

#[test]
fn notice_seen_is_read_as_a_string() {
    let sb = Sandbox::new("notice-read");
    let _env = HomeGuard::new(&sb.home());
    assert_eq!(config::ai_settings().unwrap().notice_seen, None);
    sb.config("[ai]\nnotice_seen = \"v\"\n");
    assert_eq!(
        config::ai_settings().unwrap().notice_seen.as_deref(),
        Some("v")
    );
    sb.config("[ai]\nnotice_seen = 7\n");
    let err = config::ai_settings().unwrap_err().to_string();
    assert!(err.contains("[ai] notice_seen"), "{err}");
}
