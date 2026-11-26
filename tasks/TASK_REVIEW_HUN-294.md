# TASK_REVIEW — HUN-294: [Short Title]

> Sibling of `tasks/TASK_GUIDE_HUN-294.md`. Everything here is **filled by the reviewer at Stage
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
Paused: auto-mode check failed — no `Memory:` line in the report — run the memory pass before you stop and record it there (each entry written, or `Memory: nothing new`); no test run in memory/event-trace/HUN-294.jsonl — write /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID with the task ID (minimal-track step 1): `mkdir -p /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d && printf '%s\n%s\n' "HUN-294" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID`, then rerun the task's tests so the trace records them under the task, then stop; Risk High, but no decision was DEBATED with 3 Advocates; extended track, but the trace has no `code-review` Skill call (Stage 4 runs in auto mode); Risk High, but the trace has no `security-review` Skill call [report be25f1359954]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/organize.rs` (13 tests, AC 1–10, 12 and the settled-decision test) and `src/core/organize.rs` unit tests (6: timeout scale, top-50 lists, prompt mask/cut/cap, array schema, item check table, mask before caps). `cargo test --locked --test organize`: `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.31s`; lib: `test result: ok. 74 passed; 0 failed` (includes the 6 `core::organize::tests`). AC 11 (overrides read once) is by code: `run_batch` calls `overrides::read` once and `ai::effective_with` per note, no test of the stderr count |
| Verification command run | ☑ pass | `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` at `49eafa0` content: `exit=0`; every binary `test result: ok`, e.g. `tests/organize.rs ... 13 passed`, `tests/enrich.rs ... 28 passed`, `tests/overrides.rs ... 16 passed`, `tests/leak.rs ... 46 passed`, `tests/window_api.rs ... 93 passed; 1 ignored`, lib `74 passed`; no clippy warning |
| Negative cases hold | ☑ pass | Red first: before the module existed `cargo test --locked --test organize` gave `error[E0432]: unresolved import easy_note_mcp::surfaces::organize`. Mutations of `src/surfaces/organize.rs`, each `1 file changed, 1 insertion(+), 1 deletion(-)`, then restored (`cmp` equal): M1 enrich gets the new kind instead of the old -> `a_stale_file_kind_change_is_logged_and_undone ... FAILED`, `with_a_kind_override_the_ai_kind_and_the_overrides_file_stay ... FAILED`; M2 body-hash recheck removed -> `a_note_edited_while_the_call_ran_is_skipped ... FAILED`; M3 kind override ignored -> `with_a_kind_override_the_ai_kind_and_the_overrides_file_stay ... FAILED` |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Self-review by the implementer only (no Stage 4 reviewer yet): new `src/core/organize.rs`, `src/surfaces/organize.rs`, `tests/organize.rs`; `src/surfaces/enrich.rs` change is visibility plus `call` taking the schema and timeout and `model_of` split out, covered by the unchanged `tests/enrich.rs` (28 passed). `src/core/ai.rs`, `src/core/overrides.rs`, `src/core/store.rs`, `src/core/guard.rs` not changed |
| Full smoke suite still green (no regression) | ☑ pass | Full `cargo test --locked` green, see the Verification row (all 27 test binaries `ok`) |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☐ N/A | The task has no UI: a core module and a library entry point only |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ N/A | The task has no UI |
| **UI: Responsiveness at target viewports** | ☐ N/A | The task has no UI |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured by backend-developer at commit `47899ac`, before any code change, with
`date -u; git rev-parse --short HEAD; ls src/core src/surfaces; ls src/core/organize.rs src/surfaces/organize.rs tests/organize.rs; grep -rn "organize::run_batch\|mod organize" src`:

```
2026-11-26T04:11:07Z
47899ac
src/core:
ai.rs
ai_view.rs
config.rs
context.rs
format.rs
fsutil.rs
guard.rs
mod.rs
overrides.rs
query.rs
store.rs
tags.rs

src/surfaces:
cli
window
cli.rs
dashboard.rs
enrich.rs
hook.rs
install.rs
mcp.rs
mod.rs
quick.rs
tray.rs
"src/core/organize.rs": No such file or directory (os error 2)
"src/surfaces/organize.rs": No such file or directory (os error 2)
"tests/organize.rs": No such file or directory (os error 2)
no organize module or run_batch in src
```

**AFTER**: captured by backend-developer after the change, with `date -u; ls src/core/organize.rs src/surfaces/organize.rs tests/organize.rs; grep -rn "pub fn run_batch\|^pub mod organize" src; cargo test --locked --test organize`:

```
2026-11-26T04:25:09Z
src/core/organize.rs
src/surfaces/organize.rs
tests/organize.rs
src/core/mod.rs:11:pub mod organize;
src/surfaces/organize.rs:167:pub fn run_batch(store: &Store, settings: &AiSettings, ids: &[String]) -> Result<Outcome, String> {
src/surfaces/mod.rs:9:pub mod organize;
test a_failed_call_junk_or_a_timeout_writes_nothing ... ok
test a_field_with_an_override_is_kept_and_reported ... ok
test a_huge_answer_is_read_to_1_mib_and_fails ... ok
test a_good_array_for_three_notes_writes_three_ai_files_and_log_lines ... ok
test a_stale_file_kind_change_is_logged_and_undone ... ok
test a_note_edited_while_the_call_ran_is_skipped ... ok
test each_item_is_checked_alone_and_a_bad_one_leaves_only_its_note_pending ... ok
test an_answer_that_fails_the_array_schema_writes_nothing ... ok
test raw_notes_stay_byte_identical_and_a_secret_never_reaches_stdin_ai_files_or_the_log ... ok
test the_prompt_holds_the_lists_and_the_cut_masked_notes_within_the_cap ... ok
test when_organize_fails_after_enrich_the_note_is_still_pending_for_organizing ... ok
test with_a_kind_override_the_ai_kind_and_the_overrides_file_stay ... ok
test with_enrich_off_nothing_runs ... ok
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.31s
```

**DELTA**: `surfaces::organize::run_batch` now sends up to `batch_size` pending notes to the person's `claude` in one call and writes each good answer item to its AI file (title, kind, summary, project, topic, tags, draft keys), leaving only bad items pending; before, no batch engine existed.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]
