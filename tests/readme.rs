//! HUN-264 AC 4: every `easy-note <cmd>` the README names is a real subcommand, so the docs
//! cannot drift from the CLI.

use std::collections::BTreeSet;
use std::process::Command;

fn subcommands() -> BTreeSet<String> {
    let out = Command::new(env!("CARGO_BIN_EXE_easy-note"))
        .arg("--help")
        .output()
        .unwrap();
    let help = String::from_utf8(out.stdout).unwrap();
    help.lines()
        .skip_while(|l| !l.starts_with("Commands:"))
        .skip(1)
        .take_while(|l| l.starts_with("  "))
        .filter_map(|l| l.split_whitespace().next().map(str::to_string))
        .collect()
}

/// The word after each `easy-note ` in the text (`easy-note-mcp` and `easy-note.desktop` are not
/// commands, so the name must be followed by a space).
fn named_in(text: &str) -> BTreeSet<String> {
    text.match_indices("easy-note ")
        .filter(|(at, _)| {
            *at == 0 || {
                let before = text[..*at].chars().last().unwrap();
                !(before.is_alphanumeric() || before == '-' || before == '/' || before == '.')
            }
        })
        .filter_map(|(at, m)| {
            let rest = &text[at + m.len()..];
            if !rest.starts_with(|c: char| c.is_ascii_lowercase()) {
                return None;
            }
            Some(
                rest.chars()
                    .take_while(|c| c.is_ascii_lowercase() || *c == '-')
                    .collect(),
            )
        })
        .collect()
}

/// Only code is checked: fenced blocks and `inline` spans. Prose such as "easy-note never writes"
/// is not a command.
fn code_of(markdown: &str) -> String {
    let mut code = String::new();
    let mut fenced = false;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            code.push_str(line);
            code.push('\n');
        } else {
            for span in line.split('`').skip(1).step_by(2) {
                code.push_str(span);
                code.push('\n');
            }
        }
    }
    code
}

#[test]
fn every_command_the_readme_names_is_a_real_subcommand() {
    let readme =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")).unwrap();
    let real = subcommands();
    let named = named_in(&code_of(&readme));
    assert!(
        named.len() >= 8,
        "the README should show the main commands: {named:?}"
    );
    let unknown: Vec<_> = named.difference(&real).collect();
    assert!(
        unknown.is_empty(),
        "README names commands that do not exist: {unknown:?}"
    );
}

#[test]
fn the_readme_covers_install_first_note_clients_and_boundaries() {
    let readme =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")).unwrap();
    for must in [
        "cargo install --path .",
        "easy-note init",
        "claude mcp add easy-note -- easy-note serve",
        "[mcp_servers.easy-note]",
        "~/.cursor/mcp.json",
        "easy-note install claude",
        "## Notes are plain files",
        "easy-memory",
    ] {
        assert!(readme.contains(must), "README is missing {must:?}");
    }
}

#[test]
fn the_word_finder_skips_names_that_are_not_commands() {
    let got = named_in(
        "run easy-note ls, not easy-note-mcp or easy-note.desktop; easy-note -- x; easy-note add x",
    );
    assert_eq!(got, BTreeSet::from(["add".to_string(), "ls".to_string()]));
    let code = code_of("easy-note is a tool. Run `easy-note show x`.\n```\neasy-note path\n```\n");
    assert_eq!(
        named_in(&code),
        BTreeSet::from(["path".to_string(), "show".to_string()])
    );
}
