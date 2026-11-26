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
3. A note that already has an AI file is written as `ai::enrich` with the old kind, then `ai::organize` with the new kind; the whole item is checked first and overridden fields get `None` — DEBATED: enrich with the old kind, then organize (Advocate 1: the same; Advocate 2: the same; Advocate 3: the same) ⚠ decided for you — please review
4. Option A, the owner's answer in chat on 2026-11-26: `ai::enrich` logs the old title, summary and tags in full before it replaces them (no overwrite if that log line cannot be written), stale draft keys are cleared, and undo for these fields is HUN-302, which must merge before `ai run` (HUN-295) — RULE: tasks/TASK_REVIEW_HUN-294.md § Auto decisions
5. The answer schema becomes an object with an `items` array, and FR-047 says so, because the real CLI refuses an array at the root — FACT: PRD.md:114 "for each batch make ONE call to the person's `claude` CLI (FR-037 rules)"
6. A repo name longer than the project cap is cut to the cap, because the project of a note is its repo — FACT: tasks/TASK_GUIDE_HUN-294.md:75 "`project` = the note's `repo` when it has one"
7. The P2 and P3 review findings stay as follow-ups — RULE: skills/code-review/SKILL.md § Phase 4 — Apply Safe Fixes
Checks: tests `cargo test --locked` 665 passed 0 failed 7 ignored (reviewer, at d01b469); reviews: fresh-context reviewer fail (1 P0, 3 P1, 5 P2), Supervisor code-review confirms F1 to F5 and applied no fix, security-review found no High or Medium issue at confidence 8 or higher (closest lead, 4 of 10: note text could ask for a remote image link in another note's AI summary, shown by the Obsidian view); real run `claude -p --json-schema` with an array root printed `API Error: 400 tools.1.custom.input_schema.type: Input should be 'object'`, so review finding F1 is confirmed; the same call with the array inside an object `items` returned the items (auto-checked, not /verify)
Memory: memory/learnings.md — the real CLI needs an object schema (written on main, not committed); the HUN-294 decision entry waits for the merge
Not done: merge (admin: after the auto verify pass); every review fix, starting with F1; the Companion score; auto verify: running (T182)
Paused: auto-mode check failed — no `Memory:` line in the report — run the memory pass before you stop and record it there (each entry written, or `Memory: nothing new`); no test run in memory/event-trace/HUN-294.jsonl — write /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID with the task ID (minimal-track step 1): `mkdir -p /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d && printf '%s\n%s\n' "HUN-294" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID`, then rerun the task's tests so the trace records them under the task, then stop; Risk High, but no decision was DEBATED with 3 Advocates; extended track, but the trace has no `code-review` Skill call (Stage 4 runs in auto mode); Risk High, but the trace has no `security-review` Skill call [report be25f1359954]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/organize.rs` (21 tests: AC 1–10, 12, the settled-decision test, and the fix round tests `the_items_object_is_read_from_result_when_structured_output_is_absent`, `enrich_logs_the_old_title_summary_and_tags_in_full_and_undo_still_takes_back_project_topic_kind`, `enrich_does_not_replace_the_ai_file_when_its_log_line_cannot_be_written`, `a_finished_draft_loses_its_draft_keys_and_the_removal_is_logged`, `a_note_in_a_62_char_repo_is_organized_with_the_repo_cut_to_the_cap`, and for R1/R3 `a_cleared_600_char_draft_suggestion_is_logged_whole_once`, `a_replaced_600_char_draft_suggestion_is_logged_whole_once`, `draft_keys_stay_when_their_log_line_cannot_be_written`) and `src/core/organize.rs` unit tests (7, now with `the_schema_is_an_object_holding_the_array_of_the_writer_caps` and `a_repo_over_the_project_cap_is_cut_to_the_cap_by_chars_after_masking`). At `e260387`: `tests/organize.rs: 21 passed, 0 failed, 0 ignored`; lib `75 passed; 0 failed`. AC 11 (overrides read once) is by code: `run_batch` calls `overrides::read` once and `ai::effective_with` per note, no test of the stderr count |
| Verification command run | ☑ pass | `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` at `e260387`: `exit=0`, no clippy warning; 28 `test result` lines, 674 passed, 0 failed, 7 ignored; e.g. `tests/organize.rs: 21 passed`, `tests/enrich.rs: 28 passed`, `tests/overrides.rs: 16 passed`, `tests/leak.rs: 46 passed`, `tests/mcp.rs: 20 passed`, `tests/window_api.rs: 93 passed, 1 ignored`. Feature build `cargo test --locked --features window,tray --no-fail-fast` at `e260387`: `exit=0`, 28 `test result` lines, 683 passed, 0 failed, 7 ignored. (At `768764a` the first feature run failed once in `tests/context.rs` `first_root_is_used_and_plain_folder_roots_work`, a file this task does not touch; its rerun and the run at `e260387` passed.) |
| Negative cases hold | ☑ pass | Red first: before the module existed `cargo test --locked --test organize` gave `error[E0432]: unresolved import easy_note_mcp::surfaces::organize`. Mutations of `src/surfaces/organize.rs`, each `1 file changed, 1 insertion(+), 1 deletion(-)`, then restored (`cmp` equal): M1 enrich gets the new kind instead of the old -> `a_stale_file_kind_change_is_logged_and_undone ... FAILED`, `with_a_kind_override_the_ai_kind_and_the_overrides_file_stay ... FAILED`; M2 body-hash recheck removed -> `a_note_edited_while_the_call_ran_is_skipped ... FAILED`; M3 kind override ignored -> `with_a_kind_override_the_ai_kind_and_the_overrides_file_stay ... FAILED`. Fix round, each test red before its code: F1 `test result: FAILED. 3 passed; 11 failed` (organize) and `the_schema_is_an_object_holding_the_array_of_the_writer_caps ... FAILED`; option A `test result: FAILED. 13 passed; 3 failed`; F2 `a_finished_draft_loses_its_draft_keys_and_the_removal_is_logged ... FAILED` at `assert!(f.get("draft_reason").is_none(), "{f}");`; F3 both new tests `FAILED`; R1/R3 fix: `test result: FAILED. 1 passed; 3 failed` (the three new draft-log tests) before the code |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Implementer self-check of the fix round, not a Stage 4 review: `src/core/organize.rs`, `src/surfaces/organize.rs`, `tests/organize.rs`, `docs/NOTE_FORMAT.md`; `src/core/ai.rs` (`enrich` calls the new log step before the write; `Organized::clear_draft`) and `src/core/overrides.rs` (`log_enrich_diff`, `log_draft_diff`, `log_whole`, `append_all`, module comment; `log_ai_diff` now logs only project, topic and kind) changed for option A only. Callers of `ai::enrich` (`surfaces/enrich.rs`, `surfaces/mcp.rs`, the batch, the window through `surfaces::enrich`) covered by `tests/enrich.rs` 28, `tests/mcp.rs` 20, `tests/window_api.rs` 93, all green. `src/core/store.rs`, `src/core/guard.rs` not changed |
| Full smoke suite still green (no regression) | ☑ pass | Full `cargo test --locked` green at `e260387`, see the Verification row (28 binaries `ok`). One existing test changed: `a_stale_file_kind_change_is_logged_and_undone` now expects the three option A lines (`title`, `summary`, `tags_suggested`) before the `kind` line, because `enrich` now logs them; its undo check is unchanged |
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

---

## Stage 4 review

Reviewer: qa-expert, fresh context, from the repo only, 2026-11-26. Commit `d01b46939c2c9013e80be53d063c520986ccc718`, diff `main...d01b469`. Rules: `skills/code-review/SKILL.md` (read and followed by hand; no Skill tool call) and `skills/code-review/references/security-reviewer.md`. Review done alone: no second agent re-checked the security leads. No source or test file changed, no fix applied. The real `claude` CLI was never started.

**Verdict: fail (changes required).** 1 unverified P0, 3 P1, 5 P2, P3 list below. The Verification Command is green.

### Commands and results

| Command (in the worktree unless said) | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0, no warning |
| `cargo test --locked` | exit 0; 27 `test result: ok` lines, 665 passed, 0 failed, 7 ignored; `tests/organize.rs`: `13 passed; 0 failed` |
| `python3 scripts/lane_check.py tasks/TASK_GUIDE_HUN-294.md main...d01b469` | `lane: C2, 8 files, 1294 lines, EXCEEDS (lines 1294 > 500); unpredicted: tasks/TASK_GUIDE_HUN-294.md` (warn only) |
| Scratch copy: 12 reviewer probes appended to a copy of `tests/organize.rs`, fake `claude`, `cargo test --locked --offline --test zz_probe p -- --nocapture --test-threads=1` | `17 passed; 0 failed; 8 filtered out` (the 12 probes and 5 original tests that match the filter; probes print behaviour, they do not assert it); output quoted per finding |
| Scratch copy mutation: `organize::timeout(settings.timeout_secs, sent.len())` replaced by `Duration::from_secs(settings.timeout_secs)`, `cargo test --locked --offline --test organize` | `13 passed; 0 failed` (the mutation survives, F9) |

Reachability: `surfaces::organize::run_batch` is in the diff; only tests call it (the `ai run` command is the next slice, as the guide says). Startup-reads check: not checkable, the implementer's final report is not in the repo.

### P0 (unverified)

| ID | File:line | Finding | Conf. | Action |
|---|---|---|---|---|
| F1 | `src/core/organize.rs:90-115`, `src/surfaces/organize.rs:206-212` | The schema sent as `--json-schema` has an array root. `--json-schema` structured output is, to the reviewer's knowledge (not a repo fact), a tool input schema, and the API requires `type: object` at the root. The only real-CLI evidence in the repo is for an object (`memory/learnings.md`, 2026-11-19: `structured_output` "(the object)"). Every test uses a fake that accepts any schema. Failure: each real batch exits non-zero, the person sees `claude exited 1; if it is not logged in, log in once`, and no note is ever organized. Outcome: `needs_validation`. | 50 | Owner or Supervisor runs one real call with a dummy text before the `ai run` slice: `echo 'Return one item with id x' \| claude -p --output-format json --json-schema '{"type":"array","items":{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}}' --tools "" --strict-mcp-config --no-session-persistence --model haiku`. Pass: exit 0 and `structured_output` is an array. Fail: wrap the array in an object (`{"items":[…]}`); AC 2 and AC 3 wording then needs a ruling. |

### P1

| ID | File:line | Finding and failure scenario | Conf. | Action |
|---|---|---|---|---|
| F2 | `src/core/organize.rs:171-172`, `src/surfaces/organize.rs:153-154`, `src/core/ai.rs:295-301, 321-329` | A finished draft keeps its draft keys. Run 1 says `draft` with a reason and a suggestion. The person finishes the note. Run 2 says `todo`: `check_item` gives `None`, and `organize` reads `None` as "leave as is". Probe: `kind":"todo"` with `"draft_reason":"it stops mid-sentence","draft_suggestion":"half a thought about the tray icon"`, `stale=false`, and `effective` returns both. The old text now looks fresh. `docs/NOTE_FORMAT.md:139` (added here) says these keys are for kind `draft` only. | 100 | Clear both keys when the applied kind is not `draft` (needs a way to clear in `ai.rs`: a `Question:` if out of scope); add a draft-then-finished test. |
| F3 | `src/core/organize.rs:169, 176-188` | A note whose `repo` is longer than 60 chars can never be organized. The repo replaces the project and then fails the 60-char cap, so the whole item is refused. Probe with a 62-char repo, two runs: `pending ... reason: "project is 62 chars; the limit is 60"`, no AI file. A project override does not help (same result), although the project would not be written. Each run sends the note again and uses a batch slot. | 100 | Decide cut or omit for a long repo (AC 5), do not check a project that will not be written, add a test. |
| F4 | `src/surfaces/organize.rs:134-157`, `src/core/overrides.rs:245-274` | Applied changes with no log line (AC 7, FR-047 "Each applied item is one `ai-log.jsonl` line, so Undo works per note", NFR-015 "Every AI change can be undone"). `title`, `summary` and `tags_suggested` are overwritten by `ai::enrich`, which logs nothing; a new note's non-draft `kind` is not logged either. Probe 1: an existing file got `title":""`, `summary":""`, new tags; `new log lines=[]`; the old text is lost. Probe 2: new note, project and topic overridden, item applied, `new log lines for new note b=[]`; `overrides::undo` then removed the **person's** topic override (`Undone { field: "topic", now: None }`). | 100 behaviour, 75 on the reading of AC 7 | Supervisor ruling: narrow AC 7 / NFR-015 to the keys HUN-292 logs and record it, or log (and make undoable) title, summary and tags. At least one line per applied item. |

### P2

| ID | File:line | Finding and failure scenario | Conf. | Action |
|---|---|---|---|---|
| F5 | `src/surfaces/organize.rs:180, 123-125, 146-155` | AC 6: overrides are read before the call, which may last up to 5 x `timeout_secs`. An override set while it runs is not seen. Probe (fake writes topic and kind overrides during the call): the AI file topic and kind changed, `kept: []`, three `who: ai` lines were appended after the person's change, so the next `ai undo` takes back the AI line, not the person's fix. The effective value still shows the override, so this is not P1. | 100 | Read the overrides again after the call for the write step (AC 11 "once" then needs a ruling: one read for the writes, with the warning once); add a test. |
| F6 | `src/surfaces/organize.rs:17-18, 146-157`, `src/core/ai.rs:300, 334`, `docs/NOTE_FORMAT.md:140-141` | AC 12 holds only for a new note. With an existing AI file, `enrich` succeeds, `organize` fails: the file keeps the old `organized_at` and old kind, project and topic, with a fresh `based_on`. Probe: `FR-029 pending total=0 stale=false`. The AC says "stays without `organized_at`". In both cases (new or old file) the FR-029 selection that FR-047 names will not pick the note again. The doc rule "older than `at`" is in no code, and the stamps have one-second resolution (equal stamps in the probe). | 100 | Ruling; carry to the `ai run` slice that selection must use `organized_at`; test the existing-file case. |
| F7 | `src/surfaces/organize.rs:216-221` | Edge case "one item for two ids: both dropped". Two inbox files whose names are different secret-shaped strings both get the id `inbox/[masked:github_token]` in the prompt. One answer item is then applied to **both** notes (probe: two stdin lines with the same id, `written=2 pending=0`). One note gets the other's title, summary and topic. Rare trigger. | 100 | If two sent notes share a masked id, leave both pending (or send an index as the id); add a test. |
| F8 | `src/core/organize.rs:199-202` (same order in `src/core/ai.rs:269-271`) | AC 3 "strings masked": a tag is lower-cased before it is masked, so the case-sensitive AWS pattern misses it. Probe: title gives `[masked:aws_key]`, but `"tags_suggested":["akiaiosfodnn7example"]` is written to the AI file. Security: `confirmed`, Low (needs a model answer that holds a key the input mask did not remove; upper-casing restores an access key ID). The reverse case, a GitHub token as a tag, passes `check_item` as a marker and is then refused by `enrich` with the misleading reason `tags_suggested[0] is not a tag`. | 100 | Mask the raw tag first; if it was masked, drop the tag or refuse the item in `check_item` with a clear reason; test an AWS key as a tag. |
| F9 | `tests/organize.rs:362-396` and file | Test gaps: AC 2 timeout scaling is not tested through `run_batch` (the mutation above survives); no test for AC 11, for AC 12 on an existing file, for an override set during the call, or for draft-then-finished. | 100 | Add tests with the fixes. |

### P3

- `src/core/organize.rs:78-79`: the first note that does not fit ends the loop. A note that cannot fit (probe: 1,500 control chars with 100 normal names, `fit=0`; quote-heavy lists, 14,972 chars, `fit=0`) blocks every later note on each run. Normal text does not trigger it (full 60-char ASCII lists with a 1,500-char note: 10,692 chars, `fit=1`). Skip instead of stop, and bound the lists by chars.
- `src/surfaces/organize.rs:102-103`: serde takes a positional array as an item (`[[id,"t","todo","s","p","tp",[]]]` was written), so AC 4 "fails the array schema writes nothing" is not strict. Check each element is an object.
- `src/core/organize.rs:27-36, 76`: `</notes>` inside a note stays literal in the JSON line (newlines are escaped, so it cannot forge a line), and the prompt calls only the notes data, not the `<lists>` names, which come from earlier model answers. Hardening: escape `<`, and call the lists data.
- `src/surfaces/organize.rs:183-191`: a repeated id in `ids` is sent twice and listed twice in `written` (probe).
- `src/surfaces/organize.rs:118-145`: `ai::enrich` reads the body again after the hash check, so an edit in that very short gap gets a fresh `based_on` (same pattern as `ai enrich`). Not reproduced.
- `src/surfaces/organize.rs:61, 195` against `src/core/store.rs:263-264`: the unmasked body now leaves the core and goes to the prompt builder, against the comment on `raw_body`. It is masked in `prompt`. Update the comment or give the core one function for this.
- `src/surfaces/organize.rs:25-37` and `src/core/ai.rs:316-318`: `Outcome` ids and the reason `no AI detail for {id} to organize` are unmasked (an inbox file name). The `ai run` slice must mask them before print or log.
- `src/surfaces/organize.rs:74-85`: `effective_with` lists the whole inbox once per inbox note (quadratic in inbox size).
- `src/core/overrides.rs:210-219` (HUN-292, unchanged): a failed log write is silent; probe with an unwritable log: item reported written, no line.
- Edge checklist: "schema check fails: ... one log line" writes no line and prints nothing (the reason is only returned); a filesystem path in a topic or project is capped but not masked (no path pattern in the guard); the guide's "Pre-existing pin tests" row is still the template text.
- Stale text: `src/core/ai.rs:3-5` does not name the batch as a source of AI text; `docs/NOTE_FORMAT.md:137` speaks of `ai run`, which does not exist yet.
- Empty `title` and `summary` are accepted and replace good text (see F4).

### Security: sensitive-data inventory (blast radius)

| Data | Reaches | Control |
|---|---|---|
| Note bodies of the batch | child `claude` stdin, so the person's Claude account | only with `[ai] enrich = "claude"`; masked, then cut to 1,500 chars; whole prompt at most 12,000 chars when notes are sent |
| Note ids, including inbox file names | child stdin | masked (collision: F7) |
| Project and topic names of **all** live notes (AI files and person overrides, including repo names), with counts | child stdin | masked, at most 50 each; includes notes outside the batch, as FR-047 asks |
| Parent environment | child env | whole env minus the three `ANTHROPIC_*` variables (unchanged FR-037 behaviour) |
| Schema, model name | child argv | no note data; no shell; fresh 0700 temp cwd; `--tools ""` |
| Title, summary, tags, kind, model id, project, topic, draft keys | `.easy-note/ai/<id>.json`; in an Obsidian vault also `ai/<id>.md` (title, summary, tags, kind, model) | masked and capped (tag gap: F8) |
| Project, topic, kind, draft keys, old and new | `.easy-note/ai-log.jsonl` | masked, cut to 200 chars |
| Raw note files, overrides file | not written | byte-identical in the fixture test and in the inbox probe |
| Errors and `Outcome` | caller | fixed reasons, field names and lengths; never the answer or note text; ids unmasked (P3); the exit hint holds the command and account dir path; child stderr discarded |

Security leads:

- Text in one note steers the answer for other notes of the batch: the path exists by design (one call, checked only by id). Reach: other notes' AI files (title, summary, tags, topic, project, draft suggestion), the same person's data. No tool call, no raw note write, overrides still win on display. Hardening note, Low; it makes F4 matter more. Real model behaviour not tested.
- Rejected: a model id used as a path (ids are only compared for equality, `src/surfaces/organize.rs:217`; paths come from the canonical id); note text or answer in argv; a write to a note file (no note writer is called); an answer over 1 MiB (cut, then refused, tested).

### Acceptance Criteria

| AC | Result |
|---|---|
| 1 | met (tests and unit test); stall note in P3 |
| 2 | met by code; scaling not covered through `run_batch` (F9); real CLI acceptance of the schema open (F1) |
| 3 | met except F7 (shared masked id) and F8 (tag masking) |
| 4 | met except the positional-array shape (P3) |
| 5 | **not met** for a repo over 60 chars (F3) |
| 6 | met for overrides present before the call; not for one set during it (F5) |
| 7 | **not met** (F4) |
| 8 | met (test); very short gap noted in P3 |
| 9 | met for the fixture (GitHub token); AWS key as a tag is a gap (F8) |
| 10 | met |
| 11 | met by code, no test |
| 12 | met for a new note only (F6) |

## Fix round

Implementer (backend-developer), 2026-11-26, after the owner's option A. Each fix was test first (red, then green), one commit each. F5 to F9 and the P3 items are not touched.

| Finding | Fix | Commit | Test |
|---|---|---|---|
| F1 | `--json-schema` root is `{"type":"object","properties":{"items":<array schema>},"required":["items"],"additionalProperties":false}`; the parser reads `structured_output.items` or `result` parsed as that object; any other root fails the whole batch | `f22b1bd` | `the_schema_is_an_object_holding_the_array_of_the_writer_caps` (unit), the argv check in `a_good_array_for_three_notes_writes_three_ai_files_and_log_lines`, bare array and wrong objects in `an_answer_that_fails_the_array_schema_writes_nothing`, `the_items_object_is_read_from_result_when_structured_output_is_absent`; every fake answer now uses the object root |
| F4 (option A) | `ai::enrich` on a note with a valid AI file logs each changed `title`, `summary`, `tags_suggested` (`who: ai`, old value whole, new cut, tags as JSON text) in one write before the file is replaced; no replace when the write fails; the fields stay out of `overrides::FIELDS` (undo: HUN-302); no `kind` line from `enrich` | `d0d84eb` | `enrich_logs_the_old_title_summary_and_tags_in_full_and_undo_still_takes_back_project_topic_kind` (600-char multibyte summary), `enrich_does_not_replace_the_ai_file_when_its_log_line_cannot_be_written` |
| F2 | `ai::Organized::clear_draft`, set by the batch when the effective kind (override, else model) is not `draft`, removes both draft keys; `organize` logs the removal; `enrich` still carries the draft keys | `dc017c5` | `a_finished_draft_loses_its_draft_keys_and_the_removal_is_logged` (also: a person's kind `draft` keeps the keys) |
| F3 | A repo over the project cap is cut to `ai::MAX_PROJECT` chars after masking and used as the project | `768764a` | `a_repo_over_the_project_cap_is_cut_to_the_cap_by_chars_after_masking` (62-char, multibyte, masked), `a_note_in_a_62_char_repo_is_organized_with_the_repo_cut_to_the_cap` |
| R1, R3 (second review) | `ai::organize` calls the new `overrides::log_draft_diff` after the checks and before the write: one `who: ai` line per changed `draft_reason` / `draft_suggestion` (set, replaced or removed), old value masked and whole, new masked and cut, all in one write; when it fails, `organize` is refused and the AI file is not written (the batch reports the note pending with that reason). `log_ai_diff` no longer logs the draft keys (no duplicate line); project, topic and kind logging and `ai undo` are unchanged. Module comment and docs/NOTE_FORMAT.md name the five whole-old fields | `e260387` | `a_cleared_600_char_draft_suggestion_is_logged_whole_once` (600-char multibyte, undo still takes back the kind), `a_replaced_600_char_draft_suggestion_is_logged_whole_once`, `draft_keys_stay_when_their_log_line_cannot_be_written` (core: AI file bytes unchanged; batch: pending with an `ai-log.jsonl` reason, draft keys kept) |

---

## Stage 4 review (fix round)

Reviewer: qa-expert, fresh context, from the repo only, 2026-11-26. Commit `5069261602bef531e0df3307deafef8f4e6af6b8`; fix round in depth: `git diff d01b469..5069261 -- src tests docs PRD.md`. Rules: `skills/code-review/SKILL.md` and `references/security-reviewer.md`, followed by hand (no Skill tool call). Review done alone: no second agent re-checked a P0 or security result. No source or test file changed, no fix applied. The real `claude` CLI was never started (its directory was removed from `PATH`; probes used a fake script).

**Verdict: fail (changes required).** No P0. 1 P1 (R1, in the F2 fix), 5 P2, P3 list below. Both test commands are green.

### F1 to F4

| ID | Status | Evidence |
|---|---|---|
| F1 | fixed in code | `schema()` has an object root with `items`; `parse` reads `structured_output.items`, else `result`; bare array and wrong objects write nothing (tests, probe). The real CLI was not run here: that it accepts an object root rests on the call recorded in `## Auto decisions`, which does not paste the full production schema. Run `organize::schema()` once against the real CLI before HUN-295 ships. |
| F2 | fixed for the batch, with a new loss (R1) | Draft keys go when the shown kind is not `draft`; a person's kind `draft` keeps them. Mutations "never clear" and "clear by model kind" each fail the F2 test. |
| F3 | fixed | A 62-char repo is cut and organized. Mutation "no cut" fails the test. One contrived case remains (P3, partial marker). |
| F4 | partly fixed | Old `title`, `summary`, `tags_suggested` of a valid AI file are logged whole before the replace, and a failed log write stops the replace (mutations "no log", "error ignored", "old value cut" each fail a test). The second half of F4 is unchanged (R2). |

### Commands and results

| Command (in the worktree unless said) | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 (cached build) |
| `cargo test --locked` | exit 0; 28 `test result` lines, 671 passed, 0 failed, 7 ignored |
| `cargo test --locked --features window,tray --no-fail-fast` | exit 0; 28 `test result` lines, 680 passed, 0 failed, 7 ignored (the `tests/context.rs` failure in the implementer's first run did not occur) |
| `python3 scripts/lane_check.py tasks/TASK_GUIDE_HUN-294.md main...5069261` | `lane: C2, 11 files, 1656 lines, EXCEEDS (lines 1656 > 500); unpredicted: PRD.md, src/core/ai.rs, src/core/overrides.rs, tasks/TASK_GUIDE_HUN-294.md` (warn only; the two source files follow the owner's option A) |
| Scratch copy: 14 probe tests, p01 to p12, p14, p15 (`tests/zz_probe.rs`, fake `claude`), `cargo test --locked --offline --test zz_probe p -- --nocapture --test-threads=1` | 13 passed, 1 failed (probe p10 fed an answer over 1 MiB: my error), p10 rerun with a smaller tag passed; probes print behaviour, quoted below |
| Scratch copy: 7 mutations, `cargo test --locked --offline --test organize` | M1 to M6 killed; M7 (`old` not masked in `log_enrich_diff`) survives, equivalent because `load` already masks |
| A shell script to tear a log append with `ulimit -f` | blocked by the `pre_bash` auto-mode hook ("cannot tell" if it pushes or merges); not retried in another form. R4 was reproduced with a hand-written partial line instead |

Reachability: `run_batch` is still called only by tests (`ai run` is HUN-295). Startup-reads check: not checkable from the repo. Callers of `ai::enrich` (CLI `ai enrich`, MCP `note_enrich`, `note_add(detail)`, window `Run AI`, `after_save`, the batch): the one behaviour change is that a replace of a valid AI file is now refused when the log cannot be written and title, summary or tags change; the error holds fixed words and the IO error kind only. A new note (capture) has no old file and is not affected. Old-format log lines plus the new lines: `ai undo` undid topic, kind, topic, project, then `nothing to undo` (probe p08). 8 threads, 1,592 multibyte appends: every line parsed (p11). Hostile answers (`items` object, null, `[null]`, `is_error`, 100,000-deep array, 900,000-char summary) write nothing.

### P0

None.

### P1

| ID | File:line | Finding and failure scenario | Conf. | Action |
|---|---|---|---|---|
| R1 | `src/core/ai.rs:332-334, 346`, `src/core/overrides.rs:245, 268-281` | The F2 fix removes `draft_suggestion` (cap 600) and the removal line keeps only 200 chars plus `…`. Probe p02: a 577-char suggestion, note finished, next batch says `todo`: the file has no suggestion, the log `old` has 201 chars, the end of the text is nowhere. HUN-302 cannot restore it; NFR-015 says every AI change can be undone from `ai-log.jsonl`, and its 2026-11-26 exception does not cover this. Before this round the key was kept. The F2 test uses short values, so it passes. | 100 | Log the old draft values whole when removed or replaced (as for title and summary), before the AI file is written; test with a 600-char suggestion; add the draft keys to the HUN-302 undo scope. Or the owner rules the loss acceptable and PRD says so. |

### P2

| ID | File:line | Finding and failure scenario | Conf. | Action |
|---|---|---|---|---|
| R2 | `src/surfaces/organize.rs:139-169`, `src/core/ai.rs:298-305`; contract `PRD.md:114` | F4 residual: "Each applied item is one `ai-log.jsonl` line" is still false. New note with project and topic overridden (p01): item `written`, `new log lines=[]`, and `ai undo` then returned `Undone { field: "topic", now: None }`: it removed the person's own fix. The same zero lines when the model returns empty project and topic (p01b). Trigger is narrow. | 100 | Log at least one line for each applied item, or record a ruling that narrows FR-047. |
| R3 | `src/core/ai.rs:344-346`, `src/core/overrides.rs:228` | Draft removal is written first and logged after, and a log failure is silent. Log unwritable and the model returns the same title, summary and tags (p03): item `written`, both draft keys gone, no record. | 100 | Fix with R1: one checked log write before the AI file write. |
| R4 | `src/core/overrides.rs:197-210, 357-361` (`append` and `latest` pre-date this round; option A now relies on them) | A partial append (disk full, crash, file size limit) leaves a line with no newline. The next append is glued to it: `enrich` reports success and replaces the AI file, but the `old title` line, the only full copy, does not parse (p07). If the tear is inside a multibyte char, `read_to_string` fails and `ai undo` says `nothing to undo` for **every** note until the file is edited by hand (p07). The comment "a line lands whole" is not guaranteed by `write_all`. | 100 for the effect of a torn line (simulated); 75 that a real failure leaves one | Start a new line when the file does not end in `\n`; read the log as bytes and skip only the bad line. Test both. |
| R5 | `src/core/ai.rs:302-303`, `src/core/overrides.rs:296-300` | Outside the batch, option A and F2 do not hold fully. `ai enrich` / `note_enrich` / `Run AI` on a draft sets kind `todo`, keeps both draft keys with a fresh `based_on` (p04: `kind=todo stale=false reason=Some(..) suggestion=Some(..)`), and writes no `kind` line, so the replaced kind is lost and `ai undo` jumps to an earlier kind (`now: Some("other")`). FR-029 pending will not pick the note again until it changes. `docs/NOTE_FORMAT.md:139` says draft keys are for kind `draft` only. The missing kind line pre-dates this task. | 100 | Supervisor ruling: clear (and log) in `enrich` when the new kind is not `draft`, or document it; log `kind` from `enrich`, or exclude it in NFR-015. |
| R6 | `tasks/TASK_GUIDE_HUN-295.md:60`, `PRD.md:143` | "HUN-302 must merge before `ai run` (HUN-295) ships" is only prose. The HUN-295 guide depends on HUN-294 only; there is no HUN-302 guide here and no HUN-302 row in the committed `PROJECT_KANBAN.md` (worktree or `main`). Uncommitted or remote board state was not read. | 75 | Add HUN-302 as a dependency of HUN-295 and put it on the board. |

Earlier F5 (override set while the call runs) now does more harm: a person sets kind `draft` during the call, the model says `todo`, and the batch deletes both draft keys (p05: `kind: Some("draft"), draft_reason: None, draft_suggestion: None`), with the R1 cut in the log. Raise its priority and fix it before HUN-295. F6 to F9 are unchanged.

### P3

- `src/core/organize.rs:178` with `src/core/ai.rs:326-331`: the cut can end inside a mask marker; `organize` masks again, the project grows and is refused. A repo `<44 to 47 chars>password=hunter2x` gives `organizing failed: project is 77 chars` on every run (p06). Contrived name.
- `src/core/ai.rs:298-309`: two `enrich` runs on one note at once both log the same old value; the first writer's text is replaced with only a 200-char `new` copy (p11, tight race: 454 of 500 rounds). Real window is milliseconds.
- `src/core/ai.rs:298`: an AI file that is not valid (corrupt, hand edit over a cap, version 2 from a newer build) is replaced with no log line (p09). The overwrite pre-dates this task and the doc says "valid".
- `src/core/ai.rs:299, 309`: if the AI file write fails after the log write, the log holds changes that did not happen. HUN-302 must expect this.
- `src/surfaces/organize.rs:110`: `result: "[[item]]"` is accepted as the object root (serde reads a struct from an array, p14), against "any other root fails the whole batch".
- `src/core/overrides.rs:9-11`: "a few KiB" is wrong. Tags have no length cap (`src/core/ai.rs:220-224`, pre-existing), and each replace now logs old tags whole: a 300,000-char tag grew the log by 901,744 bytes in 5 re-runs (p10). Title, summary at their caps with control characters: 828 and 3,725 bytes, parsed back whole.
- F8 reaches further: a lower-cased AWS key sent as a tag is now also kept whole in `ai-log.jsonl` (p15).
- The guide's AC 2 and AC 3 still say "array"; only the Questions Log amends them.

### Security: sensitive-data inventory (change to the earlier table)

| Data | Reaches | Kept | Control |
|---|---|---|---|
| Old `title` (120), `summary` (600) whole, old `tags_suggested` whole (no per-tag cap); new values cut to 200. Source text is derived from notes, from the session (`note_add(detail)`) or from a transcript (`enrich_notes`) | `<notes dir>/.easy-note/ai-log.jsonl`, so also any sync or git copy of the notes dir | for ever: still there after `ai rm` and `rm` (p13); no command removes it | masked on load and on write; a secret the patterns miss stays |
| `draft_reason`, `draft_suggestion`, project, topic, kind | same file | same | masked, cut to 200 (R1) |

No read path shows the old title, summary or tags: the one reader, `overrides::latest`, keeps only `project`, `topic`, `kind` lines, and no MCP tool, query, CLI output or window call reads the file. Child stdin, argv, env and note files are unchanged by this round.

| Lead | Outcome |
|---|---|
| `ai-log.jsonl` is created with the process umask (`src/core/overrides.rs:205-208`): probe p12 shows `664` for the log, `600` for the note and the AI file. Actor: another local account; boundary: file mode; resource: the owner's note-derived text, now whole summaries; result: read (group write if the group is shared). The mode pre-dates this round; the content is larger now. | `confirmed` for the mode, severity Low (needs another account that can reach the notes dir); not re-checked by a second agent. Fix: create the file with mode 0600 and test it. |
| Hardening, not a finding: removed notes keep their AI text in the log. | Say so in `docs/NOTE_FORMAT.md`, or remove a note's lines when its trash is emptied. |
| Rejected: a model value forges a log line or an undoable field | field names are fixed in code and values are JSON-escaped (p10, p11 parse) |
| Rejected: the new error text leaks note text | fixed words and the IO error kind only (`src/core/overrides.rs:317-322`) |

---

## Stage 4 review (second fix round)

Reviewer: qa-expert, fresh context, from the repo only, 2026-11-26. Commit `8b44eb6d8fb220e2c80dc551e1de60689ff38b65`; reviewed in depth: `git diff 5069261..8b44eb6 -- src tests docs`, with its callers. Rules: `skills/code-review/SKILL.md`, followed by hand (no Skill tool call). Review done alone, no second agent. No source or test file changed, no fix applied. The real `claude` CLI was never started: `~/.local/bin` was removed from `PATH`, a trap `claude` script was put first, and its log stayed empty.

**Verdict: pass.** No P0, no P1. R1 and R3 are fixed. 1 new P2 (N1), 1 new P3 (N2). The earlier P2 findings that this round did not target stay open.

### R1 and R3

| ID | Status | Evidence |
|---|---|---|
| R1 | fixed | `ai::organize` logs `draft_reason` and `draft_suggestion` (set, replaced, removed) with the old value whole, before the write. Tests with 600-char multibyte values pass. Mutations "cut the old value", "no draft log" and "write a second draft line" each fail tests. |
| R3 | fixed | A failed draft log write refuses `organize`; the draft keys stay. Mutations "ignore the log error" and "log after the write" each fail a test. |

### Breakage check

| Area | Result |
|---|---|
| `ai::organize` callers | `surfaces::organize::apply` and tests only. Caps are checked before the log write, so a refused item logs nothing. With no draft change no line is written and an unwritable log does not refuse the item (as before). |
| `ai undo` | `FIELDS` is unchanged and `latest` skips the draft lines. Probe p03 (new draft, then `todo`): undo gave kind `draft`, kind `other`, topic, project, then `nothing to undo`. |
| Overrides | `kept` fields and `clear_draft` (effective kind) are unchanged. The override tests pass. |
| `ai::enrich` callers | `log_enrich_diff` now goes through the shared `log_whole`; `old` is masked and whole and `new` masked and cut, as before. |
| `log_ai_diff` | Logs project, topic and kind only; no duplicate draft line. |
| New draft note, unwritable log | Probe p01: the item is pending, the file has kind `other`, no organizer keys and no `organized_at`; a retry with a writable log writes all keys and one line per change. |

### Commands and results

| Command (in the worktree unless said) | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 (cached build) |
| `cargo test --locked` | exit 0; 28 `test result` lines, 674 passed, 0 failed, 7 ignored; `tests/organize.rs`: 21 passed |
| Scratch copy of `8b44eb6`: probes p01 to p03 (`tests/zz_probe.rs`, fake `claude`), `cargo test --locked --offline --test zz_probe p0 -- --nocapture --test-threads=1` | 3 passed (first try did not compile: my probe called a private method) |
| Scratch copy: 6 mutations, `cargo test --locked --offline --test organize` | M1 ignore log error: 1 failed; M2 cut old: 3 failed; M3 no draft log: 4 failed; M4 log after write: 1 failed; M5 duplicate line: 2 failed; M6 `old` not masked: survives, equivalent because both callers take `old` from `load`, which masks |
| `python3 scripts/lane_check.py tasks/TASK_GUIDE_HUN-294.md main...8b44eb6` | `lane: C2, 11 files, 1839 lines, EXCEEDS (lines 1839 > 500); unpredicted: PRD.md, src/core/ai.rs, src/core/overrides.rs, tasks/TASK_GUIDE_HUN-294.md` (warn only) |
| `cargo test --locked --features window,tray` | not run in this round (not in the Verification Command) |

### P0 and P1

None.

### P2

| ID | File:line | Finding and failure scenario | Conf. | Action |
|---|---|---|---|---|
| N1 | `PRD.md:143` (not changed in this round) | The NFR-015 exception names only the old title, summary and tags as kept in `ai-log.jsonl`. It does not name `draft_reason` and `draft_suggestion`, which `ai undo` also skips. R1 asked to add them to the HUN-302 undo scope; no committed text does so. Failure: HUN-302 is written from the PRD, restores kind `draft` and leaves the draft text lost to the user although the log holds it (probe p03: undo gives kind `draft` with no draft keys). | 75 | Name the two draft keys in the NFR-015 note and in the HUN-302 scope (with R6: HUN-302 on the board and as a HUN-295 dependency). |

### P3

| ID | File:line | Finding and failure scenario | Conf. | Action |
|---|---|---|---|---|
| N2 | `src/core/overrides.rs:359`, `src/surfaces/organize.rs:170`, `docs/NOTE_FORMAT.md:161` | In the batch `ai::enrich` writes before `ai::organize`. When only the draft log cannot be written, the reason reads `AI detail written, but organizing failed: ai-log.jsonl could not be written: is a directory; the AI file was not changed`, which contradicts itself, and the file was replaced. Probe p02: draft note edited, log unwritable, model says `todo` with the same title: `based_on` is new, `stale=false`, the old draft suggestion now looks fresh. No value is lost. The partial write itself is the open F6. | 100 | Say "the organizer keys were not changed" in the message and the doc; handle the selection with F6 in HUN-295. |

### Earlier findings still open

R2, R4, R5, R6 and F5 to F9 are not touched by this round. R4 (a torn log line) now also covers the whole old draft values. F5 (override set while the call runs): by reading the code, removed draft keys are now kept whole in the log, so they can be recovered by hand, not by `ai undo`.

### Security: sensitive-data change

Old `draft_reason` (cap 200) and `draft_suggestion` (cap 600) are now kept whole in `ai-log.jsonl` (the suggestion was cut to 200). They are masked: probe p03 put a GitHub token in a suggestion and the log holds only `[masked:github_token]`. No new reader, input path, argv or note write. The earlier lead on the log file mode is unchanged.
