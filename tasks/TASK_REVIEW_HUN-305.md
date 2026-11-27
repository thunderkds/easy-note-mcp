# TASK_REVIEW — HUN-305: Dashboard redesign: full-height list and note panes, simpler list rows, name Easy Taking Shot

> Sibling of `tasks/TASK_GUIDE_HUN-305.md`. Everything here is filled at Stage 4/5.

---

## Auto decisions

Mode: auto
1. Track: extended — RULE: `docs/claude-md/tracks.md` `## Track rule` (a UI component). Complexity C2, Risk Low, so no Debate. A new task, stacked on the unmerged HUN-304 branch.
2. `update to Easy Taking Shot` is read as the product name shown by the window and the quick window. The command, the package, the notes folder, the launcher, the tray and the notifications keep `easy-note` — SAFER OPTION: no source in the project explains the phrase; a title change is easy to undo ⚠ decided for you — please review
3. `scope by the scroll top to down` is read as: the page stays still, and the list and the note each scroll on their own — SAFER OPTION: the smallest reading that also makes the note extend ⚠ decided for you — please review
4. `the list of note should just show the good` is read as: show only what helps to find a note. Rows lose the task, the tags and the kind chip; they keep time, two lines of text, `edited`, project and the AI or stale mark — SAFER OPTION: `edited` is FR-046, and search still reads task and tags ⚠ decided for you — please review
5. The list column is 300 to 440 px; the note takes the rest of the 1440 px container; the counts strip is made shorter and its `by project` label shows only when there is no bar — SAFER OPTION: no feature is removed ⚠ decided for you — please review
6. The first review (fail, 3 P1) was fixed and a second fresh reviewer checks the new commit — RULE: `docs/claude-md/pipeline-stages.md` § Stage 4: Review
7. The HUN-304 sticky panel and its click check are replaced by the two panes; two HUN-304 tests are repointed or removed — RULE: `tasks/TASK_GUIDE_HUN-305.md` § Files to Change (Predicted)
Checks: tests after the fix round, `cargo test --locked --features window,tray --no-fail-fast` — 28 suites, 774 passed, 0 failed, 7 ignored; default build 765 passed, 0 failed, 7 ignored; fmt and clippy clean in both builds; real run on virtual screens 3440x1440 and 1366x768 before the fix round (auto-checked, not /verify). The first review's full run had 1 failure, the intermittent HUN-290 lock test
Memory: pending
Not done: merge (admin: after the auto verify pass); auto verify: running (T182)

## Review findings (Stage 4)

Reviewer: qa-expert, fresh context, 2026-11-27. Diff `feat/HUN-304-window-layout...497c5e1866c7e27845487151ac72e2274e652f3f`. **Verdict: fail** (P0 0, P1 3, P2 2, P3 9). No code changed, so no fix was applied.

How it was checked: headless Chrome on a scratch copy of the assets (CSP tag removed, fake `window.ipc`), plus the real window (`target/debug/easy-note`, WebKitGTK) on Xvfb with a temp HOME and notes dir. `lane: C2, 9 files, 428 lines, FITS; unpredicted: tasks/TASK_GUIDE_HUN-305.md`. Entry point `easy-note dashboard`: reachable. Security review: not run (Risk Low). Mutation checks: none. Full run (`cargo test --locked --features window,tray --no-fail-fast`): 28 suites, 770 passed, 1 failed, 7 ignored. The failure, `hun290_ac4_the_lock_is_a_file_in_the_notes_dir_and_only_one_owner_holds_it`, is outside the diff and was not investigated.

Passed when run: AC 1, 2, 3, 4, 5, 6, 7, 9 (55-76 px) and 12 on default screens; AC 8 by code (`hayOf` is unchanged). AC 10 and AC 11 fail.

### P1

| # | File:line | Concrete failure | Confidence | Action |
|---|---|---|---|---|
| F1 | `src/surfaces/window/assets/app.js:151` (with `:479`) | A focus refresh moves the list back to the focused row. Click row 1, scroll the list, switch away and back: Chrome `scrollTop` 1800 to 0 at 800x560 and 1280x820; the real window jumps back too (`real-3-list-scrolled.png`, `real-5-after-focus-refresh.png`). With no focused row the position stays. The edge case "A focus refresh keeps both scroll positions" fails, and the Responsiveness Evidence row's claim holds only without a focused row. The re-focus call predates this diff. | 100 | In `renderList` re-focus with `focus({ preventScroll: true })`; keep scrolling for `show`, `--note`, trash and Undo (see F8) |
| F2 | `src/surfaces/window/assets/app.css:41-43,209` and `app.js:179` | AC 10 fails for a short, wide window (under 480 px high, about 976 px wide or more): the base rules still make two columns, the sticky panel is gone, and `revealDetail` returns because both tops are equal. At 1280x470 a click on row 26 leaves the note at top -1467, bottom -1073: nothing visible happens. The base branch showed it at top 16. Reach: only screens under about 560 logical px high (window minimum is `min(560, screen - 80)`). | 100 | Under 480 px high, force one column (for example `.detail { flex-basis: 100%; }` in a `max-height: 479px` block) and test it |
| F3 | `src/surfaces/window/assets/app.css:224` | AC 11 fails where the buttons wrap to two rows: `bottom: 96px` is lower than the top button row. The toast covers 8 px of `Copy id` at 760 wide (any note), of `Re-run` and `Copy id` on an inbox note at 800x560 (`toast-800x560-n5.png`), and of `Copy id` at 820x700. In the real window at 800x560 the toast ends at y 463 and the button starts at y 455; the short `Copied` toast misses it sideways, longer toasts do not. At 375, with the page at its end, the toast also covers `Move to trash` (`toast-375x812.png`; that rule predates this diff). | 100 | Raise the offset to clear two rows (136 px or more), or stop the row from wrapping, or place the toast elsewhere |

### P2

| # | File:line | Concrete failure | Confidence | Action |
|---|---|---|---|---|
| F4 | `src/surfaces/window/assets/app.js:103,680` | Search and filters keep the list's old scroll position, now that `#groups` scrolls under an always visible search box. Scroll to 3000, type `easy-kit`: the list opens at the 10th of 13 results and the newest are hidden above (`new-search-scrolled.png`). The `Stale` filter does the same (scrollTop 364 of 669). | 100 | Set `#groups.scrollTop = 0` on search, filter and clear |
| F5 | `tests/window_api.rs:4055` | The new tests match CSS substrings only. No test covers AC 9 (AC 12 keeps only the HUN-304 centring test), none covers the short, wide case of AC 10 (F2), and the AC 11 assertion pins the value that fails (F3). The removed HUN-304 test held the only assertion for `.tag, .ai-tag, .ai-title { overflow-wrap: anywhere; }` (`app.css:99`); no test checks it now. | 75 | Restore that assertion; add checks for the F2 and F3 fixes and the 80 px strip |

### P3

| # | File:line | Concrete failure | Confidence | Action |
|---|---|---|---|---|
| F6 | `app.css:222` | The pinned bar can hide a keyboard-focused control: Tab to `Add as tags` at 760x560 leaves it at y 518-548, behind the bar (top 443); same at 1280x820. | 100 | `scroll-padding-bottom` on `.detail` |
| F7 | `app.css:223` | `flex: 1 0 160px` overrides the height, so the resize handle does nothing (height 600 px set, 380 px drawn). At 800x560 the edit box is 160 px; it was 227 px. | 100 | `flex-basis: auto`, or `resize: none` |
| F8 | `app.js:479` | A repeated `show` for the row that already has focus does not scroll to it (row at y 2745, list at 243-548). In the app the focus refresh may hide this today; the F1 fix would expose it. | 75 | `scrollIntoView({ block: 'nearest' })` in `show` |
| F9 | `PRD.md:109` | FR-042 still says rows have `where-line, kind chip`, and its new last sentence says they do not. | 100 | Edit the older words |
| F10 | `app.css:57` (older) | A long search word with no spaces does not wrap: at 375 the page scrolls sideways (593 px), at 800 the text runs over the note pane (B-h). | 100 | `overflow-wrap: anywhere` |
| F11 | `app.js` `saveEdit` (older) | On a long note, a refused save leaves the `Not saved` message scrolled out of view (y -6054 at 800x560; the base branch does the same). | 100 | Scroll the message into view |
| F12 | `app.css` (older) | At 375 some targets are under 44 px (search 38, filters 30, actions 34) and body text is 14 px (B-d, B-e). | 100 | Later task |
| F13 | `index.html` (older) | The empty-folder message has no action button and does not name the next step (B-f). | 100 | Later task |
| F14 | `tasks/TASK_GUIDE_HUN-305.md` UX rows 1-2 | The rows name no width. At 375 the layout is one column (AC 10), so the buttons are not in view and page scroll moves the header: `fail (older)`. | 100 | Limit the rows to the two-pane range |

### UX verdicts (`ux-evaluate`)

Screenshots: `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8cd0dd43-7fc3-44cc-9fbf-68159c0c60ed/scratchpad/r305.gVIPvz/shots/` (names below).

| # | User story | Flow / state | Width | Verdict | Screenshot | Reason |
|---|---|---|---|---|---|---|
| 1 | note content should extend | pick a long note | 375 / 1280 | fail (older) | `new-375x812-long.png`, `new-1280x820-long.png` | 1280: pane reaches the bottom and buttons show. 375: buttons far below (F14) |
| 2 | scroll top to down | scroll the list | 375 / 1280 | fail (older) | `new-375x812-pick.png`, `new-1280x820-long.png` | 1280: a wheel on the list moves neither header nor note (measured). 375: page scroll (F14) |
| 3 | list should just show the good | the list | 375 / 1280 | pass | `new-375x812-pick.png`, `new-1280x820-long.png` | Time, first line on two lines at most, `edited` and project, one mark or none |
| B-a | baseline | default | 375 / 1280 | pass | same | `+ New note` is the most prominent control and in view |
| B-b | baseline | default | 375 / 1280 | pass | same | Name, counts, list, note read in order |
| B-c | baseline | default, edit | 375 / 1280 | pass | `new-1280x820-long.png`, `edit-1280x820-n0.png` | Aligned grid and even gaps |
| B-d | baseline | default | 375 | fail (older) | `new-375x812-pick.png` | Targets under 44 px (F12) |
| B-e | baseline | default | 375 | fail (older) | `new-375x812-pick.png` | Body 14 px (F12) |
| B-f | baseline | empty folder, no result | 375 / 1280 | fail (older) | `empty-375x812.png`, `empty-1280x820.png`, `noresult-1280x820.png` | No result has `Clear search`; empty folder has no action (F13) |
| B-g | baseline | toast after Copy id | 375 / 1280 | pass | `toast-375x812.png`, `toast-1280x820.png` | The toast shows at both widths; its placement is F3 |
| B-h | baseline | default, no result | 375 / 1280 | fail (older) | `new-375x812-pick.png` | Default screens do not scroll sideways; a long search word does (F10) |
| B-i | baseline | default | 375 / 1280 | pass | same | One accent, one button style, same radius |

ux-evaluate: 6 pass, 0 new fail, 6 fail (older), 0 not checked, 0 not checkable. Overall: fail

## Fix round (Supervisor, after the review of `497c5e1`)

| Finding | State | Check |
|---|---|---|
| F1 list jumps on a focus refresh | fixed | browser, 760 to 1920 wide: list stays at 1800 (before: 0) |
| F2 low, wide window | fixed | 1280x470: the clicked note comes into view, top 0 (before: -1676) |
| F3 toast covers a button | fixed | the toast sits over the list, left; no button covered at 760x560 (before: `Copy id`) |
| F4 search keeps the old position | fixed | after a search the list is at 0 (before: up to 2046) |
| F5 tests | partly fixed | new tests for F1-F4 and the `.tag` wrap rule; no test measures the 80 px strip |
| F6 pinned bar hides a focused control | fixed, not measured | `scroll-padding-bottom: 112px` |
| F7 edit box | fixed | 227 px at 800x560; it grows with the pane |
| F8 repeated `show` | fixed | `scrollIntoView` in `focusRow` |
| F9, F14 wording | fixed | `PRD.md` FR-042; the guide's UX rows name the range |
| F10-F13 | left, older | follow-ups |

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/window_api.rs`, four `hun305_*` tests (AC 1-8, 11). Red before the code: `test result: FAILED. 3 passed; 3 failed` (the three new tests of the first set), and `0 passed; 1 failed` for the short window. After: `test result: ok. 123 passed; 0 failed; 1 ignored`. AC 2-6, 9-12 measured in a headless browser at seven sizes |
| Verification command run | ☑ pass | `cargo test --locked --features window,tray --test window_api`: `test result: ok. 123 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out` |
| Negative cases hold | ☑ pass | Under 760 px wide the page scrolls as one column and a row click brings the note into view; the empty folder shows its message. Red runs against the prior content above |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Scope: the 9 files of the diff, their callers in the window page and shell, and `tests/window_api.rs`. Review verdict: fail (P1 3), see `## Review findings (Stage 4)` |
| Full smoke suite still green (no regression) | ☑ pass | Both builds, see `Checks:` |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | Pending |
| **UI: Visual regression (diff or verdict pasted)** | ☑ pass | Screenshots read at 800x560, 1280x820, 1920x1000: only the changes of AC 1-11 |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ pass | CSS diff adds no colour and no font. Measured: list 300 px at 800 and 440 px from 1440, gap 20 px, side padding 28 px, container 1440 px |
| **UI: Responsiveness at target viewports** | ☑ pass | Page height equals the window and no sideways scroll at 800, 1000, 1280, 1920, 3440 px; list and note reach their ends; buttons end inside the window. 375 and 640: one column, page scroll. Refresh keeps both scroll positions |

---

## Demonstration

**BEFORE** (Supervisor, 2026-11-27, assets of `1b86e12`, headless Chrome):

```
1920x1000  page scrolls; note panel 380 px wide, its buttons below the window for a long note
800x560    page scrolls; one list row shows time, one cut line, tags, kind chip and AI mark
name       easy-note
```

**AFTER** (same method, this branch):

```
800x560    page [560, 560]    list 300 x 305, note 424 x 409, buttons end at 547
1920x1000  page [1000, 1000]  list 440 x 697, note 924 x 825, buttons end at 983
name       Easy Taking Shot
```

**DELTA**: A person scrolls the list and reads a long note at the same time, with the note's buttons always in view.

**WITNESS**: Supervisor, 2026-11-27, `memory/event-trace/HUN-305.jsonl`.
