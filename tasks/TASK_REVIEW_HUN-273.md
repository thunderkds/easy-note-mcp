# TASK_REVIEW — HUN-273: Obsidian dashboard: easy-note dashboard + install/uninstall obsidian

> Sibling of `tasks/TASK_GUIDE_HUN-273.md`. Filled by the reviewer at Stage 4/5.

---

## Auto decisions

Mode: auto
1. Dashboard = an Obsidian Bases file, no window of our own; zenity kept for `quick` — RULE: `docs/ddr/0006-obsidian-as-the-dashboard.md` § Decision (owner's chat answers, 2026-11-19)
2. Task breakdown: 4 tasks (HUN-273 dashboard, HUN-274 AI views, HUN-275 claude enrich, HUN-276 icons+launcher+tray), run in parallel worktrees as the owner asked — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 12)
3. Spawn model from the CLI Model Table — FACT: `PROJECT_SPEC.md:168` "| claude | `claude` | haiku | sonnet | sonnet or opus | opus |"
Checks: pending (Stage 4)
Memory: pending (Stage 5)
Not done: merge (admin: after the auto verify pass); auto verify: not started
Paused: auto-mode check failed — no test run in memory/event-trace/HUN-273.jsonl — write /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID with the task ID (minimal-track step 1): `mkdir -p /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d && printf '%s\n%s\n' "HUN-273" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID`, then rerun the task's tests so the trace records them under the task, then stop; extended track, but the trace has no `code-review` Skill call (Stage 4 runs in auto mode) [report 3044de8cb218]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/dashboard.rs` (AC 3-5, no xdg-open), `tests/install.rs` (`hun273_*`, AC 1, 6, 7), `src/surfaces/dashboard.rs` unit tests (AC 2, 3, 5), `src/surfaces/cli.rs` `every_command` (AC 6). All pass in the run below |
| Verification command run | ☑ pass | fmt, clippy (both feature sets), test (both): exit 0. Tail:
```
finished in 0.61s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.79s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.61s
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.64s
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 167.35s
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.77s
test result: ok. 1 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out; finished in 1.96s
test result_does_not_depend_on_the_reader_zone ... ok
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```
|
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

**BEFORE**: captured before any implementation commit (cargo needs the real RUSTUP_HOME and CARGO_HOME; the run uses a temp HOME and EASY_NOTE_DIR):

```
Thu Nov 19 05:08:35 UTC 2026
$ HOME=<tmp>/home EASY_NOTE_DIR=<tmp>/notes cargo run -q -- dashboard
error: unrecognized subcommand 'dashboard'

Usage: easy-note <COMMAND>

For more information, try '--help'.
exit=2
$ HOME=<tmp>/home EASY_NOTE_DIR=<tmp>/notes cargo run -q -- install obsidian
error: invalid value 'obsidian' for '<TARGET>'
  [possible values: claude, launcher, tray]

For more information, try '--help'.
exit=2
```

**AFTER**: Thu Nov 19 05:20:48 UTC 2026 (stub `xdg-open`/`xdg-mime` on PATH, temp HOME and EASY_NOTE_DIR; the path is the temp dir):

```
$ easy-note dashboard
easy-note: no dashboard yet: run easy-note install obsidian
exit=1
$ easy-note install obsidian
✓ wrote <tmp>/notes/easy-note.base
exit=0
$ easy-note dashboard
xdg-open got: obsidian://open?path=%2Ftmp%2Ftmp.hS6PtI35uB%2Fnotes%2Feasy-note.base
exit=0
$ easy-note install obsidian
already installed (<tmp>/notes/easy-note.base)
exit=0
$ easy-note uninstall obsidian
✓ removed <tmp>/notes/easy-note.base
exit=0
```

**DELTA**: A user can now run `easy-note install obsidian` and `easy-note dashboard` to open the notes dashboard in Obsidian.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]
