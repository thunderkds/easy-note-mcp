**Date**: 2026-11-27
**Complexity Level**: C2
**Risk Level**: Low
**Priority**: P1
**Assigned agent**: Frontend-Implementer (the Supervisor implements)
**Agent guide**: `agents/frontend.md`

---

## Mandatory Startup (Do Not Skip)

1. Read `PROJECT_SPEC.md`
2. Read the memory slice in your spawn prompt. Read `memory/MEMORY.md` in full only if your work reaches something the slice does not cover
3. Read this file completely
4. Read `agents/frontend.md`
5. Apply the C2 process
6. Read `memory/codebase-map.md`
7. Tests and real runs only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/` or `~/.config/`; unset `XDG_CURRENT_DESKTOP`

---

## Requirement (Pillar 1 — Adapt the requirement)

User, chat 2026-11-27, after installing the HUN-304 build: "the UI change is not good also, the easy-note should update to Easy Taking Shot, the note content should extend, and scope by the scroll top to down, the list of note should just show the good, you should investigate the UI/UX and update the application for better".

**Restated intent**:

> The window shows the name Easy Taking Shot. The list and the note each fill the window height and scroll on their own, from top to bottom. The note takes the remaining space and its buttons stay in view. A list row shows only what helps to find a note.

**Out of scope**:

* The command name, the package name, the notes folder, the config folder, the note format
* The name in the launcher, the tray and the desktop notifications (the user confirms the name first)
* A screenshot capture feature
* New views (HUN-298, HUN-299, HUN-301) and the quick window's layout

**Requirement Refs**:

* FR-042: the dashboard window (sentence updated for this layout)
* FR-046: the window still shows `edited <when>`

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request (two phrases are unclear; the review file lists the reading taken)
- [X] Domain terms align with `PROJECT_SPEC.md` glossary
- [X] Every Acceptance Criterion below traces to a line in the Requirement
- [X] All Requirement Refs exist in `PRD.md` and are covered

Signed: auto (Supervisor), 2026-11-27.

---

## Dependencies & Reachability

**Depends on**: HUN-304 (not merged; this branch starts from `feat/HUN-304-window-layout`)

**Entry point**: `easy-note dashboard`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | The dashboard header, the dashboard window title and the quick window title say `Easy Taking Shot` | update to Easy Taking Shot |
| 2 | From 760 px wide and 480 px high up, the page itself does not scroll: the header, the counts, the search and the filters stay in place | scroll top to down |
| 3 | In that range the list scrolls on its own and reaches its last row; scrolling it does not move the note | scroll top to down |
| 4 | In that range the note pane fills the height down to the window's bottom edge, scrolls on its own and reaches the end of a long note; it takes the width the list leaves | note content should extend |
| 5 | The note's buttons are in view without scrolling, for a short and a long note | note content should extend |
| 6 | In edit mode the text box grows with the pane | note content should extend |
| 7 | A list row shows the time, the first line on at most two lines, `edited <when>` and the project, and the AI or stale mark. It shows no tags, no task and no kind chip | list should just show the good |
| 8 | Search still finds a note by tag and by task | boundary |
| 9 | The counts strip takes at most 80 px of height in the two-pane range and keeps its three counts and project bars | better UX |
| 10 | Outside that range the page scrolls as one column, and a row click brings the note into view (HUN-284) | boundary |
| 11 | The toast does not cover the note's buttons | better UX |
| 12 | Nothing scrolls sideways at 375, 640, 800, 1000, 1280, 1920 and 3440 px; the content stays a centred column of at most 1440 px | HUN-304 stays true |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | 40 notes, a long note picked, 800x560 and 1920x1000 | AC 2-5 | app run, measured in the browser, real window |
| 2 | The embedded page files | AC 1, 7 and the layout rules | automated test |

### Verification Command (exact, runnable)

```bash
cargo test --locked --features window,tray --test window_api
```

---

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Board with a picked note, edit mode, empty state, toast | screenshots before and after | only the changes of AC 1-11 |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Colors match design tokens | diff audit | no new colour |
| Typography matches spec | diff audit | IBM Plex only |
| Spacing / layout matches spec | measured in the browser | list column 300-440 px, gap 20 px, side padding 28 px |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| Narrow (375, 640 px) | screenshot, measured | one column, page scroll |
| Smallest normal window (800 x 560) | screenshot, measured, real window | two panes, no page scroll |
| Desktop (1280, 1920, 3440 px) | screenshot, measured, real window | two panes, centred above 1440 px |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| "note content should extend" | read a long note | pick a long note | the note pane reaches the window bottom and its buttons show |
| "scope by the scroll top to down" | browse many notes | scroll the list | the header and the picked note stay where they are |
| "list should just show the good" | find a note | the list | each row shows time, first line, project and at most one mark |

---

## Approach

**Pattern reference**: `src/surfaces/window/assets/app.css` (the `.set` dialog: a fixed-height flex column with one scrolling body)

**Vital slice**: the two panes with their own scroll, and the simpler rows
**Cut list**:

* A resizable divider between the panes
* A list density setting
* Renaming the launcher, tray and notifications

CSS does the layout; `app.js` changes only the row content. The pane replaces the sticky panel of HUN-304, so its sticky check and that test go.

---

## Edge Case Checklist

- [ ] A short window (under 480 px high) falls back to page scroll
- [ ] A focus refresh keeps both scroll positions
- [ ] `dashboard --note` and a second call scroll the list to the row
- [ ] An empty folder and a search with no result
- [ ] A long project name in a row does not widen the list
- [ ] Pin-test pre-flight: the row below is filled before work starts
- [ ] Stale references: grep for `kind chip`, `where-line`, `sticky`, `easy-note — `

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/window/assets/app.css` | two full-height panes, compact counts, simpler rows, pinned actions, toast position |
| `src/surfaces/window/assets/app.js` | row content; remove the sticky check |
| `src/surfaces/window/assets/index.html` | name and title |
| `src/surfaces/window/assets/quick.html` | title |
| `src/surfaces/window/mod.rs` | two window titles |
| `tests/window_api.rs` | tests for AC 1, 7 and the layout rules; repoint the HUN-304 tests |
| `tests/fixtures/window/page_harness.js` | remove `getComputedStyle` if nothing reads it |
| `PRD.md` | FR-042 sentence |
| Pre-existing pin tests | `hun304_every_allowed_width_shows_the_list_and_the_panel_side_by_side` and `hun304_a_row_click_never_scrolls_the_page_to_a_panel_that_stays_in_view` (the sticky panel goes: repoint and remove); `the_page_has_the_design_capture_dialog`, `a_row_click_brings_the_detail_into_view_only_when_it_wraps`, `hun291_the_page_shows_edited_on_the_row_and_the_full_stamp_in_the_detail` stay valid |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `src/surfaces/tray.rs`, `src/surfaces/install.rs` | HUN-296 changes the tray; the launcher name waits for the user |
| `src/core/` | no data change |

---

## Test Plan

Tests first, seen to fail. Then both builds. Then measure and screenshot the page at each width, and run the real window.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Lint passes
- [ ] Tests written AND pass, output pasted into the Evidence table
- [ ] Auto verify run started
- [ ] `memory/MEMORY.md` updated if new patterns were learned