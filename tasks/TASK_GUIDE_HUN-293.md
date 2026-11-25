**Date**: 2026-11-25
**Complexity Level**: C1
**Risk Level**: Low
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
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `config`, `AiSettings`, `doctor`, `after_save`, `HUN-275` before reading source files
 8. Read `PRD.md` (US-020, FR-038, FR-048), `GRILL_round6.md`, `BRAINSTORMING_LOG_round6.md` (Path B), `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0007-ai-detail-from-the-local-claude-cli.md`
 9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`. A fake `claude` script stands in for the model; no test starts the real CLI
10. Round 6 rule: **a raw note file is never changed by the AI** (NFR-012, NFR-015). If your change needs one, stop and write a `Question:`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-25: "easy-note is the place where I only take notes; the AI does the rest — arranges the notes, checks drafts, enriches them on a schedule." Decisions: `GRILL_round6.md` G1–G5, Path B in `BRAINSTORMING_LOG_round6.md`.

**Restated intent**:

> config.toml `[ai]` can say when the AI runs and how much: `schedule`, `interval_min`, `max_per_run`, `batch_size`, `jobs`, `digest_hour`. Wrong values are named (the key, never the value). `doctor` reports them.

**Out of scope**:

* Writing the file (the Settings slice does that)
* Starting any run
* Env vars for these keys

**Requirement Refs**:

* FR-048: new `[ai]` keys and defaults
* FR-038: existing keys unchanged
* NFR-014: the scheduler is off by default
* US-020

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are covered

---

## Dependencies & Reachability

**Depends on**: None — can start immediately

**Entry point**: `core::config` (`AiSettings`, `ai_of`), `easy-note doctor`, `easy-note ai`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `AiSettings` gains `schedule` (`off` | `tray`, default `off`), `interval_min` (default 60, minimum 5), `max_per_run` (default 20, 1–200), `batch_size` (default 10, 1–50, not above `max_per_run`), `jobs` (list of `organize`, `drafts`, `digest`; default all three), `digest_hour` (default 3, 0–23) |
| 2 | A config with none of the new keys reads as today; every existing key keeps its meaning | FR-038 |
| 3 | A bad value (wrong type, out of range, unknown job) gives an error that names the file and the key and never echoes the value | FR-048 |
| 4 | `schedule = "tray"` without `enrich = "claude"` reads fine but `effective_schedule()` is false; `doctor` says why (`schedule needs enrich = claude`) | NFR-014 |
| 5 | `doctor` prints one line with schedule, interval, per-run cap, batch size, jobs and digest hour | FR-048 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | no `[ai]` section | all defaults, schedule off | unit test |
| 2 | `interval_min = 1` | error names `interval_min`, not `1` | unit test |
| 3 | `doctor` over a temp HOME with the section | the line matches the file | tests/cli.rs |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-293.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-293.md` (review sections are never taken from the board).

## Approach

**Pattern reference**: `ai_of` in `src/core/config.rs` and its `bad(key, rule)` helper; `enrich::doctor_line`.

**Vital slice**: Parse + validate + `effective_schedule` + the doctor line.

**Cut list**: Hot reload code (the tray re-reads the file each tick in its own slice).

---

## Edge Case Checklist

- [ ] `jobs = []`: nothing to do; `ai run` says so (note for the run slice)
- [ ] `batch_size > max_per_run`: refused, names `batch_size`
- [ ] TOML integer given as a string: refused, key named
- [ ] Unknown extra keys in `[ai]` are ignored, as today
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for text this change makes wrong

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/core/config.rs` | new fields, parse, validation, `effective_schedule` |
| `src/surfaces/enrich.rs` | `doctor_line` text |
| `tests/enrich.rs`, `tests/cli.rs` | AC 1–5 |
| `README.md` | the new `[ai]` keys in the config example |
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

Table-driven config parse tests (good and bad), one `doctor` test with a temp HOME.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] **easy-verifier**: Stage 4 `Skill({ skill: "score" })` run by the Supervisor
- [ ] Security review: not required (Low risk)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md` (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-25 DECIDED scope and order: owner, chat 2026-11-25 ("let go")