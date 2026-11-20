# TASK_REVIEW — HUN-280

> Sibling of `tasks/TASK_GUIDE_HUN-280.md`. Everything here is **filled by the reviewer at Stage
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
Checks: tests: the guide's full Verification Command at fb5600b — EXIT 0, 75 `test result` lines, 1663 passed, 0 failed, 0 clippy warnings (frontend agent, 2026-11-20T11:30–11:37); reviews: code-review (Supervisor) — 0 P0, 0 P1, 2 P2 (not fixed here: `src/surfaces/quick.rs` changed outside the predicted files (`last_context` made `pub(crate)`, takes `DateTime<FixedOffset>`; behaviour unchanged, reuses the FR-027 rule instead of copying it); this branch and `feat/HUN-282-perf` both change `board`/`detail` in `api.rs` and `app.js` — expect a merge conflict, and after merging check that `add`'s reply still carries `text` for the detail panel while board items carry `search`), 0 P3; security-review (Supervisor, inline) — no findings: `add` writes only through `store.add` (guard masks secrets on write, 20000-char limit, the page cannot pass a path, id or source), `context` reads only the notes dir, every reply masked by `mask_all`, only the `easy-note://` page can send IPC; ux-evaluate (frontend agent, from the skill file) — 11 pass, 0 fail, 2 not checked (375 px only); real run `easy-note dashboard` (window build, private Xvfb :97, temp env) — chip, Shift+Enter, Enter save with context, empty Enter no file, Backspace drops chip, Esc no file, read-only folder error toast (auto-checked, not /verify); follow-up 4659003 (owner decision 2026-11-20 "yes, should be"): window `add` starts `enrich::after_save` when `[ai] enrich` and `after_save` are on, through an `on_added` hook (`api::handle_with`, `ipc_with`); the shell reads `config::notes_dir_and_after_save()` once at open, the same call as `cli::write` add (`src/surfaces/cli/write.rs:47`) and `hook.rs:65` — code-review (Supervisor) no findings; full Verification Command EXIT 0, 1675 passed, 0 failed; not covered by a test: the shell glue (needs a display)
Memory: <each entry the memory pass wrote, as `file — entry` | nothing new>
Not done: merge (yours); auto verify: running (rerun for 4659003); companion score not run this round; the 2 P2 code-review findings above (follow-ups); a real-window check that a dialog save starts `ai enrich` (shell glue, untested)

-->

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | Backend slice (AC 1, 2, 4 at the API, 6): 7 new tests in `tests/window_api.rs` (section `HUN-280: capture from the window`): `context_is_the_newest_cli_or_hook_note_under_two_hours` (AC 1, Success 1), `context_is_null_when_the_last_note_is_two_hours_old` (AC 1), `add_keeps_the_last_context_like_quick_and_returns_the_new_item` (AC 2, Success 2; new note first on the board in `today`), `add_without_keep_context_writes_no_context` (AC 2, Success 2), `add_with_keep_context_and_no_last_context_saves_the_note` (FR-001), `add_of_empty_text_is_an_error_and_writes_nothing` (AC 4, Success 4), `add_masks_a_planted_secret_on_write` (AC 6, Success 3). Red before the change (Demonstration BEFORE: 7 failed, `{"error":"unknown command"}`); green after: `cargo test --locked --features window` 2026-11-20T11:02 → `Running tests/window_api.rs` … `test result: ok. 30 passed; 0 failed`. AC 3 and AC 5 (dialog keys, toast, selection) belong to the frontend slice. Frontend slice (AC 3, 4, 5 page side; page contract): in `tests/window_api.rs` the pin test `the_page_asks_only_for_commands_the_api_answers` now lists `context` and `add` (and checks each in the ipc.js contract comment); 2 new tests in section `HUN-280 frontend slice`: `the_page_has_the_design_capture_dialog` (`+ New note` button, `role="dialog" aria-modal="true" aria-label="New note"`, chip + `Drop context`, `Note…` box, the 4 key hints, dialog hidden at start, dialog CSS values from the design) and `the_capture_keys_and_save_follow_the_guide` (AC 3: `capKey` has `e.isComposing`, `e.key === 'Enter' && !e.shiftKey`, `e.key === 'Escape'`, `e.key === 'Backspace' && box.value === '' && keepContext`; AC 4: the `if (!text.trim()) return;` guard comes before `easyNoteIpc.add(text, keepContext)`; AC 5: `closeCapture()`, `'✓ noted'`, `loadBoard()`, `select(d.id)`; error keeps the dialog: `'could not save: ' + err.message`; `openCapture` calls `easyNoteIpc.context()` and `.focus()`; the backdrop `mousedown` focus guard; `loadBoard` never touches the dialog). Red before the page change (2026-11-20 ~11:10, `cargo test --locked --test window_api`): `test the_capture_keys_and_save_follow_the_guide ... FAILED` / `test the_page_asks_only_for_commands_the_api_answers ... FAILED` / `test the_page_has_the_design_capture_dialog ... FAILED` / `app.js lacks function capKey` / `index.html lacks id="new-note">+ New note</button>` / `test result: FAILED. 29 passed; 3 failed`. Green after (final run 11:37): `Running tests/window_api.rs` → `test result: ok. 32 passed; 0 failed` in each of the default, `tray` and `window` builds. The behaviour of the keys was also checked in the real window (Demonstration AFTER). **Follow-up (owner 2026-11-20, window `add` starts `enrich::after_save`)**: 4 new tests in `tests/window_api.rs` (section `HUN-280 follow-up`): `add_hands_the_new_id_to_the_hook_once` (flag on: the hook gets the new id once), `a_failed_add_and_every_other_command_start_nothing` (empty, missing text, `board`, `context`, unknown: hook never called), `without_a_hook_add_saves_the_same_reply` (flag off: `handle` / `ipc` save, no hook), `ipc_hands_the_new_id_to_the_hook_and_keeps_the_req`. Design: `window::api::handle_with(store, request, on_added: &dyn Fn(&str))` and `window::ipc_with(...)`; `handle` and `ipc` stay and pass a no-op, so no existing caller changed. `api.rs` reads no config; the shell (`window/mod.rs` `open()`) reads `config::notes_dir_and_after_save()` once and passes a hook that calls `enrich::after_save(&store.root, id)` only when the flag is on. The seam is a hook, not a bool, because an in-process test cannot call the real `after_save`: it starts `current_exe()`, which in a test is the test binary run as `ai enrich <id>`. The real spawn stays covered by `tests/enrich.rs` (`after_save_enriches_a_note_saved_by_add` and the other three). The `window` shell glue is not driven by a test (needs a display). Red first (2026-11-20T11:51:42+07:00, `cargo test --locked --test window_api`, before any change to `src/`): `error[E0432]: unresolved imports easy_note_mcp::surfaces::window::api::handle_with, easy_note_mcp::surfaces::window::ipc_with` / `no handle_with in surfaces::window::api` / `error: could not compile easy-note-mcp (test "window_api") due to 1 previous error`. Green after: `Running tests/window_api.rs` → `test result: ok. 36 passed; 0 failed` in each of the default, `tray` and `window` builds (was 32). |
| Verification command run | ☑ pass | Run 2 (2026-11-20T10:41:13, load average 44.84): `cargo fmt --check`, `clippy` default, `cargo test --locked`, `clippy --features tray`, `cargo test --locked --features tray`, `clippy --features window` all passed (48 `test result: ok`, 0 warnings); its last step `cargo test --locked --features window` did not compile: `error: couldn't create a temp dir: No space left on device (os error 28)` (disk full, freed by the Supervisor). That last step rerun 2026-11-20T10:59:09 → 11:02:53: every suite `test result: ok` (lib 65, ai 38, cli 46, context 32, dashboard 5, enrich 24, format 13, guard 37, hook 16, hotkey 4, inbox 19, install 33, leak 46, mcp 20, perf 1 + 6 ignored, query 45, quick 26, readme 3, setup 19, store 30, tray 0, window_api 30), `EXIT 0`. Note: run 1 (10:27, load 44) and the first rerun of the last step (10:56) each failed only `tests/enrich.rs:642` `the save waited: 3.097666435s` / `4.503368899s` (a 3 s timing bound in `after_save_enriches_a_note_saved_by_{add,quick,mcp_note_add}`) under machine load; these tests do not reach `window::api` and passed in the default run and in the final run. Frontend slice, final run on the committed page (2026-11-20T11:30:10 → 11:37:27, load average 6.80): the guide's full Verification Command, all 7 steps, `EXIT 0`; 75 `test result` lines, 1663 passed, 0 failed, 0 clippy warnings, `cargo fmt --check` clean; `window_api` 32 passed in each build, `readme` 3 passed. No `tests/enrich.rs` timing flake in this run. Follow-up run (window `add` starts enrich; 2026-11-20T11:56:09 → 12:04:32+07:00, load average 5.69): the guide's full Verification Command, all 7 steps, `EXIT 0`; 75 `test result` lines, 1675 passed, 0 failed (1663 + 4 new tests x 3 builds), 0 clippy warnings, `cargo fmt --check` clean; `window_api` 36 passed in each build. No `tests/enrich.rs` timing flake in this run, so no rerun was needed. |
| Negative cases hold | ☑ pass | Red runs before each implementation commit: backend at af06283 with only the new tests, `cargo test --locked --test window_api` → 7 failed, every capture call `{"error":"unknown command"}` (Demonstration BEFORE); frontend page-contract tests failed first (3 failed: `app.js lacks function capKey`), then passed at fb5600b. Empty / whitespace text → `{"error":"empty"}` and no file; missing or non-string text → `missing text`; > 20000 chars → `the text is too long`; a read-only month folder → `cannot use the note` with the dialog text kept (real run). |
| verify | ☑ fail | auto-run by hook, session fc151371-7f15-4bfe-acda-8f80b5caa843, commit 3e53756b69e12bac6b5cc61d5c167c0a19a83405, log .claude/verify-runs/HUN-280-3e53756b69e1.log — fail |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed `d592093...HEAD` only: `src/surfaces/window/api.rs` (`context`, `add`, `now`), `src/surfaces/quick.rs` (`last_context` signature), `src/surfaces/window/assets/*` (dialog), `tests/window_api.rs`, `README.md`, and the core call `Store::add` it reaches. Skipped CLI, MCP, tray: unchanged; `quick::run` behaviour unchanged (same call with `Local::now().fixed_offset()`). |
| Full smoke suite still green (no regression) | ☑ pass | Full Verification Command at fb5600b: EXIT 0, 1663 passed, 0 failed (default, `tray`, `window` builds). Earlier backend runs hit `No space left on device` and two `tests/enrich.rs` timing flakes (`the save waited: 3.09 s`) under load; both passed on rerun and do not reach `window::api`. |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☑ pass | LLM verdict against `docs/design/Main.dc.html` board "Dashboard (clickable)" (header button, capture dialog, toast), read from screenshots of the **real window** (window build, `easy-note dashboard`, temp `HOME` / `XDG_CONFIG_HOME` / `EASY_NOTE_DIR`, a `cli` note from a scratch git repo `easy-ui-mcp` on branch `fix/t120-firefox`; `GDK_BACKEND=x11` on a private `Xvfb :97`; keys and clicks sent with XTest; not committed). With chip: `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun280/shots/r1-open-chip-1280.png` — dialog 560 px at 14vh over the dimmed board, accent border + glow, chip `easy-ui-mcp · T120` with ×, `from your last note`, `Note…` placeholder with the caret in it, key hints `Enter save · Shift+Enter new line · Esc cancel · Backspace on empty: drop context`. Two lines typed: `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun280/shots/r2-typed-1280.png`. Without chip: `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun280/shots/r5b-typed-nochip-1280.png` (chip row gone after Backspace on the empty box). Saved: `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun280/shots/r3-saved-1280.png` toast `✓ noted · easy-ui-mcp · T120`, new note first under `// today` and selected; `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun280/shots/r6-saved-nochip-1280.png` toast `✓ noted`, row `no project`. Cancel: `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun280/shots/r7-esc-1280.png` toast `cancelled · nothing saved`. Error: `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun280/shots/r8-error-1280.png` toast `could not save: cannot use the note`, dialog and text `keep me` stay. Same layout and texts as the design. Differences: the chip line says `from your last note` without the design's `18 min ago`, because `context` returns only `{repo, task}`; the design's mock `save()` closes with `nothing saved` on empty text, but AC 4 says Enter does nothing, so the page follows AC 4. |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ pass | Method: every value in the new `app.css` rules copied from the design's inline styles and checked by eye side by side. `.new-note` = design button (`height: 38px; padding: 0 16px; background: #3CF5A8; color: #0E100F; border-radius: 6px`, glow `0 0 0 1px #3CF5A840, 0 0 18px #3CF5A840`, `font: 500 14px 'IBM Plex Sans'`); `.cap-back` = overlay (`rgba(8, 10, 9, 0.6)`, `padding-top: 14vh`, `z-index: 5`); `.cap` = dialog (`width: 560px; max-width: calc(100% - 32px)`, `#181B1A`, border `#3CF5A859`, radius 10px, `padding: 14px 16px 10px`, gap 10px); `.cap-chip` (`padding: 3px 4px 3px 9px`, `#2C3230`, mono 12px `#ABB3AE`), `.cap-drop` 20×20 `#8C9590`, `.cap-from` 12px `#6E7772`; `.cap-text` (`font: 400 18px/1.5 'IBM Plex Sans'`, `#EAEDEA`, no outline as the design's inline `outline: none`); `.cap-keys` (mono 11.5px `#8C9590`, gap 14px, `border-top: 1px solid #232826`, `kbd` `#DCE0DD`). Toast reuses the slice-1 `.toast` (same values as the design toast). Pinned values in `the_page_has_the_design_capture_dialog`. Fonts are the bundled IBM Plex files; no new colour outside the design. |
| **UI: Responsiveness at target viewports** | ☑ pass | Real window at 1280×820 (default) and at 800×820 (window minimum, `src/surfaces/window/mod.rs:150`; resized with `XResizeWindow`). 1280: dialog x 360–920, centred (`/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun280/shots/r1-open-chip-1280.png`). 800: dialog x 120–680, centred, chip and key hints fit, no overflow (`/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun280/shots/r10-open-chip-800.png`, `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun280/shots/r10b-typed-after-backdrop-800.png`); saved toast and selection at 800 (`/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun280/shots/r11-saved-800.png`). Finding fixed during the run: with a long notes dir the header wrapped `+ New note` onto a second line (first 1280 shot); `.brand` now shrinks (`flex: 1 1 0`) and the dir chip ellipsises, so the button stays at the right of the header line at both widths. 375 px: not reachable (window minimum 800 px). |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: The guide's Demonstration section is a `> **Moved.**` pointer and names no command, so the
backend agent used the new AC tests as the observable: `(date -Iseconds; cargo test --locked --test window_api 2>&1 | tail -30)`,
run in the worktree at `af06283` (no implementation commit yet; only the 7 new tests appended to
`tests/window_api.rs`). `context` and `add` are unknown commands:

```
2026-11-20T10:19:03+07:00
thread 'add_keeps_the_last_context_like_quick_and_returns_the_new_item' (645038) panicked at tests/window_api.rs:1146:51:
{"error":"unknown command"}

---- context_is_the_newest_cli_or_hook_note_under_two_hours stdout ----

thread 'context_is_the_newest_cli_or_hook_note_under_two_hours' (645048) panicked at tests/window_api.rs:1092:5:
assertion `left == right` failed
  left: Object {"error": String("unknown command")}
 right: Object {"context": Null}

---- context_is_null_when_the_last_note_is_two_hours_old stdout ----

thread 'context_is_null_when_the_last_note_is_two_hours_old' (645047) panicked at tests/window_api.rs:1130:5:
assertion `left == right` failed
  left: Object {"error": String("unknown command")}
 right: Object {"context": Null}


failures:
    add_keeps_the_last_context_like_quick_and_returns_the_new_item
    add_masks_a_planted_secret_on_write
    add_of_empty_text_is_an_error_and_writes_nothing
    add_with_keep_context_and_no_last_context_saves_the_note
    add_without_keep_context_writes_no_context
    context_is_null_when_the_last_note_is_two_hours_old
    context_is_the_newest_cli_or_hook_note_under_two_hours

test result: FAILED. 23 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

error: test failed, to rerun pass `--test window_api`
```

**AFTER**: Same command at the backend commit 2502164 → `test result: ok. 30 passed; 0 failed` (backend row above); with the page slice, `cargo test --locked --test window_api` → `test result: ok. 32 passed; 0 failed` (final run 2026-11-20T11:37). Real window run (frontend slice, window build of this branch, temp `HOME` / `XDG_CONFIG_HOME` / `EASY_NOTE_DIR`, private `Xvfb :97`, `GDK_BACKEND=x11`, XTest keys):

```
$ cd <scratch>/easy-ui-mcp   # git repo, branch fix/t120-firefox
$ easy-note add "retry bug only on Firefox #flaky"
✓ noted 20261120-112454-f035 · easy-ui-mcp · T120
$ easy-note dashboard        # 2026-11-20T11:26:38+07:00
click + New note           -> dialog, chip "easy-ui-mcp · T120", caret in the box
type "check the window capture #ux", Shift+Enter, "second line", Enter (11:27:01)
                           -> toast "✓ noted · easy-ui-mcp · T120", row first in today, selected
   20261120-112701-dffe.md: repo: "easy-ui-mcp" remote: "o/sample-repo" branch: "fix/t120-firefox"
   task: "T120" tags: ["ux"] source: "quick"; body "check the window capture #ux\nsecond line"
click + New note; Enter on empty; "   " + Enter -> dialog stays, files: 2 (no note)
Backspace x3 (spaces), Backspace on empty -> chip gone
type "buy a usb-c hub for the demo", Enter -> toast "✓ noted", row "no project"
   20261120-112744-1926.md: tags: [] source: "quick" (no repo, remote, branch, task)
click + New note; type "draft"; Esc -> toast "cancelled · nothing saved", files: 3 (no note)
chmod 555 notes/2026/11; + New note; "keep me", Enter -> toast "could not save: cannot use the note",
                           dialog and "keep me" stay; files: 3
resize to 800x820; + New note; click the backdrop; type "narrow window note", Enter
                           -> toast "✓ noted · easy-ui-mcp · T120", 20261120-112951-ef3f.md source: "quick"
click + New note; click the backdrop; Esc -> toast "cancelled · nothing saved"
```

The backdrop click was first a failure: it moved focus to the page body, so typing and Esc went nowhere (`r11` first try: empty box, files stayed 3). Fixed with a `mousedown` focus guard on the dialog's backdrop, pinned in `the_capture_keys_and_save_follow_the_guide`, and re-run as above.

**DELTA**: In the dashboard window the person can now press `+ New note`, type a note (with the last project's context, or without it after Backspace) and save it with Enter, and it appears selected at the top of today; before, the window could not add a note.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-280.jsonl`, never the
implementing agent alone]
