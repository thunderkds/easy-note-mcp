# BRAINSTORMING_LOG.md
**Generated**: 2026-11-14
**Task / Context**: New plug-in — `easy-note-mcp`, a note-taking Easy Kit MCP (pre-T001, no repo yet). Scope: one-line capture of the person's own notes from any agent chat or the shell, saved as plain files in a folder the person picks.
**Skill**: `Skill({ skill: "brainstorming" })` — tier **Standard** (new repo; direction partly set in the ideas thread)
**Mode**: autonomous. The owner asked (2026-11-14) for brainstorm, grill and decisions without stops. Every call that is the owner's is made here as a recommendation, recorded with its alternative, and listed in `FINAL_REPORT.md` for review.

---

## The Problem Space

While working with agents, the owner has small thoughts all day: "retry bug in easy-ui only on Firefox", "ask about the 8 KB Codex cap", "idea: verifier pack for i18n". Today they go into a chat that scrolls away, a scratch file, or nowhere. The owner wants note taking to be easy, and wants the notes saved somewhere.

What already exists in the family, and why none of it fits (verified 2026-11-14):

| Candidate home | What it is | Why it does not fit | Source |
|---|---|---|---|
| easy-memory-mcp | Automatic agent memory: the Stop hook saves one note per agent turn (prompt + final answer) as JSON under `~/.easy-memory/<repo-key>/`; the prompt hook injects up to 5 notes into prompts | Its "note" is a machine-captured turn for agent recall, injected into prompts. A person's own notes must never be injected automatically and must be human-readable files. Merging changes easy-memory's meaning. | `easy-memory-mcp/PROJECT_SPEC.md` Architecture Summary, Glossary |
| easy-personal-mcp | Person-level analytics and dashboard | Store is metadata only (NFR-002, no message text); the one text exception needed DDR 0003 and is limited to `save_report` (≤ 2,000 chars, no edit/delete). Notes are all text and must be editable. | `easy-personal-mcp/docs/ddr/0003-store-owner-saved-investigate-reports.md` |
| easy-kit `memory/` | Per-repo Supervisor memory, Supervisor-only writes | Lives inside a repo, is about the project, and only the Supervisor writes it | `easy-kit/AGENTS.md` Memory Write Protocol |

So easy note is a new subject: **the person's own explicit notes**, across repos, owned by the person as files.

Non-negotiables (project instructions; easy-personal `PROJECT_SPEC.md` Critical Constraints as the nearest sibling):
- **Microkernel**: own repo, runs locally, usable with or without the kernel; the kernel never depends on it; siblings are conventions-only references, code is never imported or vendored.
- **Local-first**: no traffic leaves the machine; optional HTTP binds `127.0.0.1` only.
- **No secrets in output**.
- **Deterministic**: no LLM in the server; tags, search and stats are rule-based.
- Same structure as the siblings: Python 3.11+, one core shared by CLI + MCP (stdio default, streamable-http opt-in), kit artifacts in the repo.

---

## Questions for the User

None asked (autonomous run). The questions below were answered by the Supervisor; each answer is a decision in `GRILL_requirement.md` and `FINAL_REPORT.md` that the owner can override.

1. Separate repo, or a feature of easy-personal? → separate repo (Option B below).
2. One file per note, or one file per day? → one file per note (D2).
3. May agents edit or delete notes? → no; add/read only over MCP (D7).
4. How does easy-personal see notes? → reads front matter of the files, read-only (DDR 0002).

---

## Alternative Paths

| Option | Name | Summary | Invasiveness | Code Volume | Regression Risk | Recommended? |
|--------|------|---------|-------------|------------|----------------|--------------|
| A | Feature of easy-personal | `note_add` tool + notes table in `~/.easy-personal/store.sqlite`; notes page on the dashboard | High (breaks NFR-002 a second time) | ~800 lines in an existing large repo | Medium | |
| B | Separate plug-in, Markdown files | `easy-note-mcp` repo; notes are Markdown files with front matter in a folder the person picks; easy-personal only reads counts | Low (new repo; one read-only source in easy-personal later) | ~1.2k lines + tests | Low | ✅ Yes |
| C | Extend easy-memory | Add a `kind: manual` note to easy-memory's store and a `memory_note` tool | Medium | ~400 lines | High (prompt hook may inject manual notes; store is JSON, not human files) | |
| D | No server: a skill + a shell alias | A kernel skill that appends to `~/notes.md`; a `note` shell function | Low | ~50 lines | Low | |

### Option A — Feature of easy-personal
**Approach**: a `notes` table, `note_add` / `note_search` tools, a Notes page.
**Pros**: one install; the dashboard exists already.
**Cons**: makes easy-personal's privacy promise (metadata only) false for a whole feature, not one capped report; notes would sit in SQLite, not in files the person can open, edit, grep or sync; the dashboard only runs while open.
**Why it might fail**: the owner would need a second NFR-002 exception (DDR 0003 already says reports "cannot be deleted or edited"), and the first sync or edit request forces a file export anyway.

### Option B — Separate plug-in, Markdown files (chosen)
**Approach**:
```
<notes dir>/                       picked by the person (default ~/easy-notes), may be a git repo or an Obsidian vault
  2026/11/20261114-164713-a1b2.md  one note: YAML front matter + Markdown body
  .easy-note/trash/                deleted notes, restorable
src/easy_note/
  core/{format,store,context,guard,query}.py
  surfaces/{cli,mcp}.py
```
Front matter carries `id`, `created`, `repo`, `branch`, `task`, `tags`, `source`. Capture: `note_add` (MCP) or `easy-note add "..."` (CLI). Context (repo, branch, task ID) is read from git, never typed.
**Pros**: files are the product — readable, editable in any editor, greppable, syncable with git or Dropbox, an Obsidian vault for free; no database to migrate; easy-personal can read counts from front matter without storing any text; fits every family rule.
**Cons**: one more repo and install; many small files; search scans files (no index).
**Why it might fail**: (1) the MCP server cannot see which repo the agent works in → wrong tags (T001 spike settles it); (2) a person pastes a secret into a note that is git-synced → guard at add (D8); (3) search slows past ~10k files → measured in T010, index only if the target is missed.

### Option C — Extend easy-memory
**Approach**: manual notes stored next to automatic turn notes.
**Pros**: reuses easy-memory's store, search and repo keys.
**Cons**: easy-memory's prompt hook scores and injects notes into prompts; manual notes would leak into every agent context unless every path filters them; JSON files are not for humans; notes are per repo, but a person's note may belong to no repo.
**Why it might fail**: the owner already judged the merge a bad idea (ideas thread, 2026-11); two meanings for "note" in one repo breaks One Term, One Meaning.

### Option D — Skill + shell alias
**Approach**: a kernel skill "append a line to ~/notes.md" and a `note()` shell function.
**Pros**: almost no code; works today.
**Cons**: a kernel skill puts personal behaviour in the kernel (wrong side of the microkernel boundary); no context tags, no search, no stats for easy-personal; Codex and Cursor do not share it.
**Why it might fail**: it is a scratch file with extra steps; the owner asked for a plug-in deep dive.

---

## 50% Rule Check

Option B at half size: drop the MCP server and ship a CLI only (~500 lines). That fails the main use: capture "in any agent chat" without leaving it. Drop search and stats: the files can be grepped, but easy-personal needs counts per repo and task, and agents need `note_search` to answer "what did I note about X". Kept: add, list, search, read, stats, trash. Cut (see Cut List): edit over MCP, index, git automation, daily files, connectors, web UI, attachments.

---

## Adversarial Review (Option B)

| Attack | Answer |
|---|---|
| "A file per one-line note is silly; use a daily file" | Per-note files make add, edit, delete and git merge atomic per note and need no line-ID syntax. A daily view is a `list --day` query. Recorded as D2 with the daily-file alternative. |
| "Agents will spam notes" | MCP tool descriptions say "only when the person asks"; every note records `source: mcp` + client name; `easy-note ls --source mcp` and `rm` clean up; no MCP delete/edit, so an agent cannot destroy notes. |
| "Notes in a git repo leak secrets" | `note_add` refuses text that matches the secret patterns (same approach as easy-personal DDR 0003, re-implemented, not imported); reads mask secret-looking spans in hand-edited files. |
| "Cloud agent sessions (claude.ai/code) cannot reach a local MCP" | True. v1 is local only, like every sibling. Stated in Out of Scope; revisit with a sync design later. |
| "easy-personal reading the folder couples two plug-ins" | It reads only the documented, versioned front matter (a public contract, unlike easy-context's private SQLite in easy-personal DDR 0002). easy-note never depends on easy-personal. |
| "Why not SQLite + export?" | Then the files are a copy and drift. Files as the source of truth means no export, no migration, no lock-in. |

---

## Decision

**Option B**, recorded 2026-11-14 by the Supervisor in autonomous mode (owner to confirm). Locked details: `GRILL_requirement.md` D1–D16; hard-to-reverse calls in `docs/ddr/0001` (runtime and write targets) and `docs/ddr/0002` (file format as the public contract).

## Cut List (deliberately not built in v1)

- Edit and delete over MCP (CLI and the person's editor only).
- A search index or embeddings (scan files; revisit only if T010 misses its target).
- Git commit / push automation.
- Daily-journal files, templates, reminders, due dates, to-do states.
- Connectors (Notion, Apple Notes, Google Keep) and any network call.
- A web UI of its own (easy-personal shows counts and links; the folder opens in any editor or Obsidian).
- Attachments and images.
- Automatic capture from transcripts (that is easy-memory's job).

> **Round 2 (2026-11-14):** the capture path was pressure-tested in `BRAINSTORMING_LOG_capture.md`. It adds a Claude Code prompt hook, an `n` command, zero setup, mask-not-refuse, `quick` and a read-only `inbox/`.
- Codex/Cursor-specific features beyond the stdio MCP config snippet.
