# TASK_REVIEW — HUN-257: Rust CLI commands

> Sibling of `tasks/TASK_GUIDE_HUN-257.md`. Everything here is **filled by the reviewer at Stage
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
1. Closed HUN-255 (Python guard) as superseded by HUN-270 — RULE: `memory/decisions.md § 2026-11-15 — App rewritten in Rust; HUN-268 merged over a failing Companion score` ("HUN-255 (Python guard) not merged; its branch at 3c5e40f is the oracle for HUN-270")
2. Re-planned this guide for Rust (Depends on HUN-269..272; cargo verification command; `tests/leak.rs` registration) — RULE: `memory/decisions.md § 2026-11-16 — HUN-271, HUN-270, HUN-272 merged by the owner over failing Companion scores; Rust port of the core complete` ("HUN-257–265 guides still describe Python and must be re-planned in Rust")
3. Raised Risk from Low to Medium (CLI output is a leak path), so security-review runs — SAFER OPTION: tracks.md lets the Supervisor move a task only up ⚠ decided for you — please review
4. Tickets run one at a time, not in parallel as asked; parallel work stays inside one ticket (reviewers, planning) — RULE: `docs/claude-md/tracks.md` `### Session tags` ("The loop opens no pull request and runs no two tickets at once") ⚠ decided for you — please review
5. Confirmation line says `1 secret masked` / `2 secrets masked` instead of the literal `secret(s)` — SAFER OPTION: FR-021 fixes the content (id, repo, task, masked count), not the plural form ⚠ decided for you — please review
6. `edit` saves only text and tags; any other changed front-matter key is refused, original kept; scratch file inside `<notes dir>/.easy-note/edit/` — RULE: `PRD.md` NFR-003 ("Writes only inside the notes dir, the config file, …")
7. Companion score ☑ fail (74) recorded as is; the code is not reworded to dodge `redaction_hits_observed` — RULE: `skills/score/SKILL.md § Never` ("Never rerun the same ref to shop for a better number") and `memory/decisions.md § 2026-11-16 — HUN-271, HUN-270, HUN-272 merged by the owner over failing Companion scores` (same false positive)
Checks: tests `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` — 260 passed, 0 failed (Supervisor rerun 2026-11-16T08:03:49Z, 73cc93f); reviews code-review (Skill) — 0 P0, 2 P1 fixed in 73cc93f (`add -e <text>` wrote unmasked text to the scratch file; scratch file left on a killed run), 8 P2 fixed (edit extra keys/tag shape/#word tags, broken pipe exit 101, `n --version <words>`, `$EDITOR` with spaces + `$VISUAL`, control characters shown as `\xNN`, leak test exit codes), cheap P3s fixed; deferred: SIGINT ignore while the editor runs, lost update during `edit`, keep typed text on a refused edit, `Fail`→`CliError` rename; security-review (Skill) — no finding at ≥ 8 (terminal-escape lead scored 7, hardened anyway in 73cc93f); score — ☑ fail 74; real run `easy-note add hello '#a'` in a fresh temp HOME — printed `easy-note: created notes dir …/easy-notes` then `✓ noted 20261116-144849-38db` (Demonstration AFTER; auto-checked, not /verify)
Memory: nothing new yet — the diff-driven memory pass runs after the merge
Not done: merge (admin: after the auto verify pass); auto verify: not started
Paused: auto-mode check failed — Risk Medium: decision 3 (`SAFER OPTION`) is a choice — it is flagged ⚠ — so it must be `DEBATED` with exactly 2 Advocates named; Risk Medium: decision 4 (`RULE`) is a choice — it is flagged ⚠ — so it must be `DEBATED` with exactly 2 Advocates named; Risk Medium: decision 5 (`SAFER OPTION`) is a choice — it is flagged ⚠ — so it must be `DEBATED` with exactly 2 Advocates named; Risk Medium: decision 6 (`RULE`) is a choice — its RULE cites no written rule (`path:line`, `path § <heading>` or `docs/claude-md/tracks.md` `### <section>`) — so it must be `DEBATED` with exactly 2 Advocates named [report 19204e399faf]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/cli.rs` (43 tests; AC 1–7, 9, 10 mapped per test in the implementer report), `tests/leak.rs` `surface_cli` (AC 8: real binary, add by arg/pipe/`-e`/`-e <text>`, ls, search, show, edit, tag, stats, doctor, rm, trash, restore + 12 error paths), unit tests in `src/surfaces/cli.rs`, `src/surfaces/cli/write.rs`, `src/surfaces/cli/scratch.rs`, `src/core/tags.rs` |
| Verification command run | ☑ pass | Supervisor rerun 2026-11-16T08:03:49Z at 73cc93f: `cargo fmt --check` ok; `cargo clippy --all-targets -- -D warnings` `Finished`; `cargo test`: `test result: ok.` 17 + 43 (cli) + 29 + 13 + 37 + 46 (leak) + 45 + 30 passed, 0 failed |
| Negative cases hold | ☑ pass | Red runs before the fix commit 73cc93f: leak `"cli x aws_key:...: secret on disk: home/seen-3968173.txt"` (P1.1, `add -e <text>`); `edit_and_add_dash_e_sweep_scratch_files_of_dead_processes` and `doctor_lists_leftover_scratch_files_by_name_and_stays_read_only` failed with src/ reverted (P1.2); P2 tests 11 of 43 failed before the code. Mutations in 9d4f22a: unmasked `fail.msg` fails `error_text_is_masked_before_it_is_printed` + CLI leak; disabled changed-field refusal fails `edit_refuses_bad_front_matter_and_keeps_the_original` |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed `git diff origin/main..HEAD`: src/surfaces/cli.rs, cli/{read,write,scratch}.rs, src/core/tags.rs, src/core/mod.rs, Cargo.toml, tests/cli.rs, tests/leak.rs + direct callees in src/core/{store,guard,query,config,context}.rs. Skipped: src/surfaces/mcp.rs (unchanged; only the `starts_mcp` boundary checked). lane_check: `EXCEEDS (files 9 > 6; lines 1848 > 250)` — about half are tests |
| Full smoke suite still green (no regression) | ☑ pass | Full `cargo test` green (all 10 test binaries, 260 tests) at 73cc93f |
| **Companion score (easy-verifier)** | ☑ fail | overall 74 (floor missed: 74 < 80), change_risk 100 pass (detected bugfix from commit 73cc93f; branch signal feature; rule `bugfix_without_regression_test` 2 ≥ 1 passed), scope changes, ref `origin/main...HEAD` (73cc93f), ratings: architecture 100, supply-chain 100, code-quality 87, test-strategy 72 (`source_files_without_covering_test_share` 1.0: name pairing does not link src/surfaces/cli/*.rs to tests/cli.rs), requirement-fidelity 67 (`requirement_items_traced_share` 0.18), blast-radius 65 (`changed_files_in_churn_hotspots_share` 0.58), security 30 (`redaction_hits_observed` 6: guard patterns and runtime-built planted values in tests, as in HUN-269..272); abstentions: solution-fit (no static rule), reliability (insufficient measured weight), api, data, operations, infrastructure (below coverage floor); gates: none answered (rung 2 CLI, MCP not registered in this session; CLI never asks); findings sent empty; report: `/home/hungnguyenhuu/.easy-verifier-reports-local/hun-257-cli/hun-257-cli/evidence-report-changes-20261116T080519-734795Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | Pure CLI task, no UI component (guide § UI / Design Acceptance Criteria) |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | Pure CLI task, no UI component (guide § UI / Design Acceptance Criteria) |
| **UI: Responsiveness at target viewports** | ☑ N/A | Pure CLI task, no UI component (guide § UI / Design Acceptance Criteria) |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured 2026-11-16T07:24:54Z on branch feat/HUN-257-rust-cli at d7c0f35 (no implementation commit yet), `cargo build`, temp HOME `/tmp/tmp.ojZQoKhX25`:

```
$ HOME=$H EASY_NOTE_DIR= ./target/debug/easy-note add hello '#a'; echo exit=$?
easy-note: add is not implemented yet
exit=2
$ HOME=$H EASY_NOTE_DIR= ./target/debug/easy-note ls; echo exit=$?
error: unrecognized subcommand 'ls'

Usage: easy-note <COMMAND>

For more information, try '--help'.
exit=2
$ ls -A $H     # nothing written
```

**AFTER**: captured 2026-11-16T07:48:49Z on the same branch after the implementation commit, temp HOME `/tmp/tmp.EKUe9AIsNn`, no `EASY_NOTE_DIR`, no config (`/home/.../scratchpad/demo.sh`):

```
$ easy-note add hello '#a'; echo exit=$?
easy-note: created notes dir /tmp/tmp.EKUe9AIsNn/easy-notes
✓ noted 20261116-144849-38db
exit=0
$ easy-note ls; echo exit=$?
20261116-144849-38db  hello #a  #a
exit=0
$ ./n retry '#flaky' "it's!"; echo exit=$?     # n = symlink to the binary
✓ noted 20261116-144849-b696
exit=0
$ easy-note search flaky
20261116-144849-b696  retry #flaky it's!  #flaky
$ easy-note doctor
notes dir: /tmp/tmp.EKUe9AIsNn/easy-notes
2 notes checked
no problems found
$ printf 'typed at a prompt #tty\n' | script -qec "$H/n" /dev/null     # a real pty
note> ✓ noted 20261116-144850-ebb4
```

**DELTA**: a person can now type `n <words>` (or `easy-note add`) in a fresh account with no setup and get a saved, tagged note and the confirmation line, then list, search, show, edit, tag, remove, restore, count and check notes from the terminal.

**WITNESS**: Supervisor session 16bb6d5a reran the full check at 2026-11-16T08:03:49Z on 73cc93f (trace `memory/event-trace/HUN-257.jsonl`); three independent reviewers (correctness/adversarial, tests/maintainability, security) probed the binary in temp HOMEs
