"""HUN-256 query engine: list, search and stats over a notes dir (FR-010/011/015/016/017, NFR-007, D10)."""

from __future__ import annotations

import json
import re
import shutil
import time
from collections import Counter
from datetime import date, datetime, timezone
from pathlib import Path

import pytest

from easy_note.core import query

FIXTURE = Path(__file__).parent / "fixtures" / "notes_basic"


@pytest.fixture
def home(tmp_path, monkeypatch):
    """Temp HOME (NFR-003) and local zone UTC+7; restores the zone afterwards."""
    (tmp_path / "home").mkdir()
    monkeypatch.setenv("HOME", str(tmp_path / "home"))
    monkeypatch.setenv("TZ", "Asia/Ho_Chi_Minh")
    time.tzset()
    yield tmp_path
    monkeypatch.undo()
    time.tzset()


@pytest.fixture
def basic(home, monkeypatch):
    """The 30-note fixture (3 repos, offsets +07:00 and -05:00) plus one unreadable file."""
    monkeypatch.setenv("EASY_NOTE_DIR", str(FIXTURE))
    return FIXTURE


@pytest.fixture
def empty_dir(home, monkeypatch):
    root = home / "notes"
    root.mkdir()
    monkeypatch.setenv("EASY_NOTE_DIR", str(root))
    return root


def write_note(root: Path, note_id: str, created: str, body: str = "hello\n", *, tags=(),
               repo=None, task=None, source="cli") -> None:
    """One note file written by hand, so a test controls `created` exactly."""
    lines = ["---", "easy-note: 1", f'id: "{note_id}"', f'created: "{created}"']
    if repo:
        lines.append(f'repo: "{repo}"')
    if task:
        lines.append(f'task: "{task}"')
    lines += [f"tags: {json.dumps(list(tags))}", f'source: "{source}"', "---", body]
    folder = root / note_id[:4] / note_id[4:6]
    folder.mkdir(parents=True, exist_ok=True)
    (folder / f"{note_id}.md").write_text("\n".join(lines), encoding="utf-8")


# An oracle that reads the fixture files with plain regexes, never through the code under test.
def raw_notes(root: Path = FIXTURE) -> list[dict]:
    out = []
    for path in sorted(root.glob("*/*/*.md")):
        text = path.read_text(encoding="utf-8")
        if not text.startswith("---\n"):
            continue
        front, body = text[4:].split("\n---\n", 1)
        meta = dict(re.findall(r'^([a-z-]+): "?(.*?)"?$', front, re.M))
        meta["tags"] = json.loads(re.search(r"^tags: (.*)$", front, re.M).group(1))
        meta["body"] = body
        meta["local_day"] = meta["created"][:10]
        meta["instant"] = datetime.fromisoformat(meta["created"]).astimezone(timezone.utc)
        out.append(meta)
    return out


def ids(result: dict) -> list[str]:
    return [item["id"] for item in result["items"]]


def expected(pred) -> list[str]:
    hits = [n for n in raw_notes() if pred(n)]
    return [n["id"] for n in sorted(hits, key=lambda n: (-n["instant"].timestamp(), n["id"]))]


def topic(note_id: str) -> int:
    return int(note_id[-4:], 16) % 5  # the fixture cycles 5 topics


# AC1 — AND of words, any case, in title, body or tags
def test_search_needs_every_word_in_any_case(basic):
    both = query.search("retry firefox", limit=50)
    assert ids(both) == expected(lambda n: topic(n["id"]) == 0)
    assert both["total"] == 6
    assert ids(query.search("RETRY FiReFoX", limit=50)) == ids(both)
    assert query.search("retry nonexistentword")["total"] == 0


def test_search_matches_title_body_and_tags(basic):
    assert query.search("navigation")["total"] == 6  # body only (second paragraph of topic 0)
    assert query.search("changelog")["total"] == 6  # body only (topic 3)
    assert {i["id"] for i in query.search("idea/ui", limit=50)["items"]} == {
        n["id"] for n in raw_notes() if "idea/ui" in n["tags"]}  # tag only
    # "flaky" is a tag of topic 0 and the title word of topic 1
    assert {topic(i) for i in ids(query.search("flaky", limit=50))} == {0, 1}


def test_search_words_may_sit_in_different_places(basic):
    # "firefox" is in the title and a tag, "navigation" only in the body
    assert query.search("firefox navigation")["total"] == 6
    assert query.search("ci navigation")["total"] == 0


def test_search_folds_case_beyond_ascii(basic):
    assert query.search("STRASSE")["total"] == 6  # "ß" folds to "ss"
    assert query.search("straße")["total"] == 6
    assert query.search("CAFÉ résumé")["total"] == 6


def test_empty_query_is_an_error_not_every_note(basic):
    for bad in ("", "   ", "\n\t"):
        with pytest.raises(ValueError):
            query.search(bad)
    with pytest.raises(TypeError):
        query.search(None)


# AC2 — each filter narrows the result
def test_each_filter_matches_the_oracle(basic):
    cases = {
        "repo": ("beta", lambda n: n.get("repo") == "beta"),
        "task": ("T101", lambda n: n.get("task") == "T101"),
        "tag": ("firefox", lambda n: "firefox" in n["tags"]),
        "source": ("hook", lambda n: n["source"] == "hook"),
    }
    for name, (value, pred) in cases.items():
        want = expected(pred)
        assert want and len(want) < 30, name
        assert ids(query.list_notes(limit=50, **{name: value})) == want, name
        # search applies the same filters (FR-011): every fixture note has "#" in its title
        assert ids(query.search("#", limit=50, **{name: value})) == want, name
    assert ids(query.search("release", limit=50, repo="gamma")) == expected(
        lambda n: topic(n["id"]) == 3 and n.get("repo") == "gamma")


def test_tag_filter_is_exact_for_hierarchical_tags(basic):
    assert query.list_notes(tag="idea/ui")["total"] == 6
    assert ids(query.list_notes(tag="idea", limit=50)) == expected(lambda n: "idea" in n["tags"])
    assert query.list_notes(tag="ui")["total"] == 0
    assert query.list_notes(tag="idea/")["total"] == 0
    assert ids(query.list_notes(tag="FIREFOX", limit=50)) == expected(lambda n: "firefox" in n["tags"])


def test_filters_combine_with_and(basic):
    want = expected(lambda n: n.get("repo") == "alpha" and n["source"] == "cli")
    assert ids(query.list_notes(repo="alpha", source="cli", limit=50)) == want
    both = query.list_notes(repo="alpha", source="hook", limit=50)
    assert ids(both) == expected(lambda n: n.get("repo") == "alpha" and n["source"] == "hook")


def test_unknown_filter_value_gives_nothing(basic):
    assert query.list_notes(repo="no-such-repo") == {"items": [], "total": 0, "truncated": False}


def test_date_filters_use_local_dates(basic):
    assert ids(query.list_notes(day="2026-11-05", limit=50)) == expected(lambda n: n["local_day"] == "2026-11-05")
    assert ids(query.list_notes(since="2026-11-10", limit=50)) == expected(lambda n: n["local_day"] >= "2026-11-10")
    assert ids(query.list_notes(until="2026-10-31", limit=50)) == expected(lambda n: n["local_day"] <= "2026-10-31")
    window = query.list_notes(since="2026-11-03", until="2026-11-06", limit=50)
    assert ids(window) == expected(lambda n: "2026-11-03" <= n["local_day"] <= "2026-11-06")
    assert ids(query.list_notes(day="2026-11-04", since="2026-11-05")) == []  # filters AND
    assert query.list_notes(day=date(2026, 11, 5))["total"] == query.list_notes(day="2026-11-05")["total"]


@pytest.mark.parametrize("bad", ["2026-13-01", "yesterday", "", "14/11/2026", 20261114])
def test_bad_date_is_an_error(basic, bad):
    for name in ("since", "until", "day"):
        with pytest.raises(ValueError):
            query.list_notes(**{name: bad})


def test_datetime_is_not_a_date(basic):
    with pytest.raises(TypeError):
        query.list_notes(day=datetime(2026, 11, 5))


# AC3 — a note counts for the local day written in its offset, not its UTC day
def test_late_evening_note_counts_for_its_local_day(empty_dir):
    write_note(empty_dir, "20261114-233000-aaaa", "2026-11-14T23:30:00+07:00", "late\n")
    write_note(empty_dir, "20261114-210000-bbbb", "2026-11-14T21:00:00-05:00", "west\n")  # UTC: 15th
    write_note(empty_dir, "20261115-013000-cccc", "2026-11-15T01:30:00+07:00", "early\n")  # UTC: 14th
    assert ids(query.list_notes(day="2026-11-14")) == ["20261114-210000-bbbb", "20261114-233000-aaaa"]
    assert ids(query.list_notes(day="2026-11-15")) == ["20261115-013000-cccc"]
    assert query.list_notes(since="2026-11-15")["total"] == 1
    assert query.list_notes(until="2026-11-14")["total"] == 2


def test_fixture_notes_around_midnight(basic):
    assert "20261103-011500-0008" in ids(query.list_notes(day="2026-11-03", limit=50))  # 01:15+07:00, UTC 2nd
    assert "20261103-011500-0008" not in ids(query.list_notes(day="2026-11-02", limit=50))
    assert "20261029-230000-0003" in ids(query.list_notes(day="2026-10-29", limit=50))  # 23:00-05:00, UTC 30th
    assert "20261029-230000-0003" not in ids(query.list_notes(day="2026-10-30", limit=50))


@pytest.mark.parametrize("zone", ["UTC", "Asia/Ho_Chi_Minh", "Pacific/Auckland", "America/Los_Angeles"])
def test_result_does_not_depend_on_the_reader_zone(basic, monkeypatch, zone):
    want = expected(lambda n: n["local_day"] == "2026-11-03")
    monkeypatch.setenv("TZ", zone)
    time.tzset()
    assert ids(query.list_notes(day="2026-11-03", limit=50)) == want
    assert want  # the day is not empty


# AC4 — newest instant first, ties by ID, same output twice
def test_whole_fixture_is_newest_first_by_utc_instant(basic):
    assert ids(query.list_notes(limit=50)) == expected(lambda n: True)
    got = query.list_notes(limit=50)
    assert got["total"] == 30
    # the order is by instant, not by the wall-clock text of `created`
    instants = [datetime.fromisoformat(i["created"]).timestamp() for i in got["items"]]
    assert instants == sorted(instants, reverse=True)


def test_equal_instants_are_ordered_by_id(empty_dir):
    # same instant in three offsets, plus a newer and an older note
    write_note(empty_dir, "20261114-100000-cccc", "2026-11-14T10:00:00+07:00")
    write_note(empty_dir, "20261114-030000-bbbb", "2026-11-14T03:00:00+00:00")
    write_note(empty_dir, "20261113-220000-aaaa", "2026-11-13T22:00:00-05:00")
    write_note(empty_dir, "20261114-120000-dddd", "2026-11-14T12:00:00+07:00")
    write_note(empty_dir, "20261114-080000-eeee", "2026-11-14T08:00:00+07:00")
    # ID order within the tie is "20261113-220000-aaaa" < "20261114-030000-bbbb" < "20261114-100000-cccc"
    assert ids(query.list_notes()) == [
        "20261114-120000-dddd",
        "20261113-220000-aaaa", "20261114-030000-bbbb", "20261114-100000-cccc",
        "20261114-080000-eeee",
    ]


def test_clash_suffix_ids_tie_break_as_text(empty_dir):
    for suffix in ("", "-2", "-10"):
        write_note(empty_dir, f"20261114-100000-abcd{suffix}", "2026-11-14T10:00:00+07:00")
    assert ids(query.list_notes()) == [
        "20261114-100000-abcd", "20261114-100000-abcd-10", "20261114-100000-abcd-2"]


def test_same_output_twice(basic):
    calls = [
        lambda: query.list_notes(limit=50),
        lambda: query.list_notes(repo="alpha", since="2026-11-01"),
        lambda: query.search("retry firefox", limit=50),
        lambda: query.search("note", tag="ci"),
        lambda: query.stats(),
    ]
    for call in calls:
        assert json.dumps(call()) == json.dumps(call())


def test_listing_order_ignores_the_order_the_files_are_found(home, monkeypatch, tmp_path):
    first, second = tmp_path / "a", tmp_path / "b"
    for root, order in ((first, [1, 2, 3]), (second, [3, 1, 2])):
        for n in order:
            write_note(root, f"20261114-10000{n}-abcd", "2026-11-14T10:00:00+07:00")
    results = []
    for root in (first, second):
        monkeypatch.setenv("EASY_NOTE_DIR", str(root))
        results.append(json.dumps(query.list_notes()))
    assert results[0] == results[1]


# AC5 — limit cap 50, total, truncated
def test_limit_is_capped_at_fifty(empty_dir):
    for n in range(60):
        write_note(empty_dir, f"20261114-1{n // 60:01d}{n // 6:02d}{n % 6}0-abcd", f"2026-11-14T10:{n:02d}:00+07:00")
    big = query.list_notes(limit=1000)
    assert len(big["items"]) == 50
    assert big["total"] == 60
    assert big["truncated"] is True
    assert len(query.search("hello", limit=51)["items"]) == 50
    small = query.list_notes(limit=5)
    assert (len(small["items"]), small["total"], small["truncated"]) == (5, 60, True)
    assert ids(small) == ids(big)[:5]


def test_default_limit_and_truncated_flag(basic):
    default = query.list_notes()
    assert len(default["items"]) == query.DEFAULT_LIMIT == 20
    assert (default["total"], default["truncated"]) == (30, True)
    exact = query.list_notes(limit=30)
    assert (len(exact["items"]), exact["total"], exact["truncated"]) == (30, 30, False)
    assert query.list_notes(limit=50)["truncated"] is False


def test_total_counts_every_match_not_the_page(basic):
    page = query.search("retry firefox", limit=2)
    assert (len(page["items"]), page["total"], page["truncated"]) == (2, 6, True)


@pytest.mark.parametrize("bad", [0, -1, True, 2.5, "5", None])
def test_bad_limit_is_an_error(basic, bad):
    with pytest.raises(ValueError):
        query.list_notes(limit=bad)
    with pytest.raises(ValueError):
        query.search("retry", limit=bad)


# AC6 — item shape and snippet
def test_item_has_the_frozen_fields(basic):
    item = query.search("retry firefox", limit=1)["items"][0]
    assert set(item) == {"id", "title", "created", "repo", "task", "tags", "snippet"}
    raw = {n["id"]: n for n in raw_notes()}[item["id"]]
    assert item["created"] == raw["created"]
    assert item["repo"] == raw.get("repo")
    assert item["tags"] == raw["tags"]
    assert item["title"].startswith("Retry bug on Firefox #")
    json.dumps(item)  # plain data, ready for the CLI and MCP


def test_item_without_repo_or_task_has_none(empty_dir):
    write_note(empty_dir, "20261114-100000-abcd", "2026-11-14T10:00:00+07:00", "plain\n")
    item = query.list_notes()["items"][0]
    assert item["repo"] is None and item["task"] is None and item["tags"] == []


def test_snippet_is_cut_around_the_first_hit(empty_dir):
    body = "Title line\n\n" + ("alpha " * 200) + "NEEDLE here " + ("omega " * 200) + "\n"
    write_note(empty_dir, "20261114-100000-abcd", "2026-11-14T10:00:00+07:00", body)
    snippet = query.search("needle")["items"][0]["snippet"]
    assert len(snippet) <= query.SNIPPET_MAX == 200
    assert "NEEDLE here" in snippet
    assert snippet.startswith("…") and snippet.endswith("…")
    assert "\n" not in snippet


def test_snippet_picks_the_earliest_of_several_words(empty_dir):
    body = "Title\n\n" + ("x " * 300) + "second " + ("y " * 50) + "first " + "\n"
    write_note(empty_dir, "20261114-100000-abcd", "2026-11-14T10:00:00+07:00", body)
    assert "second" in query.search("first second")["items"][0]["snippet"]


def test_snippet_never_exceeds_the_limit_at_any_hit_position(empty_dir):
    for n in (0, 30, 39, 40, 41, 150, 159, 160, 161, 199, 200, 201, 500):
        write_note(empty_dir, f"20261114-1000{n // 100:02d}-{n % 100:04x}",
                   "2026-11-14T10:00:00+07:00", "q" * n + "needle" + "q" * (400 - n) + "\n")
    for item in query.search("needle", limit=50)["items"]:
        assert len(item["snippet"]) <= 200
        assert "needle" in item["snippet"]


def test_short_body_snippet_is_the_whole_body(empty_dir):
    write_note(empty_dir, "20261114-100000-abcd", "2026-11-14T10:00:00+07:00", "Title\n\nsmall body here\n")
    assert query.search("small")["items"][0]["snippet"] == "Title small body here"


def test_list_snippet_is_the_start_of_the_body(empty_dir):
    write_note(empty_dir, "20261114-100000-abcd", "2026-11-14T10:00:00+07:00", "z" * 500 + "\n")
    assert query.list_notes()["items"][0]["snippet"] == "z" * 199 + "…"


def test_hit_only_in_tags_gives_the_start_of_the_body(empty_dir):
    write_note(empty_dir, "20261114-100000-abcd", "2026-11-14T10:00:00+07:00", "Body text\n", tags=["onlytag"])
    item = query.search("onlytag")["items"][0]
    assert item["snippet"] == "Body text"


def test_snippet_finds_hit_after_characters_that_change_length_when_folded(empty_dir):
    write_note(empty_dir, "20261114-100000-abcd", "2026-11-14T10:00:00+07:00", "ß" * 300 + " target here\n")
    snippet = query.search("TARGET")["items"][0]["snippet"]
    assert "target" in snippet and len(snippet) <= 200


def test_title_is_cut_to_120(empty_dir):
    write_note(empty_dir, "20261114-100000-abcd", "2026-11-14T10:00:00+07:00", "t" * 300 + "\n")
    assert len(query.list_notes()["items"][0]["title"]) == 120


# AC7 — stats
def test_stats_counts_match_the_oracle(basic):
    notes = raw_notes()
    got = query.stats()
    assert got["total"] == 30
    assert got["by_repo"] == dict(Counter(n["repo"] for n in notes))
    assert got["by_task"] == dict(Counter(n["task"] for n in notes if "task" in n))
    assert got["by_tag"] == dict(Counter(t for n in notes for t in n["tags"]))
    assert got["by_source"] == dict(Counter(n["source"] for n in notes))
    assert got["by_month"] == dict(Counter(n["local_day"][:7] for n in notes))
    assert got["by_month"] == {"2026-11": 23, "2026-10": 7}
    assert got["oldest"] == min(n["local_day"] for n in notes) == "2026-10-28"
    assert got["newest"] == max(n["local_day"] for n in notes) == "2026-11-13"
    assert sum(got["by_repo"].values()) == 30
    assert sum(got["by_task"].values()) == 20  # a third of the notes have no task


def test_stats_counts_are_ordered_by_count_then_name(basic):
    got = query.stats()
    for key in ("by_repo", "by_task", "by_tag", "by_source", "by_month"):
        pairs = list(got[key].items())
        assert pairs == sorted(pairs, key=lambda kv: (-kv[1], kv[0])), key


def test_stats_lists_skipped_files_with_path_and_reason(basic):
    got = query.stats()
    assert got["skipped"] == [{"path": str(FIXTURE.absolute() / "2026/11/20261110-100000-bad0.md"),
                               "reason": "no front matter"}]


def test_stats_returns_no_note_text(basic):
    dump = json.dumps(query.stats(), ensure_ascii=False)
    for note in raw_notes():
        for sentence in [s for s in note["body"].splitlines() if s.strip()]:
            assert sentence not in dump
    assert "SKIPPED-FILE-TEXT" not in dump  # the unreadable file's text stays out too
    assert "Firefox" not in dump  # a title word


def test_stats_follow_the_filters(basic):
    got = query.stats(repo="alpha")
    alpha = [n for n in raw_notes() if n["repo"] == "alpha"]
    assert got["total"] == len(alpha) == 10
    assert got["by_repo"] == {"alpha": 10}
    assert got["by_source"] == dict(Counter(n["source"] for n in alpha))
    assert len(got["skipped"]) == 1  # skipped files are reported whatever the filter


def test_stats_of_an_empty_dir(empty_dir):
    assert query.stats() == {
        "total": 0, "by_repo": {}, "by_task": {}, "by_tag": {}, "by_source": {}, "by_month": {},
        "oldest": None, "newest": None, "skipped": []}


def test_stats_counts_a_repeated_tag_once_per_note(empty_dir):
    write_note(empty_dir, "20261114-100000-abcd", "2026-11-14T10:00:00+07:00", tags=["a", "a", "b"])
    assert query.stats()["by_tag"] == {"a": 1, "b": 1}


def test_list_and_search_leave_skipped_files_out(basic):
    assert "20261110-100000-bad0" not in ids(query.list_notes(limit=50))
    assert query.list_notes(limit=50)["total"] == 30


def test_queries_change_nothing_on_disk(basic, tmp_path, monkeypatch):
    copy = tmp_path / "copy"
    shutil.copytree(FIXTURE, copy)
    monkeypatch.setenv("EASY_NOTE_DIR", str(copy))
    before = {p: (p.read_bytes(), p.stat().st_mtime_ns) for p in copy.rglob("*") if p.is_file()}
    query.list_notes(limit=50), query.search("retry"), query.stats()
    after = {p: (p.read_bytes(), p.stat().st_mtime_ns) for p in copy.rglob("*") if p.is_file()}
    assert before == after
    assert {p.name for p in copy.rglob("*") if p.is_file()} == {p.name for p in FIXTURE.rglob("*") if p.is_file()}


def test_missing_notes_dir_is_empty_not_an_error(home, monkeypatch):
    monkeypatch.setenv("EASY_NOTE_DIR", str(home / "never-created"))
    assert query.list_notes() == {"items": [], "total": 0, "truncated": False}
    assert query.stats()["total"] == 0
