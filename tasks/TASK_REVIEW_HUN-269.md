# TASK_REVIEW — HUN-269: Note format and store in Rust

> Sibling of `tasks/TASK_GUIDE_HUN-269.md`. Everything here is **filled by the reviewer at Stage
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
1. YAML reader: `saphyr` + `saphyr-parser` 0.3.4 (YAML 1.2), not a hand-written reader — RULE: `docs/ddr/0005-rust-single-binary.md § Decision` item 6 ("a maintained YAML crate. `serde_yaml` is archived and is not used")
2. Accept four reader differences from the Python oracle, all on hand-edited files only (files the tool writes are unchanged): a YAML error names the line where the bad quote opens (`line 3`, Python `line 5`); a duplicate front-matter key is skipped as invalid YAML (Python kept the last value); an unquoted `id: no` reads as text and is skipped with a different reason; `created`/`updated` must be strict RFC 3339 — DEBATED: accept as-is (Advocate 1: accept all four; FR-017 `PRD.md:94` asks only that unreadable files are counted skipped with their path, never changed or hidden; `docs/NOTE_FORMAT.md:78-81` already lists invalid YAML and wrong types as skip reasons and asks for "ISO 8601 with an offset"; `docs/ddr/0005-rust-single-binary.md:36` allows a maintained YAML crate; Advocate 2: match Python before merge or amend DDR 0005, because decision 5 `docs/ddr/0005-rust-single-binary.md:32` says "Behaviour does not change", and a hand-edited file with a duplicate key that Python read is now skipped). Chosen: Advocate 1, because every case still ends as Skipped with a reason that quotes no content and no file is changed; Advocate 2's duplicate-key case is the one real loss for a reader and is named for the owner ⚠ decided for you — please review
3. Crates: chrono 0.4.45, sha2 0.11.1, gethostname 1.1.0, toml 1.2.1, thiserror 2.0.21 — RULE: `docs/ddr/0005-rust-single-binary.md § Decision` item 6 ("Time: `time` or `chrono`; hashing: `sha2`")
4. PROJECT_SPEC layout and risk table rewritten for the Rust files (b6f66d3, c4a6859) — RULE: `tasks/TASK_REVIEW_HUN-268.md § Auto decisions` Checks: "fix carried to HUN-269"
Checks: tests `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` — 6 + 5 + 13 + 29 passed (14:32 UTC, c4a6859, rebased on main a2491f8); reviews code-review (Skill) — 0 P0/P1; P2 `src/core/store.rs:269-272` `rm` opens the trashed file for write to set its mtime, so a read-only (0444) note is moved to trash and then `rm` returns EACCES (Python used `os.utime`, which succeeds; checked) — fix: `fs::File::open(&dst)?.set_modified(..)`, carried to HUN-270 part 2 (same file); P2 lane_check `EXCEEDS (lines 2270 > 500)` — 1,015 of the lines are ported oracle tests, C2 label kept; P2 fixed: stale `.py` paths in PROJECT_SPEC risk table (c4a6859); P3 `Store` exposes its test seams (`now_ns`, `link`, `rename`) as `pub` fields; startup-reads check: the implementer's report line is not in this session's context after compaction (not checked)
Memory: nothing new yet (pass runs at merge)
Not done: merge (needs a named `[admin]`); auto verify: pending
Paused: merge waits for the owner — the Companion score row is ☑ fail (67 < 80), and the merge gate blocks on it; the owner merges by hand or sends `[admin] merge HUN-269` with an explicit override

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/format.rs` (13), `tests/store.rs` (29), ported red first from the Python oracle (c4967b9). `cargo test`: `test result: ok. 13 passed; 0 failed` / `test result: ok. 29 passed; 0 failed` (14:32 UTC, c4a6859) |
| Verification command run | ☑ pass | `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` exit 0: unit 6, cli 5, format 13, store 29 passed |
| Negative cases hold | ☑ pass | Mutation `link: fs::hard_link` → `fs::rename` (`src/core/store.rs \| 2 +-`): `test result: FAILED. 23 passed; 6 failed` (write_never_replaces_existing_file_with_hard_links, two_writes_same_second_same_text_get_clash_suffix, restore_never_replaces_a_note_at_the_target, rm_moves_to_trash_and_restore_brings_back_same_bytes, rm_stamps_trash_time_so_an_old_note_is_not_emptied_at_once, empty_trash_deletes_only_files_older_than_cutoff); reverted |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed `src/core/{config,format,fsutil,store,mod}.rs`, `src/lib.rs`, `src/main.rs`, `Cargo.toml`, `tests/{format,store}.rs`, PROJECT_SPEC.md; surfaces untouched beyond the `main.rs` import |
| Full smoke suite still green (no regression) | ☑ pass | `tests/cli.rs` 5 passed, unit 6 passed on the rebased branch |
| **Companion score (easy-verifier)** | ☑ fail | overall 67 (floor missed: 67 < 80), change_risk 100 pass (feature, confirmed; test pairs: 9 confirmed, 1 rejected, fsutil.rs paired via tests/store.rs:299; main.rs unpaired), scope changes, ref `main...HEAD`, abstentions: solution-fit (no_static_rule), reliability (insufficient_measured_weight), api, data, operations, infrastructure (below_coverage_floor), ratings: architecture 100, supply-chain 100, test-strategy 72, requirement-fidelity 67, blast-radius 65, code-quality 33 (`max_function_ccn` 23 > 15 in src/core/format.rs `parse`; 12% of functions over CCN 10; `lint_config_missing`), security 30 (`redaction_hits_observed` 1; the report does not name the file), gates: solution-fit left abstained (no kit evidence for the listed PRD/brainstorming refs), divergence: no findings sent (refs had to come from the packs), report: `/home/hungnguyenhuu/.easy-verifier-reports/hun-269-store/evidence-report-changes-20261115T143553-587061Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | No UI: core library task |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | No UI: core library task |
| **UI: Responsiveness at target viewports** | ☑ N/A | No UI: core library task |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (captured before any implementation commit; command `date -u; ls src/core/; cargo test 2>&1 | tail -3`):

```
Sun Nov 15 10:35:55 AM UTC 2026
mod.rs

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Only `mod.rs` exists in `src/core/`; no format or store code, no store tests.

**AFTER** (same command, 14:32 UTC, c4a6859):

```
Sun Nov 15 02:32:22 PM UTC 2026
config.rs
format.rs
fsutil.rs
mod.rs
store.rs
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**DELTA**: The Rust core can now write, read, edit, trash, restore and list notes in the `easy-note: 1` format without ever overwriting a file.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]
