**Date**: 2026-11-14
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`
**Plan ID**: T014 (`linear-tasks.md`, `PROJECT_SPEC.md` Tasks)

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

Plan T014: "Tests: hook notes carry `session`, repo notes carry `commit`; `note_pending_enrich` lists un-enriched and stale notes only; `note_enrich` writes one AI file with caps enforced (over-long fields refused), secrets masked, `based_on`/`at` set by the server, unknown id refused; every note file byte-identical after an enrich run (hash before/after); reads show the "AI detail (not your words)" label; search hit in AI text marked `matched_in: ai`; stats unchanged except `enriched`; `edit` makes detail stale; `rm`/`restore` move the AI file; `tag --accept-ai` and `ai rm` work; `note_add(detail=…)` stores `model: capture`; the T008 tool-list test now expects exactly 7 tools + 1 prompt (D37). Manual run of `/mcp__easy-note__enrich_notes` on 20 real notes: tokens per note recorded."

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> Later, the person's own Claude session adds a title, kind, summary and suggested tags to each one-line note, stored apart from the note and labelled as not the person's words; easy-note itself runs no model.

**Out of scope** (what this task explicitly does NOT do):

* Any model call inside easy-note (DDR 0004)
* Writing AI text into a note file
* Enrichment at capture (D32), except `note_add(detail=…)`

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-028: `session`, `commit`
* FR-029: `note_pending_enrich`
* FR-030: `note_enrich`
* FR-031: `enrich_notes` prompt
* FR-032: AI detail on read/search/CLI
* NFR-012
* US-016, DDR 0004, GRILL D32–D38

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent)
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy)
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

> An agent must NOT start implementing until this gate is checked. If anything here is unclear,
> STOP and ask the Supervisor (Karpathy: Think Before Coding).

---

## Dependencies & Reachability

**Depends on**: HUN-253 (plan T003) — store; HUN-258 (plan T008) — MCP surface; HUN-259 (plan T011) — hook notes to carry `session`

**Entry point**: `note_pending_enrich`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | Hook notes store `session` (event `session_id`); notes in a git repo store `commit` (short HEAD sha) | FR-028 |
| 2 | `note_pending_enrich(limit≤20)` lists notes with no AI file or a stale one (`based_on` ≠ sha256 of note text), with id, text, created, repo, task, session | FR-029 |
| 3 | `note_enrich` writes only `.easy-note/ai/<id>.json` (`inbox__<stem>.json` for inbox); caps title ≤ 120, summary ≤ 600, tags ≤ 5, kind in enum — over-cap refused; secrets masked; `based_on`/`at` set by the server; unknown id refused | FR-030, D34 |
| 4 | Every note file is byte-identical after an enrich run (hash before/after) | NFR-012, DDR 0004 |
| 5 | Prompt `enrich_notes` is fixed text in code, asks for one subagent, reads only transcript entries from the 10 minutes before `created` (≤ 40 entries), falls back per FR-031 round 3b | FR-031 |
| 6 | Reads show `AI detail (not your words)` with model and date; search hits in AI text carry `matched_in: ai`; stats unchanged except `enriched` | FR-032, D38 |
| 7 | `edit` makes detail stale (shown `(stale)`); `rm`/`restore` move the AI file with its note; `ai rm` deletes it; `tag --accept-ai` copies suggested tags | FR-032, D36 |
| 8 | `note_add(detail=…)` (≤ 600 chars) stores an AI file with `model: capture` | FR-032 |
| 9 | The tool-list test expects exactly 7 tools + prompt `enrich_notes` | FR-012, D37 |
| 10 | Manual run of `/mcp__easy-note__enrich_notes` on 20 real notes: tokens per note recorded (answers PRD Open Question #7) | US-016 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

**Locked oracle**: `tests/test_ai_no_note_write.py`

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | Run `note_enrich` on 5 fixture notes | 5 AI files, all note hashes unchanged | automated test |
| 2 | Edit a note after enrich | it reappears in `note_pending_enrich` | automated test |

### Verification Command (exact, runnable)

```bash
uv run pytest tests/test_ai.py tests/test_mcp.py tests/test_leak.py -q
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-262.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-262.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

N/A — pure backend / CLI task; no UI component. The three UI Evidence rows are ☐ N/A.

---

## Approach

**Pattern reference**: `src/easy_note/core/store.py` atomic write (T003); `src/easy_note/surfaces/mcp.py` (T008)

**Vital slice**: `note_enrich` with caps + the never-touch-the-note guarantee

**Cut list**: Scheduling enrichment runs (the person's own schedule); Multiple AI versions per note

AI files are JSON written with the same atomic helper as notes. The staleness hash is over the note's text exactly as stored. The no-note-write guarantee is tested by hashing the notes tree around every AI operation (locked oracle).

---

## Edge Case Checklist

- [ ] Transcript file gone → enrich from note text only, no context claim
- [ ] Note deleted between pending and enrich → unknown id refused
- [ ] Model returns a quote of a secret from the transcript → masked by the guard
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: for each fact this task changes, grep README, `docs/`, comments and docstrings for the old wording

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/easy_note/core/ai.py` | new |
| `src/easy_note/surfaces/prompts.py` | new: fixed prompt text |
| `src/easy_note/surfaces/mcp.py` | 2 tools + prompt, `detail` arg |
| `src/easy_note/surfaces/hook.py` | store `session` |
| `src/easy_note/core/context.py` | `commit` |
| `src/easy_note/surfaces/cli.py` | `ai rm`, `tag --accept-ai`, AI label in `show` |
| `tests/test_ai.py` | new |
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

Unit + MCP client tests; hash-tree oracle; manual run on 20 real notes with token counts.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Security review: `Skill({ skill: "security-review" })` run (Medium/High risk)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI / client
- [ ] Supervisor notified: task ready for Stage 4 review