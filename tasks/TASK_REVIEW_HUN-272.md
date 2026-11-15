# TASK_REVIEW — HUN-272: Query engine in Rust

> Sibling of `tasks/TASK_GUIDE_HUN-272.md`. Everything here is **filled by the reviewer at Stage
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
1. Crates: caseless 0.2.2 (Unicode default case folding, the same as Python `str.casefold`; Rust `to_lowercase` does not fold `ß` to `ss`) and serde 1.0.229 with derive (result types serialize for the CLI and MCP) — RULE: `tasks/TASK_GUIDE_HUN-272.md § Rust port rules` ("prefer a maintained crate over hand-written code")
2. Python type-error tests that cannot compile in Rust were replaced by the nearest Rust check (`limit = 0` → BadLimit, empty query → EmptyQuery, a date-time string → BadDate); dates must be exactly `YYYY-MM-DD`; `limit` is a required argument and Python's default 20 is exported as `DEFAULT_LIMIT` — RULE: `tasks/TASK_GUIDE_HUN-272.md § Rust port rules` ("Where a Python test checks something Rust makes impossible ... replace it with the nearest Rust check and list it in the review file")
Checks: tests `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` — unit 6, cli 5, format 13, query 45, store 29 passed (9d77974); reviews code-review (Skill) — 0 P0/P1; P2 lane_check `EXCEEDS (lines 1639 > 250)` for C1 — 1,103 of the lines are the ported oracle tests (44 Python test functions → 45 Rust tests), label kept; P3 `src/core/query.rs:348-351` `first_hit` folds the body once per char to map positions (fine at 20,000 chars per note; revisit in HUN-265 perf check); Supervisor mutations: cap `limit.min(MAX_LIMIT)` → `limit` fails 1 test (limit_is_capped_at_fifty), AND → OR of words fails 2 tests (search_needs_every_word_in_any_case, search_words_may_sit_in_different_places), both reverted; security-review not required (Risk Low); startup reads: PROJECT_SPEC.md, the guide, agents/backend.md (system prompt), the DDR — none missing
Memory: nothing new yet (pass runs at merge)
Paused: merge waits for the owner — the Companion score row is ☑ fail (72 < 80), and the merge gate blocks on it; the owner merges by hand
Not done: merge (owner, by hand); auto verify: ☑ pass (session 77aa5397-85da-4a59-9c40-04eabf17543a, commit 7bda00152fcb1db4ac9b2d7b899752a3ff2320b4, log .claude/verify-runs/HUN-272-7bda00152fcb.log); verify findings: `by_tag` counts tags case-sensitively while the tag filter ignores case — same as the Python oracle (`git show 2d497fc:src/easy_note/core/query.py` line 62), kept; `since` after `until` gives an empty page, not an error (unspecified, kept)
Paused: done not moved — report incomplete: auto verify slot

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/query.rs` (45 tests, port of Python `tests/test_query.py`; fixture `tests/fixtures/notes_basic/`). 45 passed, 0 failed (output in the next row) |
| Verification command run | ☑ pass | implementation commit 07ee8a2; `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`:<br>```<br>Sun Nov 15 03:06:50 PM UTC 2026<br>FMT-OK<br>    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.08s<br>     Running unittests src/lib.rs (target/debug/deps/easy_note_mcp-87e71ad8fdbf8075)<br>test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>     Running unittests src/main.rs (target/debug/deps/easy_note-6a6adfaa05f54c21)<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>     Running tests/cli.rs (target/debug/deps/cli-2de62b2d428d147e)<br>test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>     Running tests/format.rs (target/debug/deps/format-ef25e05f7a981a6a)<br>test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s<br>     Running tests/query.rs (target/debug/deps/query-2042827d1767a431)<br>test result_does_not_depend_on_the_reader_zone ... ok<br>test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s<br>     Running tests/store.rs (target/debug/deps/store-a18f8bbc91b20273)<br>test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>``` |
| Negative cases hold | ☑ pass | Red run (commit before `src/core/query.rs` existed): `error[E0432] unresolved import easy_note_mcp::core::query` / `could not compile easy-note-mcp (test "query")`. Mutation (tie order of IDs reversed, `SNIPPET_MAX` 200 to 201) made 5 tests fail:<br>```<br>Cargo.lock      |  26 ++++++++<br> Cargo.toml      |   2 +<br> src/core/mod.rs |   1 +<br> tests/query.rs  | 179 +++++++++++++++++++++++++++++++++++++++++---------------<br> 4 files changed, 161 insertions(+), 47 deletions(-)<br>test equal_instants_are_ordered_by_id ... FAILED<br>test clash_suffix_ids_tie_break_as_text ... FAILED<br>test list_snippet_is_the_start_of_the_body ... FAILED<br>test result_does_not_depend_on_the_reader_zone ... ok<br>test snippet_is_cut_around_the_first_hit ... FAILED<br>test snippet_never_exceeds_the_limit_at_any_hit_position ... FAILED<br>assertion `left == right` failed<br>assertion `left == right` failed<br>assertion `left == right` failed<br>assertion `left == right` failed<br>test result: FAILED. 40 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s<br>error: test failed, to rerun pass `--test query`<br>```<br>Source restored after the mutation. |
| verify | ☑ pass | auto-run by hook, session 77aa5397-85da-4a59-9c40-04eabf17543a, commit 7bda00152fcb1db4ac9b2d7b899752a3ff2320b4, log .claude/verify-runs/HUN-272-7bda00152fcb.log — pass |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | New files `src/core/query.rs`, `tests/query.rs`; one line in `src/core/mod.rs`; `Cargo.toml`/`Cargo.lock` (2 crates). `src/core/store.rs` and `src/surfaces` untouched. |
| Full smoke suite still green (no regression) | ☑ pass | whole `cargo test`: 6 + 0 + 5 + 13 + 45 + 29 passed, 0 failed |
| **Companion score (easy-verifier)** | ☑ fail | overall 72 (floor missed: 72 < 80), change_risk 100 pass (feature, confirmed; test pairs: 2 confirmed, none unpaired), scope changes, ref `main...HEAD`, abstentions: solution-fit (no_static_rule), reliability (insufficient_measured_weight), api, data, operations, infrastructure (below_coverage_floor), ratings: architecture 100, supply-chain 100, code-quality 87 (`lint_config_missing`), test-strategy 72, requirement-fidelity 67 (12% requirement items traced), blast-radius 46 (57% of changed files in churn hotspots), security 30 (`redaction_hits_observed` 1; the report does not name the file), gates: solution-fit left abstained (no kit evidence for the listed PRD/brainstorming refs), divergence: no findings sent, report: `/home/hungnguyenhuu/.easy-verifier-reports/hun-272-query/evidence-report-changes-20261115T150938-680172Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | No UI: core library task |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | No UI: core library task |
| **UI: Responsiveness at target viewports** | ☑ N/A | No UI: core library task |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (command `date -u; ls src/core/; cargo test 2>&1 | tail -3`, run before any implementation commit; `src/core/query.rs` is absent):

```
Sun Nov 15 03:03:08 PM UTC 2026
config.rs
format.rs
fsutil.rs
mod.rs
store.rs

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**AFTER** (same command after the change, from the verification run above): `src/core/query.rs` is listed in `src/core/`; `cargo test` shows `Running tests/query.rs ... test result: ok. 45 passed; 0 failed`.

**DELTA**: Code can now call `easy_note_mcp::core::query::{list_notes, search, stats}` to get deterministic, bounded (50 items, snippet at most 200 chars) answers over the notes folder.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]


---

## Implementation notes (backend-developer)

Crates: `caseless` 0.2.2 (Unicode default case folding, same as Python `casefold`, so `ß` folds to `ss`; Rust `to_lowercase` does not), `serde` 1.0 with `derive` (result structs serialize for the MCP surface later). Existing: `chrono`, `thiserror`. No new dev-dependencies.

API: `list_notes(&Filters, limit)`, `search(query, &Filters, limit)`, `stats(&Filters)` open the store from the env; `list_notes_in`, `search_in`, `stats_in` take a `&Store` (used by the tests). Results: `Page`, `NoteItem`, `Stats`; `Counts` serializes as a JSON object in count-then-name order.

Deviations from the Python oracle (all forced by Rust types or listed in the port rules):
1. Bad `limit` of another type (bool, float, string, None) and `query=None` / a `datetime` as a date cannot compile. Checks kept: `limit = 0` gives `BadLimit`; an empty query gives `EmptyQuery`; a date-time string as a date gives `BadDate`. Negative `limit` is impossible (`usize`).
2. Dates are `&str`/`String` and must be strictly `YYYY-MM-DD` (10 chars). Python `fromisoformat` also accepts `20261105`; the Python test refused it only because it was an int. Rust refuses the string `"20261105"` and `"2026-1-5"`.
3. Snippet position uses folded-length mapping like Python `_find`; behaviour equal. Whitespace split matches Python `str.split()` (Unicode whitespace plus `\x1c-\x1f`).
4. `limit` is a required argument of the Rust functions (Python default 20 is the exported `DEFAULT_LIMIT`; the surface passes it).

Cut list (not built): no `date`-typed filter fields (strings only), no index or cache.
