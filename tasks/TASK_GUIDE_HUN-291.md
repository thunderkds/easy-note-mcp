**Date**: 2026-11-25
**Complexity Level**: C1
**Risk Level**: Low
**Priority**: P1
**Assigned agent**: Backend-Implementer + Frontend-Implementer
**Agent guide**: `agents/frontend.md, agents/backend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

 1. Read `PROJECT_SPEC.md` (glossary: *Note*, *AI detail*, *Project*, *Topic*, *Kind*, *Override*)
 2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
 3. Read this file completely
 4. Read your agent guide (`agents/frontend.md`)
 5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in your agent guide
 6. Read `memory/codebase-map.md`
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `updated`, `edit`, `query`, `board`, `HUN-286` before reading source files
 8. Read `PRD.md` (US-023, FR-010, FR-016, FR-046), `GRILL_round6.md`, `BRAINSTORMING_LOG_round6.md` (Path B), `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0007-ai-detail-from-the-local-claude-cli.md`
 9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`. A fake `claude` script stands in for the model; no test starts the real CLI
10. Round 6 rule: **a raw note file is never changed by the AI** (NFR-012, NFR-015). If your change needs one, stop and write a `Question:`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-25: "easy-note is the place where I only take notes; the AI does the rest — arranges the notes, checks drafts, enriches them on a schedule." Decisions: `GRILL_round6.md` G1–G5, Path B in `BRAINSTORMING_LOG_round6.md`.

**Restated intent**:

> A note that was edited shows when. `ls`, `search`, MCP list/search and the dashboard window show `updated` next to `created`. Only the person's own edits set it; AI runs never do.

**Out of scope**:

* Changing how `updated` is set (it stays: `edit`, `tag`, window Edit, Add as tags)
* Sorting by `updated` (order stays newest `created` first)
* An `updated` for inbox notes (they have none)

**Requirement Refs**:

* FR-046: updated shown in list output and window
* FR-016: MCP item shape gains one optional key
* NFR-012, NFR-015: AI never sets `updated`
* US-023

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [x] Restated intent confirmed to match the user's request
- [x] Domain terms align with `PROJECT_SPEC.md` glossary
- [x] Every Acceptance Criterion below traces to a line in the Requirement
- [x] All Requirement Refs exist in `PRD.md` and are covered

Signed: auto (Supervisor), 2026-11-26.

---

## Dependencies & Reachability

**Depends on**: None — can start immediately

**Entry point**: `easy-note ls|search|show`, MCP `note_list` / `note_search` / `note_read`, window `board` and `detail` commands

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `easy-note ls` and `search` print an `edited <when>` column only for notes that have `updated`; notes without it print nothing extra | FR-046 |
| 2 | `--json` output and MCP list/search/read items carry `updated` (ISO 8601) when present and omit the key when absent; the key is additive, no existing key changes | FR-016, FR-046 |
| 3 | The window's board item and list row show `edited 2h ago` (relative, local time); the detail panel shows the full stamp | FR-046 |
| 4 | `ai enrich` and the batch organizer never set or change `updated` (a test proves note file bytes and `updated` are unchanged); `tag --accept-ai` does set it, because it edits the note | NFR-012, FR-046 |
| 5 | A note whose `updated` is earlier than `created` (clock set back) shows the stamp without a negative duration | FR-046 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | a note edited with `easy-note edit` | `ls` shows `edited just now`; MCP item has `updated` | tests/cli.rs, tests/mcp.rs |
| 2 | a note never edited | no `edited` text, no key | same tests |
| 3 | window board payload for both notes | `updated` only on the edited one | tests/window_api.rs + page harness |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features window,tray -- -D warnings \
  && cargo test --locked --features window,tray
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-291.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-291.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Notes list row and detail panel with `edited` text | easy-ui screenshot, stub board with 2 notes | `edited 2h ago` beside the time; none on the never-edited note |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Colors match design tokens | CSS audit vs `docs/design/graphite-mint-tokens.json` | only existing tokens |
| Typography matches spec | computed style | bundled Plex fonts, as the dashboard |
| Spacing / layout matches spec | computed style | same spacing scale as the existing board |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| 375 px | easy-ui screenshot, `Skill({ skill: "ux-evaluate" })` | no cut-off text, no horizontal scroll |
| 1280 px | easy-ui screenshot | as the design board |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-023 | know if a note changed | open the board | edited notes show a quiet `edited …` text; the row is not taller |

---

## Approach

**Pattern reference**: `Store::update` sets `updated` (`src/core/store.rs`); `note_json` in `src/surfaces/mcp.rs`; the board payload in `src/surfaces/window/api.rs`.

**Vital slice**: Expose `updated` in the query item and the three outputs; one shared relative-time helper.

**Cut list**: A filter `--edited-since`; sorting by edit time.

---

## Edge Case Checklist

- [ ] `updated` with another offset than `created`: show in local time
- [ ] Relative time at 59 s, 59 min, 23 h, 30 d boundaries
- [ ] A skipped (unreadable) file never reaches the list
- [ ] The MCP size limit (FR-016: ≤ 50 items, snippet ≤ 200) still holds
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for text this change makes wrong

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/core/query.rs` | item carries `updated` |
| `src/surfaces/cli.rs`, `src/surfaces/cli/read.rs` | `edited` column and JSON key |
| `src/surfaces/mcp.rs` | optional `updated` in the item |
| `src/surfaces/window/api.rs`, `assets/app.js`, `assets/app.css` | payload key, row text, detail stamp |
| `tests/cli.rs`, `tests/mcp.rs`, `tests/window_api.rs`, `tests/fixtures/window/page_harness.js` | AC 1–5 |
| `docs/NOTE_FORMAT.md` | state that `updated` is read-only for AI |
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

Fixture dir with one edited and one never-edited note, through each surface. Relative-time helper as a table-driven unit test. Page harness for the row and detail text.

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