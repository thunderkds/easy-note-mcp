**Date**: 2026-11-14
**Complexity Level**: C1
**Risk Level**: Low
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`
**Plan ID**: T012 (`linear-tasks.md`, `PROJECT_SPEC.md` Tasks)

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*, *Context*, *Guard*, *Trash*, *Skipped*)
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read `agents/backend.md`
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in `agents/backend.md`
6. **C2/C3 or multi-file tasks only**: read `memory/codebase-map.md` if it exists
7. Tests only: temp `HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/easy-note/` or `~/.claude/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Plan T012: "`quick` tests with the dialog tool stubbed: text → saved `source: quick`; cancel/empty → nothing saved, exit 0; `--cwd` repo tag; last context < 2 h pre-filled and clearable, ≥ 2 h not used; no dialog tool → `tkinter`, else terminal prompt; box visible p95 < 300 ms (measured on GNOME/Wayland and X11); `hotkey` prints correct commands per desktop and writes nothing; `install launcher` writes exactly one `.desktop` file, `uninstall launcher` removes it. README snippets: VS Code `tasks.json` + `keybindings.json` using `${workspaceFolder}`, JetBrains External Tool with `$ProjectFileDir$`, macOS Shortcuts, GNOME/KDE custom shortcut; each tried once."

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> From anywhere on the desktop or in an editor, one key press or one click opens a one-line box; Enter saves the note with the right context.

**Out of scope** (what this task explicitly does NOT do):

* A key-listener daemon (D27)
* Writing OS shortcut settings (`hotkey` only prints)
* macOS/Windows launcher files (README only)
* The tray (T015)

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-022: native box, fallbacks, notification
* FR-025: `hotkey` prints commands
* FR-026: launcher file
* FR-027: last context < 2 h
* NFR-011: box visible < 300 ms
* US-011, US-014, DDR 0003, GRILL D22, D27, D28, D30, D31

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent)
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy)
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

> An agent must NOT start implementing until this gate is checked. If anything here is unclear,
> STOP and ask the Supervisor (Karpathy: Think Before Coding).

---

## Dependencies & Reachability

**Depends on**: HUN-257 (plan T007) — CLI and add path

**Entry point**: `easy-note quick`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | Dialog stubbed: entered text → note with `source: quick`; cancel or empty → nothing saved, exit 0 | FR-022 |
| 2 | `--cwd DIR` gives that repo's context | FR-022 |
| 3 | Newest `cli`/`hook` note < 2 h old pre-fills the context chip, shown in the box and clearable with one key; ≥ 2 h → no context | FR-027, D30 |
| 4 | Tool order: `osascript` (macOS) → `zenity`/`kdialog` (Linux) → `tkinter` → terminal prompt | FR-022 |
| 5 | Confirmation shown via `notify-send` / `osascript` notification where available | FR-022, FR-021 |
| 6 | Box visible p95 < 300 ms after start, measured on GNOME Wayland and X11 (numbers pasted) | NFR-011 |
| 7 | `easy-note hotkey` prints correct commands for GNOME, KDE, macOS, Windows and writes nothing (fs snapshot before/after) | FR-025 |
| 8 | `install launcher` writes exactly `~/.local/share/applications/easy-note.desktop` running `easy-note quick`; `uninstall launcher` removes it | FR-026, DDR 0003 |
| 9 | README snippets for VS Code, JetBrains, macOS Shortcuts, GNOME/KDE, each tried once (pasted) | US-011 |
| 10 | Quick surface registered in `tests/test_leak.py` and passing | NFR-002 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | Stubbed `zenity` returning `retry bug #ff` | note saved, `source: quick`, tag `ff` | automated test |
| 2 | Stubbed `zenity` exiting 1 (cancel) | no file, exit 0 | automated test |

### Verification Command (exact, runnable)

```bash
uv run pytest tests/test_quick.py tests/test_hotkey.py tests/test_install.py tests/test_leak.py -q
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-260.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-260.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

> Desktop-native surface (OS dialog / tray menu), not a web page. Design source: `DESIGN_PROMPT.md` (designer output wins when it exists). Evidence is screenshots on the owner's GNOME desktop.

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| `easy-note quick` capture box | LLM-vision read of screenshots (GNOME Wayland + X11) | matches `DESIGN_PROMPT.md` capture rules: one line, context chip visible, no extra chrome |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Colors match design tokens | LLM vision | system theme colours (native dialog / tray theme); no hard-coded palette |
| Typography matches spec | LLM vision | system UI font |
| Spacing / layout matches spec | LLM vision | single input row (+ context chip); tray: 4 menu groups in FR-033 order |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| Mobile (320–480px) | N/A | N/A — desktop-only surface; phones use `inbox/` (T013) |
| Tablet (768px) | N/A | N/A — desktop-only surface |
| Desktop (1024px+) | screenshot at 1280×800 and 2560×1440 (HiDPI) | box/menu fully on screen, readable, not clipped |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-014 | note with one key press | hotkey → box → type → Enter | box shows within 300 ms; after Enter it closes and a `✓ noted` notification shows |
| US-011 | note from the editor with repo context | editor keybinding → box with chip | chip shows the repo/task; one key clears it |
| US-014 | change my mind | box open → Esc | box closes, no note file, no notification |

---

## Approach

**Pattern reference**: `src/easy_note/surfaces/install.py` (T011) — exact-file install/uninstall

**Vital slice**: `quick` with the native box + last context; `hotkey`/launcher are small printers/writers

**Cut list**: Rich multi-line box (Shift+Enter → `-e` path instead); Custom-drawn window if native tools exist

`quick` imports only what the box needs before showing it (speed), then core after Enter. The context chip is the box's prefilled label/entry text per `DESIGN_PROMPT.md` (designer output, if any, wins).

---

## Edge Case Checklist

- [ ] Wayland: `zenity` works, `tkinter` may open off-focus — measure
- [ ] No display (`$DISPLAY`/`$WAYLAND_DISPLAY` unset) → terminal prompt
- [ ] Notification tool missing → print the line only
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: for each fact this task changes, grep README, `docs/`, comments and docstrings for the old wording

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/easy_note/surfaces/quick.py` | new |
| `src/easy_note/surfaces/install.py` | launcher target |
| `src/easy_note/surfaces/cli.py` | `quick`, `hotkey`, `install\|uninstall launcher` |
| `tests/test_quick.py`, `tests/test_hotkey.py` | new |
| `README.md` | editor/OS snippets section |
| Pre-existing pin tests | List each test in `tests/` that pins text, paths or counts of a file this task edits, and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md` | Supervisor-owned; report needed changes instead |
| `docs/ddr/*` | Decisions are owner-approved; a change needs a new DDR from the Supervisor |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/` | Kit files; Supervisor-only |
| The real `~/easy-notes/`, `~/.config/easy-note/`, `~/.claude/`, `~/.codex/`, `~/.cursor/` | Tests use a temp `HOME` and `EASY_NOTE_DIR` only (NFR-003) |

---

## Test Plan

Stubbed dialog binaries on PATH; fs snapshot for `hotkey`; manual timing on GNOME Wayland + X11; manual snippet runs.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI / client
- [ ] Supervisor notified: task ready for Stage 4 review