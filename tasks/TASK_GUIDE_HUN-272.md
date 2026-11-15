**Date**: 2026-11-15
**Complexity Level**: C1
**Risk Level**: Low
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`
**Plan ID**: R5 (DDR 0005; Rust port of HUN-256)

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

> **Rust port (DDR 0005).** The requirement and Acceptance Criteria below are the Python HUN-256 ones, kept word for word: behaviour, numbers, limits and error reasons must not change. Python names (`store.add`, `os.link`, `pytest`, `.py` paths) stand for their Rust equivalents; see **Rust port rules**.

## Requirement (Pillar 1 — Adapt the requirement)

Plan T006: "Tests: AND-of-words, case-insensitive; each filter; local-date `day`/`since`/`until` across a timezone offset; newest-first order stable for equal times (by ID); `limit` cap 50 with `total` + `truncated`; snippet ≤ 200 chars; stats counts by repo/task/tag/source/month and `skipped`; same output twice on the same files."

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> Deterministic list, search and stats over the notes folder that both the CLI and MCP call, bounded so an agent never gets an unbounded answer.

**Out of scope** (what this task explicitly does NOT do):

* Any index, ranking or embeddings (D9)
* Searching AI files (T014 adds `matched_in: ai`)
* Inbox files (T013 plugs in)

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-010: list filters
* FR-011: search AND of words
* FR-015: stats
* FR-016: ≤ 50 items, snippet ≤ 200, total, truncated
* NFR-004: provisional speed
* NFR-007: deterministic

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [x] Restated intent confirmed to match the user's request (Supervisor, auto; owner accepted DDR 0005 and the R1–R5 plan on 2026-11-15)
- [x] Domain terms align with `PROJECT_SPEC.md` glossary
- [x] Every Acceptance Criterion below traces to a line in the Requirement
- [x] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

---

## Rust port rules

**Oracle (port first, red):** `git show 2d497fc:tests/test_query.py` and `tests/fixtures/notes_basic/`. Port every test case to Rust (same inputs, same expected output) before the implementation. Where a Python test checks something Rust makes impossible (e.g. a type error that cannot compile), replace it with the nearest Rust check and list it in the review file.

- results as plain structs (serde `Serialize` for the MCP surface later); the item fields and caps (50 items, snippet ≤ 200 chars, `total`, `truncated`) are exact
- local dates use each note's own `created` offset, as in the Python code
- Code lives in the one crate from HUN-268 (`src/core/…`); tests in `tests/*.rs` run with `cargo test`. `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` stay green.
- Search Before You Build: prefer a maintained crate over hand-written code where one fits; record each crate and version in the review file.

---

## Dependencies & Reachability

**Depends on**: HUN-269 (R2) — parsed notes and skipped list from the store

**Entry point**: `easy_note_mcp::core::query::search` (Rust)

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `search('retry firefox')` matches only notes holding both words (any case) in title, body or tags | FR-011 |
| 2 | Filters `repo`, `task`, `tag`, `source`, `since`, `until`, `day` each narrow results; dates are local dates of `created` | FR-010 |
| 3 | A note created `23:30+07:00` counts for its local day, not the UTC day | D10 |
| 4 | Order: newest `created` instant first; ties broken by ID; two runs give identical output | NFR-007 |
| 5 | `limit` > 50 is capped at 50; result has `total` and `truncated` | FR-016 |
| 6 | Each item has id, title, created, repo, task, tags and a snippet ≤ 200 chars around the first hit | FR-016 |
| 7 | `stats()` returns counts by repo, task, tag, source, month, plus oldest/newest date and `skipped` with paths; no note text | FR-015, FR-017 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

**Locked oracle**: the ported Rust tests named in **Rust port rules**

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | Fixture of 30 notes across 3 repos and 2 timezones | expected ids in expected order | automated test |
| 2 | `stats()` output | contains no body text (assert no fixture sentence appears) | automated test |

### Verification Command (exact, runnable)

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-272.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-272.md` (review sections are never taken from the board).

BEFORE command: `date -u; ls src/core/; cargo test 2>&1 | tail -3`

## UI / Design Acceptance Criteria

N/A — pure backend task; no UI component. The three UI Evidence rows are ☐ N/A.

---

## Edge Case Checklist

- [ ] Words with diacritics — use `casefold()`
- [ ] Empty query → error, not every note
- [ ] Tag filter `a/b` hierarchical tags match exactly
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: for each fact this task changes, grep README, `docs/`, comments and docstrings for the old wording

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/core/query.rs` | new: list, search, stats |
| `tests/query.rs` | new: ported oracle |

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
