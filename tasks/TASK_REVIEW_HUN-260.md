# TASK_REVIEW — HUN-260: One press / one click: easy-note quick, hotkey, install launcher (Rust)

> Sibling of `tasks/TASK_GUIDE_HUN-260.md`. Everything here is **filled by the reviewer at Stage
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
1. Picked HUN-260: the first ready ticket in board order at the top Priority (P1), its `Depends on` task HUN-257 Done, not claimed; claimed it on the board (In Progress) and re-read the board before work — RULE: `docs/claude-md/tracks.md` `### Session tags` (the loop, step 1)
2. Re-planned the guide for Rust (Rust paths, cargo verification command, a binding Design list, Xvfb screenshots instead of the owner's desktop) and signed the Requirement Fidelity Gate as Supervisor — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 13)
3. Linux only: dropped `osascript`, macOS/Windows hotkey output, the macOS Shortcuts snippet and non-Linux launchers — FACT: docs/ddr/0005-rust-single-binary.md:29 "2. **Linux only** in v1 (x86_64). macOS and Windows are out of scope; NFR-006 changes to match."
4. Dropped the `tkinter` tier: `zenity` → `kdialog` → terminal prompt; no Rust GUI window in its place — FACT: docs/ddr/0005-rust-single-binary.md:31 "4. **All Python is removed** in the first Rust task"
5. The "clear the context with one key" control is a leading `-` typed in the box (zenity/kdialog entries have no custom key binding); `--cwd` wins over the last context — SAFER OPTION: works the same in every tier, keeps the context by default, Risk Low (decide and log, no Advocates) ⚠ decided for you — please review
6. AC 6 timing and the UI rows are measured by the agent under `xvfb-run` with a stub/real `zenity`; GNOME Wayland timing and look (AC 6b) and the editor snippets tried once (AC 9b) are left to the owner — SAFER OPTION: no dialog or notification is ever opened on the owner's live desktop during an auto run ⚠ decided for you — please review
Checks: pending
Memory: pending
Not done: merge (admin: after the auto verify pass); auto verify: not started; AC 6b and AC 9b (yours)
Paused: auto-mode check failed — extended track, but the trace has no `code-review` Skill call (Stage 4 runs in auto mode) [report efe16998cf08]

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

**BEFORE**: captured on main 3a41088 (`cargo build`, then each command with a temp HOME and EASY_NOTE_DIR, an empty stub-only PATH, `DISPLAY=:99`, `XDG_CURRENT_DESKTOP=GNOME`, stdin `/dev/null`), before any HUN-260 implementation commit:

```
Mon Nov 16 03:08:27 PM UTC 2026
$ easy-note quick
error: unrecognized subcommand 'quick'

Usage: easy-note <COMMAND>

For more information, try '--help'.
exit=2
$ easy-note hotkey
error: unrecognized subcommand 'hotkey'

Usage: easy-note <COMMAND>

For more information, try '--help'.
exit=2
$ easy-note install launcher
error: invalid value 'launcher' for '<TARGET>'
  [possible values: claude]

  tip: a similar value exists: 'claude'

For more information, try '--help'.
exit=2
files under temp HOME: 0
```

**AFTER**: [same command, post-change] OR [verbatim excerpt of the new content]

**DELTA**: [one sentence — what a user can now do that they could not before]

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-260.jsonl`, never the
implementing agent alone]
