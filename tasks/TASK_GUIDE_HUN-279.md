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
6. Read `docs/ddr/0009-own-dashboard-window.md`, `docs/ddr/0006-obsidian-as-the-dashboard.md`, `docs/ddr/0004-ai-detail-in-separate-files.md`, `BRAINSTORMING_LOG_dashboard-window.md`, PRD US-019, FR-034, FR-042, FR-043, FR-032, NFR-001, NFR-002, NFR-013
7. Read the design board "Dashboard (clickable)" in `docs/design/Main.dc.html` (the `filterDefs`, day groups, row and detail-panel markup and the CSS values are the spec for the page)
8. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/obsidian/`, `~/.config/easy-note/` or `~/.local/share/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-19: the design board "Dashboard (clickable)" is "what I expected"; its detail panel has `Add as tags`, `Open in editor`, `Copy id`, `Move to trash` and an Undo toast. DDR 0009 item 5 puts these in slice 2.

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> In the dashboard window, the person acts on the selected note with the design's four buttons. Each button does exactly what the matching CLI command does, through the same core function, and nothing else changes a note file.

**Out of scope** (what this task explicitly does NOT do):

* The capture dialog (slice 3)
* Editing note text in the window
* Bulk actions; keyboard shortcuts beyond the design's
* `trash --empty` from the window

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-043: window actions
* FR-032: `tag --accept-ai`, `rm`/`restore` carry the AI file
* NFR-001: `xdg-open` only (already allowed for `dashboard`)
* US-019

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent) — auto (Supervisor) 2026-11-19
- [X] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy) — auto (Supervisor) 2026-11-19
- [X] Every Acceptance Criterion below traces to a line in the Requirement — auto (Supervisor) 2026-11-19
- [X] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above — auto (Supervisor) 2026-11-19

---

## Dependencies & Reachability

**Depends on**: HUN-278 (slice 1, read-only window, FR-042)

**Entry point**: new `window::api` commands `accept_ai`, `trash`, `restore`, `open_in_editor`; buttons in the detail panel of `src/surfaces/window/assets/`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `accept_ai {id}` adds the AI `tags_suggested` to the note's tags through the same core path as `easy-note tag <id> --accept-ai` (no duplicate tags, text unchanged byte-for-byte); returns the updated `detail`. No AI file → `{"error": "no AI detail"}` | FR-043, FR-032 |
| 2 | `trash {id}` = `Store::rm_with_ai`; returns `{trashed: id}`; the page removes the row and shows the design's toast with `Undo` for 6 s | FR-043 |
| 3 | `restore {id}` = `Store::restore_with_ai`; Undo calls it; the row comes back in its place. Undo after the 6 s is not offered | FR-043 |
| 4 | `open_in_editor {id}` runs `xdg-open <note file path>` as an argv list (no shell), path resolved only through `Store` from a validated id; failure → `{"error": …}` shown as a toast | FR-043, NFR-001 |
| 5 | `Copy id` copies the id with the page clipboard API (no IPC); toast `Copied` | FR-043 |
| 6 | Every action command validates the id shape and refuses an id not in the store (`{"error": "no such note"}`); a trashed id is refused by `accept_ai`/`open_in_editor` | FR-043, DDR 0009 |
| 7 | After any action the counts and filter counts update (the page asks `board` again) | FR-043 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | note with AI tags `["x","y"]`, note tags `["y"]` | `accept_ai` → tags `["y","x"]` (CLI order rule), body bytes equal | `tests/window_api.rs` |
| 2 | `trash` then `restore` | note file and AI file back at their paths, contents equal | same |
| 3 | fake `xdg-open` on PATH | `open_in_editor` calls it once with the note path | same |
| 4 | bad / unknown / trashed ids | error objects, no change on disk (fs snapshot) | same |

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

> **Moved.** See `tasks/TASK_REVIEW_HUN-279.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-279.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Detail panel actions, Undo toast | screenshot with stub IPC vs the design board | same buttons, toast text and place |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Button and toast styles | CSS values vs `docs/design/Main.dc.html` | copied |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| 1280 px and 800 px | screenshot | buttons fit, toast does not cover the list |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-019 | take the AI tags | click `Add as tags` | tags show on the note; `easy-note show <id>` lists them |
| US-019 | undo a mistake | `Move to trash`, then `Undo` | the note is back, nothing lost |

---

## Approach

**Pattern reference**: `src/surfaces/cli.rs` `tag --accept-ai`, `rm`, `restore` handlers (reuse the same core calls); `src/surfaces/dashboard.rs` `open()` (argv `xdg-open`). Slice 1's `window::api::handle` dispatch.

**Vital slice**: the four commands in `window::api` (tested), then the buttons and toast in the page.

**Cut list**: bulk actions, edit text, keyboard shortcuts, `trash --empty`.

---

## Edge Case Checklist

- [ ] A note edited on disk while selected → action works on the current file (read fresh)
- [ ] Undo after the note was restored another way → error toast, no crash
- [ ] Clipboard API refused → toast with the id to copy by hand
- [ ] Every response masked (slice 1 leak test extended to the new commands)
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/window/api.rs` | four commands |
| `src/surfaces/window/assets/*` | buttons, toast, Undo |
| `tests/window_api.rs` | AC 1–7 |
| `README.md` | window actions line |
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
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_HUN-279.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-19 DECIDED actions reuse the CLI's core calls; Undo window 6 s (Supervisor, design toast)
