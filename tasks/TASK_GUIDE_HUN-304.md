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

User, chat 2026-11-27: "Fix the dashboard window layout. On large screens, put the content in a centred container. At every window size we allow, the layout must fit."

Earlier the same day: "it does not fit with the large screen, with the large we will have the container and align it center. with the screen size that we allow, let it fit".

**Restated intent**:

> On a wide screen the dashboard content sits in the middle, not on the left. From the smallest allowed window to the largest, the list and the note panel are usable side by side, nothing is cut or scrolls sideways, and the window never opens bigger than the screen.

**Out of scope**:

* Larger text or UI zoom on big screens
* Saving the window size or position
* The quick window (fixed size, not resizable)
* New views (HUN-298, HUN-299, HUN-301)

**Requirement Refs**:

* FR-042: the dashboard window shows the design's board (one sentence added for this layout rule)

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request
- [X] Domain terms align with `PROJECT_SPEC.md` glossary
- [X] Every Acceptance Criterion below traces to a line in the Requirement
- [X] All Requirement Refs exist in `PRD.md` and are covered

Signed: auto (Supervisor), 2026-11-27.

---

## Dependencies & Reachability

**Depends on**: `None`

**Entry point**: `easy-note dashboard`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | In a window wider than 1440 px, the content is a 1440 px column with equal empty space left and right (± 1 px). The header's contents start and end on the same two lines as the content. The header's bottom line stays full width | centred container on large screens |
| 2 | In a window wider than 1440 px, the toast's right edge sits inside the container, not at the far screen edge | centred container on large screens |
| 3 | From 800 px wide up, the list and the note panel sit side by side. From 1000 px the panel is 380 px wide, as today | fit at every allowed size |
| 4 | In two columns the note panel stays in view while the list scrolls. It is never taller than the window and scrolls inside itself. A newly picked note shows from its top | fit at every allowed size |
| 5 | At 375, 640, 800, 1000, 1280, 1920 and 3440 px nothing scrolls sideways | fit at every allowed size |
| 6 | Below 760 px the panel still goes under the list, and a row click brings it into view (HUN-284) | boundary |
| 7 | The window opens at 1280 x 820, made smaller to fit a smaller screen (80 px kept free in width and in height for desktop bars). The minimum size is 800 x 560, never larger than the opening size. With no known screen size, both stay as today | screen size that we allow |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | The page at 1920 and 3440 px | Equal left and right space; header aligned | app run, measured in the browser, screenshot |
| 2 | The page at 800 x 560 with a note picked far down a long list | Panel beside the list and in view | app run, screenshot |
| 3 | Screen sizes 3440x1440, 1920x1080, 1366x768, 1280x720, 800x600, 640x480, none, 0x0 | Opening and minimum sizes as AC 7 | automated test |
| 4 | The embedded page files | The layout rules of AC 1-4 are present | automated test |

### Verification Command (exact, runnable)

```bash
cargo test --locked --features window,tray --test window_api
```

---

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Dashboard board with a picked note | screenshots before and after | at 1280 px no visible change; at other widths only the changes of AC 1-4 |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Colors match design tokens | diff audit | no new colour |
| Typography matches spec | diff audit | no font change |
| Spacing / layout matches spec | measured in the browser | container 1440 px, side padding 28 px, column gap 20 px, panel 320-380 px |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| Narrow (375, 640 px) | screenshot, measured | one column, no sideways scroll |
| Allowed minimum (800 x 560) | screenshot, measured, real window | two columns, no sideways scroll |
| Desktop (1280, 1920, 3440 px) | screenshot, measured, real window | centred above 1440 px, two columns |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| "centre it on large screens" | read notes on a wide screen | board at 1920 and 3440 px | the content column is in the middle and the header lines up with it |
| "let it fit" | read a note in a small window | board at 800 x 560, a row picked after scrolling | the picked note shows beside the list without scrolling back |
| "let it fit" | use the smallest page | board at 375 px | no sideways scrollbar, no cut button |

---

## Approach

**Pattern reference**: `src/surfaces/window/assets/app.css` (the `@media` blocks) and `src/surfaces/window/mod.rs` (`open_capture` reads the monitor size)

**Vital slice**: centring and the two columns from 800 px
**Cut list**:

* The desktop's exact free area (a fixed 80 px allowance instead)
* Remembering the window size

The size rule is a pure function outside the `window` feature module, so the default test run covers it. Use the primary monitor, else the first monitor.

---

## Edge Case Checklist

- [ ] Wayland gives no primary monitor: use the first monitor, else today's sizes
- [ ] A screen of size 0 or smaller than the allowance gives today's sizes
- [ ] A long note or edit mode inside the sticky panel stays reachable by scrolling the panel
- [ ] A focus refresh does not move the panel's scroll position
- [ ] Pin-test pre-flight: the row below is filled before work starts
- [ ] Stale references: grep for `1280`, `800 px`, `max-width: 1440px`

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/window/assets/app.css` | centre the container and header contents and toast; two columns from 760 px; sticky panel |
| `src/surfaces/window/assets/app.js` | a newly picked note starts at the panel's top |
| `src/surfaces/window/mod.rs` | window sizes from the screen size |
| `tests/window_api.rs` | tests for AC 1-4 and 7 |
| `PRD.md` | one sentence in FR-042 |
| `tests/fixtures/window/page_harness.js` | the fake browser gets `getComputedStyle`, which `revealDetail` now reads |
| Pre-existing pin tests | `tests/window_api.rs` `a_row_click_brings_the_detail_into_view_only_when_it_wraps` (strings of `app.js`) and `the_page_has_the_design_capture_dialog` (strings of `app.css`): both stay valid, no repoint |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `src/surfaces/organize.rs`, `src/core/ai.rs`, `src/surfaces/cli.rs` | HUN-295 is in progress in another session |
| `src/surfaces/window/assets/quick.css` | the quick window is out of scope |

---

## Test Plan

Write the tests first and see them fail. Then run both builds. Then measure the page in a headless browser at each width and run the real window in a virtual display.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Lint passes
- [ ] Tests written AND pass, output pasted into the Evidence table
- [ ] Auto verify run started
- [ ] `memory/MEMORY.md` updated if new patterns were learned