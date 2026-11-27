# TASK_REVIEW — HUN-295: `easy-note ai run`: caps, batches, a run lock, back-off and `--dry-run`

> Sibling of `tasks/TASK_GUIDE_HUN-295.md`. Everything here is filled at Stage 4/5.

## Review findings (Stage 4: code review + security review)

Reviewer: qa-expert, fresh context, 2026-11-27, commit `b5516788305c2e21949d96383b135177882c1a7d`, diff `main...b551678` (11 files). Code review (`skills/code-review/SKILL.md`) and security review (`skills/code-review/references/security-reviewer.md`, Risk Medium).

**No P0, P1 or P2 finding.** Three P3 notes:

- **F1 — P3, confidence high — `src/surfaces/organize.rs:315`** `RunState::backing_off` has no caller outside the unit test at `src/surfaces/organize.rs:506-511`. `run` records `failures` / `next_after` (AC 5 holds as written), but no product path reads `next_after` yet: a broken `claude` is called again at every cron time. RUNBOOK.md says this plainly ("cron ignores its `next_after`") and the tray timer is out of scope, so this is a note for the tray-timer task, not a defect of this one.
- **F2 — P3, confidence medium — `docs/NOTE_FORMAT.md:194` vs `src/core/store.rs:377` (`rm_with_ai`) / `:423` (`restore_with_ai`)** The doc says `ai-write.lock` is held "while an AI file ... is changed". `easy-note rm` / `restore` move the AI file between `.easy-note/ai/` and `trash/ai/` without the write lock. Probe (below): `rm` of a note during an in-flight `ai run` gives `skipped: no such note`, no orphan AI file, because `apply` re-reads the note under the lock. The only window left is the few ms between that re-read and the write. AC 11 lists only AI writes, `ai set`, `ai undo`, `ai rm`, so the AC holds; the doc sentence is wider than the code.
- **F3 — P3, confidence medium — `README.md:206`, `src/core/ai.rs:332` (`release`, called from `enrich_in`)** Any successful `enrich` ends a hold, also `ai enrich --pending` and MCP `note_enrich`. `ai::pending` (`src/core/ai.rs:721`) does not skip held notes, so an MCP client that lists pending notes and enriches them redoes a note a person undid. The owner decision names only `ai enrich <id>`; the README documents the wider behavior on purpose. Owner question, not a code defect.

**Acceptance Criteria (each judged):**

| AC | Verdict | Evidence |
|----|---------|----------|
| 1 enrich off: exit 1, how-to text, nothing started | pass | `with_enrich_off_it_says_how_to_turn_it_on_and_starts_nothing` (run and `--dry-run`, 0 calls, no `.easy-note/`) |
| 2 selection, newest first, `min(--limit, max_per_run)`, batches | pass | `five_notes_in_batches_of_two_take_three_calls_and_record_the_run`, `it_takes_the_newest_notes_up_to_the_smaller_of_limit_and_max_per_run` (also `--limit 0` exit 1) |
| 3 `--dry-run` no child, no write | pass | `dry_run_prints_the_plan_starts_no_child_and_writes_nothing` compares the whole notes tree |
| 4 run lock, `already running` exit 0, dead lock never blocks | pass | `a_second_run_says_already_running_and_a_dead_runs_lock_never_blocks` (kill -9 of a run at the gate) |
| 5 state file, doubling from `interval_min`, cap 4 h, reset, by-hand ignores `next_after` | pass (see F1) | `failed_runs_double_the_wait_and_a_good_run_resets_it` (20, 40 min), `the_wait_never_passes_four_hours`, unit tests in `organize.rs` |
| 6 no `organize`/`drafts` job: does nothing, says so | pass | `without_the_organize_and_drafts_jobs_it_does_nothing_and_says_so` |
| 7 summary line; exit 0 success/partial, 1 when every call failed | pass | `a_run_stops_at_the_first_failed_call_and_a_partial_run_exits_zero_and_backs_off`; on all-failed the summary is on stderr with the `easy-note: ` prefix (cron line uses `2>&1`) |
| 8 enriched-but-not-organized picked again | pass | `a_note_enriched_but_not_organized_is_picked_again`; `ai::enrich` / `ai::organize` bodies unchanged except the lock |
| 9 overrides read once, re-read under the lock before writing | pass | one `overrides::read` in `run` (`organize.rs`), `overrides::reread` after the call; mutant M6 killed |
| 10 hold after AI undo, also in flight; `ai enrich <id>` ends it; person undo no hold | pass (see F3) | `an_undone_note_is_held_until_its_text_changes`, `an_undo_made_while_the_call_runs_holds_the_note`, `ai_enrich_by_hand_ends_the_hold`, `undoing_your_own_fix_sets_no_hold`; mutants M1-M3 killed |
| 11 one write lock around AI writes, set, undo, rm; never during the call | pass (see F2) | `set_undo_rm_and_a_run_wait_for_the_write_lock`, `a_fix_made_while_the_call_runs_is_kept`; mutants M4, M5 killed |

**Security review (Medium):** no finding. Checked: `last_error` is first line only, masked with `guard::mask`, cut to 200 chars (failure texts from `enrich::call` are fixed strings, no note text); every stderr line goes through `safe()`; the model name in `--dry-run` goes through `safe()`; lock, hold and state paths are fixed names under `<notes dir>/.easy-note/`, ids come from `canonical_id` / the store; no raw note is opened for writing (NFR-012: tests compare note bytes); a bad `ai-hold.json` makes `ai run` and `ai undo` refuse and is never overwritten (so a person's undo is never silently redone). Lock nesting: every holder of `ai::lock` calls only `_locked` / inner writers; grep found no `ai::lock` call reachable while it is held (flock is not reentrant across fds, so this matters).

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | At `b551678`: `tests/ai_run.rs` (AC 1-11), `test result: ok. 21 passed; 0 failed; 0 ignored`; two unit tests in `src/surfaces/organize.rs`. Red first, before the code: `test result: FAILED. 0 passed; 21 failed` |
| Verification command run | ☑ pass | Supervisor, at the source of `b551678`: `cargo fmt`, `cargo clippy --all-targets --locked -- -D warnings` clean, `cargo test --locked --no-fail-fast`: 29 suites, 778 passed, 0 failed, 7 ignored, exit 0 |
| Negative cases hold | ☑ pass | Reviewer, worktree at `b551678`: `cargo fmt --check` clean; `cargo clippy --all-targets --locked -- -D warnings` clean; `cargo test --locked --test ai_run --test organize --test ai`: `ok. 21 passed` / `ok. 21 passed` / `ok. 38 passed`. 6 mutants in a copy (`/tmp/hun295-review`), all killed: M1 no in-flight hold check in `apply` -> `an_undo_made_while_the_call_runs_holds_the_note FAILED`; M2 no hold skip in `to_organize` -> 2 FAILED (`an_undone_note_is_held_until_its_text_changes`, ...in-flight); M3 no `release` -> `ai_enrich_by_hand_ends_the_hold FAILED`; M4 write lock taken before the `claude` call -> `a_fix_made_while_the_call_runs_is_kept FAILED` + in-flight undo FAILED; M5 `ai set` without lock -> `set_undo_rm_and_a_run_wait_for_the_write_lock FAILED`; M6 writes use the first overrides read, not the re-read -> `a_fix_made_while_the_call_runs_is_kept FAILED`. Probes with the copy's binary, temp HOME/XDG/EASY_NOTE_DIR, fake `claude`: no `HOME` -> `easy-note: HOME is not set`, exit 1; `batch_size = 0` -> `[ai] batch_size must be a whole number from 1 to 50`, exit 1; bad `ai-hold.json` + `--dry-run` -> `ai-hold.json is not valid; fix or remove it first`, exit 1; `easy-note rm <id>` during an in-flight run -> `organized 0, skipped 1, left 0 pending, calls 1`, `skipped: no such note`, no `.easy-note/ai/` file left. |
| verify | ☐ pass / ☐ fail / ☐ N/A | |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Read the full `git diff main...b551678` (src/core/ai.rs, src/core/overrides.rs, src/surfaces/cli.rs, src/surfaces/organize.rs, tests/ai_run.rs, tests/ai.rs, tests/organize.rs, README.md, RUNBOOK.md, docs/NOTE_FORMAT.md, guide). Callers checked by grep: `run_batch` (only `organize::run` and tests/organize.rs), `ai::lock` / `_locked` holders (ai.rs, overrides.rs, organize.rs; no nested lock), AI-file writers incl. `store.rs` `rm_with_ai` / `restore_with_ai` (F2), `ai::pending` vs `to_organize`. Full suite in the copy: `cargo test --locked --no-fail-fast`: 29 suites, 778 passed, 0 failed, 7 ignored. Worktree src/tests unchanged (`git diff --quiet b551678 -- src tests`). |
| Full smoke suite still green (no regression) | ☑ pass | `cargo test --locked --features window,tray --no-fail-fast`: 29 suites, 787 passed, 0 failed, 7 ignored, exit 0; clippy with the features clean. Pin test changed: `tests/ai.rs` `note_tree` now skips `.easy-note/ai-write.lock` |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | |
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☑ N/A | No UI component: core library and command-line output only |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ pass / ☐ fail / ☑ N/A | No UI component |
| **UI: Responsiveness at target viewports** | ☐ pass / ☐ fail / ☑ N/A | No UI component |

---

## Demonstration

**BEFORE** (Supervisor, 2026-11-27, installed `/usr/bin/easy-note`, which has no `ai run`):

```
$ easy-note ai run --help
error: unrecognized subcommand 'run'

  tip: a similar subcommand exists: 'undo'
```

**AFTER** (Supervisor, 2026-11-27T09:50+07:00, debug binary of `b551678`, temp HOME and notes dir, fake `claude`, `batch_size = 2`):

```
$ easy-note ai run --dry-run
dry run: 3 notes in 2 calls with model haiku; nothing was started or written
$ easy-note ai run
organized 3, skipped 0, left 0 pending, calls 2
$ easy-note ai run
nothing to do
$ easy-note ai undo 20261127-095045-6bca
✓ undone: AI detail of 20261127-095045-6bca removed (the AI had created it)
$ easy-note ai run
nothing to do
```

**DELTA**: one command now organizes every waiting note within the limits, and an undone note stays undone.

**WITNESS**: Supervisor session; the real `claude` CLI was never started.
