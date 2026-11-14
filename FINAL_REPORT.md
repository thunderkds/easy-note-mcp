# easy-note-mcp — final plan report
**Date**: 2026-11-14 · **Mode**: autonomous (no stops; every owner-level call made and recorded)
**Verdict**: plan complete, rounds 2 and 3 included. PRD gate passed; 15 tasks (T001–T015, all v1); 4 DDRs proposed; nothing built, no repo or Linear issue created.

## What it is

A small local plug-in for the person's own notes. Notes are plain Markdown files in a folder the person picks
(default `~/easy-notes`, created by the first note). Each note gets repo, branch and task ID from git automatically.
Agents can search and read notes; only the person edits or deletes them. easy-personal can later show counts per repo
and task by reading front matter only.

## How easy is a note? (round 2)

Round 1 was **not easy enough**: in Claude Code each note cost a model turn and disturbed the task, the first note failed
until `init`, a secret made the whole note fail, the shell command was long and broke on `#` and `!`, and there was no
path from the editor or a phone. Round 2 (`BRAINSTORMING_LOG_capture.md`) fixes each:

| Where you are | What you do | Model call? |
|---|---|---|
| Claude Code | `/note retry bug on firefox` → `✓ noted … · easy-ui-mcp · T120` | No (prompt hook blocks it) |
| Codex / Cursor | `note: retry bug on firefox` | 1 turn (no hooks there) |
| Terminal | `n retry bug on firefox` | No |
| Editor / desktop | one hotkey, or click the dock icon → one-line box (`easy-note quick`) | No |
| System tray | click the easy-note tray icon → "New note" (opt-in `easy-note tray`) | No |
| Phone | save a file into `easy-notes/inbox/` with your sync app | No |

No path can lose what you typed: no setup step, and secrets are masked instead of refused.

## Claude detail (round 3)

Capture stays plain and instant. Later, you run `/mcp__easy-note__enrich_notes` in Claude Code; your Claude (one
subagent) reads each new note and, if it still exists, the session where you took it, and adds a title, kind
(bug/idea/question/todo), a short summary and suggested tags. The detail goes into a separate AI file, labelled
"not your words"; your note file is never changed. easy-note itself runs no model. Enriching at capture was rejected:
it brings back the model turn per note and can rewrite your words (`BRAINSTORMING_LOG_oneclick-enrich.md`, DDR 0004).

## Key decisions (owner may flip any; full list GRILL D1–D26)

| # | Decision | Alternative | Where |
|---|---|---|---|
| D1 | Separate repo `easy-note-mcp`, not part of easy-personal or easy-memory | Feature of easy-personal; extend easy-memory | BRAINSTORMING_LOG Options A–D |
| D2 | One Markdown file per note; files are the only store; format `easy-note: 1` is public | SQLite + export; daily files | DDR 0002, docs/NOTE_FORMAT.md |
| D6/D7 | Writes only in the notes dir + config; MCP can add and read but never edit/delete; git read-only | Full CRUD over MCP; auto-commit | DDR 0001 |
| D17/D25 | Claude Code capture by a prompt hook, installed by `easy-note install claude` (diff, backup, uninstall) | Model calls `note_add`; paste a snippet | DDR 0003 |
| D18 | Prefix `/note ` (fallback `;n ` if the spike shows the hook cannot see it) | `note:` | GRILL round 2 |
| D19–D21 | `n` command; first note needs no setup; secrets masked, never refused | `easy-note add "..."`; `init` first; refuse | PRD FR-001, FR-014, FR-020 |
| D22/D23 | `quick` one-line box for hotkeys and editors; read-only `inbox/` for phones | Capture daemon; network endpoint | PRD FR-022, FR-023 |
| D27, D28, D39 | One press = OS hotkey; one click = dock launcher or easy-note's own system tray icon (opt-in, v1) | Own key daemon; wait for easy-personal's tray | GRILL rounds 3, 3c |
| D32–D34 | Claude detail later, by your own Claude session, into separate AI files; never at capture, never inside easy-note | Enrich every note at capture; model in the server | DDR 0004 |
| D37 | MCP surface 7 tools + 1 prompt (was 5) | Keep 5 | GRILL round 3 |
| D11 | Python 3.11+, FastMCP, native `uv tool install`, stdio + HTTP on 127.0.0.1:8769 | Docker; stdlib-only | DDR 0001, PROJECT_SPEC |
| D16 | No kernel change in v1 | Kernel `note` skill now | PROJECT_SPEC follow-ups |

## Microkernel check

- Own repo, runs locally, works with no kernel and no other plug-in (NFR-008). ✅
- Kernel never depends on it; only kernel follow-up (F2) is one row in `companion-setup`. ✅
- easy-personal reads a documented file format, read-only; the optional tray item (F3) only runs `easy-note quick`. ✅
- No code imported from siblings; secret patterns re-declared. ✅
- easy-note's tray is its own; easy-personal's optional tray item (F3) would use only easy-note's public CLI. ✅
- Three flagged deviations, each with a DDR: no Docker mode (DDR 0001, like easy-personal); writes one hook entry + one launcher file (DDR 0003, like easy-memory); stores model-written detail in separate files, server still model-free (DDR 0004, like easy-personal DDR 0003).

## Biggest risks

1. The Claude Code hook: whether it sees `/note …` raw and can block it. T001 checks before T011; fallback prefix `;n `.
2. Context capture in Codex/Cursor: whether they tell the server the project folder. T001 checks; fallback is the `cwd` argument.

## Cut list (v1)

MCP edit/delete · search index or embeddings · git automation · daily journals, to-dos, reminders · connectors and any network · own web UI, key daemon, tray autostart on macOS/Windows (v1.1) · model call inside easy-note · AI edits of note files · attachments, voice, clipboard · rewriting inbox files · automatic capture from transcripts (easy-memory's job) · cloud agent sessions.

## Next step

Owner reviews D1, D2, D6/D7, D17/D18, D23, D33/D34, D37, D39. Then: create `thunderkds/easy-note-mcp`, install the kernel there with `setup.sh`, copy this folder in, and start T001 + T002 in parallel. Vital slice for "easy": T011 + T007 (`n`, zero setup) + T005 (masking), then T012 (one press), T014 (Claude detail).

## Files

`BRAINSTORMING_LOG.md` · `BRAINSTORMING_LOG_capture.md` · `BRAINSTORMING_LOG_oneclick-enrich.md` · `GRILL_requirement.md` · `PRD.md` · `PROJECT_SPEC.md` · `docs/ddr/0001-…` · `docs/ddr/0002-…` · `docs/ddr/0003-claude-code-capture-hook.md` · `docs/ddr/0004-ai-detail-in-separate-files.md` · `docs/NOTE_FORMAT.md` · `linear-tasks.md`
