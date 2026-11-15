"""T001: the recorded Claude Code UserPromptSubmit events keep their shape and stay scrubbed."""
import json
import re
from pathlib import Path

import pytest

FIXTURES = Path(__file__).parent / "fixtures" / "claude_hook"
REQUIRED = ("prompt", "cwd", "session_id")
FILES = sorted(FIXTURES.glob("*.json"))


def test_fixtures_exist():
    assert len(FILES) >= 5, f"expected recorded events in {FIXTURES}"


@pytest.mark.parametrize("path", FILES, ids=lambda p: p.name)
def test_event_has_required_keys(path):
    event = json.loads(path.read_text(encoding="utf-8"))
    for key in REQUIRED:
        assert isinstance(event.get(key), str) and event[key], f"{path.name}: missing {key}"
    assert event["hook_event_name"] == "UserPromptSubmit"


@pytest.mark.parametrize("path", FILES, ids=lambda p: p.name)
def test_event_is_scrubbed(path):
    text = path.read_text(encoding="utf-8")
    assert "/home/" not in text, f"{path.name}: holds a home path"
    assert not re.search(r"token|secret|password|api[_-]?key", text, re.IGNORECASE)


def test_note_prefix_events_reach_the_hook_raw():
    """The hook sees `/note ...` unchanged (finding of docs/experiments/T001-client-launch.md)."""
    raw = [p for p in FILES if p.name.startswith("note_prefix")]
    assert raw, "no recorded /note event"
    for path in raw:
        assert json.loads(path.read_text(encoding="utf-8"))["prompt"].startswith("/note ")


def test_worktree_event_cwd_is_the_worktree_folder():
    event = json.loads((FIXTURES / "note_prefix_linked_worktree.json").read_text(encoding="utf-8"))
    assert event["cwd"].endswith("proj-wt")
