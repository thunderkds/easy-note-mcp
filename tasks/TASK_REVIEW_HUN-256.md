# TASK_REVIEW — HUN-256: Query engine

> Sibling of `tasks/TASK_GUIDE_HUN-256.md`. Everything here is **filled by the reviewer at Stage
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
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/test_query.py` (56 tests, AC 1–7), fixture `tests/fixtures/notes_basic/` (30 notes, 3 repos, offsets +07:00 and -05:00, 1 unreadable file). Run 2026-11-15T08:54:44Z at HEAD d40b7f8: `56 passed in 9.16s` |
| Verification command run | ☑ pass | 2026-11-15T08:54:44Z, HEAD d40b7f8: `.venv/bin/pytest -q tests/test_query.py` -> `56 passed in 9.16s`; full suite `.venv/bin/pytest -q tests` -> `202 passed in 17.33s`; `ruff check` on the two new files: `All checks passed!` |
| Negative cases hold | ☑ pass | 12 one-line mutations of `src/easy_note/core/query.py`, each `git diff --stat` = `1 file changed` (non-empty), each reverted by `git checkout`; every one turned the suite red: AND to OR (2 failed); local day to UTC day (7 failed); tie order reversed (1 failed); no limit cap (1 failed); `truncated` always false (3 failed); snippet 260 (3 failed); `lower()` for `casefold()` (1 failed); stats no month (1 failed); stats no skipped (2 failed); tag substring match (1 failed); `until` exclusive (2 failed); empty query allowed (1 failed) |
| verify | ☑ pass | auto-run by hook, session a5eb168a-6264-4e86-99bd-8d6a55b743f1, commit 0d8facaa139e626915b544640a709f3777a36c1f, log .claude/verify-runs/HUN-256-0d8facaa139e.log — pass |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Stage 4 code-review (Supervisor, 2026-11-15) over `origin/main...c22deab`: `src/easy_note/core/query.py`, `tests/test_query.py`, `tests/fixtures/notes_basic/`; callers: none yet (entry point `easy_note.core.query`, consumed by HUN-257 CLI and HUN-258 MCP). lane_check: `lane: C1, 33 files, 971 lines, EXCEEDS (files 33 > 6; lines 971 > 250)` (31 are fixture notes; warn-only). P0 0, P1 0, P2 0, P3 1: `_select` in `query.py` has CCN 22 (one long filter condition); readable, but a split would lift code-quality. Cross-task note, not a finding here: `title` and `snippet` come straight from note text; per HUN-255 AC 4 the surfaces (HUN-257/HUN-258) must pass query output through `guard.mask_for_output`. Risk Low, so security-review is not required; checked anyway: read-only scan, no subprocess, no path from input. |
| Full smoke suite still green (no regression) | ☑ pass | Supervisor re-run 2026-11-15: `.venv/bin/pytest -q tests` -> `202 passed in 55.97s` (146 from main + 56 new). |
| **Companion score (easy-verifier)** | ☑ pass | easy-verifier 0.2.0, 2026-11-15T08:58Z: overall 81 (floor 80 met; architecture 100, blast-radius 100, security 100, test-strategy 96 (gate blend, agent 85 conf 0.6), solution-fit 80 (agent-rated, conf 0.5), supply-chain 75, requirement-fidelity 67, code-quality 33), change_risk 100 pass (feature, confirmed; 1 test pair confirmed), scope changes, ref `origin/main...HEAD` at 84c3c94, abstentions: api, data, infrastructure, operations, reliability. First run gave 72: its one redaction hit was this row's template placeholder text, cleared in 84c3c94 before the rescore. Gaps: `max_function_ccn` 22 (`_select`), requirement tracing 0.13, no SBOM CI step, no formatter config. report: `~/.easy-verifier-reports/hun-256-query/evidence-report-changes-20261115T085842-822333Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | Pure backend task; no UI component (guide's UI section is N/A). |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | Pure backend task; no UI component (guide's UI section is N/A). |
| **UI: Responsiveness at target viewports** | ☑ N/A | Pure backend task; no UI component (guide's UI section is N/A). |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured 2026-11-15T08:50:27Z at HEAD 66b6620, before any implementation commit:

```
$ .venv/bin/pytest -q tests/test_query.py
ERROR: file or directory not found: tests/test_query.py
no tests ran in 0.00s
$ .venv/bin/python -c "import easy_note.core.query"
ModuleNotFoundError: No module named 'easy_note.core.query'
```

**AFTER**: captured 2026-11-15T08:54:44Z at HEAD d40b7f8:

```
$ .venv/bin/pytest -q tests/test_query.py
........................................................                 [100%]
56 passed in 9.16s
$ python: query.search("retry firefox", limit=2)  # EASY_NOTE_DIR=tests/fixtures/notes_basic
{"total": 6, "truncated": true, "first": {"id": "20261110-210000-0019", "title": "Retry bug on Firefox #25", "created": "2026-11-10T21:00:00-05:00", "repo": "beta", "task": "T102", "tags": ["flaky", "firefox"], "snippet": "Retry bug on Firefox #25 The retry loop fails only on Firefox after the second navigation."}}
```

**DELTA**: `easy_note.core.query` now lists, searches and counts notes with filters, newest first, at most 50 items with `total` and `truncated`; before, the module did not exist.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-256.jsonl`, never the
implementing agent alone]
