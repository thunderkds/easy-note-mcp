# TASK_REVIEW — HUN-276: Icons from the design: app icon, launcher opens the dashboard, tray Open dashboard + saved badge

> Sibling of `tasks/TASK_GUIDE_HUN-276.md`. Filled by the reviewer at Stage 4/5.

---

## Auto decisions

Mode: auto
1. Dashboard = an Obsidian Bases file, no window of our own; zenity kept for `quick` — RULE: `docs/ddr/0006-obsidian-as-the-dashboard.md` § Decision (owner's chat answers, 2026-11-19)
2. Task breakdown: 4 tasks (HUN-273 dashboard, HUN-274 AI views, HUN-275 claude enrich, HUN-276 icons+launcher+tray), run in parallel worktrees as the owner asked — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 12)
3. Spawn model from the CLI Model Table — FACT: `PROJECT_SPEC.md:168` "| claude | `claude` | haiku | sonnet | sonnet or opus | opus |"
4. A fourth icon `easy-note-saved-symbolic.svg` carries the badge (icon-name swap, no rasteriser crate) — RULE: `AGENTS.md` § Karpathy Engineering Principles
Checks: pending (Stage 4)
Memory: pending (Stage 5)
Not done: merge (admin: after the auto verify pass); auto verify: not started
Paused: auto-mode check failed — no test run in memory/event-trace/HUN-276.jsonl — write /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID with the task ID (minimal-track step 1): `mkdir -p /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d && printf '%s\n%s\n' "HUN-276" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID`, then rerun the task's tests so the trace records them under the task, then stop; extended track, but the trace has no `code-review` Skill call (Stage 4 runs in auto mode) [report 25aa77f49477]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | tests/install.rs (ac8_install_launcher_writes_exactly_the_launcher_and_its_icons, ac3_the_launcher_text_passes_desktop_file_validate, ac4_the_old_launcher_text_is_upgraded_and_the_change_is_shown, ac2_a_second_install_..., ac2_a_deleted_icon_..., ac2_a_foreign_icon_file_..., ac8_*); src/surfaces/install.rs (each_icon_is_an_svg_with_a_view_box_and_the_design_colours); src/surfaces/tray.rs (menu, argv, icon_choice, badge_state, Badge::poll, watch_stamp). install suite: 29 passed; lib: 45 passed |
| Verification command run | ☑ pass | Full guide command, both feature sets, `EXIT=0`: fmt clean, clippy -D warnings clean (default and `--features tray`), all `test result: ok`, `cargo tree -e normal -i ksni` empty |
| Negative cases hold | ☑ pass | Red run first: before install.rs changed, `ac4_...` and `ac8_install_launcher_writes_exactly_...` failed (left: old text / 1 file; right: new text / 5 files). Foreign launcher and foreign icon: exit 1, snapshot unchanged. Tray tests were written together with their pure functions, so they had no separate red run |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☐ pass / ☐ fail | Stage 4 (Supervisor) |
| Full smoke suite still green (no regression) | ☑ pass | `cargo test --locked` and `--features tray`: no failure (leak suite 46 passed in 129 s) |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☐ N/A | No renderer on this machine (`rsvg-convert`, `inkscape`, `convert`: not found), so no PNG was rendered. The SVGs were not seen as images. Path `d` values of the 4 app-icon paths equal the board (grep count 1 each in the SVG and the board). Owner screenshot still needed. Left open for ux-evaluate / owner |
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
