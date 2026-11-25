**Date**: 2026-11-25
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P1
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
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `tray`, `ksni`, `spawn_detached`, `HUN-263`, `HUN-285` before reading source files
 8. Read `PRD.md` (US-020, FR-033, FR-048, FR-049, NFR-014), `GRILL_round6.md`, `BRAINSTORMING_LOG_round6.md` (Path B), `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0007-ai-detail-from-the-local-claude-cli.md`
 9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`. A fake `claude` script stands in for the model; no test starts the real CLI
10. Round 6 rule: **a raw note file is never changed by the AI** (NFR-012, NFR-015). If your change needs one, stop and write a `Question:`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-25: "easy-note is the place where I only take notes; the AI does the rest — arranges the notes, checks drafts, enriches them on a schedule." Decisions: `GRILL_round6.md` G1–G5, Path B in `BRAINSTORMING_LOG_round6.md`.

**Restated intent**:

> While the tray runs, `[ai] schedule = "tray"` and `enrich = "claude"`, the tray starts `easy-note ai run` every `interval_min` minutes, never two at once. Settings changes apply on the next tick with no tray restart.

**Out of scope**:

* Autostart changes
* A daemon of our own; systemd units
* The digest hour (rides in `jobs` via the digest slice)

**Requirement Refs**:

* FR-049: tray timer, back-off, one run at a time
* FR-048: schedule keys
* NFR-014: nothing runs while either switch is off
* US-020

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are covered

---

## Dependencies & Reachability

**Depends on**: HUN-295 (`ai run`)

**Entry point**: `easy-note tray` (a timer thread in `src/surfaces/tray.rs`)

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | Each tick re-reads config.toml; when `effective_schedule()` is true and the state file's `next_after` has passed, the tray spawns `<exe> ai run` detached (null stdio) and does not wait for it | FR-049 |
| 2 | A tick with the run lock held, `schedule = off`, or `enrich` off starts nothing | FR-049, NFR-014 |
| 3 | The first tick happens `interval_min` after the tray starts, not at once | FR-049 |
| 4 | The tick function takes the clock, the config and a spawner as inputs, so it is tested without a tray host | FR-049 |
| 5 | The tray menu is unchanged; one status line `AI: last run 10:05, 3 notes` is not added (cut) | FR-033 |
| 6 | With no tray host the tray exits as today and starts no run | FR-033 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | tick, schedule on, next_after in the past | one spawn | tests/tray.rs (fake spawner, fake clock) |
| 2 | tick, lock held | no spawn | same |
| 3 | tick, schedule off | no spawn | same |
| 4 | config changed between ticks | second tick follows the new interval | same |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-296.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-296.md` (review sections are never taken from the board).

## Approach

**Pattern reference**: `tray::spawn_detached`, the 1 s host check loop (HUN-285), `tests/tray.rs` fakes.

**Vital slice**: A pure `should_run(now, config, state, lock_held)` plus a timer thread that calls it.

**Cut list**: A menu status line; a `Run AI now` menu item.

---

## Edge Case Checklist

- [ ] Machine sleeps for hours: one run at wake, not a burst
- [ ] Config file missing or invalid at a tick: skip the tick, log one line, keep the timer
- [ ] `ai run` binary path vanished (upgrade): skip, one line
- [ ] The tray starts at login before the notes dir is mounted: skip until it exists
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for text this change makes wrong

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/tray.rs` | timer thread, `should_run` |
| `tests/tray.rs` | AC 1–6 |
| `README.md` | one paragraph: the tray is the timer |
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

`should_run` as a table-driven unit test; the spawner is a closure recording calls.

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