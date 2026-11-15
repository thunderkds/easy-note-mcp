# PRD — easy-note-mcp
**Last updated**: 2026-11-14
**Status**: Draft (PRD gate passed in autonomous mode; owner review pending)
**Owner**: thunderkds

> **Scope of this document**: *What* to build and *why* — product intent, user stories, requirements, success metrics.
> Technical decisions, architecture, agent config, and task state live in `PROJECT_SPEC.md`.
> Source of intent: `BRAINSTORMING_LOG.md` (2026-11-14, Option B) and `GRILL_requirement.md` (G1–G18, D1–D16).

---

## Overview

A person who works with coding agents all day has many small thoughts: a bug seen in passing, a question
for later, an idea for another repo. Writing them down means leaving the chat, opening some app, and losing
the context (which repo, which branch, which task). Most thoughts are never written down.

`easy-note-mcp` is the Easy Kit plug-in for the person's own notes. One line in any agent chat
("note: retry bug only on Firefox") or in the shell (`easy-note add "..."`) saves a note. The note is a plain
Markdown file in a folder the person picks, tagged automatically with repo, branch and task ID. Agents can
search and read notes; only the person edits or deletes them. Everything stays on the person's machine.

---

## Personas

| ID | Name | Role | Pain Point |
|----|------|------|-----------|
| P1 | Solo Builder | The owner, working in Claude Code, Codex and Cursor across many repos | Loses small thoughts; notes that exist have no repo/task context; no single place across repos. |
| P2 | Agent Session | Any agent the person works with (Supervisor or plain session) | Cannot save "note this" for the person, and cannot answer "what did I note about X" without guessing. |
| P3 | easy-personal | The family's personal dashboard (a reader, not a user) | Wants note counts and links per repo/task without storing any note text (its NFR-002). |

---

## User Stories

| ID | Story | Persona | Phase |
|----|-------|---------|-------|
| US-001 | As P1, I want to save a note in one line from any agent chat, so I never leave my flow. | P1, P2 | v1 |
| US-002 | As P1, I want each note tagged with repo, branch and task ID automatically, so I find it later in context. | P1 | v1 |
| US-003 | As P1, I want to save a note from the shell in one command, so capture works without an agent. | P1 | v1 |
| US-004 | As P1, I want notes as plain Markdown files in a folder I pick (git repo, Dropbox, Obsidian vault), so I own them and can edit them anywhere. | P1 | v1 |
| US-005 | As P1, I want to list and search notes by words, repo, task, tag and date, so I find a note fast. | P1 | v1 |
| US-006 | As P1, I want to delete a note and undo it, so cleanup is safe. | P1 | v1 |
| US-007 | As P1, I want text that looks like a secret masked before it is saved, so a synced notes folder never leaks a key and my note is still kept. | P1 | v1 |
| US-008 | As P2, I want to search and read the person's notes, so I answer "what did I note about X" with facts. | P2 | v1 |
| US-009 | As P3, I want note counts per repo and task from a documented format, so I show them without storing note text. | P3 | v1 (format) / easy-personal board (reader) |
| US-010 | As P1, I want my notes in Notion or another tool. | P1 | Later (separate brainstorm) |
| US-011 | As P1, I want one hotkey from my editor or desktop to open a one-line box, so I note mid-task without switching to a terminal. | P1 | v1 |
| US-012 | As P1, I want to drop a note from my phone into my synced notes folder, so thoughts away from the desk are kept. | P1 | v1 |
| US-014 | As P1, I want one key press or one click from anywhere on my desktop to note, so I never open a terminal. | P1 | v1 |
| US-015 | As P1, I want an easy-note icon in my system tray with "New note", so a note is a click away. | P1 | v1 (round 3c) |
| US-016 | As P1, I want Claude to add a clear title and the missing detail to my one-line notes later, without changing my words, so a note still makes sense next month. | P1, P2 | v1 |
| US-013 | As P1, I want a note typed in Claude Code to be saved without the agent seeing it, so the task is not disturbed and no tokens are spent. | P1 | v1 |

---

## Functional Requirements

| ID | Requirement | Traces to |
|----|-------------|-----------|
| FR-001 | `easy-note init [--dir PATH]` must create the notes dir and write `~/.config/easy-note/config.toml`; `EASY_NOTE_DIR` overrides the config. With no dir configured, the first write must use `~/easy-notes`, create it, and print its path once; it must never refuse for lack of setup (round 2, D20). | US-004 |
| FR-002 | A note must be one file `<dir>/YYYY/MM/<id>.md`: YAML front matter (format in `docs/NOTE_FORMAT.md`, `easy-note: 1`) + Markdown body. Title = first line (≤ 120 chars). Text ≤ 20,000 chars. | US-004 |
| FR-003 | Note ID must be `YYYYMMDD-HHMMSS-xxxx` (local time; 4 hex of sha256(created ns, hostname, text)). A clash on write must add `-2`, `-3`…; a write never overwrites a file. | US-001, US-003 |
| FR-004 | Context capture must record `repo` (folder name of the main worktree), `remote` (`owner/name`, userinfo stripped, only when origin exists) and `branch`, read with read-only git commands. The working directory comes from, in order: explicit `cwd` arg → MCP client roots → server process cwd. Outside a git repo the fields are absent, never guessed. Paths are never stored. | US-002 |
| FR-005 | `task` must come from the branch name by declared patterns (`T\d{3,4}`, `[A-Z][A-Z0-9]+-\d+`, case-insensitive for `T`, stored upper-case); an explicit task overrides; no match → absent. | US-002 |
| FR-006 | Tags must be the union of the explicit `tags` arg and `#word` tokens in the text, normalized lower-case `[a-z0-9_/-]+`. | US-002, US-005 |
| FR-007 | Every note must record `source` (`cli`, `mcp`, `hook`, `quick` or `inbox`) and, for MCP, `client` (client name from the MCP initialize handshake, if sent). | US-001 |
| FR-008 | Writes must be atomic (temp file in the same dir + rename). Notes are never rewritten by the server except by `easy-note edit` / `easy-note tag`. | US-004 |
| FR-009 | `easy-note rm <id>` must move the file to `<dir>/.easy-note/trash/`; `easy-note restore <id>` must move it back; `easy-note trash --empty --older-than 30d` deletes for good, only on that explicit command. | US-006 |
| FR-010 | `list` must filter by `repo`, `task`, `tag`, `source`, `since`, `until`, `day`, newest first, using local dates. | US-005 |
| FR-011 | `search` must match all query words (case-insensitive) in title, body or tags, with the same filters as FR-010. | US-005, US-008 |
| FR-012 | MCP tools (v1, frozen after approval): `note_add`, `note_search`, `note_list`, `note_read`, `note_stats`, and from round 3 `note_pending_enrich`, `note_enrich` (writes AI files only); prompt `enrich_notes`. No MCP tool may edit, delete, restore or move a note, or change config. | US-001, US-008 |
| FR-013 | CLI commands: `init`, `add`, `ls`, `search`, `show`, `edit` (opens `$EDITOR`, then re-validates), `tag`, `rm`, `restore`, `trash`, `stats`, `doctor` (scan for skipped files and secrets, read-only), `path`, `serve` (MCP), from round 2 `quick`, `hook-prompt`, `install claude`, `uninstall claude`, and from round 3 `hotkey`, `install|uninstall launcher`, `ai rm`, `tray`, `install|uninstall tray`. CLI and MCP must share one core. | US-003, US-005, US-006 |
| FR-014 | Secret guard: every write path (`add`, `n`, `quick`, hook, `edit`, `note_add`) must replace each match of the declared secret patterns (`core/guard.py`) with `[masked:<kind>]` before the write and report `N secret(s) masked` with the kinds, never echoing the match. Every read output (CLI and MCP) must mask matches in hand-edited and inbox files the same way (round 2, D21). | US-007 |
| FR-015 | `note_stats` / `easy-note stats --json` must return counts by repo, task, tag, source and month, plus the oldest and newest note date. No text. | US-008, US-009 |
| FR-016 | MCP list/search responses must hold ≤ 50 items, each with id, title, created, repo, task, tags and a ≤ 200-char snippet, plus `total` and `truncated`. | US-008 |
| FR-018 | *(round 2)* `easy-note hook-prompt` must read a Claude Code `UserPromptSubmit` event on stdin. A prompt that starts with the capture prefix (default `/note `, configurable, D18) must be saved with `source: hook` and context from the event's `cwd`, and must return `{"decision":"block","reason":"✓ noted <id> · <repo> · <task>"}`. Any other prompt must produce no output and exit 0. On any error it must let the prompt through and print the error as the reason only when the prefix matched. | US-001, US-013 |
| FR-019 | *(round 2)* `easy-note install claude` must add exactly one `UserPromptSubmit` hook entry to user-level Claude Code settings and write `~/.claude/commands/note.md` (fallback), after showing the diff and keeping a timestamped backup; `uninstall claude` must remove exactly those. No other key changes (DDR 0003). | US-001 |
| FR-020 | *(round 2)* Console script `n` must equal `easy-note add`. `add` must join all arguments with spaces as the note text; with no arguments in a TTY it must read one line from a prompt; `-e` must open `$EDITOR`. | US-003 |
| FR-021 | *(round 2)* Every capture path must print one confirmation line: id, repo (if any), task (if any), masked count (if any). | US-001, US-003 |
| FR-022 | *(round 2)* `easy-note quick [--cwd DIR]` must open a native one-line input (`osascript` on macOS, `zenity` or `kdialog` on Linux), else a `tkinter` box, else a terminal prompt; save with `source: quick`; show the confirmation as a desktop notification where available. Empty input or cancel saves nothing. | US-011 |
| FR-023 | *(round 2)* Every `.md` or `.txt` file directly in `<notes dir>/inbox/` must be read as a note (`source: inbox`, id `inbox/<file stem>`, created from a `YYYYMMDD-HHMMSS` file-name prefix else mtime, tags from `#words`). easy-note must never write, rename or normalize inbox files; `rm` moves them to trash. | US-012 |
| FR-024 | *(round 2)* The `note_add` description must tell the model: when the person's message starts with `note:` or `/note`, call it at once with the rest of the message and reply in one line. | US-001 |
| FR-025 | *(round 3)* `easy-note hotkey` must detect the desktop (GNOME, KDE, macOS, Windows) and print the exact commands to bind a key to `easy-note quick`; it writes nothing. | US-014 |
| FR-026 | *(round 3)* `easy-note install launcher` / `uninstall launcher` must write / remove exactly `~/.local/share/applications/easy-note.desktop` (Linux) running `easy-note quick` (DDR 0003). | US-014 |
| FR-027 | *(round 3)* `quick` must pre-fill the last context (newest `cli` or `hook` note < 2 h old), show it in the box, and drop it on one key; the box must be visible in < 300 ms (NFR-011). | US-014 |
| FR-028 | *(round 3)* Hook notes must store `session` (event `session_id`); notes captured in a git repo must store `commit` (short HEAD sha). | US-016 |
| FR-029 | *(round 3)* MCP tool `note_pending_enrich(limit≤20)` must list notes with no AI file or a stale one (`based_on` ≠ sha256 of the note text), with id, text, created, repo, task, session. | US-016 |
| FR-030 | *(round 3)* MCP tool `note_enrich(id, title, kind, summary, tags_suggested, model)` must write only `<notes dir>/.easy-note/ai/<id>.json` with the caps of D34, secrets masked, `based_on` and `at` set by the server; it must refuse an unknown id and must never open a note file for writing. | US-016 |
| FR-031 | *(round 3)* MCP prompt `enrich_notes` (fixed text in code) must ask the client's model to use one subagent, call `note_pending_enrich`, read the session transcript only when `session` is set and the file exists, explain without quoting, and call `note_enrich` per note. The server runs no model. *Round 3b:* read only the transcript entries from the 10 minutes before the note's `created` (at most 40 entries); for a note with no `session` but a `repo` (terminal, `quick`), look for a Claude Code transcript of that repo with entries in that window and use it if exactly one matches; else enrich from the note text, repo, branch and task only and set `kind` with no context claim. | US-016 |
| FR-032 | *(round 3)* Read outputs must show AI detail labelled `AI detail (not your words)` with model and date; search must match it and mark `matched_in: ai`; `easy-note tag <id> --accept-ai` copies suggested tags into the note; `easy-note ai rm <id>` deletes the AI file; `rm`/`restore` move the AI file with its note. `note_add` may take an optional `detail` (≤ 600 chars) stored as an AI file with `model: capture`. | US-016 |
| FR-033 | *(round 3c)* `easy-note tray` (optional extra `[tray]`) must show a system tray icon with menu: New note (runs `quick`), the 5 newest notes (open the file), Open notes folder, Quit. `install tray` / `uninstall tray` write / remove exactly `~/.config/autostart/easy-note-tray.desktop` (Linux). With no tray host, `tray` exits with the hotkey hint and `doctor` reports it. | US-015 |
| FR-017 | Files the parser cannot read (bad front matter, unknown `easy-note` version) must be counted as `skipped` with the path in `stats` and `doctor`, never changed and never hidden silently. | US-004 |

---

## Non-Functional Requirements

| ID | Requirement | Category |
|----|-------------|----------|
| NFR-001 | No traffic leaves the machine; HTTP mode binds `127.0.0.1` only. Subprocesses: read-only `git` (`rev-parse`, `branch --show-current`, `remote get-url`), in `quick` only, the OS input/notification tool (`osascript`, `zenity`, `kdialog`, `notify-send`), and in `tray` only, the OS opener (`xdg-open`, `open`). | Security |
| NFR-002 | No secret appears in any note file written by the tool, any response, or any log (fixture with planted secrets proves it). Logs never contain note text. | Security / Privacy |
| NFR-003 | Writes only inside the notes dir, the config file, the one hook entry + command file owned by `install claude`, the launcher file owned by `install launcher` and the autostart file owned by `install tray` (DDR 0001, DDR 0003). Nothing in any code repo is ever written. | Safety |
| NFR-004 | Search p95 < 500 ms and `note_add` p95 < 150 ms over 10,000 notes on a laptop (provisional until T010). | Performance |
| NFR-005 | Safe under a sync tool: two machines adding notes at the same time never lose a note (per-file writes, unique IDs). | Reliability |
| NFR-006 | Python 3.11+, native install `uv tool install easy-note-mcp`; macOS and Linux in v1; Windows best effort. | Portability |
| NFR-007 | Deterministic: same files + same query → same output order; no LLM in the server. | Correctness |
| NFR-008 | Works with no kernel installed, and with no other plug-in installed. | Microkernel |
| NFR-009 | *(round 2)* Hook pass path (prompt without the prefix) p95 < 50 ms, stdlib imports only; hook save path p95 < 150 ms. | Performance |
| NFR-011 | *(round 3)* `quick` box visible p95 < 300 ms after the key press. | Performance |
| NFR-012 | *(round 3)* AI detail never changes a note file and never feeds a count other than `enriched`. | Correctness |
| NFR-010 | *(round 2)* `n` / `easy-note add` cold start to confirmation p95 < 150 ms; the CLI never imports the MCP SDK. | Performance |

---

## Success Metrics / KPIs

| Metric | Baseline | Target | How measured |
|--------|----------|--------|--------------|
| Notes captured per working week | ~0 (thoughts lost) | ≥ 15 | `easy-note stats` by week |
| Notes with a repo tag (captured inside a repo) | n/a | ≥ 90 % | `stats`: notes with `repo` / notes with `source: mcp` |
| Time to capture | open an app, type, save | one prefix or hotkey, < 5 s, confirmation < 1 s | Self-reported + NFR-009/010 timings |
| Model turns spent on notes in Claude Code | 1 per note | 0 | `source: hook` share of Claude Code notes |
| Notes lost at capture (refused, setup error) | n/a | 0 | No refusal path exists (FR-001, FR-014) |
| Notes with AI detail after a weekly `enrich_notes` run | n/a | ≥ 90 % | `stats.enriched` |
| Secrets in notes dir | unknown | 0 | Leak suite + `easy-note doctor` scan |
| Skipped (unreadable) files | n/a | 0 written by the tool | `stats.skipped` |

---

## Out of Scope (v1)

- Cloud agent sessions (claude.ai/code, mobile): they cannot reach a local MCP server.
- Connectors and any network call (Notion, Apple Notes, Google Keep); Obsidian needs no code.
- Edit/delete/restore over MCP.
- Search index or embeddings. Model-written text inside a note file (AI detail lives only in separate AI files, FR-030, DDR 0004).
- Git commit/push of the notes dir.
- Daily-journal files, templates, to-dos, reminders, due dates.
- Attachments and images.
- Automatic capture from transcripts (easy-memory's job). The round 2 prompt hook captures only what the person types after the prefix.
- A key-listener daemon; macOS/Windows tray autostart (v1.1); a model call inside easy-note; AI edits of note files; network capture from a phone (the `inbox/` folder plus the person's sync tool covers it).
- Zero-model-call capture in Codex/Cursor (no hook system there).
- A web UI (easy-personal shows counts and links; reader built on its own board, follow-up F1).
- Kernel skills or hooks for notes (follow-up F2 only lists easy-note in `companion-setup`).

---

## Open Questions / Assumptions

| # | Question / Assumption | Owner | Due |
|---|----------------------|-------|-----|
| 1 | Assumed: Claude Code, Codex and Cursor start stdio servers with a cwd or roots that point at the project. Verify per client. | Supervisor | T001, before T004 |
| 2 | Default notes dir `~/easy-notes` (D13) | thunderkds | before T002 |
| 3 | Repo name `easy-note-mcp`, server id `easy-note` (D1) | thunderkds | before repo creation |
| 4 | NFR-004 targets are provisional | Supervisor | T010 |
| 6 | Owner approves the MCP surface growing from 5 to 7 tools + 1 prompt (D37) | thunderkds | before T008 |
| 7 | Token cost per enriched note (estimate 1–3k) | Supervisor | T014 |
| 5 | Does the `UserPromptSubmit` hook see `/note ...` raw, can it block, and does it run for a prompt queued while the agent works? | Supervisor | T001, before T011 |
