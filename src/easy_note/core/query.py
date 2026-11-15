"""List, search and stats over the notes dir (FR-010, FR-011, FR-015, FR-016, FR-017).

Every call scans the files through `store.iter_notes()`: no index, no ranking, no model (D9, NFR-007).
The CLI and the MCP tools call these three functions and serialize the plain dicts they return.
Same files + same arguments give the same output: newest `created` instant first, ties by ID.

A note's local date is the date in the UTC offset written in its `created` (D10): the same date as
its `YYYY/MM` folder and its ID, whatever zone the reader is in.
"""

from __future__ import annotations

from collections import Counter
from datetime import date, datetime
from typing import Any

from easy_note.core import store
from easy_note.core.format import Note

MAX_LIMIT = 50
DEFAULT_LIMIT = 20
SNIPPET_MAX = 200
_ELLIPSIS = "…"
_LEAD = 40  # chars of context kept before the first hit


def list_notes(*, repo: str | None = None, task: str | None = None, tag: str | None = None,
               source: str | None = None, since: str | date | None = None,
               until: str | date | None = None, day: str | date | None = None,
               limit: int = DEFAULT_LIMIT) -> dict[str, Any]:
    """Notes that pass every filter, newest first: `{"items", "total", "truncated"}`."""
    return _result(_select((), repo, task, tag, source, since, until, day)[0], (), limit)


def search(query: str, *, repo: str | None = None, task: str | None = None, tag: str | None = None,
           source: str | None = None, since: str | date | None = None,
           until: str | date | None = None, day: str | date | None = None,
           limit: int = DEFAULT_LIMIT) -> dict[str, Any]:
    """Notes holding every word of `query` (case-insensitive substring) in title, body or tags."""
    if not isinstance(query, str):
        raise TypeError("query must be a string")
    words = tuple(w.casefold() for w in query.split())
    if not words:
        raise ValueError("query is empty")
    return _result(_select(words, repo, task, tag, source, since, until, day)[0], words, limit)


def stats(*, repo: str | None = None, task: str | None = None, tag: str | None = None,
          source: str | None = None, since: str | date | None = None,
          until: str | date | None = None, day: str | date | None = None) -> dict[str, Any]:
    """Counts by repo, task, tag, source and month, oldest/newest date, and every skipped file.

    A note without a repo (or task) is left out of that count. No note text is returned.
    `skipped` ignores the filters: an unreadable file has no fields to filter on.
    """
    hits, skipped = _select((), repo, task, tag, source, since, until, day)
    dates = [stamp.date() for stamp, _ in hits]
    return {
        "total": len(hits),
        "by_repo": _count(n.fields.get("repo") for _, n in hits),
        "by_task": _count(n.fields.get("task") for _, n in hits),
        "by_tag": _count(t for _, n in hits for t in dict.fromkeys(n.fields["tags"])),
        "by_source": _count(n.fields["source"] for _, n in hits),
        "by_month": _count(f"{d:%Y-%m}" for d in dates),
        "oldest": min(dates).isoformat() if dates else None,
        "newest": max(dates).isoformat() if dates else None,
        "skipped": [{"path": str(s.path), "reason": s.reason} for s in skipped],
    }


def _select(words, repo, task, tag, source, since, until, day):
    since, until, day = _as_date(since, "since"), _as_date(until, "until"), _as_date(day, "day")
    tag = tag.casefold() if tag is not None else None
    hits: list[tuple[datetime, Note]] = []
    skipped: list[store.Skipped] = []
    for entry in store.iter_notes():
        if isinstance(entry, store.Skipped):
            skipped.append(entry)
            continue
        fields = entry.fields
        stamp = datetime.fromisoformat(fields["created"])
        local = stamp.date()
        if ((repo is not None and fields.get("repo") != repo)
                or (task is not None and fields.get("task") != task)
                or (source is not None and fields["source"] != source)
                or (tag is not None and tag not in (t.casefold() for t in fields["tags"]))
                or (since is not None and local < since)
                or (until is not None and local > until)
                or (day is not None and local != day)):
            continue
        if words:
            haystack = (entry.body + "\n" + " ".join(fields["tags"])).casefold()
            if not all(word in haystack for word in words):
                continue
        hits.append((stamp, entry))
    hits.sort(key=lambda hit: (-hit[0].timestamp(), hit[1].id))
    return hits, skipped


def _result(hits, words, limit: int) -> dict[str, Any]:
    if isinstance(limit, bool) or not isinstance(limit, int) or limit < 1:
        raise ValueError("limit must be a whole number of 1 or more")
    shown = hits[:min(limit, MAX_LIMIT)]
    return {
        "items": [_item(note, words) for _, note in shown],
        "total": len(hits),
        "truncated": len(hits) > len(shown),
    }


def _item(note: Note, words) -> dict[str, Any]:
    fields = note.fields
    return {
        "id": note.id,
        "title": note.title,
        "created": fields["created"],
        "repo": fields.get("repo"),
        "task": fields.get("task"),
        "tags": list(fields["tags"]),
        "snippet": _snippet(note.body, words),
    }


def _snippet(body: str, words) -> str:
    """One line of at most SNIPPET_MAX chars around the first hit in the body (start of body if none)."""
    text = " ".join(body.split())
    folded_hits = [i for i in (_find(text, w) for w in words) if i >= 0]
    start = max(0, min(folded_hits) - _LEAD) if folded_hits else 0
    budget = SNIPPET_MAX - (1 if start else 0)
    if len(text) - start > budget:
        out = text[start:start + budget - 1] + _ELLIPSIS
    else:
        out = text[start:]
    return (_ELLIPSIS + out) if start else out


def _find(text: str, folded_word: str) -> int:
    """Index in `text` of the first casefolded match (casefold can change a string's length)."""
    pos = text.casefold().find(folded_word)
    if pos < 0:
        return -1
    seen = 0
    for i, ch in enumerate(text):
        if seen >= pos:
            return i
        seen += len(ch.casefold())
    return len(text)


def _count(values) -> dict[str, int]:
    counts = Counter(v for v in values if v is not None)
    return dict(sorted(counts.items(), key=lambda kv: (-kv[1], kv[0])))


def _as_date(value: str | date | None, name: str) -> date | None:
    if value is None:
        return None
    if isinstance(value, datetime):
        raise TypeError(f"{name} must be a date or a YYYY-MM-DD string")
    if isinstance(value, date):
        return value
    try:
        return date.fromisoformat(value)
    except (TypeError, ValueError):
        raise ValueError(f"{name} must be a date like 2026-11-14") from None
