//! The binary through a real process: HUN-268 (R1) pins (`easy-note` and `n` run, `serve` answers
//! `initialize` on stdio and exits on EOF) and the HUN-257 CLI commands (FR-013, FR-020, FR-021).
//! Tests use a temp HOME and EASY_NOTE_DIR; they never touch the real ones.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_easy-note"))
}

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("easy-note-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(program: &Path, args: &[&str], home: &Path) -> Output {
    Command::new(program)
        .args(args)
        .env("HOME", home)
        .env("EASY_NOTE_DIR", home.join("notes"))
        .current_dir(home)
        .output()
        .expect("run binary")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn version_prints_name_and_number() {
    let home = sandbox("version");
    let out = run(&bin(), &["--version"], &home);
    assert!(out.status.success());
    assert_eq!(text(&out.stdout).trim(), "easy-note 0.1.0");
}

#[test]
fn n_symlink_prints_same_version() {
    let home = sandbox("n-version");
    let link = home.join("n");
    std::os::unix::fs::symlink(bin(), &link).unwrap();
    let out = run(&link, &["--version"], &home);
    assert!(out.status.success());
    assert_eq!(text(&out.stdout).trim(), "easy-note 0.1.0");
}

// ---------------------------------------------------------------------------------------------
// HUN-257: the commands, through the real binary
// ---------------------------------------------------------------------------------------------

const AWS: &str = concat!("AK", "IAIOSFODNN7EXAMPLE");
const GITHUB: &str = concat!("gh", "p_EXAMPLEexampleEXAMPLEexampleEXAMPLE123456");

/// Run `program args` in `home` with the temp env; `stdin` is piped (else /dev/null).
fn exec(
    home: &Path,
    program: &Path,
    args: &[&str],
    stdin: Option<&str>,
    tweak: impl FnOnce(&mut Command),
) -> Output {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .env("HOME", home)
        .env("EASY_NOTE_DIR", home.join("notes"))
        .env_remove("EDITOR")
        .env_remove("VISUAL")
        .current_dir(home)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    tweak(&mut cmd);
    let mut child = cmd.spawn().expect("spawn");
    if let Some(input) = stdin {
        let mut pipe = child.stdin.take().unwrap();
        pipe.write_all(input.as_bytes()).unwrap();
    }
    child.wait_with_output().expect("wait")
}

fn easy(home: &Path, args: &[&str]) -> Output {
    exec(home, &bin(), args, None, |_| {})
}

/// Success is asserted; returns stdout.
fn ok(home: &Path, args: &[&str]) -> String {
    let out = easy(home, args);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{args:?} stderr: {}",
        text(&out.stderr)
    );
    text(&out.stdout)
}

fn fails(home: &Path, args: &[&str], code: i32) -> String {
    let out = easy(home, args);
    assert_eq!(
        out.status.code(),
        Some(code),
        "{args:?} stdout: {} stderr: {}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert!(out.stdout.is_empty(), "no stdout on failure: {args:?}");
    text(&out.stderr)
}

/// `✓ noted <id> · ...` -> id.
fn noted_id(stdout: &str) -> String {
    let line = stdout.lines().next().expect("a confirmation line");
    assert!(line.starts_with("✓ noted "), "line: {line}");
    line.split_whitespace().nth(2).unwrap().to_string()
}

fn add(home: &Path, words: &[&str]) -> String {
    let mut args = vec!["add"];
    args.extend(words);
    noted_id(&ok(home, &args))
}

fn note_file(home: &Path, id: &str) -> PathBuf {
    home.join("notes")
        .join(&id[..4])
        .join(&id[4..6])
        .join(format!("{id}.md"))
}

fn parsed(home: &Path, id: &str) -> easy_note_mcp::core::format::Note {
    let raw = std::fs::read_to_string(note_file(home, id)).unwrap();
    easy_note_mcp::core::format::parse(&raw).expect("the file parses")
}

fn tag_list(note: &easy_note_mcp::core::format::Note) -> Vec<String> {
    match note.fields.get("tags") {
        Some(easy_note_mcp::core::format::Value::List(list)) => list.clone(),
        other => panic!("tags: {other:?}"),
    }
}

/// An `$EDITOR` stub: a shell script run with the file path as `$1`.
fn editor(home: &Path, name: &str, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = home.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn with_editor(home: &Path, args: &[&str], editor: &Path) -> Output {
    exec(home, &bin(), args, None, |c| {
        c.env("EDITOR", editor);
    })
}

fn link_n(home: &Path) -> PathBuf {
    let link = home.join("n");
    if !link.exists() {
        std::os::unix::fs::symlink(bin(), &link).unwrap();
    }
    link
}

#[test]
fn first_add_with_no_config_creates_easy_notes_and_prints_the_path_once() {
    let home = sandbox("zero-setup");
    let first = exec(&home, &bin(), &["add", "hello", "#a"], None, |c| {
        c.env_remove("EASY_NOTE_DIR");
    });
    assert_eq!(first.status.code(), Some(0), "{}", text(&first.stderr));
    let dir = home.join("easy-notes");
    assert!(dir.is_dir(), "~/easy-notes is created");
    let shown = format!("{}", dir.display());
    let all = format!("{}{}", text(&first.stdout), text(&first.stderr));
    assert_eq!(all.matches(&shown).count(), 1, "the path once: {all}");
    let id = noted_id(&text(&first.stdout));
    assert!(dir
        .join(&id[..4])
        .join(&id[4..6])
        .join(format!("{id}.md"))
        .is_file());
    assert!(!home.join(".config").exists(), "add writes no config");

    let second = exec(&home, &bin(), &["add", "again"], None, |c| {
        c.env_remove("EASY_NOTE_DIR");
    });
    assert_eq!(second.status.code(), Some(0));
    let all = format!("{}{}", text(&second.stdout), text(&second.stderr));
    assert!(!all.contains(&shown), "the second add stays quiet: {all}");
}

#[test]
fn n_saves_the_exact_words_with_the_hash_tag() {
    let home = sandbox("n-words");
    let n = link_n(&home);
    let out = exec(&home, &n, &["retry", "#flaky", "it's!"], None, |_| {});
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let id = noted_id(&text(&out.stdout));
    let note = parsed(&home, &id);
    assert_eq!(note.body, "retry #flaky it's!\n");
    assert_eq!(tag_list(&note), ["flaky"]);
    assert_eq!(note.get_str("source"), Some("cli"));
}

#[test]
fn tags_are_lower_cased_unique_and_need_a_space_before_the_hash() {
    let home = sandbox("tags");
    let id = add(
        &home,
        &["fix", "#Flaky", "#flaky", "a#b", "#v2/ui,", "# x", "#"],
    );
    assert_eq!(tag_list(&parsed(&home, &id)), ["flaky", "v2/ui"]);
}

#[test]
fn n_with_no_args_reads_all_of_a_piped_stdin() {
    let home = sandbox("stdin");
    let n = link_n(&home);
    let out = exec(&home, &n, &[], Some("line one #x\nline two\n"), |_| {});
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let note = parsed(&home, &noted_id(&text(&out.stdout)));
    assert_eq!(note.body, "line one #x\nline two\n");
    assert_eq!(tag_list(&note), ["x"]);
}

#[test]
fn empty_text_saves_nothing_and_exits_0() {
    let home = sandbox("empty");
    for stdin in [Some(""), Some("  \n\n"), None] {
        let out = exec(&home, &bin(), &["add"], stdin, |_| {});
        assert_eq!(out.status.code(), Some(0), "{stdin:?}");
        assert!(out.stdout.is_empty());
    }
    let out = easy(&home, &["add", "  ", ""]);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
    assert!(!home.join("notes").exists(), "nothing written, no dir made");
}

#[test]
fn dash_e_opens_editor_and_saves_what_it_wrote() {
    let home = sandbox("dash-e");
    let seen = home.join("seen.txt");
    let stub = editor(
        &home,
        "ed.sh",
        &format!(
            "cp \"$1\" '{}'\nprintf 'from the editor #ed\\n' > \"$1\"",
            seen.display()
        ),
    );
    let out = with_editor(&home, &["add", "-e", "seed", "text"], &stub);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        std::fs::read_to_string(&seen).unwrap().trim_end(),
        "seed text"
    );
    let note = parsed(&home, &noted_id(&text(&out.stdout)));
    assert_eq!(note.body, "from the editor #ed\n");
    assert_eq!(tag_list(&note), ["ed"]);
    // no scratch file is left in the notes dir
    let scratch = home.join("notes/.easy-note/edit");
    assert!(
        !scratch.exists() || std::fs::read_dir(&scratch).unwrap().next().is_none(),
        "scratch cleaned"
    );
}

#[test]
fn dash_e_empty_result_or_failing_editor_saves_nothing() {
    let home = sandbox("dash-e-empty");
    let blank = editor(&home, "blank.sh", ": > \"$1\"");
    let out = with_editor(&home, &["add", "-e"], &blank);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
    let boom = editor(&home, "boom.sh", "echo typed > \"$1\"; exit 3");
    let out = with_editor(&home, &["add", "-e"], &boom);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let missing = home.join("no-such-editor");
    let out = with_editor(&home, &["add", "-e"], &missing);
    assert_eq!(out.status.code(), Some(1));
    assert!(!home.join("notes/2026").exists(), "no note dir made");
    let ids = text(&easy(&home, &["ls"]).stdout);
    assert!(ids.trim().is_empty(), "no note saved: {ids}");
}

fn git_repo(home: &Path, name: &str, branch: &str) -> PathBuf {
    let dir = home.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["symbolic-ref", "HEAD", &format!("refs/heads/{branch}")],
    ] {
        let st = Command::new("git")
            .args(&args)
            .current_dir(&dir)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .status()
            .unwrap();
        assert!(st.success());
    }
    dir
}

#[test]
fn confirmation_line_has_id_repo_task() {
    let home = sandbox("line");
    let repo = git_repo(&home, "easy-ui-mcp", "fix/t120-firefox");
    let out = exec(&home, &bin(), &["add", "retry", "bug"], None, |c| {
        c.current_dir(&repo);
    });
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    let id = noted_id(&stdout);
    assert_eq!(stdout, format!("✓ noted {id} · easy-ui-mcp · T120\n"));
    let note = parsed(&home, &id);
    assert_eq!(note.get_str("repo"), Some("easy-ui-mcp"));
    assert_eq!(note.get_str("task"), Some("T120"));
}

#[test]
fn confirmation_line_outside_git_is_just_the_id() {
    let home = sandbox("line-plain");
    let stdout = ok(&home, &["add", "hello"]);
    let id = noted_id(&stdout);
    assert_eq!(stdout, format!("✓ noted {id}\n"));
}

#[test]
fn confirmation_line_names_masked_count_and_kinds_never_the_value() {
    let home = sandbox("line-masked");
    let one = ok(&home, &["add", &format!("key {AWS} now")]);
    assert!(
        one.trim_end().ends_with("· 1 secret masked: aws_key"),
        "{one}"
    );
    let two = ok(&home, &["add", &format!("key {AWS} and {GITHUB}")]);
    assert!(
        two.trim_end()
            .ends_with("· 2 secrets masked: aws_key, github_token"),
        "{two}"
    );
    for line in [&one, &two] {
        assert!(!line.contains(AWS) && !line.contains(GITHUB));
    }
    let id = noted_id(&one);
    let raw = std::fs::read_to_string(note_file(&home, &id)).unwrap();
    assert!(raw.contains("key [masked:aws_key] now") && !raw.contains(AWS));
}

#[test]
fn init_writes_config_and_later_adds_use_that_dir() {
    let home = sandbox("init");
    let chosen = home.join("my-notes");
    let out = ok(&home, &["init", "--dir", chosen.to_str().unwrap()]);
    assert!(out.contains(chosen.to_str().unwrap()), "{out}");
    assert!(chosen.is_dir());
    let config = std::fs::read_to_string(home.join(".config/easy-note/config.toml")).unwrap();
    assert!(config.contains(chosen.to_str().unwrap()), "{config}");
    let added = exec(&home, &bin(), &["add", "x"], None, |c| {
        c.env_remove("EASY_NOTE_DIR");
    });
    assert_eq!(added.status.code(), Some(0));
    let id = noted_id(&text(&added.stdout));
    assert!(chosen
        .join(&id[..4])
        .join(&id[4..6])
        .join(format!("{id}.md"))
        .is_file());
    assert_eq!(
        ok(&home, &["path"]).trim(),
        home.join("notes").to_str().unwrap(),
        "EASY_NOTE_DIR overrides the config"
    );
}

#[test]
fn path_prints_the_notes_dir() {
    let home = sandbox("path");
    assert_eq!(
        ok(&home, &["path"]).trim(),
        home.join("notes").to_str().unwrap()
    );
    let out = exec(&home, &bin(), &["path"], None, |c| {
        c.env_remove("EASY_NOTE_DIR");
    });
    assert_eq!(
        text(&out.stdout).trim(),
        home.join("easy-notes").to_str().unwrap()
    );
    assert!(!home.join("easy-notes").exists(), "path creates nothing");
}

#[test]
fn ls_lists_newest_first_with_filters_and_limit() {
    let home = sandbox("ls");
    let a = add(&home, &["alpha", "#one"]);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let b = add(&home, &["beta", "#two"]);
    let all = ok(&home, &["ls"]);
    let lines: Vec<&str> = all.lines().collect();
    assert_eq!(lines.len(), 2, "{all}");
    assert!(lines[0].contains(&b) && lines[0].contains("beta"), "{all}");
    assert!(lines[1].contains(&a) && lines[1].contains("alpha"), "{all}");
    let tagged = ok(&home, &["ls", "--tag", "one"]);
    assert!(tagged.contains(&a) && !tagged.contains(&b), "{tagged}");
    let cut = easy(&home, &["ls", "-n", "1"]);
    let one = text(&cut.stdout);
    assert!(one.contains(&b) && !one.contains(&a), "{one}");
    assert!(
        text(&cut.stderr).contains("1 of 2"),
        "says it is cut: {}",
        text(&cut.stderr)
    );
    fails(&home, &["ls", "-n", "0"], 2);
    fails(&home, &["ls", "--since", "not-a-date"], 2);
}

#[test]
fn search_matches_all_words_and_rejects_an_empty_query() {
    let home = sandbox("search");
    let hit = add(&home, &["Retry", "bug", "on", "Firefox"]);
    add(&home, &["something", "else"]);
    let out = ok(&home, &["search", "firefox", "RETRY"]);
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(out.contains(&hit));
    assert_eq!(ok(&home, &["search", "nothing-like-this"]).trim(), "");
    fails(&home, &["search", " "], 2);
}

#[test]
fn show_prints_the_note_masked_and_unknown_ids_fail() {
    let home = sandbox("show");
    let id = add(&home, &["hello", "#a"]);
    let shown = ok(&home, &["show", &id]);
    assert!(shown.contains(&id) && shown.contains("hello #a"), "{shown}");
    assert!(
        !shown.contains(note_file(&home, &id).to_str().unwrap()),
        "no path"
    );
    // a hand-edited file with a secret is masked on read; the bytes stay
    let path = note_file(&home, &id);
    let raw = std::fs::read_to_string(&path).unwrap();
    let hand = raw.replace("hello #a", &format!("hello {AWS}"));
    std::fs::write(&path, &hand).unwrap();
    let shown = ok(&home, &["show", &id]);
    assert!(
        shown.contains("[masked:aws_key]") && !shown.contains(AWS),
        "{shown}"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), hand);
    assert!(!ok(&home, &["ls"]).contains(AWS), "ls masks too");
    let err = fails(&home, &["show", "20200101-000000-abcd"], 1);
    assert!(err.contains("20200101-000000-abcd"), "{err}");
    fails(&home, &["show", "not-an-id"], 2);
}

#[test]
fn tag_adds_and_removes() {
    let home = sandbox("tag");
    let id = add(&home, &["hello", "#a"]);
    let out = ok(&home, &["tag", &id, "#B", "c/d"]);
    assert!(out.starts_with("✓ tagged "), "{out}");
    assert_eq!(tag_list(&parsed(&home, &id)), ["a", "b", "c/d"]);
    ok(&home, &["tag", &id, "--remove", "a", "zzz"]);
    assert_eq!(tag_list(&parsed(&home, &id)), ["b", "c/d"]);
    assert!(parsed(&home, &id).fields.contains_key("updated"));
    fails(&home, &["tag", &id, "bad tag!"], 2);
    fails(&home, &["tag", "20200101-000000-abcd", "x"], 1);
    fails(&home, &["tag", &id], 2);
}

#[test]
fn rm_then_restore_gives_back_the_same_bytes() {
    let home = sandbox("rm");
    let id = add(&home, &["to", "delete"]);
    let path = note_file(&home, &id);
    let before = std::fs::read(&path).unwrap();
    let out = ok(&home, &["rm", &id]);
    assert!(out.starts_with("✓ removed "), "{out}");
    assert!(!path.exists());
    assert!(home
        .join(format!("notes/.easy-note/trash/{id}.md"))
        .is_file());
    assert!(!ok(&home, &["ls"]).contains(&id));
    assert!(ok(&home, &["trash"]).contains(&id), "trash lists it");
    let out = ok(&home, &["restore", &id]);
    assert!(out.starts_with("✓ restored "), "{out}");
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(!ok(&home, &["trash"]).contains(&id));
}

#[test]
fn rm_and_restore_of_unknown_ids_exit_1_and_change_nothing() {
    let home = sandbox("rm-unknown");
    let id = add(&home, &["keep"]);
    let before = std::fs::read(note_file(&home, &id)).unwrap();
    fails(&home, &["rm", "20200101-000000-abcd"], 1);
    fails(&home, &["restore", "20200101-000000-abcd"], 1);
    fails(&home, &["rm", "nope"], 2);
    assert_eq!(std::fs::read(note_file(&home, &id)).unwrap(), before);
}

#[test]
fn trash_empty_older_than_deletes_only_old_files() {
    let home = sandbox("trash-empty");
    let id = add(&home, &["bin", "me"]);
    ok(&home, &["rm", &id]);
    let trashed = home.join(format!("notes/.easy-note/trash/{id}.md"));
    ok(&home, &["trash", "--empty", "--older-than", "30d"]);
    assert!(trashed.is_file(), "a fresh trash file stays");
    let out = ok(&home, &["trash", "--empty", "--older-than", "0d"]);
    assert!(out.contains(&id) && !trashed.exists(), "{out}");
    fails(&home, &["trash", "--empty"], 2);
    fails(&home, &["trash", "--older-than", "30d"], 2);
    fails(&home, &["trash", "--empty", "--older-than", "soon"], 2);
}

#[test]
fn edit_saves_the_changed_text_and_masks_a_typed_secret() {
    let home = sandbox("edit");
    let id = add(&home, &["first", "draft", "#a"]);
    let stub = editor(
        &home,
        "ed.sh",
        &format!("printf 'a key {AWS} and {GITHUB}\\n' >> \"$1\""),
    );
    let out = with_editor(&home, &["edit", &id], &stub);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let line = text(&out.stdout);
    assert!(
        line.starts_with(&format!("✓ edited {id}"))
            && line.contains("2 secrets masked: aws_key, github_token"),
        "{line}"
    );
    let raw = std::fs::read_to_string(note_file(&home, &id)).unwrap();
    assert!(!raw.contains(AWS) && !raw.contains(GITHUB), "{raw}");
    let note = parsed(&home, &id);
    assert_eq!(
        note.body,
        "first draft #a\na key [masked:aws_key] and [masked:github_token]\n"
    );
    assert!(note.fields.contains_key("updated"));
    let scratch = home.join("notes/.easy-note/edit");
    assert!(!scratch.exists() || std::fs::read_dir(&scratch).unwrap().next().is_none());
}

#[test]
fn edit_changes_tags_from_the_front_matter() {
    let home = sandbox("edit-tags");
    let id = add(&home, &["hello", "#a"]);
    let stub = editor(
        &home,
        "ed.sh",
        "sed -i 's/^tags: .*/tags: [\"a\", \"z\"]/' \"$1\"",
    );
    let out = with_editor(&home, &["edit", &id], &stub);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(tag_list(&parsed(&home, &id)), ["a", "z"]);
}

#[test]
fn edit_refuses_bad_front_matter_and_keeps_the_original() {
    let home = sandbox("edit-bad");
    let id = add(&home, &["hello"]);
    let before = std::fs::read(note_file(&home, &id)).unwrap();
    for body in [
        "sed -i '1d' \"$1\"",                                      // no opening ---
        "sed -i 's/^easy-note: 1/easy-note: 9/' \"$1\"",           // unknown version
        "sed -i 's/^id: .*/id: \"20200101-000000-abcd\"/' \"$1\"", // id changed
        "sed -i 's/^source: .*/source: \"mcp\"/' \"$1\"", // a field this command does not save
    ] {
        let stub = editor(&home, "ed.sh", body);
        let out = with_editor(&home, &["edit", &id], &stub);
        assert_eq!(out.status.code(), Some(1), "{body}: {}", text(&out.stdout));
        assert!(out.stdout.is_empty());
        assert!(
            text(&out.stderr).contains("unchanged"),
            "{}",
            text(&out.stderr)
        );
        assert_eq!(
            std::fs::read(note_file(&home, &id)).unwrap(),
            before,
            "{body}"
        );
    }
}

#[test]
fn edit_with_no_change_or_an_emptied_text_writes_nothing() {
    let home = sandbox("edit-same");
    let id = add(&home, &["hello"]);
    let before = std::fs::read(note_file(&home, &id)).unwrap();
    let same = editor(&home, "same.sh", ":");
    let out = with_editor(&home, &["edit", &id], &same);
    assert_eq!(out.status.code(), Some(0));
    assert!(text(&out.stderr).contains("no changes"));
    let emptied = editor(&home, "emptied.sh", "sed -i '/^hello$/d' \"$1\"");
    let out = with_editor(&home, &["edit", &id], &emptied);
    assert_eq!(out.status.code(), Some(1));
    let boom = editor(&home, "boom.sh", "echo junk >> \"$1\"; exit 1");
    let out = with_editor(&home, &["edit", &id], &boom);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(std::fs::read(note_file(&home, &id)).unwrap(), before);
    let out = with_editor(&home, &["edit", "20200101-000000-abcd"], &same);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn stats_prints_counts_and_json() {
    let home = sandbox("stats");
    let repo = git_repo(&home, "proj", "fix/t120-x");
    for word in ["one", "two"] {
        let out = exec(&home, &bin(), &["add", word, "#t"], None, |c| {
            c.current_dir(&repo);
        });
        assert_eq!(out.status.code(), Some(0));
    }
    add(&home, &["three"]);
    let out = ok(&home, &["stats"]);
    assert!(out.contains("3 notes"), "{out}");
    assert!(out.contains("proj") && out.contains("T120"), "{out}");
    let json = ok(&home, &["stats", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json).expect("json");
    assert_eq!(v["total"], 3);
    assert_eq!(v["by_repo"]["proj"], 2);
    assert_eq!(v["by_tag"]["t"], 2);
    let filtered: serde_json::Value =
        serde_json::from_str(&ok(&home, &["stats", "--json", "--repo", "proj"])).unwrap();
    assert_eq!(filtered["total"], 2);
}

#[test]
fn doctor_reports_skipped_files_secrets_by_kind_and_an_n_clash_read_only() {
    let home = sandbox("doctor");
    let id = add(&home, &["clean"]);
    let handwritten = add(&home, &["will", "be", "edited"]);
    let path = note_file(&home, &handwritten);
    let raw = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        raw.replace("will be edited", &format!("pasted {AWS}")),
    )
    .unwrap();
    let bad = home.join("notes/2026/01/20260101-000000-aaaa.md");
    std::fs::create_dir_all(bad.parent().unwrap()).unwrap();
    std::fs::write(&bad, "no front matter here").unwrap();
    // a different `n` earlier on PATH
    let bin_dir = home.join("pathdir");
    std::fs::create_dir_all(&bin_dir).unwrap();
    editor(&home, "pathdir/n", "echo other");
    let before = snapshot(&home.join("notes"));
    let out = exec(&home, &bin(), &["doctor"], None, |c| {
        c.env("PATH", format!("{}:/usr/bin:/bin", bin_dir.display()));
    });
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let report = text(&out.stdout);
    assert!(
        report.contains("20260101-000000-aaaa.md"),
        "skipped: {report}"
    );
    assert!(
        report.contains(&handwritten) && report.contains("aws_key"),
        "{report}"
    );
    assert!(!report.contains(AWS), "kinds only: {report}");
    assert!(
        !report.contains(&format!("{id}.md")),
        "clean notes are not listed: {report}"
    );
    assert!(
        report.contains(bin_dir.join("n").to_str().unwrap()),
        "n clash: {report}"
    );
    assert_eq!(snapshot(&home.join("notes")), before, "doctor is read-only");
}

#[test]
fn doctor_on_a_clean_dir_says_so_and_n_as_ours_is_no_clash() {
    let home = sandbox("doctor-clean");
    add(&home, &["clean"]);
    let n = link_n(&home);
    let bin_dir = home.clone();
    let out = exec(&home, &bin(), &["doctor"], None, |c| {
        c.env("PATH", format!("{}:/usr/bin:/bin", bin_dir.display()));
    });
    assert_eq!(out.status.code(), Some(0));
    let report = text(&out.stdout);
    assert!(report.contains("no problems"), "{report}");
    assert!(!report.contains("clash"), "{report}");
    let _ = n;
}

fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                let bytes = std::fs::read(&p).unwrap();
                out.push((p, bytes));
            }
        }
    }
    let mut all = vec![];
    walk(root, &mut all);
    all.sort();
    all
}

#[test]
fn usage_errors_exit_2_runtime_errors_exit_1_and_help_documents_both() {
    let home = sandbox("exit-codes");
    assert_eq!(easy(&home, &["frobnicate"]).status.code(), Some(2));
    assert_eq!(easy(&home, &["ls", "--bogus"]).status.code(), Some(2));
    assert_eq!(easy(&home, &["rm"]).status.code(), Some(2));
    let help = easy(&home, &["--help"]);
    assert_eq!(help.status.code(), Some(0));
    let help = text(&help.stdout);
    for words in ["Exit codes", "0", "1", "2"] {
        assert!(help.contains(words), "{words}: {help}");
    }
    for cmd in [
        "init", "add", "ls", "search", "show", "edit", "tag", "rm", "restore", "trash", "stats",
        "doctor", "path", "serve",
    ] {
        assert!(
            help.lines().any(|l| l.trim_start().starts_with(cmd)),
            "{cmd} in help: {help}"
        );
    }
    // a runtime failure: HOME unset and no EASY_NOTE_DIR -> no notes dir
    let out = exec(&home, &bin(), &["ls"], None, |c| {
        c.env_remove("HOME").env_remove("EASY_NOTE_DIR");
    });
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
}

#[test]
fn error_text_is_masked_before_it_is_printed() {
    let home = sandbox("masked-errors");
    for args in [
        vec!["show", AWS],
        vec!["rm", AWS],
        vec!["restore", AWS],
        vec!["tag", AWS, "x"],
        vec!["ls", "--since", AWS],
        vec!["trash", "--empty", "--older-than", AWS],
        vec!["ls", AWS],
        vec!["frobnicate", AWS],
    ] {
        let out = easy(&home, &args);
        assert_ne!(out.status.code(), Some(0), "{args:?}");
        let all = format!("{}{}", text(&out.stdout), text(&out.stderr));
        assert!(!all.contains(AWS), "{args:?}: {all}");
        assert!(
            all.contains("[masked:aws_key]")
                || args[0] == "ls"
                || args[0] == "frobnicate"
                || args[0] == "trash",
            "{args:?}: {all}"
        );
    }
}

#[test]
fn no_command_except_serve_reaches_the_mcp_module_or_a_runtime() {
    // Only mcp.rs names tokio or rmcp; only cli.rs names `mcp::` (once, for `serve`).
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/surfaces");
    let mut cli_sources = vec![];
    for entry in walk_rs(&dir) {
        let src = std::fs::read_to_string(&entry).unwrap();
        if entry.file_name().unwrap() == "mcp.rs" {
            continue;
        }
        assert!(
            !src.contains("tokio") && !src.contains("rmcp"),
            "{}",
            entry.display()
        );
        cli_sources.push((entry, src));
    }
    let calls: usize = cli_sources
        .iter()
        .map(|(_, s)| s.matches("mcp::").count())
        .sum();
    assert_eq!(calls, 1, "one call into the mcp module: `serve`");
}

fn walk_rs(dir: &Path) -> Vec<PathBuf> {
    let mut out = vec![];
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(walk_rs(&p));
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
    out
}

#[test]
fn serve_answers_initialize_then_exits_on_eof() {
    let home = sandbox("serve");
    let mut child = Command::new(bin())
        .arg("serve")
        .env("HOME", &home)
        .env("EASY_NOTE_DIR", home.join("notes"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn serve");
    let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"cli-test","version":"0"}}}"#;
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "{request}").unwrap();
    drop(stdin); // EOF
    let out = child.wait_with_output().expect("wait");
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    let first = text(&out.stdout).lines().next().unwrap_or("").to_owned();
    let reply: serde_json::Value = serde_json::from_str(&first).expect("one JSON-RPC line");
    assert_eq!(reply["id"], 1);
    assert!(reply["result"].is_object(), "reply: {first}");
    assert_eq!(reply["result"]["serverInfo"]["name"], "easy-note");
}
