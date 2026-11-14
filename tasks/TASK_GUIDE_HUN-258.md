**Date**: 2026-11-14
**Complexity Level**: C1
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`
**Plan ID**: T008 (`linear-tasks.md`, `PROJECT_SPEC.md` Tasks)

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*, *Context*, *Guard*, *Trash*, *Skipped*)
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read `agents/backend.md`
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in `agents/backend.md`
6. **C2/C3 or multi-file tasks only**: read `memory/codebase-map.md` if it exists
7. Tests only: temp `HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/easy-note/` or `~/.claude/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Plan T008: "5 tools registered and no others (test lists the names); each of the 4 read tools leaves the notes dir byte-identical (hash before/after); `note_add` adds exactly one file; responses respect FR-016 bounds; tool descriptions contain "only when the person asks"; HTTP mode refuses to bind anything but `127.0.0.1`; smoke run over stdio with the MCP SDK client."

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> Agents in Claude Code, Codex and Cursor can add, search and read the person's notes, but can never edit, delete or move one.

**Out of scope** (what this task explicitly does NOT do):

* `note_pending_enrich`, `note_enrich`, prompt `enrich_notes` (T014)
* Writing any client config (DDR 0001)

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-004: cwd from arg → roots → process cwd
* FR-007: `source: mcp`, `client`
* FR-012: tool list, no edit/delete
* FR-016: bounds
* FR-024: `note:` / `/note` description
* NFR-001: 127.0.0.1 only
* US-001, US-008

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent)
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy)
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

> An agent must NOT start implementing until this gate is checked. If anything here is unclear,
> STOP and ask the Supervisor (Karpathy: Think Before Coding).

---

## Dependencies & Reachability

**Depends on**: HUN-254 (plan T004) — context capture incl. roots; HUN-255 (plan T005) — guard + leak harness; HUN-256 (plan T006) — query engine

**Entry point**: `easy-note serve`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | Exactly `note_add`, `note_search`, `note_list`, `note_read`, `note_stats` are registered (test lists names) | FR-012 |
| 2 | Each read tool leaves the notes dir byte-identical (hash of every file before/after) | FR-012 |
| 3 | `note_add` creates exactly one file with `source: mcp` and `client` from `clientInfo.name` when sent | FR-007 |
| 4 | `note_add` context uses the `cwd` arg, else MCP roots, else process cwd (order from T001) | FR-004 |
| 5 | List/search responses ≤ 50 items, snippet ≤ 200, with `total` and `truncated` | FR-016 |
| 6 | Tool descriptions contain "only when the person asks" and the FR-024 `note:` / `/note` instruction | FR-024, D26 |
| 7 | `easy-note serve --http` binds `127.0.0.1:8769`; `--host 0.0.0.0` is refused with exit 2 | NFR-001 |
| 8 | Smoke test: MCP SDK client over stdio calls every tool | end-to-end |
| 9 | MCP surface registered in `tests/test_leak.py` and passing | NFR-002 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | SDK client `list_tools()` | the 5 names, nothing else | automated test |
| 2 | `note_read` on a hand-edited file with a token | masked output, file unchanged | automated test |

### Verification Command (exact, runnable)

```bash
uv run pytest tests/test_mcp.py tests/test_leak.py -q
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-258.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-258.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

N/A — pure backend / CLI task; no UI component. The three UI Evidence rows are ☐ N/A.

---

## Approach

**Pattern reference**: `src/easy_note/surfaces/cli.py` (T007) — thin calls into core

**Vital slice**: the 5 tools with the no-write guarantee on reads

**Cut list**: Resources and resource templates; Auth on HTTP (localhost-only bind instead)

FastMCP from the official SDK. Read `clientInfo` from the initialize request context; call `roots/list` lazily only when no `cwd` arg is given. HTTP via the SDK's streamable-http transport with a host check before start.

---

## Edge Case Checklist

- [ ] Client that does not support roots → fall back silently
- [ ] Port 8769 in use → clear error, exit 1
- [ ] Very long `text` > 20,000 → tool error, nothing written
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: for each fact this task changes, grep README, `docs/`, comments and docstrings for the old wording

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/easy_note/surfaces/mcp.py` | tools + transports |
| `tests/test_mcp.py` | new |
| `tests/test_leak.py` | register the MCP surface |
| Pre-existing pin tests | List each test in `tests/` that pins text, paths or counts of a file this task edits, and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md` | Supervisor-owned; report needed changes instead |
| `docs/ddr/*` | Decisions are owner-approved; a change needs a new DDR from the Supervisor |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/` | Kit files; Supervisor-only |
| The real `~/easy-notes/`, `~/.config/easy-note/`, `~/.claude/`, `~/.codex/`, `~/.cursor/` | Tests use a temp `HOME` and `EASY_NOTE_DIR` only (NFR-003) |

---

## Test Plan

SDK in-memory client tests; hash-before/after for read tools; HTTP bind test.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Security review: `Skill({ skill: "security-review" })` run (Medium/High risk)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI / client
- [ ] Supervisor notified: task ready for Stage 4 review