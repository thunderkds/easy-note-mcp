**Date**: 2026-11-19
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer (Rust: feature, window, IPC, `window::api`), then Frontend-Implementer (the page in `assets/`)
**Agent guide**: `agents/backend.md`, `agents/frontend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*, *AI detail*)
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read your agent guide (`agents/backend.md` or `agents/frontend.md`)
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in your agent guide
6. Read `docs/ddr/0009-own-dashboard-window.md`, `docs/ddr/0006-obsidian-as-the-dashboard.md`, `docs/ddr/0004-ai-detail-in-separate-files.md`, `BRAINSTORMING_LOG_dashboard-window.md`, PRD US-019, FR-034, FR-042, FR-044, FR-027, NFR-001, NFR-002, NFR-013
7. Read the design board "Dashboard (clickable)" in `docs/design/Main.dc.html` (the `filterDefs`, day groups, row and detail-panel markup and the CSS values are the spec for the page)
8. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/obsidian/`, `~/.config/easy-note/` or `~/.local/share/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-19: the design board "Dashboard (clickable)" is "what I expected"; it has a `+ New note` button and a capture dialog (Enter save, Shift+Enter newline, Esc cancel, Backspace on empty drops the context chip). DDR 0009 item 5 puts it in slice 3.

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> In the dashboard window, `+ New note` opens the design's capture dialog. The note is saved by the same path as `easy-note add`, with the same context rule as `quick`, and shows in the list at once.

**Out of scope** (what this task explicitly does NOT do):

* Replacing `quick`'s zenity/kdialog box (stays as is)
* A global hotkey; the hotkey chip
* Attachments, tags typed in the dialog beyond what `add` parses

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-044: capture dialog in the window
* FR-027: last-context rule
* FR-001, FR-014: a note is never refused
* US-019, US-014

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent) — auto (Supervisor) 2026-11-19
- [X] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy) — auto (Supervisor) 2026-11-19
- [X] Every Acceptance Criterion below traces to a line in the Requirement — auto (Supervisor) 2026-11-19
- [X] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above — auto (Supervisor) 2026-11-19

---

## Dependencies & Reachability

**Depends on**: HUN-278 (slice 1, read-only window, FR-042); not blocked by slice 2

**Entry point**: new `window::api` commands `context` and `add`; `+ New note` button and dialog in `src/surfaces/window/assets/`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `context` returns the FR-027 last context (newest `cli` or `hook` note < 2 h old: repo, task) or null, masked | FR-044, FR-027 |
| 2 | `add {text, keep_context: bool}` saves through the same core path as `easy-note add` with `source: quick` (a new source value would make older binaries skip the note: `docs/NOTE_FORMAT.md:55,80`, DDR 0002), context from AC 1 when `keep_context`; returns the new item | FR-044 |
| 3 | Dialog keys: Enter saves, Shift+Enter adds a newline, Esc closes without saving, Backspace in an empty box removes the context chip (then `keep_context` = false) | FR-044 |
| 4 | Empty or whitespace-only text: Enter does nothing (no note) | FR-044 |
| 5 | After save the dialog closes, the design's saved toast shows, and the new note is selected at the top of `today` | FR-044 |
| 6 | A secret in the text is masked on write exactly as `add` does (guard) | NFR-002 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | last cli note 30 min old in repo `r` | `context` → repo `r` | `tests/window_api.rs` |
| 2 | `add` with and without `keep_context` | note file with / without repo, same front matter as `easy-note add` | same |
| 3 | text with a planted secret | stored masked | same (leak) |
| 4 | empty text | `{"error": "empty"}`, no file | same |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features tray -- -D warnings \
  && cargo test --locked --features tray \
  && cargo clippy --all-targets --locked --features window -- -D warnings \
  && cargo test --locked --features window
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-280.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-280.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Capture dialog, with and without context chip | screenshot with stub IPC vs the design board | same layout and texts |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Dialog styles | CSS values vs `docs/design/Main.dc.html` | copied |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| 1280 px and 800 px | screenshot | dialog centred, fits |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-019 | jot a note from the board | `+ New note`, type, Enter | the note is at the top of today |
| US-019 | note without the project | Backspace in the empty box, type, Enter | the note has no repo |

---

## Approach

**Pattern reference**: `src/surfaces/cli.rs` `add` and `quick` (last-context rule, FR-027); slice 1's `window::api::handle`.

**Vital slice**: `context` + `add` in `window::api` (tested), then the dialog in the page.

**Cut list**: replacing `quick`, a hotkey, attachments.

---

## Edge Case Checklist

- [ ] Dialog open while the window loses focus and regains it → the board refresh does not close the dialog or drop the text
- [ ] Very long text → saved whole, as `add` does
- [ ] IME composition: Enter while composing does not save
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/window/api.rs` | `context`, `add` |
| `src/surfaces/window/assets/*` | button, dialog, toast |
| `tests/window_api.rs` | AC 1–6 |
| `README.md` | window capture line |
| Pre-existing pin tests | `tests/window_api.rs` (slice 1), `tests/cli.rs`, `tests/readme.rs` — list each and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*` | Supervisor-owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| `src/core/*` | Call existing public functions only; a missing one is a `Question:` to the Supervisor |
| `src/core/format.rs`, note files | DDR 0002 |

---

## Test Plan

`window::api` commands are pure and tested in the default build with temp `HOME`, `XDG_CONFIG_HOME`, `EASY_NOTE_DIR` (and a fake `xdg-open` on a temp `PATH` where a command opens something). Asset scan test from slice 1 stays green. Owner-only: the UX rows on the real machine.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_HUN-280.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-19 DECIDED window capture writes `source: quick`; no new source value (Supervisor, RULE `docs/NOTE_FORMAT.md:55,80`, DDR 0002)
- 2026-11-20 DECIDED a note saved from the window dialog starts `enrich::after_save` when `[ai] enrich` and `after_save` are on, as `easy-note add` does (owner: "yes, should be")
