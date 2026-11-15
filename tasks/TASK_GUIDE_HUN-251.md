**Date**: 2026-11-14
**Complexity Level**: C1
**Risk Level**: Medium
**Priority**: P0
**Assigned agent**: QA-Automation-Agent
**Agent guide**: `agents/qa.md`
**Plan ID**: T001 (`linear-tasks.md`, `PROJECT_SPEC.md` Tasks)

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*, *Context*, *Guard*, *Trash*, *Skipped*)
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read `agents/qa.md`
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in `agents/qa.md`
6. **C2/C3 or multi-file tasks only**: read `memory/codebase-map.md` if it exists
7. Tests only: temp `HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/easy-note/` or `~/.claude/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Plan T001 (`linear-tasks.md`): "For each of Claude Code, Codex, Cursor: record the stdio server's process cwd when started from a project, whether the client answers `roots/list`, and the `clientInfo.name` it sends." Round 2: "does a `UserPromptSubmit` hook receive `/note …` raw; does `{"decision":"block","reason":…}` stop the prompt and show the reason; does the hook run for a prompt typed while the agent works; which `cwd` the event carries in a worktree. Output decides the default prefix (D18)."

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> Replace two assumptions with measured facts before code depends on them: how each client tells a stdio server which project it works in (decides FR-004's cwd order), and whether the Claude Code prompt hook can capture `/note …` with no model turn (decides D18's prefix).

**Out of scope** (what this task explicitly does NOT do):

* No product code in `src/`
* No change to the person's real client settings beyond a throwaway probe entry that is removed after the run
* Cloud agent sessions (Out of Scope v1)

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-004: cwd order arg → roots → process cwd
* FR-007: `client` from the initialize handshake
* FR-018: hook prefix, block JSON
* PRD Open Questions #1 and #5
* GRILL G3, G20, D18

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent)
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy)
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

> An agent must NOT start implementing until this gate is checked. If anything here is unclear,
> STOP and ask the Supervisor (Karpathy: Think Before Coding).

---

## Dependencies & Reachability

**Depends on**: None

**Entry point**: Standalone — N/A: a spike; its output is `docs/experiments/T001-client-launch.md`, read by T004 and T011

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `docs/experiments/T001-client-launch.md` has one row per client (Claude Code, Codex, Cursor) with: process cwd when started from a project, `roots/list` answered (yes/no + value), `clientInfo.name` sent | per-client facts |
| 2 | The doc states the decided cwd order for FR-004 and, for any client with neither roots nor a project cwd, the line "pass `cwd`" for the README | decides FR-004 |
| 3 | Claude Code hook facts recorded: `/note …` reaches `UserPromptSubmit` raw or not; block JSON stops the prompt and shows the reason; hook runs for a prompt queued while the agent works; event `cwd` inside a linked worktree | round 2 questions |
| 4 | The doc states the default prefix: `/note ` if the hook sees it raw and can block, else `;n ` (D18) | decides D18 |
| 5 | Recorded real `UserPromptSubmit` event JSONs (secrets and paths scrubbed) saved under `tests/fixtures/claude_hook/` and `tests/test_T001_fixtures.py` asserts each has `prompt`, `cwd`, `session_id` | fixtures for T011 |
| 6 | No probe entry is left in any client config after the run (diff of each config before/after pasted) | negative / cleanup |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | The three clients started from a git project with a probe stdio server | a filled table, one row per client | manual run, table pasted |
| 2 | `pytest tests/test_T001_fixtures.py -q` | passes; each fixture has the 3 keys | automated test |
| 3 | A fixture file grep for `/home/` or `token` | no match | automated test |

### Verification Command (exact, runnable)

```bash
# ci: skip — the spike runs in real desktop clients; CI runs only the fixture test below
pytest tests/test_T001_fixtures.py -q
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-251.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-251.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

N/A — pure backend / CLI task; no UI component. The three UI Evidence rows are ☐ N/A.

---

## Approach

**Pattern reference**: None — no code in this repo yet

**Vital slice**: the cwd/roots table and the `/note` raw + block answer; they gate T004 and T011

**Cut list**: Measuring hook latency (T010 does it); Clients other than the three named

Write a minimal stdio probe with the MCP SDK that logs `os.getcwd()`, the `roots/list` answer and `clientInfo` to a temp file. Register it per client at project scope, start a session from a git repo and from a linked worktree, read the log, then remove the entry. For the hook, register a probe `UserPromptSubmit` command that dumps stdin to a temp file and prints the block JSON only for `/note `.

---

## Edge Case Checklist

- [ ] Claude Code may treat `/note` as an unknown slash command before the hook runs — test with and without `~/.claude/commands/note.md` present
- [ ] Codex/Cursor may start the server from `$HOME`, not the project — record it, do not guess
- [ ] Event `cwd` in a linked worktree may be the worktree folder, not the main repo (T004 maps it)
- [ ] Recorded events may hold the person's paths or prompt text — scrub before saving
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: for each fact this task changes, grep README, `docs/`, comments and docstrings for the old wording

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `docs/experiments/T001-client-launch.md` | new: the findings table and the two decisions |
| `tests/fixtures/claude_hook/*.json` | new: recorded hook events, scrubbed |
| `tests/test_T001_fixtures.py` | new: fixture shape test |
| `scripts/probe_server.py` (throwaway, may stay under `scripts/`) | a 30-line stdio server that logs cwd, roots and clientInfo to a temp file |
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

Automated: the fixture shape test. Manual: the per-client runs, with the table and config diffs pasted into the review Evidence.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Security review: `Skill({ skill: "security-review" })` run (Medium/High risk)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI / client
- [ ] Supervisor notified: task ready for Stage 4 review