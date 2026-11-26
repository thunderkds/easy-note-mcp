# TASK_REVIEW — HUN-302: Undo for the AI title, summary, tags and draft keys

> Sibling of `tasks/TASK_GUIDE_HUN-302.md`. Everything here is **filled by the reviewer at Stage
> 4/5** — it is deliberately NOT in the guide, because the implementing agent re-reads the guide on
> every turn and never fills these two sections.
>
> Consumers resolve each section **guide first, this file second** (`.claude/hooks/lib/guide_sections.py`):
> a legacy guide that still carries these sections inline keeps working unchanged, and a stray
> review file can never override an inline section.

---

## Auto decisions

Mode: auto
1. HUN-302 depends on HUN-294, which the board cache lists as Ready for Review; the dependency is met, because the hook closed HUN-294 and its code is in `main` — FACT: tasks/TASK_REVIEW_HUN-294.md:62 "Closed: row moved to Done 2026-11-26 — auto verify pass"
2. Requirement Fidelity Gate signed `auto (Supervisor)` — RULE: docs/claude-md/tracks.md § Every user stop and its auto rule
3. The log lines of one applied item share an optional `op` id; lines without it stay single undo steps; a write that changes the AI file logs an `ai_file` line — DEBATED: `op` id (Advocate 1: `op` id, one core apply, a marker line on creation; Advocate 2: `op` id, one core entry over `enrich` and `organize`, a marker line on creation) ⚠ decided for you — please review
4. After undo, a file the AI created is removed, and a replaced file comes back whole with its old `based_on`, `at`, `model` and `organized_at` — DEBATED: restore the earlier state (Advocate 1: delete or restore the whole file; Advocate 2: the same, plus a hold so a later run does not redo it) ⚠ decided for you — please review
5. The hold of Advocate 2 is not built here: `ai run` is HUN-295, and without a hold its next run can redo an undone change — RULE: tasks/TASK_GUIDE_HUN-302.md § Requirement
6. A failed undo stays the latest step: the replaced AI file is logged first, then the file is written, then the step is marked undone (review F1) — RULE: memory/decisions.md § 2026-11-26 — HUN-294 batch organizer engine; the old AI text is logged whole until undo exists; merged over a failing Companion score
7. `PRD.md` FR-030 and `PROJECT_SPEC.md` now say that `note_enrich` also writes its log lines, because a creation must be logged — RULE: tasks/TASK_GUIDE_HUN-302.md § Acceptance Criteria
8. The nine P3 findings of the second review stay as follow-ups — RULE: skills/code-review/SKILL.md § Phase 4 — Apply Safe Fixes
9. The Companion score gate `solution-fit` is left abstained: no kit-held evidence rates the design at its listed refs — RULE: skills/score/SKILL.md § The session (MCP): 1 to 3 rounds
Checks: tests `cargo test --locked` 738 passed 0 failed 7 ignored and `cargo test --locked --features window,tray --no-fail-fast` 747 passed 0 failed 7 ignored (Supervisor, before commit d9f30e1; the reviewer saw one intermittent lock test fail with the features, also on `main`); reviews: first reviewer fail (1 P1, 1 P2, 9 P3) at 7bffd27, second reviewer pass (0 P0, 0 P1, 0 P2, 9 P3) at d9f30e1, Supervisor `code-review` and `security-review` inline at d9f30e1 (no High or Medium security finding), Companion score fail (79); real run: two `ai enrich` runs with a stub `claude`, then `ai undo` twice, printed `✓ undone: AI detail of <id> is back (title, summary, tags_suggested, kind)` and `✓ undone: AI detail of <id> removed (the AI had created it)` (auto-checked, not /verify)
Memory: memory/decisions.md — HUN-302 one `ai undo` takes back one applied item; memory/learnings.md — an open-ended reviewer brief costs 30 to 40 minutes; memory/MEMORY.md — two index lines (written on main, not committed)
Not done: merge (admin: after the auto verify pass; the failing Companion score row blocks it, so it is yours); the hold and the write lock for HUN-295 (decision 5, F5); the P3 findings; auto verify: running (T182)
Paused: auto-mode check failed — no `Memory:` line in the report — run the memory pass before you stop and record it there (each entry written, or `Memory: nothing new`); Risk Medium: decision 1 (`FACT`) is a choice — its FACT has no Citation (`path:line "quoted text"`) — so it must be `DEBATED` with exactly 2 Advocates named [report ddd4faae0e6d]

## Review findings (Stage 4: code review + security review)

Reviewer: qa-expert, fresh context, commit `7bffd274aedf5082fde20aff06f4470fca74dd63`, diff `main...7bffd27`, 2026-11-26.

**Verdict: fail** — P0: 0, P1: 1, P2: 1, P3: 9. F1 breaks AC 4.

Reproductions are probe tests run in a copy of the worktree outside it (removed afterwards), on temp notes dirs. The I/O failure in F1 and F6 was made with `chmod 555` on `.easy-note/ai/`.

### F1 — P1, confidence 95 — a failed undo is already marked done, so the retry takes back the wrong step

- Where: `src/core/overrides.rs:513-514` (`append_all(...)` writes the undo lines, then `ai::apply_restore(...)` runs).
- Failure: the restore fails (write, rename or remove). The log already says the step is undone. The CLI prints only `easy-note: Permission denied (os error 13)`. The AI file still holds the new values. A retry takes the next older step.
- Case 1 (the retry removes a person's override; against AC 4 and "never removes a person's override when the latest change was an AI change"): `ai set <id> topic mine`, then one applied item that creates the AI file. The first undo fails. The retry prints `✓ undone: topic of 20261126-225108-495f is now no override`, and the state is `C file: title="one" summary="first" tags=["one"] kind="bug" project="p1" overrides={}`.
- Case 2 (an earlier AI state is skipped): items S1 then S2. The undo of S2 fails. The retry prints `✓ undone: AI detail of <id> removed (the AI had created it)`. Expected S1; the file is gone, and the third undo says `nothing to undo`. S1 can only be dug out of the log by hand.
- Regression: on `main`, `ai::restore_field(...)?` ran before the undo line was logged, so a failed undo could be retried.
- No test covers it: the order swap (mutant M9) leaves every test that was run green.
- Suggested action: when `apply_restore` fails, the step must stay the latest one. For example, append the original step lines again on that failure (this keeps "log the replaced file first"). Moving the log after the restore also fixes both cases (checked with M9 applied: the retry gives S1, and the override stays), but then a failed log write after a good restore loses the replaced text. Add a test for each case. Say in the error that nothing was undone.

### F2 — P2, confidence 99 — a torn log line makes undo restore half an item and report it as whole

- Where: `src/core/overrides.rs:478` (`step.iter().find(|l| l.field == AI_FILE)` takes any `ai_file` line, not only the one that starts the step) and `src/core/overrides.rs:229-242` (`append_all` does not start a new line when the file does not end in a newline).
- Failure: an earlier append was cut short (disk full, crash, sync). The next append is glued to it, so the first line of the next item (the `ai_file` line with the state before the item) is unreadable. `find` then takes the second `ai_file` line (after `enrich`, before `organize`) and marks all lines of the `op` undone.
- Observed: the undo returns fields `["title", "summary", "tags_suggested", "kind", "project"]`, but the file keeps `title="two" summary="second" tags=["two"] kind="idea"`; only `project` went back. The next undo removes the file, so S1 is never restored. For an item that created the file, the file is not removed.
- The test `hun302_an_item_whose_ai_file_line_is_unreadable_...` removes every `ai_file` line, so it never sees this.
- Suggested action: use the `ai_file` line only when it is the first line of the step (`step[0]`), else use the per-key path; write a leading newline when the log does not end in one; add a test that tears only the first line of a two-write item.

### P3

| ID | Conf. | file:line | Concrete failure | Suggested action |
|----|-------|-----------|------------------|------------------|
| F3 | 100 | `src/core/overrides.rs:479-512` | Per-key path (a line from HUN-294, or an unreadable `ai_file` line): the undo replaces the current summary, but no `ai_file` undo line is written and `new` was cut to 200 chars. A 600-char replaced summary is nowhere in the log afterwards (probe F, seeded with a HUN-294-shaped line). The doc comment "no AI text is lost" and `docs/NOTE_FORMAT.md` "the `ai_file` undo line holds the replaced file whole" are not true on this path. | Log an `ai_file` undo line with the replaced file on this path too, or narrow both sentences |
| F4 | 100 | `src/core/overrides.rs:229-242`, `:336` | Security, Low. Whole AI files now stay in `ai-log.jsonl`. After `ai rm` plus `rm`, the AI text is still there (probe G). The log is created with the umask mode (observed `-rw-rw-r--`) while AI files, notes and overrides are `-rw-------`. The mode and the HUN-294 lines predate this task; the amount of text grows with it. | Create the log with mode 0600; document that the log keeps old AI text, or plan a purge |
| F5 | 60 (code reading) | `src/core/overrides.rs:476-514`, `src/core/ai.rs:410-499` | No lock between the plan and the write. An AI write that lands in between is overwritten by the restore, and the log holds its new values only cut. Separately observed: two parallel `overrides::undo` calls (two threads) both succeed and take back one step (200 of 200 runs, nothing lost). It matters once `ai run` can run from the tray. | Follow-up for HUN-295: one lock per notes dir for AI writes and undo |
| F6 | 100 | `src/core/ai.rs:335-336`, `:382-383` | The log is written, then the AI write fails: the step stays in the log. The next undo prints `✓ undone: AI detail of <id> is back (title, summary, tags_suggested, kind)` and the file does not change (probe K). | Accept and document, or log a cancelling line when the write fails |
| F7 | 90 (code reading) | `src/core/overrides.rs:393-420` | AI files created before this build have no creation line. If a person's override is older than such a creation, `ai undo` removes the override although the latest change was an AI change. `docs/NOTE_FORMAT.md` says the creation cannot be undone, not that the override goes instead. | Add that sentence to the docs and tell the owner |
| F8 | 100 (needs a hand-edited or foreign log) | `src/core/overrides.rs:434-437` | A `who: "person"` line with field `title` (skipped on `main`) now reaches `undo_person`, which writes `title` into `ai-overrides.json`. A later `ai set` fails: `ai-overrides.json has invalid entries (<id>.title)` (probe E). | Refuse or skip a person line whose field is not in `FIELDS` |
| F9 | 100, mechanism predates the diff | `src/core/overrides.rs:394-396` | A line torn inside a multi-byte character makes `read_to_string` fail, so undo says `nothing to undo` although steps exist (probe L); the whole log is read, so this hits every note. Whole AI files put more non-ASCII bytes in the log. | Read bytes and decode line by line, skipping a bad line |
| F10 | 100 | `PROJECT_SPEC.md:188`, `src/surfaces/mcp.rs:3-4`, `:484` | They still say `note_enrich` writes only AI files ("one AI file only"); each call now also appends to `ai-log.jsonl`. Of the requirement docs, only `PRD.md` FR-030 was updated. | Update the three sentences; tell the owner, because the tool contract is frozen |
| F11 | 100 | `tests/overrides.rs` | Mutants survive: M8 (the logged earlier file is not masked on restore, a security negative), M9 (the F1 order), M11 (the per-key path applies the oldest value last). The new tests compare note bytes around one item and one undo only (AC 5). | Add tests for the three mutants and for repeated undo with the note bytes compared |

### Acceptance Criteria

| AC | Verdict | Evidence |
|----|---------|----------|
| 1 old title, summary, tags restored whole | pass on the normal path; breaks under F1 and F2 | `hun302_one_undo_returns_the_whole_earlier_ai_file_after_an_item`, `a_stale_file_kind_change_is_logged_and_undone`; random history probe: 2390 undos, 0 mismatches. Lines logged before this task go back one key at a time |
| 2 cleared draft reason and suggestion restored | pass | `hun302_one_undo_returns_replaced_and_cleared_draft_keys`, `a_cleared_600_char_draft_suggestion_is_logged_whole_once`; mutant M3 killed |
| 3 one item taken back together, in one call | pass on the normal path; broken by F2 | mutants M1 and M4 killed |
| 4 a creating item has a log line, so undo never removes a person's override in its place | **fail (F1)** | the line exists (M2 killed) and the normal path keeps overrides; after a failed undo the retry removes the person's override instead |
| 5 raw note files byte-identical | pass | 300 random rounds of items, enrich, organize, set and undo: note bytes identical; real CLI run of `ai set` and `ai undo`: same sha256 before and after |
| Success criterion 1 (stale file replaced by a batch) | pass | `a_stale_file_kind_change_is_logged_and_undone` |
| Success criterion 2 (new note, overrides, batch) | pass | `hun302_undo_of_a_new_ai_file_removes_it_and_keeps_the_persons_overrides` |

### Security review (Risk Medium)

Result: **pass** — no High or Medium security finding; one Low (F4) and one test gap (F11, M8).

- Secrets: a planted token in AI file fields, then an item and an undo: not in the CLI output, not in the AI file, not in a log line that easy-note wrote (probe H).
- Untrusted log content: a planted earlier file with a token and the id `../../../x` was restored masked, under the canonical id; no stray file was written. The code also checks version, caps, kind and tags. `ai undo ../../etc` is refused: `easy-note: not a note id: ../../etc`.
- Terminal output: field names come from a fixed list and pass `safe()`.
- No new dependency, process start or network call in the diff. No note file is opened for writing.

## Second review (commit d9f30e1)

Reviewer: qa-expert, fresh context, commit `d9f30e1cdc6793d9f780fdbcd468e62b50451b06`, diff `main...d9f30e1`, 2026-11-26 to 2026-11-27. Manual code and security review; the `code-review`, `security-review` and `verify` skills were not invoked.

**Verdict: pass** — P0: 0, P1: 0, P2: 0, P3: 9 (3 open or partly open from the first review, 6 new).

### First-review findings

| ID | Status at d9f30e1 | Evidence |
|----|-------------------|----------|
| F1 (P1) | **fixed** | `src/core/overrides.rs:525-551`. Real CLI with `.easy-note/ai/` read-only: `easy-note: nothing undone: Permission denied (os error 13)`; the retry removed the AI file and kept the override. Model probe: 425 failed undos left the step in place. Failed second append (`RLIMIT_FSIZE`, 163 + 829 cut points): the retry was correct at all but one cut point per case (S6). M9 killed |
| F2 (P2) | **fixed** for the reported trigger | `:246-255` starts a new line, `:501` takes only the first line. G6 killed. What is left: S1 (damaged first line) and S2 (the newline fix is not pinned by a test) |
| F3 | **fixed** | `:525-529` logs `ai_replaced` on both paths; real run on a log written by a `main` build: 3 `ai_replaced` lines after 3 per-key undos. G2 killed |
| F4 | **partly open** (P3, Low) | A new log is `0600` (observed; G3 killed) and `docs/NOTE_FORMAT.md` says the log keeps old AI text. A log made by the `main` build stays `0664` after d9f30e1 writes to it (observed) |
| F5 | **open** (P3, confidence 60, code reading, not re-run) | No lock between plan and write (`:499-531`). Not in the fix commit. Follow-up for HUN-295 |
| F6 | **fixed** (docs) | `docs/NOTE_FORMAT.md:178`. Wording limit, code reading: "that step changes nothing" does not hold for a batch item whose second write fails, because the step also holds the first write and undo takes it back |
| F7 | **fixed** (docs) | `docs/NOTE_FORMAT.md:176-178`; the documented behavior was seen on the `main`-built log (the 4th undo removed the person's override) |
| F8 | **fixed** | `:403-410`; G1 killed |
| F9 | **fixed** | `:420-423` reads bytes line by line; G5 killed |
| F10 | **partly open** (P3, confidence 100) | `PROJECT_SPEC.md:188` and the header of `src/surfaces/mcp.rs` are updated. `src/surfaces/mcp.rs:485` (`INSTRUCTIONS`, the line the finding cited as `:484`: "note_enrich writes only AI detail files"), `:452` (tool description: "Writes only the note's AI detail file") and `PRD.md:80` (FR-012) are not. Update them, or record that the frozen tool contract keeps them, and tell the owner |
| F11 | **fixed** | M8, M9 and M11 are now killed. The suggested test with note bytes compared over repeated undo was not added; the model probe below covers AC 5 for this review |

### New findings

| ID | P | Conf. | file:line | Concrete failure | Suggested action |
|----|---|-------|-----------|------------------|------------------|
| S1 | P3 | 100 (needs a log damaged from outside, e.g. hand edit or sync conflict; a cut-short easy-note append no longer causes it) | `src/core/overrides.rs:501` | The first line of an item is unreadable and the next readable line is the second write's `ai_file` line, so it is taken as the item start. Item that created the file: undo returns `Restored ["kind", "project"]`, the AI file stays, and the next undo removes the person's `topic` override while the file is still there. Replacement whose first write changed no key: `model` stays `m2`. `docs/NOTE_FORMAT.md:172-173` ("put back key by key") is not true here | Mark the first line of an item and use the whole-file path only for a marked line; or narrow the sentence |
| S2 | P3 | 100 | `tests/overrides.rs:829` | Mutants G4 (newline fix removed), G7 (replaced file logged after the restore) and G8 (failed second append ignored) survive. The torn-line test passes without the fix because both items have the same model and second, so the per-key result equals the earlier file | Give the second item `model = "m2"` (checked: passes on d9f30e1, fails on G4); add tests for the log order and for the failed second append |
| S3 | P3 | 100 (Security, Low; needs a token already in the log) | `src/core/overrides.rs:539` | The undo line takes `old: l.new.clone()` unmasked. A token planted in `new` of an AI line was copied into the undo line. On `main` this value went through `log_line`, which masks | Mask it |
| S4 | P3 | 100 (needs a hand-edited or synced AI file) | `src/core/ai.rs:438-443` | A file with `password=abcd` and a 600-char summary loads, but its masked copy is over the cap and is what gets logged. After the next `enrich`, three undos in a row refused with `the logged earlier AI file is not valid; nothing undone`; the older override step cannot be reached. `ai rm` unblocks it (next undo: `Gone`), and the message does not say so | Name `ai rm` in the message, or let the step be dropped |
| S5 | P3 | 100 | `PRD.md:114` | FR-047 still says "Each applied item is one `ai-log.jsonl` line". An item is now two or more lines that share `op`. AC 3 traces to FR-047 | Update the sentence |
| S6 | P3 | 100 (needs a write cut at exactly one byte) | `src/core/overrides.rs:421`, `:546` | The second append loses only its last newline: the CLI says `...could not be written: file too large; run undo again`, but the lines parse and the step is already done, so the retry takes the next step. In the creation case that removes the person's override. 1 of 163 and 1 of 829 cut points | In `latest`, ignore a last segment that has no newline (the retry's append then ends it) |

Not filed (code reading, not tested, pattern older than this task): the log is not fsynced and the AI file is, so after a power loss the file could survive without its log lines.

### Acceptance Criteria

| AC | Verdict | Evidence |
|----|---------|----------|
| 1 old title, summary, tags restored whole | pass | `hun302_one_undo_returns_the_whole_earlier_ai_file_after_an_item`; model probe: every successful undo equalled the earlier file; a `main`-built log (legacy lines) came back key by key with the 600-char summary whole |
| 2 cleared draft reason and suggestion restored | pass | `hun302_one_undo_returns_replaced_and_cleared_draft_keys`; draft set and clear are in the model probe |
| 3 one item taken back together, in one call | pass (damaged-log exception: S1) | M1 killed (8 red); `apply_item` undone in one call in the model probe |
| 4 a creating item has a log line; undo never removes a person's override in its place | pass | M2 killed (9 red); F1 fixed; no AI undo changed overrides in the model probe. Exceptions need damage or an exact cut (S1, S6) or are documented legacy logs (F7) |
| 5 raw note files byte-identical | pass | note bytes compared after each of 18,000 model-probe operations; real runs: sha256 of the note files unchanged |
| Success criterion 1 (stale file replaced by a batch) | pass | `a_stale_file_kind_change_is_logged_and_undone`, `hun302_one_undo_returns_the_whole_earlier_ai_file_after_an_item` |
| Success criterion 2 (new note, overrides, batch) | pass | `hun302_undo_of_a_new_ai_file_removes_it_and_keeps_the_persons_overrides` |

Model probe (reviewer test in a copy outside the worktree): 600 seeded histories of 30 random operations (`apply_item`, `enrich`, `organize`, `set`, `ai rm`, `undo`, with `ai/` read-only at random; half with an Obsidian vault) against a stack model. Output: `histories ok: 4740 undos, 425 failed-then-kept undos, 783 failed AI writes, 982 gone`.

### Security review (Risk Medium)

Result: **pass** — no High or Medium security finding. Low: S3, and what is left of F4.

- A logged earlier AI file comes back masked (M8 killed); a foreign `person` line for a non-override key is skipped (G1 killed).
- New log mode is `0600`.
- `Cargo.toml`, `Cargo.lock`, `core/guard.rs`, `core/store.rs` and `core/format.rs` are not in the diff; no new process start or network call seen in the source diff. No note file is opened for writing.
- Not re-run: the first review's path-traversal and terminal-output probes.

### Test failures seen outside the diff

- `hun290_ac4_the_lock_is_a_file_in_the_notes_dir_and_only_one_owner_holds_it` (`tests/window_api.rs:3082`, `free again`) is intermittent and also fails on `main` (filter `hun290_ac4`: main build 2 of 150, d9f30e1 7 of 150; the difference in rate was not examined); the diff cannot touch it, because that test, `tests/window_api.rs` and `src/surfaces/window/single.rs` are unchanged and the test calls only `single::acquire`.
- `a_bad_schedule_value_names_the_key_not_the_value` (`tests/enrich.rs`) failed once during the M9 run; unchanged by the diff; cause not established; not checked on `main`.
- The one `dashboard` test failure in the first feature-build run came from the reviewer's own parallel default build in the same target directory; the test passed in the next run.

### Not checked

- A fully green `--features window,tray` run: two runs stopped at the lock test above, so doc-tests did not run with features.
- F5 with real concurrent processes.
- Mutants were run against `--lib` and the tests `overrides organize ai mcp enrich`, not `leak` or `cli`.

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | At `d9f30e1`: `tests/overrides.rs` `test result: ok. 32 passed; 0 failed; 0 ignored`. New in the fix commit: `hun302_a_failed_undo_of_a_creation_leaves_the_step_and_the_override ... ok`, `hun302_a_failed_restore_does_not_skip_an_earlier_ai_state ... ok`, `hun302_a_torn_line_does_not_take_the_next_items_first_line_with_it ... ok`, `hun302_without_its_first_line_an_item_is_undone_key_by_key_and_the_replaced_file_is_kept ... ok`, `hun302_two_lines_of_one_key_in_an_item_give_back_the_oldest_value ... ok`, `hun302_a_logged_earlier_file_is_masked_when_it_comes_back ... ok`, `hun302_the_log_is_private_and_a_foreign_person_line_is_skipped ... ok`. From the first commit: `hun302_one_undo_returns_the_whole_earlier_ai_file_after_an_item ... ok`, `hun302_one_undo_returns_replaced_and_cleared_draft_keys ... ok`, `hun302_undo_of_a_new_ai_file_removes_it_and_keeps_the_persons_overrides ... ok`, and 6 more `hun302_*` ok (16 in all). `tests/organize.rs` 21 passed. Test gap: S2 |
| Verification command run | ☑ pass | Reviewer, at `d9f30e1` in the worktree (its own `target/`; temp HOME and XDG_CONFIG_HOME; display and D-Bus variables unset; failing `claude` and `gsettings` stubs first on PATH, never called): `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, each followed by an echo. Run 1: `FMT exit 0`, `CLIPPY exit 0`, `VERIFY exit 101`, 737 passed, 1 failed, 7 ignored (the failure: `hun290_ac4_the_lock_is_a_file_in_the_notes_dir_and_only_one_owner_holds_it ... FAILED`, intermittent, see Second review). Run 2, same commit: `FMT exit 0`, `CLIPPY exit 0`, `VERIFY exit 0`, 27 result lines, 738 passed, 0 failed, 7 ignored |
| Negative cases hold | ☑ pass | At `d9f30e1`. Real binaries on temp dirs: `easy-note: nothing undone: Permission denied (os error 13)`, retry `✓ undone: AI detail of <id> removed (the AI had created it)`, override kept; failed second append at every cut point: creation `cut points 163 bad 1`, replace `cut points 829 bad 1` (both the last-newline cut, S6). Model probe: `histories ok: 4740 undos, 425 failed-then-kept undos, 783 failed AI writes, 982 gone`. Mutants in a copy outside the worktree, one at a time, changed-line count non-zero for each: M8 no mask on restore killed (1 red); M9 done before restore killed (3 red, one of them the unrelated `a_bad_schedule_value...`); M11 per-key order killed (2); M1 two op ids killed (8); M2 no creation line killed (9); G1 foreign person line replayed killed (1); G2 no `ai_replaced` line killed (1); G3 no 0600 killed (1); G5 log read as one string killed (1); G6 any `ai_file` line killed (1); G4 no newline after a torn line SURVIVED; G7 replaced file logged after restore SURVIVED; G8 failed done append ignored SURVIVED. Open P3 negatives: S1, S3, S4, S6 |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed `main...d9f30e1` (13 files), the fix commit `7bffd27..d9f30e1` in detail: `src/core/overrides.rs`, `src/core/ai.rs`, `src/surfaces/{mcp,organize}.rs`, `src/surfaces/cli/write.rs`, `docs/NOTE_FORMAT.md`, `PRD.md`, `PROJECT_SPEC.md`, the new tests; plus `core/fsutil.rs`, `core/ai_view.rs`, `core/guard.rs` (read in part) and the callers of the AI writers. Not reviewed: window, tray, install and query code (unchanged) |
| Full smoke suite still green (no regression) | ☑ pass | At `d9f30e1`, default build (run 2): `VERIFY exit 0`, 738 passed, 0 failed, 7 ignored. `cargo clippy --all-targets --locked --features window,tray -- -D warnings`: `CLIPPY-F exit 0`. `cargo test --locked --features window,tray` was NOT fully green: the last two runs each printed `FEATURES exit 101`, 746 passed, 1 failed, 7 ignored, the one failure being `hun290_ac4_the_lock_is_a_file_in_the_notes_dir_and_only_one_owner_holds_it`, which also fails on `main` and is outside the diff; doc-tests did not run with features |
| **Companion score (easy-verifier)** | ☑ fail | overall 79 (floor missed: 79 < 80), change_risk 50, fail (feature, confirmed; failed rule `requirements_removed` = 1, computed from `PRD.md`; this diff rewrites the FR-030 and NFR-015 lines, and which change the rule counts was not established), scope changes, ref `main...HEAD` (HEAD `d9f30e1`), ratings: architecture 100, supply-chain 100, code-quality 87, test-strategy 72, requirement-fidelity 67, blast-radius 65, security 59 (`redaction_hits_observed` 4, not traced to lines), abstentions: solution-fit, reliability, api, data, operations, infrastructure, gates: solution-fit asked and left abstained, test pairs confirmed: 6, divergence: none returned (the two code-review findings sent first were refused: their refs are not in the dimension packs), report: `/home/hungnguyenhuu/.easy-verifier-reports/easy-note-mcp-HUN-302/evidence-report-changes-20261126T170939-996435Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☑ N/A | No UI component: core library, log file and command-line output only |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ pass / ☐ fail / ☑ N/A | No UI component |
| **UI: Responsiveness at target viewports** | ☐ pass / ☐ fail / ☑ N/A | No UI component |

---

## Demonstration

**BEFORE** (Supervisor, 2026-11-26T15:24:00Z, before the first implementation commit `7bffd27`): temp HOME and notes
dir, a stub `claude`, two `ai enrich` runs, then `ai undo` twice:

```
after run 2
  AI file: "title":"second title" "kind":"todo" "summary":"second summary"
$ easy-note ai undo 20261126-222400-918a
easy-note: nothing to undo
  exit 1
  AI file: "title":"second title" "kind":"todo" "summary":"second summary"
```

**AFTER** (Supervisor, 2026-11-26T16:21:57Z, the source of `d9f30e1`, same script):

```
$ easy-note ai undo 20261126-232156-df25
✓ undone: AI detail of 20261126-232156-df25 is back (title, summary, tags_suggested, kind)
  exit 0
  AI file: "title":"first title" "kind":"bug" "summary":"first summary"
$ easy-note ai undo 20261126-232156-df25
✓ undone: AI detail of 20261126-232156-df25 removed (the AI had created it)
  exit 0
  AI file:
```

**DELTA**: A person can take back everything the AI changed on a note, one applied item at a time, including the
title, summary, tags and draft keys.

**WITNESS**: the Supervisor ran both captures; the qa-expert reviewer ran its own real runs and probes on 2026-11-26
(`memory/event-trace/HUN-302.jsonl`).

---

## Supervisor check (2026-11-27, `[admin]`)

- `code-review` on `main...d9f30e1`: the second review's findings stand. P0: 0, P1: 0, so no fix is applied and the
  reviewed commit stays the code to merge. Entry point `ai undo` is in `src/surfaces/cli.rs`. Stale wording that the
  diff leaves (`PRD.md:80`, FR-047 at `PRD.md:114`, `src/surfaces/mcp.rs` description and instructions strings): P2 by
  the stale-reference sweep, left open because a change to those files needs a new reviewer run.
- Lane check: `lane: C2, 12 files, 1339 lines, EXCEEDS (lines 1339 > 500)` (warn only).
- `security-review`: done inline, no sub-task, because the fresh-context reviewer had already passed it independently.
  No High or Medium finding: a logged earlier file is parsed, masked, capped and written under the canonical note id;
  no new process, network call or dependency; a new log is created with mode 0600.
