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
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `ai file`, `Detail`, `note_enrich`, `ai rm`, `based_on`, `HUN-262` before reading source files
 8. Read `PRD.md` (US-021, FR-030, FR-032, FR-050, NFR-015), `GRILL_round6.md`, `BRAINSTORMING_LOG_round6.md` (Path B), `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0007-ai-detail-from-the-local-claude-cli.md`
 9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`. A fake `claude` script stands in for the model; no test starts the real CLI
10. Round 6 rule: **a raw note file is never changed by the AI** (NFR-012, NFR-015). If your change needs one, stop and write a `Question:`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-25: "easy-note is the place where I only take notes; the AI does the rest — arranges the notes, checks drafts, enriches them on a schedule." Decisions: `GRILL_round6.md` G1–G5, Path B in `BRAINSTORMING_LOG_round6.md`.

**Restated intent**:

> The AI file can hold project, topic, draft reason and draft suggestion. A person's fix of project, topic or kind is kept in its own file and wins over the AI. Every AI or person change is logged, and one command (and later one click) undoes the last change of a note.

**Out of scope**:

* The model call (slice `ai run` batch engine)
* Any window or Settings screen
* Digest files
* A daily call cap

**Requirement Refs**:

* FR-050: overrides file, change log, `ai set`, `ai undo`, topic rename map
* FR-047: the AI file keys it will write (`project`, `topic`, `draft_reason`, `draft_suggestion`, `organized_at`)
* FR-030: caps and masking for AI files
* NFR-015: raw note bytes never change
* US-021

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are covered

---

## Dependencies & Reachability

**Depends on**: None — can start immediately

**Entry point**: `easy-note ai set <id> project|topic|kind VALUE`, `easy-note ai undo <id>`; `core::ai`, new `core::overrides`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `Detail` (AI file JSON) accepts optional `project`, `topic`, `draft_reason`, `draft_suggestion`, `organized_at`; a file with none still reads; caps: project ≤ 60, topic ≤ 60, draft_reason ≤ 200, draft_suggestion ≤ 600 chars; every string masked; `kind` accepts `idea`, `todo`, `bug`, `question`, `draft`, `other` | FR-030, FR-047 |
| 2 | `<notes dir>/.easy-note/ai-overrides.json` holds per-note `project|topic|kind` values and a topic rename map; written whole by temp file + rename; values capped and masked; an unknown note id is refused | FR-050 |
| 3 | `core::ai::effective(store, id)` returns the AI values with the person's override applied for each field, and applies the topic rename map | FR-050 |
| 4 | `<notes dir>/.easy-note/ai-log.jsonl` gets one line per change: `id`, `field`, `old`, `new`, `who` (`ai` or `person`), `at`; append only; one line is written whole or not at all | FR-050 |
| 5 | `easy-note ai set <id> topic "firefox flaky"` writes the override and one log line; a bad kind or an empty value exits 1 with the allowed values named | FR-050 |
| 6 | `easy-note ai undo <id>` restores the previous value of that note's latest change (override removed or old value back), logs the undo, and says `nothing to undo` with exit 1 when the log has none | FR-050 |
| 7 | `rm` / `restore` of a note move its override entry and log lines' meaning: an override for a trashed note is kept and applies again after `restore` | FR-009, FR-050 |
| 8 | Fixture test: note file bytes are identical before and after `set`, `undo` and an AI-file write | NFR-015 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | note with AI `topic: a`, person sets topic `b` | `effective` gives `b`; log has the line; the AI file still says `a` | unit + CLI test |
| 2 | `undo` after that | `effective` gives `a` again | same |
| 3 | planted secret in a value | stored as `[masked:<kind>]` | tests/leak.rs addition |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-292.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-292.md` (review sections are never taken from the board).

## Approach

**Pattern reference**: `core::ai` (`Detail`, `enrich`, `based_on`, `check_len`, `read`) for caps and masking; `core::store` for atomic temp+rename.

**Vital slice**: Detail keys + overrides file + log + `effective` + the two CLI subcommands.

**Cut list**: Per-field undo history deeper than the last change; log rotation.

---

## Edge Case Checklist

- [ ] Two `ai set` at once (two windows or a sync tool): last whole-file write wins, the log keeps both lines
- [ ] Log line longer than the OS atomic append size: cap each value so a line stays under 4 KiB
- [ ] Override for an id that no longer exists: kept, ignored by `effective`, listed by `doctor` as an orphan
- [ ] `undo` twice: the second undo restores the change before (walk the log back for that id)
- [ ] Hand-edited, invalid `ai-overrides.json`: named by `doctor`, never overwritten silently; reads fall back to no overrides with one warning line
- [ ] Sync tool copies the AI file from another machine without the override: fine, overrides are separate
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for text this change makes wrong

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/core/ai.rs` | new optional Detail keys, caps, `effective` |
| new `src/core/overrides.rs` | overrides file, rename map, log, undo |
| `src/core/mod.rs` | module line |
| `src/surfaces/cli.rs` | `ai set`, `ai undo` |
| `docs/NOTE_FORMAT.md` | AI file keys and the two new files under `.easy-note/` |
| `tests/ai.rs`, new `tests/overrides.rs`, `tests/leak.rs`, `tests/cli.rs` | AC 1–8 |
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

Unit tests for caps, masking and `effective`; CLI tests over a temp notes dir; a byte-compare fixture for the raw note (NFR-015); the leak suite gains the new values.

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