# TASK_REVIEW — HUN-282

> Sibling of `tasks/TASK_GUIDE_HUN-282.md`. Everything here is **filled by the reviewer at Stage
> 4/5** — it is deliberately NOT in the guide, because the implementing agent re-reads the guide on
> every turn and never fills these two sections.
>
> Consumers resolve each section **guide first, this file second** (`.claude/hooks/lib/guide_sections.py`):
> a legacy guide that still carries these sections inline keeps working unchanged, and a stray
> review file can never override an inline section.

---

<!-- Optional: the final report, only for a task run in auto mode (the user's `[auto]` or `[admin]` tag; with `[admin]`, the `Not done:` line names `merge (admin: after the auto verify pass)`,
`docs/claude-md/tracks.md` `## Auto mode`). To use it, copy the block below out of this comment, to this
place, and replace every <...>. Keep `Mode: auto` as its first line, exactly as written: T182's hook reads
it. Write it to this file before the run ends; the chat message is a copy, not the report. The same
block is the Supervisor's final chat message. It is not the `delivery-report` page and not
Evidence. A taste decision (what the user wants) also ends with the flag. Write `DEBATED` only when ≥2
Advocate `Agent` calls ran for that question, naming each Advocate's option. A decision with no Advocates is
labelled `RULE`, `FACT` or `SAFER OPTION`, never `DEBATED`. A decision flagged `⚠ decided for you` is a choice, and is never labelled `FACT`. A
safety action a rule requires (for example the backup before a delete) is `RULE` without the flag.
At Risk Medium or High, a `RULE:` counts only when it names a written rule (`path:line`,
`path § <heading>`, or `docs/claude-md/tracks.md` `### <section>`), and a `FACT:` needs a Citation
(`path:line "quoted text"`); without one it is a choice and must be `DEBATED`. Good: `2. backed up the CSV
— RULE: `docs/claude-md/tracks.md` `### Stops that stay``. Bad: `1. kept recurring.csv — RULE: Surgical Changes`.
`Memory:` records the diff-driven memory pass you run before you stop: each entry written, or `nothing new`.
After an auto verify pass the hook, never you, adds `Closed: row moved to Done <date> — …` and moves the row,
or adds `Paused: done not moved — <reason>`. It moves the row only when this block holds `Mode: auto`, a
numbered decision, `Checks:`, `Memory:`, the `Not done:` auto verify slot with its result, and `Not done:`
naming the merge — no `<...>` left (`docs/claude-md/tracks.md` `### Auto close-out`).
With `/verify (yours)`, your pass goes in the `verify` row below: Result `☑ pass`, Notes `/verify by the user, session <id>, commit <sha>`.

## Auto decisions

Mode: auto
1. <what was decided> — RULE: <the rule>
2. <what was decided> — FACT: <path>:<line> "<quoted text>"
3. <what was decided> — DEBATED: <option> (Advocate 1: <its proposal>; Advocate 2: <its proposal>) ⚠ decided for you — please review
4. <what was decided> — SAFER OPTION: <why> ⚠ decided for you — please review
Paused: <only when a stop that stays paused the task: which one, and why>
Checks: tests `<command>` — <result>; reviews <skill — result>; real run `<command>` — printed `<what it printed>` (auto-checked, not /verify)
Memory: <each entry the memory pass wrote, as `file — entry` | nothing new>
Not done: merge (yours); <auto verify: running (T182) | auto verify: <result> | /verify (yours), with [admin] only after the one retry also ended blocked>

-->

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/window_api.rs`: `board_items_carry_a_search_field_not_the_full_text` (AC 3), `a_long_repetitive_body_gives_a_smaller_board_reply` (AC 3), `the_search_field_is_masked` (AC 4), `the_page_asks_for_the_board_once_when_the_window_opens` (AC 1), `typing_in_search_renders_once_and_shows_the_same_rows` (AC 2, Success 2); pin tests changed: `board_counts_orders_and_marks_the_notes` (`text` now absent), `the_focus_refresh_calls_a_function_the_page_defines` (`refreshBoard`); the planted-secret tests are unchanged and green; the page tests run `tests/fixtures/window/page_harness.js` in node (real ipc.js + app.js, fake DOM, counting stub; skipped with a printed note when node is missing). Run: `window_api` 28 passed in the default and in the `window` build (lines in the verification output below) |
| Verification command run | ☑ pass | Exact guide command, run 2026-11-20T03:52Z on the tree that is committed next, EXIT 0, no `No space left` line: `cargo fmt --check` ok, both clippy runs `-D warnings` clean, default suite all `test result: ok` (window_api 28 passed), `--features window` suite all ok (window_api 28 passed), and `test perf_window_board ... perf window_board: n=30 first 195.2 ms p50 176.0 ms p95 206.6 ms max 228.8 ms` (budget 300 ms; ok). Full log: `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun282-verify.txt`. Note: the machine had 3 agents building at the same time; an earlier window-build run under the same load failed 2 timing tests in `tests/enrich.rs` (`after_save_enriches_*`, "the save waited: 3.09 s"), not touched by this task; they passed on the rerun above. p95 old 180.9 ms vs new 206.6 ms is noise of a loaded box (the board work is the same; the page-side saving is in the IPC/JSON size, below) |
| Negative cases hold | ☑ pass | Red run on the unchanged `app.js`/`api.rs` (new tests added first): `typing_in_search_renders_once_and_shows_the_same_rows` FAILED with `"rebuilt":10 ... left: Number(10) right: 1`; `the_page_asks_for_the_board_once_when_the_window_opens` FAILED with `"focus_before_answer":{"boards":2},"focus_after_answer":{"boards":2} ... left: Number(2) right: 1`; with the new `api.rs` but the old `app.js` the body-word search returned no rows (`"body":{"rows":[]`), so the test proves the page reads `search`. Then green. `git diff --stat` of the change: api.rs, app.js, ipc.js (comment), tests/window_api.rs, tests/fixtures/window/page_harness.js |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☐ pass / ☐ fail | [what was reviewed vs. skipped, and why] |
| Full smoke suite still green (no regression) | ☐ pass / ☐ fail | |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☑ pass | Pixel diff, not eyeballing: headless Chrome (`google-chrome --headless --window-size=W,H --virtual-time-budget=3000`) on a scratch copy of the page, BEFORE = `feat/HUN-279-actions` `assets/` with the old board (`text`), AFTER = this branch with the new board (`search`), same 6 sample notes (2 today with AI detail, 1 yesterday stale, 1 yesterday question, 2 earlier; one masked token + script-tag text; board from `window::api::handle` via the scratch `sample` binary). 8 pairs (board, search `firefox flaky` + Bugs, body-word search `ui_wait_for`, no-match `zzzz`; 1280x820 and 800x560): `compare -metric AE` = 0 differing pixels in all 8 (`/before-*.png` / `after-*.png`; script `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/shoot282.sh`). Same 1 row as HUN-278 `search-filter-1280`: "retry bug only on Firefox — ui_wait_for times out ..." with chip bug and AI. Screenshots are outside the repo, not committed |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | No CSS or markup change: `git diff --stat -- src/surfaces/window/assets/app.css src/surfaces/window/assets/index.html` is empty (0 lines of output). Only `app.js` (logic) and the `ipc.js` comment changed |
| **UI: Responsiveness at target viewports** | ☑ pass | 1280x820 and 800x560 (the window minimum): all 8 AFTER screenshots are pixel-identical to BEFORE (0 differing pixels), so no overflow or layout change; 800 px shows the same stacked detail panel and the same truncated notes-dir chip as before. 375 px is not a target: the window opens at 1280x820 with a 800x560 minimum (`mod.rs`) |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (captured 2026-11-20 on the unchanged code, branch `feat/HUN-282-perf` at 574f450, before any implementation commit; temp `HOME`, notes in temp dirs):

```
$ node tests/fixtures/window/page_harness.js src/surfaces/window/assets board-before.json   # real ipc.js + app.js in a node vm, fake DOM, stub window.ipc counting requests
focus_before_answer boards: 2
focus_after_answer  boards: 2
refresh_much_later  boards: 2
search rows rebuilt after 10 keystrokes: 11 (4 notes -> 4 li per render)
typed rows: ['Retry bug only on Firefox'] | +Bugs: ['Retry bug only on Firefox'] | cleared: ['Retry bug only on Firefox', 'Test /note during a running turn', 'Explain movement in the Wake brief', 'Buy a USB-C hub']
body-word query UI_WAIT_FOR: ['Retry bug only on Firefox']
(started 2026-11-20T03:23:06Z; focus_before_answer = the shell's focus refresh runs while the page's own start-up board request is in flight; focus_after_answer = 200 ms after the answer)
```

```
$ cargo test --locked --release --test zz_probe -- --nocapture   # throwaway probe, not committed; in-process handle(board), 5,000 notes, 30 timed runs
2026-11-20T03:19:27Z
probe four: board bytes 1272
probe short: 5000 notes, board bytes 1477962, p50 201.58646ms, p95 456.591824ms
probe long600: 5000 notes, board bytes 4760244, p50 220.054804ms, p95 288.20979ms
```

```
2026-11-20T03:23:17Z
$ cargo test --locked --release --test perf -- --ignored --nocapture --test-threads=1 perf_window_board
test perf_window_board ... perf window_board: n=30 first 182.9 ms p50 148.8 ms p95 180.9 ms max 202.4 ms
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 5.28s
```

So today: 2 `board` requests at start-up (both with the focus event before and after the answer); every board note item carries the full `text`; a 4-note board is 1,272 bytes, 5,000 short notes 1,477,962 bytes, 5,000 notes of 600-character bodies 4,760,244 bytes; `perf_window_board` p95 180.9 ms on this run (the guide's ~124 ms is the HUN-278 run; this machine is noisier); typing 10 characters re-renders the list 10 times.

**AFTER** (same commands on the changed code; 2026-11-20, working tree of branch `feat/HUN-282-perf`):

```
2026-11-20T04:02:14Z
$ node tests/fixtures/window/page_harness.js src/surfaces/window/assets board-after.json
focus_before_answer boards: 1
focus_after_answer  boards: 1
refresh_much_later  boards: 2
search rows rebuilt after 10 keystrokes: 1 (4 notes -> 4 li per render)
typed rows: ['Retry bug only on Firefox'] | +Bugs: ['Retry bug only on Firefox'] | cleared: ['Retry bug only on Firefox', 'Test /note during a running turn', 'Explain movement in the Wake brief', 'Buy a USB-C hub']
body-word query UI_WAIT_FOR: ['Retry bug only on Firefox']
```

```
2026-11-20T03:42:50Z
$ cargo test --locked --release --test zz_probe -- --nocapture
probe four: board bytes 1269
probe short: 5000 notes, board bytes 1477962, p50 325.766932ms, p95 443.428506ms
probe long600: 5000 notes, board bytes 2464674, p50 294.933024ms, p95 952.874188ms
(the loaded box makes these p50/p95 noisy: three agents were compiling; bytes are exact)
```

```
2026-11-20T03:52Z  (verification command, window build)
test perf_window_board ... perf window_board: n=30 first 195.2 ms p50 176.0 ms p95 206.6 ms max 228.8 ms
```

So now: 1 `board` request at start-up whether the focus event comes before or after the answer (a focus 60 s later still refreshes: 2); the board note items have no `text` and carry `search` (the masked body's distinct words); the 4-note board is 1,269 bytes (was 1,272), 5,000 short unique-word notes 1,477,962 bytes (unchanged: every word is distinct, so the search field is as long as the text), 5,000 repetitive 600-character notes 2,464,674 bytes (was 4,760,244); ten key presses are 1 list render (was 10), and the shown rows are the same. **Finding for the Supervisor**: AC 3 says "no full text" and search must still match every word of the body, so the search field must hold every distinct word; for ordinary notes the payload therefore barely shrinks. A real shrink needs the search to move into Rust (a `board` query), which changes the page/IPC design and is outside this task.

**DELTA**: Opening the window now asks the board once instead of twice, typing a search renders the list once after you pause instead of on every key press, and the board reply no longer carries each note's raw text (the same rows show, pixel for pixel).

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-282.jsonl`, never the
implementing agent alone]
