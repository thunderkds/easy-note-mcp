//! Locked oracle for HUN-271 (R4): context capture, ported from the Python HUN-254 tests
//! (`git show 2d497fc:tests/test_context.py`, with the HUN-267 task pattern).
//!
//! Real temp git repos; a fake `git` on PATH for the timeout and spy cases. Temp HOME and
//! EASY_NOTE_DIR only. The tests change process-wide state (cwd, PATH), so each one holds a
//! global lock for its whole run, as pytest runs them one after another.
//!
//! Store part pending HUN-269: AC 7 "add still writes the note" and AC 9 "no path in any
//! written note" are checked on the `capture` output only.

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use easy_note_mcp::core::context::{capture, remote_slug, task_from_branch, Context, GIT_TIMEOUT};
use tempfile::TempDir;

const TOKEN_URL: &str = "https://user:tok@github.com/o/r.git";

static LOCK: Mutex<()> = Mutex::new(());

const TOUCHED: [&str; 8] = [
    "HOME",
    "EASY_NOTE_DIR",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_SYSTEM",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "PATH",
];

/// Temp root, temp HOME and EASY_NOTE_DIR, process cwd in a folder outside any repo.
/// Restores cwd and environment on drop.
struct Env {
    root: TempDir,
    saved: Vec<(&'static str, Option<OsString>)>,
    cwd: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl Env {
    fn new() -> Env {
        let lock = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let root = tempfile::tempdir().unwrap();
        let saved = TOUCHED.iter().map(|k| (*k, std::env::var_os(k))).collect();
        let cwd = std::env::current_dir().unwrap();
        let home = root.path().join("home");
        fs::create_dir(&home).unwrap();
        std::env::set_var("HOME", &home);
        std::env::set_var("EASY_NOTE_DIR", root.path().join("notes"));
        std::env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
        std::env::set_var("GIT_CONFIG_SYSTEM", "/dev/null");
        for key in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE"] {
            std::env::remove_var(key);
        }
        std::env::set_current_dir(root.path()).unwrap();
        Env {
            root,
            saved,
            cwd,
            _lock: lock,
        }
    }

    fn path(&self) -> &Path {
        self.root.path()
    }

    fn chdir(&self, dir: &Path) {
        std::env::set_current_dir(dir).unwrap();
    }

    /// Put a fake `git` script first on PATH.
    fn fake_git(&self, script: &str) {
        let bin = self.path().join("fake-bin");
        fs::create_dir_all(&bin).unwrap();
        let fake = bin.join("git");
        fs::write(&fake, script).unwrap();
        fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
        let old = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = vec![bin];
        paths.extend(std::env::split_paths(&old));
        std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.cwd);
        for (key, value) in &self.saved {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
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

fn make_repo(root: &Path, name: &str, remote: Option<&str>, branch: &str) -> PathBuf {
    let repo = root.join(name);
    fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-q", "-b", branch]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "first"]);
    if let Some(url) = remote {
        git(&repo, &["remote", "add", "origin", url]);
    }
    repo
}

fn ctx(
    repo: Option<&str>,
    remote: Option<&str>,
    branch: Option<&str>,
    task: Option<&str>,
) -> Context {
    let own = |s: Option<&str>| s.map(str::to_string);
    Context {
        repo: own(repo),
        remote: own(remote),
        branch: own(branch),
        task: own(task),
    }
}

fn at(cwd: &Path) -> Context {
    capture(None, Some(cwd), &[])
}

fn uri(path: &Path) -> String {
    url::Url::from_file_path(path).unwrap().to_string()
}

/// Every value of a context as one text, for "appears nowhere" checks.
fn text_of(c: &Context) -> String {
    [&c.repo, &c.remote, &c.branch, &c.task]
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

// AC 1

#[test]
fn main_repo_uses_folder_name() {
    let env = Env::new();
    let repo = make_repo(
        env.path(),
        "easy-ui-mcp",
        Some("git@github.com:thunderkds/easy-ui-mcp.git"),
        "main",
    );
    assert_eq!(
        at(&repo),
        ctx(
            Some("easy-ui-mcp"),
            Some("thunderkds/easy-ui-mcp"),
            Some("main"),
            None
        )
    );
}

#[test]
fn subfolder_of_repo_gives_the_repo_name() {
    let env = Env::new();
    let repo = make_repo(env.path(), "proj", None, "main");
    fs::create_dir_all(repo.join("sub/deep")).unwrap();
    assert_eq!(at(&repo.join("sub/deep")).repo.as_deref(), Some("proj"));
}

#[test]
fn linked_worktree_uses_main_folder_name_and_branch_task() {
    let env = Env::new();
    let repo = make_repo(env.path(), "main-repo", None, "main");
    let wt = env.path().join("wt-t148");
    git(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            wt.to_str().unwrap(),
            "-b",
            "fix/t148-x",
        ],
    );
    let got = at(&wt);
    assert_eq!(got.repo.as_deref(), Some("main-repo"));
    assert_eq!(got.task.as_deref(), Some("T148"));
    assert_eq!(got.branch.as_deref(), Some("fix/t148-x"));
}

#[test]
fn bare_repo_drops_the_dot_git_suffix() {
    let env = Env::new();
    let bare = env.path().join("central.git");
    git(
        env.path(),
        &["init", "-q", "--bare", bare.to_str().unwrap()],
    );
    assert_eq!(at(&bare).repo.as_deref(), Some("central"));
}

// AC 2

#[test]
fn no_git_gives_no_fields() {
    let env = Env::new();
    let plain = env.path().join("plain");
    fs::create_dir(&plain).unwrap();
    assert_eq!(at(&plain), Context::default());
}

#[test]
fn missing_folder_gives_no_fields() {
    // Store part pending HUN-269: `add` still succeeds with these (empty) fields.
    let env = Env::new();
    assert_eq!(at(&env.path().join("nope")), Context::default());
}

#[test]
fn no_git_program_gives_no_fields() {
    let env = Env::new();
    let repo = make_repo(env.path(), "proj", None, "main");
    let empty_bin = env.path().join("empty-bin");
    fs::create_dir(&empty_bin).unwrap();
    std::env::set_var("PATH", &empty_bin);
    assert_eq!(at(&repo), Context::default());
}

#[test]
fn no_git_keeps_explicit_task() {
    let env = Env::new();
    let got = capture(Some(" T9 "), Some(env.path()), &[]);
    assert_eq!(got, ctx(None, None, None, Some("T9")));
}

// AC 3

#[test]
fn detached_head_has_repo_but_no_branch_or_task() {
    let env = Env::new();
    let repo = make_repo(env.path(), "proj", None, "fix/t120-x");
    git(&repo, &["checkout", "-q", "--detach"]);
    assert_eq!(at(&repo), ctx(Some("proj"), None, None, None));
}

// AC 4

#[test]
fn remote_slug_of_known_forms() {
    for url in [
        TOKEN_URL,
        "https://github.com/o/r",
        "git@github.com:o/r.git",
        "ssh://git@host/o/r",
        "ssh://git@host:2222/o/r.git",
        "ssh://git@host/group/sub/o/r.git",
        "https://github.com/o/r/",
    ] {
        assert_eq!(remote_slug(url).as_deref(), Some("o/r"), "{url}");
    }
}

#[test]
fn remote_that_is_a_path_or_has_no_owner_gives_none() {
    for url in [
        "/srv/git/o/r.git",
        "../r",
        "file:///srv/git/o/r.git",
        "https://github.com/r",
        "r",
        "",
        "https://[bad/o/r",
    ] {
        assert_eq!(remote_slug(url), None, "{url:?}");
    }
}

#[test]
fn token_in_remote_is_in_no_field() {
    // Store part pending HUN-269: the same check on every file the store writes.
    let env = Env::new();
    let repo = make_repo(env.path(), "proj", Some(TOKEN_URL), "main");
    let got = at(&repo);
    assert_eq!(got.remote.as_deref(), Some("o/r"));
    let text = text_of(&got);
    assert!(!text.contains("tok") && !text.contains("user"), "{text}");
}

#[test]
fn repo_without_origin_has_no_remote() {
    let env = Env::new();
    assert_eq!(
        at(&make_repo(env.path(), "proj", None, "main")).remote,
        None
    );
}

// AC 5

#[test]
fn task_from_branch_table() {
    let cases = [
        ("fix/t120-x", Some("T120")),
        ("fix/t120", Some("T120")),
        ("feat/HUN-221-y", Some("HUN-221")),
        ("T1234-big", Some("T1234")),
        ("main", None),
        ("fix/art1234", None),
        ("fix/t120x", None),
        ("fix/t12", None),
        ("fix/t12345", None),
        ("feat/hun-221", None),
        ("fix/t120-and-t130", Some("T120")),
        // PRD FR-005 tracker pattern `[A-Z][A-Z0-9]+-\d+` (HUN-267)
        ("feat/AB2-12-x", Some("AB2-12")),
        ("fix/PROJ-1234567", Some("PROJ-1234567")),
        ("feat/A-12", None),
    ];
    for (branch, task) in cases {
        assert_eq!(task_from_branch(branch).as_deref(), task, "{branch}");
    }
}

#[test]
fn branch_task_end_to_end() {
    let env = Env::new();
    let repo = make_repo(env.path(), "proj", None, "feat/HUN-221-y");
    assert_eq!(at(&repo).task.as_deref(), Some("HUN-221"));
    git(&repo, &["checkout", "-q", "-b", "main"]);
    let got = at(&repo);
    assert_eq!(got.branch.as_deref(), Some("main"));
    assert_eq!(got.task, None);
}

#[test]
fn explicit_task_wins() {
    let env = Env::new();
    let repo = make_repo(env.path(), "proj", None, "fix/t120-x");
    assert_eq!(
        capture(Some("T999"), Some(&repo), &[]).task.as_deref(),
        Some("T999")
    );
    assert_eq!(
        capture(Some("  "), Some(&repo), &[]).task.as_deref(),
        Some("T120")
    );
}

// AC 6: explicit cwd, then MCP roots, then process cwd (docs/experiments/T001-client-launch.md)

#[test]
fn cwd_order() {
    let env = Env::new();
    let explicit = make_repo(env.path(), "explicit", None, "main");
    let rooted = make_repo(env.path(), "rooted", None, "main");
    let process = make_repo(env.path(), "process", None, "main");
    env.chdir(&process);
    let roots = [uri(&rooted)];
    let repo_of = |c: Context| c.repo;

    assert_eq!(
        repo_of(capture(None, Some(&explicit), &roots)).as_deref(),
        Some("explicit")
    );
    assert_eq!(
        repo_of(capture(None, None, &roots)).as_deref(),
        Some("rooted")
    );
    // Codex answers an empty list; a client with no roots at all is the same call.
    assert_eq!(
        repo_of(capture(None, None, &[])).as_deref(),
        Some("process")
    );
}

#[test]
fn first_root_is_used_and_plain_folder_roots_work() {
    let env = Env::new();
    let one = make_repo(env.path(), "one", None, "main");
    let two = make_repo(env.path(), "two", None, "main");
    let repo_of = |roots: &[String]| capture(None, None, roots).repo;
    assert_eq!(repo_of(&[uri(&one), uri(&two)]).as_deref(), Some("one"));
    assert_eq!(
        repo_of(&[two.to_str().unwrap().to_string()]).as_deref(),
        Some("two")
    );
    assert_eq!(
        repo_of(&["https://example.com/x".to_string(), uri(&two)]).as_deref(),
        Some("two")
    );
}

#[test]
fn root_in_a_folder_with_spaces() {
    let env = Env::new();
    let repo = make_repo(env.path(), "my proj", None, "main");
    assert_eq!(
        capture(None, None, &[uri(&repo)]).repo.as_deref(),
        Some("my proj")
    );
}

#[test]
fn a_source_that_is_not_a_repo_does_not_fall_through_to_a_guess() {
    let env = Env::new();
    let process = make_repo(env.path(), "process", None, "main");
    env.chdir(&process);
    let plain = env.path().join("plain");
    fs::create_dir(&plain).unwrap();
    assert_eq!(capture(None, None, &[uri(&plain)]), Context::default());
    assert_eq!(at(&plain), Context::default());
}

// AC 7

#[test]
fn slow_git_drops_the_fields() {
    // Store part pending HUN-269: `add` still writes the note with these (empty) fields.
    let env = Env::new();
    let repo = make_repo(env.path(), "proj", Some(TOKEN_URL), "fix/t120-x");
    env.fake_git("#!/bin/sh\nexec sleep 3\n");

    let start = Instant::now();
    let got = at(&repo);
    assert!(
        start.elapsed() < Duration::from_millis(2900),
        "one 2 s wait, not one per git call: {:?}",
        start.elapsed()
    );
    assert_eq!(got, Context::default());
}

#[test]
fn slow_git_keeps_explicit_task() {
    let env = Env::new();
    env.fake_git("#!/bin/sh\nexec sleep 3\n");
    let got = capture(Some("T5"), Some(env.path()), &[]);
    assert_eq!(got, ctx(None, None, None, Some("T5")));
}

// AC 8: the Python spy becomes a fake `git` that logs its argv and environment, then runs the real git

#[test]
fn only_read_only_git_subcommands_run() {
    let env = Env::new();
    let repo = make_repo(env.path(), "proj", Some(TOKEN_URL), "main");
    let real = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|d| d.join("git"))
        .find(|p| p.is_file())
        .unwrap();
    let log = env.path().join("git-calls.log");
    env.fake_git(&format!(
        "#!/bin/sh\necho \"$* | prompt=$GIT_TERMINAL_PROMPT dir=${{GIT_DIR-unset}}\" >> '{}'\nexec '{}' \"$@\"\n",
        log.display(),
        real.display()
    ));

    capture(None, Some(&repo), &[]);

    assert_eq!(GIT_TIMEOUT, Duration::from_secs(2));
    let calls = fs::read_to_string(&log).unwrap();
    let mut subcommands = Vec::new();
    for line in calls.lines() {
        let (argv, env_part) = line.split_once(" | ").unwrap();
        assert_eq!(env_part, "prompt=0 dir=unset", "{line}");
        let words: Vec<&str> = argv.split(' ').collect();
        let allowed = words[0] == "rev-parse"
            || words == ["branch", "--show-current"]
            || words == ["remote", "get-url", "origin"];
        assert!(allowed, "{argv}");
        subcommands.push(words[0]);
    }
    subcommands.sort_unstable();
    assert_eq!(subcommands, ["branch", "remote", "rev-parse"]);
}

#[test]
fn git_dir_from_the_environment_does_not_redirect_git() {
    let env = Env::new();
    let other = make_repo(env.path(), "other", None, "main");
    let here = make_repo(env.path(), "here", None, "main");
    std::env::set_var("GIT_DIR", other.join(".git"));
    assert_eq!(at(&here).repo.as_deref(), Some("here"));
}

// AC 9: store part pending HUN-269 (grep of the written files); here, no field holds a path

#[test]
fn no_absolute_path_in_any_field() {
    let env = Env::new();
    let repo = make_repo(
        env.path(),
        "proj",
        Some("git@github.com:o/r.git"),
        "fix/t120-x",
    );
    fs::create_dir(repo.join("sub")).unwrap();
    let wt = env.path().join("wt-t148");
    git(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            wt.to_str().unwrap(),
            "-b",
            "fix/t148-x",
        ],
    );
    let roots = [uri(&repo)];
    let all = [at(&repo.join("sub")), at(&wt), capture(None, None, &roots)];
    let root = env.path().to_str().unwrap().to_string();
    let canonical = fs::canonicalize(env.path()).unwrap();
    for c in &all {
        assert!(c.repo.is_some());
        let text = text_of(c);
        assert!(!text.contains(&root), "{text}");
        assert!(!text.contains(canonical.to_str().unwrap()), "{text}");
        assert!(
            !text.contains("/home/") && !text.contains("file://"),
            "{text}"
        );
    }
}
