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
3. `toml_edit` is added as a direct dependency so a Settings save keeps comments — DEBATED: add toml_edit and edit only the owned keys (Advocate 1: add toml_edit; Advocate 2: add toml_edit) ⚠ decided for you — please review
4. The five P2 and six P3 review findings stay as follow-ups, with no code change after the review — RULE: skills/code-review/SKILL.md § Phase 4 — Apply Safe Fixes
5. The switch caption says the timer is not built yet, because the timer is the work of HUN-296 and is not on this branch — FACT: tasks/TASK_GUIDE_HUN-296.md:61 "**Entry point**: `easy-note tray` (a timer thread in `src/surfaces/tray.rs`)"
6. The notice wording and the switch caption are approved as written, the owner's answer in chat on 2026-11-26 — RULE: tasks/TASK_REVIEW_HUN-300.md § Auto decisions
7. A save keeps a symlinked config.toml before the merge, the owner's answer in chat on 2026-11-26; this replaces decision 4 for review finding P2-1 only — RULE: tasks/TASK_REVIEW_HUN-300.md § Auto decisions
Checks: tests `cargo test --locked` 679 passed 0 failed, `cargo test --locked --features window,tray` 688 passed 0 failed (the older `hun290_ac4` lock test failed once and passed on the rerun); reviews: fresh-context reviewer pass (P0 0, P1 0, P2 5, P3 6), Supervisor code-review no P0 or P1, security-review no High or Medium issue at confidence 8 or higher, Companion score fail (73); real run: Demonstration AFTER below (auto-checked, not /verify)
Memory: nothing new on this branch; the HUN-300 decision entry is written on main after the merge
Not done: merge (admin: after the auto verify pass; the failing Companion score row blocks it, so it is yours); the other P2 and the P3 follow-ups; AC 5 tray pickup waits for HUN-296; auto verify: running (T182)
Paused: auto-mode check failed — no `Memory:` line in the report — run the memory pass before you stop and record it there (each entry written, or `Memory: nothing new`); no test run in memory/event-trace/HUN-300.jsonl — write /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID with the task ID (minimal-track step 1): `mkdir -p /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d && printf '%s\n%s\n' "HUN-300" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID`, then rerun the task's tests so the trace records them under the task, then stop; extended track, but the trace has no `code-review` Skill call (Stage 4 runs in auto mode); Risk Medium, but the trace has no `security-review` Skill call [report be25f1359954]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass / ☐ fail | tests/enrich.rs (core writer, AC 1), tests/window_api.rs (IPC and page, AC 2 to 6, success criteria 2 and 3), tests/fixtures/window/page_harness.js (page scenarios). 33 new tests, each printed ok in the window,tray run; list below. The fix round adds 6 more in tests/enrich.rs (39 in total), listed in `## Fix round`. AC 5: toast and doctor are proven; that the tray picks a change up at its next tick is NOT provable on this branch, because the tray timer (HUN-296) is not in it. |
| Verification command run | ☑ pass / ☐ fail | The guide's five commands, chained, on the tree committed as ce0ed72 (the P2-1 fix), started 2026-11-26T05:22:56Z, ended 05:30:26Z, all exit 0, first run. Temp HOME, XDG_CONFIG_HOME and EASY_NOTE_DIR, DISPLAY and WAYLAND_DISPLAY unset, a trap `claude` first on PATH (never called). cargo fmt --check clean; both clippy runs with -D warnings clean; cargo test --locked: 685 passed, 0 failed, 7 ignored in 26 result lines; cargo test --locked --features window,tray: 694 passed, 0 failed, 7 ignored in 26 result lines. The older hun290_ac4 lock test passed in both. |
| Negative cases hold | ☑ pass / ☐ fail | Red runs before each implementation step. Core: test enrich did not compile, 27 errors (no save_ai_settings, notice_seen, CHANGED_OUTSIDE). IPC: 93 passed, 12 failed, 1 ignored. Page: 87 passed, 25 failed, 1 ignored (existing page tests failed too, because the harness already loaded the missing settings.js). UX fixes (short toast, no empty band): 109 passed, 3 failed, 1 ignored; one of the 3 was the older test hun290_ac4_the_lock_is_a_file_in_the_notes_dir_and_only_one_owner_holds_it, which passed in every later run (likely a race with child processes of parallel tests, not proven). No red run exists for the 375 px max-height change (checked on screenshots) or for the new caption assertion. Fix round (P2-1) red run, before the change to src/core/config.rs: of the 6 symlink tests 5 failed and 1 passed (the loop, which the read step already refused). Refusals covered by tests: a dangling link into a missing folder, a link loop, bad values, missing keys, invalid TOML, changed outside, no write permission, notice not confirmed, schedule with AI off, foreign page origin. |
| verify | ☑ pass | auto-run by hook, session b1c5a671-93c7-4d83-a25b-52f355ee29e8, commit 0866601386e5d9b4a22f380c282b70fa9bfd0312, log .claude/verify-runs/HUN-300-0866601386e5.log — pass |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass / ☐ fail | Fresh-context reviewer, `main...96b68c4`. Read: the diff of `src/core/config.rs`, `window/api.rs`, `window/mod.rs`, `assets/{settings.js,app.js,app.css,index.html,ipc.js}`, `Cargo.toml`, `Cargo.lock`, `README.md`, `tests/{enrich.rs,window_api.rs,fixtures/window/page_harness.js}`. Read as direct callers or neighbours: `surfaces/enrich.rs` (child argv, child env, `doctor` lines), `core/fsutil.rs`, `core/guard.rs` (masking), `assets/capture.js`. Skipped: store, MCP, hook, CLI and tray code (not changed by this diff). Details in `## Stage 4 review`. |
| Full smoke suite still green (no regression) | ☑ pass / ☐ fail | Both feature sets green on ce0ed72: 685 and 694 passed, 0 failed (verification row above). |
| **Companion score (easy-verifier)** | ☑ fail | overall 73 (floor 80 missed), change_risk pass (feature, value 100), scope changes, ref `main...HEAD` at 291ce2c, abstentions: reliability, api, data, operations, infrastructure; gates: blast-radius (70) and solution-fit (85) answered, none left abstained; no findings were sent, so no divergence. security 30 comes from redaction hits over the changed files; the diff adds no key-shaped value (grep of added lines: 0), and one review note quotes a contrived `secret=` example. report: `/home/hungnguyenhuu/.easy-verifier-reports/easy-note-mcp-HUN-300/evidence-report-changes-20261126T050303-955159Z.html` |
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

---

## Stage 4 review

Reviewer: fresh context, commit `96b68c4049cf2c0f39149f0bdc4c7f7969bbfd69`, diff `main...96b68c4` (main = merge base `9df4590`). Method: `skills/code-review/SKILL.md` and `references/security-reviewer.md`, applied by reading them. No source or test file changed.

**Verdict: pass. 0 P0, 0 P1, 5 P2, 6 P3.** Not ready to merge yet for a reason outside the code: the guide's Questions Log still says `OPEN HITL: owner approves the wording of the notice before merge`, and the `verify` row is empty. P2-1 to P2-4 are recommended before the merge.

### Commands run

All in the worktree, with a temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`, `DISPLAY` and `WAYLAND_DISPLAY` unset, and a trap `claude` script first on `PATH`. The trap was never called.

The guide's Verification Command, chained:

| Run | Step | Result |
|---|---|---|
| 1 (start 04:49:04Z) | `cargo fmt --check` | exit 0 |
| 1 | `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| 1 | `cargo test --locked` | **exit 101**: 678 passed, 1 failed, 7 ignored; the chain stopped (P2-5) |
| 2 (04:52:49Z to 04:59:11Z) | `cargo fmt --check` | exit 0 |
| 2 | `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| 2 | `cargo test --locked` | exit 0: 679 passed, 0 failed, 7 ignored (26 result lines) |
| 2 | `cargo clippy --all-targets --locked --features window,tray -- -D warnings` | exit 0 |
| 2 | `cargo test --locked --features window,tray` | exit 0: 688 passed, 0 failed, 7 ignored (26 result lines) |

The clippy steps ended in about 1 s each, from the build cache of the worktree. Six Settings page tests were also run alone with `--nocapture`: 6 passed in 10.05 s with `node` v26.7.0 and no `skipped: no node` line.

Probes: two throwaway tests in a `git archive` copy of the commit, outside the worktree, calling `window::api::handle`.

Lane check (warn only): `lane: C2, 14 files, 1953 lines, EXCEEDS (lines 1953 > 500); unpredicted: Cargo.toml, src/surfaces/window/assets/app.css, src/surfaces/window/assets/index.html, src/surfaces/window/assets/ipc.js, src/surfaces/window/assets/settings.js, src/surfaces/window/mod.rs, tasks/TASK_GUIDE_HUN-300.md, tests/fixtures/window/page_harness.js`. The Supervisor decides if the task was under-labelled.

### Acceptance Criteria

| # | Verdict | Evidence |
|---|---|---|
| 1 | pass, with gaps | `config.rs` `write_ai`, `set_ai`. Other keys, tables and comments stay for LF files (13 odd shapes probed, all parse after the save). Temp file plus rename; bad input refused with the key named (`save_ai_settings_refuses_invalid_input_naming_the_key_and_changes_nothing`). Gaps: P2-1, P2-2, P3-3, P3-4. `toml_edit` is logged as a DEBATE decision in the guide. |
| 2 | pass | All nine fields in `index.html`; values from `settings_get`; `ai_settings_shown` judges them with `ai_of`; a missing key is refused. |
| 3 | pass for turning on | Page: `set_cancel` sends no `settings_save`. Core: `api.rs:229` refuses off-to-on without `confirm_notice: true` or a matching `notice_seen`; probes with `schedule = "tray"` and AI off, a forged `notice_seen` and `confirm_notice: 1` were all refused. Notice text matches the AC word for word. See P2-3, P2-4, P3-1, P3-2. |
| 4 | pass | `command` and account folder are `<dd>` text, with the file path above. Never written: `ai_settings_shown` copies them from the file, `set_ai` has no such key. |
| 5 | partly proven | Toast and `doctor` are tested; the open window reads `after_save` at each save (`window/mod.rs`). The tray tick cannot be checked: the timer (HUN-296) is not on this branch. |
| 6 | pass | Switch disabled with `Turn on AI first.`; the core refuses `tray` with AI off, with and without confirm. |

Success criteria 1, 2 and 3: pass (named tests ran green in run 2). Edge cases: missing file, invalid TOML, changed outside, bad interval and no write permission are covered by passing tests. Entry points `settings_get`, `settings_save` and `config::save_ai_settings` exist; the window itself calls `save_ai_settings_if`.

### Findings

**P0**: none. **P1**: none.

**P2**

- **P2-1** (confidence 100, probe) `src/core/config.rs:389-401`: a symlinked `config.toml` is replaced by a regular file. Scenario: the file links to a dotfiles repo; the person saves in Settings; `error` is null, the link is gone, the target still holds the old text, the new file is mode 0600. A later edit of the dotfile (for example `enrich = "off"`) then has no effect. A dangling link is replaced the same way. `save_dir` already does this for `init`; this change makes a window button do it. Action: resolve the link and write beside its target; add a test.
- **P2-2** (confidence 100, probe) `src/core/config.rs:456`: a changed `jobs` list loses every comment inside the array. Before: `"organize", # keep me`, `# "drafts", disabled for now`, `"digest", # nightly` on separate lines. After unticking one job: `jobs = ["organize"]`. AC 1 asks to keep comments and README says they stay. The test `save_ai_settings_replaces_a_changed_jobs_list_and_keeps_its_line_comment` covers only the comment after the line. Action: edit the array in place, or state the limit.
- **P2-3** (confidence 100, code) `src/surfaces/window/assets/settings.js:114-120, 218-224, 193-197`: the switch saves at once when turned on, but not when turned off. Scenario: switch on, `Confirm` (written, toast `Saved`); switch off; `Close`. No save is sent and the file still says `schedule = "tray"`. No run follows today (no timer), but it would with HUN-296. `Confirm` also writes every other unsaved field of the form, such as `after_save`, which the notice does not mention. Action: save the off state too, or warn on Close, or let Confirm only set the switch and Save write it; add a page test.
- **P2-4** (confidence 100, probe and test `set_already_on`) `settings.js:84`, `api.rs:229`, `README.md:214-215`: while `Organize automatically` is on, a new model is saved with no notice, and `notice_seen` is not updated. The same holds for an account folder changed in the file. README says `another account folder or model shows it again`; that is true only when the switch is turned on again. AC 3 names only turning on, so the Supervisor or owner decides: ask again for a new text, or correct the README.
- **P2-5** (confidence 100 for the failure) `tests/window_api.rs:3082`: `hun290_ac4_the_lock_is_a_file_in_the_notes_dir_and_only_one_owner_holds_it` failed once with `free again` (run 1) and passed in run 2; the implementer saw the same. The test and `window/single.rs` are not changed here. Likely cause, not proven: a child process of a parallel test holds a copy of the lock descriptor until it execs. This diff adds more tests that start processes; the failure rate was not measured. Action: separate task to make the test stable (CI runs `cargo test --locked`).

**P3**

- **P3-1** (100, probe) `api.rs:176, 198`: the same notice text can be asked twice. Confirm with `haiku`, turn off, save model `sonnet`, type `haiku` again, turn on: `confirmed_model` is null so the page asks, although the core accepts it as already seen. Against `not shown again for the same text`; the error is on the safe side.
- **P3-2** (100, probe) `api.rs:160-171, 191`: masked values are used as real ones. `model = "my-secret=abcdef123"` is rewritten to `my-secret=[masked:secret_assign]` by a save that changes only the interval. Account folders `/a/secret=AAAA1111` and `/a/secret=BBBB2222` share one notice version. Both need odd values. Action: hash the unmasked account; do not write back a masked model.
- **P3-3** (100, probe) `config.rs:394-401`: the file mode becomes 0600, and a `config.toml` that the person made read-only (0400) in a writable folder is overwritten without a word.
- **P3-4** (100, probe) `config.rs:379-401`: a CRLF file is saved as LF and a BOM is removed, so lines the screen does not own change. In a multi-line inline `ai` table the last comment moves to the last new key.
- **P3-5** (75, code only, no real browser) `settings.js:239-243`: only a click on the backdrop keeps focus. A click on text inside either dialog can move focus to the page, where Escape reaches no handler. `capture.js` guards its whole dialog.
- **P3-6** (100) `tasks/TASK_GUIDE_HUN-300.md:172`: the `Pre-existing pin tests` row, which the guide says to fill before work starts, still holds its placeholder; the edge-case checklist is unticked.

### Security (Risk Medium)

Actor: page script, or a bad message on the IPC channel. Boundary: IPC into the core. Resources: `config.toml` and the argv of the child `claude`. No confirmed vulnerability.

- Rejected, TOML injection through `model`: three payloads with quotes, newlines and table headers were stored as one string; no `command` key appeared.
- Rejected, writing `command` or `claude_config_dir`: ignored in the request (code, test and probe).
- Rejected, argv injection: `model` is one argv item after `--model`, no shell; a leading `-` is refused.
- Rejected, other origin: `ipc_from` answers only `easy-note://localhost`; the quick window answers no settings command.
- Rejected, script injection in the page: `settings.js` sets text through `textContent` and `value` only; CSP unchanged.
- Rejected, file text or secrets in errors: parse errors keep the first line only, and answers pass `mask_for_output`. Odd input (wrong types, deep nesting, non-UTF-8 file, a directory as the file) gave an error object, no panic, no temp file left.
- Hardening notes: `confirm_notice` is a flag the page sends, so the core cannot know a dialog was shown. `model` has no length cap and accepts NUL over IPC (3 MB was written). A very narrow gap lies between the stamp check and the rename in `write_ai`.
- `needs_validation`: `config.rs:312` names the account from the window's `CLAUDE_CONFIG_DIR` when the file sets none. Missing fact: the environment of the process that will run the timer (HUN-296). Safe check: in HUN-296, compare the notice version in that process before a run.
- For the owner, no AC requires it: turning on `AI` with `Run after each save` sends masked text at each save with no notice, and the hint does not say that text is sent.

### UI Evidence rows, checked against the code

- Visual regression: not re-run. The screenshots sit in a session scratchpad outside the repo; the code does give the capture dialog's panel and the dashed disabled switch with its reason.
- Design system: confirmed 12 hex colours, one `rgba` value (the capture backdrop), spacing 4, 8, 12, 16, 20 px, IBM Plex only. One correction: `#3CF5A840` and `#3CF5A859` are not literally in the tokens file; they are alpha forms of `#3CF5A8` that the existing dialog and toast already use.
- Responsiveness: the CSS under `max-width: 480px` gives the header buttons, switches, inputs, checkbox rows and dialog buttons 44 px. Not measured again. The real window cannot go below 800 x 560 (`window/mod.rs`).

### Not verified

- `skills/ux-evaluate` and the `code-review` / `security-review` Skill calls: this reviewer had no Skill tool, so the trace holds no such call. No second agent re-checked the security leads.
- The real window on a display, and the startup-reads line of the implementer's report (not in the repo).

---

## Fix round

Finding P2-1 (a save replaced a symlinked `config.toml` with a regular file), fixed in commit `ce0ed72`, as the owner asked.

Fix, in `src/core/config.rs`: `write_ai` follows the links to the final target (new `link_end`, at most 40 links), makes the temp file in the target's folder and renames it over the target, so the link stays a link. A dangling link creates its target when the folder exists; else the save is refused with an error that names the config file and the target. A link loop is refused when the file is read (`cannot read <file>: Too many levels of symbolic links`). A plain file is written as before.

Tests, in `tests/enrich.rs`, each printed ok in both runs:

- `save_ai_settings_writes_through_a_relative_symlink` (also: the temp file is beside the target, and a later edit of the target is read)
- `save_ai_settings_writes_through_an_absolute_symlink`
- `save_ai_settings_follows_a_chain_of_symlinks_to_its_end`
- `save_ai_settings_creates_the_target_of_a_dangling_symlink`
- `save_ai_settings_refuses_a_dangling_symlink_into_a_missing_folder`
- `save_ai_settings_refuses_a_symlink_loop`

Not changed: `save_dir` (`easy-note init`) still replaces a symlinked file (follow-up). File modes are handled as before, so the written target gets mode 0600 (P3-3). CRLF handling and every other finding are unchanged. README says nothing wrong about this, so it is unchanged. This fix was made after the Stage 4 review commit and has had no second review.
