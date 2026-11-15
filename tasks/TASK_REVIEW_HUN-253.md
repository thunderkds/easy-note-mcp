# TASK_REVIEW — HUN-253: Note format and store

> Sibling of `tasks/TASK_GUIDE_HUN-253.md`. Everything here is **filled by the reviewer at Stage
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
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/test_format.py`, `tests/test_store.py`, fixtures `tests/fixtures/notes/2026/11/*.md`. AC1 `test_round_trip_of_tool_written_fixture_is_byte_identical`, `test_added_note_round_trips_byte_identical`; AC2 `test_id_is_local_time_plus_sha256_of_ns_hostname_text`, `test_two_writes_same_second_same_text_get_clash_suffix`, `test_month_folder_uses_local_date_not_utc`; AC3 `test_write_never_replaces_existing_file[True/False]`; AC4 `test_killed_write_leaves_no_md_file` (child process os._exit inside os.link), `test_failing_replace_on_no_link_filesystem_leaves_no_md`, `test_failing_replace_during_update_keeps_old_bytes`; AC5 `test_rm_moves_to_trash_and_restore_brings_back_same_bytes`, `test_restore_never_replaces_a_note_at_the_target`, `test_empty_trash_deletes_only_files_older_than_cutoff`, `test_rm_stamps_trash_time_so_an_old_note_is_not_emptied_at_once`; AC6 `test_bad_files_are_skipped_with_path_and_left_unchanged`, `test_bad_fixture_files_raise_format_error`, `test_invalid_front_matter_raises_format_error`, `test_unreadable_file_is_skipped_not_fatal`; AC7 `test_update_keeps_unknown_keys_and_sets_updated`, `test_unknown_keys_render_verbatim_after_known_keys`; AC8 `test_first_add_creates_default_dir_once`, `test_env_dir_beats_config_and_config_beats_default`, `test_config_round_trips_awkward_paths`, `test_broken_config_raises_clear_error`; AC9 `test_title_is_first_line_cut_to_120_chars`, `test_long_first_line_is_kept_whole_in_the_file`, `test_check_text_refuses_over_20000_chars_with_clear_error`, `test_oversize_text_is_refused_before_any_write`; AC10 `test_note_format_doc_is_final_and_its_example_round_trips`, `test_note_format_doc_lists_every_key_the_code_knows`. Run: `uv run pytest tests/test_format.py tests/test_store.py -q` → 72 passed in 0.19s (2026-11-15T04:41:30Z). Filled by backend-developer. |
| Verification command run | ☑ pass | 2026-11-15T04:41:27Z<br>HEAD 9b1659c<br>$ uv run --locked pytest -q<br>........................................................................ [ 72%]<br>...........................                                              [100%]<br>99 passed in 2.37s<br>$ uvx ruff check --isolated src tests<br>PLW1510 `subprocess.run` without explicit `check` argument<br>  --> tests/test_T001_probes.py:24:12<br>PLW1510 `subprocess.run` without explicit `check` argument<br>  --> tests/test_T001_probes.py:73:13<br>PLW1510 `subprocess.run` without explicit `check` argument<br>  --> tests/test_T001_probes.py:78:17<br>Found 3 errors.<br>(the 3 PLW1510 findings are in `tests/test_T001_probes.py`, unchanged from `main` (`git diff main -- tests/test_T001_probes.py` is empty); `uvx ruff check --isolated src tests/test_format.py tests/test_store.py` → All checks passed!) Filled by backend-developer. |
| Negative cases hold | ☑ pass | 2026-11-15T04:41:17Z<br>## M1: store._write_new uses os.replace (overwrites) instead of os.link<br> src/easy_note/core/store.py \| 2 +-<br> 1 file changed, 1 insertion(+), 1 deletion(-)<br><br>tests/test_store.py:124: AssertionError<br>=========================== short test summary info ============================<br>FAILED tests/test_store.py::test_two_writes_same_second_same_text_get_clash_suffix<br>FAILED tests/test_store.py::test_write_never_replaces_existing_file[True] - A...<br>FAILED tests/test_store.py::test_write_never_replaces_existing_file[False] - ...<br>FAILED tests/test_store.py::test_killed_write_leaves_no_md_file - assert 0 == 9<br>4 failed, 26 passed in 0.24s<br>## M2: format._value writes strings unquoted<br> src/easy_note/core/format.py \| 2 +-<br> 1 file changed, 1 insertion(+), 1 deletion(-)<br>FAILED tests/test_format.py::test_render_quotes_strings_so_yaml_keeps_them_strings[del\x7f]<br>FAILED tests/test_format.py::test_render_quotes_strings_so_yaml_keeps_them_strings[]<br>FAILED tests/test_format.py::test_note_format_doc_is_final_and_its_example_round_trips<br>19 failed, 23 passed in 0.18s<br>## after revert<br>Both mutations reverted (empty `git diff --stat` after). Filled by backend-developer. |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Stage 4 code-review (Supervisor, 2026-11-15) over `main...15ad1ce`: `src/easy_note/core/{format,store,config}.py`, `tests/test_format.py`, `tests/test_store.py`, fixtures, `docs/NOTE_FORMAT.md`. P0 0, P1 0, P2 1 (`docs/ddr/0002-file-format-as-public-contract.md:23` still says draft), P3 2 (`update` read-then-replace can drop a concurrent sync edit; 3 pre-existing PLW1510 in `tests/test_T001_probes.py`). security-review: pass, P0 0 / P1 0 (SafeLoader only, quoted scalars, ID fullmatch blocks traversal, 0600 files). lane_check: `C2, 12 files, 1089 lines, EXCEEDS (lines 1089 > 500)` — tests and fixtures; C2 kept. Entry point `easy_note.core.store.add` found. Startup reads: all 3 present. AC 9 reading confirmed by Supervisor: title is the first line cut to 120 on display; the file keeps the typed words (PRD FR-002). |
| Full smoke suite still green (no regression) | ☑ pass | Supervisor re-run 2026-11-15: `.venv/bin/pytest -q tests` -> `99 passed in 2.05s` (27 pre-existing from main + 72 new); no regression. |
| **Companion score (easy-verifier)** | ☑ fail | overall 63 (floor missed, < 80; architecture 100, blast-radius 65, code-quality 67, requirement-fidelity 67, supply-chain 55, test-strategy 60, security 30), change_risk 100 pass (feature, confirmed; 4 test pairs confirmed), scope changes, ref `main...HEAD` at 15ad1ce, abstentions: solution-fit (no_static_rule), reliability (insufficient_measured_weight), api, data, operations, infrastructure (below_coverage_floor), gates: code-quality and solution-fit left abstained, no findings sent. Security 30 is `redaction_hits_observed` 148: hex file hashes in `.claude/harness-lock.json` (kit file read as repo context, not this change), none a secret. report: `/home/hungnguyenhuu/.easy-verifier-reports/hun-253-store/evidence-report-changes-20261115T045448-593802Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | Pure backend task; no UI component (guide's UI section is N/A). |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | No UI component. |
| **UI: Responsiveness at target viewports** | ☑ N/A | No UI component. |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (captured by backend-developer at HEAD b60c1d1, before any new file existed):

```text
2026-11-15T04:32:15Z
$ uv run pytest tests/test_format.py tests/test_store.py -q
Using CPython 3.13.2
Creating virtual environment at: .venv
   Building easy-note-mcp @ file:///home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/worktrees/hun-253-store
      Built easy-note-mcp @ file:///home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/worktrees/hun-253-store
Installed 36 packages in 23ms
ERROR: file or directory not found: tests/test_format.py


no tests ran in 0.00s
exit=4
$ uv run python -c "import easy_note.core.store"
Traceback (most recent call last):
  File "<string>", line 1, in <module>
    import easy_note.core.store
ModuleNotFoundError: No module named 'easy_note.core.store'
exit=1
```

**AFTER** (backend-developer, HEAD 9b1659c):

```text
2026-11-15T04:41:30Z
$ uv run pytest tests/test_format.py tests/test_store.py -q
........................................................................ [100%]
72 passed in 0.19s
exit=0
$ uv run python -c "import easy_note.core.store"
exit=0
```

**DELTA**: core code can now write, read, edit, trash and restore one note in the final `easy-note: 1` format, with no overwrite and no partial file, where before `easy_note.core.store` did not exist.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-253.jsonl`, never the
implementing agent alone]
