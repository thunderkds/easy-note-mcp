"""HUN-254 context capture: repo, remote, branch, task from read-only git, never a path or a token.

Real temp git repos; a fake slow `git` on PATH for the timeout case. Temp HOME and EASY_NOTE_DIR only.
"""

from __future__ import annotations

import os
import stat
import subprocess
import time
from pathlib import Path

import pytest

from easy_note.core import context, store
from easy_note.core import format as fmt

TOKEN_URL = "https://user:tok@github.com/o/r.git"


@pytest.fixture
def env(tmp_path, monkeypatch):
    home = tmp_path / "home"
    home.mkdir()
    monkeypatch.setenv("HOME", str(home))
    monkeypatch.setenv("EASY_NOTE_DIR", str(tmp_path / "notes"))
    monkeypatch.setenv("GIT_CONFIG_GLOBAL", os.devnull)
    monkeypatch.setenv("GIT_CONFIG_SYSTEM", os.devnull)
    for key in ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE"):
        monkeypatch.delenv(key, raising=False)
    monkeypatch.chdir(tmp_path)  # a folder that is not in any repo
    return tmp_path


def git(cwd: Path, *args: str) -> None:
    subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t.invalid", *args],
                   cwd=cwd, check=True, capture_output=True)


def make_repo(root: Path, name: str = "proj", *, remote: str | None = None, branch: str = "main") -> Path:
    repo = root / name
    repo.mkdir()
    git(repo, "init", "-q", "-b", branch)
    git(repo, "commit", "-q", "--allow-empty", "-m", "first")
    if remote:
        git(repo, "remote", "add", "origin", remote)
    return repo


def written_files(env: Path) -> list[Path]:
    return [p for p in (env / "notes").rglob("*") if p.is_file()]


def add_note(**capture_args) -> Path:
    added = store.add("a note", source="cli", fields=context.capture(**capture_args))
    return added.path


def front_matter(path: Path) -> dict:
    return fmt.parse(path.read_text(encoding="utf-8")).fields


# AC 1


def test_main_repo_uses_folder_name(env):
    repo = make_repo(env, "easy-ui-mcp", remote="git@github.com:thunderkds/easy-ui-mcp.git")
    assert context.capture(cwd=repo) == {
        "repo": "easy-ui-mcp", "remote": "thunderkds/easy-ui-mcp", "branch": "main"}


def test_subfolder_of_repo_gives_the_repo_name(env):
    repo = make_repo(env)
    (repo / "sub" / "deep").mkdir(parents=True)
    assert context.capture(cwd=repo / "sub" / "deep")["repo"] == "proj"


def test_linked_worktree_uses_main_folder_name_and_branch_task(env):
    repo = make_repo(env, "main-repo")
    wt = env / "wt-t148"
    git(repo, "worktree", "add", "-q", str(wt), "-b", "fix/t148-x")
    got = context.capture(cwd=wt)
    assert got["repo"] == "main-repo"
    assert got["task"] == "T148"
    assert got["branch"] == "fix/t148-x"


def test_bare_repo_drops_the_dot_git_suffix(env):
    bare = env / "central.git"
    git(env, "init", "-q", "--bare", str(bare))
    assert context.capture(cwd=bare)["repo"] == "central"


# AC 2


def test_no_git_gives_no_fields(env):
    plain = env / "plain"
    plain.mkdir()
    assert context.capture(cwd=plain) == {}


def test_missing_folder_gives_no_fields_and_add_succeeds(env):
    fields = context.capture(cwd=env / "nope")
    assert fields == {}
    assert store.add("a note", source="cli", fields=fields).path.is_file()


def test_no_git_program_gives_no_fields(env, monkeypatch):
    repo = make_repo(env)
    empty_bin = env / "empty-bin"
    empty_bin.mkdir()
    monkeypatch.setenv("PATH", str(empty_bin))
    assert context.capture(cwd=repo) == {}


def test_no_git_keeps_explicit_task(env):
    assert context.capture(task=" T9 ", cwd=env) == {"task": "T9"}


# AC 3


def test_detached_head_has_repo_but_no_branch_or_task(env):
    repo = make_repo(env, branch="fix/t120-x")
    git(repo, "checkout", "-q", "--detach")
    assert context.capture(cwd=repo) == {"repo": "proj"}


# AC 4


@pytest.mark.parametrize("url", [
    TOKEN_URL,
    "https://github.com/o/r",
    "git@github.com:o/r.git",
    "ssh://git@host/o/r",
    "ssh://git@host:2222/o/r.git",
    "ssh://git@host/group/sub/o/r.git",
    "https://github.com/o/r/",
])
def test_remote_slug(url):
    assert context.remote_slug(url) == "o/r"


@pytest.mark.parametrize("url", [
    "/srv/git/o/r.git", "../r", "file:///srv/git/o/r.git", "https://github.com/r", "r", "", "https://[bad/o/r",
])
def test_remote_that_is_a_path_or_has_no_owner_gives_none(url):
    assert context.remote_slug(url) is None


def test_token_in_remote_is_in_no_written_file(env):
    repo = make_repo(env, remote=TOKEN_URL)
    assert context.capture(cwd=repo)["remote"] == "o/r"
    path = add_note(cwd=repo)
    assert front_matter(path)["remote"] == "o/r"
    for file in written_files(env):
        data = file.read_bytes()
        assert b"tok" not in data and b"user" not in data


def test_repo_without_origin_has_no_remote(env):
    assert "remote" not in context.capture(cwd=make_repo(env))


# AC 5


@pytest.mark.parametrize("branch, task", [
    ("fix/t120-x", "T120"),
    ("fix/t120", "T120"),
    ("feat/HUN-221-y", "HUN-221"),
    ("T1234-big", "T1234"),
    ("main", None),
    ("fix/art1234", None),
    ("fix/t120x", None),
    ("fix/t12", None),
    ("fix/t12345", None),
    ("feat/hun-221", None),
    ("fix/t120-and-t130", "T120"),
])
def test_task_from_branch(branch, task):
    assert context.task_from_branch(branch) == task


def test_branch_task_end_to_end(env):
    repo = make_repo(env, branch="feat/HUN-221-y")
    assert context.capture(cwd=repo)["task"] == "HUN-221"
    git(repo, "checkout", "-q", "-b", "main")
    got = context.capture(cwd=repo)
    assert got["branch"] == "main" and "task" not in got


def test_explicit_task_wins(env):
    repo = make_repo(env, branch="fix/t120-x")
    assert context.capture(cwd=repo, task="T999")["task"] == "T999"
    assert context.capture(cwd=repo, task="  ")["task"] == "T120"


# AC 6: explicit cwd, then MCP roots, then process cwd (docs/experiments/T001-client-launch.md)


def test_cwd_order(env, monkeypatch):
    explicit = make_repo(env, "explicit")
    rooted = make_repo(env, "rooted")
    process = make_repo(env, "process")
    monkeypatch.chdir(process)
    root_uri = rooted.as_uri()

    assert context.capture(cwd=explicit, roots=[root_uri])["repo"] == "explicit"
    assert context.capture(roots=[root_uri])["repo"] == "rooted"
    assert context.capture(roots=[])["repo"] == "process"  # Codex answers an empty list
    assert context.capture(roots=None)["repo"] == "process"
    assert context.capture()["repo"] == "process"


def test_first_root_is_used_and_plain_folder_roots_work(env):
    one, two = make_repo(env, "one"), make_repo(env, "two")
    assert context.capture(roots=[one.as_uri(), two.as_uri()])["repo"] == "one"
    assert context.capture(roots=[str(two)])["repo"] == "two"
    assert context.capture(roots=["https://example.com/x", two.as_uri()])["repo"] == "two"


def test_root_in_a_folder_with_spaces(env):
    repo = make_repo(env, "my proj")
    assert context.capture(roots=[repo.as_uri()])["repo"] == "my proj"


def test_a_source_that_is_not_a_repo_does_not_fall_through_to_a_guess(env, monkeypatch):
    process = make_repo(env, "process")
    monkeypatch.chdir(process)
    plain = env / "plain"
    plain.mkdir()
    assert context.capture(roots=[plain.as_uri()]) == {}
    assert context.capture(cwd=plain) == {}


# AC 7


def test_slow_git_drops_the_fields_and_add_still_writes(env, monkeypatch):
    repo = make_repo(env, remote=TOKEN_URL, branch="fix/t120-x")
    fake_bin = env / "fake-bin"
    fake_bin.mkdir()
    fake = fake_bin / "git"
    fake.write_text("#!/bin/sh\nexec sleep 3\n")
    fake.chmod(fake.stat().st_mode | stat.S_IEXEC)
    monkeypatch.setenv("PATH", f"{fake_bin}{os.pathsep}{os.environ['PATH']}")

    start = time.monotonic()
    fields = context.capture(cwd=repo)
    assert time.monotonic() - start < 2.9  # one 2 s wait, not one per git call
    assert fields == {}

    path = add_note(cwd=repo)
    meta = front_matter(path)
    assert not {"repo", "remote", "branch", "task"} & meta.keys()
    assert path.is_file()


def test_slow_git_keeps_explicit_task(env, monkeypatch):
    def timeout(*args, **kwargs):
        raise subprocess.TimeoutExpired(args[0], 2)

    monkeypatch.setattr(context.subprocess, "run", timeout)
    assert context.capture(cwd=env, task="T5") == {"task": "T5"}


# AC 8


def test_only_read_only_git_subcommands_run(env, monkeypatch):
    repo = make_repo(env, remote=TOKEN_URL)
    calls = []
    real_run = subprocess.run

    def spy(argv, **kwargs):
        calls.append(list(argv))
        assert kwargs.get("shell") is not True
        assert kwargs["timeout"] == 2
        assert kwargs["env"]["GIT_TERMINAL_PROMPT"] == "0"
        return real_run(argv, **kwargs)

    monkeypatch.setattr(context.subprocess, "run", spy)
    context.capture(cwd=repo)
    allowed = (["rev-parse"], ["branch", "--show-current"], ["remote", "get-url", "origin"])
    assert calls
    for argv in calls:
        assert argv[0] == "git"
        assert argv[1] == "rev-parse" or argv[1:] in allowed, argv
    assert {a[1] for a in calls} == {"rev-parse", "branch", "remote"}


def test_git_dir_from_the_environment_does_not_redirect_git(env, monkeypatch):
    other = make_repo(env, "other")
    here = make_repo(env, "here")
    monkeypatch.setenv("GIT_DIR", str(other / ".git"))
    assert context.capture(cwd=here)["repo"] == "here"


# AC 9


def test_no_absolute_path_in_any_written_file(env):
    repo = make_repo(env, remote="git@github.com:o/r.git", branch="fix/t120-x")
    (repo / "sub").mkdir()
    wt = env / "wt-t148"
    git(repo, "worktree", "add", "-q", str(wt), "-b", "fix/t148-x")
    add_note(cwd=repo / "sub")
    add_note(cwd=wt)
    add_note(roots=[repo.as_uri()])
    files = written_files(env)
    assert len(files) == 3
    for file in files:
        data = file.read_text(encoding="utf-8")
        assert str(env) not in data
        assert str(env.resolve()) not in data
        assert "/home/" not in data and "file://" not in data
