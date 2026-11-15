//! Context of a new note: repo, remote, branch, task (FR-004, FR-005). Stub: the red step.

use std::path::Path;
use std::time::Duration;

pub const GIT_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Context {
    pub repo: Option<String>,
    pub remote: Option<String>,
    pub branch: Option<String>,
    pub task: Option<String>,
}

pub fn capture(_task: Option<&str>, _cwd: Option<&Path>, _roots: &[String]) -> Context {
    unimplemented!()
}

pub fn task_from_branch(_branch: &str) -> Option<String> {
    unimplemented!()
}

pub fn remote_slug(_url: &str) -> Option<String> {
    unimplemented!()
}
