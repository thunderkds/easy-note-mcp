**Date**: 2026-11-23
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer (Rust: `quick` window path, the shell), then Frontend-Implementer (the capture-only page in `src/surfaces/window/assets/`)
**Agent guide**: `agents/backend.md`, `agents/frontend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

 1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Context*, *Quick box*)
 2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
 3. Read this file completely
 4. Read your agent guide (`agents/backend.md` or `agents/frontend.md`)
 5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in your agent guide
 6. Read `memory/codebase-map.md`
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `quick`, `capture dialog`, `HUN-260`, `HUN-280`, `HUN-281` before reading source files
 8. Read `docs/ddr/0009-own-dashboard-window.md`, `docs/design/Main.dc.html` (the capture dialog markup and CSS), PRD US-011, US-014, US-015, FR-022, FR-027, FR-033, FR-044, NFR-011
 9. **UI task — screenshots**: take screenshots with easy-ui on the capture page with a stub IPC, as HUN-280 did, and judge them with `Skill({ skill: "ux-evaluate" })`
10. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-22: "the new note from tray is not style". Offered "open the dashboard window with the dialog open", the owner answered: "anyway to style from the dialog without open the dashboard".

Today: tray `New note` and the `Ctrl+Alt+N` hotkey both run `easy-note quick`, which opens a native `zenity` (else `kdialog`) box: plain, unstyled. The dashboard window has the design's capture dialog (`+ New note`, HUN-280), but only inside the full board.

**Restated intent**:

> In a build with the `window` feature and a display, `easy-note quick` opens a small window of its own that holds only the design's capture dialog, the same one as the dashboard's `+ New note`: same look, Enter saves, Shift+Enter adds a line, Esc cancels, the context chip shown and dropped with Backspace. It never shows the board. It closes on save or cancel. Tray `New note` and the hotkey get it, because both run `quick`. Without the feature or a display, `quick` stays the zenity / kdialog / terminal path.

**Out of scope**:

* The dashboard's `+ New note` (unchanged)
* A different hotkey or tray menu text (FR-033 labels from HUN-285 stay)
* macOS / other platforms
* Keeping the small window open between notes

**Requirement Refs**:

* FR-022: `quick` opens a one-line input (gets the window path first: Supervisor PRD edit at Stage 2)
* FR-027: context pre-fill and drop on one key
* FR-044: the design's capture dialog behaviour
* NFR-011: quick box visible p95 < 300 ms (see Q1)
* US-014, US-015: one click / one key from the desktop or the tray

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request — owner, chat 2026-11-23 ("Start HUN-289 now")
- [X] Domain terms align with `PROJECT_SPEC.md` glossary — Supervisor 2026-11-23
- [X] Every Acceptance Criterion below traces to a line in the Requirement — Supervisor 2026-11-23
- [ ] All Requirement Refs exist in `PRD.md` and are covered (FR-022 wording to be updated by the Supervisor)
- [X] Questions Log Q1 answered by the owner — "measure, then decide" (2026-11-23); the number is below, the choice is still open

---

## Dependencies & Reachability

**Depends on**: HUN-280 (merged: the capture dialog and `context` / `add` commands), HUN-281 (merged: the `NO_DISPLAY` fallback). Coordinate with HUN-288 (Run AI): both touch `src/surfaces/window/mod.rs`; merge one, then rebase the other.

**Entry point**: `easy-note quick` (`src/surfaces/quick.rs` `run`); tray `Action::NewNote`; the hotkey command line written by `install launcher`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | In a `window` build with `WAYLAND_DISPLAY` or `DISPLAY` set, `easy-note quick` opens a small undecorated-or-plain window (size of the dialog, centred) showing only the design's capture dialog; no board, no board request | restated intent |
| 2 | Enter saves through the same path as `quick` today (`source: quick`, the FR-027 context or the `--cwd` context, `#words` tags, masking, `after_save` rule), shows the saved confirmation (`notify-send` when found, as today), and closes the window; exit 0 | FR-022, FR-027 |
| 3 | Esc, the window's close button, or an empty Enter writes nothing and closes the window; exit 0 (a cancel, as today) | FR-022 |
| 4 | Shift+Enter adds a newline; Backspace in an empty box drops the context chip; IME composition is ignored — the same key rules as the dashboard dialog (one shared implementation, not a copy) | FR-044 |
| 5 | `quick --cwd DIR` uses that folder's context in the chip, as the zenity path does | FR-027 |
| 6 | When the window cannot open (no `window` feature, no display, or the HUN-281 `NO_DISPLAY` case), `quick` takes today's zenity / kdialog / terminal path unchanged | FR-022 |
| 7 | The page talks only `context` and `add` (no `board`, `detail`, `edit`, `trash`); IPC origin and CSP checks are the same as the dashboard's | DDR 0009, HUN-284 |
| 8 | Time from `quick` start to the dialog visible is measured on the owner's machine (release build) and reported against NFR-011 | NFR-011 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | `quick` path choice for each case (feature on/off, display set/unset, open failed) | window path or zenity path per AC 1, 6 | unit test of the choice function (like `dashboard::use_window`) |
| 2 | the capture page with a stub IPC: type, Enter; Esc; empty Enter; Shift+Enter; Backspace on empty | AC 2–4: `add` sent once with the text, or nothing sent; a close request sent | page harness (`tests/fixtures/window/page_harness.js`) |
| 3 | `add` through the quick window's store with `--cwd` context | note with `source: quick` and that context | `tests/quick.rs` or `tests/window_api.rs` |
| 4 | the capture page's IPC asks for `board` | refused (`unknown command` for this window) | `tests/window_api.rs` |
| 5 | release build on the owner's machine | time to visible, p50 / p95 of 20 runs | manual, pasted into the review file |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features window,tray -- -D warnings \
  && cargo test --locked --features window,tray
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-289.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-289.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Quick window: empty, with context chip, with text, error | easy-ui screenshots with stub IPC | looks like the dashboard's capture dialog (same CSS), without the board or the dim backdrop behind it |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Colors match design tokens | CSS audit vs `docs/design/graphite-mint-tokens.json` | only existing tokens; the dialog CSS is shared, not copied |
| Typography matches spec | computed style | bundled Plex fonts, as the dashboard |
| Spacing / layout matches spec | computed style | dialog box fills the window with its design padding |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| The quick window's own size (dialog width) | screenshot | no scroll bars, no cut-off chip or hint line |
| 1280 px screen | screenshot | window centred |
| Mobile | N/A | desktop only |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-015 | note from the tray | tray → New note → type → Enter | styled dialog, saved toast/notification, window gone |
| US-014 | note from the hotkey | Ctrl+Alt+N → type → Enter | same |
| US-014 | change my mind | open → Esc | window gone, nothing saved |
| US-014 | note without the last context | open → Backspace on empty | chip gone; saved note has no repo/task |

---

## Approach

**Pattern reference**: `src/surfaces/window/mod.rs` (`open`, `NO_DISPLAY`, `allowed_url`, `ipc_from_with`, the custom protocol and CSP); HUN-280's capture dialog in `app.js` / `app.css` / `index.html`; `dashboard::use_window` / `window_here` for the path choice; `quick::run` for the save path, `--cwd` context and `notify-send`.

**Vital slice**: the path choice in `quick` plus a second window entry in `window` (a capture-only page served from the same protocol) that saves through the same code `quick` uses, then the page.

**Cut list**: remembering the window position; a global single instance; animations.

Split the capture dialog's JS and CSS so the dashboard and the quick page load the same file (for example `capture.js` beside `app.js`); do not copy it. The quick page gets its context from `quick` (with `--cwd`) at start, through the initialization script as a JSON value (like `initial_note_script` in HUN-286), or through `context` answered by the quick window's own handler.

---

## Edge Case Checklist

- [ ] Two `quick` runs at once (two hotkey presses): two windows, or the second exits; pick one and test it (Questions Log)
- [ ] Focus: the text box has focus when the window shows, also when another app had focus (window manager may refuse; note what happens on GNOME and KDE)
- [ ] `notify-send` missing: the save still closes the window, no error
- [ ] The secret guard: a planted secret is masked on write, as `quick` does today
- [ ] Very long text: as the dashboard dialog does
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help (`quick` help text names zenity) for the quick box description

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/quick.rs` | window path first in a `window` build with a display; zenity path as the fallback |
| `src/surfaces/window/mod.rs` | a capture-only window entry (`open_capture` or similar) |
| `src/surfaces/window/api.rs` | the command set for the capture window (`context`, `add`, close) |
| `src/surfaces/window/assets/` | capture-only page; dialog JS/CSS shared with the dashboard |
| `tests/window_api.rs`, `tests/fixtures/window/page_harness.js`, `tests/quick.rs` | AC 1–7 |
| `README.md`, CLI help for `quick` | the quick box description |
| Pre-existing pin tests | asset-serving list test, inline-style scan, `tests/readme.rs`, `tests/cli.rs` help text, `quick` tool-choice tests — list each and its handling |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*`, `docs/design/*` | Supervisor / owner owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| `src/core/*` | Call existing public functions only; a missing one is a `Question:` to the Supervisor |
| `src/surfaces/tray.rs` | `New note` already runs `quick`; no change needed |

---

## Test Plan

Path choice as a unit test. The capture page with a stub IPC in the page harness for the key rules and the close request. The save path with `--cwd` over a temp store. The command-set restriction in `tests/window_api.rs`. Owner-only: tray New note and the hotkey on the real machine, and the NFR-011 timing.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] **easy-verifier**: Stage 4 `Skill({ skill: "score" })` run by the Supervisor
- [ ] Security review: `Skill({ skill: "security-review" })` run (Medium risk: a second webview that writes notes)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md` (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-22 DECIDED the styled dialog opens on its own, without the dashboard (owner, chat 2026-11-22: "anyway to style from the dialog without open the dashboard")
* 2026-11-23 DECIDED tray `New note` and the hotkey both get it, because both run `quick` (Supervisor; the owner's request names the tray, and one `quick` path keeps them the same)
* 2026-11-23 OPEN Q1 (owner): NFR-011 asks for the quick box visible p95 < 300 ms. A WebKitGTK window may take longer to show than zenity. If the measured p95 is over 300 ms: Option A — keep the styled window and change NFR-011 to the measured figure; Option B — keep zenity as the hotkey path and use the styled window for the tray only. Supervisor recommendation: measure first (AC 8), then decide with the number.
* 2026-11-23 DECIDED owner: build it now; for Q1, measure first and bring the number before changing the hotkey path
* 2026-11-23 MEASURED Q1 (release build, owner's machine, GNOME, under XWayland, temp notes dir): window mapped p50 95 ms, p95 114 ms (20 runs); dialog painted (accent border on screen) in 6/10 runs at 300 ms, 8/10 at 350 ms, 10/10 at 400 ms. So the dialog is visible at about p50 300 ms and p95 400–450 ms: over NFR-011's 300 ms. Owner to choose Option A or B
* 2026-11-23 DECIDED two quick runs at once: two windows (no single instance; cut list). Each saves on its own
