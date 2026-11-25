**Date**: 2026-11-25
**Complexity Level**: C1
**Risk Level**: Low
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

 1. Read `PROJECT_SPEC.md` (glossary: *Note*, *AI detail*, *Project*, *Topic*, *Kind*, *Override*)
 2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
 3. Read this file completely
 4. Read your agent guide (`agents/backend.md`)
 5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in your agent guide
 6. Read `memory/codebase-map.md`
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `dashboard`, `tray lock`, `window`, `HUN-278`, `HUN-286` before reading source files
 8. Read `PRD.md` (US-023, FR-033, FR-042, FR-045), `GRILL_round6.md`, `BRAINSTORMING_LOG_round6.md` (Path B), `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0007-ai-detail-from-the-local-claude-cli.md`
 9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`. A fake `claude` script stands in for the model; no test starts the real CLI
10. Round 6 rule: **a raw note file is never changed by the AI** (NFR-012, NFR-015). If your change needs one, stop and write a `Question:`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-25: "easy-note is the place where I only take notes; the AI does the rest — arranges the notes, checks drafts, enriches them on a schedule." Decisions: `GRILL_round6.md` G1–G5, Path B in `BRAINSTORMING_LOG_round6.md`.

**Restated intent**:

> `easy-note dashboard` (tray Open dashboard, a tray recent-note click, the launcher, the CLI) never opens a second dashboard window for the same notes dir. A second call finds the open window, brings it to the front (and shows `--note <id>` when given) and exits 0.

**Out of scope**:

* The capture-only quick window (HUN-289): it keeps two windows at once
* The Obsidian fallback path (no `window` build or no display)
* A window per different notes dir (each notes dir keeps its own window)

**Requirement Refs**:

* FR-045: one window per notes dir, focus on a second call, stale lock never blocks
* FR-033: tray clicks run `dashboard`
* US-023

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are covered

---

## Dependencies & Reachability

**Depends on**: None — can start immediately (tray lock pattern: `src/surfaces/tray.rs` `acquire_lock`)

**Entry point**: `easy-note dashboard [--note ID]` (`src/surfaces/dashboard.rs`, `src/surfaces/window/mod.rs`); tray `Action::OpenDashboard` and `open_note`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | With a dashboard window open, `easy-note dashboard` starts no second window process and exits 0 within 2 s | FR-045 |
| 2 | The open window is brought to the front (present / un-minimize); where the window manager refuses focus, the window requests attention and the exit code is still 0 | FR-045 |
| 3 | `easy-note dashboard --note <id>` makes the open window show that note | FR-045, FR-033 |
| 4 | The lock is `<notes dir>/.easy-note/window.lock`, an exclusive OS lock held while the window lives; a window that was killed leaves a lock the next start takes over | FR-045 |
| 5 | A tray recent-note click and tray Open dashboard use the same path (no change to the tray menu text) | FR-033 |
| 6 | A different notes dir (`EASY_NOTE_DIR`) opens its own window; the quick window and the Obsidian fallback are unchanged | FR-045, FR-022 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | window lock held by a live process | second call exits 0, sends a focus request, starts no window | unit test of the lock + request channel (temp notes dir) |
| 2 | lock file left by a dead process | new window starts normally | test: lock a file in a child, kill the child, start |
| 3 | `--note <id>` on a held lock | the request carries the id; the window handler shows it | page harness / `tests/window_api.rs` |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features window,tray -- -D warnings \
  && cargo test --locked --features window,tray
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-290.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-290.md` (review sections are never taken from the board).

## Approach

**Pattern reference**: `tray::acquire_lock` (exclusive `try_lock`); `dashboard::open_note`; the window's `initial_note_script` (HUN-286) for `--note`.

**Vital slice**: The lock plus a small request channel (a Unix socket `<notes dir>/.easy-note/window.sock`, line `focus [id]`) that the window thread reads and turns into a present call.

**Cut list**: Remembering window position and size; a request channel for other commands; Windows or macOS.

---

## Edge Case Checklist

- [ ] Socket path longer than the 108-byte limit: fall back to a short path under `$XDG_RUNTIME_DIR` named by a hash of the notes dir, and test it
- [ ] Two `dashboard` calls at the same moment: exactly one window, the other focuses it
- [ ] The window is still starting when the second call arrives: the second call waits up to 2 s for the socket, else exits 0 with one line
- [ ] Wayland refuses to raise a window: no crash, attention request only
- [ ] The lock dir `.easy-note/` does not exist yet: create it (as the tray does)
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for text this change makes wrong

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/dashboard.rs` | take the lock first; on a held lock send the request and exit |
| `src/surfaces/window/mod.rs` | listen on the socket, present the window, show a note on request |
| `tests/dashboard.rs`, `tests/window_api.rs` | AC 1–6 |
| `README.md`, `RUNBOOK.md` | one line: a second click focuses the window |
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

Lock and request channel as unit tests over a temp notes dir (a child process holds the lock). The `--note` request through the page harness. Owner-only: tray clicks on the real desktop.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] **easy-verifier**: Stage 4 `Skill({ skill: "score" })` run by the Supervisor
- [ ] Security review: not required (Low risk)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md` (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-25 DECIDED a Unix socket beside the lock is the request channel (Supervisor; no new dependency)