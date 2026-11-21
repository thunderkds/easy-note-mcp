# TASK_GUIDE — HUN-285: Tray: starts at login and with the app, menu texts from the design
**Date**: 2026-11-21
**Complexity Level**: C1
**Risk Level**: Low
**Priority**: P1
**Assigned agent**: Backend-Implementer (Rust: `src/surfaces/tray.rs`, `src/surfaces/dashboard.rs`)
**Agent guide**: `agents/backend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:
1. Read `PROJECT_SPEC.md`
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read `agents/backend.md`
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in `agents/backend.md`
6. **easy-memory first**: call `mcp__easy-memory__memory_search` with `tray`, `HUN-263`, `HUN-276` before reading source files
7. Read `docs/design/TrayMenu.dc.html`, PRD FR-033, FR-039, FR-041, NFR-001, NFR-004
8. **UI task — screenshots**: the tray menu is a desktop-native surface: easy-ui does not apply. Take an OS screenshot of the open menu and judge it with `Skill({ skill: "ux-evaluate" })`
9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/` or `~/.config/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-21: "seem like the tray does not run by the new design, and it does not start with the application yet, let check and update it". Owner picked: the tray starts at login **and** when the app (dashboard window) opens; the hotkey hint shows as `New note · Ctrl+Alt+N`.

Supervisor's check on the owner's machine (Ubuntu GNOME, `ubuntu-appindicators` enabled, `/usr/bin/easy-note` built with `window,tray`): `~/.config/autostart/easy-note-tray.desktop` ran at login and the tray quit at once. Journal, 2026-11-21 17:04:01: `easy-note: no tray host (failed to register to the StatusNotifierWatcher: org.freedesktop.DBus.Error.ServiceUnknown: The name org.kde.StatusNotifierWatcher was not provided by any .service files); GNOME needs the AppIndicator extension; ...`. Started by hand later, the tray works. Cause: at login the autostart entry runs before the AppIndicator extension owns `org.kde.StatusNotifierWatcher`, and `tray::run` gives up on the first try.

Design gaps (`docs/design/TrayMenu.dc.html`): the New note row carries the hint `Ctrl+Alt+N`; the empty state's disabled row reads `No notes yet. Your first one appears here.` (today: `no notes yet`).

**Restated intent**:
> The tray icon is there after every login, and opening the dashboard window starts the tray when it is not running. The menu texts match the design.

**Out of scope**:
- The design's `new` marker beside `Open dashboard` (a design-board note for a then-new item, not a menu text)
- Opening a recent note in the app instead of `xdg-open` (the in-app note task does it)
- macOS/Windows tray autostart (PRD non-goal, v1.1)
- Re-registering after the tray host goes away mid-session (today: exit with the message; unchanged)

**Requirement Refs**:
- FR-033: tray, autostart, no-host message
- FR-039: tray menu from the design
- US-015

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request — owner (chat 2026-11-21, two answers)
- [X] Domain terms align with `PROJECT_SPEC.md` glossary — Supervisor 2026-11-21
- [X] Every Acceptance Criterion below traces to a line in the Requirement — Supervisor 2026-11-21
- [X] All Requirement Refs exist in `PRD.md` and are covered — Supervisor 2026-11-21 (PRD FR-033 gets one sentence on the wait and the start from `dashboard`: Supervisor edit)

---

## Dependencies & Reachability

**Depends on**: None

**Entry point**: `tray::run` (wait for the host) and `dashboard::run` (start the tray); menu texts in `tray::menu`

---

## Acceptance Criteria

| # | Criterion (testable) | Traces to requirement |
|---|----------------------|-----------------------|
| 1 | When no tray host is on the bus at start, `easy-note tray` checks again every 1 s for up to 60 s and shows the icon as soon as the host appears; it still holds the single-instance lock while it waits | "does not start" at login |
| 2 | No host after 60 s: the same stderr message and exit 1 as today | FR-033 |
| 3 | `easy-note dashboard` in a build with both `tray` and `window`, when the window opens, starts `<this exe> tray` detached (argv list, stdio null, not waited on by the window); when a tray already runs, the child exits on the lock and nothing else happens; no error is shown in the window | "start with the application" |
| 4 | A build without `tray`, or the Obsidian fallback path, starts no tray | NFR-010 / FR-033 |
| 5 | Menu row 1 reads `New note · Ctrl+Alt+N`; the empty-state disabled row reads `No notes yet. Your first one appears here.`; all other rows and their order are unchanged | FR-039, design |
| 6 | `install tray` writes the autostart file with the same content as today (no `X-GNOME-Autostart-Delay`); a second `setup` run still writes nothing | FR-033, FR-041 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| # | Given | Expect | How it's checked |
|---|-------|--------|------------------|
| 1 | a pure wait function fed "absent ×3, then present" | returns ready after the 4th check; fed absent for the whole window → gives up after 60 checks | unit test in `tray.rs` (inject the check and the sleep; no real D-Bus, no real sleep) |
| 2 | `menu(&[], now)` and `menu(&five, now)` | labels per AC 5 | `tray.rs` tests, updated |
| 3 | the tray-start argv | `(<exe>, ["tray"])` | unit test of a pure argv function |
| 4 | owner's machine: log out and in | tray icon is there | manual (owner), journal shows no "no tray host" line |
| 5 | tray quit, then launcher → dashboard window | tray icon appears | manual (owner) |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features tray -- -D warnings \
  && cargo test --locked --features tray \
  && cargo clippy --all-targets --locked --features window,tray -- -D warnings \
  && cargo test --locked --features window,tray
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** Filled by the reviewer in `tasks/TASK_REVIEW_<ID>.md`.

---

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_<ID>.md`.

---

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
|-------------------|---------------------|-----------------|
| Tray menu, with notes and empty | OS screenshot + `ux-evaluate` | labels and order as `docs/design/TrayMenu.dc.html` (hint in the label) |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
|-----------|---------------------|-----------------|
| Colors / typography | N/A | the panel draws the menu; the design says its colours follow the desktop theme |
| Labels, order, 40-char cut | `tray.rs` unit tests | as the design |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
|----------|---------------------|-----------------|
| N/A | — | a panel menu has no viewport; the desktop draws it |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
|------------|-----------|--------------|-----------------------------|
| US-015 | tray after login | log in | icon in the top bar within 60 s; menu opens |
| US-015 | tray with the app | tray not running, open the dashboard from the launcher | icon appears; window opens as before |
| US-015 | empty store | menu with no notes | disabled row `No notes yet. Your first one appears here.` |

---

## Approach

**Pattern reference**: `src/surfaces/tray.rs` `argv` / `icon::start` (argv list, stdio null, reaped on a thread) and `icon::host_present`.

**Vital slice**: the wait loop in `tray::run` (the login fix), then the start from `dashboard::run`.

**Cut list**: re-register after the host goes away; the `new` marker; a delay key in the autostart file.

Wait loop: before `tray.spawn()`, loop `host_present()` with a 1 s sleep up to 60 checks; take the lock first so a second tray started during the wait exits at once. Keep the loop logic in a pure function over `FnMut() -> bool` and a sleeper so it is tested without D-Bus. Start from `dashboard`: only under `#[cfg(all(feature = "tray", feature = "window"))]` and only on the path that opens the window, before the event loop runs; the child is reaped on a thread like `icon::start`.

---

## Edge Case Checklist

- [ ] Two autostart runs at login (duplicate desktop entries): the second exits on the lock, during the wait too
- [ ] `dashboard` started from a terminal: the tray child does not hold the terminal's stdio
- [ ] `current_exe` fails: the window still opens; no tray, no panic
- [ ] Underscore in labels still doubled (`mnemonic_safe`); `·` and `+` are not mnemonic markers
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for `no notes yet` and for the tray start wording

---

## Files to Change (Predicted)

| File | Change |
|------|--------|
| `src/surfaces/tray.rs` | wait loop, labels, tray-start argv helper |
| `src/surfaces/dashboard.rs` | start the tray when the window opens (tray+window build) |
| `tests/tray.rs` | adjust if it pins labels |
| `README.md` | one line: the tray waits for the panel at login and starts with the dashboard |
| Pre-existing pin tests | `src/surfaces/tray.rs` tests (`the_menu_lists_...`, `an_empty_store_...`), `tests/tray.rs`, `tests/readme.rs`, `tests/setup.rs` — repoint the label asserts; list each in the review |

## Files Must NOT Touch

| File | Reason |
|------|--------|
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*`, `docs/design/*` | Supervisor / owner owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| `src/core/*` | No core change is needed |
| `scripts/build_deb.sh`, `tests/deb.rs` | the `.deb` work on branch `feat/HUN-285-deb` (another session's open work; it used the ID HUN-285 before this issue took it) |

---

## Test Plan

Unit tests in `tray.rs` for the wait function (injected check + sleeper), labels and the start argv; the existing tray/setup integration tests stay green in all three feature sets. Owner-only: log out/in and launcher-start checks on the real GNOME session.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] **easy-verifier**: Stage 4 `Skill({ skill: "score" })` run by the Supervisor
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md` (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

- 2026-11-21 DECIDED tray starts at login and when the dashboard window opens (owner)
- 2026-11-21 DECIDED hint shown in the label as `New note · Ctrl+Alt+N`, always (owner); GNOME's AppIndicator menu shows no dbusmenu shortcut text
- 2026-11-21 DECIDED wait 60 s, 1 s steps (Supervisor default)
