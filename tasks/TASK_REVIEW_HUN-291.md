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
Paused: auto-mode check failed — no `Memory:` line in the report — run the memory pass before you stop and record it there (each entry written, or `Memory: nothing new`); no test run in memory/event-trace/HUN-291.jsonl — write /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID with the task ID (minimal-track step 1): `mkdir -p /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d && printf '%s\n%s\n' "HUN-291" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID`, then rerun the task's tests so the trace records them under the task, then stop; extended track, but the trace has no `code-review` Skill call (Stage 4 runs in auto mode) [report be25f1359954]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/cli.rs` (`ls_and_search_show_edited_only_on_an_edited_note` AC 1, `an_updated_before_created_or_in_the_future_shows_the_local_stamp` AC 5, `ai_runs_never_set_updated_and_accept_ai_does` AC 4), `tests/mcp.rs` (`list_search_and_read_carry_updated_only_when_the_note_has_it` AC 2), `tests/window_api.rs` (four `hun291_*` tests, AC 3 and 5) with `tests/fixtures/window/page_harness.js`, `src/core/query.rs` (`ago_changes_unit_at_each_boundary`). All pass; output below the table |
| Verification command run | ☑ pass | Full guide command on the source committed as `c2f6101` (run just before the commit, no source change between), 2026-11-26T11:15:45 to 11:25:53 +07:00, with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`: `VERIFY EXIT 0`; fmt and both clippy runs clean; default features 655 passed, 0 failed, 7 ignored; `window,tray` 664 passed, 0 failed, 7 ignored |
| Negative cases hold | ☑ pass | Red run before the implementation: cli 0 passed 3 failed, the new mcp test failed, the 4 new window tests failed. Mutation for AC 4 (a `store.update` call added at the top of `ai::enrich`, then of `ai::organize`): `git diff --stat` showed `src/core/ai.rs`, 1 insertion each time, and `ai_runs_never_set_updated_and_accept_ai_does` FAILED on `byte for byte` both times. After the revert the diff of `src/core/ai.rs` is empty and the test passes. Output below the table |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☐ pass / ☐ fail | [what was reviewed vs. skipped, and why] |
| Full smoke suite still green (no regression) | ☑ pass | The whole `cargo test --locked` suite in both feature sets: 0 failed (verification row) |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
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
