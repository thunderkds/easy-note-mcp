**Date**: 2026-11-19
**Complexity Level**: C2
**Risk Level**: Low
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*)
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read `agents/backend.md`
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in `agents/backend.md`
6. C2: read `memory/codebase-map.md` if it exists; read `docs/design/README.md`, `docs/design/Icons.dc.html`, `docs/design/TrayMenu.dc.html`, `src/surfaces/install.rs`, `src/surfaces/tray.rs`
7. Tests only: temp `HOME` and `EASY_NOTE_DIR`; never open the real `~/.local/share/`, `~/.config/autostart/` or `~/easy-notes/`; no test spawns a real tray

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-19: "refer to this design, and update the application …, adding the icon, application can open the dashboard, not only the command, we should support the native way." Design boards: `docs/design/Icons.dc.html` (app icon, tray icon, "just saved" badge) and `docs/design/TrayMenu.dc.html` (menu order). The dashboard is opened by `easy-note dashboard` (FR-034, parallel task; this task only runs it as a subprocess).

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> easy-note looks like an installed desktop app: its own icon in the app grid, dock and tray; clicking the app opens the dashboard; "New note" stays one click away from the launcher and the tray.

**Out of scope** (what this task explicitly does NOT do):

* The `dashboard` command itself (FR-034 task)
* PNG rasterising (SVG in hicolor is enough for GNOME/KDE)
* Replacing the zenity capture box (DDR 0006 item 4)
* macOS / Windows icons

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-039: icon files, launcher `Icon=`/`Exec=`/New note action, tray icon, Open dashboard, saved badge
* FR-026: `install launcher` / `uninstall launcher` exact files (now the launcher plus its icons)
* FR-033: tray menu (adds Open dashboard)
* NFR-003: writes only the files `install launcher` owns
* US-014, US-015, US-017

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent) — auto (Supervisor) 2026-11-19
- [X] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy) — auto (Supervisor) 2026-11-19
- [X] Every Acceptance Criterion below traces to a line in the Requirement — auto (Supervisor) 2026-11-19
- [X] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above — auto (Supervisor) 2026-11-19

---

## Dependencies & Reachability

**Depends on**: None (runs `easy-note dashboard` by argv; the command lands in the parallel FR-034 task)

**Entry point**: `install launcher` (`install::launcher`); `Open dashboard` (tray menu item, `Action::OpenDashboard` in `src/surfaces/tray.rs`)

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | Four SVG files in `assets/icons/`, drawn exactly from `docs/design/Icons.dc.html` with `{{accent}}` = `#3CF5A8`: `easy-note.svg` (64-unit app icon), `easy-note-16.svg` (16 px variant, no text lines or cursor), `easy-note-symbolic.svg` (tray outline, `fill`/`stroke` `#bebebe` per the freedesktop symbolic convention), `easy-note-saved-symbolic.svg` (outline + check badge). Embedded with `include_str!`. A test checks each parses as XML with an `<svg` root and a `viewBox` | FR-039 |
| 2 | `install launcher` writes exactly: `~/.local/share/applications/easy-note.desktop`, `~/.local/share/icons/hicolor/scalable/apps/easy-note.svg`, `…/hicolor/16x16/apps/easy-note.svg` (the 16 variant), `…/hicolor/symbolic/apps/easy-note-symbolic.svg`, `…/hicolor/symbolic/apps/easy-note-saved-symbolic.svg`. Same DDR 0003 rules per file: a second run writes nothing and says `already installed`; a file that is not ours is kept and named (exit 1, nothing else written); `uninstall launcher` removes only byte-identical files of ours. fs snapshot: nothing else changes | FR-039, FR-026, NFR-003 |
| 3 | The launcher text: `Type=Application`, `Name=easy-note`, `Comment=Your notes, one press away`, `Exec=easy-note dashboard`, `Icon=easy-note`, `Terminal=false`, `Categories=Utility;`, `Actions=new-note;`, and `[Desktop Action new-note]` `Name=New note`, `Exec=easy-note quick`. Passes `desktop-file-validate` when that tool exists (test skips with a note otherwise) | FR-039 |
| 4 | An install made by the old launcher text (`Exec=easy-note quick`, no icons) is upgraded by `install launcher`: the old byte-identical file is replaced with the new one (recognised as ours by an exact match with the previous constant), and the change is shown | FR-026 (upgrade) |
| 5 | Tray menu (pure `tray::menu`): `New note`, `Open dashboard`, separator, recent 5 (or `no notes yet`), separator, `Open notes folder`, `Quit`, as `docs/design/TrayMenu.dc.html`. `argv(Action::OpenDashboard, exe, root)` = `(exe, ["dashboard"])`. The existing menu tests are updated, not deleted | FR-039, FR-033 |
| 6 | Tray icon: `icon_name` = `easy-note-symbolic` and `icon_theme_path` = `~/.local/share/icons` when that symbolic file exists there, else the old `accessories-text-editor` (pure `icon_choice(installed: bool, saved: bool) -> &str`, unit-tested) | FR-039 |
| 7 | Saved badge: the tray's 1 s poll compares the modified time of `<notes dir>/<YYYY>/<MM>` (this month) and `<notes dir>/inbox`; on a change it shows `easy-note-saved-symbolic` for 3 s, then back. No note listing on the poll (one or two `stat` calls). Pure `badge_state(last_change, now) -> bool` unit-tested at 0, 2.9, 3.1 s | FR-039, NFR-004 |
| 8 | Default build (no `tray` feature) links no new crate; `cargo tree -e normal -i ksni` still empty; `install launcher` works in a default build | NFR-010 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | temp HOME, `install launcher` ×2, `uninstall launcher` | 5 files, `already installed`, all 5 removed | `tests/install.rs` |
| 2 | foreign file at the scalable icon path | kept, named, exit 1, no other file written | `tests/install.rs` |
| 3 | old launcher text present | replaced by the new text | `tests/install.rs` |
| 4 | `menu(recent, now)` | New note, Open dashboard, sep, … | `src/surfaces/tray.rs` unit test |
| 5 | `badge_state` at 0 / 2.9 / 3.1 s | true / true / false | unit test |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features tray -- -D warnings \
  && cargo test --locked --features tray \
  && ! cargo tree -e normal -i ksni --locked 2>/dev/null | grep -q ksni
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-276.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-276.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

> Desktop-native surfaces (icons, launcher, tray menu). Design source: `docs/design/Icons.dc.html`, `docs/design/TrayMenu.dc.html`. The tray needs a StatusNotifierItem host, so the live screenshots are the owner's; the agent renders each SVG to PNG for review if `rsvg-convert` or `inkscape` exists, and judges them with `Skill({ skill: "ux-evaluate" })` against the design board.

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| 4 SVG icons | render to PNG at 16, 22, 48, 128 px (if a renderer exists), LLM vision vs `docs/design/Icons.dc.html` | same shapes: sheet, folded corner, two lines, neon fold and cursor on the app icon |
| App grid + tray on GNOME | owner screenshot | easy-note icon in the grid; tray shows the outline; badge after a save |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Colors match design tokens | grep the SVGs | app icon: `#181B1A`, `#2C3230`, `#EAEDEA`, `#8C9590`, `#3CF5A8` (`docs/design/graphite-mint-tokens.json`); symbolic: `#bebebe` only |
| Typography matches spec | N/A | N/A — icons hold no text |
| Spacing / layout matches spec | diff of path data vs the design board | path `d` values equal the board's |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| Mobile (320–480px) | N/A | N/A — desktop-only surface |
| Tablet (768px) | N/A | N/A — desktop-only surface |
| Desktop (1024px+) | rendered PNGs at 16 and 22 px | the sheet stays sharp; 16 px drops the lines (design note) |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-017 | open the dashboard from the app grid | click easy-note in the app grid | `easy-note dashboard` runs (launcher `Exec=`) |
| US-014 | new note from the launcher | right-click the app → New note | `easy-note quick` runs |
| US-015 | see a save happened | save a note while the tray runs | badge icon for 3 s, then the plain outline |

---

## Approach

**Pattern reference**: `src/surfaces/install.rs` — `own_desktop_file` / exact-file install rules; `src/surfaces/tray.rs` — pure menu/argv functions and the `#[cfg(feature = "tray")]` split; `tests/install.rs` — fs snapshot tests.

**Vital slice**: the app icon + launcher pointing at the dashboard (AC 1–3); the tray item and badge are small.

* Generalise `own_desktop_file` into an owned-file-set install (all files checked first, then written), so a foreign file means nothing is written.
* Keep the old launcher constant as `LAUNCHER_DESKTOP_V1` only for the upgrade match (AC 4).

**Cut list**: PNG icons; `gtk-update-icon-cache`; a "saved" pulse animation; icon in the `quick` zenity box (`--window-icon` is a follow-up).

---

## Edge Case Checklist

- [ ] `~/.local/share/icons/hicolor/…` dirs missing → created
- [ ] Person deleted one icon by hand → `install launcher` writes the missing one, keeps the rest
- [ ] Notes dir month folder missing (no note this month) → badge poll treats "missing" as no change
- [ ] Clock change / mtime in the future → badge shows at most 3 s
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README and docs for `Exec=easy-note quick`, `accessories-text-editor`, "The theme icon until the designer's set exists"

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `assets/icons/*.svg` (new, 4 files) | icons from the design |
| `src/surfaces/install.rs` | launcher text, icon files, owned-file-set install, V1 upgrade |
| `src/surfaces/tray.rs` | `Action::OpenDashboard`, menu order, icon choice, badge poll |
| `tests/install.rs`, `tests/tray.rs` | AC tests |
| `README.md` | launcher and tray sections |
| Pre-existing pin tests | `tests/install.rs` launcher block, `src/surfaces/tray.rs` menu tests, `tests/readme.rs` — list each and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*`, `docs/design/*` | Supervisor-owned / design source |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| `src/surfaces/cli.rs` `Command`/`Target` enums, `src/surfaces/dashboard.rs`, `src/core/*` | Parallel tasks own them; this task needs no new command |
| The owner's live desktop | Screenshots are the owner's |

---

## Test Plan

`cargo test` (both feature sets): install/uninstall/upgrade with fs snapshots; menu, argv, icon-choice and badge unit tests; SVG parse test. Optional renders with `rsvg-convert` for the visual check. Owner-only: GNOME screenshots of the app grid and tray.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-19 DECIDED a fourth icon file `easy-note-saved-symbolic.svg` carries the badge, so the tray swaps icon names instead of drawing pixmaps (Supervisor, RULE: `AGENTS.md` § Karpathy Engineering Principles, Simplicity First — no SVG rasteriser crate)
* 2026-11-19 DECIDED the badge watches folder mtimes, not the note list (Supervisor, RULE: `PRD.md` NFR-004 — no full scan per second)
* 2026-11-19 DECIDED icons and the launcher-to-dashboard change are one task with the tray item, since all edit `install.rs`/`tray.rs` (Supervisor, RULE: `AGENTS.md` § Karpathy Engineering Principles, Surgical Changes)