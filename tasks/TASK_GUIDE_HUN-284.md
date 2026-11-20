**Date**: 2026-11-20
**Complexity Level**: C1
**Risk Level**: Medium
**Priority**: P2
**Assigned agent**: Frontend-Implementer (page and the CSP / origin checks in `src/surfaces/window/mod.rs`)
**Agent guide**: `agents/frontend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*, *AI detail*)
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read your agent guide (`agents/frontend.md`)
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in your agent guide
6. Read `docs/ddr/0009-own-dashboard-window.md`, PRD FR-042, FR-043, NFR-002, the HUN-278 review notes in `tasks/TASK_REVIEW_HUN-278.md` (`Checks:` line), and the design board "Dashboard (clickable)" in `docs/design/Main.dc.html`
7. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/easy-note/` or `~/.local/share/`

---

## Requirement (Pillar 1 — Adapt the requirement)

HUN-278 security-review (2 P3) and code-review (1 P2) findings, filed by the Supervisor as Linear HUN-284 on 2026-11-20; the owner said "yes" to adding the review findings to the board and "start all the tasks as parallel".

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> The window's page runs under a stricter Content Security Policy with no `'unsafe-inline'` styles, and the window accepts navigation and IPC only from the exact origin `easy-note://localhost`. At the 800 px minimum width, clicking a note brings its detail panel into view.

**Out of scope** (what this task explicitly does NOT do):

* Any new feature or visual redesign
* The toast placement at 800 px (kept as designed in HUN-279)
* Performance work (HUN-282)

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-042: the page is embedded and loads nothing from the network; reaches notes only through IPC
* FR-043: the detail panel the actions live in
* NFR-002: no change to masking

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent) — Supervisor 2026-11-20
- [X] Domain terms align with `PROJECT_SPEC.md` glossary — Supervisor 2026-11-20
- [X] Every Acceptance Criterion below traces to a line in the Requirement — Supervisor 2026-11-20
- [X] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above — Supervisor 2026-11-20

---

## Dependencies & Reachability

**Depends on**: HUN-279 (branch `feat/HUN-279-actions`, not merged yet: this task's branch is cut from it, because both change the page)

**Entry point**: `pub const CSP` and `allowed_url` in `src/surfaces/window/mod.rs`; the row click handler (`select`) in `src/surfaces/window/assets/app.js`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `CSP` has no `'unsafe-inline'`; every style the page needs is in `app.css` (no `style=` attribute and no `element.style` write that the CSP would block; check with the asset scan) | stricter CSP |
| 2 | The real window renders the board unchanged under the new CSP (screenshot of the real window, not only headless) | stricter CSP |
| 3 | `allowed_url` accepts only `easy-note://localhost/...` (other hosts on the `easy-note` scheme, other schemes, and `easy-note://localhost.evil/` are refused) | exact origin |
| 4 | The IPC handler ignores a request whose source URL is not `easy-note://localhost` (if wry exposes the source; if it does not, record that fact with the wry API line as a Citation and keep AC 3 as the guard) | exact origin |
| 5 | At 800 px, clicking a row scrolls the detail panel into view (top of the panel visible); at 1280 px nothing scrolls | 800 px |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | `CSP` constant | no `unsafe-inline` | `tests/window_api.rs` |
| 2 | URLs `easy-note://localhost/index.html`, `easy-note://other/`, `easy-note://localhost.evil/`, `https://x/` | first allowed, rest refused | `tests/window_api.rs` |
| 3 | all served assets | no inline style the CSP blocks | asset scan test |
| 4 | real window, 800 px, click the third row | panel top visible | screenshot pasted |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features window -- -D warnings \
  && cargo test --locked --features window
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-284.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-284.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Board and detail panel at 1280 px | real-window screenshot vs the HUN-279 screenshots | no visible change |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Styles moved from inline to `app.css` | values grepped in `docs/design/Main.dc.html` | same values |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| 800 px | real-window screenshot after a row click | panel in view, no horizontal scroll |
| 1280 px | screenshot | two columns, no scroll jump |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-019 | read a note in a narrow window | 800 px, click a row | the detail shows without scrolling by hand |

---

## Approach

**Pattern reference**: `src/surfaces/window/mod.rs` `CSP`, `allowed_url`, `with_ipc_handler`; `tests/window_api.rs` CSP and navigation-guard tests.

**Vital slice**: AC 1–3 (the CSP and origin pin); AC 5 is one `scrollIntoView` call guarded by the layout width.

**Cut list**: Trusted Types; a nonce-based CSP.

---

## Edge Case Checklist

- [ ] SVG icon templates with `stroke`/`fill` attributes still render (attributes, not inline style)
- [ ] Toast and filter bars still render without inline style
- [ ] `prefers-reduced-motion`: scroll without smooth animation
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/window/mod.rs` | CSP, `allowed_url`, IPC source check |
| `src/surfaces/window/assets/*` | move inline styles to `app.css`; scroll the panel at 800 px |
| `tests/window_api.rs` | AC 1–4 |
| Pre-existing pin tests | `tests/window_api.rs` CSP / navigation tests — list each and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*` | Supervisor-owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| `src/core/*` | not needed |
| `src/surfaces/window/api.rs` | HUN-282 / HUN-280 |

---

## Test Plan

Default-build tests for the CSP string and `allowed_url`; the asset scan for inline styles; a real-window run (`GDK_BACKEND=x11` on a private Xvfb, temp env) with screenshots at 1280 and 800 px.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_HUN-284.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log
