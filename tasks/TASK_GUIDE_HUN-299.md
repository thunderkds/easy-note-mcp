**Date**: 2026-11-25
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Frontend-Implementer + Backend-Implementer
**Agent guide**: `agents/frontend.md, agents/backend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

 1. Read `PROJECT_SPEC.md` (glossary: *Note*, *AI detail*, *Project*, *Topic*, *Kind*, *Override*)
 2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
 3. Read this file completely
 4. Read your agent guide (`agents/frontend.md`)
 5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in your agent guide
 6. Read `memory/codebase-map.md`
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `window`, `edit`, `toast`, `trash undo`, `HUN-279`, `HUN-286` before reading source files
 8. Read `PRD.md` (US-021, FR-050, FR-052), `GRILL_round6.md`, `BRAINSTORMING_LOG_round6.md` (Path B), `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0007-ai-detail-from-the-local-claude-cli.md`
 9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`. A fake `claude` script stands in for the model; no test starts the real CLI
10. Round 6 rule: **a raw note file is never changed by the AI** (NFR-012, NFR-015). If your change needs one, stop and write a `Question:`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-25: "easy-note is the place where I only take notes; the AI does the rest — arranges the notes, checks drafts, enriches them on a schedule." Decisions: `GRILL_round6.md` G1–G5, Path B in `BRAINSTORMING_LOG_round6.md`.

**Restated intent**:

> When the AI is wrong, the person fixes it in the window and the fix stays. They can change a note's project, topic or kind, rename a topic or merge two topics for all notes, and undo the last change.

**Out of scope**:

* Editing the note text (the existing Edit)
* Bulk move of many notes by selection
* A history screen

**Requirement Refs**:

* FR-050: overrides win, rename/merge, log and undo
* FR-052: topic rename and merge in the window
* US-021

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are covered

---

## Dependencies & Reachability

**Depends on**: HUN-292 (overrides, log, undo), HUN-298 (views)

**Entry point**: Window IPC `set_field`, `rename_topic`, `merge_topics`, `undo_ai`; each calls the same core function as `ai set` / `ai undo`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | In the detail panel the AI project, topic and kind show as editable chips; a change calls `set_field` (core override write + log line `who: person`) and the row regroups at once | FR-050 |
| 2 | A topic header in the Topic grouping has `Rename` and `Merge into…`; rename writes the rename map; merge rewrites the override for every member note; both log one line per note | FR-050, FR-052 |
| 3 | An Undo toast (6 s, as Move to trash) after each change calls `undo_ai` for that note; a rename or merge undoes all its notes in one click | FR-050 |
| 4 | Fixing a field never changes the note file or `updated`; the next `ai run` keeps the fix (test with the fake `claude` returning another value) | NFR-015, FR-050 |
| 5 | Inbox notes can be fixed too (the override file is ours, not the note) | FR-050 |
| 6 | Values are masked and capped as in the CLI; a bad kind is refused with the allowed kinds in the toast | FR-050 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | `set_field` topic on a note, then fake `ai run` | override still wins | tests/window_api.rs + tests/ai_run.rs |
| 2 | merge topic A into B with 3 notes | 3 overrides, 3 log lines, 1 Undo restores all | tests/window_api.rs |
| 3 | page harness: edit chip, Enter, Esc, Undo toast | IPC calls as expected | page harness |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features window,tray -- -D warnings \
  && cargo test --locked --features window,tray
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-299.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-299.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Detail panel chips (view, editing, error); topic header menu; Undo toast | easy-ui screenshots with stub IPC | chips look like the existing tag chips; menu is keyboard reachable |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Colors match design tokens | CSS audit vs `docs/design/graphite-mint-tokens.json` | only existing tokens |
| Typography matches spec | computed style | bundled Plex fonts, as the dashboard |
| Spacing / layout matches spec | computed style | same spacing scale as the existing board |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| 375 px | easy-ui screenshot, `Skill({ skill: "ux-evaluate" })` | no cut-off text, no horizontal scroll |
| 1280 px | easy-ui screenshot | as the design board |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-021 | fix a wrong topic | open note, click topic chip, type, Enter | row regroups, Undo toast shows |
| US-021 | merge two topics | topic header, Merge into B | all notes move; one Undo restores |

---

## Approach

**Pattern reference**: `edit` / `trash` / `restore` IPC handlers and the Undo toast (HUN-279, HUN-286) in `window/api.rs` and `app.js`.

**Vital slice**: Three IPC commands over the core functions of the overrides slice, chip editing, rename/merge menu, Undo toast.

**Cut list**: Drag and drop; multi-select; keyboard shortcuts (UX slice).

---

## Edge Case Checklist

- [ ] Rename to an existing topic = merge: say so in the confirm text
- [ ] Rename to empty or only spaces: refused
- [ ] A note changed outside the window while editing a chip: the override is for the id, so it still applies
- [ ] Undo after the AI changed the same field later: restores the AI's newer value, not a stale one (per-note log order)
- [ ] Two quick edits then Undo: undoes only the latest
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for text this change makes wrong

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/window/api.rs` | four IPC commands over core functions |
| `src/surfaces/window/assets/app.js`, `app.css`, `index.html`, `ipc.js` | chips, menu, toast |
| `tests/window_api.rs`, `tests/fixtures/window/page_harness.js` | AC 1–6 |
| Pre-existing pin tests | list each test that pins current behavior (help text, asset list, README, MCP tool count) and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*`, `docs/design/*` | Supervisor / owner owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| Note files and the note writer (`Store::update`, `Store::add` in `src/core/store.rs`) | The AI never changes a raw note (NFR-012, NFR-015); call existing public functions only; a missing one is a `Question:` to the Supervisor |
| `src/core/guard.rs` patterns | Secret guard rules are frozen; call them, never edit them |

---

## Test Plan

IPC tests over a temp notes dir; page harness for the chip and toast flows; one end-to-end test with the fake `claude` proving the override survives a run.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] **easy-verifier**: Stage 4 `Skill({ skill: "score" })` run by the Supervisor
- [ ] Security review: `Skill({ skill: "security-review" })` run (note text reaches the model, or a new write path)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md` (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-25 DECIDED scope and order: owner, chat 2026-11-25 ("let go")