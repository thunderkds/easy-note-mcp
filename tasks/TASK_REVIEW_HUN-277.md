# TASK_REVIEW — HUN-277: easy-note setup: launcher, tray and the Obsidian vault in one command

> Sibling of `tasks/TASK_GUIDE_HUN-277.md`. Filled by the reviewer at Stage 4/5.

---

## Auto decisions

Mode: auto
1. Obsidian open → write nothing and ask to close it; `setup` runs launcher, obsidian, tray and never `install claude` — RULE: `docs/ddr/0008-register-the-obsidian-vault.md` § Decision
2. Kept the `dashboard` hint "run easy-note install obsidian" for a missing `.base` file (not changed to `setup`) — FACT: PRD.md:96 "When `<notes dir>/easy-note.base` is missing it says to run `easy-note install obsidian`, exit 1"
3. One P3 left as a suggestion (`setup` picks each step by matching its name string) — RULE: `.claude/skills/code-review/SKILL.md` § Phase 4 "Present P2/P3 fixes as suggestions only."
4. Companion score not run (N/A row) — FACT: `templates/TASK_REVIEW_template.md` Evidence row "the merge gate blocks on ☑ fail only, never on an absent or N/A row" ⚠ decided for you — please review
5. Merge HUN-277 although the owner now wants easy-note's own dashboard window — RULE: owner's answer in chat 2026-11-19 ("Merge it now"); the window is a new task that supersedes DDR 0006
Checks: tests `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked && cargo clippy --all-targets --locked --features tray -- -D warnings && cargo test --locked --features tray` — exit 0, 0 failed (code at 2bd1447); reviews code-review — 0 P0/P1/P2, 1 P3 kept; security-review — no findings (10 candidates rejected, all confidence ≤2); real run `easy-note setup` on the owner's machine via the try script — vault `/home/hungnguyenhuu/easy-notes` added, `easy-note dashboard` opened `easy-note.base` in Obsidian's Bases view (checked in `~/easy-notes/.obsidian/workspace.json`)
Memory: memory/decisions.md — 2026-11-19 owner wants easy-note's own dashboard window (the design's Main board), superseding DDR 0006; HUN-277 kept
Not done: merge (admin: after the auto verify pass); auto verify: running (T182); the own dashboard window (new task, to be planned)

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/setup.rs` (19 tests, AC 2–8, 10, 11), `tests/dashboard.rs` (AC 9 hint), `tests/install.rs` (pin updates), `src/surfaces/install.rs` unit tests (AC 1 path, AC 3 cover by components, AC 4 running check, random id), `src/surfaces/cli.rs` `every_command` (AC 10). All pass in the run below |
| Verification command run | ☑ pass | fmt, clippy (both feature sets), test (both): exit 0 at 2bd1447. Tail:
```
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.16s
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 123.69s
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
test result: ok. 1 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out; finished in 2.05s
test result_does_not_depend_on_the_reader_zone ... ok
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.00s
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
EXIT 0
```
|
| Negative cases hold | ☑ pass | Red run before the code: `tests/setup.rs` 1 passed, 18 failed (`ac2_install_adds_one_vault…`, `ac4_when_obsidian_runs_nothing_is_written…`, `ac6_a_bad_vault_list_is_left_as_it_is_and_named`, `ac8_a_failing_step_does_not_stop_the_next_and_setup_exits_1` …); BEFORE capture below: `setup` → "unrecognized subcommand", exit 2 |
| verify | ☐ pass / ☐ fail / ☐ N/A | |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed: `src/surfaces/install.rs` (`obsidian`, `register_vault`, `Settings::read_with`, `write_backup`), `cli.rs` (`setup`, `installed`), `dashboard.rs` (`run`, `handler`), tests, README. Callers of `install::obsidian` (cli only) checked. Skipped: `src/core/*`, `tray.rs` (untouched). Lane: `C2, 10 files, 1288 lines, EXCEEDS (lines 1288 > 500)` — tests and plan docs; warn only |
| Full smoke suite still green (no regression) | ☑ pass | full `cargo test --locked` and `--features tray`: every `test result: ok`, 0 failed |
| **Companion score (easy-verifier)** | ☑ N/A | not run for HUN-277 (see Auto decision 4) |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | no UI of ours: terminal output only; Obsidian draws the window (guide § UI) |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | no UI of ours |
| **UI: Responsiveness at target viewports** | ☑ N/A | no UI of ours |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured 2026-11-19T08:45:22Z on commit `8047fec` (no implementation commit yet), debug build,
`env -i PATH=$S/bin:/usr/bin:/bin HOME=$S/home XDG_CONFIG_HOME=$S/xdg EASY_NOTE_DIR=$S/home/easy-notes`,
`$S/bin/xdg-mime` prints `obsidian.desktop`, `$S/xdg/obsidian/obsidian.json` holds one vault at the child
`<notes dir>/notes` (the owner's case). `S=/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/271aecd5-0d60-4de1-85a8-6a2749f104f0/scratchpad/before`.

```
$ easy-note setup
error: unrecognized subcommand 'setup'

Usage: easy-note <COMMAND>

For more information, try '--help'.
exit 2
$ easy-note install obsidian
✓ wrote /tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/271aecd5-0d60-4de1-85a8-6a2749f104f0/scratchpad/before/home/easy-notes/easy-note.base
exit 0
$ cat obsidian.json
{"vaults":{"aaaa":{"path":"/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/271aecd5-0d60-4de1-85a8-6a2749f104f0/scratchpad/before/home/easy-notes/notes","ts":1,"open":true}},"frame":"native"}
$ ls -a notes dir
easy-note.base
```

**AFTER**: captured 2026-11-19T10:27:50Z on commit `2bd1447`, debug build with `--features tray`, same sandbox shape as BEFORE
(`S=…/scratchpad/after`, `EASY_NOTE_PROC_DIR=$S/proc` empty, so Obsidian is not running):

```
$ easy-note setup
launcher:
✓ wrote $S/home/.local/share/applications/easy-note.desktop
✓ wrote $S/home/.local/share/icons/hicolor/scalable/apps/easy-note.svg
✓ wrote $S/home/.local/share/icons/hicolor/16x16/apps/easy-note.svg
✓ wrote $S/home/.local/share/icons/hicolor/symbolic/apps/easy-note-symbolic.svg
✓ wrote $S/home/.local/share/icons/hicolor/symbolic/apps/easy-note-saved-symbolic.svg
obsidian:
✓ wrote $S/home/easy-notes/easy-note.base
backup: $S/xdg/obsidian/obsidian.json.easy-note-20261119-172751.bak
✓ added $S/home/easy-notes to Obsidian's vaults
tray:
✓ wrote $S/home/.config/autostart/easy-note-tray.desktop
Using Claude Code? Run easy-note install claude to add the /note hook.
exit 0
$ cat obsidian.json
{"vaults":{"aaaa":{"path":"$S/home/easy-notes/notes","ts":1,"open":true},"2f5a1a03612d73c4":{"path":"$S/home/easy-notes","ts":1795084071384,"open":true}},"frame":"native"}
$ easy-note setup        # second run
obsidian:
already installed ($S/home/easy-notes/easy-note.base)
already an Obsidian vault ($S/home/easy-notes)
tray:
already installed ($S/home/.config/autostart/easy-note-tray.desktop)
exit 0
```

**DELTA**: one `easy-note setup` now makes the notes folder an Obsidian vault (plus launcher, icons and tray), so the launcher opens the dashboard with no clicks inside Obsidian.

**WITNESS**: the Supervisor ran the AFTER capture 2026-11-19T10:27:50Z (`memory/event-trace/HUN-277.jsonl`); the owner ran `easy-note setup` and `easy-note dashboard` on the real machine the same day (vault `/home/hungnguyenhuu/easy-notes` added, backup `obsidian.json.easy-note-20261119-171805.bak`).
