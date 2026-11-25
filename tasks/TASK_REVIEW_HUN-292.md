# TASK_REVIEW — HUN-292

> Sibling of `tasks/TASK_GUIDE_T[NNN].md`. Everything here is **filled by the reviewer at Stage
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
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/overrides.rs` (12 tests, AC 1-8 and edge cases), `tests/leak.rs` (new surface `overrides`, all planted secrets); `src/core/ai.rs` unit tests unchanged |
| Verification command run | ☑ pass | `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` -> exit=0, 627 passed, 0 failed (summed over all `test result:` lines), run 2026-11-25 |
| Negative cases hold | ☑ pass | Mutation: in `ai::effective` the override and AI value swapped (`git diff --stat -- src/core/ai.rs` non-empty) -> `tests/overrides.rs`: 10 passed, 2 FAILED (`effective_applies_the_override_...`, `undo_restores_...`); reverted. Pin test `tests/format.rs` note_format_doc... failed on the word `draft` in NOTE_FORMAT.md until narrowed to `**Status**: Draft`. |
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

**BEFORE**: captured 2026-11-25T11:14:28+07:00 on main (74a290b), temp HOME/XDG_CONFIG_HOME/EASY_NOTE_DIR, `target/debug/easy-note add "retry bug on firefox"`, then `ai set <id> topic "firefox flaky"` and `ai undo <id>`:

```
$ easy-note ai set 20261125-111428-dd1e topic "firefox flaky"
error: unrecognized subcommand 'set'
Usage: easy-note ai <COMMAND>
exit=2
$ easy-note ai undo 20261125-111428-dd1e
error: unrecognized subcommand 'undo'
Usage: easy-note ai <COMMAND>
exit=2
```

**AFTER**: 2026-11-25T11:26:26+07:00, same sandbox method, branch feat/HUN-292-ai-overrides:

```
$ easy-note ai set <id> topic "firefox flaky"   -> ✓ topic of <id> set        exit=0
$ easy-note ai set <id> kind rant                -> easy-note: kind must be one of bug, idea, question, todo, draft, other   exit=1
$ easy-note ai undo <id>                         -> ✓ undone: topic of <id> is now no override   exit=0
$ easy-note ai undo <id>                         -> easy-note: nothing to undo   exit=1
ai-log.jsonl:
{"id":"…","field":"topic","old":null,"new":"firefox flaky","who":"person","at":"2026-11-25T11:26:26+07:00"}
{"id":"…","field":"topic","old":"firefox flaky","new":null,"who":"person","at":"2026-11-25T11:26:26+07:00","undo":true}
```

**DELTA**: A person can fix a note's project, topic or kind with `ai set`, the fix wins over the AI, every change is logged, and `ai undo` takes the latest one back.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]
