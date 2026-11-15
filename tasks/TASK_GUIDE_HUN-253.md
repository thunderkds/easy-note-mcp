**Date**: 2026-11-14
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P0
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`
**Plan ID**: T003 (`linear-tasks.md`, `PROJECT_SPEC.md` Tasks)

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

Plan T003: "Tests: round-trip of a tool-written note is byte-identical; ID format and clash suffix (two writes in the same second with the same text); a write never replaces an existing file; temp file + rename (no partial file after a killed write, simulated); `rm` → trash, `restore` → back, same bytes; bad front matter and `easy-note: 2` counted as skipped with path, file unchanged; unknown keys kept on edit. `docs/NOTE_FORMAT.md` final."

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> The notes folder is the only store and its file format is a public contract (DDR 0002). This task makes reading and writing one note exact, safe under sync tools, and impossible to lose by overwrite.

**Out of scope** (what this task explicitly does NOT do):

* Context capture (T004)
* Secret masking (T005)
* List/search (T006)
* Inbox files (T013)
* AI files (T014)

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-001: notes dir resolution, zero setup (`EASY_NOTE_DIR` → config → `~/easy-notes`, created on first write)
* FR-002: file path, front matter, title ≤ 120, text ≤ 20,000
* FR-003: ID and clash suffix
* FR-008: atomic writes
* FR-009: trash/restore/empty
* FR-017: skipped files
* NFR-005: two machines never lose a note

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent)
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy)
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

> An agent must NOT start implementing until this gate is checked. If anything here is unclear,
> STOP and ask the Supervisor (Karpathy: Think Before Coding).

---

## Dependencies & Reachability

**Depends on**: HUN-252 (plan T002) — package skeleton and test runner

**Entry point**: `easy_note.core.store.add`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `render(parse(f))` of a tool-written note is byte-identical to `f` | FR-002, DDR 0002 |
| 2 | ID matches `^\d{8}-\d{6}-[0-9a-f]{4}$` from local time + sha256(created ns, hostname, text); two writes in the same second with the same text give `…-xxxx` and `…-xxxx-2` | FR-003 |
| 3 | A write never replaces an existing file (`os.link`/`O_EXCL` or equivalent); test pre-creates the target and asserts both files survive | FR-003, NFR-005 |
| 4 | Writes go temp-in-same-dir + `os.replace`; a simulated kill between write and rename leaves no partial `.md` note | FR-008 |
| 5 | `store.rm(id)` moves to `<dir>/.easy-note/trash/`, `store.restore(id)` moves it back with identical bytes; `store.empty_trash(older_than)` deletes only trash files older than the cutoff | FR-009 |
| 6 | Bad front matter and `easy-note: 2` files are listed as skipped with their path; their bytes are unchanged | FR-017 |
| 7 | `store.update` keeps unknown front-matter keys and sets `updated` | FR-008, NOTE_FORMAT |
| 8 | No config and no `EASY_NOTE_DIR`: first `add` creates `~/easy-notes` (temp HOME) and returns `created_dir=True` once; `EASY_NOTE_DIR` beats the config | FR-001 |
| 9 | Title > 120 chars is cut at the first line rule; text > 20,000 chars is refused with a clear error (the only allowed refusal: over-size) | FR-002 |
| 10 | `docs/NOTE_FORMAT.md` status changed from Draft to Final for v1 and matches the code | DDR 0002 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | Two `store.add('x')` calls in one frozen second | two files, second has `-2` | automated test |
| 2 | Monkeypatched `os.replace` raising | no new `.md` under `YYYY/MM/` | automated test |
| 3 | A file with `easy-note: 2` | in `skipped`, bytes unchanged | automated test |

### Verification Command (exact, runnable)

```bash
uv run pytest tests/test_format.py tests/test_store.py -q
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-253.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-253.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

N/A — pure backend / CLI task; no UI component. The three UI Evidence rows are ☐ N/A.

---

## Approach

**Pattern reference**: None — first core module

**Vital slice**: exact round-trip + never-overwrite + atomic write; these protect the person's files

**Cut list**: File locking across machines (unique IDs + exclusive create cover NFR-005); Any index or cache

Front matter with PyYAML `safe_load` / a fixed-order renderer so round-trips are byte-stable. Exclusive create (`os.open(O_CREAT|O_EXCL)`) on the temp name, write, fsync, then `os.link` to the final name (fails if it exists → next suffix), then unlink temp. Clock and hostname injected for tests.

---

## Edge Case Checklist

- [ ] YAML turns `no`/`on`/dates into bools/dates — quote strings on render, keep `created` as a string
- [ ] Windows has no hard links on some filesystems — fall back to `O_EXCL` on the final name
- [ ] Month folder uses the local date of `created`, not UTC (D10)
- [ ] Hand-edited file with CRLF line endings must still parse
- [ ] Obsidian adds its own keys — unknown keys are kept
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: for each fact this task changes, grep README, `docs/`, comments and docstrings for the old wording

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/easy_note/core/format.py` | new |
| `src/easy_note/core/store.py` | new |
| `src/easy_note/core/config.py` | new: dir resolution, `config.toml` read/write (tomllib + minimal writer) |
| `tests/test_format.py`, `tests/test_store.py` | new |
| `docs/NOTE_FORMAT.md` | finalize |
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

Unit tests per AC row with a temp notes dir, a frozen clock and a fixed hostname; a fixtures dir with bad front matter and a v2 file.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Security review: `Skill({ skill: "security-review" })` run (Medium/High risk)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI / client
- [ ] Supervisor notified: task ready for Stage 4 review