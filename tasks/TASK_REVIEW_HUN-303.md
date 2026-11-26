# TASK_REVIEW — HUN-303: `easy-note setup` binds Ctrl+Alt+N on GNOME

> Sibling of `tasks/TASK_GUIDE_HUN-303.md`. Everything here is **filled by the reviewer at Stage
> 4/5** — it is deliberately NOT in the guide, because the implementing agent re-reads the guide on
> every turn and never fills these two sections.
>
> Consumers resolve each section **guide first, this file second** (`.claude/hooks/lib/guide_sections.py`):
> a legacy guide that still carries these sections inline keeps working unchanged, and a stray
> review file can never override an inline section.

Reviewer: qa-expert, fresh context, 2026-11-26. Commit `7983685770e0ac5b1b382e1d25c1f6688de57298`,
diff `main...7983685` (main = `dabf507`).

---

## Verdict

**Pass. P0: 0, P1: 0, P2: 3, P3: 5. All 10 Acceptance Criteria hold; AC 3 has one narrow input that it misses (F4).**
No finding loses or replaces an entry of the person's shortcut list, or takes a used key, for a shortcut made in GNOME Settings
with a plain or full-path command. No fix was applied (report only). F1 to F8 are for the Supervisor to fix or waive.

## Findings

| ID | Sev | Conf | File:line | Finding | Action |
|----|-----|------|-----------|---------|--------|
| F1 | P2 | 100 | `PRD.md:130` (NFR-001) | NFR-001 lists the allowed subprocesses. `gsettings` is not in the list, but `install hotkey`, `uninstall hotkey` and `setup` now start it (`src/surfaces/install.rs:302`). | Add `gsettings` (these three commands, GNOME only) to NFR-001. |
| F2 | P2 | 100 | `scripts/build_deb.sh:115-116` | The package description says `setup` adds "the launcher, the tray and the Obsidian vault entry". It now adds the shortcut too. The guide puts this file under "Files Must NOT Touch" (reason: no maintainer script), so this is the Supervisor's choice. | Change the text only, or waive. |
| F3 | P2 | 100 | `tests/hotkey.rs:422-433` | The three read-backs have no test of their own. Removing any one of them (`install.rs:397`, `:435`, `:450`) leaves 21/21 tests green (mutations M7, M8, M9). The `readonly` stub fails every write, so one check hides the next. Also, a missing `gsettings` is tested only through `install hotkey`, not through `setup` (AC 8; checked by hand here). | Add a stub mode that ignores only one write (the list, one key, one reset). |
| F4 | P3 | 100 | `src/surfaces/install.rs:468-472` | AC 3: `runs_quick` splits on spaces. A command `'/opt/my apps/easy-note' quick`, `sh -c 'easy-note quick'` or `env FOO=1 easy-note quick` with `<Super>n` is not seen, so `install hotkey` adds a second shortcut on Ctrl+Alt+N. The person's entry and key stay; the new one is added only if Ctrl+Alt+N is free. | Strip quotes around the program, or look for `easy-note` followed by `quick` in the words. |
| F5 | P3 | 100 | `src/surfaces/install.rs:376-399` | A path that is listed and half-done is not always completed. Order of writes is name, command, binding. If the binding write fails after the command write, the next run says `it has no key, so you turned it off` and writes nothing (reproduced, A1/A2). The same message appears when a person stopped the manual `easy-note hotkey` steps before the binding step (this case follows AC 3). | Write `binding` before `command`, so a failed run leaves `command` empty and the next run completes it. |
| F6 | P3 | 100 | `src/surfaces/install.rs:371-394` | Only `command` decides whether the entry at our path is free. A listed entry there with an empty command but the person's own name and `<Super>g` is overwritten with our name and key (reproduced, B). Reachable only by hand (`gsettings` or dconf), because GNOME Settings makes `customN` paths. | Complete only when each key is empty or already ours; else report `kept`. |
| F7 | P3 | 100 | `README.md:53-54` | "Each step prints under its own heading (`launcher:`, `obsidian:`, `tray:`)" omits `hotkey:`. | Add `hotkey:`. |
| F8 | P3 | 100 | `README.md:38-39` | The new sentence says `apt remove` does not remove what `setup` added and that `easy-note uninstall <target>` does. After `apt remove` there is no `easy-note` to run, and `uninstall obsidian` never removes the vault entry (DDR 0008 item 3). | Say: run `easy-note uninstall <target>` before `apt remove`; the vault entry stays. |

## Needs validation

None.

## Acceptance Criteria

Stub runs used a cleared environment, a temp HOME and a stub-only `PATH`.

| AC | Verdict | Evidence |
|----|---------|----------|
| 1 | pass | `install_adds_the_shortcut_and_writes_its_keys_before_the_list` (4 writes, list last). Replay of this machine's real `list-recursively` output (1815 lines) with an empty custom list: exit 0, 4 writes, `added Ctrl+Alt+N for easy-note quick (...)`. |
| 2 | pass | `install_keeps_every_other_shortcut_in_order`, `a_list_that_cannot_be_read_is_never_written` (5 bad values); M2 goes red. Extra: paths `"/a'b/"` and `'/c, d/'` stay byte for byte through install and uninstall. A list entry with no trailing slash: exit 1, 0 writes. |
| 3 | pass, except F4 | `a_second_install_writes_nothing`, `a_shortcut_of_the_person_that_runs_quick_is_left_alone`, `our_shortcut_with_a_cleared_key_stays_off`; M4 red. `/usr/bin/easy-note quick --cwd /x` is recognised. |
| 4 | pass | Custom and GNOME-setting tests; M1, M3, M10 red. Extra: `<CONTROL><ALT>N`, `<Ctl><Alt>n`, `<Mod1><Primary>n` are refused; `<Control><Alt><Shift>n` is not a conflict. Real data with one line rewritten to `<Primary><Alt>N`: refused, names `org.gnome.desktop.wm.keybindings switch-to-workspace-down`, 0 writes. |
| 5 | pass | `another_command_at_our_path_is_kept`; M5, M6 red. Empty command: see F6. |
| 6 | pass | `uninstall_removes_only_our_shortcut`. Extra: a failed `reset` gives exit 1 and a second run finishes. A full-path command at our path is `kept` (as AC 6 says). |
| 7 | pass | `another_desktop_runs_no_command` (KDE, sway, empty). Extra: `setup` with the variable unset: exit 0, `skipped`, no `gsettings` call. |
| 8 | pass | `a_failed_shortcut_step_fails_setup_but_not_the_other_steps`. Extra, `setup` on GNOME with no `gsettings`: exit 1, `Using Claude Code?` still printed, `easy-note: setup: hotkey failed`. |
| 9 | pass | `the_gnome_text_names_install_hotkey` (no call), `ac7_hotkey_writes_nothing_and_runs_no_command`. |
| 10 | pass | Read in the diff: README "Set up" (add, check, remove) and "Bind a key"; `PRD.md` FR-054, FR-041, NFR-003; DDR 0003 item 8 amends item 7. See F7, F8. The owner's "yes" is taken from the guide; the chat was not checked. |

## Checks of the assumptions about real `gsettings` (2.88.0, GNOME Shell 50.1, read only)

- `get` of the list prints `['...easy-note/']`; GLib prints an empty `as` as `@as []`; both parse.
- An unset entry key prints `''` with exit 0. A path with no trailing slash: `Path must end with a slash (/)`, exit 1.
- GLib prints a string with `'` in double quotes and escapes `\\` and `\n`; `literal` reads both quote forms. `\uXXXX` is read as `uXXXX` (display only; the list keeps the raw text).
- GLib parses the value forms that the code passes to `set` (`'<Control><Alt>n'`, `['/a/', '/b/']`, `@as []`). This was a parse in GLib only.
- Every line of the real `list-recursively` output went through the conflict check; no false match.
- Not verified: that a real `set` saves, and that a real `set` exits 0 when dconf cannot save. No write was allowed.

## Security review (Risk Medium; rules `references/security-reviewer.md`)

No confirmed finding and no `needs_validation` lead. Rejected leads:

| Lead | Why rejected |
|------|--------------|
| Shell injection through setting values | `Command` with an argument list, no shell (`install.rs:302-305`). |
| Option injection in `gsettings` arguments | The schema argument starts with the constant `MEDIA_KEYS`; the list value starts with `[` or `@`. |
| Terminal escapes from another shortcut's name or binding | `literal` drops the backslash of an escape; names print with `{:?}`; output goes through `safe()`. |
| A planted `gsettings` on `PATH` | Same user and same trust level as the caller; no boundary is crossed. |
| Panic on malformed output (denial of service) | Slices fall on literal boundaries; bad lists (`nonsense`, `['/a/'`, `[1, 2]`, empty) give exit 1. |

## Other reviewer checks

- Reachability: entry point `install hotkey` is in `src/surfaces/cli.rs`, `quick.rs` text, `README.md` and `tests/hotkey.rs`.
- Lane check: `lane: C1, 10 files, 872 lines, EXCEEDS (files 10 > 6; lines 872 > 250); unpredicted: PROJECT_SPEC.md, docs/ddr/0008-register-the-obsidian-vault.md, tasks/TASK_GUIDE_HUN-303.md` (warn only: C1 may be too low).
- Startup-reads check: not done; the spawn prompt gave no implementer report.
- Over-engineering: none found.

## Notes (not findings)

- The shortcut command is the bare `easy-note quick` (AC 1). The success line does not check that the desktop session finds `easy-note`. With `~/.cargo/bin` outside the session `PATH`, the key would do nothing.
- After a person removes the shortcut in GNOME Settings, the next `setup` adds it again (only a cleared key stays off).
- The list is read first and written after several calls. A change made in GNOME Settings in that short time would be lost. Not reproduced.
- `Gsettings::run` has no timeout.
- Extension schemas installed only for one user are probably not in `list-recursively`. Not checked.
- This desktop already had the `easy-note` entry (`easy-note quick`, `<Control><Alt>n`) when the review started, and had the same values at the end. The review ran no write command.

---

## Auto decisions

Mode: auto
1. The Requirement Fidelity Gate is signed `auto (Supervisor)` — RULE: docs/claude-md/tracks.md § Every user stop and its auto rule
2. `setup` and `install hotkey` write one GNOME shortcut, which changes the rule that OS shortcut settings are never written; the owner approved this in chat — FACT: tasks/TASK_GUIDE_HUN-303.md:25 "the user replied \"yes\""
3. Our listed entry with a cleared key stays off (added to AC 3) — SAFER OPTION: it keeps a choice the person made ⚠ decided for you — please review
4. Review findings F1 to F8 stay as follow-ups, with no change after the review — RULE: skills/code-review/SKILL.md § Phase 4 — Apply Safe Fixes
5. The auto verify run was started with `XDG_CURRENT_DESKTOP` unset and `GSETTINGS_BACKEND=memory` — SAFER OPTION: no agent writes the real desktop settings ⚠ decided for you — please review
6. Three old cargo build caches in this session's scratch folder were deleted after the `/tmp` quota ran out — SAFER OPTION: only regenerable files of this session
7. HUN-302 (AI undo) was left to the session easy-note-mcp-79, which asked for it; this session had not started it — SAFER OPTION: one owner per task ⚠ decided for you — please review
8. Sensitive-path hook: under `[admin]` it did not ask about one reviewer Bash call (`d.key`, pattern `**/*.key`, a Python method name, no key file); logged to `/home/hungnguyenhuu/.claude/harness-anchors/easy-note-mcp-6858179d/sensitive-admin-df0fca59-a759-4a06-a24b-092db63ce7d8.jsonl` — RULE: docs/claude-md/tracks.md § Admin tag
Checks: tests `cargo test --locked` 739 passed 0 failed, `cargo test --locked --features window,tray` 748 passed 0 failed (at 7983685), `cargo test --locked --test hotkey --test setup` 21 and 19 passed 0 failed (at fea21be); reviews: fresh-context reviewer pass (P0 0, P1 0, P2 3, P3 5), Supervisor code-review no P0 or P1, security-review no finding, Companion score fail (70); real run: Demonstration AFTER below (auto-checked, not /verify)
Memory: memory/decisions.md — HUN-303 entry and the HUN-291 entry; memory/learnings.md — isolated desktop checks and the /tmp quota; memory/MEMORY.md — three index lines (all uncommitted on main)
Not done: merge (admin: after the auto verify pass; the failing Companion score row blocks it, so it is yours); F1 to F8; the verify run's notes (full-path or wrapped `quick` commands are missed; a deleted entry comes back; `uninstall` keeps an entry changed to a full path); a real Ctrl+Alt+N press; auto verify: ☑ pass (session c66c2d47-a6ee-45af-ba21-61c10faac46d, commit fea21bece9551c5b476a67351378d6f09f269777, log .claude/verify-runs/HUN-303-fea21bece955.log)

The verify run's note "Your desktop still has no shortcut" is wrong: it came from the memory settings backend of decision 5. A read of the real settings after the run shows the `easy-note` entry with `<Control><Alt>n`.

---

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass / ☐ fail | `tests/hotkey.rs` (17 new HUN-303 tests below line 129; 21 in the file). AC 10 (docs) has no automated test. Gaps: F3 |
| Verification command run | ☑ pass / ☐ fail | `cargo test --locked --test hotkey --test setup` (cleared env, temp HOME, tripwire `gsettings`/`dconf`/`claude` first on PATH): `test result: ok. 21 passed; 0 failed` and `test result: ok. 19 passed; 0 failed`; `no tripwire calls` |
| Negative cases hold | ☑ pass / ☐ fail | Red run, this commit's `tests/hotkey.rs` on `src/` of `dabf507`: `test result: FAILED. 4 passed; 17 failed`. Mutations of `src/surfaces/install.rs` on an exported copy (2 diff lines each; unchanged copy 21 passed): M1 no used-key check 19/2 failed; M2 other entries dropped 20/1; M3 `mod1` not alt 20/1; M4 cleared key turned on 20/1; M5 foreign command overwritten 20/1; M6 uninstall ignores command 20/1; M10 custom conflicts skipped 20/1. **Survived (21 passed): M7, M8, M9, the three read-backs (F3)** |
| verify | ☑ pass | auto-run by hook, session c66c2d47-a6ee-45af-ba21-61c10faac46d, commit fea21bece9551c5b476a67351378d6f09f269777, log .claude/verify-runs/HUN-303-fea21bece955.log — pass |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass / ☐ fail | Reviewed: the 10 files of the diff, callers `cli.rs` `installed()`/`setup()`, `quick.rs` `find_in_path`/`hotkey`, tests that run `setup` or `hotkey` (`tests/setup.rs`, `tests/hotkey.rs`, `tests/cli.rs` loop), CI file, stale references. Skipped: all other source |
| Full smoke suite still green (no regression) | ☑ pass / ☐ fail | `cargo fmt --check` exit 0; clippy `-D warnings` clean for the default and `window,tray` builds. `cargo test --locked`: `passed 739 failed 0 ignored 7`. `cargo test --locked --features window,tray` was **never green in one run**, each time one failure in a file this diff does not touch: run 1 `leak.rs` `planted_secret_never_leaks` (`tag` exit 1, stopped the run, cause not found; passed in runs 2 and 3); run 2 `passed 747 failed 1`, `hook.rs` p95 279 ms at load average 12 (alone: 16/16); run 3 `passed 747 failed 1`, `enrich.rs:878` rejects the text `-1`, which the random temp folder name `sched-bad-1hd0KH` held (alone: 48/48). No tripwire call. **Supervisor rerun at `7983685`**, `cargo test --locked --features window,tray --no-fail-fast`, temp HOME, `XDG_CURRENT_DESKTOP` and `DBUS_SESSION_BUS_ADDRESS` unset: `features exit 0`, `window,tray: passed 748 failed 0 ignored 7`. An earlier Supervisor run had 744/4 while the reviewer built into the same target folder (`setup` ac8, two `tray.rs` "no tray" tests, `hook.rs` p95); none of the 4 failed in the rerun |
| **Companion score (easy-verifier)** | ☐ pass / ☑ fail / ☐ N/A | overall 70 (floor missed: 70 < 80), change_risk 100, pass (feature, confirmed; no failed rule), scope changes, ref `main...HEAD` (HEAD `62ae5b0`), ratings: architecture 100, supply-chain 100, test-strategy 72, requirement-fidelity 67, code-quality 67, blast-radius 65, security 18 (`redaction_hits_observed` 1, `sink_hits_observed` 2; the sink lines are `src/surfaces/quick.rs:226`, `:258`, `tests/hotkey.rs:49`, `:229`; the redaction hit names no line, and a pattern search of the added lines found only the reviewed commit hash), abstentions: solution-fit, reliability, api, data, operations, infrastructure, gates: none answered, solution-fit left abstained (one more rating, even 100, cannot reach 80), test pairs confirmed: 3, divergence: none, the code-review findings were not accepted (their refs are not in the dimension packs), report: `/home/hungnguyenhuu/.easy-verifier-reports/easy-note-mcp-HUN-303/evidence-report-changes-20261126T150514-885918Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☑ N/A | No UI component: command-line output and desktop settings only |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ pass / ☐ fail / ☑ N/A | No UI component |
| **UI: Responsiveness at target viewports** | ☐ pass / ☐ fail / ☑ N/A | No UI component |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (Supervisor, 2026-11-26T10:41:21Z = 17:41:21 +07, before the implementation commit `7983685` of 17:56:57 +07):
the new tests of `tests/hotkey.rs` on the unchanged source. All 17 new tests printed `FAILED`, among them

```
test setup_binds_the_key ... FAILED
assertion `left == right` failed: error: invalid value 'hotkey' for '<TARGET>'
```

The reviewer, who did not hold that capture, showed the same from the prior side: the prior
source (`dabf507`) has `pub enum Target { Claude, Launcher, Tray, Obsidian }` with no `Hotkey`, and the new tests run
on that source on 2026-11-26 give `test result: FAILED. 4 passed; 17 failed`.

**AFTER**: `easy-note install hotkey` at `7983685`, stub fed with this machine's real read-only output, 2026-11-26:
`added Ctrl+Alt+N for easy-note quick (GNOME custom shortcut /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/easy-note/)`
and, with the entry present, `already bound: <Control><Alt>n runs easy-note quick (...)` with 0 writes.

**DELTA**: A person on GNOME can run `easy-note setup` (or `install hotkey`) to bind Ctrl+Alt+N to the new-note box, and remove it with `uninstall hotkey`.

**WITNESS**: qa-expert reviewer, 2026-11-26. No `memory/event-trace/HUN-303.jsonl` was in the worktree.

---

## Supervisor check (2026-11-26, `[admin]`)

- `code-review` on `main...7983685`: the reviewer's findings stand (F1, F5, F7 read in the source and docs). P0: 0, P1: 0,
  so no fix is applied and the reviewed commit stays the code to merge.
- `security-review`: done by the Supervisor inline on `main...7983685` (the skill's own diff showed `main...origin/main`,
  which holds already merged tasks; no sub-task was spawned, the fresh-context reviewer had already done an independent
  pass). No finding: `gsettings` gets an argument list, the written values are constants or list entries as `gsettings`
  printed them, and no new input crosses a trust boundary.
- Lane check: C1 is too low for the size (10 files, 872 lines). Warn only.

**Not done**

- F1 to F8 are open (3 P2, 5 P3). A change to any file outside the review record needs a new reviewer run.
  F1 (`PRD.md` NFR-001 does not list `gsettings`), F5 (write order), F7 and F8 (two wrong sentences in `README.md`) are small.
- No real `gsettings set`/`reset` and no real Ctrl+Alt+N press was exercised.
- The `.deb` description text (F2) is unchanged: the guide forbids touching `scripts/build_deb.sh`.

