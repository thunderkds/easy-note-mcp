# TASK_REVIEW — HUN-279: Dashboard window actions

> Sibling of `tasks/TASK_GUIDE_HUN-279.md`. Everything here is **filled by the reviewer at Stage
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
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/window_api.rs` (backend slice: AC 1, 2, 3, 4, 6; edge rows Undo-after-restore and leak. AC 2/3 row removal and toast, AC 5, AC 7 are the frontend slice). Default build, `cargo test --locked`: `test accept_ai_without_ai_detail_is_an_error_and_writes_nothing ... ok`<br>`test trash_then_restore_brings_back_the_note_and_its_ai_file ... ok`<br>`test open_in_editor_runs_xdg_open_once_with_the_note_path ... ok`<br>`test actions_refuse_bad_and_unknown_ids_and_change_nothing ... ok`<br>`test no_planted_secret_leaves_through_the_actions ... ok`<br>`test accept_ai_adds_the_suggested_tags_like_the_cli ... ok`<br>`test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s` (the same 21 of 21 in the `tray` and `window` builds)<br>**Frontend slice** (`tests/window_api.rs`, page contract: AC 2/3 6 s Undo, AC 5 clipboard API + `Copied`, AC 7 board reload after each IPC action, every command the page asks for is one `handle` answers and the `ipc.js` contract names it). Red first, before the page change: `test the_page_asks_only_for_commands_the_api_answers ... FAILED` (`left: {"board", "detail"}` `right: {"accept_ai", "board", "detail", "open_in_editor", "restore", "trash"}`), `test result: FAILED. 2 passed; 2 failed`. Green at a590905: `test the_page_asks_only_for_commands_the_api_answers ... ok`<br>`test the_page_has_the_design_actions_and_the_undo_window ... ok`<br>`test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s`. AC 2, 3, 5, 7 behaviour in the page also seen in the real window run (see the UI rows and the Demonstration). |
| Verification command run | ☑ pass | The guide's full command (fmt; clippy and test in the default, `tray` and `window` builds), run by the backend implementer from 2026-11-20T02:09:23Z to 2026-11-20T02:24:21Z: `exit=0`; 0 lines match `FAILED`, `panicked`, `^warning` or `^error`; 72 `test result` lines, 1630 passed, 0 failed; clippy `-D warnings` finished in all 3 builds.<br>Frontend implementer, the same full command after the page change, from `2026-11-20T02:30:18Z` to `2026-11-20T02:38:47Z` (tree = commit a590905): `exit=0`; 0 lines match `FAILED`, `panicked`, `^warning` or `^error`; 75 `test result` lines, 1636 passed, 0 failed (`readme` 3/3: `every_command_the_readme_names_is_a_real_subcommand ... ok`). An earlier run at 02:29:51Z stopped at `cargo fmt --check` (`Diff in tests/window_api.rs:1042`); fixed with `cargo fmt`, then the run above. |
| Negative cases hold | ☐ pass / ☐ fail | [paste the red run against the prior content, or the mutation diff (`git diff --stat`, non-empty) before each verdict] |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☐ pass / ☐ fail | [what was reviewed vs. skipped, and why] |
| Full smoke suite still green (no regression) | ☐ pass / ☐ fail | |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☑ pass | LLM verdict against `docs/design/Main.dc.html` board "Dashboard (clickable)", read from screenshots (not committed). **Real window** (`easy-note dashboard`, window build at a590905, temp `HOME` / `XDG_CONFIG_HOME` / `EASY_NOTE_DIR` with 5 sample notes, 3 with AI detail; `GDK_BACKEND=x11` on a private `Xvfb :99`, so the clicks did not touch the live desktop; clicks sent with XTest; captures with `import -window`): `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/shots/real-1-selected.png` — detail panel ends with the design's action row: border-top line, `Open in editor` and `Copy id` on the left, `Move to trash` in red (#E8968F) on the right; in the AI box `suggested #firefox #flaky #e2e` and `Add as tags` at the right. `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/shots/real-2-copy.png` toast `Copied` at bottom right (`xclip -o -selection clipboard` → `20261120-092515-a1b2`). `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/shots/real-3-accept.png` toast `tags added: #firefox #e2e`, the panel shows `#flaky #firefox #e2e`, `Add as tags` gone (no tag missing), the row's line updated. `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/shots/real-5-trash.png` row gone, next note selected, counts 2→1 today / 5→4 total / 3→2 projects / `Bugs 0`, toast `moved to trash` + green `Undo`. `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/shots/real-9-undone.png` toast `restored`, row back in its place (first under `// today`), counts back to 2/5/3. **Headless Chrome** (scratch copy of `assets/` with a stub `window.ipc` fed recorded real `window::api::handle` answers; the copy drops the meta CSP because a `file://` page cannot load scripts under `default-src easy-note:`): `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/shots/h-trash-1280.png`, `h-copyno-1280.png` (`copy it by hand: 20261120-091646-a1b2`), `h-undo-elsewhere-1280.png` (`could not undo: no such note`), `h-openfail-800.png` (`could not open: xdg-open failed`). Same buttons, labels, toast text and place as the design. Differences: the copy toast says `Copied` (AC 5) where the design says `copied <id>`; `Open in editor` toast says `opened <id>.md` where the design says `opens <id>.md in $EDITOR` (the window runs `xdg-open`, not `$EDITOR`); error and copy-by-hand toasts are not in the design; `Add as tags` is not shown on inbox notes (read-only, the backend refuses them). |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ pass | Method: each new CSS rule in `src/surfaces/window/assets/app.css` compared value by value with the inline styles of the design board (accent #3CF5A8, glow on). `.actions` = design action row `display: flex; flex-wrap: wrap; gap: 8px; padding-top: 4px; border-top: 1px solid #1F2321`. `.act` = `height: 34px; padding: 0 14px; background: transparent; color: #DCE0DD; border: 1px solid #2C3230; border-radius: 6px; font: 500 13px 'IBM Plex Sans'`; `.act.danger` = `margin-left: auto; color: #E8968F`. `.ai-accept` = `margin-left: auto; height: 30px; padding: 0 12px; … font: 500 12.5px 'IBM Plex Sans'`. `.toast` = `position: fixed; right: 24px; bottom: 24px; z-index: 6; display: flex; align-items: center; gap: 14px; max-width: 420px; padding: 12px 16px; background: #181B1A; border: 1px solid #3CF5A859 (accentLine); border-radius: 6px; box-shadow: 0 0 0 1px #3CF5A840, 0 0 18px #3CF5A840 (neonGlow); font-size: 13px; color: #EAEDEA`; text in IBM Plex Mono; `.toast-undo` = `background: none; border: 0; padding: 0; color: #7EDDB0; font: 500 13px 'IBM Plex Sans'`. Additions not in the design: `box-sizing: border-box` on the toast, `overflow-wrap: anywhere` and `user-select: text` on its text (a long id wraps and can be selected for copy by hand). No new colour, font or radius. Toast times from the design script: 2800 ms, 6000 ms with Undo. |
| **UI: Responsiveness at target viewports** | ☑ pass | 1280 px (the window's default size, 1280x820) and 800 px (its minimum width, `with_min_inner_size(800, 560)`). 1280: `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/shots/real-5-trash.png`, `h-trash-1280.png`, `h-copyno-1280.png` — the three buttons fit on one line in the 380 px detail column; the toast sits at bottom right over the detail column only (the list ends at x=852; the longest toast, `copy it by hand: <id>`, starts at x≈933), so it does not cover the list. 800: `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/shots/h-sel-800-full.png` — the columns stack (list, then detail), the buttons fit on one line, no horizontal scroll, no clipped control; `h-trash-800.png` and `h-openfail-800.png` — the toast sits over the lower right of the detail panel, not the list. Finding (no fix, design placement kept): at 800 px with a list taller than the window, the fixed toast can cover the right end of the bottom row (kind chip, `AI` mark) for 2.8–6 s; the row stays usable on its left part. |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: `date -u; git rev-parse --short HEAD; cargo test --locked --test window_api` with the new HUN-279 tests in
`tests/window_api.rs` and no implementation (every action command answers `unknown command`), captured by the
backend implementer before the first implementation commit:

```
2026-11-20T02:07:23Z
1cb5eb2
test every_reference_in_the_page_is_served ... ok
test every_served_asset_is_scanned_and_safe ... ok
test an_empty_or_missing_notes_dir_gives_zeros ... ok
test accept_ai_without_ai_detail_is_an_error_and_writes_nothing ... FAILED
test navigation_stays_on_the_app_scheme ... ok
test inbox_notes_are_on_the_board_and_have_a_detail ... ok
test bad_requests_answer_an_error_object ... ok
test accept_ai_adds_the_suggested_tags_like_the_cli ... FAILED
test the_csp_allows_only_the_app_scheme ... ok
test no_planted_secret_leaves_through_the_actions ... FAILED
test the_focus_refresh_calls_a_function_the_page_defines ... ok
test the_reply_is_a_quoted_js_string ... ok
test the_window_opens_only_when_built_and_a_display_is_set ... ok
test board_counts_orders_and_marks_the_notes ... ok
test open_in_editor_runs_xdg_open_once_with_the_note_path ... FAILED
test actions_refuse_bad_and_unknown_ids_and_change_nothing ... FAILED
test trash_then_restore_brings_back_the_note_and_its_ai_file ... FAILED
test detail_has_the_ai_detail_and_the_context ... ok
test ipc_echoes_req_on_answers_and_errors ... ok
test top_projects_keep_three_ties_by_name ... ok
test no_planted_secret_leaves_through_board_or_detail ... ok
  left: Object {"error": String("unknown command")}
 right: Object {"error": String("no AI detail")}
  left: Null
 right: "20261119-090000-acdc"
  left: Object {"error": String("unknown command")}
 right: Object {"opened": String("20261119-090000-acdc")}
  left: Object {"error": String("unknown command")}
 right: Object {"error": String("missing id")}
  left: Object {"error": String("unknown command")}
 right: Object {"trashed": String("20261119-090000-acdc")}
test result: FAILED. 15 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

**AFTER**: the same command, after the backend (1878867) and frontend (a590905) commits, run by the frontend implementer:

```
2026-11-20T02:54:20Z
a590905
test every_reference_in_the_page_is_served ... ok
test every_served_asset_is_scanned_and_safe ... ok
test an_empty_or_missing_notes_dir_gives_zeros ... ok
test inbox_notes_are_on_the_board_and_have_a_detail ... ok
test accept_ai_without_ai_detail_is_an_error_and_writes_nothing ... ok
test detail_has_the_ai_detail_and_the_context ... ok
test bad_requests_answer_an_error_object ... ok
test board_counts_orders_and_marks_the_notes ... ok
test navigation_stays_on_the_app_scheme ... ok
test the_csp_allows_only_the_app_scheme ... ok
test the_focus_refresh_calls_a_function_the_page_defines ... ok
test the_page_has_the_design_actions_and_the_undo_window ... ok
test actions_refuse_bad_and_unknown_ids_and_change_nothing ... ok
test the_window_opens_only_when_built_and_a_display_is_set ... ok
test the_reply_is_a_quoted_js_string ... ok
test the_page_asks_only_for_commands_the_api_answers ... ok
test ipc_echoes_req_on_answers_and_errors ... ok
test top_projects_keep_three_ties_by_name ... ok
test open_in_editor_runs_xdg_open_once_with_the_note_path ... ok
test trash_then_restore_brings_back_the_note_and_its_ai_file ... ok
test no_planted_secret_leaves_through_the_actions ... ok
test no_planted_secret_leaves_through_board_or_detail ... ok
test accept_ai_adds_the_suggested_tags_like_the_cli ... ok
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

Real window run (window build at a590905, temp `HOME` / `XDG_CONFIG_HOME` / `EASY_NOTE_DIR`, `GDK_BACKEND=x11` on a private `Xvfb :99`, a fake `xdg-open` first on `PATH`), started `2026-11-20T02:52:54Z`:

```
$ easy-note dashboard   (window build; DISPLAY=:99 GDK_BACKEND=x11 EASY_NOTE_DIR=<scratch>/env/notes-real)
click Copy id      -> toast "Copied"; xclip -o -selection clipboard: [20261120-092515-a1b2]
click Add as tags  -> toast "tags added: #firefox #e2e"
$ easy-note show 20261120-092515-a1b2
tags: ["flaky", "firefox", "e2e"]
body bytes unchanged
click Open in editor -> fake xdg-open calls: 1:<scratch>/env/notes-real/2026/11/20261120-092515-a1b2.md
click Move to trash  -> .easy-note/trash/20261120-092515-a1b2.md, .easy-note/trash/ai/20261120-092515-a1b2.md.json
click Undo (1.47 s after trash) -> toast "restored"
57f89d6d…5569  env/notes-real/2026/11/20261120-092515-a1b2.md
b099909d…e8bb  env/notes-real/.easy-note/ai/20261120-092515-a1b2.json
note file and AI file back at their paths, contents equal
```

An earlier try clicked `Undo` more than 6 s after `Move to trash`: the toast was already gone, the click did nothing, and the note stayed in the trash (AC 3: no Undo after 6 s); `easy-note restore 20261120-092515-a1b2` printed `✓ restored 20261120-092515-a1b2`.

**DELTA**: In the dashboard window the person can now add the AI's suggested tags, open the note in their editor, copy its id and move it to the trash with a 6-second Undo, and the counts update at once; before, the window could only show notes.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-279.jsonl`, never the
implementing agent alone]
