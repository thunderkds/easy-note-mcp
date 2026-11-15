//! Where the notes dir is (FR-001): `EASY_NOTE_DIR`, else `dir` in config.toml, else `~/easy-notes`.

use std::path::{Path, PathBuf};

use crate::core::fsutil;

/// config.toml exists but cannot be used. The message names the file.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ConfigError(String);

fn home() -> Result<PathBuf, ConfigError> {
    std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| ConfigError("HOME is not set".to_string()))
}

pub fn config_path() -> Result<PathBuf, ConfigError> {
    Ok(home()?.join(".config/easy-note/config.toml"))
}

/// A leading `~` or `~/` means the home dir.
fn expand_user(value: &str) -> Result<PathBuf, ConfigError> {
    if value == "~" {
        home()
    } else if let Some(rest) = value.strip_prefix("~/") {
        Ok(home()?.join(rest))
    } else {
        Ok(PathBuf::from(value))
    }
}

pub fn notes_dir() -> Result<PathBuf, ConfigError> {
    if let Some(env) = std::env::var_os("EASY_NOTE_DIR").filter(|v| !v.is_empty()) {
        return expand_user(&env.to_string_lossy());
    }
    let path = config_path()?;
    let default = || Ok(home()?.join("easy-notes"));
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return default(),
        Err(e) => return Err(ConfigError(format!("cannot read {}: {e}", path.display()))),
    };
    let data: toml::Table = text
        .parse()
        .map_err(|e| ConfigError(format!("cannot read {}: {e}", path.display())))?;
    match data.get("dir") {
        None => default(),
        Some(toml::Value::String(s)) if !s.is_empty() => expand_user(s),
        Some(_) => Err(ConfigError(format!(
            "{}: 'dir' must be a non-empty string",
            path.display()
        ))),
    }
}

/// Write `dir = "<path>"` as the whole config.toml, atomically (used by `easy-note init`).
pub fn save_dir(path: &Path) -> Result<(), ConfigError> {
    let target = config_path()?;
    let io_err = |e: std::io::Error| ConfigError(format!("cannot write {}: {e}", target.display()));
    let parent = target.parent().expect("config path has a parent");
    std::fs::create_dir_all(parent).map_err(io_err)?;
    let mut table = toml::Table::new();
    table.insert(
        "dir".to_string(),
        toml::Value::String(path.to_string_lossy().into_owned()),
    );
    let data = toml::to_string(&table)
        .map_err(|e| ConfigError(format!("cannot write {}: {e}", target.display())))?;
    let (mut file, tmp) = fsutil::create_temp(parent).map_err(io_err)?;
    let result = fsutil::write_all_sync(&mut file, data.as_bytes())
        .and_then(|()| std::fs::rename(&tmp, &target));
    fsutil::unlink_quiet(&tmp);
    result.map_err(io_err)
}
