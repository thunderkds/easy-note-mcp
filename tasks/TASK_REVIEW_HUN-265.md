# TASK_REVIEW — HUN-265: Perf check over 10,000 notes: search, note_add, hook and n timings (Rust)

> Sibling of `tasks/TASK_GUIDE_HUN-265.md`. Everything here is **filled by the reviewer at Stage
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
1. Took HUN-265 into a parallel run with HUN-262 and HUN-263 (the user's request "spawn agents parallel, supervisor [admin]"), each in its own worktree; HUN-266 left out (Owner manual task) — RULE: `docs/claude-md/pipeline-stages.md` § Stage 3: Parallel Execution via Isolation
2. Re-planned the guide for Rust (one new `tests/perf.rs`, ignored timing tests on the release binary, fixture check in CI) and signed the Requirement Fidelity Gate as Supervisor — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 13)
3. Python generator/bench scripts dropped; the fixture is a test helper only — FACT: docs/ddr/0005-rust-single-binary.md:31 "4. **All Python is removed** in the first Rust task"
4. Warm disk cache only; a cold-cache search run is left to the owner (needs root) — SAFER OPTION: no root command in an auto run, Risk Low (decide and log, no Advocates) ⚠ decided for you — please review
5. Timings measured on this WSL2 machine, labelled WSL2 in the Evidence; a native Linux run is left to the owner — SAFER OPTION: report what was measured, never claim native numbers ⚠ decided for you — please review
Checks: tests `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` — all green (perf: 1 passed, 5 ignored); reviews code-review — 0 P0/P1/P2, 2 P3 (`reports/code-review_feat-HUN-265-perf_20261117T103600.html`); real run `cargo test --release --test perf -- --ignored --nocapture --test-threads=1` — printed 5 `perf` lines, all budgets met on WSL2 (auto-checked, not /verify)
Memory: memory/learnings.md — 2026-11-17 `board_sync.py update --guide` pushes the guide of the checkout the script lives in; memory/MEMORY.md — its index line
Not done: merge (admin: after the auto verify pass); auto verify: running (T182); native Linux and cold-cache timings (yours)

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/perf.rs`: AC 1 `fixture_has_10000_valid_notes` (in `cargo test`: `test result: ok. 1 passed; 0 failed; 5 ignored`); AC 2–8 `perf_search`, `perf_note_add`, `perf_hook_pass`, `perf_hook_save`, `perf_n_cold_start` (ignored, `--release`): `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 11.51s`; AC 9 this table and the Demonstration |
| Verification command run | ☑ pass | Supervisor rerun 2026-11-17T03:34:31Z, WSL2 (`6.18.58.1-microsoft-standard-WSL2`, AMD Ryzen 7 5700X 8-Core, 16 threads, 15 GiB RAM; power state not readable on WSL2): `perf hook_pass: n=30 first 1.8 ms p50 1.3 ms p95 1.5 ms max 1.6 ms` · `perf hook_save: n=30 first 15.6 ms p50 14.1 ms p95 16.0 ms max 16.0 ms` · `perf n_cold_start: n=30 first 15.1 ms p50 14.1 ms p95 15.0 ms max 15.6 ms` · `perf note_add: n=30 first 13.7 ms p50 5.0 ms p95 5.3 ms max 5.4 ms` · `perf search: n=30 first 176.8 ms p50 168.6 ms p95 173.1 ms max 180.5 ms`. NFR-004 met (173 < 500; 5.3 < 150), NFR-009 met (1.5 < 50; 16.0 < 150), NFR-010 met (15.0 < 150); fmt and clippy `-D warnings` exit 0 |
| Negative cases hold | ☑ pass | Mutation `BUDGET_SEARCH_MS` 500 → 1 (`git diff --stat`: `tests/perf.rs \| 2 +-`): `perf search: n=30 first 180.6 ms p50 176.6 ms p95 216.5 ms max 255.8 ms` then `panicked at tests/perf.rs:206:5: search: p95 216.5 ms is over the 1 ms budget`, `test result: FAILED. 0 passed; 1 failed` — the line prints before the assert (AC 8); mutation reverted (`git checkout tests/perf.rs`) |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed `tests/perf.rs` (the only code file) against `tests/hook.rs`, `tests/quick.rs`, `tests/query.rs`, `tests/mcp.rs` patterns and the hook fixtures it reads; `src/` skipped (unchanged) |
| Full smoke suite still green (no regression) | ☑ pass | `cargo test`: lib 27, cli 43, context 29, format 13, guard 37, hook 14, hotkey 4, inbox 19, install 23, leak 46, mcp 20, perf 1 (+5 ignored), query 45, quick 26, readme 3, store 30 — all `0 failed` |
| **Companion score (easy-verifier)** | ☑ N/A | Not run: a test-only change (one measurement file); no companion tool run in this auto run |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | No UI: the task adds one test file (`tests/perf.rs`) and no screen |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | No UI: the task adds one test file (`tests/perf.rs`) and no screen |
| **UI: Responsiveness at target viewports** | ☑ N/A | No UI: the task adds one test file (`tests/perf.rs`) and no screen |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured in worktree `.claude/worktrees/HUN-265` at main 1836e22, before any HUN-265 implementation commit:

```
Tue Nov 17 03:25:40 UTC 2026
$ cargo test --test perf
  Downloaded tempfile v3.27.0
error: no test target named `perf` in default-run packages

help: a target with a similar name exists: `leak`
exit=101
$ ls tests/perf.rs
ls: cannot access 'tests/perf.rs': No such file or directory
$ git rev-parse --short HEAD
1836e22
```

**AFTER**: at e50f039 (2026-11-17T03:34:31Z), `cargo test --release --test perf -- --ignored --nocapture --test-threads=1`:

```
perf hook_pass: n=30 first 1.8 ms p50 1.3 ms p95 1.5 ms max 1.6 ms
perf hook_save: n=30 first 15.6 ms p50 14.1 ms p95 16.0 ms max 16.0 ms
perf n_cold_start: n=30 first 15.1 ms p50 14.1 ms p95 15.0 ms max 15.6 ms
perf note_add: n=30 first 13.7 ms p50 5.0 ms p95 5.3 ms max 5.4 ms
perf search: n=30 first 176.8 ms p50 168.6 ms p95 173.1 ms max 180.5 ms
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 11.51s
```

**DELTA**: The owner can now measure the five speed budgets over 10,000 notes with one command, and NFR-004/009/010 are confirmed in `PRD.md` with measured numbers instead of provisional targets.

**WITNESS**: the Supervisor (session e61f3928), rerun in the HUN-265 worktree at 2026-11-17T03:34:31Z, separately from the implementing agent's run at 03:31:29Z; trace `memory/event-trace/HUN-265.jsonl`
