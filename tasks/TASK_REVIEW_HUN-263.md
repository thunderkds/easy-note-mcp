# TASK_REVIEW — HUN-263: System tray icon: easy-note tray and install/uninstall tray (Rust, Linux)

> Sibling of `tasks/TASK_GUIDE_HUN-263.md`. Everything here is **filled by the reviewer at Stage
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
1. Took HUN-263 into a parallel run with HUN-262 and HUN-265 (the user's request "spawn agents parallel, supervisor [admin]"), each in its own worktree; HUN-266 left out (Owner manual task) — RULE: `docs/claude-md/pipeline-stages.md` § Stage 3: Parallel Execution via Isolation
2. Re-planned the guide for Rust (`ksni` StatusNotifierItem behind an off-by-default `tray` cargo feature, pure menu/argv/lock functions, temp-HOME install tests) and signed the Requirement Fidelity Gate as Supervisor — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 13)
3. Linux only; pystray, the macOS `open` opener and `assets/icon.png` dropped — FACT: docs/ddr/0005-rust-single-binary.md:29 "2. **Linux only** in v1 (x86_64). macOS and Windows are out of scope; NFR-006 changes to match."
4. No `.github/workflows/ci.yml` change: the `--features tray` clippy/test run is local only; a CI tray step is a follow-up — SAFER OPTION: CI/CD is High risk and outside the slice, Risk Low (decide and log, no Advocates) ⚠ decided for you — please review
5. `install tray` refuses in a default build (exit 1, writes nothing), so no autostart file can start a binary that has no tray — SAFER OPTION: never install a login item that fails at every login ⚠ decided for you — please review
6. Theme icon `accessories-text-editor` until the `DESIGN_PROMPT.md` §3 icons exist; `doctor` reports the tray host only in a tray build — RULE: `AGENTS.md` § Karpathy Engineering Principles (Simplicity First) ⚠ decided for you — please review
7. GNOME/KDE screenshots (AC 1b) and idle RSS (AC 7) left to the owner: Xvfb has no tray host, and no tray is ever opened on the owner's live desktop during an auto run — SAFER OPTION: same rule as HUN-260 decision 6 ⚠ decided for you — please review
Checks: pending
Memory: pending
Not done: merge (admin: after the auto verify pass); auto verify: not started; AC 1b and AC 7 (yours); CI step for `--features tray` (follow-up)

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

**BEFORE**: captured at main 1836e22, before any HUN-263 implementation commit:

```
Tue Nov 17 03:27:39 UTC 2026
binary: release build of main 1836e22, temp HOME and EASY_NOTE_DIR
$ easy-note tray
error: unrecognized subcommand 'tray'

  tip: some similar subcommands exist: 'tag', 'trash'

Usage: easy-note <COMMAND>

For more information, try '--help'.
exit=2
$ easy-note install tray
error: invalid value 'tray' for '<TARGET>'
  [possible values: claude, launcher]

For more information, try '--help'.
exit=2
$ easy-note uninstall tray
error: invalid value 'tray' for '<TARGET>'
  [possible values: claude, launcher]

For more information, try '--help'.
exit=2
files under temp HOME: 0
```

**AFTER**: [same command, post-change] OR [verbatim excerpt of the new content]

**DELTA**: [one sentence — what a user can now do that they could not before]

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-263.jsonl`, never the
implementing agent alone]
