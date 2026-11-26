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
Memory: nothing new
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

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/overrides.rs` (25 passed; new: `hun302_one_undo_returns_the_whole_earlier_ai_file_after_an_item ... ok`, `hun302_one_undo_returns_replaced_and_cleared_draft_keys ... ok`, `hun302_undo_of_a_new_ai_file_removes_it_and_keeps_the_persons_overrides ... ok`, `hun302_a_second_enrich_that_changes_only_the_kind_is_undone ... ok`, `hun302_lines_without_an_op_are_undone_one_at_a_time_and_an_old_undo_line_takes_its_field ... ok`, `hun302_undo_does_not_bring_back_a_removed_ai_file ... ok`, `hun302_no_ai_file_is_written_when_its_log_line_cannot_be ... ok`, `hun302_the_cli_says_what_undo_did ... ok`, `hun302_an_item_whose_ai_file_line_is_unreadable_still_takes_back_all_its_keys ... ok`); `tests/organize.rs` (21 passed, undo expectations updated). Each AC has a normal-path test; the failure cases F1 and F2 have none |
| Verification command run | ☑ pass | Run by the reviewer at `7bffd27` in the worktree (its own `target/`; temp HOME and XDG_CONFIG_HOME; display and D-Bus variables unset; failing `claude` and `gsettings` stubs first on PATH, never called): `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` with an echo after each step printed `FMT exit 0`, `CLIPPY exit 0`, `VERIFY exit 0`; 26 test binaries, 731 passed, 0 failed, 7 ignored |
| Negative cases hold | ☑ fail | Reviewer probes: F1 (failed undo, then the retry removes a person's override) and F2 (torn line, half restore) do not hold. Mutants in a copy outside the worktree, one at a time against `--lib` and the tests `overrides organize ai leak mcp enrich`; `git diff --no-index --shortstat` against the worktree file was non-empty before each verdict, `1 file changed` each: M1 two op ids per item (+1 -1) killed, 4 tests red; M2 no creation line (+4 -1) killed, 6; M3 keep current draft keys (+4) killed, 2; M4 take the last `ai_file` line (+1 -1) killed, 4; M5 created file not removed (-1) killed, 4; M6 AI write before log (+1 -1) killed, 2; M7 old undo line pops the top (+2 -2) killed, 1; M8 no mask on restore (-1) SURVIVED; M9 restore before undo log (+1 -1) SURVIVED; M10 keep current `based_on` and `organized_at` (+4) killed, 2; M11 per-key order reversed (-1) SURVIVED; M12 undo line drops the replaced file (+1 -1) killed, 1 |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed: the 10 files of `main...7bffd27` (`src/core/ai.rs`, `src/core/overrides.rs`, `src/surfaces/cli/write.rs`, `src/surfaces/organize.rs`, `tests/ai.rs`, `tests/organize.rs`, `tests/overrides.rs`, `docs/NOTE_FORMAT.md`, `PRD.md`, the guide), plus the callers of the AI writers (`surfaces/enrich.rs`, `surfaces/mcp.rs` `note_add` and `note_enrich`) and `core/fsutil.rs`. Skipped: window page, tray, install and query code (not changed; their tests ran). Seen, not filed, outside the diff: `enrich` copies the old organizer keys after its cap check, so a hand-edited key that grows when masked gives an AI file that does not load |
| Full smoke suite still green (no regression) | ☑ pass | Default build: `cargo test --locked` 731 passed, 0 failed, 7 ignored (lib 76, ai 38, cli 51, enrich 48, leak 46, mcp 21, organize 21, overrides 25, window_api 116, and the other binaries). With features: `cargo clippy --all-targets --locked --features window,tray -- -D warnings` printed `CLIPPY-F exit 0`; `cargo test --locked --features window,tray` printed `FEATURES exit 0`, 740 passed, 0 failed, 7 ignored (display and D-Bus variables unset) |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [filled at Stage 4] |
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☑ N/A | No UI component: core library, log file and command-line output only |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ pass / ☐ fail / ☑ N/A | No UI component |
| **UI: Responsiveness at target viewports** | ☐ pass / ☐ fail / ☑ N/A | No UI component |

---

## Demonstration

**BEFORE**: [captured before the first implementation commit]

**AFTER**: [same command, post-change]

**DELTA**: [one sentence]

**WITNESS**: [who ran it and when]
