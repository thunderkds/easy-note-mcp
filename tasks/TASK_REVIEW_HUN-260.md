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
Checks: tests `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` at 9e8df5e — 376 passed, 0 failed (quick 26, hotkey 4, install 23, leak 46); reviews code-review — 0 P0, 0 P1, 1 P2 (silent cancel on a dialog error, follow-up), security-review not required (Risk Low); UX — Xvfb screenshots pass; real run `xvfb-run` real zenity box (agent, killed before save) — showed `Note — proj · T120 (start with - to drop)` (auto-checked, not /verify)
Memory: nothing new (follow-up recorded in Evidence: only exit 1 should be a cancel)
Not done: merge (admin: after the auto verify pass); auto verify: running (T182); AC 6b GNOME Wayland timing and look, AC 9b README snippets tried once (yours); html-report pages not generated
Paused: auto-mode check failed — extended track, but the trace has no `code-review` Skill call (Stage 4 runs in auto mode) [report efe16998cf08]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/quick.rs` (26: AC 1–6, 10), `tests/hotkey.rs` (4: AC 7), `tests/install.rs` (+6 `ac8_*`: AC 8), `tests/leak.rs` `surface_quick` in `SURFACES` (AC 10), unit tests in `src/surfaces/quick.rs` (3). `cargo test`: quick `26 passed; 0 failed`, hotkey `4 passed; 0 failed`, install `23 passed; 0 failed`, leak `46 passed; 0 failed`. AC 6b and AC 9b are the owner's by design. |
| Verification command run | ☑ pass | Supervisor rerun at 9e8df5e: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` — fmt clean, clippy `Finished`, 376 passed, 0 failed (unit 27, cli 43, context 29, format 13, guard 37, hook 14, hotkey 4, inbox 19, install 23, leak 46, mcp 20, query 45, quick 26, store 30). |
| Negative cases hold | ☑ pass | Red before implementation (agent report, `cargo test --no-fail-fast --test quick --test hotkey --test install --test leak`): hotkey `0 passed, 4 failed`; install `17 passed, 6 failed`; leak `45 passed, 1 failed`; quick `1 passed, 25 failed` (`error: unrecognized subcommand 'quick'`). Mutations caught: window 3 h → `ac3_a_note_two_hours_old_or_more_gives_no_context` failed; `-x` as drop marker → `ac3_only_a_lone_dash_drops_the_context_not_dash_words` failed. |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | code-review over `main...HEAD`: `src/surfaces/quick.rs`, `src/surfaces/install.rs`, `src/surfaces/cli.rs`, `src/surfaces/mod.rs`, README.md; callers of `confirmation`/`safe`; tests read for AC coverage. Lane: `C1, 14 files, 1702 lines, EXCEEDS (files 14 > 6; lines 1702 > 250)` (869 test lines, 4 PNGs). Result: 0 P0, 0 P1, 1 P2 (`quick.rs:203` any non-zero dialog exit is a silent cancel, e.g. zenity rejecting the non-ASCII label in a non-UTF-8 locale; follow-up). security-review not required (Risk Low). |
| Full smoke suite still green (no regression) | ☑ pass | Same run: pre-existing binaries keep their counts (cli 43, context 29, format 13, guard 37, hook 14, inbox 19, mcp 20, query 45, store 30); install 17 → 23 and leak 45 → 46 only by new tests. |
| **Companion score (easy-verifier)** | ☑ N/A | easy-verifier not run in this auto run; the merge gate blocks only on ☑ fail |
| **UI: Visual regression (diff or verdict pasted)** | ☑ pass | Real `/usr/bin/zenity` 4.2.1 under `xvfb-run`, LLM-vision read by the Supervisor: `reports/hun-260/quick-cwd-1280x800.png` shows title `easy-note`, text `Note — proj · T120 (start with - to drop)`, one entry row, Cancel/OK, no extra chrome; `quick-nocontext-2560x1440.png` shows `Note` and one entry row. GNOME Wayland look is the owner's (AC 6b). |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ pass | LLM vision on the same PNGs: native GTK dialog, system theme colours and UI font, no hard-coded palette (easy-note sets none; only argv `--title`/`--text`). |
| **UI: Responsiveness at target viewports** | ☑ pass | Xvfb 1280×800 and 2560×1440: the box (~330×260 px at the top-left, no window manager) is fully on screen, readable, not clipped. Mobile/tablet N/A (desktop-only surface). |

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
