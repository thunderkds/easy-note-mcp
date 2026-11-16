//! The temp file `edit` and `add -e` give the editor: `<notes dir>/.easy-note/edit/<name>-<pid>.md`.
//!
//! It lives inside the notes dir because easy-note writes nowhere else (DDR 0001). It holds what
//! the person types, so a run that is killed must not leave it for ever: the next `edit` or
//! `add -e` sweeps the files of processes that are gone, and `doctor` lists them.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use crate::core::store::Store;

fn dir_of(root: &Path) -> PathBuf {
    root.join(".easy-note").join("edit")
}

/// `<name>-<pid>.md` gives the pid.
fn pid_of(file_name: &str) -> Option<u32> {
    file_name
        .strip_suffix(".md")?
        .rsplit_once('-')?
        .1
        .parse()
        .ok()
}

/// A process other than this one that still runs (Linux only, DDR 0005).
fn is_live(pid: u32) -> bool {
    pid != std::process::id() && Path::new("/proc").join(pid.to_string()).exists()
}

/// Files in the scratch dir that no running `easy-note` owns: the file of a dead process, and any
/// other file (an editor's swap or backup file) unless its name holds the name of a live scratch
/// file. Sorted.
pub fn leftovers(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir_of(root)) else {
        return vec![];
    };
    let files: Vec<(PathBuf, String)> = entries
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| (e.path(), e.file_name().to_string_lossy().into_owned()))
        .collect();
    let live: Vec<&str> = files
        .iter()
        .filter(|(_, name)| pid_of(name).is_some_and(is_live))
        .map(|(_, name)| name.as_str())
        .collect();
    let mut gone: Vec<PathBuf> = files
        .iter()
        .filter(|(_, name)| match pid_of(name) {
            Some(pid) => !is_live(pid),
            None => !live.iter().any(|l| name.contains(l)),
        })
        .map(|(path, _)| path.clone())
        .collect();
    gone.sort();
    gone
}

/// A scratch file. Removed when dropped, with the folders this run made.
pub struct Scratch {
    pub path: PathBuf,
    root: PathBuf,
    root_existed: bool,
}

impl Scratch {
    /// Sweep the leftovers of dead runs, then make `<name>-<pid>.md` holding `content` (0600).
    pub fn create(store: &Store, name: &str, content: &str) -> io::Result<Scratch> {
        let root_existed = store.root.exists();
        let dir = dir_of(&store.root);
        fs::create_dir_all(&dir)?;
        let scratch = Scratch {
            path: dir.join(format!("{name}-{}.md", std::process::id())),
            root: store.root.clone(),
            root_existed,
        };
        for old in leftovers(&store.root) {
            let _ = fs::remove_file(old);
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&scratch.path)?;
        file.write_all(content.as_bytes())?;
        Ok(scratch)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
        let edit_dir = dir_of(&self.root);
        // `remove_dir` fails on a folder that holds anything, so a person's files are safe.
        let _ = fs::remove_dir(&edit_dir);
        if let Some(hidden) = edit_dir.parent() {
            let _ = fs::remove_dir(hidden);
        }
        if !self.root_existed {
            let _ = fs::remove_dir(&self.root);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pid_comes_from_the_file_name() {
        assert_eq!(pid_of("new-123.md"), Some(123));
        assert_eq!(pid_of("20261114-164713-ab12-77.md"), Some(77));
        assert_eq!(pid_of("20261114-164713-ab12.md"), None);
        assert_eq!(pid_of(".new-123.md.swp"), None);
        assert_eq!(pid_of("stray.txt"), None);
    }
}
