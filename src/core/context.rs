//! Context of a new note: repo, remote, branch, task, commit (FR-004, FR-005, FR-028).
//!
//! `capture` returns only the fields it can state with certainty, for the store. A field that
//! cannot be read is left out, never guessed. No path is ever returned (FR-004).
//!
//! Git is read-only (NFR-001): only `rev-parse` (the repo, and `--short HEAD` for the commit),
//! `branch --show-current` and `remote get-url origin`, each with a 2 s timeout, no shell, no
//! prompt. A timeout or a missing `git` drops every git field at once, so a slow repo costs one
//! wait and a note is still written. A repo with no commit yet has no `commit`, only that.
//!
//! Working folder, as decided in docs/experiments/T001-client-launch.md: the explicit `cwd`, else
//! the first MCP root, else the process cwd. The first source that is present is the one used;
//! a source that is not a git repo gives no fields rather than falling through to a guess.

use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::time::Duration;

use percent_encoding::percent_decode_str;
use regex::Regex;
use url::Url;
use wait_timeout::ChildExt;

/// Wait for one git call (NFR-001).
pub const GIT_TIMEOUT: Duration = Duration::from_secs(2);

/// Tried in this order; the first match in the branch name wins (FR-005). A hit may not touch a
/// letter or digit on either side, so `art1234` and `T1234x` do not match. The `regex` crate has
/// no look-around, so the boundary is part of the match and the task is capture group 1.
static TASK_PATTERNS: LazyLock<[(Regex, bool); 2]> = LazyLock::new(|| {
    [
        (
            Regex::new(r"(?:^|[^A-Za-z0-9])([Tt][0-9]{3,4})(?:[^A-Za-z0-9]|$)").unwrap(),
            true,
        ),
        (
            Regex::new(r"(?:^|[^A-Za-z0-9])([A-Z][A-Z0-9]+-[0-9]+)(?:[^A-Za-z0-9]|$)").unwrap(),
            false,
        ),
    ]
});
static SCP_REMOTE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:[^@/\s]+@)?[^:/\s]+:(?P<path>.+)$").unwrap());
static BAD_SEGMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\s@:?#]").unwrap());

/// Git variables that would point git at another repo than the folder we ask about.
const GIT_ENV_DROP: [&str; 5] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
];

/// The context fields of a note. `None` means the field is absent from the note.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Context {
    pub repo: Option<String>,
    pub remote: Option<String>,
    pub branch: Option<String>,
    pub task: Option<String>,
    /// Short HEAD sha at capture.
    pub commit: Option<String>,
}

impl Context {
    /// The fields in the form `Store::add` takes; an absent field stays `None` and is left out.
    pub fn fields(&self) -> Vec<(&'static str, Option<&str>)> {
        vec![
            ("repo", self.repo.as_deref()),
            ("remote", self.remote.as_deref()),
            ("branch", self.branch.as_deref()),
            ("task", self.task.as_deref()),
            ("commit", self.commit.as_deref()),
        ]
    }
}

/// git is missing or exceeded the timeout: all git fields are dropped.
struct GitUnavailable;

/// Context fields for a new note.
///
/// `roots` are the MCP `roots/list` answer (`file://` URIs or plain folders); an empty slice
/// means absent. An explicit `task` always wins over the branch.
pub fn capture(task: Option<&str>, cwd: Option<&Path>, roots: &[String]) -> Context {
    let mut context = Context {
        task: task
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string),
        ..Context::default()
    };
    let Some(folder) = working_folder(cwd, roots) else {
        return context;
    };
    if let Ok(found) = git_fields(&folder) {
        context.repo = found.repo;
        context.remote = found.remote;
        context.branch = found.branch;
        context.commit = found.commit;
        context.task = context.task.or(found.task);
    }
    context
}

pub fn task_from_branch(branch: &str) -> Option<String> {
    TASK_PATTERNS.iter().find_map(|(pattern, upper)| {
        let task = pattern.captures(branch)?.get(1)?.as_str();
        Some(if *upper {
            task.to_uppercase()
        } else {
            task.to_string()
        })
    })
}

/// `owner/name` of a remote URL with userinfo, host, port, query and `.git` dropped; else `None`.
///
/// A local path or `file://` remote gives `None`, because it would be a filesystem path.
pub fn remote_slug(url: &str) -> Option<String> {
    let url = url.trim();
    let path = if url.contains("://") {
        let parts = Url::parse(url).ok()?;
        if parts.scheme() == "file" || parts.host_str().is_none_or(str::is_empty) {
            return None;
        }
        parts.path().to_string()
    } else {
        SCP_REMOTE.captures(url)?["path"].to_string()
    };
    let mut segments: Vec<String> = path
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .map(|s| percent_decode_str(s).decode_utf8_lossy().into_owned())
        .collect();
    if let Some(last) = segments.last_mut() {
        if let Some(stem) = last.strip_suffix(".git") {
            *last = stem.to_string();
        }
    }
    let [.., owner, name] = segments.as_slice() else {
        return None;
    };
    let bad = |s: &String| s.is_empty() || s == "." || s == ".." || BAD_SEGMENT.is_match(s);
    if bad(owner) || bad(name) {
        return None;
    }
    Some(format!("{owner}/{name}"))
}

fn working_folder(cwd: Option<&Path>, roots: &[String]) -> Option<PathBuf> {
    if let Some(cwd) = cwd.filter(|c| !c.as_os_str().is_empty()) {
        return absolute(cwd);
    }
    roots
        .iter()
        .find_map(|root| root_to_folder(root))
        .or_else(|| std::env::current_dir().ok())
}

fn root_to_folder(root: &str) -> Option<PathBuf> {
    if root.is_empty() {
        return None;
    }
    if !root.contains("://") {
        return absolute(Path::new(root));
    }
    // `to_file_path` accepts only an empty or `localhost` host, and decodes `%20` and the like.
    Url::parse(root).ok()?.to_file_path().ok()
}

/// Absolute path with `.` and `..` resolved by text, as the symlinks are not followed.
fn absolute(path: &Path) -> Option<PathBuf> {
    let full = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    let mut out = PathBuf::new();
    for part in full.components() {
        match part {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    Some(out)
}

fn git_fields(folder: &Path) -> Result<Context, GitUnavailable> {
    let mut found = Context::default();
    let Some(top) = git(
        folder,
        &["rev-parse", "--git-common-dir", "--is-bare-repository"],
    )?
    else {
        return Ok(found);
    };
    let lines: Vec<&str> = top.lines().collect();
    let [common, is_bare] = lines.as_slice() else {
        return Ok(found);
    };
    let common_dir = absolute(&folder.join(common));
    let base = |p: Option<&Path>| {
        p.and_then(Path::file_name)
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    let mut name = base(common_dir.as_deref());
    if name == ".git" && *is_bare != "true" {
        name = base(common_dir.as_deref().and_then(Path::parent));
    } else if let Some(stem) = name.strip_suffix(".git") {
        name = stem.to_string();
    }
    if !name.is_empty() {
        found.repo = Some(name);
    }

    // Empty on a detached HEAD.
    if let Some(branch) = git(folder, &["branch", "--show-current"])?.filter(|b| !b.is_empty()) {
        found.task = task_from_branch(&branch);
        found.branch = Some(branch);
    }

    // Fails in a repo with no commit yet: then only `commit` is absent.
    found.commit = git(folder, &["rev-parse", "--short", "HEAD"])?.filter(|c| !c.is_empty());

    found.remote = git(folder, &["remote", "get-url", "origin"])?
        .as_deref()
        .and_then(remote_slug);
    Ok(found)
}

/// Stdout of one read-only git call; `Ok(None)` when git says no (not a repo, no origin).
fn git(folder: &Path, args: &[&str]) -> Result<Option<String>, GitUnavailable> {
    let mut command = Command::new("git");
    command
        .args(args)
        .current_dir(folder)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    for key in GIT_ENV_DROP {
        command.env_remove(key);
    }
    // `git` or the folder is missing: no context.
    let mut child = command.spawn().map_err(|_| GitUnavailable)?;
    let status = match child.wait_timeout(GIT_TIMEOUT) {
        Ok(Some(status)) => status,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(GitUnavailable);
        }
    };
    if !status.success() {
        return Ok(None);
    }
    let mut out = Vec::new();
    child
        .stdout
        .take()
        .ok_or(GitUnavailable)?
        .read_to_end(&mut out)
        .map_err(|_| GitUnavailable)?;
    let text = String::from_utf8(out).map_err(|_| GitUnavailable)?;
    Ok(Some(text.trim().to_string()))
}
