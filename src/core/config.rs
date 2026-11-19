//! Where the notes dir is (FR-001): `EASY_NOTE_DIR`, else `dir` in config.toml, else `~/easy-notes`.
//! Also the `[ai]` section (FR-038, DDR 0007): whether `ai enrich` may start the person's `claude`.

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

/// config.toml as a table; `None` when the file does not exist.
fn read_table(path: &Path) -> Result<Option<toml::Table>, ConfigError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(ConfigError(format!("cannot read {}: {e}", path.display()))),
    };
    text.parse()
        .map(Some)
        .map_err(|e| ConfigError(format!("cannot read {}: {e}", path.display())))
}

fn env_dir() -> Option<Result<PathBuf, ConfigError>> {
    std::env::var_os("EASY_NOTE_DIR")
        .filter(|v| !v.is_empty())
        .map(|env| expand_user(&env.to_string_lossy()))
}

/// `dir` of a read config.toml, else `~/easy-notes`.
fn dir_of(data: Option<&toml::Table>, path: &Path) -> Result<PathBuf, ConfigError> {
    match data.and_then(|d| d.get("dir")) {
        None => Ok(home()?.join("easy-notes")),
        Some(toml::Value::String(s)) if !s.is_empty() => expand_user(s),
        Some(_) => Err(ConfigError(format!(
            "{}: 'dir' must be a non-empty string",
            path.display()
        ))),
    }
}

pub fn notes_dir() -> Result<PathBuf, ConfigError> {
    if let Some(env) = env_dir() {
        return env;
    }
    let path = config_path()?;
    dir_of(read_table(&path)?.as_ref(), &path)
}

/// The `[ai]` section of config.toml (FR-038). `enrich` is true for `"claude"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiSettings {
    pub enrich: bool,
    pub command: String,
    pub claude_config_dir: Option<PathBuf>,
    pub model: String,
    pub timeout_secs: u64,
    pub after_save: bool,
}

impl Default for AiSettings {
    fn default() -> AiSettings {
        AiSettings {
            enrich: false,
            command: "claude".to_string(),
            claude_config_dir: None,
            model: "haiku".to_string(),
            timeout_secs: 120,
            after_save: false,
        }
    }
}

/// `[ai]` of a read config.toml. Unknown keys are ignored. An error names the key, never the value.
fn ai_of(data: Option<&toml::Table>, path: &Path) -> Result<AiSettings, ConfigError> {
    let mut settings = AiSettings::default();
    let Some(section) = data.and_then(|d| d.get("ai")) else {
        return Ok(settings);
    };
    let bad = |key: &str, rule: &str| ConfigError(format!("{}: [ai] {key} {rule}", path.display()));
    let toml::Value::Table(ai) = section else {
        return Err(bad("section", "must be a table"));
    };
    let text = |key: &str| match ai.get(key) {
        None => Ok(None),
        Some(toml::Value::String(s)) if !s.is_empty() => Ok(Some(s.as_str())),
        Some(_) => Err(bad(key, "must be a non-empty string")),
    };
    settings.enrich = match ai.get("enrich") {
        None => false,
        Some(toml::Value::String(s)) if s == "off" => false,
        Some(toml::Value::String(s)) if s == "claude" => true,
        Some(_) => return Err(bad("enrich", "must be \"off\" or \"claude\"")),
    };
    if let Some(command) = text("command")? {
        settings.command = command.to_string();
    }
    settings.claude_config_dir = text("claude_config_dir")?.map(expand_user).transpose()?;
    if let Some(model) = text("model")? {
        // The model is one argv item after `--model`; a leading dash would read as a flag.
        if model.starts_with('-') {
            return Err(bad("model", "must not start with -"));
        }
        settings.model = model.to_string();
    }
    settings.timeout_secs = match ai.get("timeout_secs") {
        None => settings.timeout_secs,
        Some(toml::Value::Integer(n)) if (1..=600).contains(n) => n.unsigned_abs(),
        Some(_) => return Err(bad("timeout_secs", "must be a whole number from 1 to 600")),
    };
    settings.after_save = match ai.get("after_save") {
        None => false,
        Some(toml::Value::Boolean(b)) => *b,
        Some(_) => return Err(bad("after_save", "must be true or false")),
    };
    Ok(settings)
}

/// `[ai]` in config.toml; a missing file or section gives the defaults (`enrich` off).
pub fn ai_settings() -> Result<AiSettings, ConfigError> {
    let path = config_path()?;
    ai_of(read_table(&path)?.as_ref(), &path)
}

/// For a save path: the notes dir, and whether the save starts `ai enrich` (`enrich = "claude"`
/// and `after_save = true`), from one read of config.toml. A bad `[ai]` section, or a bad file
/// or no HOME while `EASY_NOTE_DIR` names the dir, means no start: a save never fails for `[ai]`.
pub fn notes_dir_and_after_save() -> Result<(PathBuf, bool), ConfigError> {
    let read = config_path().and_then(|path| Ok((read_table(&path)?, path)));
    let after_save = read.as_ref().is_ok_and(|(data, path)| {
        ai_of(data.as_ref(), path).is_ok_and(|ai| ai.enrich && ai.after_save)
    });
    let dir = match env_dir() {
        Some(env) => env?,
        None => {
            let (data, path) = read?;
            dir_of(data.as_ref(), &path)?
        }
    };
    Ok((dir, after_save))
}

/// The capture prefix of the Claude Code hook when config.toml sets none (DDR 0003, D18).
pub const DEFAULT_CAPTURE_PREFIX: &str = "/note ";

/// `capture_prefix` in config.toml, else `/note `. A missing file or key gives the default. An
/// unreadable file, a bad TOML text, or a value that is not a non-empty string is an error (an
/// empty prefix would match every prompt); the hook treats any error as "use the default".
pub fn capture_prefix() -> Result<String, ConfigError> {
    let path = config_path()?;
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DEFAULT_CAPTURE_PREFIX.to_string())
        }
        Err(e) => return Err(ConfigError(format!("cannot read {}: {e}", path.display()))),
    };
    let data: toml::Table = text
        .parse()
        .map_err(|e| ConfigError(format!("cannot read {}: {e}", path.display())))?;
    match data.get("capture_prefix") {
        None => Ok(DEFAULT_CAPTURE_PREFIX.to_string()),
        Some(toml::Value::String(s)) if !s.is_empty() => Ok(s.clone()),
        Some(_) => Err(ConfigError(format!(
            "{}: 'capture_prefix' must be a non-empty string",
            path.display()
        ))),
    }
}

/// Set `dir = "<path>"` in config.toml, atomically, keeping every other key and table, such as
/// `[ai]` (used by `easy-note init`). An unreadable file is an error: it is not replaced.
pub fn save_dir(path: &Path) -> Result<(), ConfigError> {
    let target = config_path()?;
    let io_err = |e: std::io::Error| ConfigError(format!("cannot write {}: {e}", target.display()));
    let parent = target.parent().expect("config path has a parent");
    std::fs::create_dir_all(parent).map_err(io_err)?;
    let mut table = read_table(&target)?.unwrap_or_default();
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
