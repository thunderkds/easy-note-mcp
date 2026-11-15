"""Context of a new note: repo, remote, branch, task (FR-004, FR-005).

`capture()` returns only the fields it can state with certainty, for `store.add(fields=...)`.
A field that cannot be read is left out, never guessed. No path is ever returned (FR-004).

Git is read-only (NFR-001): only `rev-parse`, `branch --show-current` and `remote get-url origin`,
each with a 2 s timeout, no shell, no prompt. A timeout or a missing `git` drops every git field
at once, so a slow repo costs one wait and a note is still written.

Working folder, as decided in docs/experiments/T001-client-launch.md: the explicit `cwd`, else
the first MCP root, else the process cwd. The first source that is present is the one used;
a source that is not a git repo gives no fields rather than falling through to a guess.
"""

from __future__ import annotations

import os
import re
import subprocess
from collections.abc import Sequence
from pathlib import PurePosixPath
from urllib.parse import unquote, urlsplit

GIT_TIMEOUT = 2  # seconds, per git call (NFR-001)

# Tried in this order; the first match in the branch name wins (FR-005).
# A hit may not touch a letter or digit on either side, so `art1234` and `T1234x` do not match.
_TASK_PATTERNS = (
    (re.compile(r"(?<![A-Za-z0-9])([Tt]\d{3,4})(?![A-Za-z0-9])"), str.upper),
    (re.compile(r"(?<![A-Za-z0-9])([A-Z]{2,}-\d{1,6})(?![A-Za-z0-9])"), str),
)
_SCP_REMOTE = re.compile(r"^(?:[^@/\s]+@)?[^:/\s]+:(?!//)(?P<path>.+)$")
_BAD_SEGMENT = re.compile(r"[\s@:?#]")
# Git variables that would point git at another repo than the folder we ask about.
_GIT_ENV_DROP = ("GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_INDEX_FILE", "GIT_OBJECT_DIRECTORY")


class _GitUnavailable(Exception):
    """git is missing or exceeded the timeout: all git fields are dropped."""


def capture(*, task: str | None = None, cwd: str | os.PathLike | None = None,
            roots: Sequence[str] | None = None) -> dict[str, str]:
    """Context fields for a new note; keys are a subset of `store.CONTEXT_KEYS`.

    `roots` are the MCP `roots/list` answer (`file://` URIs or plain folders); empty means absent.
    An explicit `task` always wins over the branch.
    """
    fields: dict[str, str] = {}
    explicit = task.strip() if isinstance(task, str) else None
    if explicit:
        fields["task"] = explicit

    folder = _working_folder(cwd, roots)
    if folder is None:
        return fields
    try:
        git_fields = _git_fields(folder)
    except _GitUnavailable:
        return fields
    if "task" in fields:
        git_fields.pop("task", None)
    fields.update(git_fields)
    return fields


def task_from_branch(branch: str) -> str | None:
    for pattern, normalize in _TASK_PATTERNS:
        match = pattern.search(branch)
        if match:
            return normalize(match.group(1))
    return None


def remote_slug(url: str) -> str | None:
    """`owner/name` of a remote URL with userinfo, host, port, query and `.git` dropped; else None.

    A local path or `file://` remote gives None, because it would be a filesystem path.
    """
    url = url.strip()
    if "://" in url:
        try:
            parts = urlsplit(url)
        except ValueError:
            return None
        if parts.scheme.lower() == "file" or not parts.netloc:
            return None
        path = parts.path
    else:
        match = _SCP_REMOTE.match(url)
        if not match:
            return None
        path = match.group("path")
    segments = [unquote(s) for s in PurePosixPath(path).parts if s != "/"]
    if segments:
        segments[-1] = segments[-1].removesuffix(".git")
    if len(segments) < 2:
        return None
    owner, name = segments[-2:]
    for segment in (owner, name):
        if not segment or segment in (".", "..") or _BAD_SEGMENT.search(segment):
            return None
    return f"{owner}/{name}"


def _working_folder(cwd, roots) -> str | None:
    if cwd:
        return os.path.abspath(os.fspath(cwd))
    for root in roots or ():
        folder = _root_to_folder(root)
        if folder:
            return folder
    try:
        return os.getcwd()
    except OSError:
        return None


def _root_to_folder(root: str) -> str | None:
    if not isinstance(root, str) or not root:
        return None
    if "://" not in root:
        return os.path.abspath(root)
    parts = urlsplit(root)
    if parts.scheme.lower() != "file" or parts.netloc not in ("", "localhost"):
        return None
    return unquote(parts.path) or None


def _git_fields(folder: str) -> dict[str, str]:
    top = _git(folder, "rev-parse", "--git-common-dir", "--is-bare-repository")
    if top is None:
        return {}
    lines = top.splitlines()
    if len(lines) != 2:
        return {}
    common_dir = os.path.abspath(os.path.join(folder, lines[0]))
    name = os.path.basename(common_dir)
    if name == ".git" and lines[1] != "true":
        name = os.path.basename(os.path.dirname(common_dir))
    else:
        name = name.removesuffix(".git")
    fields: dict[str, str] = {}
    if name:
        fields["repo"] = name

    branch = _git(folder, "branch", "--show-current")
    if branch:  # empty on a detached HEAD
        fields["branch"] = branch
        task = task_from_branch(branch)
        if task:
            fields["task"] = task

    remote = _git(folder, "remote", "get-url", "origin")
    slug = remote_slug(remote) if remote else None
    if slug:
        fields["remote"] = slug
    return fields


def _git(folder: str, *args: str) -> str | None:
    """Stdout of one read-only git call; None when git says no (not a repo, no origin)."""
    env = {k: v for k, v in os.environ.items() if k not in _GIT_ENV_DROP}
    env["GIT_TERMINAL_PROMPT"] = "0"
    try:
        done = subprocess.run(["git", *args], cwd=folder, env=env, capture_output=True,
                              text=True, timeout=GIT_TIMEOUT, stdin=subprocess.DEVNULL)
    except subprocess.TimeoutExpired:
        raise _GitUnavailable from None
    except (OSError, ValueError):
        # `git` or the folder is missing, or the output is not text: no context.
        raise _GitUnavailable from None
    if done.returncode != 0:
        return None
    return done.stdout.strip()
