**Date**: 2026-11-20
**Complexity Level**: C1
**Risk Level**: Low
**Priority**: P2
**Assigned agent**: Frontend-Implementer (the page in `assets/`; the small `board` payload change in `window::api` is in scope)
**Agent guide**: `agents/frontend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*, *AI detail*)
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read your agent guide (`agents/frontend.md`)
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in your agent guide
6. Read `docs/ddr/0009-own-dashboard-window.md`, PRD FR-042, NFR-002, NFR-013, and the HUN-278 code-review notes in `tasks/TASK_REVIEW_HUN-278.md` (`Checks:` line)
7. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/easy-note/` or `~/.local/share/`

---

## Requirement (Pillar 1 — Adapt the requirement)

HUN-278 code review, 3 P2 findings (performance), filed by the Supervisor as Linear HUN-282 on 2026-11-20; the owner said "yes" to adding the review findings to the board and "start all the tasks as parallel".

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> The dashboard window asks for the board once when it opens (not twice), re-renders the note list on search without rebuilding it on every key press, and the `board` reply no longer carries each note's full text when the list and search do not need it. What the person sees does not change.

**Out of scope** (what this task explicitly does NOT do):

* Any visual change, new filter or new control
* The detail panel (`detail` already sends the full text)
* The window actions (HUN-279) and the capture dialog (HUN-280)

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-042: the board, its search and its focus refresh
* NFR-013: board p95 under 1 s over 5,000 notes
* NFR-002: every string still masked

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent) — Supervisor 2026-11-20
- [X] Domain terms align with `PROJECT_SPEC.md` glossary — Supervisor 2026-11-20
- [X] Every Acceptance Criterion below traces to a line in the Requirement — Supervisor 2026-11-20
- [X] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above — Supervisor 2026-11-20

---

## Dependencies & Reachability

**Depends on**: HUN-279 (branch `feat/HUN-279-actions`, not merged yet: this task's branch is cut from it, because both change `app.js`)

**Entry point**: `loadBoard` and the `search` input handler in `src/surfaces/window/assets/app.js`; `fn board` in `src/surfaces/window/api.rs`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | On window open the page sends exactly one `board` request (the shell's focus refresh and the page's own start-up load no longer both fire for the first paint) | asks once |
| 2 | Typing in search does not rebuild every row element per key press: either a short debounce (≤ 150 ms) or a filter that hides/shows existing rows; the shown rows, counts and empty texts are the same as today | search |
| 3 | The `board` reply's note items carry only the fields the list, filters and search use; search still matches every word against the text, project, task and tags exactly as today (if the full text is needed for search, keep a search field, not the raw body) | payload |
| 4 | Every string in the reply is still masked; the slice 1 planted-secret test stays green | NFR-002 |
| 5 | `tests/perf.rs` window_board budget still passes; record the new p95 next to the old one (~124 ms) | NFR-013 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | page start-up with a stub `window.ipc` that counts requests | 1 `board` request before any focus event | `tests/window_api.rs` page-contract test (or a headless stub run, pasted) |
| 2 | 6 sample notes, query `firefox flaky` + Bugs | same 1 row as the HUN-278 screenshot `search-filter-1280` | test on the `board` answer + screenshot |
| 3 | `board` over the fixture | no note item has the old full-body field (or it is replaced by the agreed search field) | `tests/window_api.rs` |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features window -- -D warnings \
  && cargo test --locked --features window \
  && cargo test --locked --release --test perf -- --ignored --nocapture --test-threads=1 perf_window_board
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-282.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-282.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Board at 1280 px, with and without a search | screenshot vs the HUN-278 screenshots | no visible change |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| CSS | `git diff` of `app.css` | unchanged, or N/A with that diff pasted |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| 1280 px and 800 px | screenshot | same as before |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-019 | find a note fast | type a search of two words | the list narrows as before, with no visible lag |

---

## Approach

**Pattern reference**: the existing `renderList` / `shownNotes` split in `app.js` (HUN-279) and the `REFRESH` hook in `src/surfaces/window/mod.rs:18`.

**Vital slice**: the single start-up load and the slimmer payload; the search render change only if it stays small.

**Cut list**: virtual scrolling; caching between focus refreshes.

---

## Edge Case Checklist

- [ ] Focus event that arrives before the first load answers → still one board render with the latest data
- [ ] Empty notes dir → same empty text as today
- [ ] Search with an empty query after typing → all rows back
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/window/assets/app.js` | one start-up load; search render |
| `src/surfaces/window/api.rs` | `board` note items without the full body |
| `tests/window_api.rs` | AC 1–4 |
| Pre-existing pin tests | `tests/window_api.rs` (slices 1–2), `tests/perf.rs` — list each and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*` | Supervisor-owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| `src/core/*` | Call existing public functions only |
| `src/surfaces/window/mod.rs` CSP and navigation guard | HUN-284 |

---

## Test Plan

`window::api` tests in the default build with temp env; a page-contract test that counts `board` requests with a stub; the perf test in release. Screenshots by the HUN-278 method (headless Chrome on a scratch copy of `assets/` with a stub `window.ipc`).

---

## Completion Checklist

- [ ] Implementation done
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_HUN-282.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log
