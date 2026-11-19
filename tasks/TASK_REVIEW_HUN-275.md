# TASK_REVIEW — HUN-275: AI detail from the local claude CLI: ai enrich + [ai] settings

> Sibling of `tasks/TASK_GUIDE_HUN-275.md`. Filled by the reviewer at Stage 4/5.

---

## Auto decisions

Mode: auto
1. Dashboard = an Obsidian Bases file, no window of our own; zenity kept for `quick` — RULE: `docs/ddr/0006-obsidian-as-the-dashboard.md` § Decision (owner's chat answers, 2026-11-19)
2. Task breakdown: 4 tasks (HUN-273 dashboard, HUN-274 AI views, HUN-275 claude enrich, HUN-276 icons+launcher+tray), run in parallel worktrees as the owner asked — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 12)
3. Spawn model from the CLI Model Table — FACT: `PROJECT_SPEC.md:168` "| claude | `claude` | haiku | sonnet | sonnet or opus | opus |"
4. `claude -p` receives the masked note text only; `after_save` kept, default false — DEBATED: text only + after_save default off (Advocate 1: text plus repo/branch/task/created; Advocate 2: text only, after_save default off; Advocate 3: text only, no after_save) — settled by RULE: `PRD.md` § Non-Functional Requirements NFR-001 and § Functional Requirements FR-038 ⚠ decided for you — please review
Checks: pending (Stage 4)
Memory: pending (Stage 5)
Not done: merge (admin: after the auto verify pass); auto verify: not started
Paused: auto-mode check failed — no test run in memory/event-trace/HUN-275.jsonl — write /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID with the task ID (minimal-track step 1): `mkdir -p /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d && printf '%s\n%s\n' "HUN-275" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID`, then rerun the task's tests so the trace records them under the task, then stop; Risk High: decision 3 (`FACT`) is a choice — its FACT has no Citation (`path:line "quoted text"`) — so it must be `DEBATED` with exactly 3 Advocates named; extended track, but the trace has no `code-review` Skill call (Stage 4 runs in auto mode); Risk High, but the trace has no `security-review` Skill call [report cd980ec7422f]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☐ pass / ☐ fail | [test file path(s) — required before Done] |
| Verification command run | ☐ pass / ☐ fail | [paste actual output] |
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

**BEFORE**: captured on branch `feat/HUN-275-claude-enrich` at `2be87be`, before any implementation commit, with a temp `HOME` and `EASY_NOTE_DIR` (`$S` = the agent's scratchpad; `RUSTUP_HOME`/`CARGO_HOME` pinned so cargo runs under the temp `HOME`):

```
$ date -u
Thu Nov 19 05:11:56 UTC 2026
$ HOME=$S/before/home EASY_NOTE_DIR=$S/before/notes cargo run -q -- ai enrich --pending; echo "exit $?"
error: unrecognized subcommand 'enrich'

Usage: easy-note ai <COMMAND>

For more information, try '--help'.
exit 2
$ HOME=$S/before/home EASY_NOTE_DIR=$S/before/notes cargo run -q -- doctor; echo "exit $?"
notes dir: /tmp/claude-1000/-home-thunderkds-workspace-pets-hungnguyen111-easy-note-mcp/787574a0-3c07-428c-b66c-57bbc928438d/scratchpad/before/notes
0 notes checked
no problems found
exit 0
```

**AFTER**: [same command, post-change] OR [verbatim excerpt of the new content]

**DELTA**: [one sentence — what a user can now do that they could not before]

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]
