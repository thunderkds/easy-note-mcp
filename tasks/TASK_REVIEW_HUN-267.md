# TASK_REVIEW — HUN-267: Align context task pattern with PRD FR-005

> Sibling of `tasks/TASK_GUIDE_HUN-267.md`. Everything here is **filled by the reviewer at Stage
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
1. Ran as the minimal track — RULE: `docs/claude-md/tracks.md` `## Tracks` (C0, Risk Low, one file, no user data or format change)
2. Used the PRD FR-005 pattern verbatim, `[A-Z][A-Z0-9]+-\d+` — FACT: PRD.md:66 "`T\d{3,4}`, `[A-Z][A-Z0-9]+-\d+`, case-insensitive for `T`"
3. Kept tracker keys case-sensitive — FACT: PRD.md:66 "case-insensitive for `T`"
4. Left DDR 0002's "draft" wording unchanged — SAFER OPTION: DDRs are owner-approved and its Status is "Proposed (owner to accept)" ⚠ decided for you — please review
Checks: tests `.venv/bin/pytest -q tests` — 205 passed; reviews none (minimal track); real run `context.capture` on a temp repo on `feat/AB2-12-demo` — printed `{'repo': 'rr', 'branch': 'feat/AB2-12-demo', 'task': 'AB2-12'}` (auto-checked, not /verify)
Memory: nothing new (the HUN-254 override entry in memory/decisions.md already records this follow-up)
Not done: merge (admin: after the auto verify pass); auto verify: running (T182)

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/test_context.py::test_task_from_branch` new cases `feat/AB2-12-x` -> `AB2-12`, `fix/PROJ-1234567` -> `PROJ-1234567`, `feat/A-12` -> None. Before the fix: `2 failed, 12 passed`. |
| Verification command run | ☑ pass | 2026-11-15, HEAD 76cf68f: `.venv/bin/pytest -q tests` -> `205 passed in 11.09s`; `uvx ruff check --isolated src/easy_note/core/context.py tests/test_context.py` -> `All checks passed!` |
| Negative cases hold | ☑ pass | The old pattern is the negative case: with it, the two new cases fail (run before the fix, see row 1). `fix/art1234`, `feat/hun-221`, `feat/A-12` still give no task. |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ N/A | Minimal track (`docs/claude-md/tracks.md`): one regex line in `src/easy_note/core/context.py`; Decision record in `tasks/MINIMAL_LOG.md`. |
| Full smoke suite still green (no regression) | ☑ pass | `205 passed in 11.09s` (202 from main + 3 new cases). |
| **Companion score (easy-verifier)** | ☑ N/A | Minimal track: no Stage 4 score step. |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | Backend-only change; no UI. |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | Backend-only change; no UI. |
| **UI: Responsiveness at target viewports** | ☑ N/A | Backend-only change; no UI. |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: [pasted timestamped command output showing the thing absent/failing, captured before the
first implementation commit] OR [verbatim excerpt of the prior content, for non-executable changes]

**AFTER**: [same command, post-change] OR [verbatim excerpt of the new content]

**DELTA**: [one sentence — what a user can now do that they could not before]

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-267.jsonl`, never the
implementing agent alone]
