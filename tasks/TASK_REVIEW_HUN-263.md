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
Checks: tests `cargo fmt --check`, `cargo clippy --all-targets --locked [--features tray] -- -D warnings`, `cargo test --locked [--features tray]` — 392 passed / 0 failed (default), 397 / 0 (tray), `cargo tree -e normal -i ksni` finds nothing in a default build; reviews code-review — 0 P0/P1, 1 P2 (UX not checked: no tray host headless), 2 P3 (`reports/code-review_feat-HUN-263-tray_20261117T104306.html`); real run `easy-note tray` / `install tray` on the default build — refuse with the install command, exit 1, write nothing (auto-checked, not /verify)
Memory: nothing new (this run's learning, `board_sync.py update --guide` pushes its own checkout's guide, is written once on HUN-265's branch)
Not done: merge (admin: after the auto verify pass); auto verify: ☑ pass (session 66d12ff4-b2a1-4ce0-bca6-688a6fb44bc6, commit 3589f2d7ddaf6e03d02318ab600d0a1b8e7431b5, log .claude/verify-runs/HUN-263-3589f2d7ddaf.log); AC 1b and AC 7 (yours); CI step for `--features tray` (follow-up)
Closed: row moved to Done 2026-11-17 — auto verify pass (session 66d12ff4-b2a1-4ce0-bca6-688a6fb44bc6, commit 3589f2d7ddaf6e03d02318ab600d0a1b8e7431b5); Gates 5 and 6 checked

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | AC 2 `src/surfaces/tray.rs` tests `the_menu_lists_the_five_newest_notes_between_the_fixed_items`, `an_empty_store_shows_one_disabled_item`, `a_long_title_is_cut_to_forty_characters_and_a_secret_stays_masked`, `relative_time_steps`, `each_action_is_an_argv_list`, `recent_reads_the_newest_notes_with_their_paths_and_skips_unreadable_files`; AC 3 `tests/tray.rs` (dead bus → exit 1 + hint; doctor line) and `the_doctor_line_names_a_missing_host_only`; AC 4 `tests/cli.rs` default-build `tray`; AC 5 `tests/install.rs` tray install/uninstall + `ac5_a_default_build_refuses_install_tray_and_writes_nothing` + no-autostart-from-other-commands in `tests/cli.rs`; AC 6 `cargo tree` check + source pin in `tests/cli.rs`; AC 8 `a_second_lock_on_the_same_notes_dir_is_refused_until_the_first_ends`; AC 9 `tests/leak.rs` `tray` surface. Default: 392 passed, 0 failed; `--features tray`: 397 passed, 0 failed |
| Verification command run | ☑ pass | Supervisor rerun 2026-11-17T03:40:19Z: `FMT_OK`; clippy default `Finished dev profile` (no warnings); clippy `--features tray` `Finished dev profile` (no warnings); `default: passed 392 failed 0`; `tray: passed 397 failed 0`; `cargo tree -e normal -i ksni` → `error: package ID specification ksni did not match any packages` (no ksni in a default build) |
| Negative cases hold | ☑ pass | Mutation in `install::tray` (`if false && action == Action::Install && !cfg!(feature = "tray")`, `git diff --stat`: `src/surfaces/install.rs \| 2 +-`): `test ac5_a_default_build_refuses_install_tray_and_writes_nothing ... FAILED`, `test result: FAILED. 24 passed; 1 failed`; reverted. Also in the suite: dead bus → exit 1, second lock refused, shell-looking path stays one argv item |
| verify | ☑ pass | auto-run by hook, session 66d12ff4-b2a1-4ce0-bca6-688a6fb44bc6, commit 3589f2d7ddaf6e03d02318ab600d0a1b8e7431b5, log .claude/verify-runs/HUN-263-3589f2d7ddaf.log — pass |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed `tray.rs`, the `install.rs`, `cli.rs`, `cli/read.rs` and `Cargo.toml` diffs and their tests; callers of `launcher` (now via `own_desktop_file`) checked through `tests/install.rs`; `Cargo.lock` read as a crate list only; `src/core/` skipped (unchanged) |
| Full smoke suite still green (no regression) | ☑ pass | Whole `cargo test --locked`, default and `--features tray`: 0 failed (incl. `tests/leak.rs`, `tests/install.rs` launcher tests, doctor tests) |
| **Companion score (easy-verifier)** | ☑ N/A | Not run in this auto run (no companion tool session); the merge gate blocks on ☑ fail only |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | Not checkable here: the tray menu is drawn by the desktop's StatusNotifier host, and Xvfb has none, so no tray was ever shown (no tray is opened on the owner's live desktop in an auto run). The menu model is checked by unit tests (order, labels, 40-char cut, masking). Owner screenshots on GNOME + AppIndicator and KDE are AC 1b (yours) |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | Not checkable here: the tray menu is drawn by the desktop's StatusNotifier host, and Xvfb has none, so no tray was ever shown (no tray is opened on the owner's live desktop in an auto run). The menu model is checked by unit tests (order, labels, 40-char cut, masking). Owner screenshots on GNOME + AppIndicator and KDE are AC 1b (yours) |
| **UI: Responsiveness at target viewports** | ☑ N/A | Not checkable here: the tray menu is drawn by the desktop's StatusNotifier host, and Xvfb has none, so no tray was ever shown (no tray is opened on the owner's live desktop in an auto run). The menu model is checked by unit tests (order, labels, 40-char cut, masking). Owner screenshots on GNOME + AppIndicator and KDE are AC 1b (yours) |

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

**AFTER**: the same commands on the branch, default build, then a `--features tray` build with a dead session bus:

```
Tue Nov 17 03:43:45 UTC 2026
binary: debug build of feat/HUN-263-tray a4dab39 (default features), temp HOME and EASY_NOTE_DIR
$ easy-note tray
easy-note: this build has no tray; install one with: cargo install --path . --locked --features tray
exit=1
$ easy-note install tray
easy-note: this build has no tray; install one with: cargo install --path . --locked --features tray
exit=1
$ easy-note uninstall tray
not installed (/tmp/tmp.dRXsQHCeL7/.config/autostart/easy-note-tray.desktop)
exit=0
files under temp HOME: 0
```

```
Tue Nov 17 03:43:59 UTC 2026
binary: debug build of feat/HUN-263-tray a4dab39 --features tray, temp HOME and EASY_NOTE_DIR, DBUS_SESSION_BUS_ADDRESS=unix:path=<temp>/none
$ easy-note add first note
easy-note: created notes dir /tmp/tmp.B7WzEVL21M/notes
✓ noted 20261117-104359-90fd · easy-note-mcp · HUN-263
exit=0
$ easy-note tray
easy-note: no tray host (D-Bus connection error: Failed to connect to address `unix:path=/tmp/tmp.B7WzEVL21M/none`: No such file or directory (os error 2)); GNOME needs the AppIndicator extension; the one-press path is a hotkey: run easy-note hotkey
exit=1
$ easy-note install tray
✓ wrote /tmp/tmp.B7WzEVL21M/.config/autostart/easy-note-tray.desktop
exit=0
--- /tmp/tmp.B7WzEVL21M/.config/autostart/easy-note-tray.desktop
[Desktop Entry]
Type=Application
Name=easy-note tray
Exec=easy-note tray
Terminal=false
$ easy-note install tray
already installed (/tmp/tmp.B7WzEVL21M/.config/autostart/easy-note-tray.desktop)
exit=0
$ easy-note uninstall tray
✓ removed /tmp/tmp.B7WzEVL21M/.config/autostart/easy-note-tray.desktop
exit=0
files under temp HOME/.config: 0
```

**DELTA**: With a `--features tray` build a person can run `easy-note tray` for a menu of New note, the 5 newest notes and the notes folder, and `install tray` makes it start at login; a default build says how to get one and installs nothing.

**WITNESS**: the Supervisor (session e61f3928), in the HUN-263 worktree at 2026-11-17T03:43:45Z and 03:43:59Z, separately from the implementing agent's runs; trace `memory/event-trace/HUN-263.jsonl`
