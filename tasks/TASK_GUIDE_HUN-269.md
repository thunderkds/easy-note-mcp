**Date**: 2026-11-15
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P0
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`
**Plan ID**: R2 (DDR 0005; Rust port of HUN-253)

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

> **Rust port (DDR 0005).** The requirement and Acceptance Criteria below are the Python HUN-253 ones, kept word for word: behaviour, numbers, limits and error reasons must not change. Python names (`store.add`, `os.link`, `pytest`, `.py` paths) stand for their Rust equivalents; see **Rust port rules**.

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

- [x] Restated intent confirmed to match the user's request (Supervisor, auto; owner accepted DDR 0005 and the R1–R5 plan on 2026-11-15)
- [x] Domain terms align with `PROJECT_SPEC.md` glossary
- [x] Every Acceptance Criterion below traces to a line in the Requirement
- [x] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

---

## Rust port rules

**Oracle (port first, red):** `git show 2d497fc:tests/test_format.py`, `git show 2d497fc:tests/test_store.py` (Python HUN-253), and the fixtures under `tests/fixtures/`. Port every test case to Rust (same inputs, same expected output) before the implementation. Where a Python test checks something Rust makes impossible (e.g. a type error that cannot compile), replace it with the nearest Rust check and list it in the review file.

- `os.link` / `O_EXCL` → `std::fs::hard_link` from a temp file made with `OpenOptions::create_new`, then `sync_all`; `os.replace` → `std::fs::rename` in the same dir
- sha256 → the `sha2` crate; local time with offset → `time` or `chrono` (pick one, record it)
- front matter: a hand-written reader/writer for the DDR 0002 subset, or a maintained YAML crate (not the archived `serde_yaml`); AC 1 (byte-identical round-trip) decides
- AC 10 (NOTE_FORMAT Final) is already done on main (0b5d044): keep `docs/NOTE_FORMAT.md` as is unless the code proves it wrong
- Code lives in the one crate from HUN-268 (`src/core/…`); tests in `tests/*.rs` run with `cargo test`. `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` stay green.
- Search Before You Build: prefer a maintained crate over hand-written code where one fits; record each crate and version in the review file.

---

## Dependencies & Reachability

**Depends on**: HUN-268 (R1) — cargo skeleton and test runner

**Entry point**: `easy_note_mcp::core::store::add` (Rust)

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

**Locked oracle**: the ported Rust tests named in **Rust port rules**

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | Two `store.add('x')` calls in one frozen second | two files, second has `-2` | automated test |
| 2 | Monkeypatched `os.replace` raising | no new `.md` under `YYYY/MM/` | automated test |
| 3 | A file with `easy-note: 2` | in `skipped`, bytes unchanged | automated test |

### Verification Command (exact, runnable)

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-269.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-269.md` (review sections are never taken from the board).

BEFORE command: `date -u; ls src/core/; cargo test 2>&1 | tail -3`

## UI / Design Acceptance Criteria

N/A — pure backend task; no UI component. The three UI Evidence rows are ☐ N/A.

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
| `src/core/config.rs` | new: notes dir (`EASY_NOTE_DIR` → config → `~/easy-notes`) |
| `src/core/format.rs` | new: parse/render, DDR 0002 subset |
| `src/core/store.rs` | new: add, read, update, rm, restore, empty_trash, iter_notes, Skipped |
| `src/core/mod.rs`, `src/lib.rs` (if needed) | export modules |
| `tests/format.rs`, `tests/store.rs` | new: ported oracle |
| `docs/NOTE_FORMAT.md` | only if a Rust detail changes wording (it must not change the format) |

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
