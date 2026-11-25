**Date**: 2026-11-25
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Frontend-Implementer + Backend-Implementer
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
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `window`, `board`, `app.js`, `filters`, `HUN-278`, `HUN-288` before reading source files
 8. Read `PRD.md` (US-020, FR-042, FR-043, FR-052, NFR-013), `GRILL_round6.md`, `BRAINSTORMING_LOG_round6.md` (Path B), `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0007-ai-detail-from-the-local-claude-cli.md`
 9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`. A fake `claude` script stands in for the model; no test starts the real CLI
10. Round 6 rule: **a raw note file is never changed by the AI** (NFR-012, NFR-015). If your change needs one, stop and write a `Question:`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-25: "easy-note is the place where I only take notes; the AI does the rest — arranges the notes, checks drafts, enriches them on a schedule." Decisions: `GRILL_round6.md` G1–G5, Path B in `BRAINSTORMING_LOG_round6.md`.

**Restated intent**:

> The window shows what the AI arranged. The person switches the grouping between Day, Project, Topic and Kind, opens a Drafts view that shows each draft with the AI's reason and suggested completion, sees on every note whether it is organized, and sees in the header how many notes wait, with a `Run now` button.

**Out of scope**:

* Fixing or renaming (next slice)
* The Settings screen
* Digest display
* Writing anything to a note

**Requirement Refs**:

* FR-052: group-by, Drafts view, AI status line, waiting count and `Run now`
* FR-042, FR-043: window rules, IPC only, strings masked
* NFR-013: board shown p95 < 1 s over 5,000 notes
* US-020

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are covered

---

## Dependencies & Reachability

**Depends on**: HUN-294 (AI file keys written), HUN-295 (`ai run` for `Run now`)

**Entry point**: `easy-note dashboard`: the board page and `board` / `run_ai` IPC commands

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | The board payload carries per note: `project`, `topic`, `kind`, `organized_at`, `draft` (flag), values after overrides; all strings masked | FR-052, FR-042 |
| 2 | A group-by switch (Day, Project, Topic, Kind) regroups the list with counts; the choice is kept in the page for the session; Day stays the default | FR-052 |
| 3 | A Drafts view lists notes with kind `draft`; each row shows the note's first line and the AI's `draft_reason`; the detail panel shows `draft_suggestion` under `AI detail (not your words)`; there is no button that writes it into the note | FR-052, G5 |
| 4 | Each note shows `organized 2h ago`, `waiting` or `not organized`; the header shows `N waiting` and a `Run now` button | FR-052 |
| 5 | `Run now` calls IPC `run_ai`, which starts `ai run` detached (same checks as the CLI: opt-in and lock), shows a toast with the result line, and refreshes on completion; when `enrich` is off the toast says how to turn it on | FR-052, NFR-014 |
| 6 | 5,000 notes: the board shows in under 1 s and regrouping in under 200 ms (measured, pasted) | NFR-013 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | stub board with 6 notes, 2 projects, 3 topics, 1 draft | each grouping shows the right headings and counts; Drafts shows 1 | page harness |
| 2 | `run_ai` with enrich off | toast with the how-to text, nothing started | tests/window_api.rs |
| 3 | `board` payload with an override | override value shown | tests/window_api.rs |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features window,tray -- -D warnings \
  && cargo test --locked --features window,tray
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-298.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-298.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Board grouped by Day, Project, Topic, Kind; Drafts view; header with `N waiting` and `Run now`; note rows with AI status | easy-ui screenshots with stub IPC (5 states) | match the design's board language; status text quiet, not a badge storm |

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
| US-020 | see what the AI arranged | open board, switch to Topic | notes sit under topic headings with counts |
| US-020 | find unfinished notes | click Drafts | each draft shows reason and suggestion; none writes to the note |
| US-020 | run the AI now | click Run now | toast with result; waiting count falls |

---

## Approach

**Pattern reference**: `board` and `filters` in `src/surfaces/window/api.rs` and `assets/app.js`; HUN-288's `enrich` IPC command and toast.

**Vital slice**: Payload keys, group-by switch, Drafts view, status text, `Run now`.

**Cut list**: Saved views; drag between groups; a digest card.

---

## Edge Case Checklist

- [ ] A note with no project or topic: group `Unsorted`, last
- [ ] 100 topics: the group list scrolls, no layout break
- [ ] `Run now` clicked twice: the second says `already running`
- [ ] Window loses focus during a run: refresh on focus, as today
- [ ] A very long topic name: truncated with a title tooltip
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for text this change makes wrong

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/window/api.rs` | payload keys, `run_ai` |
| `src/surfaces/window/assets/app.js`, `app.css`, `index.html`, `ipc.js` | switch, views, status, button |
| `tests/window_api.rs`, `tests/fixtures/window/page_harness.js` | AC 1–6 |
| `README.md` | screens description |
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

Page harness with a stub IPC for the views and states; `tests/window_api.rs` for the payload and `run_ai`; a 5,000-note timing run pasted into the review file.

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