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
6. Read `docs/ddr/0009-own-dashboard-window.md`, `docs/ddr/0006-obsidian-as-the-dashboard.md`, `docs/ddr/0004-ai-detail-in-separate-files.md`, `BRAINSTORMING_LOG_dashboard-window.md`, PRD US-019, FR-034, FR-042, NFR-001, NFR-002, NFR-013
7. Read the design board "Dashboard (clickable)" in `docs/design/Main.dc.html` (the `filterDefs`, day groups, row and detail-panel markup and the CSS values are the spec for the page)
8. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/obsidian/`, `~/.config/easy-note/` or `~/.local/share/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-19, after `easy-note dashboard` opened the Obsidian Bases table: "it show the obsidian, not the note board that I expect"; "that is what I expected" + the design canvas link (board "Dashboard (clickable)"); pick "Build own window"; "let merge and start the dashboard".

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> In a `window` build on a desktop, the launcher, the tray's "Open dashboard" and `easy-note dashboard` open easy-note's own window that looks like the design's board and shows the person's real notes: counts, search, kind filters, notes by day and a detail panel with the AI detail. This slice only reads; it never changes a note.

**Out of scope** (what this task explicitly does NOT do):

* Any write: `Add as tags`, `Move to trash` + Undo, `Open in editor`, `Copy id` (slice 2, FR-043). The buttons are not drawn in this slice
* The capture dialog and `+ New note` (slice 3, FR-044). The button is not drawn in this slice
* Header status chips (tray on, hotkey, `N without AI detail`); single-instance window; window size/position memory
* CI job for `--features window` (Supervisor adds it after merge; `.github/` is not yours)
* macOS/Windows

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-042: the read-only board window
* FR-034: unchanged fallback without the feature or a display
* NFR-001: no network; NFR-002: masked output; NFR-013: first paint < 1 s at 5,000 notes
* US-019, US-017

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent) — auto (Supervisor) 2026-11-19, owner picked "Build own window"
- [X] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy) — auto (Supervisor) 2026-11-19
- [X] Every Acceptance Criterion below traces to a line in the Requirement — auto (Supervisor) 2026-11-19
- [X] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above — auto (Supervisor) 2026-11-19

---

## Dependencies & Reachability

**Depends on**: HUN-273, HUN-276, HUN-277 (all merged)

**Entry point**: `dashboard` (`dashboard::run` in `src/surfaces/dashboard.rs` chooses the window when `cfg(feature = "window")` and a display is set); new `window::api::handle`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `Cargo.toml`: feature `window = ["dep:wry", "dep:tao"]`, both optional, exact versions pinned by `Cargo.lock`. `cargo build` (no features) and `cargo build --features tray` pull no GUI crate (`cargo tree -e normal` shows neither `wry` nor `tao`) | DDR 0009 item 1 |
| 2 | Pure `dashboard::use_window(built: bool, wayland: Option<&str>, display: Option<&str>) -> bool`: true only when built and one of the two vars is non-empty. `run()` opens the window when true, else the FR-034 path unchanged (existing `tests/dashboard.rs` stays green unchanged) | FR-042, FR-034 |
| 3 | `window::api::handle(store, request_json) -> response_json` is pure (no window, no display) and testable from `tests/window_api.rs`. Commands: `board` → `{notes_dir, today, total, projects, top_projects:[{name,n}]≤3, notes:[…]}`; `detail {id}` → one note. Unknown command, missing field, or an id that is not the note-id shape → `{"error": "<short text>"}`, never a panic | FR-042, DDR 0009 item 3 |
| 4 | Each `notes[]` item: `id, created, time ("HH:MM" local), day ("today"\|"yesterday"\|"earlier", local zone, same rule as query `day`), first (first line), text, repo, task, tags, source, kind (AI kind or null; core `todo` sent as `to-do` to match the design), ai (bool), stale (bool)`. Order: newest first. Skipped files are not in `notes` and do not crash | FR-042 |
| 5 | `detail` adds `masked` (true when the stored text has a masked secret), `context` (branch/commit/session when present) and `ai: {title, kind, summary, tags_suggested, model, at, stale}` or null, from `ai::read` | FR-042, FR-032 |
| 6 | Every string in every response passes `guard::mask_for_output` (leak test: a note and an AI file with planted secrets from `tests/fixtures`; the response JSON contains none of them) | NFR-002 |
| 7 | Counts: `today` = notes with local day today; `total` = all notes; `projects` = distinct non-empty `repo`; `top_projects` = the 3 largest by count, ties by name | FR-042 |
| 8 | The page (embedded with `include_bytes!`/`include_str!`, served by a custom `easy-note://` protocol): header with app mark and notes dir chip; counts strip; search box (every lower-cased word must appear in text, repo, task or a tag); filters All, Bugs, Ideas, Questions, To-dos, No AI detail, Stale with counts, exactly as the design's `filterDefs`; day groups with the design's labels (`today`, `yesterday`, `earlier this week`); rows (time, first line, where-line `repo · task · #tags` or `no project`, kind chip, AI / stale mark); clicking a row shows the detail panel. A note without AI detail shows the design's "No AI detail yet" text. Write buttons and `+ New note` are not drawn | FR-042 |
| 9 | Safety: CSP `default-src easy-note:; style-src easy-note: 'unsafe-inline'` or stricter; a navigation handler allows only `easy-note://`; no devtools in a release build; every note string is set with `textContent` (grep test over `assets/*.js`: no `innerHTML`, `outerHTML`, `insertAdjacentHTML`, `document.write`, `eval`, `new Function`); IBM Plex Sans + Mono woff2 (OFL, licence file next to them) bundled; no `http:`/`https:` URL in any asset (test) | NFR-001, DDR 0009 item 2 |
| 10 | The page asks `board` on load and again on each window focus, so a note added while the window is open shows without a restart | FR-042 |
| 11 | `WEBKIT_DISABLE_DMABUF_RENDERER=1` is set for the process only when it is unset, before the webview starts | DDR 0009 Consequences |
| 12 | NFR-013: `tests/perf.rs`-style test (ignored by default, like the existing perf tests) times `handle(board)` over 5,000 synthetic notes in a temp dir; p95 is printed; < 300 ms for the handler leaves the page its share of 1 s. The owner measures first paint on the real machine | NFR-013 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | temp notes dir with 4 notes (today, yesterday, last week, one with AI detail, one stale) | `board` returns counts 1/4/…, items in order, right `day`, `kind`, `ai`, `stale` | `tests/window_api.rs` |
| 2 | `detail` for the AI note; for a note with no AI file | ai object with model/at/summary; `ai: null` | same |
| 3 | requests `{}`, `{"cmd":"nope"}`, `{"cmd":"detail","id":"../../etc/passwd"}` | `{"error":…}` each, no panic, no file read outside the store | same |
| 4 | planted-secret fixtures | no secret in any response | same (leak) |
| 5 | `use_window` truth table | true only built + display | unit |
| 6 | asset scan | no unsafe DOM sinks, no remote URL | `tests/window_api.rs` or `tests/window_assets.rs` |
| 7 | `cargo tree` without `window` | no `wry`/`tao` | Verification Command |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features tray -- -D warnings \
  && cargo test --locked --features tray \
  && cargo clippy --all-targets --locked --features window -- -D warnings \
  && cargo test --locked --features window \
  && ! cargo tree -e normal --locked | grep -E '^.*(wry|tao) v'
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-278.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-278.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| Board, 1280 px, sample notes | screenshot of the page (`assets/index.html` with a stub `window.ipc` fed the `board` JSON of the test fixture) next to the design board "Dashboard (clickable)" | same layout, colours, fonts and labels; differences listed |
| Detail panel with AI detail / without / stale | same | design's texts: `AI detail (not your words)`, "No AI detail yet", stale warning |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Graphite Mint colours, IBM Plex fonts, spacing | compare CSS values with `docs/design/Main.dc.html` | values copied, not approximated |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| 1280 px | screenshot | two columns: notes + detail |
| 800 px (smallest window) | screenshot | detail panel below or beside without overlap; no horizontal scroll |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-019 | see my notes on the board I designed | click the launcher (window build installed) | the window shows the board with the real notes (owner-run) |
| US-019 | find one note | type two words in search, pick a kind filter | only notes matching every word and the kind show; counts on filters match |
| US-019 | read the AI detail | click a note with AI detail | detail panel shows it labelled `AI detail (not your words)` |

---

## Approach

**Pattern reference**: feature gate like `tray` (`Cargo.toml`, `#[cfg(feature = "tray")]` in `src/surfaces/mod.rs` / `cli.rs`); `src/surfaces/dashboard.rs` `choose()` (pure choice, tested); reads like `src/surfaces/mcp.rs` / `cli.rs` `show` (masking, AI detail label); `tests/dashboard.rs` (temp env).

**Vital slice**: `window::api::handle` for `board` + `detail` (AC 3–7, fully tested without a display), the window shell that serves the page and forwards IPC (AC 9–11), and the page port of the board (AC 8).

* `src/surfaces/window/mod.rs` (`cfg(feature = "window")`): tao event loop + wry `WebViewBuilder` with `with_custom_protocol("easy-note", …)`, `with_ipc_handler`, `with_navigation_handler`; answers by `evaluate_script("window.__easyNote.reply(<json>)")` with the JSON passed as a JS string literal via `serde_json::to_string` (never string-concatenated raw).
* `src/surfaces/window/api.rs`: not feature-gated (so tests run in the default build); pure over `&Store`.
* `src/surfaces/window/assets/`: `index.html`, `app.css`, `app.js`, `fonts/*.woff2`, `fonts/OFL.txt`.

**Cut list**: write actions; capture dialog; status chips; hotkey chip; single instance; remembered window size; live file watching (focus refresh is enough); CI job; a JS test runner (asset scan tests in Rust only).

---

## Edge Case Checklist

- [ ] No display or a build without `window` → Obsidian path, exit as today
- [ ] Every string sent to the page passes `guard::mask_for_output`; a masked note shows "secret masked"
- [ ] Note text is rendered with `textContent`, never `innerHTML`; a note containing `<script>` shows as text
- [ ] CSP: no network loads; Plex fonts come from the binary; navigation away from `easy-note://` is refused
- [ ] IPC request with an unknown command or a bad id → an error object, never a panic
- [ ] Day groups use the local time zone (today / yesterday / earlier), same as `query` `day`
- [ ] Notes added while the window is open appear on window focus, without a restart
- [ ] AI detail `stale` matches `ai::read`'s flag; a note without AI detail shows "No AI detail yet"
- [ ] 5,000 notes: handler p95 measured (AC 12), not assumed
- [ ] A second `easy-note dashboard` while a window is open opens a second window (single instance is cut)
- [ ] Empty notes dir → board with zeros and the design's empty text, no error
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `Cargo.toml`, `Cargo.lock` | feature `window`, optional `wry`, `tao` |
| `src/surfaces/window/mod.rs` (new) | window, protocol, IPC, navigation guard |
| `src/surfaces/window/api.rs` (new) | pure `handle` |
| `src/surfaces/window/assets/*` (new) | page, CSS, JS, fonts, licence |
| `src/surfaces/mod.rs` | `pub mod window;` (`api` always, shell under the feature) |
| `src/surfaces/dashboard.rs` | `use_window`, `run()` branch |
| `tests/window_api.rs` (new), `tests/perf.rs` | AC 3–7, 9, 12 |
| `README.md` | dashboard section: window build, `--features window`, the apt package `libwebkit2gtk-4.1-dev`, Obsidian fallback |
| Pre-existing pin tests | `tests/dashboard.rs`, `tests/cli.rs` help text, `tests/readme.rs` — list each and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*` | Supervisor-owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| `src/core/*` | Read through existing public functions only; a missing accessor is a `Question:` to the Supervisor |
| `src/core/format.rs`, note files | DDR 0002 |
| `src/surfaces/tray.rs`, `src/surfaces/install.rs` | The launcher and tray already run `easy-note dashboard` |

---

## Test Plan

`cargo test` in all three feature sets. `window::api` is pure and tested in the default build with temp `EASY_NOTE_DIR`. Asset safety by a Rust test that reads `src/surfaces/window/assets/`. Visual check: the frontend agent opens `index.html` with a stub `window.ipc` in a headless browser if one is on the machine, else reports the screenshot rows as not run. Owner-only: `cargo build --release --features "tray window"`, install, click the launcher.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_HUN-278.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-19 DECIDED own window, not Obsidian (owner pick "Build own window")
* 2026-11-19 DECIDED wry + tao webview behind feature `window`, read-only first (Supervisor under `[admin]`, tracks.md row 4, ⚠)
* 2026-11-19 DECIDED bundle IBM Plex; hide status and hotkey chips in this slice (Supervisor, BRAINSTORMING_LOG_dashboard-window.md)
