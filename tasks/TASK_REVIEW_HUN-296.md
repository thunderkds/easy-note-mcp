# TASK_REVIEW — HUN-296: Tray timer: run `ai run` on the schedule while the tray is up

> Sibling of `tasks/TASK_GUIDE_HUN-296.md`. Everything here is **filled by the reviewer at Stage
> 4/5** — it is deliberately NOT in the guide, because the implementing agent re-reads the guide on
> every turn and never fills these two sections.
>
> Consumers resolve each section **guide first, this file second** (`.claude/hooks/lib/guide_sections.py`):
> a legacy guide that still carries these sections inline keeps working unchanged, and a stray
> review file can never override an inline section.

---

## Stage 4 review (fresh-context reviewer, 2026-11-27)

Range `476b3c8...fab32bb` only (HUN-295 below it has its own review).

**code-review: pass. 0 P0, 0 P1, 1 P2, 6 P3. security-review (Risk Medium): pass, no P0 to P2; one P3 (S1).**

### Findings

| # | Sev | Where | Finding | Fix |
|---|-----|-------|---------|-----|
| F1 | P2 | `src/surfaces/tray.rs` `AiTimer::tick`, `safe(&why)` | A config.toml that does not parse gives a multi-line stderr message with the quoted config line, not one line (real run below). `safe()` keeps `\n`. The guide's edge case says "log one line" and `RUNBOOK.md` says "one `no timed AI run` line". `a_bad_config_says_one_line_once_and_the_timer_goes_on` passes because it feeds a made-up one-line error. | Format only `why.lines().next()` (as `config::unreadable` does); give the test a multi-line input. |
| F2 | P3 | `RUNBOOK.md` "Scheduled AI runs" | "read `ai-run-state.json` for what happened" is not true for errors before the state write. Probe: invalid `ai-hold.json`, `ai run` exits 1 and writes no state file. From the tray (null stdio) such a failure repeats each interval with no trace. A tray started by the window also has null stderr, so F1's line is lost there. | Say in the RUNBOOK to run `easy-note ai run` by hand when the state file does not move. |
| F3 | P3 | `src/surfaces/tray.rs` `icon::run` | The notes dir is `store.root` from the tray's start; the child reads the current config. After a `dir` change the back-off and the lock are read in the old folder, so the back-off is ignored; if the old folder is gone, timed runs stop silently until a restart. The child's lock still prevents two runs. | Resolve `config::notes_dir()` at each look, or document the restart. |
| F4 | P3 | `AiTimer::tick` | `run_lock_held` is an eager argument, so every look takes a shared lock on `ai-run.lock`, also with the schedule off. A manual `ai run` that locks in that instant prints "already running". The window is very small. By reading, not reproduced. | Probe the lock only after the other conditions hold. |
| F5 | P3 | `AiTimer::tick` `root.is_dir()` | In the real tray this is true from the start, because `acquire_lock` (older code) creates `<root>/.easy-note`. "Skip until the notes dir exists" (guide edge case) holds only if the folder goes away later; an unmounted mount point gets a folder. `a_notes_dir_that_is_not_there_yet_is_waited_for` tests the timer alone. | Record as a limit; no change needed for this task. |
| F6 | P3 | `icon::run`, `.github/workflows` | No test covers the wiring of the timer into the loop, and CI builds without `--features tray`, so CI does not compile it. The AC 6 assertion catches a run started before the host check (M6), but cannot catch a timer that ticks there, because the first run is 5 minutes later. | Follow-up: a CI step `cargo clippy --features tray`. |
| S1 | P3 | `README.md` "The tray runs it for you" | A tray started at login does not get a shell's `CLAUDE_CONFIG_DIR`. With no `claude_config_dir` in `[ai]`, timed runs can use `~/.claude` although the notice in the window named another account folder. Same user, masked text. By reading. | Next to the `command` advice, say to set `claude_config_dir` too. |

### Security checks that hold

- NFR-014: the only spawn is behind `effective_schedule()` (schedule `tray` and enrich `claude`); config is read at each look; the child checks `enrich` again.
- argv only (`<current_exe> ai run`), no shell, no user-controlled argument, no new input, network path or note write. The new `run_lock_held` creates nothing.
- Never two runs: the child takes `ai-run.lock` itself; the probe is only an early skip.
- The config error passes through `safe()` (secrets masked, control characters escaped).

### Checks run (temp HOME, XDG dirs and notes dir; fake `claude` only)

- `cargo fmt --check` ok; `cargo clippy --all-targets --locked -- -D warnings` ok, also with `--features tray`.
- `cargo test --locked --no-fail-fast`: 788 passed, 1 failed (the flaky test below), 7 ignored.
- `cargo test --locked --features tray --lib --test tray --test tray_timer`: 79 + 2 + 11 passed.
- Mutants on `src/surfaces/tray.rs`, 6 of 6 killed, file restored: M1 schedule only instead of `effective_schedule()`; M2 no back-off; M3 lock ignored; M4 first run at once; M5 config complaint never reset; M6 run spawned before the host check (`ac3_no_tray_host...` panics `no AI run`).

Real run of the `--features tray` binary on a private D-Bus session with a fake StatusNotifierWatcher, `interval_min = 5`, config broken for 100 s in the middle:

```
2026-11-27T10:32:34+07:00 tray start
2026-11-27T10:33:14+07:00 config broken
2026-11-27T10:34:54+07:00 config restored
claude.log:  2026-11-27T10:37:34+07:00 called; cwd=<temp>/easy-note-enrich-5239b-37f3510a-0; CLAUDE_CONFIG_DIR=unset
tray.err:
easy-note: no timed AI run: cannot read <temp>/.config/easy-note/config.toml: TOML parse error at line 3, column 12
  |
3 | schedule = tray-broken
  |            ^^^^^^^^^^^
invalid boolean, expected `true`
state: {"last_run":"2026-11-27T10:37:34+07:00","organized":0,"skipped":1,"left":1,"calls":1,"failures":0,"next_after":null,"last_error":null}
10:38:44 tray alive: yes; no child or zombie left
```

One call, 5 min 0 s after the start, none before; the message came once (F1 shows its shape).

### Out of this diff, for the Supervisor

- FR-049 also says a failed run "shows one line in Settings". No window code reads `last_error`, and the guide has no AC for it. Confirm which task owns it.
- The diff changes a UI caption (`index.html`); Hard-Stop Gate 6 needs the three UI rows filled or N/A with a reason.
- `window_api::hun290_ac4_the_lock_is_a_file_in_the_notes_dir_and_only_one_owner_holds_it` failed once in the full run and passed alone; not chased.

---

## Follow-up review of the fixes (fresh-context reviewer, 2026-11-27)

Range `fab32bb67f9727ce344cab1deaa476aca8f56393...d3ab419a446b2cea7cafe2c109d70af2e2c618d4` only.

**Pass: F1, F2, F4 and S1 are fixed. No new finding in this diff.** F3, F5 and F6 were not claimed and stay as recorded above.

- F1 fixed: the line is `why.lines().next()` through `safe()`, so the quoted config line is not printed; `read_table`'s error starts with the path and `TOML parse error at line N, column M`. The reason on the last line is dropped, as `config::unreadable` does. The test now feeds a multi-line error.
- F2 fixed: `RUNBOOK.md` says such a run "may write no state" and to run `easy-note ai run` by hand. By reading `organize::run`: the `?` on `waiting()` returns before any `state.write`.
- F4 fixed: `!organize::run_lock_held(root)` is the last operand of `&&`, so the lock is probed only when the dir exists and a run is due. No test pins that order (by reading).
- S1 fixed: `README.md` names the unset `CLAUDE_CONFIG_DIR` and says to set `claude_config_dir`, an existing `[ai]` key.

Checks (temp HOME, XDG dirs and notes dir; fake `claude` first on `PATH`, never called): `cargo fmt --check` ok; `cargo clippy --all-targets --locked -- -D warnings` ok, also with `--features tray`; `cargo test --locked --features tray --lib --test tray --test tray_timer`: 79 + 2 + 11 passed. Full suite not run. Mutants on `src/surfaces/tray.rs`, 2 of 2 killed, file restored (same sha256): `safe(&why)` again fails `a_bad_config_says_one_line_once_and_the_timer_goes_on` (`tests/tray_timer.rs:185`); the lock probe removed fails `a_held_run_lock_starts_nothing` (`tests/tray_timer.rs:119`).

---

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass / ☐ fail | `tests/tray_timer.rs` (AC 1-4, new, 11 tests: `test result: ok. 11 passed; 0 failed`), `tests/tray.rs` (AC 6: `ac3_no_tray_host_exits_1_with_the_hotkey_hint` now asserts no AI run). AC 5: the menu code is unchanged and its pin tests pass. Deviation from the guide: the timer tests are not in `tests/tray.rs`, because that file is built only with the `tray` feature. Red first: `error[E0432]: unresolved imports ... ai_run_argv, ... AiTimer` before the code |
| Verification command run | ☑ pass / ☐ fail | Supervisor, source of `d3ab419`: `cargo fmt`, `cargo clippy --all-targets --locked -- -D warnings` clean, `cargo test --locked --no-fail-fast`: 30 suites, 789 passed, 0 failed, 7 ignored, exit 0 |
| Negative cases hold | ☑ pass / ☐ fail | F1 red run: `a_bad_config_says_one_line_once_and_the_timer_goes_on ... FAILED` before the fix. Reviewer mutants: 6 of 6 killed at `fab32bb`, 2 of 2 killed at `d3ab419` |
| verify | ☐ pass / ☐ fail / ☐ N/A |  |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass / ☐ fail | Fresh-context reviews of `476b3c8...fab32bb` and `fab32bb...d3ab419`; HUN-295 below it has its own review. Open P3, accepted as limits: F3 (notes dir kept from the tray start), F5 (`root.is_dir()` true from the start), F6 (loop wiring untested, CI builds without `tray`) |
| Full smoke suite still green (no regression) | ☑ pass / ☐ fail | `cargo test --locked --features window,tray --no-fail-fast` at `d3ab419`: 30 suites, 798 passed, 0 failed, 7 ignored, exit 0. `window_api::hun290_ac4_...` failed once in two earlier runs and passed on the rerun: a known flaky test outside this diff |
| **Companion score (easy-verifier)** | ☐ pass / ☑ fail / ☐ N/A | overall 76 (floor missed: 76 < 80), change_risk 100 pass (feature, confirmed; both test pairs confirmed after the first round showed `organize.rs` as unpaired), scope changes, ref `feat/HUN-295-ai-run...HEAD` at `d3ab419`, dimensions: architecture 100, supply-chain 100, test-strategy 72, requirement-fidelity 67, code-quality 67, blast-radius 65, security 59; abstentions: solution-fit (no_static_rule), reliability (insufficient_measured_weight), api, data, operations, infrastructure (below_coverage_floor); gate solution-fit left abstained (no listed ref concerns this change); review findings not sent (the packs hold none of their refs); a first call with a 40-hex base rated no change (the ref was masked). report: /home/hungnguyenhuu/.easy-verifier-reports/easy-note-mcp-HUN-296/evidence-report-changes-20261127T034716-515610Z.html |
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☑ N/A | One caption string in Settings changed to a shorter one; no layout, style or component change; pinned by `tests/window_api.rs` `the_page_has_the_settings_gear_dialog_and_notice` |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ pass / ☐ fail / ☑ N/A | Same element and class (`set-hint`); no token, colour or font change |
| **UI: Responsiveness at target viewports** | ☐ pass / ☐ fail / ☑ N/A | The new caption is shorter than the old one in the same wrapping element |

---

## Demonstration

**BEFORE**: at `476b3c8` (the base, before any implementation commit):

```
$ date -Is; grep -cE 'ai run|"run"|Timer' src/surfaces/tray.rs
2026-11-27T10:14:53+07:00
0
```

**AFTER**: at `d3ab419`:

```
$ date -Is; grep -cE 'ai run|"run"|Timer' src/surfaces/tray.rs
2026-11-27T10:52:13+07:00
7
```

Real tray binary at the source of `fab32bb`, private D-Bus session, stand-in tray host, temp notes dir, fake `claude`:

```
2026-11-27T10:23:14+07:00 tray started
2026-11-27T10:24:14+07:00 after 1 min: calls=0
2026-11-27T10:24:14+07:00 schedule switched on in config.toml, tray not restarted
2026-11-27T10:28:22+07:00 calls=1 tray alive: yes
state: {"last_run":"2026-11-27T10:28:14+07:00","organized":1,"skipped":0,"left":0,"calls":1,"failures":0,"next_after":null,"last_error":null}
zombies: 0
```

**DELTA**: With `Organize automatically` on, the running tray organizes new notes with no command to type.

**WITNESS**: Supervisor session, 2026-11-27 10:23 to 10:52; the fresh-context reviewer ran the tray again at 10:32 to 10:38 (review section above).
