//! The `easy-note` command line. Stubs only: no note logic yet.

use std::ffi::OsString;
use std::path::Path;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "easy-note", version, about = "Capture one-line notes")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Run the MCP server over stdio.
    Serve,
    /// Add a note (`n` is the same command).
    Add {
        /// The note text.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        text: Vec<String>,
    },
}

/// When the binary is started as `n`, run it as `easy-note add`.
/// Help and version flags pass through, so `n --version` prints the version.
pub fn normalize_args(mut args: Vec<OsString>) -> Vec<OsString> {
    let as_n = args
        .first()
        .and_then(|a| Path::new(a).file_name())
        .is_some_and(|name| name == "n");
    let is_flag = |a: &OsString| matches!(a.to_str(), Some("--version" | "-V" | "--help" | "-h"));
    if as_n && !args.get(1).is_some_and(is_flag) {
        args.insert(1, OsString::from("add"));
    }
    args
}

/// Only `serve` starts the MCP server and its async runtime (NFR-010).
pub fn starts_mcp(command: &Command) -> bool {
    matches!(command, Command::Serve)
}

/// Parse the arguments, run the command, return the exit code.
pub fn run(args: Vec<OsString>) -> i32 {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(err) => {
            let code = err.exit_code();
            let _ = err.print();
            return code;
        }
    };
    if starts_mcp(&cli.command) {
        return match super::mcp::serve() {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("easy-note: serve failed: {err}");
                1
            }
        };
    }
    eprintln!("easy-note: add is not implemented yet");
    2
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
                text: vec!["hello".into()]
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

    #[test]
    fn only_serve_starts_mcp() {
        assert!(starts_mcp(&Command::Serve));
        assert!(!starts_mcp(&Command::Add { text: vec![] }));
    }
}
