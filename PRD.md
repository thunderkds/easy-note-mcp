# PRD — easy-note-mcp
**Last updated**: 2026-11-25 (round 6: AI organizer, scheduler, settings, UX — see GRILL_round6.md)
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
| US-017 | As P1, I want my notes dir to be my Obsidian "brain": one click opens a dashboard of my notes by day, project, task and tag, and the AI detail shows there too, so I browse and link notes in the app I already use. | P1 | v1 (round 4) |
| US-018 | As P1, I want easy-note to ask my own Claude subscription (the local `claude` CLI, the account I pick) to write the AI detail, so I do not have to start an `enrich_notes` run myself. | P1 | v1 (round 4) |
| US-019 | As P1, I want one click to open easy-note's own notes board, the one in the design (counts, search, kind filters, notes by day, the AI detail beside each note), so I see and handle my notes without another app. | P1 | v1 (round 5) |
| US-020 | As P1, I want to only take notes and have the AI arrange them (project, topic, kind, drafts, summaries) on a schedule, so I never organize by hand. | P1 | v1 (round 6) |
| US-021 | As P1, I want to fix an AI guess once and have it stay fixed, so I trust the arrangement. | P1 | v1 (round 6) |
| US-022 | As P2, I want one call that returns the digest and notes of a project or topic, so I answer with the person's own context. | P2 | v1 (round 6) |
| US-023 | As P1, I want one dashboard window, the time each note was last edited, and a settings screen, so the app is clear and calm to use. | P1 | v1 (round 6) |
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
| FR-012 | MCP tools (v1, frozen after approval): `note_add`, `note_search`, `note_list`, `note_read`, `note_stats`, and from round 3 `note_pending_enrich`, `note_enrich` (writes AI files only); prompt `enrich_notes`. No MCP tool may edit, delete, restore or move a note, or change config. *(round 6)* Plus read-only `note_context` (FR-051): 8 tools. | US-001, US-008 |
| FR-013 | CLI commands: `init`, `add`, `ls`, `search`, `show`, `edit` (opens `$EDITOR`, then re-validates), `tag`, `rm`, `restore`, `trash`, `stats`, `doctor` (scan for skipped files and secrets, read-only), `path`, `serve` (MCP), from round 2 `quick`, `hook-prompt`, `install claude`, `uninstall claude`, and from round 3 `hotkey`, `install|uninstall launcher`, `ai rm`, `tray`, `install|uninstall tray`. CLI and MCP must share one core. | US-003, US-005, US-006 |
| FR-014 | Secret guard: every write path (`add`, `n`, `quick`, hook, `edit`, `note_add`) must replace each match of the declared secret patterns (`core/guard.py`) with `[masked:<kind>]` before the write and report `N secret(s) masked` with the kinds, never echoing the match. Every read output (CLI and MCP) must mask matches in hand-edited and inbox files the same way (round 2, D21). | US-007 |
| FR-015 | `note_stats` / `easy-note stats --json` must return counts by repo, task, tag, source and month, plus the oldest and newest note date. No text. | US-008, US-009 |
| FR-016 | MCP list/search responses must hold ≤ 50 items, each with id, title, created, repo, task, tags and a ≤ 200-char snippet, plus `total` and `truncated`. | US-008 |
| FR-018 | *(round 2)* `easy-note hook-prompt` must read a Claude Code `UserPromptSubmit` event on stdin. A prompt that starts with the capture prefix (default `/note `, configurable, D18) must be saved with `source: hook` and context from the event's `cwd`, and must return `{"decision":"block","reason":"✓ noted <id> · <repo> · <task>"}`. Any other prompt must produce no output and exit 0. On any error it must let the prompt through and print the error as the reason only when the prefix matched. | US-001, US-013 |
| FR-019 | *(round 2)* `easy-note install claude` must add exactly one `UserPromptSubmit` hook entry to `settings.json` and write `commands/note.md` (fallback), both in the Claude Code config dir (`$CLAUDE_CONFIG_DIR` when set, else `~/.claude`) (HUN-251 finding), after showing the diff and keeping a timestamped backup; `uninstall claude` must remove exactly those. No other key changes (DDR 0003). | US-001 |
| FR-020 | *(round 2)* Console script `n` must equal `easy-note add`. `add` must join all arguments with spaces as the note text; with no arguments in a TTY it must read one line from a prompt; `-e` must open `$EDITOR`. | US-003 |
| FR-021 | *(round 2)* Every capture path must print one confirmation line: id, repo (if any), task (if any), masked count (if any). | US-001, US-003 |
| FR-022 | *(round 2)* `easy-note quick [--cwd DIR]` must open a small input box: in a `window` build with a display, easy-note's own capture window (the design's capture dialog, HUN-289); otherwise a native input (`osascript` on macOS, `zenity` or `kdialog` on Linux), else a `tkinter` box, else a terminal prompt; save with `source: quick`; show the confirmation as a desktop notification where available. Empty input or cancel saves nothing. | US-011 |
| FR-023 | *(round 2)* Every `.md` or `.txt` file directly in `<notes dir>/inbox/` must be read as a note (`source: inbox`, id `inbox/<file stem>`, created from a `YYYYMMDD-HHMMSS` file-name prefix else mtime, tags from `#words`). easy-note must never write, rename or normalize inbox files; `rm` moves them to trash. | US-012 |
| FR-024 | *(round 2)* The `note_add` description must tell the model: when the person's message starts with `note:` or `/note`, call it at once with the rest of the message and reply in one line. | US-001 |
| FR-025 | *(round 3)* `easy-note hotkey` must detect the desktop (GNOME, KDE, macOS, Windows) and print the exact commands to bind a key to `easy-note quick`; it writes nothing. | US-014 |
| FR-026 | *(round 3)* `easy-note install launcher` / `uninstall launcher` must write / remove exactly `~/.local/share/applications/easy-note.desktop` (Linux) running `easy-note quick` (DDR 0003). | US-014 |
| FR-027 | *(round 3)* `quick` must pre-fill the last context (newest `cli` or `hook` note < 2 h old), show it in the box, and drop it on one key (Backspace in an empty box in the window; a leading `- ` in the native box); the box must be visible in < 300 ms (NFR-011). | US-014 |
| FR-028 | *(round 3)* Hook notes must store `session` (event `session_id`); notes captured in a git repo must store `commit` (short HEAD sha). | US-016 |
| FR-029 | *(round 3)* MCP tool `note_pending_enrich(limit≤20)` must list notes with no AI file or a stale one (`based_on` ≠ sha256 of the note text), with id, text, created, repo, task, session. | US-016 |
| FR-030 | *(round 3)* MCP tool `note_enrich(id, title, kind, summary, tags_suggested, model)` must write only `<notes dir>/.easy-note/ai/<id>.json` (and, from round 6, its lines in `ai-log.jsonl`, FR-050) with the caps of D34, secrets masked, `based_on` and `at` set by the server; it must refuse an unknown id and must never open a note file for writing. | US-016 |
| FR-031 | *(round 3)* MCP prompt `enrich_notes` (fixed text in code) must ask the client's model to use one subagent, call `note_pending_enrich`, read the session transcript only when `session` is set and the file exists, explain without quoting, and call `note_enrich` per note. The server runs no model. *Round 3b:* read only the transcript entries from the 10 minutes before the note's `created` (at most 40 entries); for a note with no `session` but a `repo` (terminal, `quick`), look for a Claude Code transcript of that repo with entries in that window and use it if exactly one matches; else enrich from the note text, repo, branch and task only and set `kind` with no context claim. | US-016 |
| FR-032 | *(round 3)* Read outputs must show AI detail labelled `AI detail (not your words)` with model and date; search must match it and mark `matched_in: ai`; `easy-note tag <id> --accept-ai` copies suggested tags into the note; `easy-note ai rm <id>` deletes the AI file; `rm`/`restore` move the AI file with its note. `note_add` may take an optional `detail` (≤ 600 chars) stored as an AI file with `model: capture`. | US-016 |
| FR-033 | *(round 3c)* `easy-note tray` (optional extra `[tray]`) must show a system tray icon with menu: New note (runs `quick`), the 5 newest notes (open the file), Open notes folder, Quit. `install tray` / `uninstall tray` write / remove exactly `~/.config/autostart/easy-note-tray.desktop` (Linux). With no tray host, `tray` exits with the hotkey hint and `doctor` reports it. *(HUN-286)* A recent-note click opens `easy-note dashboard --note <id>` in a build with the `window` feature and a display; otherwise it opens the file with `xdg-open`. *(HUN-285)* At start `tray` checks for the host every 1 s for up to 60 s before it gives up (at login the panel can start after the autostart entry); in a build with `tray` and `window`, the dashboard window starts `easy-note tray` when it opens, and a tray that already runs exits on the lock. | US-015 |
| FR-034 | *(round 4)* `easy-note dashboard` must open the notes dashboard in Obsidian with `xdg-open "obsidian://open?path=<notes dir>/easy-note.base"` (argv list, no shell). When `<notes dir>/easy-note.base` is missing it says to run `easy-note install obsidian`, exit 1; when no `obsidian://` handler is registered (`xdg-mime query default x-scheme-handler/obsidian` is empty) it says Obsidian is not installed and opens the notes dir with `xdg-open` instead (DDR 0006). | US-017 |
| FR-035 | *(round 4)* `easy-note install obsidian` / `uninstall obsidian` must write / remove exactly `<notes dir>/easy-note.base` (an Obsidian Bases file: views over `YYYY/MM` notes by day, project, task, tag and source), with the DDR 0003 rules: diff shown, a second run writes nothing, a file that is not ours is kept and named. The file is not a note (NOTE_FORMAT: only `YYYY/MM` files are notes). | US-017 |
| FR-036 | *(round 4)* When the notes dir is inside an Obsidian vault (a `.obsidian` folder in the notes dir or a parent), every AI file write must also write `<notes dir>/ai/<id>.md`: front matter `easy-note-ai-view: 1`, `note: "[[<id>]]"`, `model`, `at`, `kind`, `stale`; a body headed `AI detail (not your words)`; every string masked. It is a view of the JSON AI file, never a note and never read back. `easy-note ai render` rewrites all views; `rm` / `restore` / `trash --empty` move or delete the view with its AI file (DDR 0006). | US-017, US-016 |
| FR-037 | *(round 4)* `easy-note ai enrich <id>…` / `--pending [--limit N]` must, only when `[ai] enrich = "claude"` in config.toml, run the person's own Claude Code CLI: `claude -p --output-format json --json-schema <AI detail schema> --tools "" --strict-mcp-config --no-session-persistence --model <model>`, note text (masked) on stdin, `CLAUDE_CONFIG_DIR` from `[ai] claude_config_dir` when set, `ANTHROPIC_API_KEY` and `ANTHROPIC_AUTH_TOKEN` removed from the child environment, a timeout; the answer is written through the `note_enrich` writer (caps, kinds, masking, `based_on`), `model` = the model the CLI reports. Off by default: with `enrich` unset or `"off"` the command says how to turn it on, exit 1, starts nothing (DDR 0007). | US-018 |
| FR-038 | *(round 4)* config.toml `[ai]` section: `enrich` (`"off"` \| `"claude"`, default `"off"`), `command` (default `"claude"`), `claude_config_dir` (optional, `~` expanded), `model` (default `"haiku"`), `timeout_secs` (default 120), `after_save` (default `false`: when `true`, a note saved by `add`/`quick`/`hook`/MCP starts `ai enrich <id>` as a detached child and never waits for it). `easy-note doctor` reports the section, whether `command` is on `PATH`, and the account folder in use. | US-018 |
| FR-039 | *(round 4)* App and tray icons from the design (`docs/design/` copy of the Graphite Mint canvas): `install launcher` also writes `easy-note.svg` to `~/.local/share/icons/hicolor/scalable/apps/` and the 16 px variant to `hicolor/16x16/apps/`, and `easy-note-symbolic.svg` plus `easy-note-saved-symbolic.svg` (the check badge) to `hicolor/symbolic/apps/`; the launcher gets `Icon=easy-note`, `Exec=easy-note dashboard` and a `New note` desktop action running `easy-note quick`. The tray uses `easy-note-symbolic` (theme path set to the installed folder), adds `Open dashboard` under `New note`, and shows the check badge for 3 s after a note is saved. `uninstall launcher` removes exactly the files it wrote. | US-014, US-015, US-017 |
| FR-040 | *(round 5)* `easy-note install obsidian` must also make the notes dir an Obsidian vault: create `<notes dir>/.obsidian/` and add the notes dir to Obsidian's vault list (`$XDG_CONFIG_HOME/obsidian/obsidian.json`, else `~/.config/obsidian/obsidian.json`) under a new random id, keeping every other vault and key, with a backup. Nothing is written when a vault already is the notes dir or a parent of it, when the file is not valid JSON (named, exit 1), or when Obsidian is running (it says to close Obsidian and run again, exit 1). `uninstall obsidian` never edits the vault list (DDR 0008). | US-017 |
| FR-041 | *(round 5)* `easy-note setup` must run `install launcher`, `install obsidian`, in a tray build `install tray`, and `install hotkey` (FR-054, added 2026-11-26), each under its own heading; a failed step does not stop the next; exit 1 if any failed. A second run writes nothing. It never runs `install claude`; its last line suggests it. | US-017 |
| FR-042 | *(round 5)* In a build with the `window` feature and a display (`WAYLAND_DISPLAY` or `DISPLAY` set), `easy-note dashboard` must open easy-note's own window showing the design's board (`docs/design/Main.dc.html`, board "Dashboard (clickable)"), read-only: header with the notes dir, counts strip (notes today, notes in total, projects, top 3 projects as bars), search (every word must match the text, project, task or a tag), kind filters with counts (All, Bugs, Ideas, Questions, To-dos, No AI detail, Stale), notes grouped by local day (today, yesterday, earlier) with time, first line, a second line (see HUN-305 below) and AI / stale mark, and a detail panel (id, source, the person's words, masked notice, context, tags, AI detail labelled `AI detail (not your words)` with model, date, stale warning, title, kind, summary, suggested tags). The page is embedded in the binary, loads nothing from the network (fonts bundled), reaches the notes only through IPC, and every string it gets is masked. It refreshes when the window gets focus. Without the feature or a display, FR-034 is unchanged (DDR 0009). *(2026-11-27, HUN-304)* Layout: the content is a centred column at most 1440 px wide; from 760 x 480 px up the list and the detail panel sit side by side; the window opens no larger than the screen, and its minimum size shrinks with it. *(2026-11-27, HUN-305)* The window is named `Easy Taking Shot` (the command stays `easy-note`). In the side-by-side range the page does not scroll: the list and the detail panel each fill the window height and scroll on their own, and the panel's buttons stay in view. A list row shows the time, the first line (at most two lines), `edited <when>`, the project and the AI / stale mark; it no longer shows the task, the tags or the kind chip. | US-019, US-017 |
| FR-043 | *(round 5)* The window's detail panel must offer `Add as tags` (same effect as `easy-note tag <id> --accept-ai`), `Move to trash` (same as `rm`, AI file moves with it) with an Undo toast for 6 s (same as `restore`), `Edit` *(HUN-286, replaces `Open in editor`)*: the note's text and tags are edited in the panel and saved through the same core path as `easy-note edit` (masking, re-validate, `updated` stamp); a save is refused when the note changed outside the window since it was loaded; inbox notes stay read-only. `Run AI` *(HUN-288)*: runs the same AI step as `easy-note ai enrich <id>` on that note in the background (label `Re-run` when AI detail exists, one run per note, errors shown as a toast, nothing written if the note changed while it ran; the click is the opt-in, so no `[ai] enrich` setting is needed). And `Copy id`. The window opens no note in another app. Each action goes through the same core function as the CLI; nothing else changes a note file (DDR 0009). | US-019 |
| FR-044 | *(round 5)* The window's `+ New note` must open the design's capture dialog: Enter saves (same path as `easy-note add`, context from the last note as in FR-027), Shift+Enter adds a newline, Esc cancels, Backspace in an empty box drops the context chip; the new note appears in the list at once (DDR 0009). | US-019, US-014 |
| FR-045 | *(round 6)* `easy-note dashboard` must keep one window per notes dir: a second call (tray click, launcher, CLI) must focus the open window (and show the note given by `--note <id>`) and start no second window. The lock is `<notes dir>/.easy-note/window.lock`; a stale lock (dead process) never blocks. | US-023 |
| FR-046 | *(round 6)* The list output (`ls`, `search`, MCP list/search) and the window must show `updated` beside `created` when the note has it, in local time (`edited 2h ago` in the window, full stamp in the detail). AI runs never set `updated`; only `edit`, `tag` and the window's Edit and Add as tags do (NFR-012). | US-023 |
| FR-047 | *(round 6, Path B)* `easy-note ai run [--limit N] [--dry-run]` must pick notes with no AI file or a stale one (as FR-029), at most `[ai] max_per_run` (default 20), split them into batches of `[ai] batch_size` (default 10), and for each batch make ONE call to the person's `claude` CLI (FR-037 rules). The prompt holds the current project and topic lists (names and counts, at most 50) and the batch's notes (masked, each cut to 1,500 chars, at most 12,000 chars per call). The answer is a JSON object whose `items` array holds one item per note id: title, kind (`idea`, `todo`, `bug`, `question`, `draft`, `other`), summary, `project`, `topic`, suggested tags and, for a draft, `draft_reason` and `draft_suggestion`. The server checks each item alone (known id, caps, masking, `based_on` unchanged); a bad item leaves only that note pending, and a failed call leaves the whole batch pending. Results go into AI files only (caps and masking as FR-030). `project` is the note's `repo` when it has one; otherwise the model names it. A field with a person's override (FR-050) keeps the override. Same opt-in as FR-037; it never opens a note file for writing. Each applied item is one `ai-log.jsonl` line, so Undo works per note. | US-020 |
| FR-048 | *(round 6)* config.toml `[ai]` gains `schedule` (`"off"` \| `"tray"`, default `"off"`), `interval_min` (default 60, min 5), `max_per_run` (default 20), `batch_size` (default 10), `jobs` (list of `organize`, `drafts`, `digest`; default all three), `digest_hour` (default 3). `schedule = "tray"` is set only from the Settings switch "Organize automatically", which shows once what is sent (masked note text, own `claude` CLI, account dir, model); it is the person's opt-in for unattended runs. `easy-note doctor` reports the section. | US-020 |
| FR-049 | *(round 6)* The tray must run `easy-note ai run` as a detached child every `interval_min` while `schedule = "tray"`, never two at once (lock `<notes dir>/.easy-note/ai-run.lock`), and the digest job once a day at `digest_hour`. A failed run backs off (interval doubles to a 4 h cap) and shows one line in Settings. The same command can be run from cron or a systemd timer. | US-020 |
| FR-050 | *(round 6)* A person's fix of project, topic or kind (window or `easy-note ai set <id> project|topic|kind VALUE`) must go to `<notes dir>/.easy-note/ai-overrides.json`, never to a note or AI file. The next `ai run` must keep it. Renaming or merging a topic in the window rewrites the override for all notes of that topic. Every AI change and override goes to `<notes dir>/.easy-note/ai-log.jsonl` (id, field, old, new, who, at); `easy-note ai undo <id>` and the window's Undo restore the previous value. | US-021 |
| FR-051 | *(round 6)* The `digest` job must write `<notes dir>/.easy-note/ai/digest/<project|topic>.json` (summary of at most 1,200 chars, `based_on` hash of the member notes, `at`, masked). MCP tool `note_context(project?, topic?, limit≤10)` must be read-only and return the digest plus up to 10 notes with the FR-016 item shape (≤ 200-char snippet) and `total`, `truncated`; with neither argument it returns the project and topic lists with counts. FR-012 is amended: tools are now 8. | US-022 |
| FR-052 | *(round 6)* The window must add: group-by switch (Day, Project, Topic, Kind), a Drafts view (kind `draft`, showing `draft_reason` and `draft_suggestion` labelled `AI detail (not your words)`; the person finishes the note with Edit), topic rename and merge, an AI status line per note (`organized 2h ago`, `waiting`, `not organized`) and in the header the waiting count with `Run now`, and a Settings screen (gear icon) for `[ai]`: `enrich`, `schedule`, `interval_min`, `max_per_run`, `jobs`, `model`, `claude_config_dir`, `after_save`. Settings write config.toml through one core function; nothing else in the file changes. | US-020, US-021, US-023 |
| FR-053 | *(round 6)* UX pass: keyboard keys in the window (`/` search, `j`/`k` move, `e` edit, `n` new note, `?` key list), a first-run empty state, and every change checked with `ux-evaluate` at 375 px and 1280 px. | US-023 |
| FR-054 | *(2026-11-26, owner: chat "yes", HUN-303)* `easy-note install hotkey` must, on GNOME only, add one custom shortcut (`.../custom-keybindings/easy-note/`: command `easy-note quick`, binding `<Control><Alt>n`) with the person's `gsettings`. Every other entry stays. It writes nothing when a shortcut already runs `easy-note quick`, when the key is in use, or when that path holds another command. `uninstall hotkey` removes only that entry, and only while it runs `easy-note quick`. On another desktop both print how to find the steps and run nothing. | US-011 |
| FR-017 | Files the parser cannot read (bad front matter, unknown `easy-note` version) must be counted as `skipped` with the path in `stats` and `doctor`, never changed and never hidden silently. | US-004 |

---

## Non-Functional Requirements

| ID | Requirement | Category |
|----|-------------|----------|
| NFR-001 | No traffic leaves the machine; HTTP mode binds `127.0.0.1` only. Subprocesses: read-only `git` (`rev-parse`, `branch --show-current`, `remote get-url`), in `quick` only, the OS input/notification tool (`osascript`, `zenity`, `kdialog`, `notify-send`), and in `tray` and `dashboard` only, the OS opener (`xdg-open`, `open`) and `xdg-mime query`. *(round 4)* One opt-in exception: `ai enrich` (FR-037) starts the person's own `claude` CLI, which sends the masked note text to the person's Claude account; nothing else leaves the machine, and nothing does while `[ai] enrich` is `"off"` (DDR 0007). | Security |
| NFR-002 | No secret appears in any note file written by the tool, any response, or any log (fixture with planted secrets proves it). Logs never contain note text. | Security / Privacy |
| NFR-003 | Writes only inside the notes dir, the config file, the one hook entry + command file owned by `install claude`, the launcher file and the four icon files owned by `install launcher` (FR-039), the autostart file owned by `install tray`, one vault entry plus `<notes dir>/.obsidian/` added by `install obsidian` (FR-040, DDR 0008), and the one GNOME custom shortcut added by `install hotkey` (FR-054) (DDR 0001, DDR 0003). Nothing in any code repo is ever written. | Safety |
| NFR-004 | Search p95 < 500 ms and `note_add` p95 < 150 ms over 10,000 notes on a laptop. Confirmed by HUN-265 (2026-11-17, release build, WSL2, warm cache): search p95 173 ms, `note_add` p95 5.3 ms. | Performance |
| NFR-005 | Safe under a sync tool: two machines adding notes at the same time never lose a note (per-file writes, unique IDs). | Reliability |
| NFR-006 | One Rust binary (`easy-note`, with `n` as a link to it), installed with `cargo install`; Linux (x86_64) only in v1 (DDR 0005). *(round 5)* On Debian and Ubuntu `scripts/build_deb.sh` also builds an installable `.deb` of that same binary (features `window,tray`), with `Depends` read from the binary by `dpkg-shlibdeps`, no maintainer script and no `n` link; each person runs `easy-note setup` after the install (HUN-287). | Portability |
| NFR-007 | Deterministic: same files + same query → same output order; no LLM in the server. *(round 4)* `ai enrich` runs a model only in the person's own `claude` CLI, as a child process, and only on request or with `after_save`; no read path, count or search order depends on it (DDR 0007). | Correctness |
| NFR-008 | Works with no kernel installed, and with no other plug-in installed. | Microkernel |
| NFR-009 | *(round 2)* Hook pass path (prompt without the prefix) p95 < 50 ms, no interpreter start (native binary, DDR 0005); hook save path p95 < 150 ms. Confirmed by HUN-265 (2026-11-17, WSL2): pass p95 1.5 ms, save p95 16.0 ms. | Performance |
| NFR-011 | *(round 3)* `quick` native box (zenity/kdialog) visible p95 < 300 ms after the key press. The own capture window (HUN-289) is measured at about p50 300 ms, p95 400-450 ms to first paint (XWayland, owner's machine); the owner chose to keep the window as the default and not hold it to this number (2026-11-23, Option A); a release-build 20-run measure is still to be recorded. | Performance |
| NFR-012 | *(round 3)* AI detail never changes a note file and never feeds a count other than `enriched`. | Correctness |
| NFR-010 | *(round 2)* `n` / `easy-note add` cold start to confirmation p95 < 150 ms; the CLI never imports the MCP SDK. Confirmed by HUN-265 (2026-11-17, WSL2): `n` p95 15.0 ms. | Performance |
| NFR-013 | *(round 5)* The dashboard window shows the board p95 < 1 s after `easy-note dashboard` over 5,000 notes on the owner's machine (release build, warm cache). | Performance |
| NFR-014 | *(round 6)* One named exception to NFR-001 and NFR-007: scheduled `ai run` starts the person's `claude` CLI without a click, only while `[ai] enrich = "claude"` and `schedule = "tray"` (FR-048). No other path changes. With either off, nothing runs. A run is bounded by `interval_min`, `max_per_run` and the per-call timeout (FR-047, FR-049). | Security / Privacy |
| NFR-015 | *(round 6)* A raw note file is byte-identical after any number of `ai run`, digest or override operations (test over a fixture dir). Every AI change can be undone from `ai-log.jsonl`. *(2026-11-26, HUN-302)* One `ai undo` takes back one applied item whole: title, summary, tags, kind, project, topic and draft keys, with the file's earlier `based_on`, `at`, `model` and `organized_at`; a file the item created is removed. A line logged before HUN-302 is taken back one key at a time. HUN-302 must merge before `ai run` (HUN-295) ships. | Correctness |

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
- Search index or embeddings. Model-written text inside a note file (AI detail lives only in separate AI files, FR-030, DDR 0004; round 6 keeps this: organize, drafts and digests are AI files too).
- *(round 6)* Organizing by place (folder, machine), a scheduler daemon of our own (the tray is the timer), a model call from MCP, and a daily call cap.
- Git commit/push of the notes dir.
- Daily-journal files, templates, to-dos, reminders, due dates.
- Attachments and images.
- Automatic capture from transcripts (easy-memory's job). The round 2 prompt hook captures only what the person types after the prefix.
- A key-listener daemon; macOS/Windows tray autostart (v1.1); a model call inside easy-note other than the opt-in `claude` CLI child of FR-037 (no API key, no SDK, no HTTP client); AI edits of note files; network capture from a phone (the `inbox/` folder plus the person's sync tool covers it).
- Zero-model-call capture in Codex/Cursor (no hook system there).
- A web UI served on a local port (round 4: the dashboard was an Obsidian Bases file, DDR 0006; round 5: easy-note's own window replaces it where a desktop is present, DDR 0009, FR-042; easy-personal shows counts and links, follow-up F1).
- Kernel skills or hooks for notes (follow-up F2 only lists easy-note in `companion-setup`).

---

## Open Questions / Assumptions

| # | Question / Assumption | Owner | Due |
|---|----------------------|-------|-----|
| 1 | Assumed: Claude Code, Codex and Cursor start stdio servers with a cwd or roots that point at the project. Verify per client. | Supervisor | T001, before T004 |
| 2 | Default notes dir `~/easy-notes` (D13) | thunderkds | before T002 |
| 3 | Repo name `easy-note-mcp`, server id `easy-note` (D1) | thunderkds | before repo creation |
| 4 | NFR-004 targets are provisional — closed: confirmed by HUN-265 on WSL2 (`tests/perf.rs`); a native Linux and a cold-cache run stay with the owner | Supervisor | T010 |
| 6 | Owner approves the MCP surface growing from 5 to 7 tools + 1 prompt (D37) | thunderkds | before T008 |
| 7 | Token cost per enriched note (estimate 1–3k) | Supervisor | T014 |
| 8 | Owner approves the MCP surface growing from 7 to 8 tools (`note_context`, FR-051) | thunderkds | before the `note_context` task is merged |
| 5 | Does the `UserPromptSubmit` hook see `/note ...` raw, can it block, and does it run for a prompt queued while the agent works? | Supervisor | T001, before T011 |
