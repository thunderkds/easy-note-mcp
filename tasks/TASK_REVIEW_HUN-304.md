# TASK_REVIEW — HUN-304: Dashboard window layout: centred container on large screens, fit every allowed window size

> Sibling of `tasks/TASK_GUIDE_HUN-304.md`. Everything here is filled at Stage 4/5.

---

## Auto decisions

Mode: auto
1. Track: extended — RULE: `docs/claude-md/tracks.md` `## Track rule` (a UI component). Complexity C2 (more than 3 files), Risk Low, so no Debate.
2. Container: keep the design's 1440 px width and centre it; the header's contents and the toast use the same two edges — SAFER OPTION: the width is already in `app.css`; only its position changes ⚠ decided for you — please review
3. "Fit" at the smallest window: list and note panel side by side from 760 px (was about 976 px); the panel is 320 to 380 px wide — SAFER OPTION: an earlier session showed the user this change beside the request text that the user then sent ⚠ decided for you — please review
4. In two columns the panel stays in view and scrolls by itself — SAFER OPTION: without it, side by side leaves a picked note above the visible area ⚠ decided for you — please review
5. Window size on a small screen: open at most (screen - 80 px) in width and height; the minimum shrinks with it — SAFER OPTION: the window library reports no free desktop area; a fixed allowance adds no dependency ⚠ decided for you — please review
6. No larger text on large screens, no saved window size — RULE: `AGENTS.md` § Karpathy Engineering Principles
7. Pin test: the page test's fake browser gets `getComputedStyle` — RULE: `tasks/TASK_GUIDE_HUN-304.md` § Files to Change (Predicted)
8. The seven P3 findings are left as follow-ups; no code changed after the review — RULE: `skills/code-review/SKILL.md` § Workflow
9. The failing Companion score stands: no rescore and no change of the row — RULE: `skills/score/SKILL.md` § Never
Checks: tests `cargo test --locked --features window,tray --no-fail-fast` — 28 suites, 768 passed, 1 failed (the known intermittent HUN-290 lock test), 7 ignored; later runs of `--test window_api`: 121 passed, then 120 passed with the same lock test failed, then 121 passed; the reviewer's full run 769 passed, 0 failed; default build 760 passed, 0 failed; fmt and clippy clean in both builds; real run on virtual screens 3440x1440, 1920x1080, 1366x768, 800x600 (auto-checked, not /verify); review `code-review` by a fresh reviewer — pass, 0 P0, 0 P1, 0 P2, 7 P3; Companion score fail, 79 under the floor of 80
Memory: `memory/decisions.md` — HUN-304 layout fix, waits for a hand merge; `memory/learnings.md` — a stuck sticky panel reads as wrapped, and a headless smooth scroll hides it; `memory/MEMORY.md` — one line for each
Paused: merge not run — the Companion score row is `☑ fail` (79, floor 80) and the merge gate blocks a fail row, which is a safety-hook block (`docs/claude-md/tracks.md` `### Stops that stay`). The merge, the push of the branch, `release-plan` and the cleanup wait for the owner
Not done: merge (admin: after the auto verify pass); auto verify: ☑ pass (session 603d7ab5-9164-48dc-a1bf-4aedfe395fd4, commit 99a6922a4bc2e7265b76b26314bec5225483226c, log .claude/verify-runs/HUN-304-99a6922a4bc2.log)
Closed: row moved to Done 2026-11-27 — auto verify pass (session 603d7ab5-9164-48dc-a1bf-4aedfe395fd4, commit 99a6922a4bc2e7265b76b26314bec5225483226c); Gates 5 and 6 checked

## Review findings (Stage 4)

Reviewer: fresh-context `qa-expert`, 2026-11-27, `main...df464c68a1d8f440a5ed63e4b3e75b799567c590`. Risk Low: no security review.

**Verdict: pass. P0 0, P1 0, P2 0, P3 7.** No finding blocks the merge. ux-evaluate is `fail` overall only because of four `fail (older)` baseline checks.

Scope: `PRD.md`, `src/surfaces/window/assets/app.css`, `app.js`, `src/surfaces/window/mod.rs`, `tasks/TASK_GUIDE_HUN-304.md`, `tests/window_api.rs`, `tests/fixtures/window/page_harness.js`; direct callers `select` -> `revealDetail`, the dashboard `open` in `mod.rs`, the page harness's tests. Lane check: `lane: C2, 7 files, 362 lines, FITS; unpredicted: tasks/TASK_GUIDE_HUN-304.md`. Entry point `easy-note dashboard` is reachable (ran it). Startup-reads check: N/A (the Supervisor implemented; no implementer report). Reviewers: correctness, testing, maintainability, standards, adversarial, over-engineering (nothing), ux-evaluate. Safe fixes applied: none (the caller forbade code changes).

### P0 - Must Fix
None.

### P1 - Should Fix
None.

### P2 - Recommended
None.

### P3 - Optional

| # | File:line | Concrete failure | Confidence | Action |
|---|---|---|---|---|
| F1 | `src/surfaces/window/assets/app.css:115` (`.ai-by`); also `:97` `.pick`, `:68` `.error` | A long unbroken string makes the sticky panel scroll sideways (AC 5). `ai.model` `my_local_model_with_a_long_underscore_name_v2` at 800 px: panel `scrollWidth` 349, `clientWidth` 305. Before the change the 800 px panel was 729 px and did not overflow. The page itself does not scroll. Real model ids with hyphens (`claude-haiku-4-5-20251001`, `us.anthropic.claude-sonnet-4-5-20250929-v1:0`) wrap and pass. A synthetic unbroken path in a detail or edit error does the same (713 and 705 px); the backend errors listed in `ipc.js` are short, so this part is unlikely. `ai.model` has no length cap in `src/core/ai.rs` `check` | 100 | Add `overflow-wrap: anywhere` to `.ai-by`, or set it once on `.detail` |
| F2 | `src/surfaces/window/mod.rs:117` (`dashboard_sizes`); `PRD.md:109` | On a screen narrower than 840 px the window is narrower than 760 px, so it shows one column. Real window on 800x600: 720x520, list above `Pick a note to read it.`. This matches AC 3 and AC 7 as written, but not the restated intent ("From the smallest allowed window to the largest ... side by side"). The new FR-042 sentence says both "smallest window (800 px wide)" and "its minimum size shrinks with it". `DELTA` below says "down to the smallest window". The test `hun304_every_allowed_width_...` checks only `dashboard_sizes(None)` | 100 | Say in FR-042 and `DELTA` that a window under 760 px (screen under 840 px) shows one column |
| F3 | `src/surfaces/window/mod.rs:506` | The size comes from the primary (else first) monitor, not from the monitor where the window opens. Large primary plus a small second screen: a window placed on the small one can be larger than it. Small primary: the window opens smaller on the large one. The guide chose this rule (`## Approach`); not run with two monitors | 75 | Accept, or record it in the cut list |
| F4 | `tests/window_api.rs` (`hun304_*`); `src/surfaces/window/mod.rs:512-516` | No automated test fails if the builder stops using `dashboard_sizes` (constants restored in `with_inner_size` / `with_min_inner_size`), or if a later CSS rule brings sideways scroll (AC 5). AC 1-4 tests match CSS / JS strings, as the guide asks. The behaviour holds today (runs below). From reading the tests; no mutation was run | 75 | Optional |
| F5 | `tests/window_api.rs:703-704` | Stale comment: "HUN-284 AC 5: at 800 px the detail panel sits below the list". At 800 px it now sits beside the list. The assertions stay valid | 100 | Say "below 760 px" |
| F6 | `src/surfaces/window/assets/app.css:49`; Evidence row "UI: Visual regression" | With classic (space-taking) scrollbars the panel's own scrollbar takes 15 px at 1280 px: card 380 -> 365 px and `Move to trash` wraps to a second line (before: four buttons on one row). Seen in headless Chrome. The real WebKitGTK window under Xvfb used an overlay scrollbar and kept 380 px; its buttons were below the fold, so they were not observed there. The row "panel (380 px) keep their width" holds for overlay scrollbars only | 100 | Note it, or accept |
| F7 | `app.css`, `index.html` (older, from `ux-evaluate`) | B-c, B-d, B-e, B-f `fail (older)`, see the table. None is caused by this diff | 100 | Optional, separate task |

Older, outside this diff, no investigation: after an edit save refused as "changed outside the window" at 800x560, the alert sits above the visible part of the panel (before the change it sat above the visible part of the page in the same way).

### Acceptance Criteria, measured by the reviewer

| AC | Verdict | Observed |
|---|---|---|
| 1 | pass | Left / right space: 0.5 / 0.5 at 1456 px, 21 / 21 at 1497, 232.5 / 232.5 at 1920, 992.5 / 992.5 at 3440. Header contents start and end on the content's two lines at each width; header width = page width |
| 2 | pass | 3440 px: toast right edge 2404.5 = content right edge 2404.5 |
| 3 | pass | 759 px one column; 760, 800, 919, 999 px two columns, panel 320-379 px; 1000 px and up 380 px |
| 4 | pass | 800x560, row 40 picked far down: panel top 16, bottom 544. Panel scroll 400 stays after a focus refresh and after `Re-run`; another note starts at 0; `--note n45` lands in view. From page top, wheel over the panel scrolls the panel, then the page; the actions are reachable |
| 5 | pass for ordinary content; see F1 | Page and panel `scrollWidth` = `clientWidth` at 375, 640, 800, 1000, 1280, 1920, 3440 px; also empty and load-error states and a 70-letter search |
| 6 | pass | 375, 640, 759 px: a row click brings the panel to the top (`panelTop` 0) |
| 7 | pass | Real window (private Xvfb, X11): 3440x1440 -> 1280x820, minimum 800x560; 1366x768 -> 1280x688, 800x560; 800x600 -> 720x520, 720x520; scale 2 on 3840x2160 -> 2560x1640 device px (1280x820 logical); scale 2 on 1920x1080 -> 1760x920 device px (880x460 logical), minimum 1600x920 |

Tests: `cargo test --locked --features window,tray --no-fail-fast` with temp dirs, one run: 28 suites, 769 passed, 0 failed, 7 ignored (`window_api`: 121 passed, 1 ignored). `cargo fmt --check` clean. The default-feature build and clippy were not run again. Mutation checks: none.

Method for the page: scratch copy of the assets (old `9ccc442` and new) without the CSP meta tag, fake `window.ipc`, headless Chrome 158 driven over its debugging port, `deviceScaleFactor` 1, classic scrollbars.

### ux-evaluate (reviewer)

Screenshots in `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8cd0dd43-7fc3-44cc-9fbf-68159c0c60ed/scratchpad/rv304/shots/` (called `S/` below).

| # | User story | Flow / state | Width | Verdict | Screenshot | Reason |
|---|---|---|---|---|---|---|
| 1 | "centre it on large screens" | board at 1920 and 3440 | 1920 / 3440 | pass | `S/after-1920.png`, `S/after-3440.png` | content column in the middle; brand and `+ New note` line up with it |
| 2 | "let it fit" | 800x560, row picked after scrolling | 800 | pass | `S/after-800-far.png` | row `note 40` highlighted, its panel beside the list from `n40` down, no scroll back |
| 3 | "let it fit" | board at 375 | 375 | pass | `S/after-375-top.png`, `S/after-375-clicked.png` | only a vertical scrollbar; filters wrap; no cut button |
| B-a | baseline | default, empty, error | 375 / 1280 | pass | `S/after-375-empty.png`, `S/after-1280.png` | green `+ New note` is the most prominent action, in view at the top |
| B-b | baseline | default | 375 / 1280 | pass | `S/after-375-top.png`, `S/after-1280.png` | header, counts, search, filters, list, panel in one order |
| B-c | baseline | counts strip | 375 | fail (older) | `S/after-375-empty.png` | a stray right border on the stacked count cells |
| B-d | baseline | filters, search, panel actions | 375 | fail (older) | `S/after-375-top.png` | measured: search 38, filters 30, `Add as tags` 30, actions 34 px |
| B-e | baseline | body text | 375 | fail (older) | `S/after-375-top.png` | list and filter text under 16 px (base 14 px) |
| B-f | baseline | empty, load error | 375 / 1280 | fail (older) | `S/after-375-error.png`, `S/after-1280-error.png`, `S/after-1280-empty.png` | empty state is clear; the load error `Could not load notes: ...` says what happened, but not what to do next, and has no retry button |
| B-g | baseline | toast after `Move to trash` | 375 / 1280 | pass | `S/after-375-toast.png`, `S/after-1280-toast.png` | toast `moved to trash` with `Undo` visible, inside the content column |
| B-h | baseline | default | 375 | pass | `S/after-375-top.png` | one column, no sideways scrollbar (ordinary content) |
| B-i | baseline | default, Settings, capture | 375 / 1280 | pass | `S/after-1280-settings.png`, `S/after-375-settings.png`, `S/after-1280-capture.png` | one mint accent, one button style, same corner radius |

ux-evaluate: 8 pass, 4 fail (older), 0 not checked, 0 not checkable. Overall: fail (older debt only; no `fail` caused by this diff).

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/window_api.rs`, five `hun304_*` tests (AC 1-4 and 7). Red before the code: `test result: FAILED. 0 passed; 4 failed`, and for the click scroll `0 passed; 1 failed`. At `df464c6`: `test result: ok. 121 passed; 0 failed; 1 ignored`. AC 5 and 6 measured in a headless browser at 375, 640, 800, 1000, 1280, 1920 and 3440 px |
| Verification command run | ☑ pass | `cargo test --locked --features window,tray --test window_api`: `test result: ok. 121 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out` |
| Negative cases hold | ☑ pass | Screen unknown, 0x0, 80x80 and NaN keep 1280x820 and 800x560 (`hun304_the_window_is_never_larger_than_the_screen`). Red runs against the prior content above |
| verify | ☑ pass | auto-run by hook, session 603d7ab5-9164-48dc-a1bf-4aedfe395fd4, commit 99a6922a4bc2e7265b76b26314bec5225483226c, log .claude/verify-runs/HUN-304-99a6922a4bc2.log — pass |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Stage 4 reviewer, `main...df464c6`: the 7 changed files, `select` -> `revealDetail`, the dashboard `open` in `mod.rs` and the page harness's tests. `lane: C2, 7 files, 362 lines, FITS; unpredicted: tasks/TASK_GUIDE_HUN-304.md`. See `## Review findings (Stage 4)` |
| Full smoke suite still green (no regression) | ☑ pass | Both builds, see `Checks:`. The one failure, `hun290_ac4_the_lock_is_a_file_in_the_notes_dir_and_only_one_owner_holds_it`, is recorded in `memory/learnings.md` as intermittent on `main`; it passed on the rerun |
| **Companion score (easy-verifier)** | ☑ fail | overall 79 (floor missed: 79 < 80), change_risk pass 100 (bugfix, confirmed), scope changes, ref `main...HEAD` at `df464c6`, abstentions: solution-fit, reliability, api, data, operations, infrastructure; gates: code-quality answered (75, confidence 0.6), solution-fit left abstained; divergence: none returned. Below 80: blast-radius 46 (6 of 7 changed files are often-changed files), requirement-fidelity 67 (repo-wide share of requirement items traced), code-quality 69 (one function in `mod.rs` has complexity 16 against 15, and no lint config file), test-strategy 72 (it finds no covering test for `app.js` and `mod.rs`, although `tests/window_api.rs` tests both). report: `/home/hungnguyenhuu/.easy-verifier-reports/easy-note-mcp-HUN-304/evidence-report-changes-20261127T032614-826997Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☑ pass | Measured before and after. At 1280 px the list (809 px) and panel (380 px) keep their width; with a scrollbar that takes space the panel's content narrows 15 px and `Move to trash` can wrap (review F6; the real window used an overlay scrollbar). Other changes: centred above 1440 px, two columns from 760 px, panel stays in view with its own scroll |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ pass | CSS diff adds no colour and no font; `color-scheme: dark` on the panel, as `.set` has. Measured: container 1440 px, side padding 28 px, gap 20 px, panel 320 px at 800 and 380 px from 1000 |
| **UI: Responsiveness at target viewports** | ☑ pass | No sideways scroll at 375, 640, 800, 1000, 1280, 1920, 3440 px (page scroll width equals viewport width). 1920: 233 px left, 232 px right. 3440: 993 / 992. Real window: 1280x688 on 1366x768, 720x520 on 800x600, resized to 800x560 and 3440x1440 |

### ux-evaluate (implementer)

| # | Flow / state | Width | Verdict | Reason |
|---|---|---|---|---|
| 1 | board on a wide screen | 1920 / 3440 | pass | column in the middle, header lines up |
| 2 | 800x560, row picked after scrolling | 800 | pass | note shows beside the list, no scroll back |
| 3 | smallest page | 375 | pass | no sideways scrollbar, no cut button |
| B-a, B-b, B-f, B-g, B-i | default, toast, empty | 375 / 1280 | pass | `+ New note` in view; one accent colour |
| B-h | default | 375 | pass | no sideways scroll |
| B-c | counts strip | 375 | fail (older) | a stray right border on the stacked count cells |
| B-d | filters | 375 | fail (older) | filter buttons are 30 px high |
| B-e | body text | 375 | fail (older) | body text is 14 px |

The three `fail (older)` rows are on parts the diff does not change. The window's minimum is 800 px; 375 px is reached only on a screen narrower than about 455 px.

---

## Demonstration

**BEFORE** (Supervisor, 2026-11-27T09:56:41+07:00, assets of `9ccc442`, headless Chrome):

```
1920x1000 main x=0 w=1440 (465 px empty on the right)   detail (1032, -834, 380)  <- picked note out of view
3440x1340 main x=0 w=1440 (1985 px empty on the right)
800x560   notes w=729, detail y=1459 below the list
```

**AFTER** (same script, `df464c6`):

```
1920x1000 left 233 right 232 main_w 1440   detail (1265, 16, 380, 968)
3440x1340 left 993 right 992 main_w 1440
800x560   notes_w 389, detail (437, 16, 320, 528) beside the list
screen 1366x768: window ['1280x688'] minimum ('800', '560')
screen 800x600:  window ['720x520']  minimum ('720', '520')
```

**DELTA**: The dashboard sits in the middle of a wide screen, shows the note beside the list in windows from 760 px wide, and no longer opens larger than a small screen. A screen narrower than 840 px gets a narrower window with one column (review F2).

**WITNESS**: Supervisor, 2026-11-27, `memory/event-trace/HUN-304.jsonl`.
