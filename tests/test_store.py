"""HUN-253 store: dir resolution, IDs and clashes, never-overwrite, atomic writes, trash, skipped files, update."""

from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
import sys
import textwrap
import time
from datetime import timedelta
from pathlib import Path

import pytest

from easy_note.core import config, store
from easy_note.core import format as fmt

FIXTURES = Path(__file__).parent / "fixtures" / "notes"
FROZEN_NS = 1_794_649_633_123_456_789  # 2026-11-14T16:47:13.123456789+07:00
HOST = "test-host"
ID_RE = r"^\d{8}-\d{6}-[0-9a-f]{4}$"


@pytest.fixture
def env(tmp_path, monkeypatch):
    """Temp HOME + EASY_NOTE_DIR (NFR-003), local zone UTC+7, frozen clock, fixed hostname."""
    home = tmp_path / "home"
    home.mkdir()
    monkeypatch.setenv("HOME", str(home))
    monkeypatch.setenv("EASY_NOTE_DIR", str(tmp_path / "notes"))
    monkeypatch.setenv("TZ", "Asia/Ho_Chi_Minh")
    time.tzset()
    monkeypatch.setattr(store, "_now_ns", lambda: FROZEN_NS)
    monkeypatch.setattr(store, "_hostname", lambda: HOST)
    yield tmp_path / "notes"
    monkeypatch.undo()
    time.tzset()


def _md_files(root: Path) -> list[Path]:
    return sorted(p for p in root.rglob("*.md") if ".easy-note" not in p.parts)


def _snapshot(root: Path) -> dict[str, bytes]:
    return {str(p.relative_to(root)): p.read_bytes() for p in sorted(root.rglob("*")) if p.is_file()}


# AC1 — a note written by the store round-trips byte for byte


def test_added_note_round_trips_byte_identical(env):
    added = store.add("retry bug only on Firefox\n\nsecond line", source="mcp", tags=["flaky"],
                      fields={"repo": "easy-ui-mcp", "branch": "no", "task": "T120", "client": None})
    raw = added.path.read_bytes()
    assert fmt.render(fmt.parse(raw.decode("utf-8"))).encode("utf-8") == raw
    note = store.read(added.id)
    assert note.fields["branch"] == "no" and "client" not in note.fields
    assert note.body == "retry bug only on Firefox\n\nsecond line\n"


# AC2 — ID format and clash suffix


def test_id_is_local_time_plus_sha256_of_ns_hostname_text(env):
    added = store.add("x", source="cli")
    digest = hashlib.sha256(f"{FROZEN_NS}\n{HOST}\nx".encode()).hexdigest()[:4]
    assert added.id == f"20261114-164713-{digest}"
    assert __import__("re").match(ID_RE, added.id)
    assert added.path == env / "2026" / "11" / f"{added.id}.md"
    assert store.read(added.id).fields["created"] == "2026-11-14T16:47:13+07:00"


def test_two_writes_same_second_same_text_get_clash_suffix(env):
    first = store.add("x", source="cli")
    second = store.add("x", source="cli")
    assert second.id == f"{first.id}-2"
    assert store.add("x", source="cli").id == f"{first.id}-3"
    assert sorted(p.stem for p in _md_files(env)) == [first.id, f"{first.id}-2", f"{first.id}-3"]
    assert store.read(second.id).id == second.id  # front matter id equals the file stem


def test_month_folder_uses_local_date_not_utc(env, monkeypatch):
    # 2026-11-30T20:00:00Z is already 2026-12-01 03:00 in UTC+7
    monkeypatch.setattr(store, "_now_ns", lambda: 1_796_068_800 * 10**9)
    added = store.add("x", source="cli")
    assert added.path.parent == env / "2026" / "12"
    assert added.id.startswith("20261201-030000-")


# AC3 — a write never replaces an existing file


@pytest.mark.parametrize("hard_links", [True, False])
def test_write_never_replaces_existing_file(env, monkeypatch, hard_links):
    digest = hashlib.sha256(f"{FROZEN_NS}\n{HOST}\nx".encode()).hexdigest()[:4]
    target = env / "2026" / "11" / f"20261114-164713-{digest}.md"
    target.parent.mkdir(parents=True)
    target.write_bytes(b"synced from the other machine\n")
    if not hard_links:
        def no_link(src, dst):
            raise PermissionError(1, "Operation not permitted")
        monkeypatch.setattr(os, "link", no_link)
    added = store.add("x", source="cli")
    assert added.path != target and added.id.endswith("-2")
    assert target.read_bytes() == b"synced from the other machine\n"
    assert added.path.read_bytes().startswith(b"---\neasy-note: 1\n")
    assert not list(target.parent.glob(".*"))  # temp file removed


# AC4 — temp file in the same dir + rename; a killed write leaves no partial .md


def test_killed_write_leaves_no_md_file(env, tmp_path):
    """A real kill: the child process dies inside os.link, so no `finally` runs."""
    script = textwrap.dedent("""
        import os
        from easy_note.core import store
        os.link = lambda src, dst: os._exit(9)
        store.add("half written", source="cli")
    """)
    done = subprocess.run([sys.executable, "-c", script], env=dict(os.environ), timeout=30, check=False)
    assert done.returncode == 9
    assert _md_files(env) == []


def test_failing_replace_on_no_link_filesystem_leaves_no_md(env, monkeypatch):
    def no_link(src, dst):
        raise PermissionError(1, "Operation not permitted")

    def broken_replace(src, dst):
        raise OSError(5, "Input/output error")

    monkeypatch.setattr(os, "link", no_link)
    monkeypatch.setattr(os, "replace", broken_replace)
    with pytest.raises(OSError, match="Input/output error"):
        store.add("x", source="cli")
    assert _md_files(env) == []
    assert list((env / "2026" / "11").iterdir()) == []  # temp file removed too


def test_failing_replace_during_update_keeps_old_bytes(env, monkeypatch):
    added = store.add("x", source="cli")
    before = _snapshot(env)

    def broken_replace(src, dst):
        raise OSError(5, "Input/output error")

    monkeypatch.setattr(os, "replace", broken_replace)
    with pytest.raises(OSError):
        store.update(added.id, text="new text")
    assert _snapshot(env) == before


# AC5 — rm → trash, restore → back with identical bytes, empty_trash by age


def test_rm_moves_to_trash_and_restore_brings_back_same_bytes(env):
    added = store.add("keep me", source="cli")
    original = added.path.read_bytes()
    trashed = store.rm(added.id)
    assert trashed == env / ".easy-note" / "trash" / f"{added.id}.md"
    assert not added.path.exists() and trashed.read_bytes() == original
    with pytest.raises(store.NoteNotFound):
        store.read(added.id)
    assert store.restore(added.id) == added.path
    assert added.path.read_bytes() == original and not trashed.exists()


def test_rm_and_restore_unknown_id_raise_not_found(env):
    with pytest.raises(store.NoteNotFound):
        store.rm("20261114-164713-0000")
    with pytest.raises(store.NoteNotFound):
        store.restore("20261114-164713-0000")


def test_restore_never_replaces_a_note_at_the_target(env):
    added = store.add("x", source="cli")
    trashed = store.rm(added.id)
    added.path.write_bytes(b"newer copy from sync\n")
    with pytest.raises(FileExistsError):
        store.restore(added.id)
    assert added.path.read_bytes() == b"newer copy from sync\n" and trashed.exists()


def test_empty_trash_deletes_only_files_older_than_cutoff(env):
    old = store.rm(store.add("old", source="cli").id)
    new = store.rm(store.add("new", source="cli").id)
    forty_days_ago = time.time() - 40 * 86400
    os.utime(old, (forty_days_ago, forty_days_ago))
    assert store.empty_trash(timedelta(days=30)) == [old]
    assert not old.exists() and new.exists()


def test_rm_stamps_trash_time_so_an_old_note_is_not_emptied_at_once(env):
    added = store.add("written long ago", source="cli")
    long_ago = time.time() - 400 * 86400
    os.utime(added.path, (long_ago, long_ago))
    trashed = store.rm(added.id)
    assert store.empty_trash(timedelta(days=30)) == []
    assert trashed.exists()


# AC6 — bad front matter and `easy-note: 2` are skipped with their path, bytes unchanged


def test_bad_files_are_skipped_with_path_and_left_unchanged(env):
    shutil.copytree(FIXTURES, env)
    (env / "2026" / "11" / "20261114-170500-0bad.md").write_bytes(b"---\xff\xfe not utf-8\n")
    (env / "2026" / "11" / "20261114-170600-0f0f.md").write_text(
        (FIXTURES / "2026" / "11" / "20261114-164713-a1b2.md").read_text())  # id differs from file name
    (env / "README.md").write_text("not a note, outside YYYY/MM\n")
    before = _snapshot(env)

    found = list(store.iter_notes())

    skipped = {item.path.name: item.reason for item in found if isinstance(item, store.Skipped)}
    assert skipped == {
        "20261114-170000-bad1.md": "front matter is not valid YAML (line 5)",
        "20261114-170100-0002.md": "unsupported version 2",
        "20261114-170200-0003.md": "no front matter",
        "20261114-170500-0bad.md": "not UTF-8",
        "20261114-170600-0f0f.md": "id does not match file name",
    }
    assert all(item.path.is_absolute() for item in found)
    notes = [item.id for item in found if isinstance(item, fmt.Note)]
    assert notes == ["20261114-164713-a1b2", "20261114-170300-c71f", "20261114-170400-0b5d"]
    assert _snapshot(env) == before


# AC7 — update keeps unknown keys and sets `updated`


def test_update_keeps_unknown_keys_and_sets_updated(env):
    shutil.copytree(FIXTURES, env)
    note = store.update("20261114-170400-0b5d", tags=["flaky", "obsidian"])
    out = note.path.read_text()
    assert "aliases:\n  - firefox retry\n" in out
    assert "date: 2024-01-01 # Obsidian date property\n" in out
    assert "cssclasses: [wide]\n" in out
    assert 'updated: "2026-11-14T16:47:13+07:00"\n' in out
    assert 'tags: ["flaky", "obsidian"]\n' in out
    assert out.endswith("---\nHand-edited in Obsidian\n")


def test_update_text_replaces_body_and_checks_size(env):
    added = store.add("first", source="cli")
    assert store.update(added.id, text="second").body == "second\n"
    assert store.read(added.id).body == "second\n"
    with pytest.raises(fmt.TextTooLong):
        store.update(added.id, text="x" * 20_001)


def test_read_refuses_a_value_that_is_not_an_id(env):
    for bad in ("../../etc/passwd", "20261114-164713-a1b2/../x", "20261114-164713-A1B2", ""):
        with pytest.raises(ValueError, match="not a note id"):
            store.read(bad)


# AC8 — zero setup: ~/easy-notes is created by the first add; EASY_NOTE_DIR beats the config


def test_first_add_creates_default_dir_once(env, monkeypatch, tmp_path):
    monkeypatch.delenv("EASY_NOTE_DIR")
    first = store.add("x", source="cli")
    assert first.created_dir is True
    assert first.path.is_relative_to(tmp_path / "home" / "easy-notes")
    assert store.add("y", source="cli").created_dir is False


def test_env_dir_beats_config_and_config_beats_default(env, monkeypatch, tmp_path):
    config.save_dir(tmp_path / "from config")
    assert store.add("x", source="cli").path.is_relative_to(env)
    monkeypatch.delenv("EASY_NOTE_DIR")
    assert store.add("x", source="cli").path.is_relative_to(tmp_path / "from config")


def test_config_round_trips_awkward_paths(env, monkeypatch, tmp_path):
    monkeypatch.delenv("EASY_NOTE_DIR")
    odd = tmp_path / 'quo"te \\ back é 😀 tab\t del\x7f'
    config.save_dir(odd)
    assert config.notes_dir() == odd
    assert config.config_path() == tmp_path / "home" / ".config" / "easy-note" / "config.toml"


def test_broken_config_raises_clear_error(env, monkeypatch):
    monkeypatch.delenv("EASY_NOTE_DIR")
    config.config_path().parent.mkdir(parents=True)
    config.config_path().write_text("dir = [unclosed\n")
    with pytest.raises(config.ConfigError, match="config.toml"):
        config.notes_dir()


# AC9 (store side) — the full text is kept; only over-size text is refused, before anything is written


def test_long_first_line_is_kept_whole_in_the_file(env):
    added = store.add("t" * 300, source="cli")
    note = store.read(added.id)
    assert note.body == "t" * 300 + "\n" and note.title == "t" * 120


def test_oversize_text_is_refused_before_any_write(env):
    with pytest.raises(fmt.TextTooLong, match="limit is 20,000"):
        store.add("x" * 20_001, source="cli")
    assert not env.exists()


@pytest.mark.parametrize(
    ("kwargs", "message"),
    [
        ({"source": "fax"}, "unknown source"),
        ({"source": "cli", "tags": "flaky"}, "tags"),
        ({"source": "cli", "fields": {"id": "x"}}, "not a context field"),
        ({"source": "cli", "fields": {"repo": 3}}, "must be a string"),
    ],
)
def test_add_rejects_bad_arguments(env, kwargs, message):
    with pytest.raises((TypeError, ValueError), match=message):
        store.add("x", **kwargs)


def test_tags_may_be_any_iterable_and_update_checks_them(env):
    added = store.add("x", source="cli", tags=(t for t in ["a", "b"]))
    assert store.read(added.id).fields["tags"] == ["a", "b"]
    with pytest.raises(TypeError, match="tags"):
        store.update(added.id, tags=[1])


def test_unreadable_file_is_skipped_not_fatal(env):
    added = store.add("x", source="cli")
    added.path.chmod(0)
    try:
        found = list(store.iter_notes())
    finally:
        added.path.chmod(0o600)
    assert found == [store.Skipped(added.path, "cannot read file")]
