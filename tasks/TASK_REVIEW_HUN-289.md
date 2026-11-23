# TASK_REVIEW — HUN-289: quick opens the design's capture dialog in its own small window

> Sibling of `tasks/TASK_GUIDE_HUN-289.md`. Filled by the QA reviewer (independent, read-only on source)
> at Stage 4 on branch `feat/HUN-289-quick`, HEAD `2c34c795d8c302632837c03691ab4e52297e66c0`
> (2 commits ahead of main: b5d2082, 2c34c79). Verdict, score and security sections are left for the Supervisor.

---

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass (AC 1, 2, 3, 4, 5, 7 partly, see gaps) | `tests/window_api.rs` (`the_quick_window_*`, `the_quick_page_*`), `tests/fixtures/window/quick_harness.js` (node, ran: node found at `~/.local/bin/node`), `tests/fixtures/window/page_harness.js`, `tests/ai.rs` (fallback env), `tests/quick.rs` (zenity path, unchanged) |
| Verification command run | ☑ pass | see "Verification output" below: exit 0; 600 passed / 0 failed / 6 ignored (default build), 608 passed / 0 failed / 6 ignored (`--features window,tray`) |
| Negative cases hold | ☑ pass (by test reading; no mutation run) | `the_quick_window_refuses_every_other_command` (board, detail, edit, trash, restore, accept_ai, x, bad JSON, missing cmd, foreign origins `localhost.evil`, `https://x/`, empty source -> `None`); empty `add` -> `error: empty`; Esc / empty Enter -> `close` only. Not done: a red run against prior content or a mutation diff |
| verify | ☐ pass / ☐ fail / ☐ N/A | left for the auto verify hook (the Supervisor's own Xvfb run: type + Enter saved a note with `source: quick`; Esc and empty Enter saved nothing; Shift+Enter saved a two-line note. Not driven: context chip, 20,000-char path, secret masking, tray, hotkey, Wayland focus, timing (owner); it is not the hook's verify) |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Read in full: `src/surfaces/quick.rs`, `src/surfaces/window/mod.rs` diff, `quick.html/js/css`, `capture.js`, `ipc.js` diff, quick tests in `tests/window_api.rs`, README diff. Skipped: `app.js` refactor internals (covered by the unchanged-green dashboard page harness), core, tray |
| Full smoke suite still green (no regression) | ☑ pass | both builds, 0 failed (below) |
| **Companion score (easy-verifier)** | ☑ fail | overall 70 (floor missed: 70 < 80), change_risk 100 pass (bugfix, no failed rule), scope changes, ref `main...HEAD`, abstentions: solution-fit (no_static_rule), reliability (insufficient_measured_weight), api, data, operations, infrastructure (below_coverage_floor), gates: solution-fit asked once, left abstained (its listed refs are brainstorm-log lines, no held evidence stands behind them), drag is security 18 (redaction_hits 4 across docs, tests and fixtures, sink_hits 2 in quick.rs; not yet confirmed as false positives, same pattern as earlier tasks), blast-radius 65, code-quality 67, requirement-fidelity 67, no findings sent (verifier rejected evidence_refs outside its packs), report: `/home/hungnguyenhuu/.easy-verifier-reports/easy-note-mcp-HUN-289/evidence-report-changes-20261123T104623-086436Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☐ N/A | NOT checked by this reviewer (no screenshots taken; no display session). Owner / Supervisor `ux-evaluate` |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ pass / ☐ fail / ☐ N/A | NOT checked. Note: `quick.css` hard-codes `#121413` twice instead of a token; `app.css` reused (shared, not copied) |
| **UI: Responsiveness at target viewports** | ☐ pass / ☐ fail / ☐ N/A | NOT checked (fixed 592x226 logical window, `overflow: hidden`; no scroll or cut-off check done) |

### Verification output

Command (exact, from the guide), run in the worktree:

```
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features window,tray -- -D warnings \
  && cargo test --locked --features window,tray
```

Result: exit code 0 (`rc=0`); fmt clean; both clippy runs with `-D warnings` clean.

| Build | Passed | Failed | Ignored |
|---|---|---|---|
| default (`cargo test --locked`) | 600 | 0 | 6 (the `perf.rs` opt-in tests) |
| `--features window,tray` | 608 | 0 | 6 |

Counts are the sum of every `test result:` line in the run. The window,tray build runs 8 more tests than the default build (608 vs 600; the extra ones are feature-gated tests in several files, for example the lib unit tests 71 vs 68).
Quick-related tests that ran and passed in both builds:

```
test the_quick_page_is_served_and_loads_only_the_dialog ... ok
test the_quick_window_add_hands_the_text_back_and_close_closes ... ok
test the_quick_window_answers_context_from_the_chip_quick_computed ... ok
test the_quick_window_masks_a_secret_in_the_chip ... ok
test the_quick_window_refuses_every_other_command ... ok
test the_quick_page_keys_follow_the_dashboard_dialog ... ok      (needs node; node was present, so not skipped)
test result: ok. 68 passed; 0 failed; 0 ignored  (tests/window_api.rs, default build; 68 again in the window,tray build)
test result: ok. 26 passed; 0 failed; 0 ignored  (tests/quick.rs, both builds)
```

Full log: `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/333106bb-e524-49ed-9953-6f5253fef92a/scratchpad/full.txt` (session scratch, not durable).

---

## Acceptance criteria

| # | Criterion | Status | Evidence |
|---|---|---|---|
| 1 | Window build + display: small window with only the capture dialog, no board request | Code present, display part owner-only | `quick.rs:86-95` `answer()` -> `window::open_capture`; `window/mod.rs` `open_capture` (592x226, centred on X11, `with_navigation_handler(url == QUICK_START)`); `quick.html` has no `groups/search/detail/app.js` (`the_quick_page_is_served_and_loads_only_the_dialog`). No test opens a real window (headless) |
| 2 | Enter saves via the same path (`source: quick`, context, tags, masking, `after_save`), confirmation, window closes, exit 0 | Pass (save path shared) | `Capture::Save` -> returned to `quick::run`, which is the unchanged `store.add(... "quick" ...)` + `notify` + `after_save` (`quick.rs:49-83`); `the_quick_window_add_hands_the_text_back_and_close_closes`; harness `enter` row: one `add` with text and `keep_context:true`, second Enter sends nothing |
| 3 | Esc, window close, empty Enter write nothing and close; exit 0 | Pass | harness `esc` and `empty_enter` rows send `{cmd: close}`; `CloseRequested` handled in `open_capture`; `Capture::Close` -> `Ok(None)` -> `run` returns `Ok(())` |
| 4 | Shift+Enter newline; Backspace on empty drops chip; IME ignored; one shared implementation | Pass | `capture.js` shared by `app.js` and `quick.js` (`index.html` and `quick.html` load it; asserted in the served-page test); harness `shift_enter`, `composing`, `drop_chip` rows; dashboard `page_harness.js` still green with `capture.js` |
| 5 | `quick --cwd DIR` uses that folder's context in the chip | Pass (by construction) | context computed in `quick::run` before `answer()` (`quick.rs:42-48`) and handed to `open_capture`; `the_quick_window_answers_context_from_the_chip_quick_computed`; `--cwd` zenity tests in `tests/quick.rs` (`ac2_cwd_*`). No test drives `--cwd` through the window path itself |
| 6 | Window cannot open (no feature, no display, `NO_DISPLAY`) -> zenity / kdialog / terminal unchanged | Pass | `answer()` falls through on `window_here()` false or `open_capture` `Err`; `open_capture` stub `Err` without the feature; `event_loop()` maps GTK-init panic to `NO_DISPLAY`; `tests/quick.rs` runs with `DISPLAY=:99` and passes in the `window,tray` build (26 passed), `tests/ai.rs` adjusted to X11-only dummy display; `use_window` unit tests `window_api.rs:463-467` |
| 7 | Page talks only `context` and `add` (and `close`); IPC origin and CSP same as dashboard | Pass, with note | `capture_ipc` answers `context`, `add`, `close`, all else `unknown command` (`the_quick_window_refuses_every_other_command`); `capture_from` uses `allowed_url` (foreign origins -> `None`); `quick.html` meta CSP `default-src easy-note:` and the header on every response (`mod.rs:359`, test at `window_api.rs:449`). Note: AC text says `context`/`add` only; `close` was added (needed for Esc, since `add` of empty is an error) and the dashboard answers `unknown command` to it (`window_api.rs:1075-1084`) |
| 8 | Time to dialog visible measured on owner's machine (release build), against NFR-011 | Owner-only, partly recorded | Guide Questions Log has a `MEASURED Q1` line (mapped p50 95 ms / p95 114 ms over 20 runs; painted 6/10 at 300 ms, 8/10 at 350 ms, 10/10 at 400 ms). Not reproducible by this reviewer; no p50/p95 of time-to-paint over 20 runs is pasted. Over NFR-011 (300 ms p95): owner must choose Option A or B |

---

## Findings (ranked)

No P0. No failing test.

**P1**
- None in code. Decision item for the owner (not a defect): the measured paint time (about p50 300 ms, p95 400-450 ms) is over NFR-011 (p95 < 300 ms), yet `quick` (hotkey and tray) now takes the window path by default, which is de facto Option A. PRD NFR-011 / FR-022 text is still the old wording (guide gate "All Requirement Refs exist in PRD.md" is unchecked).

**P2**
- Text over 20,000 chars loses the typed text. `capture_ipc` `add` has no length check; the window hides first (`HIDE_FLUSH`), then `quick::run` calls `store.add`, which refuses with `TextTooLong` (`core/format.rs:124`). The user sees only a "note not saved" notification. The dashboard dialog keeps the dialog and text open with "the text is too long" (`api.rs:96`); the guide's edge case says "Very long text: as the dashboard dialog does". The zenity path has the same loss, so it is not a regression, but it is a gap against the guide and against the "no capture path may drop typed text" constraint. Fix idea (for the implementer): call `check_text` in `capture_ipc` `add` and reply `{"error":"the text is too long"}` so the page keeps the dialog.
- Window path itself is not covered by any automated test: `quick::answer` (`keep || chip.is_none()`), the real `open_capture` event loop, hide-then-save order, and fast-Enter timing are only checked by reading and by the page harness. Needs the owner's display run (list below).

**P3**
- No blur/focus-loss handling: on Wayland (GNOME) `with_position` and `with_always_on_top` are ignored and the compositor may refuse focus, so keystrokes can go to another app while the window sits unfocused; the only exits are Esc inside the dialog or the close button. Measurement so far was under XWayland. Guide edge case "Focus ... note what happens on GNOME and KDE" is still open.
- `quick.css` hard-codes `#121413` (twice) rather than reusing a token from `app.css`; guide UI row says "only existing tokens". Check against `docs/design/graphite-mint-tokens.json`.
- `capture_ipc`'s `add` Save path bypasses the per-request `req` reply by design; the page's `add` promise never resolves (the window closes). Harmless, but a page that kept running after a failed Rust close would stay in `saving = true`. Not reachable today.
- `std::panic::set_hook` swap in `event_loop()` is process-global and not restored on a panic other than the GTK one; single thread at that point, so low risk.
- Stale wording: `src/surfaces/cli.rs:188` help for `quick` still says "Open a one-line box"; README line 3 / 227 still say "one-line". Correct enough (the dialog is a text box) but the guide listed CLI help for review; PRD FR-022 (native input, `zenity`) and NFR-001 subprocess wording are Supervisor-owned and unchanged.
- Dev-only: `with_devtools(cfg!(debug_assertions))` on the quick window, same as the dashboard.

### Risk review of the changed code

| Area | Result |
|---|---|
| IPC command allow-list | Tight. `capture_ipc` has an explicit `close`/`add`/`context` match; everything else `unknown command`. The quick window handler never touches the store itself: it returns `Capture::Save`, and `quick::run` saves. So even a compromised page cannot read, edit, trash or list notes through this window |
| Origin and navigation | `capture_from` requires `ORIGIN` + `/` or end (`localhost.evil` refused, tested). Navigation handler is stricter than the dashboard's: exactly `QUICK_START`. New-window requests denied |
| Id and path handling | The quick window handles no ids or paths. `respond()` serves only the fixed `asset()` table (no file path from the URL). It serves all assets, including `app.js`/`index.html`, to any page of the origin, but navigation is pinned to `quick.html` |
| Secret masking | Chip repo/task passes `guard::mask_for_output` (tested). Note text is masked on write by the unchanged `store.add`; the confirmation line and notification use the masked result. Window never echoes the typed text back. Not re-run: a planted-secret `add` through the window path end to end (needs a display) |
| CSP | `default-src easy-note:` both as a meta tag in `quick.html` and as the response header; `quick.html` has no inline script or style. Same as the dashboard |
| zenity fallback | Falls through on feature off, no display, `open_capture` `Err` (including the caught GTK-init panic). Verified green via `tests/quick.rs` in the `window,tray` build with `DISPLAY=:99` |
| Two runs at once | Two windows, each saves on its own (decided in the guide); no test |

---

## Owner-only evidence (cannot be filled by this reviewer)

- AC 8 / Success Criterion 5: release-build time to dialog visible, p50 and p95 over 20 runs, pasted here (the guide log has mapped p50/p95 over 20 runs but time-to-paint over only 10 runs), then the Option A / Option B decision for NFR-011.
- Real display runs: tray `New note` then type then Enter (window gone, notification shown, note has `source: quick`); `Ctrl+Alt+N` same; Esc (nothing saved, no file); Backspace on empty (note has no repo/task); `quick --cwd <repo>` chip shows that repo.
- Focus behaviour on GNOME Wayland and, if available, KDE (is the text box focused when another app had focus; does the window centre).
- Planted secret typed into the window: file and notification masked.
- Text over 20,000 chars: confirm the P2 behaviour on the real window.
- Two hotkey presses at once: two windows, both save.
- `notify-send` missing: window still closes, no error.
- UI rows: easy-ui screenshots (empty, with chip, with text, error) and `ux-evaluate`, since this reviewer took none.

---

## Demonstration

Left for the Supervisor (BEFORE / AFTER / DELTA / WITNESS).

## Verdict / Score / Security review

Left for the Supervisor.

## Auto decisions

Mode: auto

1. The quick window stays the default for `quick`, and NFR-011 (p95 < 300 ms) applies to the native box, not the window (measured p50 95 ms, p95 114 ms to map; dialog painted slower) — RULE: `tasks/TASK_GUIDE_HUN-289.md:222` (Q1 = Option A, owner: "go with your recommendations", chat 2026-11-23; PRD FR-022, FR-027 and NFR-011 reworded to match, `PRD.md:125`).
2. The QA reviewer's P1 and P2 findings were fixed in the code (`check_text` limit for the window path, trimmed check, `onError` line in `quick.js`, help and README wording); the P3 items were left — RULE: `skills/code-review/SKILL.md:26` (P1 must fix) and `skills/code-review/SKILL.md:28` (P3 optional).
3. The verify row was reset to empty so the hook writes its own result, and `main` is not merged into this branch by me because the gate allows no merge-in under `[admin]` — RULE: `docs/claude-md/tracks.md` `### Admin tag`.

Checks: code-review Skill call made over `main...HEAD` (20 files, 1027 lines; lane check EXCEEDS because of the docs commit, no P0 and no P1 open) (auto-checked, not /verify); security-review Skill call made, the HUN-289 source diff (`window/mod.rs`, `quick.rs`, `quick.html`, `quick.js`) read directly: IPC only from the `easy-note://localhost` origin, navigation pinned to `quick.html`, only `context`, `add` and `close` answered, text capped by `check_text`, every reply string through `mask_for_output`, the note saved by the same `quick` code, no finding at confidence 8 or more (auto-checked, not /verify); QA review by a fresh-context reviewer and the guide Verification Command, both builds 0 failed (auto-checked, not /verify); real run under Xvfb with release `--features window,tray`: Enter saved a note with `source: quick`, Esc and empty Enter saved nothing, Shift+Enter saved two lines (auto-checked, not /verify); Companion score overall 70, floor missed (Evidence row).

Memory: nothing new on this branch, to keep `memory/` out of a merge conflict with main; the HUN-289 decision entry and follow-ups are written on main after the merge, as for HUN-288 (ffd0b15).

Not done: merge (admin: after the auto verify pass; the Companion score also fails, so the owner merges by hand), the delivery report, `main` merged into this branch (the gate refuses it under `[admin]`), owner-only checks (20-run quick-dialog timing, tray New note, `Ctrl+Alt+N`, planted secret in the window, GNOME Wayland focus), auto verify: running (log `.claude/verify-runs/HUN-289-30eab2e72d31.log`)
