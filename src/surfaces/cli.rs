//! The `easy-note` command line (FR-013). Each command is a thin call into the core.
//!
//! Exit codes: 0 ok, 1 runtime error, 2 usage error. Every error text is masked before it is
//! printed (FR-014): a bad argument may be a pasted secret and an error message may echo it.

mod read;
mod scratch;
mod write;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::core::ai::AiError;
use crate::core::config::ConfigError;
use crate::core::guard;
use crate::core::query::{Filters, QueryError, DEFAULT_LIMIT};
use crate::core::store::StoreError;

const EXIT_CODES: &str = "Exit codes: 0 ok, 1 runtime error, 2 usage error.";

#[derive(Debug, Parser)]
#[command(name = "easy-note", version, about = "Capture one-line notes", after_help = EXIT_CODES)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// The narrowing options of `ls`, `search` and `stats` (FR-010). Dates are `YYYY-MM-DD`.
#[derive(Debug, Default, PartialEq, Eq, Args)]
pub struct FilterArgs {
    #[arg(long)]
    repo: Option<String>,
    #[arg(long)]
    task: Option<String>,
    #[arg(long)]
    tag: Option<String>,
    #[arg(long)]
    source: Option<String>,
    #[arg(long)]
    since: Option<String>,
    #[arg(long)]
    until: Option<String>,
    #[arg(long)]
    day: Option<String>,
}

impl From<FilterArgs> for Filters {
    fn from(f: FilterArgs) -> Filters {
        Filters {
            repo: f.repo,
            task: f.task,
            tag: f.tag,
            source: f.source,
            since: f.since,
            until: f.until,
            day: f.day,
        }
    }
}

/// What `install` and `uninstall` set up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Target {
    /// The Claude Code `/note` hook and `commands/note.md` in the Claude Code config dir.
    Claude,
    /// The `easy-note.desktop` launcher in `~/.local/share/applications`.
    Launcher,
    /// The tray's login item, `easy-note-tray.desktop` in `~/.config/autostart`.
    Tray,
    /// The `easy-note.base` Obsidian dashboard in the notes dir, and the notes dir as a vault.
    Obsidian,
}

/// What `ai` does with a note's AI detail file.
#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum AiAction {
    /// Delete the AI detail of a note (the note stays as it is).
    Rm { id: String },
    /// Rewrite the Obsidian view of every AI detail (needs a `.obsidian` folder in or above the notes dir).
    Render,
    /// Ask your own `claude` CLI to write the AI detail of notes (needs `[ai] enrich = "claude"`).
    Enrich {
        #[arg(required_unless_present = "pending", conflicts_with = "pending")]
        ids: Vec<String>,
        /// Enrich the notes with no AI detail or a stale one, newest first.
        #[arg(long)]
        pending: bool,
        /// With --pending: at most this many notes (1 to 20, default 20).
        #[arg(long, conflicts_with = "ids")]
        limit: Option<usize>,
    },
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Run the MCP server over stdio, or over HTTP on 127.0.0.1 with `--http`.
    Serve {
        /// Serve streamable HTTP instead of stdio.
        #[arg(long)]
        http: bool,
        /// The address to bind. Only 127.0.0.1 is allowed.
        #[arg(long, default_value = "127.0.0.1", requires = "http")]
        host: String,
        /// The HTTP port.
        #[arg(long, default_value_t = super::mcp::DEFAULT_PORT, requires = "http")]
        port: u16,
    },
    /// Choose the notes dir and save it in the config file.
    Init {
        /// The notes dir (default: the dir in use now).
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// Add a note (`n` is the same command). With no text: read one line, or all of a pipe.
    Add {
        /// Write the note in $EDITOR (the text, if any, is the start of the file).
        #[arg(short, long)]
        edit: bool,
        /// The note text: all words, joined by spaces. `#word` makes a tag.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        text: Vec<String>,
    },
    /// List notes, newest first.
    Ls {
        #[command(flatten)]
        filters: FilterArgs,
        /// How many notes to show (at most 50).
        #[arg(short = 'n', long, default_value_t = DEFAULT_LIMIT)]
        limit: usize,
    },
    /// Find notes that hold every word, in title, text or tags.
    Search {
        #[arg(required = true)]
        words: Vec<String>,
        #[command(flatten)]
        filters: FilterArgs,
        /// How many notes to show (at most 50).
        #[arg(short = 'n', long, default_value_t = DEFAULT_LIMIT)]
        limit: usize,
    },
    /// Print one note.
    Show { id: String },
    /// Edit a note in $EDITOR. The text and tags can change; secrets are masked on save.
    Edit { id: String },
    /// Add tags to a note, or take them off with --remove.
    Tag {
        id: String,
        #[arg(short, long)]
        remove: bool,
        /// Add the tags its AI detail suggests.
        #[arg(long, conflicts_with = "remove")]
        accept_ai: bool,
        #[arg(required_unless_present = "accept_ai")]
        tags: Vec<String>,
    },
    /// Move a note to the trash.
    Rm { id: String },
    /// Move a note back from the trash.
    Restore { id: String },
    /// List the trash, or with --empty --older-than 30d delete old trash for good.
    Trash {
        #[arg(long, requires = "older_than")]
        empty: bool,
        /// Age in days, like 30d.
        #[arg(long, requires = "empty")]
        older_than: Option<String>,
    },
    /// Work with the AI detail of a note (`ai rm <id>`).
    Ai {
        #[command(subcommand)]
        action: AiAction,
    },
    /// Count notes by repo, task, tag, source and month.
    Stats {
        #[arg(long)]
        json: bool,
        #[command(flatten)]
        filters: FilterArgs,
    },
    /// Look for skipped files, secrets in hand-edited files and a clashing `n` (read-only).
    Doctor,
    /// Print the notes dir.
    Path,
    /// Claude Code `UserPromptSubmit` hook: read the event on stdin, save a `/note ...` prompt.
    HookPrompt,
    /// Open a one-line box and save the answer as a note (for a key binding or a click).
    Quick {
        /// Take repo, branch and task from this folder (an editor passes its project folder).
        #[arg(long)]
        cwd: Option<PathBuf>,
    },
    /// Print the commands that bind a key to `easy-note quick` (changes nothing).
    Hotkey,
    /// Show an icon in the system tray (needs a build with `--features tray`).
    Tray,
    /// Open the dashboard: easy-note's own window in a `window` build on a desktop, else
    /// `easy-note.base` in Obsidian (run `install obsidian` first).
    Dashboard {
        /// Open the window on this note (its detail shown).
        #[arg(long, value_name = "ID")]
        note: Option<String>,
    },
    /// Set up the launcher, the Obsidian dashboard and vault, and the tray login item in one go.
    Setup,
    /// Add the Claude Code hook (shows a diff, keeps a backup of settings.json), the launcher, the
    /// tray login item, or the Obsidian dashboard file and vault.
    Install { target: Target },
    /// Remove what `install` added (a file that is not ours stays).
    Uninstall { target: Target },
}

/// When the binary is started as `n`, run it as `easy-note add`.
/// A help or version flag passes through only as the one argument, so `n --version` prints the
/// version and `n --version bump to 2.0` saves a note.
pub fn normalize_args(mut args: Vec<OsString>) -> Vec<OsString> {
    let as_n = args
        .first()
        .and_then(|a| Path::new(a).file_name())
        .is_some_and(|name| name == "n");
    let is_flag = |a: &OsString| matches!(a.to_str(), Some("--version" | "-V" | "--help" | "-h"));
    let sole_flag = args.len() == 2 && is_flag(&args[1]);
    if as_n && !sole_flag {
        args.insert(1, OsString::from("add"));
    }
    args
}

/// Only `serve` starts the MCP server and its async runtime (NFR-010). The `match` lists every
/// command, so a new one does not compile until it is classified.
pub fn starts_mcp(command: &Command) -> bool {
    match command {
        Command::Serve { .. } => true,
        Command::Init { .. }
        | Command::Add { .. }
        | Command::Ls { .. }
        | Command::Search { .. }
        | Command::Show { .. }
        | Command::Edit { .. }
        | Command::Tag { .. }
        | Command::Rm { .. }
        | Command::Restore { .. }
        | Command::Trash { .. }
        | Command::Ai { .. }
        | Command::Stats { .. }
        | Command::Doctor
        | Command::Path
        | Command::HookPrompt
        | Command::Quick { .. }
        | Command::Hotkey
        | Command::Tray
        | Command::Dashboard { .. }
        | Command::Setup
        | Command::Install { .. }
        | Command::Uninstall { .. } => false,
    }
}

/// A failed command: the exit code and the text for stderr (masked when printed).
#[derive(Debug)]
pub struct Fail {
    code: i32,
    msg: String,
}

impl Fail {
    /// The reader of our output went away (`easy-note ls | head`): stop without a word.
    fn reader_gone() -> Fail {
        Fail {
            code: 0,
            msg: String::new(),
        }
    }

    fn runtime(msg: impl Into<String>) -> Fail {
        Fail {
            code: 1,
            msg: msg.into(),
        }
    }

    fn usage(msg: impl Into<String>) -> Fail {
        Fail {
            code: 2,
            msg: msg.into(),
        }
    }
}

impl From<StoreError> for Fail {
    fn from(err: StoreError) -> Fail {
        match &err {
            StoreError::NotFound(id) => Fail::runtime(format!("no note with id {id}")),
            // The store prints the id with `{:?}`, which escapes quotes; the guard cannot match
            // an escaped secret. Print the id as it was typed, then mask it.
            StoreError::BadId(id) => Fail::usage(format!("not a note id: {id}")),
            StoreError::Invalid(_) => Fail::usage(err.to_string()),
            _ => Fail::runtime(err.to_string()),
        }
    }
}

impl From<QueryError> for Fail {
    fn from(err: QueryError) -> Fail {
        match err {
            QueryError::Store(err) => err.into(),
            other => Fail::usage(other.to_string()),
        }
    }
}

impl From<AiError> for Fail {
    fn from(err: AiError) -> Fail {
        match err {
            AiError::Store(err) => err.into(),
            refused => Fail::usage(refused.to_string()),
        }
    }
}

impl From<ConfigError> for Fail {
    fn from(err: ConfigError) -> Fail {
        Fail::runtime(err.to_string())
    }
}

impl From<std::io::Error> for Fail {
    fn from(err: std::io::Error) -> Fail {
        if err.kind() == std::io::ErrorKind::BrokenPipe {
            Fail::reader_gone()
        } else {
            Fail::runtime(err.to_string())
        }
    }
}

type Outcome = Result<(), Fail>;

/// Text that came from a note, an argument or an error: secrets masked, and every control
/// character but newline and tab shown as `\xNN`, so no note can drive the terminal.
pub(crate) fn safe(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in guard::mask_for_output(text).chars() {
        if c.is_control() && c != '\n' && c != '\t' {
            out.push_str(&format!("\\x{:02x}", u32::from(c)));
        } else {
            out.push(c);
        }
    }
    out
}

/// Print `text` and a newline to stdout. A closed pipe is not an error (see `Fail::reader_gone`).
fn say(text: &str) -> Outcome {
    say_raw(&format!("{text}\n"))
}

fn say_raw(text: &str) -> Outcome {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    out.write_all(text.as_bytes())?;
    out.flush()?;
    Ok(())
}

/// The kinds of a list, each once, in first-seen order.
fn unique_kinds<'a>(kinds: &[&'a str]) -> Vec<&'a str> {
    let mut unique: Vec<&str> = vec![];
    for kind in kinds {
        if !unique.contains(kind) {
            unique.push(kind);
        }
    }
    unique
}

/// "1 note", "3 notes".
fn count(n: usize, word: &str) -> String {
    format!("{n} {word}{}", if n == 1 { "" } else { "s" })
}

/// The one confirmation line every write prints (FR-021):
/// `✓ noted <id> · <repo> · <task> · N secret(s) masked: <kinds>`.
pub(crate) fn confirmation(
    verb: &str,
    id: &str,
    details: &[Option<&str>],
    masked: &[&str],
) -> String {
    let mut line = format!("✓ {verb} {id}");
    for detail in details.iter().flatten() {
        line.push_str(" · ");
        line.push_str(&safe(detail));
    }
    if !masked.is_empty() {
        line.push_str(&format!(
            " · {} masked: {}",
            count(masked.len(), "secret"),
            unique_kinds(masked).join(", ")
        ));
    }
    line
}

fn dispatch(command: Command) -> Outcome {
    match command {
        Command::Serve { .. } => Err(Fail::runtime("serve is not a note command")),
        Command::Init { dir } => write::init(dir),
        Command::Add { edit, text } => write::add(edit, &text),
        Command::Ls { filters, limit } => read::ls(filters.into(), limit),
        Command::Search {
            words,
            filters,
            limit,
        } => read::search(&words.join(" "), filters.into(), limit),
        Command::Show { id } => read::show(&id),
        Command::Edit { id } => write::edit(&id),
        Command::Tag {
            id,
            remove,
            accept_ai,
            tags,
        } => write::tag(&id, remove, accept_ai, &tags),
        Command::Rm { id } => write::rm(&id),
        Command::Restore { id } => write::restore(&id),
        Command::Trash { empty, older_than } => write::trash(empty, older_than.as_deref()),
        Command::Ai {
            action: AiAction::Rm { id },
        } => write::ai_rm(&id),
        Command::Ai {
            action: AiAction::Render,
        } => write::ai_render(),
        Command::Ai {
            action:
                AiAction::Enrich {
                    ids,
                    pending,
                    limit,
                },
        } => super::enrich::run(&ids, pending, limit).map_err(Fail::runtime),
        Command::Stats { json, filters } => read::stats(json, filters.into()),
        Command::Doctor => read::doctor(),
        Command::Path => read::path(),
        Command::HookPrompt => {
            super::hook::run();
            Ok(())
        }
        Command::Quick { cwd } => super::quick::run(cwd.as_deref()).map_err(Fail::runtime),
        Command::Hotkey => {
            let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
            say_raw(&super::quick::hotkey(&desktop))
        }
        Command::Tray => super::tray::run().map_err(Fail::runtime),
        Command::Dashboard { note } => dashboard(note.as_deref()),
        Command::Setup => setup(),
        Command::Install { target } => installed(target, super::install::Action::Install),
        Command::Uninstall { target } => installed(target, super::install::Action::Uninstall),
    }
}

/// `dashboard [--note <id>]`: an id that fails the store's id check is a usage error before
/// anything opens; a valid id that names no note is the page's "no such note".
fn dashboard(note: Option<&str>) -> Outcome {
    if let Some(id) = note {
        match crate::core::store::Store::open()?.canonical_id(id) {
            Ok(_) | Err(StoreError::NotFound(_)) => {}
            Err(e) => return Err(e.into()),
        }
    }
    super::dashboard::run(note).map_err(Fail::runtime)
}

/// `install` / `uninstall`: print what the installer reports. The text may quote lines of the
/// person's settings file, so it is masked like every other output.
fn installed(target: Target, action: super::install::Action) -> Outcome {
    let result = match target {
        Target::Claude => super::install::claude(action).map_err(|e| (String::new(), e)),
        Target::Launcher => super::install::launcher(action).map_err(|e| (String::new(), e)),
        Target::Tray => super::install::tray(action).map_err(|e| (String::new(), e)),
        Target::Obsidian => super::install::obsidian(action),
    };
    match result {
        Ok(text) => say_raw(&safe(&text)),
        Err((done, message)) => {
            say_raw(&safe(&done))?;
            Err(Fail::runtime(message))
        }
    }
}

/// `setup` (FR-041): `install launcher`, `install obsidian` and, in a tray build, `install tray`,
/// each under its own heading. A failed step does not stop the next; exit 1 if any failed. It
/// never runs `install claude` (DDR 0008 item 4).
fn setup() -> Outcome {
    use super::install::{self, Action::Install};
    let mut failed = vec![];
    for name in ["launcher", "obsidian", "tray"] {
        say(&format!("{name}:"))?;
        let result = match name {
            "launcher" => install::launcher(Install).map_err(|e| (String::new(), e)),
            "obsidian" => install::obsidian(Install),
            _ if cfg!(feature = "tray") => install::tray(Install).map_err(|e| (String::new(), e)),
            _ => Ok("tray not built in: skipped\n".to_string()),
        };
        match result {
            Ok(text) => say_raw(&safe(&text))?,
            Err((done, message)) => {
                say_raw(&safe(&done))?;
                eprintln!("easy-note: {}", safe(message.trim_end()));
                failed.push(name);
            }
        }
    }
    say("Using Claude Code? Run easy-note install claude to add the /note hook.")?;
    if failed.is_empty() {
        Ok(())
    } else {
        Err(Fail::runtime(format!(
            "setup: {} failed",
            failed.join(", ")
        )))
    }
}

/// Parse the arguments, run the command, return the exit code.
pub fn run(args: Vec<OsString>) -> i32 {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(err) => {
            let code = err.exit_code();
            let shown = err.render().to_string();
            if err.use_stderr() {
                eprint!("{}", safe(&shown));
            } else {
                let _ = say_raw(&shown);
            }
            return code;
        }
    };
    if starts_mcp(&cli.command) {
        if let Command::Serve { http, host, port } = &cli.command {
            return run_serve(*http, host, *port);
        }
    }
    match dispatch(cli.command) {
        Ok(()) => 0,
        Err(fail) => {
            if fail.code != 0 {
                eprintln!("easy-note: {}", safe(&fail.msg));
            }
            fail.code
        }
    }
}

/// `serve`: a host other than 127.0.0.1 exits 2 before anything binds; a failed start exits 1.
fn run_serve(http: bool, host: &str, port: u16) -> i32 {
    let mode = if http {
        if let Err(message) = super::mcp::check_host(host) {
            eprintln!("easy-note: {}", safe(&message));
            return 2;
        }
        super::mcp::Mode::Http { port }
    } else {
        super::mcp::Mode::Stdio
    };
    match super::mcp::serve(mode) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("easy-note: serve failed: {}", safe(&err.to_string()));
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn n_becomes_add() {
        assert_eq!(
            normalize_args(os(&["/home/x/.cargo/bin/n", "hello", "world"])),
            os(&["/home/x/.cargo/bin/n", "add", "hello", "world"])
        );
    }

    #[test]
    fn n_keeps_version_and_help_flags() {
        for flag in ["--version", "-V", "--help", "-h"] {
            assert_eq!(normalize_args(os(&["n", flag])), os(&["n", flag]));
        }
    }

    #[test]
    fn n_flag_with_more_words_is_a_note() {
        assert_eq!(
            normalize_args(os(&["n", "--version", "bump", "to", "2.0"])),
            os(&["n", "add", "--version", "bump", "to", "2.0"])
        );
        assert_eq!(
            normalize_args(os(&["n", "-h", "me"])),
            os(&["n", "add", "-h", "me"])
        );
    }

    #[test]
    fn safe_masks_secrets_and_shows_control_characters() {
        assert_eq!(
            safe("a\u{1b}[2Jb\u{7}\u{7f}\u{9b}c\rd"),
            "a\\x1b[2Jb\\x07\\x7f\\x9bc\\x0dd"
        );
        assert_eq!(safe("tab\there\nnext é ✓"), "tab\there\nnext é ✓");
        assert_eq!(
            safe(&format!("k {}", concat!("AK", "IAIOSFODNN7EXAMPLE"))),
            "k [masked:aws_key]"
        );
    }

    #[test]
    fn easy_note_name_is_not_rewritten() {
        let args = os(&["easy-note", "add", "hello"]);
        assert_eq!(normalize_args(args.clone()), args);
    }

    #[test]
    fn n_hello_parses_as_add_hello() {
        let cli = Cli::try_parse_from(normalize_args(os(&["n", "hello"]))).unwrap();
        assert_eq!(
            cli.command,
            Command::Add {
                edit: false,
                text: vec!["hello".into()]
            }
        );
    }

    #[test]
    fn n_keeps_hyphen_words_and_dash_e_only_in_front() {
        let parse = |args: &[&str]| {
            Cli::try_parse_from(normalize_args(os(args)))
                .unwrap()
                .command
        };
        assert_eq!(
            parse(&["n", "-e"]),
            Command::Add {
                edit: true,
                text: vec![]
            }
        );
        assert_eq!(
            parse(&["n", "fix", "-e", "--odd"]),
            Command::Add {
                edit: false,
                text: vec!["fix".into(), "-e".into(), "--odd".into()]
            }
        );
    }

    #[test]
    fn version_never_reaches_a_command() {
        // `--version` ends parsing in clap, so no command (and no MCP runtime) is chosen.
        for argv in [
            os(&["easy-note", "--version"]),
            normalize_args(os(&["n", "--version"])),
        ] {
            let err = Cli::try_parse_from(argv).unwrap_err();
            assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
        }
    }

    /// One value of every command. A new `Command` variant does not compile in `starts_mcp`
    /// until it is classified; this list makes the test name it too.
    fn every_command() -> Vec<Command> {
        let id = || "20261114-164713-ab12".to_string();
        vec![
            Command::Serve {
                http: false,
                host: "127.0.0.1".into(),
                port: 8769,
            },
            Command::Init { dir: None },
            Command::Add {
                edit: false,
                text: vec![],
            },
            Command::Ls {
                filters: FilterArgs::default(),
                limit: 1,
            },
            Command::Search {
                words: vec!["x".into()],
                filters: FilterArgs::default(),
                limit: 1,
            },
            Command::Show { id: id() },
            Command::Edit { id: id() },
            Command::Tag {
                id: id(),
                remove: false,
                accept_ai: false,
                tags: vec!["x".into()],
            },
            Command::Rm { id: id() },
            Command::Restore { id: id() },
            Command::Trash {
                empty: false,
                older_than: None,
            },
            Command::Ai {
                action: AiAction::Rm { id: id() },
            },
            Command::Ai {
                action: AiAction::Enrich {
                    ids: vec![],
                    pending: true,
                    limit: None,
                },
            },
            Command::Stats {
                json: false,
                filters: FilterArgs::default(),
            },
            Command::Doctor,
            Command::Path,
            Command::HookPrompt,
            Command::Quick { cwd: None },
            Command::Hotkey,
            Command::Tray,
            Command::Dashboard { note: None },
            Command::Setup,
            Command::Install {
                target: Target::Claude,
            },
            Command::Uninstall {
                target: Target::Claude,
            },
            Command::Install {
                target: Target::Launcher,
            },
            Command::Uninstall {
                target: Target::Launcher,
            },
            Command::Install {
                target: Target::Tray,
            },
            Command::Uninstall {
                target: Target::Tray,
            },
            Command::Install {
                target: Target::Obsidian,
            },
            Command::Uninstall {
                target: Target::Obsidian,
            },
        ]
    }

    #[test]
    fn only_serve_starts_mcp() {
        for command in every_command() {
            assert_eq!(
                starts_mcp(&command),
                matches!(command, Command::Serve { .. }),
                "{command:?}"
            );
        }
    }

    #[test]
    fn confirmation_line_shape() {
        assert_eq!(
            confirmation("noted", "i1", &[None, None], &[]),
            "✓ noted i1"
        );
        assert_eq!(
            confirmation(
                "noted",
                "i1",
                &[Some("repo"), None, Some("T1")],
                &["aws_key"]
            ),
            "✓ noted i1 · repo · T1 · 1 secret masked: aws_key"
        );
        assert_eq!(
            confirmation("edited", "i1", &[], &["jwt", "jwt", "bearer"]),
            "✓ edited i1 · 3 secrets masked: jwt, bearer"
        );
    }
}
