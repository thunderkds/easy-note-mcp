**Date**: 2026-11-15
**Complexity Level**: C1
**Risk Level**: High
**Priority**: P0
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`
**Plan ID**: R3 (DDR 0005; Rust port of HUN-255, never merged in Python)

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

> **Rust port (DDR 0005).** The requirement and ACs 1–7 below are the Python HUN-255 ones, kept word for word. ACs 8–10 add the contract settled in HUN-255's Stage 4 review (verified Python branch `feat/HUN-255-secret-guard` at 3c5e40f). Python names stand for their Rust equivalents.

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

- [x] Restated intent confirmed to match the user's request (Supervisor, auto; owner accepted DDR 0005 and the R1–R5 plan on 2026-11-15)
- [x] Domain terms align with `PROJECT_SPEC.md` glossary
- [x] Every Acceptance Criterion below traces to a line in the Requirement
- [x] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

---

## Rust port rules

**Oracle (port first, red), by qa-expert, not the implementer:** `git show 3c5e40f:tests/test_guard.py`, `git show 3c5e40f:tests/test_leak.py`, `git show 3c5e40f:tests/test_store.py` (`test_update_never_writes_a_file_that_does_not_parse`), and the fixtures `git show 3c5e40f:tests/fixtures/secrets/planted.json` and `dev_notes_50.txt` (rewrite planted values so no committed literal trips push protection; see Edge Cases).

- **Part 1** (depends on HUN-268): `src/core/guard.rs` with `KINDS`, `mask(text) -> (String, Vec<&str>)`, `mask_for_output(text) -> String`; the same ten kinds in the same declared order, overlap = union with longest-span kind, idempotent, never logs. Reference: `git show 3c5e40f:src/easy_note/core/guard.py`.
- **Part 2** (depends on HUN-269 on main): wire the guard into `src/core/store.rs` per ACs 3, 4, 8–10. Reference: `git show 3c5e40f:src/easy_note/core/store.py`.
- Code lives in the one crate from HUN-268; tests in `tests/*.rs`; `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` stay green. Record each crate and version in the review file.

---

## Dependencies & Reachability

**Depends on**: HUN-268 (R1) for part 1; HUN-269 (R2) for part 2

**Entry point**: `easy_note_mcp::core::guard::mask` (Rust)

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
| 8 | Masking covers every free-text part on write and read: body, every tag, every context field value, and unknown front-matter text (`extra`); `id`, `created`, `updated`, `source` and the version key are never masked | FR-014, NFR-002 (HUN-255 security review) |
| 9 | `store::add` returns `Added { id, path, created_dir, masked }`; `store::update` returns `Updated { note, masked }`; `masked` lists kinds in order body, tags, context fields (CONTEXT_KEYS order for update, given order for add), extra; repeats kept; a clean write reports nothing | FR-014 (HUN-255 decision 8, 3-Advocate debate) |
| 10 | `store::update` renders, parses the result back, and returns a format error with no note text and writes nothing when masking would make the file unparseable; the size check runs on the masked text | FR-008, FR-014 (HUN-255 decision 10) |

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

**Locked oracle**: `tests/leak.rs` and `tests/guard.rs` (qa-expert)

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | `store.add('key AKIA…EXAMPLE and ghp_…')` | file holds `[masked:aws_key]`, `[masked:github_token]`; result `masked=2` | automated test |
| 2 | Hand-written file with a JWT, read via core | output masked, file bytes unchanged | automated test |

### Verification Command (exact, runnable)

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-270.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-270.md` (review sections are never taken from the board).

BEFORE command: `date -u; ls src/core/; cargo test 2>&1 | tail -3`

## UI / Design Acceptance Criteria

N/A — pure backend task; no UI component. The three UI Evidence rows are ☐ N/A.

---

## Edge Case Checklist

- [ ] Overlapping matches (a JWT inside `password=`) — mask once, longest span wins
- [ ] Secrets split across lines (private key block) — mask the whole block
- [ ] Git commit shas and `sha256:` values must not be masked
- [ ] Masked marker itself must not re-match on a second pass
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: for each fact this task changes, grep README, `docs/`, comments and docstrings for the old wording

---
- [ ] GitHub push protection: no fake token literal that looks like a real Slack, GitHub, AWS, OpenAI or Anthropic key may appear in any committed file (the Python branch was blocked on `xoxb-…`). Build planted values at runtime (`concat!`, `format!`, or joining parts), keep them fake, and check with `git diff | grep -E "xox[abprs]-|ghp_|AKIA|sk-"` before each commit
- [ ] Look-around: the Python patterns use `(?!...)`; the `regex` crate has no look-around. Restate them, or use `fancy-regex` for those patterns only; the leak oracle and the near-miss tests decide

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/core/guard.rs` | new (part 1) |
| `src/core/store.rs` | call the guard on write and read (part 2) |
| `tests/guard.rs`, `tests/leak.rs` | new: ported oracle (qa-expert) |
| `tests/fixtures/secrets/` | planted values, rewritten so no literal trips push protection |

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
