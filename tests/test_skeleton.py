"""HUN-252 skeleton: version flag, `n` = `add` dispatch, CLI never imports MCP, `serve` exits on EOF."""

from __future__ import annotations

import json
import os
import subprocess
import sys

import pytest

import easy_note
from easy_note.surfaces import cli


@pytest.fixture
def isolated_env(tmp_path, monkeypatch):
    """Temp HOME and EASY_NOTE_DIR: a test never touches the person's real folders (NFR-003)."""
    home = tmp_path / "home"
    home.mkdir()
    monkeypatch.setenv("HOME", str(home))
    monkeypatch.setenv("EASY_NOTE_DIR", str(tmp_path / "notes"))
    return dict(os.environ)


def test_version_flag_prints_package_version(capsys):
    with pytest.raises(SystemExit) as exc:
        cli.main(["--version"])
    assert exc.value.code == 0
    assert capsys.readouterr().out.strip() == easy_note.__version__ == "0.1.0"


def test_n_version_matches_easy_note_version(capsys):
    with pytest.raises(SystemExit) as exc:
        cli.main_n(["--version"])
    assert exc.value.code == 0
    assert capsys.readouterr().out.strip() == easy_note.__version__


def test_n_dispatches_to_add_with_all_arguments(monkeypatch):
    seen = []
    monkeypatch.setattr(cli, "main", lambda argv=None: seen.append(list(argv)) or 0)
    assert cli.main_n(["retry", "bug", "on", "firefox"]) == 0
    assert seen == [["add", "retry", "bug", "on", "firefox"]]


def test_add_parses_text_words():
    args = cli.build_parser().parse_args(["add", "retry", "bug"])
    assert args.command == "add"
    assert args.text == ["retry", "bug"]


def test_cli_import_does_not_load_mcp(isolated_env):
    code = "import easy_note.surfaces.cli, sys; print('mcp' in sys.modules)"
    out = subprocess.run(
        [sys.executable, "-c", code],
        capture_output=True, text=True, check=True, env=isolated_env, timeout=30,
    )
    assert out.stdout.strip() == "False"


@pytest.mark.parametrize("stdin_text", ["", "\n"])
def test_serve_exits_cleanly_on_eof(isolated_env, stdin_text):
    done = subprocess.run(
        [sys.executable, "-m", "easy_note.surfaces.cli", "serve"],
        input=stdin_text, capture_output=True, text=True, check=False, env=isolated_env, timeout=30,
    )
    assert done.returncode == 0
    assert "Traceback" not in done.stderr
    # stdout is the protocol channel: only JSON-RPC lines may appear on it
    for line in done.stdout.splitlines():
        assert json.loads(line)["jsonrpc"] == "2.0"
    if not stdin_text:
        assert done.stdout == ""
