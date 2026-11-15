# TASK_REVIEW — HUN-270: Secret guard in Rust

> Sibling of `tasks/TASK_GUIDE_HUN-270.md`. Everything here is **filled by the reviewer at Stage
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
1. The guard restates Python's look-ahead patterns without look-around and uses the `regex` crate only (no `fancy-regex`): `openai_key` skips a match that starts with `sk-ant-` in code; `password_assign`/`secret_assign` match the key by regex and read the value with a hand-written scanner that stops at a `[masked:` marker — RULE: `docs/ddr/0005-rust-single-binary.md § Decision` item 6 ("Either restate them without look-around, or use `fancy-regex` for those patterns only. The leak oracle decides."); the implementer compared Python and Rust `mask` on 30 edge strings with identical output
2. AC 6 (no note text in logs) is checked in the core by source scans of `src/core/guard.rs` and `src/core/store.rs` for printing and logging macros; real stdout/stderr (and log, if a surface adds a logger) capture moves to the surface tasks, which register in the `SURFACES` registry of `tests/leak.rs` — DEBATED: option 3 (Advocate 1: keep the source scan; the crate has no logger, so a DEBUG handler test would pass by capturing nothing, `Cargo.toml:18-30`, `tests/leak.rs:325-326`; Advocate 2: add a `log` dev-dependency with a capturing logger at Trace plus a stdout/stderr check of the binary now, because a name scan misses `write!(io::stderr())`, aliases and dependency logging, and `src/surfaces/cli.rs:60,65` prints `{err}`; Advocate 3: keep the core scan and make HUN-257/258/259 register in `SURFACES` with real capture, as AC 5 already says surfaces add themselves, `tasks/TASK_GUIDE_HUN-270.md:37,82`, `tests/leak.rs:208-218`). Chosen: Advocate 3, because the core has no logger and no process today, and the CLI, MCP and hook are where a logger and output streams appear. Follow-ups: (a) HUN-257/258/259 guides name `tests/leak.rs` (not `tests/test_leak.py`) and require real stdout/stderr capture, plus a DEBUG-level log capture if the surface adds a logger; (b) HUN-257 masks error text before `eprintln!` (`src/surfaces/cli.rs` prints `{err}`, and `StoreError` echoes a bad ID or source); (c) widen the core scan to every file under `src/core/` (a qa-expert change to the locked test) ⚠ decided for you — please review
3. Planted secrets in fixtures and tests are split into parts and joined at runtime, so no token literal is in the repo — RULE: `tasks/TASK_GUIDE_HUN-270.md § Edge Case Checklist` ("Build planted values at runtime (`concat!`, `format!`, or joining parts), keep them fake")
4. `Store::rm` sets the trash mtime through a read-only handle (Stage 4 finding carried from HUN-269) — RULE: `tasks/TASK_REVIEW_HUN-269.md § Auto decisions` Checks: "fix carried to HUN-270 part 2 (same file)"
Checks: tests `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` — guard 37, leak 40, store 30, format 13, cli 5, unit 6 passed (a946a46); reviews code-review (Skill) — 0 P0/P1; P2 lane_check `EXCEEDS (files 10 > 6; lines 2587 > 250)` for C1 — about 2,100 lines are the locked oracle tests and fixtures, label kept; P3 `src/core/store.rs` error text echoes caller input for a bad ID or source (`not a note id: {id:?}`, `unknown source {source:?}`), as the Python oracle did (`git show 3c5e40f:src/easy_note/core/store.py` lines 76, 264) — the surface tasks must mask error text before output (HUN-257, HUN-258); startup reads: part 2 implementer listed PROJECT_SPEC.md, the guide, agents/backend.md (system prompt) and the DDR (first 30 lines) — none missing; security-review (Skill, opus sub-task, reproduced in a throwaway crate) — 1 finding at confidence 8: Medium, sanitization bypass, `src/core/store.rs:510-512` `mask_note` masks the raw YAML source of unknown front-matter keys (`extra`), so a double-quoted escape directly before a token (`desc: "creds:\n<AKIA…>"`, `"Authorization:\tBearer <token>"`) removes the `\b` word boundary the anchored kinds need (`src/core/guard.rs` lines 72, 75, 79, 82, 86, 94, 97); `read`, `iter_notes` and `update` return the key unmasked and `update` writes it back with `masked: []` — P1, fixed in part 3 (0cf5ba5 red: `FAILED. 42 passed; 4 failed`; 39db71e green: `ok. 46 passed`): `mask_note` keeps the raw pass and adds `mask_decoded`, which re-emits a key whose decoded strings hold a secret; Supervisor re-ran `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` at 39db71e: guard 37, leak 46, store 30, format 13, cli 5, unit 6 passed; P3 cut list: a list or map value with a secret collapses to one marker; leads below 8 (not blocking): a value that holds the literal `[masked:` is skipped whole by `password_assign`/`secret_assign` (`guard.rs:168`, `201-203`; confidence 7); a YAML value on the next line after `password:` is not masked (confidence 5; part 3 decoded-value masking covers `extra`); a note file named after a token leaks it in `Note.id` (confidence 3); error text: no leak found
Memory: nothing new yet (pass runs at merge)
Not done: score; merge (owner, by hand if the score fails); auto verify: pending
Paused: auto-mode check failed — Risk High, but no decision was DEBATED with 3 Advocates [report 2dbd1a12fc6e]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | Part 1 (ACs 1, 2, 4 helper, 6, 7): `tests/guard.rs`. Part 2 (store wiring, ACs 3, 4, 5, 6, 8, 9, 10): `tests/leak.rs` (40 tests) and `tests/store.rs` (30 tests, incl. `rm_moves_a_read_only_note_and_stamps_its_mtime`). All qa-expert oracle, locked, not edited. Output below in the verification row. |
| Verification command run | ☑ pass | Part 1, run 2026-11-15 14:28:39 UTC in the worktree: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` exit 0. `tests/guard.rs`: `test result: ok. 37 passed; 0 failed`. Other targets: `src/main.rs` 6 passed, `tests/cli.rs` 5 passed. |
| Verification command run (part 2) | ☑ pass | Run 2026-11-15 15:53:04 UTC after `cargo clean -p easy-note-mcp`, `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` exit 0: `tests/guard.rs` 37 passed; `tests/leak.rs` 40 passed; `tests/store.rs` 30 passed; `tests/format.rs` 13 passed; `tests/cli.rs` 5 passed; `src/lib.rs` 6 passed; 0 failed in every target. |
| Negative cases hold | ☑ pass | Red runs: part 1 `cargo test --test guard` E0432 on missing `mask`, `mask_for_output`, `KINDS` (c83f667); part 2 `cargo test --no-run` 40 + 2 compile errors on missing `Added.masked`, `Updated` (c655b39). Supervisor mutations on a946a46: drop `mask_note` in `iter_notes` (`src/core/store.rs \| 3 +--`) → `FAILED. 37 passed; 3 failed` (hand_edited_file_is_masked_on_read_and_iter_notes_and_bytes_unchanged, hand_edited_front_matter_is_masked_on_read_and_iter_notes_and_bytes_unchanged, hand_edited_front_matter_with_all_three_places_and_two_notes); skip `guard::mask` on the body in `add` (`src/core/store.rs \| 2 +-`) → `FAILED. 32 passed; 8 failed`; both reverted. QA no-op guard check: 16 of 37 guard tests fail |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed `src/core/guard.rs` (part 1 report and tests), the full `src/core/store.rs` diff (c655b39..a946a46), `src/core/mod.rs`, `Cargo.toml`; tests read for contract only; no surface calls the core yet |
| Full smoke suite still green (no regression) | ☑ pass | cli 5, unit 6, format 13 pass; HUN-269 store tests still pass (30 with the new rm test) |
| **Companion score (easy-verifier)** | ☑ fail | overall 71 (floor missed: 71 < 80), change_risk 100 pass (feature; round 1 failed `new_code_without_test_share` 0.25 > 0.2 on `src/core/format.rs` until the `format.rs`↔`tests/leak.rs` pair was confirmed with ref `tests/leak.rs:1380`), scope changes, ref `main...HEAD` (7b2c3c0), ratings: architecture 100, supply-chain 100, test-strategy 72, requirement-fidelity 67, blast-radius 65, security 59 (`redaction_hits_observed` 64: the guard patterns and runtime-built planted values in tests), code-quality 33 (CCN over 10: `format::parse`, `store` functions); abstentions: solution-fit (no_static_rule), reliability (insufficient_measured_weight), api, data, operations, infrastructure (below_coverage_floor); gates: 7 test pairs confirmed; code-quality and solution-fit gates left abstained (no kit-held evidence to cite); findings sent empty (code-review refs not in the pack form), no divergence; report: `/home/hungnguyenhuu/.easy-verifier-reports/hun-270-guard/evidence-report-changes-20261115T171600-583674Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | No UI: core library task |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | No UI: core library task |
| **UI: Responsiveness at target viewports** | ☑ N/A | No UI: core library task |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (command `date -u; ls src/core/; cargo test 2>&1 | tail -3`, run in the worktree before any implementation commit):

```
Sun Nov 15 02:26:03 PM UTC 2026
guard.rs
mod.rs
For more information about this error, try `rustc --explain E0432`.
error: could not compile `easy-note-mcp` (test "guard") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

**BEFORE (part 3)** (command `date -u; cargo test --test leak 2>&1 | tail -5`, run in the worktree before the fix commit, with the six new part 3 tests added; the folded/next-line and the clean-escape tests were already green because the raw text holds a word boundary there):

```
Sun Nov 15 05:13:47 PM UTC 2026
    escaped_token_inside_a_list_or_map_value_never_leaks

test result: FAILED. 42 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s

error: test failed, to rerun pass `--test leak`
```

Failing: `escape_before_an_anchored_token_in_an_unknown_key_never_leaks`, `a_hex_or_unicode_escape_that_spells_the_token_never_leaks`, `escaped_token_after_a_raw_masked_one_reports_both_kinds_in_order`, `escaped_token_inside_a_list_or_map_value_never_leaks`.

**AFTER** (part 1, same test target): `cargo test --test guard` printed `test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s`.

**AFTER** (part 2, 2026-11-15 15:53 UTC, `cargo test`): `tests/leak.rs` `test result: ok. 40 passed; 0 failed`, `tests/store.rs` `test result: ok. 30 passed; 0 failed`, `tests/guard.rs` `test result: ok. 37 passed; 0 failed`. BEFORE for part 2: those targets did not compile (missing `Added.masked`, `Updated`).

**AFTER (part 3)** (2026-11-15 17:14:28 UTC, `date -u; cargo test --test leak 2>&1 | tail -5`): `test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s`. Full `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`: guard 37, leak 46, store 30, format 13, cli 5, unit 6 passed, 0 failed.

**DELTA**: A secret typed into a note body, tag, context field or unknown front-matter text is now written, read, listed and reported only as `[masked:<kind>]`, and `add` and `update` report the kinds masked; the note is still kept.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]


## Part 1 implementation notes (backend-developer)

- Crate added: `regex` 1.13 (Cargo.lock resolves 1.13.1). `fancy-regex` not used.
- Look-around restated, no `fancy-regex` needed:
  - `openai_key`: regex `sk-(?:proj-)?[A-Za-z0-9_-]{20,}` plus a check that the match does not start with `sk-ant-` (Python: `(?!ant-)`).
  - `password_assign` and `secret_assign`: a regex finds the key (`(?i)(?:words)["']?[ \t]*[=:][ \t]*`); a hand scanner reads the value with the Python alternatives in order (closed quoted, unquoted, unclosed quoted) and stops at a `[masked:` marker. This removes the `(?!\[masked:)` look-aheads.
  - `private_key`: `\Z` becomes `\z`.
- Deviation from the Python oracle: none found. A differential run of Python `guard.mask` (3c5e40f) against the Rust `mask` on 30 edge strings (short and punctuated values, unclosed quotes, markers, unicode, sk-ant boundary, bearer) gave identical `(text, kinds)` for all 30. The scratch files were not committed.
- `guard.rs` holds no logging or printing macro; `guard_source_has_no_logging_or_printing` passes.
- No token literal in any committed file; `git diff | grep -E "xox[abprs]-|ghp_|AKIA|sk-"` shows only pattern text.

## Part 2 implementation notes (backend-developer)

- Changed `src/core/store.rs` only. `add` masks text, tags and fields before the write; `read` and `iter_notes` mask body, tags, context values and `extra` without rewriting files; `update` reads the raw file, applies new text and tags, masks once over the whole note, re-terminates each `extra` chunk, renders, parses back, then writes (error `masking would leave the note unreadable; nothing written` quotes no note text).
- `rm` now opens the moved note read-only for `set_modified` (0444 note fix, HUN-269 Stage 4 finding).
- New crates: none.
- Deviation from the Python oracle: none found. Python checks the size of the masked text; so does this code. Error order in `add`: guard and size check run before the source check, as in Python.
- `store.rs` holds no printing or logging macro; `store_source_has_no_logging_or_printing` passes.
- No token literal in the diff: `git diff | grep -E "xox[abprs]-|ghp_|AKIA|sk-"` printed nothing.

## Part 3 implementation notes (backend-developer)

- Fix for the security-review finding (escape bypass). `src/core/store.rs` `mask_note` keeps the raw-text pass on each unknown key, then a second pass (`mask_decoded`) parses the chunk with saphyr and masks every decoded string, in nested lists and maps. When a decoded string holds a secret, the key is written again as `"<key>": "<masked text>"` (`quote()`), or `"<key>": "[masked:<kind>]"` when the value is a list, a map or not a string. Kinds: raw pass first, then decoded pass (`masked` keeps repeats, one per secret).
- `src/core/format.rs`: `quote` is now `pub(crate)` (one word), no other change.
- Read paths still do not rewrite the file; `update` still parses back before writing.
- New crates: none. Not changed: the `[masked:` skip in `password_assign`/`secret_assign`, any surface.
- Cut list: no re-emission that keeps clean siblings inside a list or map value (the whole value becomes the marker); a chunk with more than one top-level key is not decoded (a chunk is one key by construction in `raw_unknown`).
- Tests added to `tests/leak.rs` (6): `\n`, `\t`, `\r` before each anchored kind (aws, github x2, slack, openai x2, anthropic, jwt, bearer), `\x41` and `\u0041` escapes, folded/next-line values, list and map values, raw plus escaped token in one value (kind order), and an escape-bearing clean key kept byte for byte. The folded/next-line and clean-key tests were green before the fix (the raw text has a word boundary there); the other four were red.
