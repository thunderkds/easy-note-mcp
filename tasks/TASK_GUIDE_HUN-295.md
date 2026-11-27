**Date**: 2026-11-25
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P0
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

 1. Read `PROJECT_SPEC.md` (glossary: *Note*, *AI detail*, *Project*, *Topic*, *Kind*, *Override*)
 2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
 3. Read this file completely
 4. Read your agent guide (`agents/backend.md`)
 5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in your agent guide
 6. Read `memory/codebase-map.md`
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `ai enrich --pending`, `after_save`, `lock`, `tray.lock`, `HUN-275` before reading source files
 8. Read `PRD.md` (US-020, FR-047, FR-048, FR-049, NFR-014), `GRILL_round6.md`, `BRAINSTORMING_LOG_round6.md` (Path B), `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0007-ai-detail-from-the-local-claude-cli.md`
 9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`. A fake `claude` script stands in for the model; no test starts the real CLI
10. Round 6 rule: **a raw note file is never changed by the AI** (NFR-012, NFR-015). If your change needs one, stop and write a `Question:`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-25: "easy-note is the place where I only take notes; the AI does the rest — arranges the notes, checks drafts, enriches them on a schedule." Decisions: `GRILL_round6.md` G1–G5, Path B in `BRAINSTORMING_LOG_round6.md`.

**Restated intent**:

> One command runs the AI over everything new or stale, within the limits: at most `max_per_run` notes, in batches of `batch_size`, never two runs at once, and a quiet back-off when `claude` fails. It is the command the tray timer, cron and the window's `Run now` all call.

**Out of scope**:

* The tray timer
* The digest job (it plugs into `jobs` later)
* A window or Settings screen

**Requirement Refs**:

* FR-047: pending selection, caps, batches
* FR-048: `[ai]` keys
* FR-049: lock `ai-run.lock`, back-off
* NFR-014: opt-in; nothing runs while `enrich` is off
* US-020

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [x] Restated intent confirmed to match the user's request
- [x] Domain terms align with `PROJECT_SPEC.md` glossary
- [x] Every Acceptance Criterion below traces to a line in the Requirement
- [x] All Requirement Refs exist in `PRD.md` and are covered

---

## Dependencies & Reachability

**Depends on**: HUN-294 (batch engine), HUN-302 (undo for the AI title, summary and tags)

**Entry point**: `easy-note ai run [--limit N] [--dry-run]`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | With `[ai] enrich` not `"claude"`, `ai run` says how to turn it on, exits 1 and starts nothing (as `ai enrich`) | FR-037, NFR-014 |
| 2 | It picks notes with no AI file or a stale one (same test as `pending`), newest first, at most `min(--limit, max_per_run)`, and runs them in batches of `batch_size` | FR-047 |
| 3 | `--dry-run` prints how many notes, how many calls and the model, starts no child and writes nothing | FR-047 |
| 4 | An exclusive lock `<notes dir>/.easy-note/ai-run.lock`: a second `ai run` exits 0 with `already running`; a lock from a dead process never blocks | FR-049 |
| 5 | State file `<notes dir>/.easy-note/ai-run-state.json` holds last run time, counts, `failures`, `next_after` and one line `last_error`; a failed call doubles the wait (from `interval_min`, cap 4 h); a good run resets it; `ai run` called by hand ignores `next_after` but still records | FR-049 |
| 6 | When `jobs` lacks `organize` and `drafts`, the run does nothing and says so | FR-048 |
| 7 | The last line is one summary: `organized N, skipped M, left K pending, calls C`; exit 0 on success or partial, 1 when every call failed | FR-047 |
| 8 | The pending test for organizing is: no AI file, or a stale one (as `ai::pending`), OR an AI file whose `organized_at` is missing or older than its `at`. A note that `ai::enrich` wrote but `ai::organize` could not (the batch engine reports `Enriched`) is therefore picked up again by the next run. It needs no change to `ai::enrich` or `ai::organize` | FR-047 |
| 9 | *(changed 2026-11-27)* The run calls `overrides::read` once per run and passes the loaded overrides on; it never loops over `ai::effective`. Just before it writes a batch, under the write lock, it reads the overrides again, so a fix made during the call is kept | FR-050 |
| 10 | *(added 2026-11-27)* After `ai undo` takes back an AI change, `ai run` skips that note until its text changes, also when the undo happens while the run's call is in flight. A successful `ai enrich <id>` ends the hold. Undo of a person's own fix sets no hold | NFR-015, owner 2026-11-27 |
| 11 | *(added 2026-11-27)* One write lock per notes dir `.easy-note/ai-write.lock` is held around every AI file write, `ai set`, `ai undo` and `ai rm`, only while writing, never during the `claude` call | FR-050, owner 2026-11-27 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | 5 pending notes, batch_size 2, fake claude | 3 calls, 5 AI files, state file updated | tests/ai_run.rs |
| 2 | two runs at once | second exits 0, one child only | same |
| 3 | fake claude fails 2 runs in a row | `failures` 2, `next_after` = 4 × interval | same |
| 4 | `--dry-run` | no child, no file written | same |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-295.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-295.md` (review sections are never taken from the board).

## Approach

**Pattern reference**: `enrich::run` (`--pending`, `--limit`), `tray::acquire_lock`, `enrich::after_save` (detached spawn).

**Vital slice**: Selection + batching loop + lock + state file + summary line.

**Cut list**: A progress bar; a per-job time budget; notifications.

---

## Edge Case Checklist

- [ ] The notes dir is on a read-only mount: clear error, exit 1
- [ ] The clock jumps back: `next_after` is compared with the monotonic-free wall clock; a `next_after` more than 4 h ahead is treated as expired
- [ ] 0 pending notes: print `nothing to do`, exit 0, no child
- [ ] A note saved while the run works: picked up next run
- [ ] Cron runs it with a bare env (no `HOME`): the existing config error text, exit 1
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for text this change makes wrong

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/cli.rs` | `ai run` subcommand |
| `src/surfaces/organize.rs` | the loop, lock and state |
| `src/core/config.rs` | none expected; call only |
| `src/core/ai.rs`, `src/core/overrides.rs` | write lock, hold, run selection (AC 8, 10, 11) |
| `tests/organize.rs` | `run_batch` takes the loaded overrides (AC 9) |
| `docs/NOTE_FORMAT.md` | hold and lock files |
| new `tests/ai_run.rs` | AC 1–7 |
| `README.md`, `RUNBOOK.md` | `ai run` and a cron example |
| Pre-existing pin tests | list each test that pins current behavior (help text, asset list, README, MCP tool count) and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*`, `docs/design/*` | Supervisor / owner owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| Note files and the note writer (`Store::update`, `Store::add` in `src/core/store.rs`) | The AI never changes a raw note (NFR-012, NFR-015); call existing public functions only; a missing one is a `Question:` to the Supervisor |
| `src/core/guard.rs` patterns | Secret guard rules are frozen; call them, never edit them |

---

## Test Plan

Fake `claude` with a counter file to count calls; temp notes dir; lock tested with a child process; state file read back.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] **easy-verifier**: Stage 4 `Skill({ skill: "score" })` run by the Supervisor
- [ ] Security review: `Skill({ skill: "security-review" })` run (note text reaches the model, or a new write path)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md` (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-25 DECIDED scope and order: owner, chat 2026-11-25 ("let go")
* 2026-11-27 DECIDED owner, chat: after an AI undo, hold the note until its text changes (`ai enrich <id>` ends the hold) → AC 10
* 2026-11-27 DECIDED owner, chat: one short write lock and a second read of the overrides before writing → AC 9 changed, AC 11
