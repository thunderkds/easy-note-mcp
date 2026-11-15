"""The notes dir is the only store (DDR 0002): new IDs, never-overwrite atomic writes, trash, skipped files.

A new note is written to a temp file in its month folder, fsynced, then hard-linked to its final
name: `os.link` fails if the name exists, so a write can never replace a file (NFR-005). On a
clash the ID gets `-2`, `-3`, ... A kill before the link leaves only a `.*.tmp` file, never a
partial `.md` note (FR-008).
"""

from __future__ import annotations

import hashlib
import os
import re
import socket
import tempfile
import time
from collections.abc import Callable, Iterable, Iterator, Mapping
from dataclasses import dataclass
from datetime import datetime, timedelta
from pathlib import Path

from easy_note.core import config
from easy_note.core.format import (
    SOURCES,
    VERSION,
    FormatError,
    Note,
    check_text,
    parse,
    render,
)

CONTEXT_KEYS = ("repo", "remote", "branch", "task", "client", "session", "commit")
_ID = re.compile(r"\d{8}-\d{6}-[0-9a-f]{4}(?:-[1-9][0-9]*)?")
_MAX_CLASH = 1000


class NoteNotFound(LookupError):
    pass


@dataclass(frozen=True)
class Added:
    id: str
    path: Path
    created_dir: bool  # True only for the add that created the notes dir


@dataclass(frozen=True)
class Skipped:
    path: Path
    reason: str  # never quotes file content


def _now_ns() -> int:
    return time.time_ns()


def _hostname() -> str:
    return socket.gethostname()


def add(text: str, *, source: str, tags: Iterable[str] = (),
        fields: Mapping[str, str | None] | None = None) -> Added:
    """Write one new note. `fields` holds context keys (CONTEXT_KEYS); None values are left out."""
    check_text(text)
    if source not in SOURCES or source == "inbox":
        raise ValueError(f"unknown source {source!r}")
    tags = _tag_list(tags)
    context = {}
    for key, value in (fields or {}).items():
        if key not in CONTEXT_KEYS:
            raise ValueError(f"{key!r} is not a context field")
        if value is not None and not isinstance(value, str):
            raise TypeError(f"field {key!r} must be a string")
        if value is not None:
            context[key] = value

    root = config.notes_dir()
    try:
        root.mkdir(parents=True)
        created_dir = True
    except FileExistsError:
        created_dir = False

    ns = _now_ns()
    created = datetime.fromtimestamp(ns // 1_000_000_000).astimezone()
    digest = hashlib.sha256(f"{ns}\n{_hostname()}\n{text}".encode()).hexdigest()[:4]
    base_id = f"{created:%Y%m%d-%H%M%S}-{digest}"
    body = text if text.endswith("\n") else text + "\n"

    def content(note_id: str) -> bytes:
        meta = {"easy-note": VERSION, "id": note_id, "created": created.isoformat(timespec="seconds"),
                **context, "tags": tags, "source": source}
        return render(Note(fields=meta, body=body)).encode("utf-8")

    month = root / f"{created:%Y}" / f"{created:%m}"
    month.mkdir(parents=True, exist_ok=True)
    path = _write_new(month, base_id, content)
    return Added(id=path.stem, path=path, created_dir=created_dir)


def read(note_id: str) -> Note:
    path = _note_path(note_id)
    try:
        raw = path.read_bytes()
    except FileNotFoundError:
        raise NoteNotFound(note_id) from None
    note = _parse_file(path, raw)
    if isinstance(note, Skipped):
        raise FormatError(note.reason)
    return note


def update(note_id: str, *, text: str | None = None, tags: Iterable[str] | None = None) -> Note:
    """Rewrite one note (`edit` / `tag`, FR-008): unknown keys kept, `updated` set, temp + os.replace."""
    if text is not None:
        check_text(text)
    if tags is not None:
        tags = _tag_list(tags)
    note = read(note_id)
    if text is not None:
        note.body = text if text.endswith("\n") else text + "\n"
    if tags is not None:
        note.fields["tags"] = tags
    updated = datetime.fromtimestamp(_now_ns() // 1_000_000_000).astimezone()
    note.fields["updated"] = updated.isoformat(timespec="seconds")
    fd, tmp = tempfile.mkstemp(prefix=".", suffix=".tmp", dir=note.path.parent)
    try:
        _write_fd(fd, render(note).encode("utf-8"))
        os.replace(tmp, note.path)
    finally:
        os.close(fd)
        _unlink_quiet(tmp)
    return note


def rm(note_id: str) -> Path:
    """Move a note to `<dir>/.easy-note/trash/`; its mtime becomes the time it was trashed."""
    src = _note_path(note_id)
    if not src.is_file():
        raise NoteNotFound(note_id)
    dst = _trash_dir() / src.name
    dst.parent.mkdir(parents=True, exist_ok=True)
    _move_no_replace(src, dst)
    os.utime(dst)
    return dst


def restore(note_id: str) -> Path:
    dst = _note_path(note_id)
    src = _trash_dir() / dst.name
    if not src.is_file():
        raise NoteNotFound(note_id)
    dst.parent.mkdir(parents=True, exist_ok=True)
    _move_no_replace(src, dst)
    return dst


def empty_trash(older_than: timedelta) -> list[Path]:
    """Delete trash files trashed more than `older_than` ago; return them, sorted."""
    trash = _trash_dir()
    if not trash.is_dir():
        return []
    cutoff = time.time() - older_than.total_seconds()
    gone = sorted(p for p in trash.iterdir() if p.is_file() and p.stat().st_mtime < cutoff)
    for path in gone:
        path.unlink()
    return gone


def iter_notes() -> Iterator[Note | Skipped]:
    """Every `<dir>/YYYY/MM/*.md`, in path order: a Note, or Skipped with its path and reason (FR-017)."""
    root = config.notes_dir()
    for path in sorted(root.glob("[0-9][0-9][0-9][0-9]/[0-9][0-9]/*.md")):
        if not path.is_file():
            continue
        try:
            raw = path.read_bytes()
        except OSError:
            yield Skipped(path.absolute(), "cannot read file")
            continue
        yield _parse_file(path.absolute(), raw)


def _parse_file(path: Path, raw: bytes) -> Note | Skipped:
    try:
        note = parse(raw.decode("utf-8"))
    except UnicodeDecodeError:
        return Skipped(path, "not UTF-8")
    except FormatError as exc:
        return Skipped(path, str(exc))
    if note.id != path.stem:
        return Skipped(path, "id does not match file name")
    note.path = path
    return note


def _tag_list(tags: Iterable[str]) -> list[str]:
    out = None if isinstance(tags, str) else list(tags)
    if out is None or not all(isinstance(t, str) for t in out):
        raise TypeError("tags must be an iterable of strings, not one string")
    return out


def _note_path(note_id: str) -> Path:
    if not isinstance(note_id, str) or not _ID.fullmatch(note_id):
        raise ValueError(f"not a note id: {note_id!r}")
    return config.notes_dir() / note_id[:4] / note_id[4:6] / f"{note_id}.md"


def _trash_dir() -> Path:
    return config.notes_dir() / ".easy-note" / "trash"


def _write_new(folder: Path, base_id: str, content: Callable[[str], bytes]) -> Path:
    fd, tmp = tempfile.mkstemp(prefix=".", suffix=".tmp", dir=folder)  # O_EXCL, mode 0600
    try:
        for n in range(1, _MAX_CLASH + 1):
            note_id = base_id if n == 1 else f"{base_id}-{n}"
            final = folder / f"{note_id}.md"
            _write_fd(fd, content(note_id))
            try:
                os.link(tmp, final)
                return final
            except FileExistsError:
                continue
            except OSError:
                # No hard links here (some Windows / FUSE filesystems): reserve the name with
                # O_EXCL, then swap the full content in. A kill in between leaves an empty
                # file, which readers report as skipped.
                try:
                    os.close(os.open(final, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600))
                except FileExistsError:
                    continue
                try:
                    os.replace(tmp, final)
                except BaseException:
                    _unlink_quiet(final)
                    raise
                return final
        raise FileExistsError(f"{_MAX_CLASH} notes already use ID {base_id}")
    finally:
        os.close(fd)
        _unlink_quiet(tmp)


def _write_fd(fd: int, data: bytes) -> None:
    os.lseek(fd, 0, os.SEEK_SET)
    os.ftruncate(fd, 0)
    view = memoryview(data)
    while view:
        view = view[os.write(fd, view):]
    os.fsync(fd)


def _move_no_replace(src: Path, dst: Path) -> None:
    try:
        os.link(src, dst)
    except FileExistsError:
        raise FileExistsError(f"{dst} already exists; nothing moved") from None
    except OSError:
        if dst.exists():  # no hard links here: best-effort check, then rename
            raise FileExistsError(f"{dst} already exists; nothing moved") from None
        os.replace(src, dst)
        return
    os.unlink(src)


def _unlink_quiet(path: str | Path) -> None:
    try:
        os.unlink(path)
    except FileNotFoundError:
        pass
