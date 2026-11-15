**Date**: 2026-11-14
**Complexity Level**: C1
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`
**Plan ID**: T013 (`linear-tasks.md`, `PROJECT_SPEC.md` Tasks)

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

Plan T013: "Tests: `inbox/20261114-183002.txt` → id `inbox/20261114-183002`, created from the name; a file with no time prefix → mtime; `#tags` parsed; file with front matter → its keys win; list/search/stats/read include inbox notes; the inbox dir is byte-identical after every read command; `rm inbox/…` → trash; subfolders and other extensions ignored."

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> A note saved from a phone into `<notes dir>/inbox/` by any sync app shows up like any other note, and easy-note never rewrites that file.

**Out of scope** (what this task explicitly does NOT do):

* A network endpoint (D23)
* Normalizing inbox files into dated folders

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-023: inbox rules
* FR-014: inbox files masked on read
* US-012
* NOTE_FORMAT Inbox section
* GRILL D23

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent)
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy)
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

> An agent must NOT start implementing until this gate is checked. If anything here is unclear,
> STOP and ask the Supervisor (Karpathy: Think Before Coding).

---

## Dependencies & Reachability

**Depends on**: HUN-253 (plan T003) — store and format; HUN-256 (plan T006) — query engine; HUN-257 (plan T007) — CLI rm/show

**Entry point**: `easy_note.core.store.iter_inbox`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `inbox/20261114-183002.txt` → id `inbox/20261114-183002`, `created` from the name (local time), `source: inbox` | FR-023 |
| 2 | A file with no time prefix → `created` = mtime | FR-023 |
| 3 | `#tags` in the text become tags; valid `easy-note: 1` front matter keys win | FR-023 |
| 4 | `ls`, `search`, `stats`, `show` and the MCP read tools include inbox notes | FR-023 |
| 5 | The inbox dir is byte-identical (hash + mtime) after every read command | FR-023 |
| 6 | `rm inbox/<stem>` moves the file to trash; `restore` puts it back | FR-023, FR-009 |
| 7 | Subfolders and extensions other than `.md`/`.txt` are ignored | FR-023 |
| 8 | Inbox surface registered in `tests/test_leak.py` (secret in an inbox file is masked on every read) | FR-014 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | Inbox fixture with 3 files | 3 notes listed with inbox ids | automated test |
| 2 | `easy-note search` run | inbox hashes unchanged | automated test |

### Verification Command (exact, runnable)

```bash
uv run pytest tests/test_inbox.py tests/test_leak.py -q
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-261.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-261.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

N/A — pure backend / CLI task; no UI component. The three UI Evidence rows are ☐ N/A.

---

## Approach

**Pattern reference**: `src/easy_note/core/store.py` (T003)

**Vital slice**: read-only parse + inclusion in queries

**Cut list**: Watching the folder for changes; Attachments in inbox

Inbox notes are parsed views, never written. IDs with `inbox/` prefix route rm/restore to the inbox path; trash keeps a sidecar of the original relative path.

---

## Edge Case Checklist

- [ ] Sync-app temp files (`.~lock`, `.icloud`) — ignored by extension
- [ ] Two files with the same stem `.md` and `.txt` — both listed, ids disambiguated by extension
- [ ] Non-UTF-8 file → skipped with path
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: for each fact this task changes, grep README, `docs/`, comments and docstrings for the old wording

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/easy_note/core/store.py` | `iter_inbox`, inbox ids in rm/restore |
| `src/easy_note/core/query.py` | include inbox |
| `tests/test_inbox.py` | new |
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

Fixture inbox dir; hash/mtime snapshot around every read command.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Security review: `Skill({ skill: "security-review" })` run (Medium/High risk)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI / client
- [ ] Supervisor notified: task ready for Stage 4 review