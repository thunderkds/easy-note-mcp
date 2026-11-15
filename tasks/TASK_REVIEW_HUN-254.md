# TASK_REVIEW — HUN-254: Context capture

> Sibling of `tasks/TASK_GUIDE_HUN-254.md`. Everything here is **filled by the reviewer at Stage
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
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/test_context.py` (47 tests, real temp git repos; AC 1-9: see the AC map in the final report). `.venv/bin/pytest tests/test_context.py -q` at HEAD 1c95104, 2026-11-15T08:41:01Z: `47 passed in 5.57s` |
| Verification command run | ☑ pass | 2026-11-15T08:40:21Z, HEAD cf4c91c (later commit 1c95104 only added `check=False`, rerun 47 passed): `$ .venv/bin/pytest tests/test_context.py -q` → `47 passed in 5.53s`; full suite `.venv/bin/pytest -q tests` → `146 passed in 16.29s` (99 before + 47 new) |
| Negative cases hold | ☑ pass | Mutation of `src/easy_note/core/context.py` (restored after each; `diff` non-empty), each turned a test red: A repo name from `basename(folder)` instead of main worktree → `test_subfolder_of_repo_gives_the_repo_name` FAILED; B task lookbehind weakened → `test_task_from_branch[fix/art1234-None]` FAILED; C `GIT_TIMEOUT = 10` → `test_slow_git_drops_the_fields_and_add_still_writes` FAILED; D `remote_slug` returns raw url (token leak) → `test_main_repo_uses_folder_name` FAILED; E roots tried before explicit cwd → `test_cwd_order` FAILED; F explicit task dropped in favour of branch → `test_explicit_task_wins` FAILED. Restored file: `47 passed` |
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

**BEFORE**: captured 2026-11-15T08:37:28Z at HEAD 1cc996a (no implementation commit yet), in the worktree:

```
$ .venv/bin/pytest tests/test_context.py -q
ERROR: file or directory not found: tests/test_context.py

no tests ran in 0.00s
$ .venv/bin/python -c "import easy_note.core.context"
    import easy_note.core.context
ModuleNotFoundError: No module named 'easy_note.core.context'
```

**AFTER**: 2026-11-15T08:41:01Z, HEAD 1c95104, same command:

```
$ .venv/bin/pytest tests/test_context.py -q
...............................................                          [100%]
47 passed in 5.57s
```

**DELTA**: `easy_note.core.context.capture(task=, cwd=, roots=)` now returns the repo, remote, branch and task of the folder a note is written from, ready for `store.add(fields=...)`, with no path, no token and no wait over 2 s.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-254.jsonl`, never the
implementing agent alone]
