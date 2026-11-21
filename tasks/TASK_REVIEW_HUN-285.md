# TASK_REVIEW — HUN-285: Tray: starts at login and with the app, menu texts from the design

> Sibling of `tasks/TASK_GUIDE_HUN-285.md`. Everything here is **filled by the reviewer at Stage
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
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `src/surfaces/tray.rs` tests: `the_wait_returns_ready_after_the_fourth_check_when_the_host_appears`, `the_wait_gives_up_after_sixty_checks_with_no_host`, `a_host_present_at_start_needs_one_check_and_no_sleep`, `the_tray_start_argv_is_this_exe_and_tray`, repointed `the_menu_lists_the_five_newest_notes_between_the_fixed_items`, `an_empty_store_shows_one_disabled_item`. Supervisor rerun 2026-11-21: `cargo test --locked --features window,tray --lib tray` → `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 53 filtered out` |
| Verification command run | ☐ pass / ☐ fail | [paste actual output] |
| Negative cases hold | ☐ pass / ☐ fail | [paste the red run against the prior content, or the mutation diff (`git diff --stat`, non-empty) before each verdict] |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed `src/surfaces/tray.rs`, `src/surfaces/dashboard.rs`, `src/surfaces/window/mod.rs` (`open` callers: `dashboard::run` only), `README.md`. lane_check: `lane: C1, 5 files, 366 lines, EXCEEDS (lines 366 > 250); unpredicted: src/surfaces/window/mod.rs, tasks/TASK_GUIDE_HUN-285.md` — the guide file is 209 of those lines; code change ~150 lines; `window/mod.rs` deviation accepted (start must not run on the NO_DISPLAY fallback, AC 4) |
| Full smoke suite still green (no regression) | ☐ pass / ☐ fail | |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☐ N/A | Labels and order pinned by the two repointed `tray.rs` menu tests. OS screenshot of the open menu: pending the owner's GNOME session (the menu opens only on a click on the panel) |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | The panel draws the menu; `docs/design/TrayMenu.dc.html` says its colours follow the desktop theme. Ours are the labels and order only (covered by the visual-regression row) |
| **UI: Responsiveness at target viewports** | ☑ N/A | A panel menu has no viewport; the desktop draws it |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (captured 2026-11-21T18:12:02+07:00, commit acd0722, before any implementation commit):

```
$ grep -n 'sleep\|host_present\|"New note"\|no notes yet' src/surfaces/tray.rs
66:        Entry::Item("New note".to_string(), Action::NewNote),
71:        Entry::Disabled("no notes yet".to_string()),
233:pub fn doctor_line(host_present: bool) -> Option<String> {
390:    pub fn host_present() -> bool {
$ cargo test --locked --features tray --lib surfaces::tray
test surfaces::tray::tests::an_empty_store_shows_one_disabled_item ... ok      (pins "no notes yet")
test surfaces::tray::tests::the_menu_lists_the_five_newest_notes_between_the_fixed_items ... ok  (pins "New note")
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out
```

`tray::run` has no wait: `tray.spawn()` is called once (tray.rs:365-367) and its error returns `no tray host (...)`; `host_present` is not called in `run`. `grep sleep` finds no match. `dashboard::run` has no tray start (no `tray` in src/surfaces/dashboard.rs). Today's labels: row 1 `New note`, empty row `no notes yet`.

**AFTER** (2026-11-21, agent run, before commit): the guide's full Verification Command exited 0 (fmt, clippy -D warnings and `cargo test --locked` in the three feature sets: default, `tray`, `window,tray`; every `test result` line `ok`, 0 failed). New unit tests in `src/surfaces/tray.rs`: `the_wait_returns_ready_after_the_fourth_check_when_the_host_appears`, `the_wait_gives_up_after_sixty_checks_with_no_host`, `a_host_present_at_start_needs_one_check_and_no_sleep`, `the_tray_start_argv_is_this_exe_and_tray`; label tests repointed to `New note · Ctrl+Alt+N` and `No notes yet. Your first one appears here.`. Red run before the code: `error[E0425]: cannot find function \`wait_for_host\`` (x3) and `tray_start_argv`.

**DELTA**: [one sentence — what a user can now do that they could not before]

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]

## Stage 4 — code-review (Supervisor, 2026-11-21)

Scope: `main...feat/HUN-285-tray`. Personas: correctness, testing, maintainability, standards, over-engineering (new functions). Security / performance / migration / api: not activated (no input handling, no shell, no queries; argv lists only). ux-evaluate: menu is desktop-native, owner screenshot pending.
Entry points `tray::run`, `dashboard::run`, `tray::menu`: found. Startup reads: PROJECT_SPEC.md, the guide, and `agents/backend.md` (loaded as the agent's system prompt) — Permanent-Rule reads met.

- P0: None.
- P1: None.
- P2: None.
- P3: `README.md:256,262` — two new lines run past the file's ~100-column wrap. Optional reflow.
- P3: process — the agent did not read PRD FR-033/FR-039 (guide step 7); no effect found on the diff.
- Supervisor edit (guide gate): `PRD.md` FR-033 gets the 60 s wait and the start from the dashboard window.

Stale-reference sweep: `no notes yet` remains only in `docs/design/TrayMenu.dc.html` (design artboard label, owner-owned, not a menu text).
Verdict: no P0/P1; 0 safe fixes needed.
