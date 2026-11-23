**Date**: 2026-11-23
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer (Rust: `window::api` command, the window shell's async reply), then Frontend-Implementer (the page in `src/surfaces/window/assets/`)
**Agent guide**: `agents/backend.md`, `agents/frontend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

 1. Read `PROJECT_SPEC.md` (glossary: *Note*, *AI detail*, *Inbox*)
 2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
 3. Read this file completely
 4. Read your agent guide (`agents/backend.md` or `agents/frontend.md`)
 5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in your agent guide
 6. Read `memory/codebase-map.md`
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `enrich`, `claude -p`, `window`, `HUN-275`, `HUN-280` before reading source files
 8. Read `docs/ddr/0007-*.md` (AI detail from the person's own `claude`), `docs/ddr/0009-own-dashboard-window.md`, `docs/design/Main.dc.html` (detail panel, AI detail section), PRD US-016, US-018, US-019, FR-038, FR-043, NFR-012
 9. **UI task — screenshots**: the page is a web page in a webview: take screenshots with easy-ui on the page with a stub IPC, as HUN-278/HUN-280/HUN-286 did, and judge them with `Skill({ skill: "ux-evaluate" })`
10. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`; never start the real `claude` (use a stub program on `PATH`, as the HUN-275 tests do)

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-22: "The AI is not run through the button". Asked which button, the owner picked: "Add a 'Run AI' button" in the note's detail panel.

Today: the detail panel shows the AI detail when `<notes dir>/.easy-note/ai/<id>.json` exists, and offers `Add as tags` for the AI tags. Nothing in the window starts the AI step: it runs only from `easy-note ai enrich <id>` / `--pending`, or after a save when `[ai] enrich = "claude"` and `after_save = true` (`enrich::after_save`). The owner's machine has no `~/.config/easy-note/config.toml`, so the AI step never runs there.

**Restated intent**:

> The detail panel has a `Run AI` button. One click runs the same AI step as `easy-note ai enrich <id>` on that note, without blocking the window. The panel shows it is running, then shows the new AI detail, or the error in words the person can act on (enrich off, `claude` not found, not logged in, timed out).

**Out of scope**:

* Running AI on many notes from the window (`--pending`)
* Editing the AI detail
* A setting screen for `[ai]` in the window
* Changing what `ai enrich` writes or how it calls `claude` (DDR 0007 stays as is)
* Inbox notes: AI detail for them works as `ai enrich` does today; no new rule

**Requirement Refs**:

* FR-043: detail panel actions (gets `Run AI`: Supervisor PRD edit at Stage 2)
* FR-038, US-018, DDR 0007: AI detail from the person's own `claude` CLI, opt-in
* US-016, NFR-012: AI detail never changes the note file
* US-019: handle notes in the app

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are covered (FR-043 wording to be updated by the Supervisor)
- [ ] Questions Log Q1 answered by the owner

---

## Dependencies & Reachability

**Depends on**: HUN-286 (merged 2026-11-22: the detail panel's current buttons and `base` handling)

**Entry point**: `window::api` command `enrich`; the `Run AI` button in the detail panel (`src/surfaces/window/assets/app.js`)

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `{"cmd": "enrich", "id"}` runs the same per-note step as `easy-note ai enrich <id>` (same settings, same `claude -p` argv and env, same AI file and `ai/<id>.md` view) and answers with the note's `detail`, its AI detail filled in | restated intent |
| 2 | The call does not block the window: the window keeps answering `board`, `detail` and the focus refresh while `claude` runs; the answer reaches the page when the child ends | restated intent |
| 3 | Errors come back as `{"error": "<words>"}` and write nothing: enrich off (per Q1), `claude` not found, not logged in (the `login_hint` text), timed out, bad answer from `claude`, unknown / trashed / bad id | restated intent |
| 4 | The note file is byte-for-byte unchanged by `enrich` | NFR-012 |
| 5 | Panel: `Run AI` sits in the actions row; for a note with AI detail it reads `Re-run`. While running, the button is disabled with `AI running…`; a second click on the same note sends nothing. When the answer comes, the AI detail section shows the new title, kind, summary and suggested tags; an error shows as a toast and the button comes back | restated intent |
| 6 | Selecting another note while AI runs does not lose the answer: the board row gets its AI mark, and coming back to the note shows the AI detail | restated intent |
| 7 | Every string in the answer goes through `guard::mask_for_output` (the same `mask_all` as every other command) | NFR-002 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | a note, a stub `claude` on `PATH` that prints a valid answer | `enrich` answers `detail` with that AI detail; AI file written; note file unchanged | `tests/window_api.rs` |
| 2 | stub `claude` missing / exits 1 with a login message / sleeps past the timeout / prints bad JSON | errors per AC 3; no AI file written | same |
| 3 | enrich off in config | error per Q1 | same |
| 4 | a slow stub `claude` | a `board` request answered while it runs | shell test or a `window::api` test of the threading seam |
| 5 | page with stub IPC: click Run AI, answer later; click again while running; switch note while running | AC 5, 6 | easy-ui run + screenshots |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features window,tray -- -D warnings \
  && cargo test --locked --features window,tray
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-288.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-288.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Detail panel actions row, idle / running / after answer / error | easy-ui screenshots with stub IPC | `Run AI` uses the existing `act` button style; nothing else in the panel moves |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Colors match design tokens | CSS audit vs `docs/design/graphite-mint-tokens.json` | only existing tokens |
| Typography matches spec | computed style | button text in the panel's button font |
| Spacing / layout matches spec | computed style | button in the existing `acts` row |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| 800 px | screenshot | actions row fits without horizontal scroll |
| 1280 px | screenshot | same |
| Mobile | N/A | a desktop window; min width 800 px (HUN-284) |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-016 | get AI detail for one note | select note → Run AI | `AI running…`, then the AI detail section filled in |
| US-018 | claude not set up | Run AI with no `claude` / not logged in | a toast that names the fix (install, log in, or the config line) |
| US-016 | AI detail is old | note with stale AI → Re-run | the stale mark goes, new AI detail shown |

---

## Approach

**Pattern reference**: `enrich::run` / `enrich_one` (the per-note step: reuse it, do not copy it); `window::api::accept_ai` (core call, then `detail`); the window shell's `EventLoopBuilder::<String>::with_user_event()` and `reply_script` in `src/surfaces/window/mod.rs` (how a reply reaches the page).

**Vital slice**: a `window::api` function that runs the per-note enrich step and returns `detail` (tested with a stub `claude`), then the shell runs it off the event-loop thread and sends the reply through the event-loop proxy, then the button.

**Cut list**: a progress bar; cancel; running AI on several notes; an `[ai]` settings screen.

Threading: `claude -p` can take many seconds. The IPC handler must not wait for it on the GTK thread. Run the step on its own thread (or as the child `<exe> ai enrich <id>` like `enrich::after_save`, then read the AI file) and post the reply with the request's `req` through the event-loop proxy. Keep one run per note at a time.

---

## Edge Case Checklist

- [ ] The window is closed while `claude` runs: the child is not left as a zombie, nothing panics
- [ ] The note is trashed or edited while AI runs: the answer for a note no longer on the board is dropped by the page; an edited note gets a stale AI mark as today
- [ ] `claude` is on `~/.local/bin` but the window was started from the desktop with a short `PATH`: `resolve` finds it or the error names `[ai] command`
- [ ] A secret in the AI answer: masked on output (AC 7)
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help, `ipc.js` header for the detail-panel button list

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/window/api.rs` | `enrich` command |
| `src/surfaces/window/mod.rs` | run it off the event-loop thread; reply through the proxy |
| `src/surfaces/enrich.rs` | expose the per-note step for the window (`pub(crate)`), no behaviour change |
| `src/surfaces/window/assets/{app.js,ipc.js,app.css}` | `Run AI` button and its states |
| `tests/window_api.rs`, `tests/fixtures/window/page_harness.js` | AC 1–7 |
| `README.md` | window actions line |
| Pre-existing pin tests | `tests/window_api.rs` commands-the-api-answers list, ipc.js contract pin, button label tests, `tests/readme.rs` — list each and its handling |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*`, `docs/design/*` | Supervisor / owner owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| `src/core/*` | Call existing public functions only; a missing one is a `Question:` to the Supervisor |
| `src/surfaces/quick.rs` | the styled quick window task changes it |

---

## Test Plan

The enrich step over a temp store with a stub `claude` on `PATH` (as `tests/` for HUN-275 do): success, each error, file unchanged. The threading seam: a slow stub and a `board` request answered meanwhile. The page: easy-ui with a stub IPC for the UX rows. Owner-only: Run AI with the real `claude` on the real machine.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] **easy-verifier**: Stage 4 `Skill({ skill: "score" })` run by the Supervisor
- [ ] Security review: `Skill({ skill: "security-review" })` run (Medium risk: the webview can now start a child process)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md` (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-22 DECIDED a `Run AI` button in the detail panel (owner pick, chat 2026-11-22)
* 2026-11-23 DECIDED Q1 = Option A (Supervisor under [admin], owner said "spawn parallel"; owner may override; needs a DDR 0007 note) — original question: with no `[ai]` section (`enrich` off, the default; the owner has no config.toml), should `Run AI` still run? Option A: yes — the click is the opt-in, defaults apply (`command = "claude"`); `after_save` stays off. Option B: no — the toast says `AI is off: set [ai] enrich = "claude" in ~/.config/easy-note/config.toml`, as `easy-note ai enrich` says today. Supervisor recommendation: A, because the person pressed the button for exactly this note; DDR 0007's opt-in is about AI running without being asked. A needs a DDR 0007 note.