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

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☐ pass / ☐ fail | Part 1 (ACs 1, 2, 4 helper, 6, 7): `tests/guard.rs`. Part 2 (store wiring, ACs 3, 4, 5, 6, 8, 9, 10): `tests/leak.rs` (40 tests) and `tests/store.rs` (30 tests, incl. `rm_moves_a_read_only_note_and_stamps_its_mtime`). All qa-expert oracle, locked, not edited. Output below in the verification row. |
| Verification command run | ☐ pass / ☐ fail | Part 1, run 2026-11-15 14:28:39 UTC in the worktree: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` exit 0. `tests/guard.rs`: `test result: ok. 37 passed; 0 failed`. Other targets: `src/main.rs` 6 passed, `tests/cli.rs` 5 passed. |
| Verification command run (part 2) | ☐ pass / ☐ fail | Run 2026-11-15 15:53:04 UTC after `cargo clean -p easy-note-mcp`, `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` exit 0: `tests/guard.rs` 37 passed; `tests/leak.rs` 40 passed; `tests/store.rs` 30 passed; `tests/format.rs` 13 passed; `tests/cli.rs` 5 passed; `src/lib.rs` 6 passed; 0 failed in every target. |
| Negative cases hold | ☐ pass / ☐ fail | [paste the red run against the prior content, or the mutation diff (`git diff --stat`, non-empty) before each verdict] |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☐ pass / ☐ fail | [what was reviewed vs. skipped, and why] |
| Full smoke suite still green (no regression) | ☐ pass / ☐ fail | |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☐ N/A | [screenshot path or LLM verdict — required for UI tasks, Hard-Stop Gate 6] |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ pass / ☐ fail / ☐ N/A | [method used + output] |
| **UI: Responsiveness at target viewports** | ☐ pass / ☐ fail / ☐ N/A | [viewports tested, any overflow findings] |

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

**AFTER** (part 1, same test target): `cargo test --test guard` printed `test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s`.

**AFTER** (part 2, 2026-11-15 15:53 UTC, `cargo test`): `tests/leak.rs` `test result: ok. 40 passed; 0 failed`, `tests/store.rs` `test result: ok. 30 passed; 0 failed`, `tests/guard.rs` `test result: ok. 37 passed; 0 failed`. BEFORE for part 2: those targets did not compile (missing `Added.masked`, `Updated`).

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
