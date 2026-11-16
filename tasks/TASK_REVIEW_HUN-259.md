# TASK_REVIEW — HUN-259: Claude Code capture hook (Rust)

> Sibling of `tasks/TASK_GUIDE_HUN-259.md`. Everything here is **filled by the reviewer at Stage
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
1. Picked HUN-259: the first ready ticket in board order at the top Priority (P1), all `Depends on` tasks Done, not claimed; claimed it on the board (In Progress) and re-read the board before work — RULE: `docs/claude-md/tracks.md` `### Session tags` (the loop, step 1)
2. Re-planned the guide for Rust (Rust paths, cargo verification command, AC 4 as a no-store pass path with a timing bound, AC 11 safe refusal of bad settings, a binding Design list) and signed the Requirement Fidelity Gate as Supervisor — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 13)
3. After a prefix match, any error lets the prompt through and prints the error as `reason` with no `decision` key — FACT: PRD.md:78 "On any error it must let the prompt through and print the error as the reason only when the prefix matched."
4. Spawned the Backend-Implementer on `sonnet` (C2 cell "sonnet or opus", Risk Medium) — FACT: PROJECT_SPEC.md:168 "| claude | `claude` | haiku | sonnet | sonnet or opus | opus |"
5. AC 10 (manual smoke in real Claude Code) is left to the owner, not run by the Supervisor or an agent — RULE: tasks/TASK_GUIDE_HUN-259.md § Files Must NOT Touch (the real `~/.claude/` is off limits)
Checks: pending
Memory: pending
Not done: merge (admin: after the auto verify pass); auto verify: not started; AC 10 manual smoke (yours)
Paused: auto-mode check failed — no test run in memory/event-trace/HUN-259.jsonl — write /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID with the task ID (minimal-track step 1): `mkdir -p /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d && printf '%s\n%s\n' "HUN-259" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID`, then rerun the task's tests so the trace records them under the task, then stop [report b0716639a0c8]

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

**BEFORE** (captured 2026-11-16 12:04:46 UTC, base commit cb6507c, `cargo build` then temp HOME and EASY_NOTE_DIR, before any implementation commit):

```
$ date -u
Mon Nov 16 12:04:46 PM UTC 2026
$ printf '%s' '{"prompt":"/note hello","cwd":"<worktree>"}' | target/debug/easy-note hook-prompt; echo exit=$?
error: unrecognized subcommand 'hook-prompt'

Usage: easy-note <COMMAND>

For more information, try '--help'.
exit=2
$ target/debug/easy-note install claude; echo exit=$?
error: unrecognized subcommand 'install'

  tip: a similar subcommand exists: 'init'

Usage: easy-note <COMMAND>

For more information, try '--help'.
exit=2
$ ls -A $HOME      # nothing written
```

**AFTER** (captured 2026-11-16 14:42:23 UTC by `date -u`, same commands, temp HOME and EASY_NOTE_DIR, `<worktree>` = this branch's worktree; the run was made after the change, before the commit):

```
$ date -u
Mon Nov 16 02:42:23 PM UTC 2026
$ printf '%s' '{"prompt":"/note hello","cwd":"<worktree>"}' | target/debug/easy-note hook-prompt; echo exit=$?
{"decision":"block","reason":"✓ noted 20261116-214223-78ec · easy-note-mcp · HUN-259"}
exit=0
$ target/debug/easy-note install claude; echo exit=$?
--- /dev/null
+++ <tmp HOME>/.claude/settings.json
@@ -1,0 +1,15 @@
+{
+  "hooks": {
+    "UserPromptSubmit": [
+      {
+        "hooks": [
+          {
+            "type": "command",
+            "command": "easy-note hook-prompt",
+            "timeout": 5
+          }
+        ]
+      }
+    ]
+  }
+}
✓ added the easy-note hook to <tmp HOME>/.claude/settings.json
✓ wrote <tmp HOME>/.claude/commands/note.md
exit=0
$ target/debug/easy-note install claude; echo exit=$?     # second run
already installed (<tmp HOME>/.claude)
exit=0
```

**DELTA**: In Claude Code, the owner can run `easy-note install claude` once, and then `/note …` saves a note with no model turn (the hook answers `{"decision":"block","reason":"✓ noted <id> · <repo> · <task>"}`), while every other prompt passes untouched in about 2 ms; before, both subcommands did not exist and exited 2.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]
