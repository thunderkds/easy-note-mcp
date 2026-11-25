# TASK_REVIEW — HUN-293

> Sibling of `tasks/TASK_GUIDE_T[NNN].md`. Everything here is **filled by the reviewer at Stage
> 4/5** — it is deliberately NOT in the guide, because the implementing agent re-reads the guide on
> every turn and never fills these two sections.
>
> Consumers resolve each section **guide first, this file second** (`.claude/hooks/lib/guide_sections.py`):
> a legacy guide that still carries these sections inline keeps working unchanged, and a stray
> review file can never override an inline section.

---


## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | tests/enrich.rs (schedule_keys_are_read_and_the_bounds_hold, a_bad_schedule_value_names_the_key_not_the_value, tray_schedule_without_enrich_reads_but_is_not_effective, the_schedule_line_shows_every_key, settings_default_when_the_file_or_the_section_is_missing); tests/cli.rs (doctor_prints_the_ai_schedule_line_from_the_file) |
| Verification command run | ☑ pass | fmt --check and clippy -D warnings clean; cargo test --locked all `test result: ok`, 0 failed (lib 68; tests/enrich.rs 38; tests/cli.rs 48 after +1; others unchanged/ok). Tests were written with the code in one pass, so no separate red run was captured. |
| Negative cases hold | ☑ pass | Mutation run by the Supervisor: `interval_min` lower bound changed from 5 to 1 in `src/core/config.rs`; `cargo test --locked --test enrich` then printed `test a_bad_schedule_value_names_the_key_not_the_value ... FAILED` and `test result: FAILED. 27 passed; 1 failed`. Reverted with `git checkout src/core/config.rs`; the file is clean again. |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Fresh-context reviewer read the diff `main...72a7cc5`: `src/core/config.rs`, `src/surfaces/enrich.rs`, `src/surfaces/cli/read.rs`, README, `tests/enrich.rs`, `tests/cli.rs`. Callers of `AiSettings` (hook, quick, enrich) were checked for the new fields via `..default()`. Window and MCP code not touched, not reviewed. |
| Full smoke suite still green (no regression) | ☑ pass | `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` exit 0 by the implementer and again by the reviewer; every `test result` line `ok`, 0 failed. |
| **Companion score (easy-verifier)** | ☑ N/A | Not run in this session: the easy-verifier tool is not available here. The merge gate blocks on a `fail` row only. |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | No UI change in this task (config and doctor text only). |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | No UI change in this task. |
| **UI: Responsiveness at target viewports** | ☑ N/A | No UI change in this task. |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (2026-11-25 04:13:30 UTC, main at 74a290b, no implementation commit yet): `[ai]` has only enrich, command, claude_config_dir, model, timeout_secs, after_save. The new keys (schedule, interval_min, max_per_run, batch_size, jobs, digest_hour) do not exist. Verification command (`cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`) exited rc=0 and ended with:
```
test result: ok. 68 passed; 0 failed (lib)
test result: ok. 38 / 47 / 32 / 8 / 3 / 24 / 13 / 37 / 16 / 4 / 19 passed; 0 failed (integration binaries)
```
Prior doctor line with `[ai] enrich = "claude"`: `ai: claude via claude (<path>), account dir default, model haiku, after_save false`; no schedule text.

**AFTER**: [same command, post-change] OR [verbatim excerpt of the new content]

**DELTA**: [one sentence — what a user can now do that they could not before]

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]

## Stage 4 review

Reviewer: fresh-context qa-expert, commit 72a7cc5, scope: diff `main...72a7cc5` (config.rs, enrich.rs, read.rs doctor, README, tests/enrich.rs, tests/cli.rs).

Checks run in the worktree: `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` — exit 0, no warnings, every `test result` line `ok`, 0 failed (lib 68, enrich 38, cli 48, others ok).

### Acceptance Criteria

| # | Result | Trace |
|---|--------|-------|
| 1 | pass | src/core/config.rs:107-111 fields; defaults at :131-136 (off, 60, 20, 10, all three jobs, 3); bounds in `whole` at :189-199 (interval 5.., max_per_run 1-200, batch 1-50, digest 0-23); batch > max_per_run refused at :203; jobs validated at :211-232 |
| 2 | pass | Missing keys fall to defaults; existing keys untouched; tests/enrich.rs `settings_default_when_the_file_or_the_section_is_missing` and the unchanged every-key test (with `..default()`) pass |
| 3 | pass | `bad(key, rule)` messages are fixed text, never the value. Test `a_bad_schedule_value_names_the_key_not_the_value` covers 16 bad inputs (wrong type, range, unknown job, batch > max), asserts `[ai] <key>` and `config.toml` present and sample values absent |
| 4 | pass | `effective_schedule()` at config.rs:119-121; doctor suffix `schedule needs enrich = claude` at enrich.rs (describe_schedule); test `tray_schedule_without_enrich_reads_but_is_not_effective` |
| 5 | pass | `schedule_line()` wired at src/surfaces/cli/read.rs:209; tests `the_schedule_line_shows_every_key` and tests/cli.rs `doctor_prints_the_ai_schedule_line_from_the_file` (temp HOME) |

Edge cases: `jobs = []` reads (empty, shown as `none`); string-typed integer refused with key; unknown extra keys ignored (unchanged path); duplicate jobs de-duplicated.

### Findings

- P3 src/core/config.rs:197-199 (`interval_min` call at :199): the error for `interval_min` reads "must be a whole number from 5 to 4294967295". Correct but noisy; a "5 or more" wording would be clearer. Not blocking.
- P3 src/core/config.rs:203: when both `batch_size` and `max_per_run` are valid alone but conflict, the error blames `batch_size` only. Matches the guide (names `batch_size`). No action.
- P3 src/surfaces/enrich.rs `schedule_line`: on a bad config, `doctor` prints the same error twice (once in `ai:` line, once in `ai schedule:` line). Cosmetic.
- P3 tests/enrich.rs `a_bad_schedule_value_names_the_key_not_the_value`: the leak list holds "201" and "-1", short strings that could match unrelated text in a path. They pass today; weak but acceptable.
- P3 tasks/TASK_REVIEW_HUN-293.md Evidence: "Negative cases hold" row is unfilled and the implementer notes no red run was captured (tests written with the code). The Supervisor must fill it (mutation diff or red run) before Done.

No P0, P1 or P2 findings. No raw note file is touched (NFR-012 holds); no env vars added (out of scope respected).

Verdict: pass with findings

## Auto decisions

Mode: auto
1. Kept `interval_min` without an upper bound; the review's P3 about the noisy `4294967295` text is a follow-up — RULE: `tasks/TASK_GUIDE_HUN-293.md` AC 1 gives a minimum for `interval_min` and no maximum
2. Left the doubled `doctor` error line for a bad config as a P3 follow-up; no behaviour in the guide's AC is wrong — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` row 17 (a P3 finding is not fixed on this branch; it goes in the report)
3. Filled the `Negative cases hold` row with a Supervisor mutation run instead of asking the implementer for a red run — FACT: `tasks/TASK_REVIEW_HUN-293.md` Stage 4 review finding 5 "The Supervisor must fill it"
Checks: tests `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` — exit 0, every `test result` ok, 0 failed; reviews code-review by a fresh-context reviewer on 72a7cc5 — pass with findings (P3 only); real run `easy-note doctor` over a temp HOME with a `[ai]` section — printed `ai schedule: tray, every 30 min, up to 40 notes per run, batch 8, jobs drafts+digest, digest at 4h` (auto-checked, not /verify)
Memory: nothing new on this branch; the HUN-293 decision entry is written on main after the merge
Not done: merge (admin: after the auto verify pass); auto verify: running (T182)
