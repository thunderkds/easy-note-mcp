**Date**: 2026-11-14
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`
**Plan ID**: T011 (`linear-tasks.md`, `PROJECT_SPEC.md` Tasks)

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

Plan T011: "Hook tests with recorded event JSON: prefix → one file, `source: hook`, repo from event `cwd`, block JSON with id/repo/task; other prompt → no stdout, exit 0, no file; malformed stdin → prompt passes; masking applies. Install tests on a temp settings file: diff shown, backup created and never overwritten, run twice = once, existing easy-memory hook entries byte-identical, symlink kept; uninstall removes exactly its entry and `note.md`. Manual smoke in real Claude Code: `/note test` shows `✓ noted …` and the model does not answer."

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> In Claude Code, `/note …` saves a note with no model turn and no tokens; every other prompt passes untouched and fast.

**Out of scope** (what this task explicitly does NOT do):

* Codex/Cursor hooks (they have none)
* Project-scope settings (DDR 0003: user-level only)
* The `session` field (T014)

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-018: hook behaviour
* FR-019: install/uninstall exactly one entry + `note.md`
* NFR-009: pass path stdlib-only, p95 < 50 ms
* US-013, DDR 0003, GRILL D17, D18, D25

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent)
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy)
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

> An agent must NOT start implementing until this gate is checked. If anything here is unclear,
> STOP and ask the Supervisor (Karpathy: Think Before Coding).

---

## Dependencies & Reachability

**Depends on**: HUN-251 (plan T001) — hook facts and default prefix; HUN-255 (plan T005) — guard; HUN-257 (plan T007) — CLI and core add path

**Entry point**: `easy-note hook-prompt`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | With a recorded event whose prompt starts with the prefix: one file, `source: hook`, context from event `cwd`, stdout `{"decision":"block","reason":"✓ noted <id> · <repo> · <task>"}` | FR-018 |
| 2 | Any other prompt: no stdout, exit 0, no file | FR-018 |
| 3 | Malformed stdin or a core error: the prompt passes; the error is shown as the reason only when the prefix matched | FR-018 |
| 4 | The pass path imports only the stdlib (subprocess test on `sys.modules`); core is imported only after a prefix match | NFR-009 |
| 5 | Prefix is configurable (`config.toml` `capture_prefix`), default from T001's decision | D18 |
| 6 | `install claude` on a temp settings file shows a diff, writes a timestamped backup (never overwrites an older one), adds exactly one `UserPromptSubmit` entry and `~/.claude/commands/note.md`; running it twice changes nothing | FR-019, DDR 0003 |
| 7 | Existing hook entries (e.g. easy-memory's) are byte-identical after install and uninstall; a symlinked settings file stays a symlink | DDR 0003 |
| 8 | `uninstall claude` removes exactly its entry and `note.md` | FR-019 |
| 9 | Hook surface registered in `tests/test_leak.py` and passing | NFR-002 |
| 10 | Manual smoke in real Claude Code: `/note test` shows `✓ noted …` and the model does not answer (pasted) | US-013 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | `tests/fixtures/claude_hook/note_prefix.json` on stdin | block JSON + one file | automated test |
| 2 | `{"prompt":"fix the bug"}` on stdin | empty stdout, exit 0 | automated test |

### Verification Command (exact, runnable)

```bash
uv run pytest tests/test_hook.py tests/test_install.py tests/test_leak.py -q
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-259.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-259.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

N/A — pure backend / CLI task; no UI component. The three UI Evidence rows are ☐ N/A.

---

## Approach

**Pattern reference**: `tests/fixtures/claude_hook/` (T001)

**Vital slice**: the block-on-prefix path and the safe settings install

**Cut list**: Windows settings paths beyond `~/.claude`; Auto-update of an old entry's command string

Hook reads stdin JSON with `json`, checks the prefix, and only then imports core. Install edits settings JSON preserving key order (`json` with `object_pairs_hook`), identifies its own entry by command string `easy-note hook-prompt`, writes via temp + replace on the resolved path so symlinks stay.

---

## Edge Case Checklist

- [ ] Settings file missing → create with only our entry
- [ ] Settings file with comments or invalid JSON → refuse, change nothing
- [ ] Prompt `/notes` must not match prefix `/note `
- [ ] Prefix followed by nothing → no note, prompt passes
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: for each fact this task changes, grep README, `docs/`, comments and docstrings for the old wording

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/easy_note/surfaces/hook.py` | new, stdlib-only top level |
| `src/easy_note/surfaces/install.py` | new: claude target |
| `src/easy_note/surfaces/cli.py` | `hook-prompt`, `install claude`, `uninstall claude` |
| `tests/test_hook.py`, `tests/test_install.py` | new |
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

Recorded event fixtures from T001; temp settings files incl. a symlink and an easy-memory entry; manual smoke in real Claude Code.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Security review: `Skill({ skill: "security-review" })` run (Medium/High risk)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI / client
- [ ] Supervisor notified: task ready for Stage 4 review