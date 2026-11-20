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
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/window_api.rs` (backend slice: AC 1, 2, 3, 4, 6; edge rows Undo-after-restore and leak. AC 2/3 row removal and toast, AC 5, AC 7 are the frontend slice). Default build, `cargo test --locked`: `test accept_ai_without_ai_detail_is_an_error_and_writes_nothing ... ok`<br>`test trash_then_restore_brings_back_the_note_and_its_ai_file ... ok`<br>`test open_in_editor_runs_xdg_open_once_with_the_note_path ... ok`<br>`test actions_refuse_bad_and_unknown_ids_and_change_nothing ... ok`<br>`test no_planted_secret_leaves_through_the_actions ... ok`<br>`test accept_ai_adds_the_suggested_tags_like_the_cli ... ok`<br>`test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s` (the same 21 of 21 in the `tray` and `window` builds) |
| Verification command run | ☑ pass | The guide's full command (fmt; clippy and test in the default, `tray` and `window` builds), run by the backend implementer from 2026-11-20T02:09:23Z to 2026-11-20T02:24:21Z: `exit=0`; 0 lines match `FAILED`, `panicked`, `^warning` or `^error`; 72 `test result` lines, 1630 passed, 0 failed; clippy `-D warnings` finished in all 3 builds. |
| Negative cases hold | ☐ pass / ☐ fail | [paste the red run against the prior content, or the mutation diff (`git diff --stat`, non-empty) before each verdict] |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☐ pass / ☐ fail | [what was reviewed vs. skipped, and why] |
| Full smoke suite still green (no regression) | ☐ pass / ☐ fail | |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☐ N/A | [screenshot path or LLM verdict — required for UI tasks, Hard-Stop Gate 6] |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ pass / ☐ fail / ☐ N/A | [method used + output] |
| **UI: Responsiveness at target viewports** | ☐ pass / ☐ fail / ☐ N/A | [viewports tested, any overflow findings] |

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

**AFTER**: [same command, post-change] OR [verbatim excerpt of the new content]

**DELTA**: [one sentence — what a user can now do that they could not before]

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-279.jsonl`, never the
implementing agent alone]
