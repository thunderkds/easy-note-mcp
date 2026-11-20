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
Checks: tests `<command>` — <result>; reviews <skill — result>; real run `<command>` — printed `<what it printed>` (auto-checked, not /verify)
Memory: <each entry the memory pass wrote, as `file — entry` | nothing new>
Not done: merge (yours); <auto verify: running (T182) | auto verify: <result> | /verify (yours), with [admin] only after the one retry also ended blocked>

-->

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | Backend slice (AC 1, 2, 4 at the API, 6): 7 new tests in `tests/window_api.rs` (section `HUN-280: capture from the window`): `context_is_the_newest_cli_or_hook_note_under_two_hours` (AC 1, Success 1), `context_is_null_when_the_last_note_is_two_hours_old` (AC 1), `add_keeps_the_last_context_like_quick_and_returns_the_new_item` (AC 2, Success 2; new note first on the board in `today`), `add_without_keep_context_writes_no_context` (AC 2, Success 2), `add_with_keep_context_and_no_last_context_saves_the_note` (FR-001), `add_of_empty_text_is_an_error_and_writes_nothing` (AC 4, Success 4), `add_masks_a_planted_secret_on_write` (AC 6, Success 3). Red before the change (Demonstration BEFORE: 7 failed, `{"error":"unknown command"}`); green after: `cargo test --locked --features window` 2026-11-20T11:02 → `Running tests/window_api.rs` … `test result: ok. 30 passed; 0 failed`. AC 3 and AC 5 (dialog keys, toast, selection) belong to the frontend slice. |
| Verification command run | ☑ pass | Run 2 (2026-11-20T10:41:13, load average 44.84): `cargo fmt --check`, `clippy` default, `cargo test --locked`, `clippy --features tray`, `cargo test --locked --features tray`, `clippy --features window` all passed (48 `test result: ok`, 0 warnings); its last step `cargo test --locked --features window` did not compile: `error: couldn't create a temp dir: No space left on device (os error 28)` (disk full, freed by the Supervisor). That last step rerun 2026-11-20T10:59:09 → 11:02:53: every suite `test result: ok` (lib 65, ai 38, cli 46, context 32, dashboard 5, enrich 24, format 13, guard 37, hook 16, hotkey 4, inbox 19, install 33, leak 46, mcp 20, perf 1 + 6 ignored, query 45, quick 26, readme 3, setup 19, store 30, tray 0, window_api 30), `EXIT 0`. Note: run 1 (10:27, load 44) and the first rerun of the last step (10:56) each failed only `tests/enrich.rs:642` `the save waited: 3.097666435s` / `4.503368899s` (a 3 s timing bound in `after_save_enriches_a_note_saved_by_{add,quick,mcp_note_add}`) under machine load; these tests do not reach `window::api` and passed in the default run and in the final run. |
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

**AFTER**: [same command, post-change] OR [verbatim excerpt of the new content]

**DELTA**: [one sentence — what a user can now do that they could not before]

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-280.jsonl`, never the
implementing agent alone]
