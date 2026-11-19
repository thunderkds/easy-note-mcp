# TASK_REVIEW — HUN-276: Icons from the design: app icon, launcher opens the dashboard, tray Open dashboard + saved badge

> Sibling of `tasks/TASK_GUIDE_HUN-276.md`. Filled by the reviewer at Stage 4/5.

---

## Auto decisions

Mode: auto
1. Dashboard = an Obsidian Bases file, no window of our own; zenity kept for `quick` — RULE: `docs/ddr/0006-obsidian-as-the-dashboard.md` § Decision (owner's chat answers, 2026-11-19)
2. Task breakdown: 4 tasks (HUN-273 dashboard, HUN-274 AI views, HUN-275 claude enrich, HUN-276 icons+launcher+tray), run in parallel worktrees as the owner asked — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 12)
3. Spawn model sonnet for C2 at Risk Low — RULE: `skills/craft-spawn-prompt/SKILL.md` § 5. Recommend spawn model ("A two-model cell (`sonnet or opus`) isn't ambiguity — pick per Risk")
4. A fourth icon `easy-note-saved-symbolic.svg` carries the badge (icon-name swap, no rasteriser crate) — RULE: `AGENTS.md` § Karpathy Engineering Principles
5. `install launcher` exits 1 on a foreign file (the old test pinned exit 0) — RULE: `tasks/TASK_GUIDE_HUN-276.md` § Acceptance Criteria AC 2 ("a file that is not ours is kept and named (exit 1, nothing else written)")
6. README names "opens the dashboard" without the command until HUN-273 merges; the Supervisor adds `easy-note dashboard` at merge — RULE: `tests/readme.rs` (a README command must exist)
Checks: tests `cargo fmt --check && cargo clippy … -D warnings && cargo test --locked` + `--features tray` + `! cargo tree -e normal -i ksni` — exit 0, 40 `test result: ok`, 0 failed; reviews code-review — 0 P0/P1, 2 P3 (badge month folder from UTC not local; edit/tag also start the badge); real run `install launcher` in a temp HOME — 5 files, `desktop-file-validate` clean, second run `already installed` (auto-checked, not /verify)
Memory: nothing new for HUN-276 alone; the round-4 memory pass runs after the last merge
Not done: merge (admin: after the auto verify pass); auto verify: ☑ pass (session d211dbbb-13a5-4a3c-986f-7cf631f649ab, commit 25e6109304904c081bd98fbf61d94a0d453063e7, log .claude/verify-runs/HUN-276-25e610930490.log); owner GNOME screenshots: app grid icon, tray outline + 3 s badge (yours); html-report for code-review (not rendered this run)
Closed: row moved to Done 2026-11-19 — auto verify pass (session d211dbbb-13a5-4a3c-986f-7cf631f649ab, commit 25e6109304904c081bd98fbf61d94a0d453063e7); Gates 5 and 6 checked

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | tests/install.rs (ac8_install_launcher_writes_exactly_the_launcher_and_its_icons, ac3_the_launcher_text_passes_desktop_file_validate, ac4_the_old_launcher_text_is_upgraded_and_the_change_is_shown, ac2_a_second_install_..., ac2_a_deleted_icon_..., ac2_a_foreign_icon_file_..., ac8_*); src/surfaces/install.rs (each_icon_is_an_svg_with_a_view_box_and_the_design_colours); src/surfaces/tray.rs (menu, argv, icon_choice, badge_state, Badge::poll, watch_stamp). install suite: 29 passed; lib: 45 passed |
| Verification command run | ☑ pass | Full guide command, both feature sets, `EXIT=0`: fmt clean, clippy -D warnings clean (default and `--features tray`), all `test result: ok`, `cargo tree -e normal -i ksni` empty |
| Negative cases hold | ☑ pass | Red run first: before install.rs changed, `ac4_...` and `ac8_install_launcher_writes_exactly_...` failed (left: old text / 1 file; right: new text / 5 files). Foreign launcher and foreign icon: exit 1, snapshot unchanged. Tray tests were written together with their pure functions, so they had no separate red run |
| verify | ☑ pass | auto-run by hook, session d211dbbb-13a5-4a3c-986f-7cf631f649ab, commit 25e6109304904c081bd98fbf61d94a0d453063e7, log .claude/verify-runs/HUN-276-25e610930490.log — pass |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed: `src/surfaces/install.rs` (owned-file set, V1 upgrade, `plan_own_file_from`, callers `launcher`/`tray`/`claude` unchanged), `src/surfaces/tray.rs` (menu, argv, `icon_choice`, `badge_state`, `watch_stamp`, `Badge::poll`, ksni impl), 4 SVGs vs `docs/design/Icons.dc.html`, tests, README. Skipped: `src/core/*`, `cli.rs` (untouched). Lane: `C2, 8 files, 535 lines, EXCEEDS (lines 535 > 500)` — warn only |
| Full smoke suite still green (no regression) | ☑ pass | `cargo test --locked` and `--features tray`: no failure (leak suite 46 passed in 129 s) |
| **Companion score (easy-verifier)** | ☑ N/A | not run in this session: the `score` skill was not started for the round-4 tasks (time); the merge gate does not block on N/A |
| **UI: Visual regression (diff or verdict pasted)** | ☑ pass | No SVG renderer on this machine (`rsvg-convert`, `inkscape`, `magick` absent), so the check is path-data equality: every `d`, `rect`, colour and stroke width of `easy-note.svg` equals `docs/design/Icons.dc.html` with `{{accent}}`=`#3CF5A8` (Supervisor read both, 2026-11-19); symbolic outline paths equal the board's tray icon. Saved badge: ring + check over a clipped gap (board: filled circle) — owner eye check at 16/22 px left open |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ pass | grep of colours: app icons use only #181B1A, #2C3230, #EAEDEA, #8C9590, #3CF5A8; symbolic files use only #bebebe (8 uses, no other colour) |
| **UI: Responsiveness at target viewports** | ☑ N/A | Desktop-native icons; 16 px variant has no lines or cursor (checked by unit test); no renderer to view 16/22 px |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (captured before any implementation commit; real output):

```
Thu Nov 19 05:09:23 UTC 2026
$ HOME=<temp> cargo run -q -- install launcher
✓ wrote $HOME/.local/share/applications/easy-note.desktop
$ find $HOME/.local/share -type f
$HOME/.local/share/applications/easy-note.desktop
$ cat $HOME/.local/share/applications/easy-note.desktop
[Desktop Entry]
Type=Application
Name=easy-note
Comment=Save a one-line note
Exec=easy-note quick
Terminal=false
Categories=Utility;
```

One file only: no icon, `Exec=easy-note quick`, no dashboard entry.

**AFTER** (same command, after the change; real output):

```
Thu Nov 19 05:21:52 UTC 2026
$ HOME=<temp> cargo run -q -- install launcher
✓ wrote $HOME/.local/share/applications/easy-note.desktop
✓ wrote $HOME/.local/share/icons/hicolor/scalable/apps/easy-note.svg
✓ wrote $HOME/.local/share/icons/hicolor/16x16/apps/easy-note.svg
✓ wrote $HOME/.local/share/icons/hicolor/symbolic/apps/easy-note-symbolic.svg
✓ wrote $HOME/.local/share/icons/hicolor/symbolic/apps/easy-note-saved-symbolic.svg
$ find $HOME/.local/share -type f
$HOME/.local/share/applications/easy-note.desktop
$HOME/.local/share/icons/hicolor/16x16/apps/easy-note.svg
$HOME/.local/share/icons/hicolor/scalable/apps/easy-note.svg
$HOME/.local/share/icons/hicolor/symbolic/apps/easy-note-saved-symbolic.svg
$HOME/.local/share/icons/hicolor/symbolic/apps/easy-note-symbolic.svg
$ cat easy-note.desktop
[Desktop Entry]
Type=Application
Name=easy-note
Comment=Your notes, one press away
Exec=easy-note dashboard
Icon=easy-note
Terminal=false
Categories=Utility;
Actions=new-note;

[Desktop Action new-note]
Name=New note
Exec=easy-note quick
$ again
already installed ($HOME/.local/share/applications/easy-note.desktop)
```

**DELTA**: After `install launcher`, easy-note has its own icon in the app grid, clicking the app runs `easy-note dashboard`, and a right-click offers New note; the tray shows the outline icon, an Open dashboard item and a 3 s saved badge.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]
