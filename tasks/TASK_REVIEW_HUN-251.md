# TASK_REVIEW — HUN-251: Spike: how Claude Code, Codex and Cursor start stdio MCP servers, and what the prompt hook sees

> Sibling of `tasks/TASK_GUIDE_HUN-251.md`. Everything here is **filled by the reviewer at Stage
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
1. Ran easy-verifier on the branch at the user's request and wrote the Companion score row as fail (69, then 64) — RULE: `skills/score/SKILL.md:16` "The row is `pass` only when `overall ≥ 80` and no `change_risk` rule failed"
2. Spawned qa-expert to add `tests/test_T001_probes.py` for the two probe scripts, then rescored once at 1d3d666 — RULE: `skills/score/SKILL.md:70` "Rescore only after the code changed"
3. Left the solution-fit gate abstained; the offered refs are PRD user stories that do not describe a spike — RULE: `skills/score/SKILL.md:52` "A gate you cannot answer from held evidence stays abstained"
4. Ran the missing Risk-Medium `security-review` on the spike diff: pass, P0 0, P1 0 (env-hint logging in `scripts/probe_server.py` is local and throwaway; hardening suggestion only) — RULE: `AGENTS.md` § Permanent Rules "Stage 4 security-review is mandatory for Medium/High risk tasks"
Checks: tests `uv run --with pytest pytest tests -q` — 20 passed; reviews code-review — P0 0 / P1 0 / P2 0 / P3 1, security-review — pass, easy-verifier — fail 64 (test-strategy 45: no test config or CI on a branch cut from main); real run `probe_server.py` over stdio in `tests/test_T001_probes.py` — logged start/initialize/roots (auto-checked, not /verify)
Memory: memory/learnings.md — easy-verifier on a branch cut from main scores test-strategy low until the skeleton (HUN-252) merges
Not done: merge (yours); auto verify: pass; then merge main into this branch (it was cut from main and lacks the kit and its guide) and rescore
Paused: done not moved — the verify result is not confirmed by its run record (the merge gate's rule); Gate 5: no `## Evidence` section; report incomplete: auto verify slot

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/test_T001_fixtures.py` covers AC 5 (each fixture has `prompt`, `cwd`, `session_id`) and success criterion 3 (no `/home/`, no `token`). Run `uv run --with pytest pytest tests/test_T001_fixtures.py -q` -> `15 passed in 0.01s` (2026-11-14T16:47Z). AC 1-4 and 6 are measured findings in `docs/experiments/T001-client-launch.md`, not automated. AC 1 is PARTIAL: the Cursor row is not measured (cursor-agent not signed in; manual steps are in the doc). Follow-up 2026-11-15 (Companion `new_code_without_test_share`): `tests/test_T001_probes.py` runs `scripts/probe_hook.py` and `scripts/probe_server.py` as subprocesses (offline, stdlib + pytest; hook: `/note ` -> block JSON + event file, other prompt -> no output + event file, `EASY_NOTE_HOOK_PREFIX` override, invalid JSON -> no output, no crash; server: `initialize` -> `start`/`initialize`/`roots` log records with cwd, `clientInfo.name`, roots capability and the roots answer, mcp SDK taken from the uv offline cache when the interpreter lacks it, skipped if neither has it). Negative case: renaming the prefix env var in the test to `EASY_NOTE_HOOK_PREFIXX` gave `FAILED tests/test_T001_probes.py::test_hook_prefix_env_var_changes_match - json.decoder.JSONDecodeError` and `1 failed, 19 passed in 2.03s`; restored -> `20 passed in 1.93s`. |
| Verification command run | ☑ pass | `uv run --with pytest pytest tests/test_T001_fixtures.py -q` -> `...............` then `15 passed in 0.01s`. The guide's bare `pytest` ran through `uv run --with pytest` because no project venv exists yet (HUN-252 is parallel). Follow-up 2026-11-15: `uv run --with pytest pytest tests -q` -> `....................` then `20 passed in 1.14s` (15 fixture tests + 5 probe tests). |
| Negative cases hold | ☑ pass | Mutation: a fixture cwd set to `/home/bob/proj`, and `session_id` deleted from another -> `FAILED test_event_has_required_keys[passthrough_main_repo.json]`, `FAILED test_event_is_scrubbed[note_prefix_from_subdir.json]`, `2 failed, 13 passed`. Fixtures regenerated from the recorded events -> `15 passed`. Prior content: `ERROR: file or directory not found: tests/test_T001_fixtures.py` / `no tests ran`. |
| verify | ☑ pass | auto-run by hook, session 2ec73865-e949-442e-9d3a-387a24ed0119, commit 9e46e982dd7936d1a93306a09c7d477e82c6dd39, log .claude/verify-runs/HUN-251-9e46e982dd79.log — pass; done not moved: Gate 5: no `## Evidence` section |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | New files only: `scripts/probe_server.py`, `scripts/probe_hook.py`, `docs/experiments/T001-client-launch.md`, `tests/fixtures/claude_hook/*.json`, `tests/test_T001_fixtures.py`, this file. No `src/`, no existing file edited. Stage 4 code-review (Supervisor, 2026-11-15) over `git diff d33b2a2..d268d82`: P0 0, P1 0, P2 0, P3 1 (scrub test greps only `/home/`; macOS `/Users/` paths would pass — acceptable, fixtures are Linux-recorded). Entry point Standalone — N/A. Startup reads: all 3 present (guide and role guide read from the main checkout). AC 1 accepted as partial by the owner 2026-11-15: Cursor row tracked in HUN-266. Spec follow-up applied: installer honors `CLAUDE_CONFIG_DIR` (FR-019, DDR 0003 item 7, HUN-259 AC 6b). |
| Full smoke suite still green (no regression) | ☑ pass | No other tests exist in the repo (`tests/` did not exist before). The one new test file passes. |
| **Companion score (easy-verifier)** | ☑ fail | overall 67 (floor missed, < 80; architecture 100, code-quality 87, test-strategy 65, supply-chain 55, security 30), change_risk 100 pass (feature, confirmed; test pairs confirmed: `scripts/probe_server.py` and `scripts/probe_hook.py` -> `tests/test_T001_probes.py`), scope changes, ref `main...HEAD` at 98a3604 (after merging main), abstentions: solution-fit (no_static_rule), requirement-fidelity, blast-radius, reliability (insufficient_measured_weight), api, data, operations, infrastructure (below_coverage_floor), gates: solution-fit left abstained, no findings sent. Security 30 is `redaction_hits_observed` 152: all hex hashes (147 in `.claude/harness-lock.json`, a kit file read as repo context; 5 in this file: four sha256 config checksums for AC 6 and one log path), none a secret. Prior runs: 69 at a454ece, 64 at 1d3d666. report: `/home/hungnguyenhuu/.easy-verifier-reports/agent-a2a993b47b37d30d3/evidence-report-changes-20261115T041026-219966Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | Pure backend/doc task, no UI component. |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | No UI component. |
| **UI: Responsiveness at target viewports** | ☑ N/A | No UI component. |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (2026-11-14T16:42:39Z, before any commit; `ls tests` -> `No such file or directory`):

```
$ uv run --with pytest pytest tests/test_T001_fixtures.py -q
ERROR: file or directory not found: tests/test_T001_fixtures.py

no tests ran in 0.00s
```

**BEFORE, follow-up (2026-11-15T02:45:50Z, before the test commit)**: `ls tests/test_T001_probes.py` -> `ls: cannot access 'tests/test_T001_probes.py': No such file or directory`.

**AFTER** (2026-11-14T16:47:34Z):

```
$ uv run --with pytest pytest tests/test_T001_fixtures.py -q
...............                                                          [100%]
15 passed in 0.01s
```

**DELTA**: T004 and T011 can read measured facts (per-client launch cwd and roots, the hook's raw `/note ` and block behavior) from `docs/experiments/T001-client-launch.md`, with six real scrubbed hook events as fixtures. Cursor and the interactive Claude Code rows are still open for the owner.

**WITNESS**: qa-expert agent (not the author of any product code; there is none), session of 2026-11-14 16:42-16:50Z. Clients: Claude Code 2.1.325, codex-cli 0.163.0. The Supervisor re-runs the test through the auto verify run.

### Per-client table (AC 1; full detail in `docs/experiments/T001-client-launch.md`)

| Client | Process cwd | `roots/list` | `clientInfo.name` |
|---|---|---|---|
| Claude Code 2.1.325 | launch folder (main repo, linked worktree, subfolder: each as launched) | declared (`listChanged: true`); one root = the launch folder | `claude-code` |
| Codex 0.163.0 | launch folder (also `$HOME` when launched there) | no capability; request answered with `[]` | `codex-mcp-client` |
| Cursor | NOT MEASURED (cursor-agent not signed in) | NOT MEASURED | NOT MEASURED |

### AC 6: no probe entry left (checksums of the three user files, sha256, before and after the whole run)

```
before (2026-11-14T16:42:36Z)                                  after (2026-11-14T16:47:47Z)
0f5bfd38c49bf976723bdec827dcd83d25337f6d152342fc7e35f381b8323dac  ~/.claude/settings.json   (same)
24a955fbcbcc818064bed71727124338d604442ce43af361cb791d6f438cd28e  ~/.codex/config.toml      (same)
0e1921435953cf506d38ad8639c3a5778fdfc24e14021fba8df87c5c2eda192a  ~/.cursor/mcp.json        (same)
$ diff before after -> no output ("IDENTICAL (3 files)")
```

Also unchanged: `~/.claude-personal/settings.json` (the `CLAUDE_CONFIG_DIR` of this machine) `df59e98ace8ab99c0c8935fb8920f944b95dd30ebfaa5020a41719a95763d87a`; `~/.cursor/cli-config.json` `9a6f8dff461757bb5dde1e57ed77af8fc635716984534cefe9ae6bc57cb64930`; `~/.cursor/hooks.json` `58ea9d7ec2bbfcd5776ef0d7033e52a7f2e9fd33ba69a91144189a2cb1c76507`; `~/.claude.json` `436f42e1a37c904610501e662906ccc3ea6d3e457eca33285ac274a3b6c4a238` (user-scope MCP list: `easy-verifier` only). `grep` for `easy-note-probe|easynoteprobe|probe_server|probe_hook` in the four settings/MCP files -> no match. `~/.claude/commands/note.md` absent. The probe reached each client only through `--mcp-config`/`--settings` temp files (Claude Code), `-c` overrides (Codex), and a project-scope `.cursor/mcp.json` in the temp repo (never used: Cursor did not run).
