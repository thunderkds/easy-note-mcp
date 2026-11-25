# TASK_REVIEW — HUN-290

> Sibling of `tasks/TASK_GUIDE_T[NNN].md`. Everything here is **filled by the reviewer at Stage
> 4/5** — it is deliberately NOT in the guide, because the implementing agent re-reads the guide on
> every turn and never fills these two sections.
>
> Consumers resolve each section **guide first, this file second** (`.claude/hooks/lib/guide_sections.py`):
> a legacy guide that still carries these sections inline keeps working unchanged, and a stray
> review file can never override an inline section.

---


## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/window_api.rs` (9 tests `hun290_*`: lock file, stale-lock takeover by a killed child, focus request with and without id, bad line, no-listener wait, long path, other notes dir, show script, page `show`), `tests/dashboard.rs` (`hun290_a_second_dashboard_call_focuses_the_open_window`, `window` feature only: NOT run here, see below). Run 2026-11-25: `cargo test --locked --test window_api hun290` -> `test result: ok. 9 passed; 0 failed` |
| Verification command run | ☑ pass | Both halves ran after the GTK packages were installed (2026-11-25): `cargo fmt --check` clean; `cargo clippy --all-targets --locked --features window,tray -- -D warnings` clean; `cargo test --locked --features window,tray` no failed test; `cargo test --locked` 0 failed. The first window run FAILED `hun290_a_second_dashboard_call_focuses_the_open_window` (tests/dashboard.rs:294, exit 1, `no dashboard yet`); see `## Fix after the window build` |
| Negative cases hold | ☑ pass | Mutation run by the Supervisor: in `src/surfaces/window/single.rs:57` `return Ok(None)` on `WouldBlock` changed to `{}` (a held lock no longer stops the second caller). `cargo test --locked --test window_api hun290` then printed `hun290_ac4_the_lock_is_a_file_in_the_notes_dir_and_only_one_owner_holds_it ... FAILED`, `hun290_ac4_a_killed_window_leaves_a_lock_the_next_start_takes_over ... FAILED` and `test result: FAILED. 7 passed; 2 failed`. Reverted with `git checkout src/surfaces/window/single.rs`. |
| verify | ☑ pass | auto-run by hook, session f4690ae2-ac3f-4191-9ae8-cd4d94a08c49, commit 8446f529f97b974e3e92ce980758410cc6384b0d, log .claude/verify-runs/HUN-290-8446f529f97b.log — pass |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Fresh-context reviewer read `main...375dc7d`: `src/surfaces/window/single.rs` (new), `window/mod.rs`, `dashboard.rs::run`, `app.js`, `page_harness.js`, `tests/window_api.rs`, `tests/dashboard.rs`, README, RUNBOOK. Tray was checked to call `dashboard` unchanged. Callers of `window::open` read for the new `Owner` argument. |
| Full smoke suite still green (no regression) | ☑ pass | Default build and `--features window,tray` build: all suites ok, 0 failed (after the fix) |
| **Companion score (easy-verifier)** | ☑ N/A | Not run in this session: the easy-verifier tool is not available here. The merge gate blocks on a `fail` row only. |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | No visible UI change: a second `dashboard` call focuses the existing window. Only an invisible `__easyNote.show` page hook was added (page-harness tested). |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | No CSS or markup change. |
| **UI: Responsiveness at target viewports** | ☑ N/A | No layout change. |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (2026-11-25T04:13:48Z, main at 74a290b, no implementation commit yet):

```
$ grep -rn "window.lock\|window.sock" src tests; echo "grep exit=$?"
grep exit=1
```

No window lock and no request socket exist in the code: `dashboard::run` calls `window::open` directly, so each `easy-note dashboard` (tray, launcher, CLI) opens its own window.

**AFTER** (2026-11-25, branch feat/HUN-290-window-lock):

```
$ grep -rn "window.lock\|window.sock" src | head -3
src/surfaces/window/single.rs:3://! `<notes dir>/.easy-note/window.lock` ...
$ cargo test --locked --test window_api hun290
test hun290_ac4_a_killed_window_leaves_a_lock_the_next_start_takes_over ... ok
test hun290_ac1_ac3_a_second_call_sends_a_focus_request_with_the_note_id ... ok
test result: ok. 9 passed; 0 failed
```

**DELTA**: A second `easy-note dashboard` (tray, launcher, CLI) brings the open window to the front, shows `--note <id>`, and exits 0 instead of opening another window.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]

## Stage 4 review

Reviewer: fresh-context, read-only, commit 375dc7deff9c7b4f48af539ff8b841c2dc207e8d, diff `main...375dc7d`. Scope: `src/surfaces/window/single.rs` (new), `window/mod.rs`, `dashboard.rs::run`, `app.js` `show`, and the new tests. Skipped: tray, quick window, unrelated modules (no code change there).

### Commands run (default build, in the worktree)

`cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` -> exit 0 (fmt clean, clippy clean, all test binaries `test result: ok`, 0 failed).

### Could not verify

* `--features window,tray` build: no pkg-config / GTK here. NOT compiled or run: `shell::open` and the `Ui` enum in `src/surfaces/window/mod.rs`, and `tests/dashboard.rs` `hun290_a_second_dashboard_call_focuses_the_open_window` (`#[cfg(feature = "window")]`). Checked by reading only. The Supervisor or CI must run `cargo clippy/test --locked --features window,tray`.
* Real focus behaviour on X11/Wayland (AC 2) cannot be observed here.

### Acceptance Criteria

| # | Verdict | Evidence |
|---|---------|----------|
| 1 | pass (window half read-only) | `dashboard.rs:190-196` acquire returns None -> `focus_open_window` -> Ok (exit 0). Lock/request channel tested in `tests/window_api.rs` `hun290_ac1_ac3_*`; end-to-end CLI test is window-gated, not run. |
| 2 | pass by reading | `mod.rs` Focus arm: `set_minimized(false)`, `set_visible(true)`, `set_focus()`, `request_user_attention(Informational)`; no unwrap, no crash path; exit stays 0. |
| 3 | pass | `single.rs` `request_focus_within` sends `focus <id>`; `show_script` passes id as one JSON string (`mod.rs` `show_script`); `app.js` `__easyNote.show`; page-harness scenario `show_request`; test `hun290_ac3_*`. |
| 4 | pass | `single.rs:acquire` uses `.easy-note/window.lock` + `File::try_lock`; killed-child takeover test `hun290_ac4_a_killed_window_*` passes. |
| 5 | pass | Tray goes through the `dashboard` command (unchanged); no tray text change in diff. |
| 6 | pass | Per-notes-dir lock (`hun290_ac6_*`); quick window and Obsidian path untouched (`NO_DISPLAY` falls through, owner dropped on return). |

Edge cases: long socket path -> hash path under `$XDG_RUNTIME_DIR` (tested, `single.rs:socket_path`); simultaneous calls -> exclusive lock; window still starting -> client retries 50 ms steps up to 2 s, then one stderr line and exit 0 (tested). Wayland refusal -> attention request only.

### Findings

* P3 `src/surfaces/window/single.rs:34-38` (socket_path fallback): with no `$XDG_RUNTIME_DIR` the socket goes in the shared temp dir (`/tmp`) under a predictable name. Another local user can pre-create that path; `acquire` then fails at `remove_file`/`bind` with "cannot listen", so `dashboard` errors (denial of service only; the request carries just a note id). Suggest a 0700 per-user subdirectory, or accept as Low risk.
* P3 `single.rs:acquire` (lines 62-72) / `dashboard.rs:190`: on a `window` build with no usable display, `acquire` binds the socket and `open` returns `NO_DISPLAY`; the lock is released but a dead socket file stays. Harmless (next owner removes it, clients get connection refused and retry up to 2 s), but a second call arriving during that fallback window loses its focus line silently. Cosmetic.
* P3 `single.rs:7-9`: `std::os::unix` imports are unconditional in a module compiled in every build, so a non-unix build breaks. Out of scope per the guide's cut list (Windows/macOS); note only.
* P3 `tests/dashboard.rs` (new test): a `window`-gated test asserts `elapsed < 2 s` per call; on a loaded CI machine this is timing-sensitive. Not run here.

No P0/P1/P2 findings. Evidence rows "Verification command run" and "Full smoke suite" are satisfied for the default build only; the feature half is open.

## Auto decisions

Mode: auto
1. Fixed a lock bug the window build exposed (`Owner::serve` dropped the lock file when it returned) with a new default-build test, red first — FACT: `tests/dashboard.rs:294` "assertion `left == right` failed: easy-note: no dashboard yet" and `tasks/TASK_GUIDE_HUN-290.md` AC 1 "starts no second window process"
2. The four P3 findings (shared `/tmp` socket fallback, dead socket on `NO_DISPLAY`, unix-only imports, timing-sensitive test) are follow-ups, not fixed here — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` row 17 (P3 is reported, not a blocker)
3. Marked the three UI rows N/A because the task changes no visible UI — FACT: `tasks/TASK_GUIDE_HUN-290.md` Out of scope "The capture-only quick window (HUN-289)" and AC list name focus behavior only, no screen
Checks: tests `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`, then the same with `--features window,tray` — exit 0, no failed test (default and window builds); reviews code-review by a fresh-context reviewer on 375dc7d, again on the fix commit (pending); real run: `target/debug/easy-note dashboard` with the lock held by another process (python `flock`) and `DISPLAY=:99` — exit 0, no window started (auto-checked, not /verify)
Memory: nothing new on this branch; the HUN-290 decision entry is written on main after the merge
Not done: merge (admin: after the auto verify pass); real focus behavior on X11 and Wayland (the window was never opened on a real display in this run); auto verify: not started yet

Code-review (Supervisor, `Skill({ skill: "code-review" })` over `main...c9953a8`, bounded to the diff): no P0, P1 or P2 beyond the reviewer's findings. Lane check: `lane: C1, 9 files, 446 lines, EXCEEDS (files 9 > 6; lines 446 > 250); unpredicted: src/surfaces/window/assets/app.js, src/surfaces/window/single.rs, tests/fixtures/window/page_harness.js` — the task was under-labelled: C2 would fit. The three unpredicted files are the page hook, the new module and the page-harness case; all are in scope of AC 3 and AC 4.
Paused: done not moved — report incomplete: auto verify slot

## Fix after the window build

With the GTK packages installed, `cargo test --locked --features window,tray` ran for the first time and `hun290_a_second_dashboard_call_focuses_the_open_window` failed (`tests/dashboard.rs:294`: the second `dashboard` call exited 1 with `no dashboard yet: run easy-note install obsidian`, because it had taken the lock and then fell back to Obsidian).

Cause: `Owner::serve(self, ...)` moved only `self.listener` into its thread (Rust 2021 closures capture only the fields they use), so the `_lock` file closed when `serve` returned and a second `dashboard` call found the lock free. The default-build tests never called `serve`, so they did not see it.

RED (default build, before the fix): `cargo test --locked --test window_api hun290_serving` printed `test hun290_serving_requests_keeps_the_lock_held ... FAILED` and `test result: FAILED. 0 passed; 1 failed`.
Fix: `let owner = self;` inside the thread closure in `src/surfaces/window/single.rs`, so the whole value (and its lock) moves into the thread.
GREEN: `cargo test --locked --test window_api hun290` — `test result: ok. 10 passed`; the window-feature run then had no failed test.
Manual run (default `target/debug/easy-note` built with the window feature): lock held by a python `flock`, `DISPLAY=:99`, `easy-note dashboard` — exit 0, no window started.

## Stage 4 review (second, 80d74a8)

Reviewed `main...80d74a8` in the worktree (fresh context, code-review skill).

Checks run (all exit 0, no failed test):
- `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`: pass.
- `cargo clippy --all-targets --locked --features window,tray -- -D warnings && cargo test --locked --features window,tray`: pass (includes `hun290_a_second_dashboard_call_focuses_the_open_window` and `hun290_serving_requests_keeps_the_lock_held`).

Questions asked:
1. Does the fix keep the lock for the process lifetime? Yes. `single.rs:87-91` `let owner = self;` moves the whole `Owner` (lock `File` and listener) into the thread. The thread loops on `incoming()`, which never ends, so the lock lives until the process exits. Before `serve`, `open` holds `owner` as a parameter (`window/mod.rs:471`) from `acquire` on, so there is no gap between `acquire` and `serve`. The window run is covered: the event loop runs in the main thread while the serve thread holds the lock; close ends the process and the OS drops the lock. Error paths in `open` before `serve` drop `owner` (lock released, intended; `NO_DISPLAY` falls back).
2. Same disjoint-capture problem elsewhere? No. `move ||` / `move |` sites: `single.rs:87` (fixed), `mod.rs:205` (thread; captures no multi-field struct), `mod.rs:389,497,513,525,529` (closures capture `store`, `running`, `proxy`, `focus`, `webview`, `window` as whole bindings, not fields of a lock-holding struct), `tray.rs:361,397` (no `Owner` or lock involved). `event_loop.run(move ...)` captures `window` and `webview` whole.
3. Focus path in `shell::open`: no crash path. The `Focus` arm calls `set_minimized(false)`, `set_visible(true)`, `set_focus()`, `request_user_attention(..)` (all infallible in tao) and `let _ = webview.evaluate_script(..)`; the note id goes in as one JSON string (`show_script`). `focus.send_event` result is ignored, so a closed loop is not a panic. The serve thread ignores bad lines (`parse` returns `None`), caps a line at 4096 bytes and sets a 1 s read timeout, so a stalled client cannot block later requests for more than 1 s.

### Findings

* P3 `src/surfaces/window/single.rs:93-107` (serve loop): one connection is handled at a time with a 1 s read timeout, so a client that connects and sends nothing delays other focus requests by up to 1 s. Local-only, no data exposure. Accept, or use a short thread per connection.
* P3 `tests/window_api.rs` `hun290_serving_requests_keeps_the_lock_held`: it guards the field-capture bug in the default build. It would also catch a later regression that moves the lock out of `Owner`, but not one inside `shell::open`; the window-gated `tests/dashboard.rs` test covers that. Fine as is.
* Earlier P3 findings (shared-temp socket fallback, dead socket after `NO_DISPLAY`, unix-only imports, timing-sensitive window test) still stand and are unchanged by 80d74a8.

No P0, P1 or P2 findings. The first review's open item (feature half not compiled) is closed by the green window,tray run above. Real focus behavior on a live X11/Wayland session was not exercised here (no display run by this reviewer); that stays for `/verify`.

Verdict: pass with findings
