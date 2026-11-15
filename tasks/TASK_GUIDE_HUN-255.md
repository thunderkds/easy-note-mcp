**Date**: 2026-11-14
**Complexity Level**: C1
**Risk Level**: High
**Priority**: P0
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`
**Plan ID**: T005 (`linear-tasks.md`, `PROJECT_SPEC.md` Tasks)

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

Plan T005: "Declared patterns (at least: AWS key, GitHub token, Slack token, OpenAI/Anthropic-style key, private key block, JWT, `password=`/`secret=` assignments, generic high-entropy bearer). Tests (round 2, mask not refuse): each planted secret … is written as `[masked:<kind>]`, the rest of the note is kept, the confirmation names the count and kinds, and the value is not echoed; hand-edited file with secrets is masked in \[every read output\]; log capture holds no note text; a grep of every test output for planted values finds nothing."

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> A secret typed into a note never reaches disk, a response or a log, and the note itself is still kept (D21).

**Out of scope** (what this task explicitly does NOT do):

* Surfaces themselves (each surface task adds itself to the leak suite)
* Entropy scanning of whole files beyond the declared bearer rule

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-014: mask on every write and read path
* NFR-002: no secret in any file, response or log
* US-007
* GRILL D8 (revised), D21

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent)
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy)
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

> An agent must NOT start implementing until this gate is checked. If anything here is unclear,
> STOP and ask the Supervisor (Karpathy: Think Before Coding).

---

## Dependencies & Reachability

**Depends on**: HUN-253 (plan T003) — store.add / [store.read](<http://store.read>) to wrap

**Entry point**: `easy_note.core.guard.mask`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `core/guard.py` declares named patterns: `aws_key`, `github_token`, `slack_token`, `openai_key`, `anthropic_key`, `private_key`, `jwt`, `password_assign`, `secret_assign`, `bearer` | FR-014 |
| 2 | `mask(text)` returns `(masked_text, [kinds])`; each match becomes `[masked:<kind>]`; surrounding text is unchanged | FR-014, D21 |
| 3 | `store.add` and `store.update` call the guard before the write; the result reports the count and kinds, never the value | FR-014 |
| 4 | A read helper `guard.mask_for_output` masks hand-edited files; `store.read` output used by surfaces passes through it | FR-014 read side |
| 5 | `tests/test_leak.py` is a parametrized harness: a registry of surfaces (core now; CLI, MCP, hook, quick, inbox added by their tasks) × planted secrets; it greps every captured stdout, stderr, response, log record and written file for the raw values and finds none | NFR-002 |
| 6 | Logging: a test attaches a handler at DEBUG and asserts no note text appears in any record | NFR-002 |
| 7 | False-positive check: a fixture of 50 normal dev notes (`sha256:` hashes, UUIDs, commit shas, URLs) masks nothing | keep the person's words |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

**Locked oracle**: `tests/test_leak.py`

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | `store.add('key AKIA…EXAMPLE and ghp_…')` | file holds `[masked:aws_key]`, `[masked:github_token]`; result `masked=2` | automated test |
| 2 | Hand-written file with a JWT, read via core | output masked, file bytes unchanged | automated test |

### Verification Command (exact, runnable)

```bash
uv run pytest tests/test_guard.py tests/test_leak.py -q
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-255.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-255.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

N/A — pure backend / CLI task; no UI component. The three UI Evidence rows are ☐ N/A.

---

## Approach

**Pattern reference**: `src/easy_note/core/store.py` (T003)

**Vital slice**: mask-on-write in `store` + the leak harness every later surface plugs into

**Cut list**: Configurable user patterns (v1 has declared ones only)

Patterns re-declared here (never imported from siblings — PROJECT_SPEC constraint). Run them in a fixed order; replace right-to-left by span so offsets stay valid. Planted test values use documented fake formats (e.g. `AKIAIOSFODNN7EXAMPLE`).

---

## Edge Case Checklist

- [ ] Overlapping matches (a JWT inside `password=`) — mask once, longest span wins
- [ ] Secrets split across lines (private key block) — mask the whole block
- [ ] Git commit shas and `sha256:` values must not be masked
- [ ] Masked marker itself must not re-match on a second pass
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: for each fact this task changes, grep README, `docs/`, comments and docstrings for the old wording

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/easy_note/core/guard.py` | new |
| `src/easy_note/core/store.py` | call the guard on write |
| `tests/test_guard.py` | new |
| `tests/test_leak.py` | new harness (oracle by qa-expert) |
| `tests/fixtures/secrets/` | planted values (fake, well-known example formats) |
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

Unit tests per pattern (hit + near-miss), the leak harness, the false-positive corpus, log capture.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Security review: `Skill({ skill: "security-review" })` run (Medium/High risk)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI / client
- [ ] Supervisor notified: task ready for Stage 4 review