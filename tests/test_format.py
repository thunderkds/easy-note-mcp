"""HUN-253 note format: exact round-trip, YAML traps, version check, unknown keys, title and size rules."""

from __future__ import annotations

from pathlib import Path

import pytest
import yaml

from easy_note.core import format as fmt

FIXTURES = Path(__file__).parent / "fixtures" / "notes" / "2026" / "11"


def _note(**fields) -> fmt.Note:
    base = {
        "easy-note": 1,
        "id": "20261114-164713-a1b2",
        "created": "2026-11-14T16:47:13+07:00",
        "tags": [],
        "source": "cli",
    }
    base.update(fields)
    return fmt.Note(fields=base, body="text\n")


# AC1 — render(parse(f)) of a tool-written note is byte-identical to f


def test_round_trip_of_tool_written_fixture_is_byte_identical():
    raw = (FIXTURES / "20261114-164713-a1b2.md").read_bytes()
    assert fmt.render(fmt.parse(raw.decode("utf-8"))).encode("utf-8") == raw


@pytest.mark.parametrize(
    "value",
    ["no", "on", "yes", "null", "~", "2026-11-14", "12", "0x1f", "a: b", "#x", "- x", "[x]",
     'quote " and \\ slash', "tab\there", "é ü 中文", "emoji 😀", "nel\x85", "ls ", "del\x7f", ""],
)
def test_render_quotes_strings_so_yaml_keeps_them_strings(value):
    note = _note(branch=value, tags=[value] if value else [])
    text = fmt.render(note)
    again = fmt.parse(text)
    assert again.fields["branch"] == value
    assert fmt.render(again) == text
    assert yaml.safe_load(text.split("---\n")[1])["branch"] == value  # a plain YAML reader agrees


def test_render_uses_fixed_key_order_whatever_the_input_order():
    shuffled = {"source": "cli", "tags": ["a"], "repo": "r", "created": "2026-11-14T16:47:13+07:00",
                "id": "20261114-164713-a1b2", "easy-note": 1}
    keys = [line.split(":")[0] for line in fmt.render(fmt.Note(fields=shuffled, body="x\n")).splitlines()[1:6]]
    assert keys == ["easy-note", "id", "created", "repo", "tags"]


def test_created_stays_a_string_even_when_hand_written_unquoted():
    text = "---\neasy-note: 1\nid: x\ncreated: 2026-11-14T16:47:13+07:00\ntags: []\nsource: cli\n---\nbody\n"
    assert fmt.parse(text).fields["created"] == "2026-11-14T16:47:13+07:00"


def test_crlf_file_parses():
    note = fmt.parse((FIXTURES / "20261114-170300-c71f.md").read_bytes().decode("utf-8"))
    assert note.fields["id"] == "20261114-170300-c71f"
    assert note.body == "Edited on Windows\r\nsecond line\r\n"
    assert note.title == "Edited on Windows"


# AC6 (parse side) — bad front matter and `easy-note: 2` are refused with a reason, never "fixed"


@pytest.mark.parametrize(
    ("name", "reason"),
    [
        ("20261114-170000-bad1.md", "not valid YAML"),
        ("20261114-170100-0002.md", "unsupported version 2"),
        ("20261114-170200-0003.md", "no front matter"),
    ],
)
def test_bad_fixture_files_raise_format_error(name, reason):
    with pytest.raises(fmt.FormatError, match=reason):
        fmt.parse((FIXTURES / name).read_text(encoding="utf-8"))


@pytest.mark.parametrize(
    "front",
    [
        "easy-note: true\nid: x\ncreated: 2026-11-14T16:47:13+07:00\ntags: []\nsource: cli\n",  # bool is not 1
        "id: x\ncreated: 2026-11-14T16:47:13+07:00\ntags: []\nsource: cli\n",  # version missing
        "easy-note: 1\ncreated: 2026-11-14T16:47:13+07:00\ntags: []\nsource: cli\n",  # id missing
        "easy-note: 1\nid: x\ncreated: yesterday\ntags: []\nsource: cli\n",  # created not ISO
        "easy-note: 1\nid: x\ncreated: 2026-11-14T16:47:13\ntags: []\nsource: cli\n",  # no offset
        "easy-note: 1\nid: x\ncreated: 2026-11-14T16:47:13+07:00\ntags: flaky\nsource: cli\n",  # tags not a list
        "easy-note: 1\nid: x\ncreated: 2026-11-14T16:47:13+07:00\ntags: []\nsource: fax\n",  # unknown source
        "easy-note: 1\nid: x\ncreated: 2026-11-14T16:47:13+07:00\ntags: []\nsource: cli\nrepo: 12\n",
        "- just\n- a list\n",
    ],
)
def test_invalid_front_matter_raises_format_error(front):
    with pytest.raises(fmt.FormatError):
        fmt.parse(f"---\n{front}---\nbody\n")


def test_format_error_reason_never_quotes_file_content():
    secret_line = "easy-note: 1\nid: \"ghp_SECRETVALUE\n"
    with pytest.raises(fmt.FormatError) as exc:
        fmt.parse(f"---\n{secret_line}---\nbody\n")
    assert "ghp_" not in str(exc.value)


# AC7 (parse side) — unknown keys are kept verbatim, known keys stay canonical


def test_unknown_keys_render_verbatim_after_known_keys():
    raw = (FIXTURES / "20261114-170400-0b5d.md").read_text(encoding="utf-8")
    out = fmt.render(fmt.parse(raw))
    assert "aliases:\n  - firefox retry\n" in out
    assert "date: 2024-01-01 # Obsidian date property\n" in out
    assert "cssclasses: [wide]\n" in out
    assert out.index("source:") < out.index("aliases:")
    assert yaml.safe_load(out.split("---\n")[1])["date"].isoformat() == "2024-01-01"  # still a YAML date


# AC9 — title is the first line cut to 120 chars; text over 20,000 chars is refused


def test_title_is_first_line_cut_to_120_chars():
    long_first = "a" * 130
    note = fmt.Note(fields={}, body=f"{long_first}\nrest\n")
    assert note.title == "a" * 120
    assert fmt.Note(fields={}, body="short\nrest\n").title == "short"


def test_check_text_refuses_over_20000_chars_with_clear_error():
    fmt.check_text("x" * 20_000)
    with pytest.raises(fmt.TextTooLong, match="20,001 chars; the limit is 20,000"):
        fmt.check_text("x" * 20_001)
