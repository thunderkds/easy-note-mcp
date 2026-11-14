# TASK_REVIEW — HUN-252: Repo skeleton: uv project, src/easy_note/{core,surfaces}, CLI and MCP entry points over one core, CI

> Sibling of `tasks/TASK_GUIDE_HUN-252.md`. Everything here is **filled by the reviewer at Stage
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
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/test_skeleton.py` (7 tests). AC 3 `n`/`easy-note` version: `test_version_flag_prints_package_version`, `test_n_version_matches_easy_note_version`. AC 2 `n` = `add`: `test_n_dispatches_to_add_with_all_arguments`, `test_add_parses_text_words`. AC 5 no `mcp` in CLI: `test_cli_import_does_not_load_mcp`. AC 4 `serve` EOF: `test_serve_exits_cleanly_on_eof[]` and `[\n]`. AC 1, 6, 7 are file/CI facts checked by hand (see below). |
| Verification command run | ☑ pass | `uv run pytest -q` -> `.......  [100%]` / `7 passed in 0.87s`. `uv tool install --force .` -> `Installed 2 executables: easy-note, n`; `easy-note --version` -> `0.1.0`; `n --version` -> `0.1.0`; `uv tool uninstall easy-note-mcp` -> `Uninstalled 2 executables: easy-note, n`. `uvx ruff check --isolated src tests` -> `All checks passed!`. CI workflow (`.github/workflows/ci.yml`) is not run here: it runs on GitHub after push. |
| Negative cases hold | ☑ pass | Mutation: added `import mcp` at top of `src/easy_note/surfaces/cli.py` -> `FAILED tests/test_skeleton.py::test_cli_import_does_not_load_mcp` / `1 failed, 6 passed`; line removed again -> `7 passed`. Also: with `mcp` 2.3.0 resolved, `serve` died with `No module named 'mcp.server.fastmcp'` and the serve test went red, so the dependency is pinned `mcp>=1.2,<2`. |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Stage 4 code-review (Supervisor, 2026-11-14), scope = `git diff cfa3c55..49f199f`: pyproject.toml, src/easy_note/**, tests/test_skeleton.py, .github/workflows/ci.yml, RUNBOOK.md, .gitignore, uv.lock (not read line by line). Findings: P0 0, P1 0, P2 1 (`mcp>=1.2,<2` pin keeps FastMCP v1 API; recorded as a decision, HUN-258 builds on it), P3 1 (comments name plan IDs T008/T009 instead of board IDs HUN-258/HUN-264; mapping is in PROJECT_SPEC.md). lane_check: `C1, 11 files, 245 lines, EXCEEDS (files 11 > 6)` — boilerplate (empty `__init__`, `uv.lock`, `.gitkeep`), label kept at C1. Entry point `easy-note --version` reachable. Startup reads: all 3 required present. |
| Full smoke suite still green (no regression) | ☑ pass | Supervisor re-run in the worktree: `uv run pytest -q` -> `7 passed in 0.98s` (first suite; nothing to regress) |
| **Companion score (easy-verifier)** | ☐ N/A | easy-verifier is not installed in this project (companion-setup not run) |
| **UI: Visual regression (diff or verdict pasted)** | ☐ N/A | Pure-backend task; no UI component (guide's UI section is N/A) |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ N/A | Pure-backend task; no UI component (guide's UI section is N/A) |
| **UI: Responsiveness at target viewports** | ☐ N/A | Pure-backend task; no UI component (guide's UI section is N/A) |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured 2026-11-14T16:42Z in this worktree at commit cfa3c55, before any implementation commit (no `pyproject.toml`, no `src/`, no `tests/` exist):

```
$ uv run pytest -q
no tests ran in 0.05s
(exit=5)
$ easy-note --version
(eval):1: command not found: easy-note
exit=127
$ n --version
(eval):1: command not found: n
exit=127
```

**AFTER**: captured 2026-11-14T16:44Z, same worktree, after the change:

```
$ uv run pytest -q
.......                                                                  [100%]
7 passed in 0.87s
$ uv tool install --force .
...
Installed 2 executables: easy-note, n
$ easy-note --version
0.1.0
$ n --version
0.1.0
$ printf '\n' | easy-note serve ; echo $?     # stderr: pydantic "Invalid JSON" log for the blank line, no traceback
exit 0
$ uv tool uninstall easy-note-mcp
Uninstalled 2 executables: easy-note, n
```

**DELTA**: A person can run `uv tool install .` and get the `easy-note` and `n` commands (both print `0.1.0`), start an MCP stdio server with `easy-note serve`, and run a green test suite locally and in CI.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]
