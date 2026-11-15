"""Parse and render one note in the `easy-note: 1` format (docs/NOTE_FORMAT.md, DDR 0002).

`render(parse(f)) == f` for every file this module rendered: known keys are written in a fixed
order with every string double-quoted, so YAML never turns `no`, `on` or a date into another type.
Unknown keys (for example ones Obsidian adds) are kept as their original YAML source text.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Any

import yaml

VERSION = 1
MAX_TEXT = 20_000
MAX_TITLE = 120
SOURCES = ("cli", "mcp", "hook", "quick", "inbox")
STRING_KEYS = ("id", "created", "updated", "repo", "remote", "branch", "task", "source",
               "client", "session", "commit")
KEY_ORDER = ("easy-note", "id", "created", "updated", "repo", "remote", "branch", "task", "tags",
             "source", "client", "session", "commit")
REQUIRED = ("easy-note", "id", "created", "tags", "source")

_FRONT = re.compile(r"\A---\r?\n(.*?\r?\n)?---\r?(?:\n|\Z)", re.DOTALL)
# Chars a YAML 1.1 reader rejects or reads as a line break, even inside a quoted string.
_YAML_UNSAFE = re.compile("[\x7f-\x9f  \ud800-\udfff￾￿]")


class FormatError(ValueError):
    """The file is not a readable `easy-note: 1` note. The message never quotes file content."""


class TextTooLong(ValueError):
    """Note text over MAX_TEXT chars: the only refusal on write (FR-002)."""


class _Loader(yaml.SafeLoader):
    """SafeLoader that keeps timestamps as strings, so `created` reads back exactly as written."""


_Loader.yaml_implicit_resolvers = {
    first: [(tag, rx) for tag, rx in resolvers if tag != "tag:yaml.org,2002:timestamp"]
    for first, resolvers in yaml.SafeLoader.yaml_implicit_resolvers.items()
}


@dataclass
class Note:
    fields: dict[str, Any]  # known keys only, validated
    body: str  # everything after the closing `---` line, unchanged
    extra: list[str] = field(default_factory=list)  # unknown keys, original YAML text, in file order
    path: Path | None = None  # set by the store; never rendered

    @property
    def id(self) -> str:
        return self.fields["id"]

    @property
    def title(self) -> str:
        return self.body.split("\n", 1)[0].rstrip("\r")[:MAX_TITLE]


def check_text(text: str) -> None:
    if len(text) > MAX_TEXT:
        raise TextTooLong(f"note text is {len(text):,} chars; the limit is {MAX_TEXT:,}")


def parse(text: str) -> Note:
    match = _FRONT.match(text)
    if not match:
        raise FormatError("no front matter")
    front = match.group(1) or ""
    try:
        data = yaml.load(front, Loader=_Loader)
    except yaml.YAMLError as exc:
        mark = getattr(exc, "problem_mark", None)
        raise FormatError("front matter is not valid YAML" + (f" (line {mark.line + 2})" if mark else "")) from None
    if not isinstance(data, dict):
        raise FormatError("front matter is not a mapping")
    _validate(data)
    known = {key: data[key] for key in KEY_ORDER if key in data}
    extra = _raw_unknown(front) if len(known) < len(data) else []
    return Note(fields=known, body=text[match.end():], extra=extra)


def render(note: Note) -> str:
    lines = [f"{key}: {_value(note.fields[key])}\n" for key in KEY_ORDER if key in note.fields]
    return "---\n" + "".join(lines) + "".join(note.extra) + "---\n" + note.body


def _validate(data: dict) -> None:
    version = data.get("easy-note")
    if type(version) is not int:
        raise FormatError("missing or non-integer easy-note version")
    if version != VERSION:
        raise FormatError(f"unsupported version {version}")
    for key in REQUIRED:
        if key not in data:
            raise FormatError(f"missing required key {key!r}")
    for key in STRING_KEYS:
        if key in data and not isinstance(data[key], str):
            raise FormatError(f"key {key!r} is not a string")
    if not isinstance(data["tags"], list) or not all(isinstance(t, str) for t in data["tags"]):
        raise FormatError("key 'tags' is not a list of strings")
    if data["source"] not in SOURCES:
        raise FormatError("unknown source")
    for key in ("created", "updated"):
        if key in data:
            try:
                stamp = datetime.fromisoformat(data[key])
            except ValueError:
                raise FormatError(f"key {key!r} is not an ISO 8601 time") from None
            if stamp.tzinfo is None:
                raise FormatError(f"key {key!r} has no UTC offset")


def _raw_unknown(front: str) -> list[str]:
    """Source text of each top-level unknown key, from the key to the start of the next key."""
    pairs = yaml.compose(front, Loader=_Loader).value
    out = []
    for i, (key, _value_node) in enumerate(pairs):
        if key.value in KEY_ORDER:
            continue
        end = pairs[i + 1][0].start_mark.index if i + 1 < len(pairs) else len(front)
        chunk = front[key.start_mark.index:end].rstrip("\r\n")
        out.append(chunk + "\n")
    return out


def _value(value: Any) -> str:
    if isinstance(value, list):
        return "[" + ", ".join(_quote(item) for item in value) + "]"
    if isinstance(value, int) and not isinstance(value, bool):
        return str(value)
    return _quote(value)


def _quote(text: str) -> str:
    # A JSON string is a valid YAML double-quoted scalar; escape what YAML would misread.
    return _YAML_UNSAFE.sub(lambda m: f"\\u{ord(m.group()):04x}", json.dumps(text, ensure_ascii=False))
