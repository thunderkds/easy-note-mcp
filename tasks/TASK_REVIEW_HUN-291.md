# TASK_REVIEW — HUN-291: show the edited time

> Sibling of `tasks/TASK_GUIDE_HUN-291.md`. Everything here is **filled by the reviewer at Stage
> 4/5** — it is deliberately NOT in the guide, because the implementing agent re-reads the guide on
> every turn and never fills these two sections.
>
> Consumers resolve each section **guide first, this file second** (`.claude/hooks/lib/guide_sections.py`):
> a legacy guide that still carries these sections inline keeps working unchanged, and a stray
> review file can never override an inline section.

---

<!-- Optional: the final report, only for a task run in auto mode (the user's `[auto]` or `[admin]` tag; with `[admin]`, the `Not done:` line names `merge (admin: after the auto verify pass)`,
`docs/claude-md/tracks.md` `## Auto mode`). To use it, copy the block below out of this comment, to this
place, and replace every <...>. Keep `Mode: auto` as its first line, exactly as written: T182's hook reads
it. Write it to this file before the run ends; the chat message is a copy, not the report. The same
block is the Supervisor's final chat message. It is not the `delivery-report` page and not
Evidence. A taste decision (what the user wants) also ends with the flag. Write `DEBATED` only when ≥2
Advocate `Agent` calls ran for that question, naming each Advocate's option. A decision with no Advocates is
labelled `RULE`, `FACT` or `SAFER OPTION`, never `DEBATED`. A decision flagged `⚠ decided for you` is a choice, and is never labelled `FACT`. A
safety action a rule requires (for example the backup before a delete) is `RULE` without the flag.
At Risk Medium or High, a `RULE:` counts only when it names a written rule (`path:line`,
`path § <heading>`, or `docs/claude-md/tracks.md` `### <section>`), and a `FACT:` needs a Citation
(`path:line "quoted text"`); without one it is a choice and must be `DEBATED`. Good: `2. backed up the CSV
— RULE: `docs/claude-md/tracks.md` `### Stops that stay``. Bad: `1. kept recurring.csv — RULE: Surgical Changes`.
`Memory:` records the diff-driven memory pass you run before you stop: each entry written, or `nothing new`.
After an auto verify pass the hook, never you, adds `Closed: row moved to Done <date> — …` and moves the row,
or adds `Paused: done not moved — <reason>`. It moves the row only when this block holds `Mode: auto`, a
numbered decision, `Checks:`, `Memory:`, the `Not done:` auto verify slot with its result, and `Not done:`
naming the merge — no `<...>` left (`docs/claude-md/tracks.md` `### Auto close-out`).
With `/verify (yours)`, your pass goes in the `verify` row below: Result `☑ pass`, Notes `/verify by the user, session <id>, commit <sha>`.

## Auto decisions

Mode: auto
1. <what was decided> — RULE: <the rule>
2. <what was decided> — FACT: <path>:<line> "<quoted text>"
3. <what was decided> — DEBATED: <option> (Advocate 1: <its proposal>; Advocate 2: <its proposal>) ⚠ decided for you — please review
4. <what was decided> — SAFER OPTION: <why> ⚠ decided for you — please review
Paused: <only when a stop that stays paused the task: which one, and why>
Checks: tests `<command>` — <result>; reviews <skill — result>; real run `<command>` — printed `<what it printed>` (auto-checked, not /verify)
Memory: <each entry the memory pass wrote, as `file — entry` | nothing new>
Not done: merge (yours); <auto verify: running (T182) | auto verify: <result> | /verify (yours), with [admin] only after the one retry also ended blocked>

-->

## Auto decisions

Mode: auto
1. The three ready tasks (HUN-291, HUN-294, HUN-300) are built in parallel, one worktree and one implementer each, because the user asked for parallel execution — RULE: docs/claude-md/pipeline-stages.md § Stage 3: Parallel Execution via Isolation
2. Requirement Fidelity Gate signed `auto (Supervisor)` — RULE: docs/claude-md/tracks.md § Every user stop and its auto rule
3. AC 2's `--json` clause is waived and no flag is added: `ls` and `search` have no such flag and FR-046 and FR-016 do not ask for one — SAFER OPTION: the smaller change, and a flag can be added later ⚠ decided for you — please review
4. The six P3 review findings stay as follow-ups, with no code change after the review — RULE: skills/code-review/SKILL.md § Finding Classification
Checks: tests `cargo test --locked` 655 passed 0 failed, `cargo test --locked --features window,tray` 664 passed 0 failed; reviews: fresh-context reviewer pass (P0 0, P1 0, P2 1, P3 6), Supervisor code-review no P0 or P1, Companion score fail (74); real run: Demonstration AFTER below (auto-checked, not /verify)
Memory: nothing new on this branch; the HUN-291 decision entry is written on main after the merge
Not done: merge (admin: after the auto verify pass; the failing Companion score row blocks it, so it is yours); the six P3 follow-ups; auto verify: running (T182)
Paused: auto-mode check failed — no `Memory:` line in the report — run the memory pass before you stop and record it there (each entry written, or `Memory: nothing new`); no test run in memory/event-trace/HUN-291.jsonl — write /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID with the task ID (minimal-track step 1): `mkdir -p /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d && printf '%s\n%s\n' "HUN-291" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID`, then rerun the task's tests so the trace records them under the task, then stop; extended track, but the trace has no `code-review` Skill call (Stage 4 runs in auto mode) [report be25f1359954]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/cli.rs` (`ls_and_search_show_edited_only_on_an_edited_note` AC 1, `an_updated_before_created_or_in_the_future_shows_the_local_stamp` AC 5, `ai_runs_never_set_updated_and_accept_ai_does` AC 4), `tests/mcp.rs` (`list_search_and_read_carry_updated_only_when_the_note_has_it` AC 2), `tests/window_api.rs` (four `hun291_*` tests, AC 3 and 5) with `tests/fixtures/window/page_harness.js`, `src/core/query.rs` (`ago_changes_unit_at_each_boundary`). All pass; output below the table |
| Verification command run | ☑ pass | Full guide command on the source committed as `c2f6101` (run just before the commit, no source change between), 2026-11-26T11:15:45 to 11:25:53 +07:00, with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`: `VERIFY EXIT 0`; fmt and both clippy runs clean; default features 655 passed, 0 failed, 7 ignored; `window,tray` 664 passed, 0 failed, 7 ignored |
| Negative cases hold | ☑ pass | Red run before the implementation: cli 0 passed 3 failed, the new mcp test failed, the 4 new window tests failed. Mutation for AC 4 (a `store.update` call added at the top of `ai::enrich`, then of `ai::organize`): `git diff --stat` showed `src/core/ai.rs`, 1 insertion each time, and `ai_runs_never_set_updated_and_accept_ai_does` FAILED on `byte for byte` both times. After the revert the diff of `src/core/ai.rs` is empty and the test passes. Output below the table |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Stage 4 reviewer, `main...89ceb0e`. Reviewed: the 13 changed files; callers `src/surfaces/mcp.rs` (list, search, read), `src/surfaces/tray.rs` (`recent`, `relative`), `src/core/format.rs` (time check), `src/core/store.rs` (clock, `update`, inbox notes), `src/surfaces/cli.rs`; docs sweep of README, RUNBOOK, `docs/NOTE_FORMAT.md`, `PROJECT_SPEC.md`, `PRD.md` FR lines. Skipped: the rest of the repo (no caller of the changed code found there); screenshots and the event trace, which are outside this worktree |
| Full smoke suite still green (no regression) | ☑ pass | The whole `cargo test --locked` suite in both feature sets: 0 failed (verification row) |
| **Companion score (easy-verifier)** | ☑ fail | overall 74 (floor 80 missed), change_risk pass (feature, value 100), scope changes, ref `main...HEAD` at 7275b0c, abstentions: reliability, api, data, operations, infrastructure; gates: code-quality (74) and solution-fit (85) answered, none left abstained; no findings were sent, so no divergence. security 30 comes from 3 redaction hits over the changed files; the diff adds no secret-shaped value (grep of added lines: 0), and `tests/mcp.rs` already holds planted test secrets. report: `/home/hungnguyenhuu/.easy-verifier-reports/easy-note-mcp-HUN-291/evidence-report-changes-20261126T044232-422744Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☑ pass | Headless Chrome, stub IPC fed real `window::api::handle` answers (easy-ui not used), before and after at 375 and 1280 px, in `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/df0fca59-a759-4a06-a24b-092db63ce7d8/scratchpad/hun291/ux/`. `edited 2h ago` shows beside the time on the edited row only; the never-edited row has no such text; the detail shows the full stamp; row height 57 px before and after. Also rendered in WebKitGTK 2.52.6 (`webkit-375.png`, `webkit-1280.png`) |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ pass | Computed style: row line IBM Plex Mono 12px `#8C9590` (token text-muted), detail line 12.5px `#ABB3AE` (text-2), bundled Plex fonts loaded. No new color or font; the CSS adds layout rules only, with the existing 12px and 2px spacing. Older fail, outside this diff and not fixed: the row title is 14px at 375 px, under the 16px of `ux-evaluate` check B-e |
| **UI: Responsiveness at target viewports** | ☑ pass | 375 and 1280 px: page scroll width equals the viewport, the `edited` text is never cut, the full stamp stays on one line. The first 375 px run cut `edited just now` and the date; a narrow-window CSS rule fixed it and the shots were retaken. Older fails, outside this diff and not fixed: touch targets under 44px at 375 px (New note 38, search 38, filters 30, actions 34; check B-d) and 14px body text (B-e). The real dashboard window has a minimum size of 800x560 |

Pasted output (HUN-291).

New tests, from the full verification log (each ran in both feature sets):

```
test core::query::tests::ago_changes_unit_at_each_boundary ... ok
test an_updated_before_created_or_in_the_future_shows_the_local_stamp ... ok
test ai_runs_never_set_updated_and_accept_ai_does ... ok
test ls_and_search_show_edited_only_on_an_edited_note ... ok
test list_search_and_read_carry_updated_only_when_the_note_has_it ... ok
test hun291_an_updated_before_created_shows_the_stamp_in_the_window ... ok
test hun291_board_and_detail_carry_updated_only_on_the_edited_note ... ok
test hun291_the_windows_edit_and_add_as_tags_show_edited_just_now ... ok
test hun291_the_page_shows_edited_on_the_row_and_the_full_stamp_in_the_detail ... ok
VERIFY EXIT 0
2026-11-26T11:25:53+07:00
```

Red run before the implementation:

```
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 48 filtered out   (tests/cli.rs)
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 19 filtered out   (tests/mcp.rs)
test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 93 filtered out   (tests/window_api.rs)
```

Mutation check for AC 4:

```
== mutation in ai::enrich
 src/core/ai.rs | 1 +
 1 file changed, 1 insertion(+)
test ai_runs_never_set_updated_and_accept_ai_does ... FAILED
assertion `left == right` failed: byte for byte
== mutation in ai::organize
 src/core/ai.rs | 1 +
 1 file changed, 1 insertion(+)
test ai_runs_never_set_updated_and_accept_ai_does ... FAILED
assertion `left == right` failed: byte for byte
== after revert
diff stat of src/core/ai.rs: []
test ai_runs_never_set_updated_and_accept_ai_does ... ok
```

`ux-evaluate` (implementer run): 8 pass, 2 fail (both older), 0 not checked.

| Check | Width | Verdict | Reason |
|---|---|---|---|
| US-023 edited text, row not taller | 375 / 1280 | pass | `edited 2h ago` on edited rows only; row 57px before and after |
| B-a primary action | 375 / 1280 | pass | `+ New note` prominent and in view |
| B-b hierarchy | 375 / 1280 | pass | one reading order |
| B-c spacing | 375 / 1280 | pass | existing 12px and 2px spacing |
| B-d 44px targets | 375 | fail (older) | New note 38px, search 38px, filters 30px, actions 34px; unchanged by this diff |
| B-e 16px body text | 375 | fail (older) | row title 14px; unchanged by this diff |
| B-f states | 375 / 1280 | pass | edited and never edited; the guide names no other state |
| B-g feedback | 375 / 1280 | pass | an edited note shows `edited just now` (window test and stub shot) |
| B-h phone layout | 375 | pass | no horizontal scroll; `edited` text never cut |
| B-i one visual language | 375 / 1280 | pass | no new color or control |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured at `9df4590` (no implementation commit yet), binary from `cargo build --locked`. Command:
`sh demo.sh target/debug/easy-note`, where `demo.sh` sets a temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`,
adds two notes, edits the second with a stub `EDITOR`, then runs `ls`, `search note` and greps `updated:`.
The note file has `updated`, and neither `ls` nor `search` shows it:

```
2026-11-26T11:08:24+07:00
easy-note: created notes dir /tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/df0fca59-a759-4a06-a24b-092db63ce7d8/scratchpad/hun291/hun291-demo.wrS76y/notes
✓ noted 20261126-110824-2f6b
$ easy-note edit <id>
✓ edited 20261126-110825-7ef6
$ easy-note ls
20261126-110825-7ef6  note that gets edited #demo  #demo
20261126-110824-2f6b  never edited note
$ easy-note search note
20261126-110825-7ef6  note that gets edited #demo  #demo
20261126-110824-2f6b  never edited note
$ grep updated: <note files>
notes/2026/11/20261126-110825-7ef6.md:updated: "2026-11-26T11:08:26+07:00"
```

**AFTER**: same command at `c2f6101`:

```
2026-11-26T11:28:41+07:00
easy-note: created notes dir /tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/df0fca59-a759-4a06-a24b-092db63ce7d8/scratchpad/hun291/hun291-demo.b8nTCY/notes
✓ noted 20261126-112841-b585
$ easy-note edit <id>
✓ edited 20261126-112842-f00d
$ easy-note ls
20261126-112842-f00d  note that gets edited #demo  #demo  edited just now
20261126-112841-b585  never edited note
$ easy-note search note
20261126-112842-f00d  note that gets edited #demo  #demo  edited just now
20261126-112841-b585  never edited note
$ grep updated: <note files>
notes/2026/11/20261126-112842-f00d.md:updated: "2026-11-26T11:28:44+07:00"
```

**DELTA**: A person now sees which notes were edited, and when, in `ls`, `search`, MCP list and search, and the dashboard window.

**WITNESS**: Run by the HUN-291 implementer and recorded in `memory/event-trace/HUN-291.jsonl` of the main checkout (BEFORE record 2026-11-26T04:08:26Z; AFTER output stamped 2026-11-26T11:28:41+07:00). No second party has witnessed it yet.

---

## Stage 4 review

Reviewer: HUN-291, fresh context, 2026-11-26. Commit `89ceb0e63fca213266afd07d6b64f1454f714d15`, diff `main...89ceb0e`. Risk Low, so no `security-review` run.

**Verdict: pass. P0 0, P1 0, P2 1, P3 6. No source or test file changed, no fix applied.** One clause of AC 2 (`--json`) has nothing to check (P2 below); the Supervisor must amend or waive it.

`lane_check`: `lane: C1, 12 files, 464 lines, EXCEEDS (files 12 > 6; lines 464 > 250); unpredicted: src/surfaces/window/assets/app.css, src/surfaces/window/assets/app.js, src/surfaces/window/assets/ipc.js, tasks/TASK_GUIDE_HUN-291.md` (warn only: the task may be under-labelled).

### Acceptance Criteria

| AC | Result | Code and test |
|----|--------|---------------|
| 1 | met | `src/surfaces/cli/read.rs` `print_page`; `ls_and_search_show_edited_only_on_an_edited_note` compares both whole lines for `ls` and `search` |
| 2 | met for every output that exists; the `--json` clause cannot be met or checked | `NoteItem.updated` is skipped when absent; `note_read` returns all front matter; `list_search_and_read_carry_updated_only_when_the_note_has_it`. See P2 |
| 3 | met | `add_edited` in `src/surfaces/window/api.rs`, `renderRow` / `renderDetail` in `app.js`; `hun291_board_and_detail_carry_updated_only_on_the_edited_note`, `hun291_the_page_shows_edited_on_the_row_and_the_full_stamp_in_the_detail` |
| 4 | met | `ai_runs_never_set_updated_and_accept_ai_does` (fake `claude`): `ai enrich`, `ai::organize`, `ai set`, `ai undo` leave the bytes equal; `tag --accept-ai` sets `updated`. There is no batch run command yet, so the batch organizer is covered at its writer only |
| 5 | met | `set_back` and the negative-age branch in `src/core/query.rs`; `hun291_an_updated_before_created_shows_the_stamp_in_the_window` (set back), `an_updated_before_created_or_in_the_future_shows_the_local_stamp` (future) |

Entry points (`ls`, `search`, `show`, `note_list`, `note_search`, `note_read`, `board`, `detail`) all reach the change; `show` is unchanged and already prints `updated`.

### Findings

| Sev | Where | Finding and failure scenario | Conf | Action |
|-----|-------|------------------------------|------|--------|
| P2 | `tasks/TASK_GUIDE_HUN-291.md` AC 2; `src/surfaces/cli.rs:136`, `:144` | AC 2 names "`--json` output", but `ls` and `search` have no `--json`: both print `error: unexpected argument '--json' found`. Only `stats --json` exists and it holds no note items. Scenario: a script written from AC 2 runs `easy-note ls --json` and fails. No record in the guide or in Auto decisions says the clause was dropped. PRD FR-046 and FR-016 do not ask for the flag | 100 | Supervisor: remove or waive the clause and record it; a new flag needs its own request |
| P3 | `tests/cli.rs:1385-1392` | The set-back case uses `updated` 2020-01-01, more than 30 days old, so it prints a date without the `set_back` rule. Scenario: delete the filter at `src/core/query.rs:368` and this test still passes; only the window test fails | 100 | Use a recent `created` and an earlier `updated` |
| P3 | `tests/window_api.rs:1224` | Comment says that with `details` "the HUN-286 edit scenarios run too"; the harness now needs `roles` for them. Scenario: a caller passes notes only and expects edit results that are never produced | 100 | Say that `roles` is needed |
| P3 | `src/core/query.rs:377`; `src/surfaces/tray.rs:121` | Two relative-time helpers with two wordings: the tray shows `5 min ago` (age of `created`), the new text shows `5m ago`. Scenario: a later change to the units reaches only one of them | 75 | Optional: one helper, or leave both on purpose |
| P3 | `src/surfaces/window/assets/app.js:632` (older behavior) | A focus refresh reloads the board, not the open detail. Scenario: note open in the window, `easy-note edit` in a terminal, focus the window: the row says `edited just now`, the panel has no stamp until the row is clicked again | 100 | Optional follow-up |
| P3 | `app.css` (older, from `ux-evaluate`) | Touch targets under 44px at 375 px (check B-d). Rules unchanged by this diff | 75 | Optional; the real window is at least 800x560 (`mod.rs:483`) |
| P3 | `app.css` (older, from `ux-evaluate`) | Row title is 14px at 375 px, under 16px (check B-e). Unchanged by this diff | 100 | Optional |

Not checked: the `Startup reads:` line of the implementer's final report (the report is not in the repo); `ux-evaluate` was not run again (this reviewer had no Skill tool), its table above was read and checked against the CSS.

### UI Evidence rows against the code

- Visual regression, responsiveness: re-measured with headless Chrome, a stub IPC and 5 hand-made rows, assets of `89ceb0e` at 375, 481, 800 and 1280 px, assets of `main` at 375 px. Every row is 57px (also on `main`); no `edited …` text is cut; the full stamp is on one line inside the panel; no horizontal scroll. The recorded rows hold. The narrow rule (`max-width: 480px`) also moves the second line of never-edited rows under the chip; it never applies in the real window (minimum 800px).
- Design system: the new CSS adds no color or font. `.row-where` is `#8C9590` (token `text-muted`), `.context` is `#ABB3AE` (`text-2`), as recorded.
- WebKitGTK screenshots and the recorded screenshot files were not opened (outside the repo). The guide named easy-ui; the row says headless Chrome was used.

### Commands run at `89ceb0e`

Run with a temp `HOME` and `XDG_CONFIG_HOME`, no `DISPLAY` or `WAYLAND_DISPLAY`, `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, and a trap `claude` first on `PATH` (`~/.local/bin` removed). The trap was never called.

```
cargo fmt --check                                                        FMT_OK (exit 0)
cargo clippy --all-targets --locked -- -D warnings                       CLIPPY_DEFAULT_OK (exit 0)
cargo test --locked                                                      EXIT 0   passed 655 failed 0 ignored 7 suites 27
cargo clippy --all-targets --locked --features window,tray -- -D warnings   CLIPPY EXIT 0
cargo test --locked --features window,tray                               TEST EXIT 0   passed 664 failed 0 ignored 7 suites 27
```

The nine new tests passed (`ago_changes_unit_at_each_boundary`, three in `tests/cli.rs`, one in `tests/mcp.rs`, four `hun291_*`). Red run and mutation runs were not repeated.
