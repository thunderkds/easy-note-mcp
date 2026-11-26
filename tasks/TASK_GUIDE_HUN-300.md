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
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `config`, `AiSettings`, `after_save`, `window`, `HUN-275`, `HUN-278` before reading source files
 8. Read `PRD.md` (US-020, US-023, FR-038, FR-048, FR-052, NFR-014), `GRILL_round6.md`, `BRAINSTORMING_LOG_round6.md` (Path B), `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0007-ai-detail-from-the-local-claude-cli.md`
 9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`. A fake `claude` script stands in for the model; no test starts the real CLI
10. Round 6 rule: **a raw note file is never changed by the AI** (NFR-012, NFR-015). If your change needs one, stop and write a `Question:`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-25: "easy-note is the place where I only take notes; the AI does the rest — arranges the notes, checks drafts, enriches them on a schedule." Decisions: `GRILL_round6.md` G1–G5, Path B in `BRAINSTORMING_LOG_round6.md`.

**Restated intent**:

> A gear icon opens Settings. The person turns the AI on or off, turns on "Organize automatically" (the scheduler) after reading what is sent, picks how often and how much, and which jobs run. The window writes config.toml through one core function.

**Out of scope**:

* Notes dir change
* Hotkey or tray settings
* Per-note settings
* Editing `command` to an arbitrary program from the window (it stays file-only, as a safety rule)

**Requirement Refs**:

* FR-048: the keys and the opt-in notice
* FR-052: the Settings screen fields
* NFR-014: opt-in text and the switch are the only way to `schedule = "tray"` from the window
* US-020, US-023

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [x] Restated intent confirmed to match the user's request
- [x] Domain terms align with `PROJECT_SPEC.md` glossary
- [x] Every Acceptance Criterion below traces to a line in the Requirement
- [x] All Requirement Refs exist in `PRD.md` and are covered

Signed: auto (Supervisor), 2026-11-26.

---

## Dependencies & Reachability

**Depends on**: HUN-293 (`[ai]` keys)

**Entry point**: Window gear icon; IPC `settings_get`, `settings_save`; core `config::save_ai_settings`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `config::save_ai_settings(&AiSettings)` writes only the `[ai]` keys it owns and keeps every other key and table of config.toml (and the comments, when `toml_edit` is already a dependency; adding a dependency needs the Supervisor's OK); atomic temp + rename; invalid input is refused with the key named | FR-048 |
| 2 | Settings shows: AI on/off (`enrich`), Organize automatically (`schedule`), interval, notes per run, batch size, jobs (three checkboxes), model, account folder, `Run after each save`; values come from `settings_get` and are validated by the same core rules | FR-052 |
| 3 | Turning on Organize automatically first shows the notice once: `Masked note text is sent to your own claude CLI (account folder: <dir>, model: <model>). Nothing runs until you confirm.` with Confirm and Cancel; the switch changes only on Confirm; the confirmed notice text version is stored (`[ai] notice_seen`) so it is not shown again for the same text | NFR-014, G1 |
| 4 | `command` and `claude_config_dir` are shown read-only (edit in the file); the screen says where the file is | FR-048 |
| 5 | Saving shows a toast; the tray picks the change up at its next tick (no restart); `doctor` shows the saved values | FR-049 |
| 6 | `schedule` cannot be set to `tray` while `enrich` is off: the switch is disabled with the reason | NFR-014 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | save_ai_settings over a config with other tables and comments | other content unchanged | unit test |
| 2 | page harness: toggle schedule, Cancel the notice | no `settings_save` call | page harness |
| 3 | confirm the notice | `schedule = "tray"` written, `notice_seen` set | tests/window_api.rs |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features window,tray -- -D warnings \
  && cargo test --locked --features window,tray
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-300.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-300.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Gear icon; Settings panel (off, on, error); the notice dialog | easy-ui screenshots with stub IPC | same dialog language as the capture dialog; disabled state readable |

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
| US-020 | turn on the AI organizer | gear, switch, read notice, Confirm | switch on, toast `Saved`; no run before Confirm |
| US-023 | change how often | gear, interval 30, Save | toast; file shows 30 |

---

## Approach

**Pattern reference**: `core::config` (`ai_of`, error style); the capture dialog and toast code in `app.js`; HUN-278's IPC guard.

**Vital slice**: Core save function + two IPC commands + the panel with the notice.

**Cut list**: Import/export of settings; per-job models; a test-connection button.

---

## Edge Case Checklist

- [ ] config.toml missing: create it with only `[ai]`
- [ ] config.toml invalid TOML: refuse to save, name the file, change nothing
- [ ] File changed on disk after `settings_get`: refuse the save with `changed outside`, as Edit does
- [ ] Interval typed as 0 or text: refused with the key named
- [ ] Window opened with no `window` write permission to `~/.config`: clear toast
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for text this change makes wrong

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/core/config.rs` | `save_ai_settings`, `notice_seen` |
| `src/surfaces/window/api.rs` | `settings_get`, `settings_save` |
| `src/surfaces/window/assets/app.js`, `app.css`, `index.html`, `ipc.js` | gear icon, panel, notice dialog |
| `tests/window_api.rs`, `tests/enrich.rs`, page harness | AC 1–6 |
| `README.md` | Settings paragraph |
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

Config round-trip unit tests with a rich fixture; IPC tests; page harness for the notice flow.

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

* 2026-11-25 OPEN HITL: owner approves the wording of the notice before merge* 2026-11-26 DEBATE add toml_edit as a direct dependency so the Settings save keeps comments → yes, add toml_edit 0.25 (already locked) and edit only the owned keys (Advocate 1: add toml_edit; Advocate 2: add toml_edit) ⚠ decided for you — please review
* 2026-11-26 DECIDED by the owner: the notice wording and the switch caption are approved as written; this closes the open HITL item of 2026-11-25.
* 2026-11-26 DECIDED by the owner: a save keeps a symlinked config.toml (write through the link) before the merge.
