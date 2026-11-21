# TASK_GUIDE — HUN-286: Dashboard window: read and edit a note in the app, not in Obsidian
**Date**: 2026-11-21
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer (Rust: `window::api` `edit`, `dashboard --note`, tray click), then Frontend-Implementer (the page in `src/surfaces/window/assets/`)
**Agent guide**: `agents/backend.md`, `agents/frontend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:
1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*, *AI detail*, *Inbox*)
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read your agent guide (`agents/backend.md` or `agents/frontend.md`)
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in your agent guide
6. Read `memory/codebase-map.md`
7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `window`, `detail panel`, `HUN-279`, `HUN-280` before reading source files
8. Read `docs/ddr/0009-own-dashboard-window.md`, `docs/design/Main.dc.html` (detail panel markup and CSS), PRD US-019, FR-013 (`edit`), FR-042, FR-043, NFR-002, NFR-013
9. **UI task — screenshots**: the page is a web page in a webview: take screenshots with easy-ui on the page with a stub IPC, as HUN-278/HUN-280 did, and judge them with `Skill({ skill: "ux-evaluate" })`
10. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-21: "by the way, the detail of note to view and edit should show in the application, not from the obsidian".

Today: the window's detail panel shows the note read-only; its `Open in editor` button runs `xdg-open <file>`, which opens Obsidian on the owner's machine; a tray recent-note click also runs `xdg-open <file>` (Obsidian). Editing is only possible outside the app (`easy-note edit` in `$EDITOR`, or Obsidian).

**Restated intent**:
> A note is read and edited inside easy-note's own window. The detail panel gets an `Edit` mode (text and tags) that saves through the same core path as `easy-note edit`. A tray recent-note click opens the window on that note instead of Obsidian.

**Out of scope**:
- Rich Markdown preview or a Markdown editor (plain text area; the panel already shows the text as the design does)
- Editing the AI detail
- One window instance shared between clicks (a second click opens a second window, as `dashboard` does today)
- Editing inbox notes (read-only by FR, `StoreError::ReadOnly`)
- The Obsidian path when there is no display or no `window` feature (unchanged: FR-034)

**Requirement Refs**:
- FR-043: detail panel actions (`Open in editor` is replaced by `Edit`: Supervisor PRD edit)
- FR-042, US-019: own window, handle notes without another app
- FR-013 `edit`: same core behaviour (masking, re-validate, `updated` stamp)
- NFR-002: secrets masked on write
- FR-033: tray recent-note click (now opens the window when it can)

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request — owner approved the plan (chat 2026-11-21)
- [X] Domain terms align with `PROJECT_SPEC.md` glossary — Supervisor 2026-11-21
- [X] Every Acceptance Criterion below traces to a line in the Requirement — Supervisor 2026-11-21
- [X] All Requirement Refs exist in `PRD.md` and are covered — Supervisor 2026-11-21 (FR-043 and FR-033 wording updated by the Supervisor)

---

## Dependencies & Reachability

**Depends on**: None (HUN-285 touches `tray.rs` labels and start; this task touches the tray click only: merge HUN-285 first if both are open, then rebase)

**Entry point**: `window::api` command `edit`; the `Edit` button in the detail panel (`src/surfaces/window/assets/app.js`); `easy-note dashboard --note <id>`; tray `Action::Open`

---

## Acceptance Criteria

| # | Criterion (testable) | Traces to requirement |
|---|----------------------|-----------------------|
| 1 | `{"cmd": "edit", "id", "text", "tags": [..], "base": "<text the page loaded>"}` saves through `Store::update` (same as `easy-note edit`: masking, re-validate, `updated` stamp) and returns the note's `detail`, with `masked` true when the guard masked something | FR-043, FR-013, NFR-002 |
| 2 | When the note's current text differs from `base` (changed outside the window), `edit` writes nothing and returns `{"error": "the note changed outside the window"}`; the page then reloads the note and keeps the person's draft in the box | edit in the app safely |
| 3 | Empty or whitespace-only text → `{"error": "empty"}`, nothing written; an inbox note → the `inbox notes are read-only` error; a trashed / unknown id → `no such note`; tags are cleaned by the same rule as `easy-note tag` (no `#`, no duplicates) | FR-013 |
| 4 | Detail panel: `Edit` replaces `Open in editor`. Edit mode shows a text area with the note text and a tags field; `Save` (and Ctrl+Enter) saves, `Cancel` (and Esc) leaves without writing; after save the panel shows the new text, the list row updates, and the AI detail shows `Stale` when the text changed | US-019 |
| 5 | An inbox note shows no `Edit` button, and the panel says it is read-only | FR-013 |
| 6 | The board refresh on focus (`REFRESH`) does not leave edit mode or drop the draft | edit in the app |
| 7 | `easy-note dashboard --note <id>` opens the window with that note selected and its detail shown; an id that fails the store's id check is refused before the page sees it (exit 2, usage message); an unknown valid id opens the board with the panel's "Could not read the note" text | tray click opens the app |
| 8 | Tray recent-note click: in a `window` build with a display, runs `<exe> dashboard --note <id>` (argv list); otherwise `xdg-open <file>` as today | FR-033 |
| 9 | `open_in_editor` is removed from `window::api`, `ipc.js` and the page; no other code path in the window runs `xdg-open` on a note | owner: not Obsidian |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| # | Given | Expect | How it's checked |
|---|-------|--------|------------------|
| 1 | a note, `edit` with new text and tags | file has the new body and tags, `updated` set, other front matter kept | `tests/window_api.rs` |
| 2 | file changed on disk after load, `edit` with the old `base` | error per AC 2, file unchanged byte for byte | same |
| 3 | text with a planted secret | stored masked, `masked: true` | same (`tests/leak.rs` style) |
| 4 | empty text / inbox id / trashed id | errors per AC 3, no write | same |
| 5 | `dashboard --note "../x"` | exit 2, no window | `tests/dashboard.rs` |
| 6 | tray click argv with window build + display, and without | `dashboard --note <id>` / `xdg-open <file>` | `tray.rs` unit test |
| 7 | page with stub IPC: Edit → type → Ctrl+Enter; Esc; refresh during edit | AC 4, 6 | easy-ui run + screenshots |

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
| Detail panel, read and edit mode | easy-ui screenshots with stub IPC | read mode unchanged except `Edit` in place of `Open in editor`; edit mode uses the panel's existing text and input styles |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
|-----------|---------------------|-----------------|
| Colors match design tokens | CSS audit vs `docs/design/graphite-mint-tokens.json` | only existing tokens; no new colours |
| Typography matches spec | computed style | text area uses the panel body font |
| Spacing / layout matches spec | computed style | buttons in the existing `acts` row |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
|----------|---------------------|-----------------|
| 800 px | screenshot | edit mode fits the panel; Save/Cancel visible without horizontal scroll |
| 1280 px | screenshot | same |
| Mobile | N/A | a desktop window; min width 800 px (HUN-284) |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
|------------|-----------|--------------|-----------------------------|
| US-019 | fix a typo in a note | select note → Edit → change → Save | panel and row show the new text; no other app opens |
| US-019 | give up an edit | Edit → change → Esc | panel shows the old text; file unchanged |
| US-019 | note changed in another app | Edit, change file on disk, Save | message "changed outside the window", draft still in the box, note reloaded |
| US-015 | open a note from the tray | tray → recent note | the window opens with that note's detail shown |
| US-019 | inbox note | select an inbox note | no Edit button; read-only line shown |

---

## Approach

**Pattern reference**: `window::api::accept_ai` / `trash` (core call, then `detail`); `src/surfaces/cli.rs` `Edit` (tags cleaning, `Store::update`); HUN-280's dialog key handling in `app.js` (IME-safe Enter).

**Vital slice**: `edit` in `window::api` (tested), then the Edit mode in the panel. `--note` and the tray click come next.

**Cut list**: Markdown preview; a shared single window; editing AI detail; autosave.

`--note`: validate with the store's id check in the CLI; pass the id to the page by an initialization script holding the id as a JSON string (`serde_json::to_string`), never by string-building into JS; the page selects it after the first board load. The CSP and the exact-origin checks from HUN-284 stay as they are.

---

## Edge Case Checklist

- [ ] IME composition: Ctrl+Enter / Esc while composing does nothing
- [ ] Text with only a trailing-newline change: saved, no false "changed outside" (compare as `Store::update` writes: `with_newline`)
- [ ] A masked note (`[masked:…]` in the text) edited and saved: stays masked, no secret re-entered
- [ ] Very long text: saved whole, or the `the text is too long` error shown
- [ ] Selecting another note while editing: ask nothing, keep it simple — leave edit mode and drop the draft only after a confirm, or block the switch; pick one and test it (frontend decides, records in Questions Log)
- [ ] `after_save` enrich: an edit does what `easy-note edit` does today (check `cli.rs`; do not add a new trigger)
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help, `ipc.js` header for `Open in editor` / `open_in_editor`

---

## Files to Change (Predicted)

| File | Change |
|------|--------|
| `src/surfaces/window/api.rs` | `edit`; remove `open_in_editor` |
| `src/surfaces/window/mod.rs` | initial note id for the page |
| `src/surfaces/window/assets/{app.js,ipc.js,app.css}` | Edit mode; remove Open in editor |
| `src/surfaces/cli.rs`, `src/surfaces/dashboard.rs` | `dashboard --note <id>` |
| `src/surfaces/tray.rs` | recent-note click argv |
| `tests/window_api.rs`, `tests/dashboard.rs` | AC 1–3, 7; replace the `open_in_editor` tests |
| `README.md` | window edit line |
| Pre-existing pin tests | `tests/window_api.rs` (`open_in_editor_runs_xdg_open_once_with_the_note_path`, the trashed-id loop at line 945), asset scan test, `tests/readme.rs`, `tests/cli.rs` help text, `tray.rs` `each_action_is_an_argv_list` — list each and its handling |

## Files Must NOT Touch

| File | Reason |
|------|--------|
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*`, `docs/design/*` | Supervisor / owner owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| `src/core/*` | Call `Store::update` and existing public functions only; a missing one is a `Question:` to the Supervisor |
| `scripts/build_deb.sh`, `tests/deb.rs` | the `.deb` work on branch `feat/HUN-285-deb` (another session's open work) |

---

## Test Plan

`window::api::edit` is pure over a temp store and tested in the default build. `--note` validation in `tests/dashboard.rs` (no window opened: refuse before open). Tray click argv as a unit test. The page: easy-ui with a stub IPC for the UX rows. Owner-only: tray click and edit on the real machine.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] **easy-verifier**: Stage 4 `Skill({ skill: "score" })` run by the Supervisor
- [ ] Security review: `Skill({ skill: "security-review" })` run (Medium risk: a webview IPC now writes note files)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md` (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

- 2026-11-21 DECIDED `Edit` replaces `Open in editor`; the window no longer opens a note in another app (owner: "should show in the application, not from the obsidian")
- 2026-11-21 DECIDED edit covers text and tags, as `easy-note edit` does (Supervisor recommendation, owner approved 2026-11-21)
- 2026-11-21 DECIDED stale-write guard by `base` text compare (Supervisor default)
- 2026-11-21 DECIDED (frontend-developer) selecting another note while editing: **block the switch**, no confirm dialog. While the panel is in edit mode the selection stays on that note: a row click shows the toast `save or cancel the edit first` and puts the caret back in the draft; Undo of a trash and a new note from `+ New note` do not change the selection either. Esc / Cancel or Save leave edit mode, then rows select as before. Why: a `confirm()` dialog depends on WebKitGTK's script-dialog handling in wry (not tested in this project), and blocking never drops a draft. Test: `tests/window_api.rs` `another_note_is_not_selected_while_editing` (page harness scenario `edit_switch`)
- 2026-11-21 DECIDED (frontend-developer) when the note being edited leaves the board (trashed or deleted in another app, seen on a board reload), edit mode ends with the selection, as the read panel did before; a save would get `no such note` anyway
