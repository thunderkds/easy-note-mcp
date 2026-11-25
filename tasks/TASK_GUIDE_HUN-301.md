**Date**: 2026-11-25
**Complexity Level**: C1
**Risk Level**: Low
**Priority**: P2
**Assigned agent**: Frontend-Implementer
**Agent guide**: `agents/frontend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

 1. Read `PROJECT_SPEC.md` (glossary: *Note*, *AI detail*, *Project*, *Topic*, *Kind*, *Override*)
 2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
 3. Read this file completely
 4. Read your agent guide (`agents/frontend.md`)
 5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in your agent guide
 6. Read `memory/codebase-map.md`
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `app.js` key handling, `editKey`, `empty`, `ux-evaluate`, `HUN-278`, `HUN-280` before reading source files
 8. Read `PRD.md` (US-023, FR-053), `GRILL_round6.md`, `BRAINSTORMING_LOG_round6.md` (Path B), `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0007-ai-detail-from-the-local-claude-cli.md`
 9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`. A fake `claude` script stands in for the model; no test starts the real CLI
10. Round 6 rule: **a raw note file is never changed by the AI** (NFR-012, NFR-015). If your change needs one, stop and write a `Question:`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-25: "easy-note is the place where I only take notes; the AI does the rest — arranges the notes, checks drafts, enriches them on a schedule." Decisions: `GRILL_round6.md` G1–G5, Path B in `BRAINSTORMING_LOG_round6.md`.

**Restated intent**:

> The window feels quick and clear to use from the keyboard, explains itself when empty, and every screen passes the baseline UX checks at 375 px and 1280 px.

**Out of scope**:

* New features
* Re-skinning the design
* Mobile app

**Requirement Refs**:

* FR-053: keys `/ j k e n ?`, empty state, `ux-evaluate`
* US-023

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are covered

---

## Dependencies & Reachability

**Depends on**: HUN-298, HUN-299, HUN-300 (all screens exist)

**Entry point**: The dashboard window (`src/surfaces/window/assets/*`)

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `/` focuses search, `j` / `k` move the selection down and up through the visible notes, `e` edits the selected note, `n` opens the new-note dialog, `?` shows the key list, `Esc` closes any dialog; keys do nothing while typing in a field or during IME composition | FR-053 |
| 2 | An empty board (no notes) shows a first-run card: what easy-note is, how to save a note (`n`, hotkey, tray), and that the AI arranges notes after the owner turns it on in Settings, with a link to Settings | FR-053 |
| 3 | Every screen built in this round (board, groupings, Drafts, detail, fixes, Settings, notice) is checked with `Skill({ skill: "ux-evaluate" })` at 375 px and 1280 px; every fail is fixed or listed with a reason in the review file | FR-053 |
| 4 | Focus order and visible focus ring on every control; every icon-only button has an accessible name | FR-053 |
| 5 | No new network request and no new dependency | NFR-001 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | page harness: press `j`, `j`, `e` | second note selected, edit mode open | page harness |
| 2 | empty board stub | first-run card visible with the Settings link | page harness + screenshot |
| 3 | `ux-evaluate` run | all baseline checks pass or a written reason | pasted into the review file |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features window,tray -- -D warnings \
  && cargo test --locked --features window,tray
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-301.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-301.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Every screen of the window, plus the empty state and the `?` key list | easy-ui screenshots at 375 and 1280 px | no clipped text; focus ring visible |

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
| US-023 | use the board with the keyboard | `/`, type, `j`, `e` | search, select, edit without the mouse |
| US-023 | first run | open with no notes | the card tells how to start |

---

## Approach

**Pattern reference**: `editKey` and the capture dialog key code in `assets/app.js` / `capture.js`.

**Vital slice**: The key map, the empty state, the evaluate-and-fix loop.

**Cut list**: User-defined shortcuts; command palette; animations.

---

## Edge Case Checklist

- [ ] Keys while a dialog is open
- [ ] `j` at the last note, `k` at the first
- [ ] Non-US keyboard layouts: use `key`, not `keyCode`; `?` needs Shift
- [ ] Screen reader: selection changes announced
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for text this change makes wrong

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/window/assets/app.js`, `app.css`, `index.html` | keys, empty card, focus styles |
| `tests/fixtures/window/page_harness.js` | key and empty-state cases |
| `README.md` | key list |
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

Page harness for keys and the empty state; screenshots at 375 and 1280 px for every screen; the `ux-evaluate` report pasted.

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

* 2026-11-25 OPEN HITL: owner design review of the UX pass before merge