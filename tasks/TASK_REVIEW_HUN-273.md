# TASK_REVIEW — HUN-273: Obsidian dashboard: easy-note dashboard + install/uninstall obsidian

> Sibling of `tasks/TASK_GUIDE_HUN-273.md`. Filled by the reviewer at Stage 4/5.

---

## Auto decisions

Mode: auto
1. Dashboard = an Obsidian Bases file, no window of our own; zenity kept for `quick` — RULE: `docs/ddr/0006-obsidian-as-the-dashboard.md` § Decision (owner's chat answers, 2026-11-19)
2. Task breakdown: 4 tasks (HUN-273 dashboard, HUN-274 AI views, HUN-275 claude enrich, HUN-276 icons+launcher+tray), run in parallel worktrees as the owner asked — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 12)
3. Spawn model sonnet for C1 — FACT: PROJECT_SPEC.md:168 "| claude | `claude` | haiku | sonnet | sonnet or opus | opus |"
4. Stage 4 P1 fixes applied by the Supervisor: Bases filter `/(^|\/)\d{4}\/\d{2}$/.matches(file.folder)` AND `file.hasProperty("easy-note")`; Today view `file.name.startsWith(today().format("YYYYMMDD"))`; sort by `file.name` DESC — FACT: obsidian.md/help/bases/functions "The String type has no matches() function … `/abc/.matches("abcde")`"; "The date string should be in the format YYYY-MM-DD HH:mm:ss"
Checks: tests `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` + the same with `--features tray` — exit 0, 0 failed (commit d649e8d); reviews code-review — 2 P1 fixed, 1 P2 (sort key undocumented, kept), 1 P3 (install::obsidian reads the "kept " prefix); real run `easy-note dashboard` with stub xdg tools — opened `obsidian://open?path=…easy-note.base` (auto-checked, not /verify)
Memory: pending (Stage 5)
Not done: merge (admin: after the auto verify pass); auto verify: running (T182); owner screenshot of the real Obsidian dashboard (yours); html-report for code-review (not rendered this run)
Paused: done not moved — report incomplete: auto verify slot

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
| Negative cases hold | ☑ pass | Red run against the prior content is the BEFORE capture below: `easy-note dashboard` → "unrecognized subcommand", `install obsidian` → "invalid value 'obsidian'", both exit 2. Foreign `.base` file and missing `.base` file covered by `tests/install.rs` / `tests/dashboard.rs` (exit 1, nothing started). |
| verify | ☑ pass | auto-run by hook, session 7140881f-c2d0-4457-ba7d-85e618f83313, commit 245533888b6d27af869946cce636a59a821d5d0b, log .claude/verify-runs/HUN-273-245533888b6d.log — pass |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed: `src/surfaces/dashboard.rs`, `install.rs` (`own_file` split, `obsidian`), `cli.rs` arms, `mod.rs`, both test files, README section. Callers of `own_desktop_file` (launcher, tray) checked unchanged. Skipped: `src/core/*` (untouched; AC 7 test proves the store ignores the file). Lane: `C1, 7 files, 579 lines, EXCEEDS` (mostly tests; warn only). |
| Full smoke suite still green (no regression) | ☑ pass | full `cargo test --locked` and `--features tray` at d649e8d: every `test result: ok`, 0 failed (leak suite 46 passed in 154.81s) |
| **Companion score (easy-verifier)** | ☑ N/A | not run in this session: the `score` skill was not started for the round-4 tasks (time); the merge gate does not block on N/A |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | Obsidian draws the window (DDR 0006); easy-note owns only the `.base` text. Checked against Obsidian's Bases docs (syntax, functions) and by the parse test; the owner's screenshot of the real window stays an owner check |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | Obsidian's theme draws colours and type; view names and groupings match `docs/design/Main.dc.html` (today / by project) — parse test |
| **UI: Responsiveness at target viewports** | ☑ N/A | desktop app window drawn by Obsidian; easy-note is Linux desktop only (NFR-006) |

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
