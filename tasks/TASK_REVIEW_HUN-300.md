# TASK_REVIEW — HUN-300: [Short Title]

> Sibling of `tasks/TASK_GUIDE_HUN-300.md`. Everything here is **filled by the reviewer at Stage
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

## Auto decisions

Mode: auto
1. The three ready tasks (HUN-291, HUN-294, HUN-300) are built in parallel, one worktree and one implementer each, because the user asked for parallel execution — RULE: docs/claude-md/pipeline-stages.md § Stage 3: Parallel Execution via Isolation
2. Requirement Fidelity Gate signed `auto (Supervisor)` — RULE: docs/claude-md/tracks.md § Every user stop and its auto rule
Paused: auto-mode check failed — no `Memory:` line in the report — run the memory pass before you stop and record it there (each entry written, or `Memory: nothing new`); no test run in memory/event-trace/HUN-300.jsonl — write /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID with the task ID (minimal-track step 1): `mkdir -p /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d && printf '%s\n%s\n' "HUN-300" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID`, then rerun the task's tests so the trace records them under the task, then stop; extended track, but the trace has no `code-review` Skill call (Stage 4 runs in auto mode); Risk Medium, but the trace has no `security-review` Skill call [report be25f1359954]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass / ☐ fail | tests/enrich.rs (core writer, AC 1), tests/window_api.rs (IPC and page, AC 2 to 6, success criteria 2 and 3), tests/fixtures/window/page_harness.js (page scenarios). 33 new tests, each printed ok in the window,tray run; list below. AC 5: toast and doctor are proven; that the tray picks a change up at its next tick is NOT provable on this branch, because the tray timer (HUN-296) is not in it. |
| Verification command run | ☑ pass / ☐ fail | The guide's five commands, chained, on the tree committed as 8f5a8f8, started 2026-11-26T04:38:44Z, ended 04:47:11Z, all exit 0. cargo fmt --check clean; both clippy runs with -D warnings clean; cargo test --locked: 679 passed, 0 failed, 7 ignored in 26 result lines; cargo test --locked --features window,tray: 688 passed, 0 failed, 7 ignored in 26 result lines. |
| Negative cases hold | ☑ pass / ☐ fail | Red runs before each implementation step. Core: test enrich did not compile, 27 errors (no save_ai_settings, notice_seen, CHANGED_OUTSIDE). IPC: 93 passed, 12 failed, 1 ignored. Page: 87 passed, 25 failed, 1 ignored (existing page tests failed too, because the harness already loaded the missing settings.js). UX fixes (short toast, no empty band): 109 passed, 3 failed, 1 ignored; one of the 3 was the older test hun290_ac4_the_lock_is_a_file_in_the_notes_dir_and_only_one_owner_holds_it, which passed in every later run (likely a race with child processes of parallel tests, not proven). No red run exists for the 375 px max-height change (checked on screenshots) or for the new caption assertion. Refusals covered by tests: bad values, missing keys, invalid TOML, changed outside, no write permission, notice not confirmed, schedule with AI off, foreign page origin. |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☐ pass / ☐ fail | [what was reviewed vs. skipped, and why] |
| Full smoke suite still green (no regression) | ☑ pass / ☐ fail | Both feature sets green: 679 and 688 passed, 0 failed (verification row above). |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☑ pass / ☐ fail / ☐ N/A | pass by verdict on stub-IPC screenshots in /tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/df0fca59-a759-4a06-a24b-092db63ce7d8/scratchpad/hun300-ux.I61moj (files named state-width.png; no earlier baseline images to diff). Read at 375 and 1280: dashboard with gear, loading, off, on, notice, saved, saved-closed, bad value, no write permission, unreadable file; same panel, border and buttons as the capture dialog; disabled switch is dashed and shows Turn on AI first. After the last two edits (error height, switch caption) re-read: err-read-375, err-perm-375, off-375, on-1280. |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ pass / ☐ fail / ☐ N/A | Script over the new CSS block against docs/design/graphite-mint-tokens.json: 12 hex colours, 0 outside the tokens; rgba values rgba(8, 10, 9, 0.6) (the capture dialog's existing backdrop, not in the tokens file). Computed text fonts in headless Chrome: IBM Plex Sans and IBM Plex Mono only (the one Arial entry is a checkbox input with no text). Gap, padding and margin values in px: 4, 8, 12, 16, 20. Dialog radius 10 px as the capture dialog, controls 6 px. |
| **UI: Responsiveness at target viewports** | ☑ pass / ☐ fail / ☐ N/A | 375x812 and 1280x820, 12 states each (375 in a 375 px frame). Final measure run: no horizontal overflow of page or dialog, no element outside the viewport, no target under 44x44 at 375 in header, dialog or notice. Fixed on the way: error toast covering the dialog, empty band in the unreadable state, 12 px error text, folder chip squeezed by the gear, clipped unreadable-file message at 375. |

New tests, from `cargo test --locked --features window,tray`:

```
test save_ai_settings_changes_only_its_keys_and_keeps_every_other_byte ... ok
test save_ai_settings_never_writes_the_command_or_the_account_folder ... ok
test save_ai_settings_leaves_an_unchanged_file_byte_for_byte ... ok
test save_ai_settings_replaces_a_changed_jobs_list_and_keeps_its_line_comment ... ok
test save_ai_settings_creates_a_missing_file_with_only_the_ai_table ... ok
test save_ai_settings_adds_a_real_table_when_the_file_has_none ... ok
test save_ai_settings_edits_an_inline_ai_table_in_place ... ok
test save_ai_settings_edits_dotted_ai_keys ... ok
test save_ai_settings_refuses_invalid_input_naming_the_key_and_changes_nothing ... ok
test save_ai_settings_refuses_a_file_it_cannot_use_and_changes_nothing ... ok
test save_ai_settings_without_write_permission_is_an_error_that_names_the_file ... ok
test save_ai_settings_if_refuses_a_file_changed_since_it_was_read ... ok
test the_notice_is_the_agreed_text_and_its_version_follows_the_text ... ok
test notice_seen_is_read_as_a_string ... ok
test settings_get_answers_the_values_the_read_only_keys_and_the_notice ... ok
test settings_get_of_a_file_it_cannot_use_names_the_file ... ok
test organize_turns_on_only_with_the_confirmed_notice_and_records_it ... ok
test a_seen_notice_is_not_asked_again_until_its_text_changes ... ok
test organize_cannot_be_on_while_the_ai_is_off ... ok
test settings_save_refuses_a_bad_value_naming_the_key_and_writes_nothing ... ok
test settings_save_refuses_a_file_changed_after_settings_get ... ok
test settings_save_never_writes_the_command_or_the_account_folder ... ok
test doctor_shows_the_values_saved_from_the_window ... ok
test a_save_after_a_settings_change_follows_the_new_run_after_each_save ... ok
test the_settings_answers_mask_secrets_and_stay_on_the_app_page ... ok
test settings_save_without_write_permission_says_which_file ... ok
test the_gear_opens_settings_with_the_values_of_the_file ... ok
test cancel_on_the_notice_saves_nothing_and_leaves_the_switch_off ... ok
test confirm_on_the_notice_saves_once_and_shows_saved ... ok
test a_seen_notice_is_not_shown_again_and_a_new_text_is ... ok
test save_sends_the_typed_values_and_a_failed_save_keeps_the_dialog ... ok
test a_changed_or_unreadable_file_is_read_again ... ok
test the_page_has_the_settings_gear_dialog_and_notice ... ok
```

UX verdicts (`skills/ux-evaluate`), screenshots in `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/df0fca59-a759-4a06-a24b-092db63ce7d8/scratchpad/hun300-ux.I61moj`:

| Row | Verdict | Reason |
|---|---|---|
| US-020 turn on the organizer | pass | notice shows the AC 3 text with Confirm and Cancel; after Confirm the switch is on and the toast says Saved; the page test sees no run command |
| US-023 change how often | pass | Save shows the toast Saved; the saved interval is proven by the IPC and doctor tests |
| B-a primary action | pass | Save, Confirm and Try again in view at both widths |
| B-b hierarchy | pass | title, form, then actions |
| B-c spacing | pass | one grid; at 375 the number fields wrap 2 and 1 |
| B-d touch targets | pass | measured at 375, none under 44x44 |
| B-e readable text | pass | at 375 body and inputs 16 px, captions and errors 14 px, file path and the older toast 13 px |
| B-f states | pass | loading, off, on, saved, bad value, no permission, unreadable file; each error shows its text and button |
| B-g feedback | pass | toast, and the message in the dialog |
| B-h phone layout | pass | no horizontal scroll, no clipped control; the form scrolls inside the dialog |
| B-i one visual language | pass | one accent, one button style |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (captured before the first implementation commit; no `settings_get`, no `settings_save`, no `save_ai_settings`, no `notice_seen`, no direct `toml_edit`):

```
$ date -u +%Y-%m-%dT%H:%M:%SZ && git rev-parse HEAD && git status --short && grep -rnE 'settings_get|settings_save|save_ai_settings|notice_seen|toml_edit' src tests Cargo.toml README.md; echo "grep exit $? (1 = no match)"
2026-11-26T04:11:32Z
1e01f1811be37c7913b223894899c2624740bf8c
grep exit 1 (1 = no match)
```

**AFTER** (same command at `8f5a8f8`; 131 output lines, trimmed here to the definitions, `Cargo.toml` and `README.md` lines):

```
$ date -u +%Y-%m-%dT%H:%M:%SZ && git rev-parse HEAD && git status --short && grep -rnE 'settings_get|settings_save|save_ai_settings|notice_seen|toml_edit' src tests Cargo.toml README.md; echo "grep exit $? (1 = no match)"
2026-11-26T04:47:17Z
8f5a8f81b86ac50249538f14215b5e00f58be165
src/core/config.rs:125:    pub notice_seen: Option<String>,
src/core/config.rs:363:pub fn save_ai_settings(settings: &AiSettings) -> Result<(), ConfigError> {
src/core/config.rs:368:pub fn save_ai_settings_if(settings: &AiSettings, base: &str) -> Result<(), ConfigError> {
src/surfaces/window/assets/ipc.js:157:    settingsGet: function () { return request('settings_get'); },
src/surfaces/window/api.rs:173:fn settings_get() -> Result<Value, String> {
src/surfaces/window/api.rs:205:fn settings_save(request: &Value) -> Result<Value, String> {
README.md:214:screen and records `notice_seen`, so the same text is not shown again; another account folder or
Cargo.toml:34:toml_edit = { version = "0.25", default-features = false, features = ["parse", "display"] }
grep exit 0 (1 = no match)
```

**DELTA**: A person can now open Settings with the gear in the window and change the AI switches, limits, jobs and model, confirming what is sent before `schedule = "tray"` is saved, while the rest of config.toml stays as it was; the tray timer that would act on that value (HUN-296) is not on this branch, so "the tray picks it up at its next tick" (AC 5) is not provable here.

**WITNESS**: `memory/event-trace/HUN-300.jsonl` (main checkout) records the verification command at 2026-11-26T04:41:44.635833+00:00; its end (04:47:11Z) and the test counts come from the implementer's logs, not from the trace. No second witness yet: the auto verify run or the owner provides it.
