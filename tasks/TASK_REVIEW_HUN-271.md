# TASK_REVIEW — HUN-271: Context capture in Rust

> Sibling of `tasks/TASK_GUIDE_HUN-271.md`. Everything here is **filled by the reviewer at Stage
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
1. Crates: regex 1 (task pattern, no look-around needed), url 2 + percent-encoding 2 (remote and `file://` root parsing), wait-timeout 0.2 (2 s git timeout), tempfile 3 (dev) — RULE: `tasks/TASK_GUIDE_HUN-271.md § Rust port rules` ("spawn + `wait_timeout` crate"; "the `regex` crate is fine here"; "prefer a maintained crate over hand-written code")
2. The task pattern stays case-sensitive (`[A-Z][A-Z0-9]+-\d+`, plus `[Tt]\d{3,4}` upper-cased), not "after upper-casing" as the guide's port rule says — RULE: `tasks/TASK_GUIDE_HUN-271.md § Rust port rules` ("as in `git show 2d497fc:src/easy_note/core/context.py`"): the oracle code and its tests are case-sensitive, and the guide wording was a Supervisor error
3. The store link is a 9-line `Context::fields()` that returns the `(key, Option<&str>)` list `Store::add` takes; no store or surface change — RULE: `tasks/TASK_GUIDE_HUN-271.md § Dependencies & Reachability` ("The store link (passing context fields to `store::add`) is wired when HUN-269 is on main")
Checks: tests `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` — unit 6, cli 5, context 29, format 13, store 29 passed (d32f123, rebased on main 98fe207); reviews code-review (Skill) — 0 P0/P1; P2 lane_check `EXCEEDS (lines 1106 > 250)` for C1 — 668 of the lines are the ported oracle tests, label kept; P3 `src/core/context.rs:257-263` stdout is read after `wait_timeout`, so output over the pipe buffer (64 KiB) would stall to the timeout (git outputs here are one line each); P3 `Cargo.toml` new crates use major-only versions (`regex = "1"`) while the rest pin a minor (Cargo.lock pins all); startup-reads check: the part-2 implementer listed PROJECT_SPEC.md, the guide, agents/backend.md (delivered as its system prompt) and the DDR, and said it read PROJECT_SPEC.md and the DDR only in part — no Permanent-Rule read missing; security-review — see Evidence
Memory: nothing new yet (pass runs at merge)
Not done: merge (needs a named `[admin]`); auto verify: pending

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/context.rs` (29 tests). AC 7 and AC 9 store parts added: `slow_git_drops_the_fields_and_add_still_writes`, `no_absolute_path_in_any_written_file`; also `missing_folder_gives_no_fields_and_add_succeeds`, `token_in_remote_is_in_no_written_file` (ported from tests/test_context.py at 2d497fc). They write through `store::add` with `Context::fields()` into a temp `EASY_NOTE_DIR`. Red run against a stub `Context::fields` (returns no fields): `no_absolute_path_in_any_written_file` and `token_in_remote_is_in_no_written_file` FAILED, 27 passed; green: `cargo test` cli 5, format 13, store 29, context 29, lib 6, all ok; `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean |
| Verification command run | ☑ pass | `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` exit 0: unit 6, cli 5, context 29, format 13, store 29 passed (d32f123) |
| Negative cases hold | ☑ pass | Mutation 1, drop `command.env_remove(key)` (`src/core/context.rs \| 2 +-`): `FAILED. 28 passed; 1 failed` (git_dir_from_the_environment_does_not_redirect_git). Mutation 2, `GIT_TIMEOUT` 2 s → 10 s (`src/core/context.rs \| 2 +-`): `FAILED. 27 passed; 2 failed` (slow_git_drops_the_fields, only_read_only_git_subcommands_run). Red run of AC 7/AC 9 against the stub `Context::fields`: `FAILED. 27 passed; 2 failed` (b258018). Both mutations reverted |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed `src/core/context.rs` in full, `src/core/mod.rs`, `Cargo.toml`, `tests/context.rs` (test list against the 25 Python test functions); no caller in `src/surfaces` yet |
| Full smoke suite still green (no regression) | ☑ pass | cli 5, unit 6, format 13, store 29 pass on the rebased branch |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | No UI: core library task |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | No UI: core library task |
| **UI: Responsiveness at target viewports** | ☑ N/A | No UI: core library task |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: command `date -u; ls src/core/; cargo test 2>&1 | tail -3`, captured before the first implementation commit:

```
Sun Nov 15 10:36:54 AM UTC 2026
mod.rs

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`src/core/` holds only `mod.rs`; there is no `context.rs` and no context test (the 5 passing tests are `tests/cli.rs`).

**AFTER**: same command, Sun Nov 15 10:39:22 AM UTC 2026:

```
context.rs
mod.rs
tests/context.rs: test result: ok. 29 passed; 0 failed (cargo test, whole suite also green: cli 5, lib 6, format 13, store 29)
```

**DELTA**: `easy_note_mcp::core::context::capture` returns repo, remote, branch and task from read-only git (2 s timeout, no path, no token), which it could not before.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]
