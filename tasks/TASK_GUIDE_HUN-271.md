**Date**: 2026-11-15
**Complexity Level**: C1
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`
**Plan ID**: R4 (DDR 0005; Rust port of HUN-254)

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*, *Context*, *Guard*, *Trash*, *Skipped*)
2. Read `docs/ddr/0005-rust-single-binary.md` in full
3. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
4. Read this file completely
5. Read `agents/backend.md`
6. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in `agents/backend.md`
7. Tests only: temp `HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/easy-note/` or `~/.claude/`

---

> **Rust port (DDR 0005).** The requirement and Acceptance Criteria below are the Python HUN-254 ones, kept word for word: behaviour, numbers, limits and error reasons must not change. Python names (`store.add`, `os.link`, `pytest`, `.py` paths) stand for their Rust equivalents; see **Rust port rules**.

## Requirement (Pillar 1 — Adapt the requirement)

Plan T004: "Tests on temp git repos: main repo, linked worktree (repo = main folder name), no git (fields absent), detached HEAD (branch absent), remote `https://user:tok@github.com/o/r.git` → `o/r` and no token anywhere in the file; branch `fix/t120-x` → `T120`, `feat/HUN-221-y` → `HUN-221`, `main` → absent; explicit task wins; cwd order per T001; git timeout 2 s → fields absent, add still succeeds; no path in any written file."

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> Each note gets the right repo, remote, branch and task with read-only git, and never a guessed value or a filesystem path.

**Out of scope** (what this task explicitly does NOT do):

* Reading `PROJECT_KANBAN.md` for the task (D5 rejected it)
* Any git write
* The `commit` field (T014)

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-004: repo/remote/branch, cwd order, no paths
* FR-005: task patterns
* NFR-001: only read-only git subcommands
* GRILL D4, D5, G5, G6

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [x] Restated intent confirmed to match the user's request (Supervisor, auto; owner accepted DDR 0005 and the R1–R5 plan on 2026-11-15)
- [x] Domain terms align with `PROJECT_SPEC.md` glossary
- [x] Every Acceptance Criterion below traces to a line in the Requirement
- [x] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

---

## Rust port rules

**Oracle (port first, red):** `git show 2d497fc:tests/test_context.py` (Python HUN-254 + HUN-267 task pattern). Port every test case to Rust (same inputs, same expected output) before the implementation. Where a Python test checks something Rust makes impossible (e.g. a type error that cannot compile), replace it with the nearest Rust check and list it in the review file.

- subprocess → `std::process::Command` with no shell, `GIT_TERMINAL_PROMPT=0`, a 2 s timeout (spawn + `wait_timeout` crate, or a thread + kill)
- task pattern (HUN-267, PRD FR-005): `[A-Z][A-Z0-9]+-\d+` after upper-casing, as in `git show 2d497fc:src/easy_note/core/context.py`; the `regex` crate is fine here (no look-around)
- AC 7 "add still writes the note" is checked once HUN-269 is on main; before that, assert `capture` returns no fields on timeout
- Code lives in the one crate from HUN-268 (`src/core/…`); tests in `tests/*.rs` run with `cargo test`. `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` stay green.
- Search Before You Build: prefer a maintained crate over hand-written code where one fits; record each crate and version in the review file.

---

## Dependencies & Reachability

**Depends on**: HUN-268 (R1) — cargo skeleton. HUN-251 findings (cwd order) stay valid. The store link (passing context fields to `store::add`) is wired when HUN-269 is on main; until then test `capture` on its own

**Entry point**: `easy_note_mcp::core::context::capture` (Rust)

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | Main repo → `repo` = folder name; linked worktree → `repo` = main worktree folder name (via `git rev-parse --git-common-dir`) | FR-004, G5 |
| 2 | No git → `repo`, `remote`, `branch`, `task` all absent | FR-004 |
| 3 | Detached HEAD → `branch` absent, `repo` present | FR-004 |
| 4 | `https://user:tok@github.com/o/r.git`, `git@github.com:o/r.git`, `ssh://git@host/o/r` → `o/r`; the token appears in no written file (grep) | FR-004, G6 |
| 5 | Branch `fix/t120-x` → `T120`; `feat/HUN-221-y` → `HUN-221`; `main` → absent; explicit `task` wins | FR-005 |
| 6 | cwd order = explicit `cwd` → MCP roots → process cwd, as decided in T001's doc | FR-004 |
| 7 | A git call that exceeds 2 s → context fields absent, `add` still writes the note | NFR-001, no capture path drops text |
| 8 | Only `rev-parse`, `branch --show-current`, `remote get-url origin` are run (subprocess spy) | NFR-001 |
| 9 | No absolute path or cwd appears in any written note (grep of the temp dir for the temp root) | FR-004 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

**Locked oracle**: the ported Rust tests named in **Rust port rules**

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | Temp repo + worktree `wt-t148` on branch `fix/t148-x` | `repo` = main folder, `task` = `T148` | automated test |
| 2 | Fake `git` that sleeps 3 s on PATH | note saved, no context fields | automated test |

### Verification Command (exact, runnable)

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-271.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-271.md` (review sections are never taken from the board).

BEFORE command: `date -u; ls src/core/; cargo test 2>&1 | tail -3`

## UI / Design Acceptance Criteria

N/A — pure backend task; no UI component. The three UI Evidence rows are ☐ N/A.

---

## Edge Case Checklist

- [ ] `T\d{3,4}` must not match inside words like `art1234` — anchor on `/` or `-` boundaries
- [ ] Bare repo or `.git` file pointing elsewhere
- [ ] Remote with port `ssh://git@host:2222/o/r.git`
- [ ] Branch names with uppercase task in lower case (`fix/t120`) → `T120`
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: for each fact this task changes, grep README, `docs/`, comments and docstrings for the old wording

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/core/context.rs` | new |
| `tests/context.rs` | new: ported oracle, temp git repos |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md` | Supervisor-owned; report needed changes instead |
| `docs/ddr/*` | Decisions are owner-approved |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `templates/`, `packs/` | Kit files; Supervisor-only |
| Kit tooling scripts listed in Out of scope | They run the pipeline |
| The real `~/easy-notes/`, `~/.config/easy-note/`, `~/.claude/`, `~/.codex/`, `~/.cursor/` | Tests use a temp `HOME` and `EASY_NOTE_DIR` only (NFR-003) |

---

## Completion Checklist

- [ ] Ported oracle red, then green
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Lint passes (`cargo fmt --check`, `cargo clippy -D warnings`)
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log
