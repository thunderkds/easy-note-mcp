# 0001. Native runtime and write targets

**Status**: Proposed (Supervisor, autonomous run; owner to accept) · **Amended by**: DDR 0003 (Claude Code hook + launcher install), DDR 0004 (AI detail files)
**Date**: 2026-11-14
**Deciders**: thunderkds
**Related**: PRD FR-001, FR-008, FR-009, FR-012, NFR-001, NFR-003, NFR-006 · GRILL D6, D7, D11 · easy-personal DDR 0001

---

## Context

Two family conventions do not fit a note tool. easy-ui-mcp and easy-verifier-mcp ship as Docker services, and
they write only to `reports/`. easy-note's whole job is to write files the person keeps, in a folder the person
picks, and agent clients start its stdio server as a command on the host.

DDR gate: hard to reverse (the write rules shape every surface) ✅, surprising without context (a plug-in that writes outside `reports/`) ✅, genuine trade-off ✅. 3 of 3.

---

## Decision

1. **Native only** (`uv tool install easy-note-mcp`). No Docker image in v1.
2. easy-note may write **only**:
   - new note files under the notes dir, and `.easy-note/trash/` inside it;
   - an existing note, only through `easy-note edit` / `easy-note tag` (person-run CLI);
   - `~/.config/easy-note/config.toml`, only through `easy-note init`.
3. Of the MCP tools only `note_add` writes, and it only creates a new file. *(DDR 0004: `note_enrich` also writes, only AI files under `.easy-note/ai/`.)* No MCP tool edits, deletes, restores, moves or changes config.
4. Git is read-only (`rev-parse`, `branch --show-current`, `remote get-url origin`). No commit, fetch or push, even when the notes dir is a git repo.
5. easy-note never writes agent client config (`~/.claude.json`, `~/.codex/config.toml`, `~/.cursor/mcp.json`) or any project file; the README shows the snippet and the person runs it. *Exception, DDR 0003:* one hook entry in user-level `~/.claude/settings.json` and `~/.claude/commands/note.md`, only through `easy-note install claude`.
7. Files in `<notes dir>/inbox/` are never written, renamed or normalized (round 2, FR-023).
6. A permanent delete happens only on `easy-note trash --empty`.

---

## Alternatives Considered

| Alternative | Pros | Cons | Why not chosen |
|-------------|------|------|----------------|
| Native, narrow write list (chosen) | Notes go where the person wants; agents cannot destroy notes | Deviates from Docker parity | **Selected** |
| Docker like easy-ui / easy-verifier | Family parity, isolation | Needs a bind mount of the notes dir and repos to read git; cwd inside the container is not the project | Context capture breaks |
| Full CRUD over MCP | Agent can fix a typo for the person | One wrong call deletes notes | Risk outweighs a small gain; editor or CLI does it |
| Auto-commit notes repo | Sync with no step | Writes git history; may push secrets; needs network | Breaks local-first and no-surprise rules |

---

## Consequences

### Positive
- A person's notes cannot be changed or lost through an agent.
- Notes are ordinary files; git, Dropbox, Obsidian work with no code.

### Negative (accepted trade-offs)
- Second family plug-in (after easy-personal) with no Docker mode.
- Sync and backup are the person's job.

### Follow-up
- [ ] T008 test: each of the 4 read tools leaves the notes dir byte-identical; `note_add` adds exactly one file.
