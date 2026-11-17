# TASK_REVIEW — HUN-262: Claude detail: session/commit fields, AI files, note_pending_enrich, note_enrich, enrich_notes prompt (Rust)

> Sibling of `tasks/TASK_GUIDE_HUN-262.md`. Everything here is **filled by the reviewer at Stage
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
1. Took HUN-262 into a parallel run with HUN-263 and HUN-265 (the user's request "spawn agents parallel, supervisor [admin]"), each in its own worktree; HUN-266 left out (Owner manual task) — RULE: `docs/claude-md/pipeline-stages.md` § Stage 3: Parallel Execution via Isolation
2. Re-planned the guide for Rust (core `ai.rs`, two new MCP tools and the `enrich_notes` prompt in `mcp.rs`, hook `session`/`commit`, CLI `ai rm` and `tag --accept-ai`) and signed the Requirement Fidelity Gate as Supervisor — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 13)
3. Python `prompts.py` and the uv/pytest steps dropped; the prompt text is a `const` in `src/surfaces/mcp.rs` — FACT: docs/ddr/0005-rust-single-binary.md:31 "4. **All Python is removed** in the first Rust task"
4. AI files are written temp + rename so stale detail can be replaced (the note writer never overwrites) — FACT: src/core/store.rs:283 "Rewrite one note (`edit` / `tag`, FR-008): unknown keys kept, `updated` set, temp + rename."
5. A trashed note's AI file moves to `.easy-note/trash/ai/<note trash file name>.json`, is emptied with its note by the note's mtime, and is restored without replace — DEBATED: trash/ai/ keyed by the note's trash file name (Advocate 1: trash/ai/ keyed by the note's trash file name, pairwise delete by the note's mtime; Advocate 2: the same, plus orphan cleanup) ⚠ decided for you — please review
6. The `note_add(detail)` AI file holds summary = masked detail, title = the note's title, kind = other, tags_suggested = [], model = capture; a fresh capture file is not pending — DEBATED: the full D34 shape with server defaults (Advocate 1: full shape, documented in NOTE_FORMAT; Advocate 2: full shape plus listing capture files as pending, refused by PRD.md:89 "must list notes with no AI file or a stale one") ⚠ decided for you — please review
7. Stage 4 findings fixed in f644d0f: restore-clash warning and NOTE_FORMAT made true, `get_prompt` errors masked like `call_tool`, inbox AI naming limit documented, 4 data-safety tests added; PROJECT_SPEC MCP contract rows updated for `detail` and `ai` — RULE: `docs/claude-md/pipeline-stages.md` § Stage 4: Review (item 7, "Address all findings before moving to Stage 5")
8. Left as follow-ups, not fixed here: caps on `model` and suggested-tag length, `session` id shape, masked inbox ids in pending, control characters in `model`/`title`, restore order race, `--accept-ai` on stale detail, `select` loading AI files on `list`, small duplication, leftover AI temp files, a stronger `--accept-ai` test — RULE: `skills/code-review/SKILL.md` § Phase 4 — Apply Safe Fixes ("Present P2/P3 fixes as suggestions only")
9. The implementer's fix commit was stopped by the step limit after the work was done; the Supervisor reviewed the diff, reran the full suite (418 passed, 0 failed) and committed it as f644d0f — RULE: `docs/claude-md/tracks.md` `### Stops that stay` (a stop only when the same sub-agent hits the step limit twice)
Checks: tests `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` — 418 passed, 0 failed; reviews code-review — 0 P0/P1, 4 P2 fixed or documented, 1 P2 + 9 P3 follow-ups (`reports/code-review_feat-HUN-262-claude-detail_20261117T110610.html`); security-review — no high-confidence vulnerabilities, 2 candidates filtered as false positives 9/10 and 8/10 (`reports/security-review_feat-HUN-262-claude-detail_20261117T110610.html`); real run `easy-note ai rm` / `tag --accept-ai` / `serve` tools/list and prompts/list on the branch build — 7 tools and the `enrich_notes` prompt listed (auto-checked, not /verify)
Memory: nothing new in memory/ on this branch (the run's learnings are written once on HUN-265's branch, to keep the parallel branches free of memory conflicts)
Not done: merge (admin: after the auto verify pass); auto verify: running (T182); AC 10 token count over 20 notes (yours); follow-ups of decision 8 (a new task); readiness incomplete: `Approved by` empty, no codebase map (yours)

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/ai.rs` (29 tests: the locked oracle `note_files_stay_byte_identical_around_every_ai_operation`, AC 1–9, success criteria 1–5, plus the 4 Stage 4 data-safety tests), `tests/mcp.rs` (`exactly_the_seven_tools_and_one_prompt_are_registered`; the real-binary smoke test calls the 2 new tools), `tests/hook.rs` and `tests/context.rs` (session and commit), `tests/leak.rs` (`ai` surface and the `prompts/get` case; AC 11), `tests/query.rs` (`enriched`). AC 10 is the owner's. Full run: 418 passed, 0 failed |
| Verification command run | ☑ pass | Supervisor rerun 2026-11-17T03:51:33Z at 2a2540a (`FMT_OK`; clippy `Finished dev profile`, no warnings; `passed 414 failed 0`) and after the fix commit f644d0f (`FMT_OK`; clippy clean; `passed 418 failed 0`) |
| Negative cases hold | ☑ pass | Mutation `store.rs:480` `AiCarry::InTheWay` → `AiCarry::Done` (diff stat: `src/core/store.rs \| 2 +-`): `test rm_keeps_both_when_a_trashed_ai_file_is_in_the_way ... FAILED`, `test restore_keeps_a_live_ai_file_and_warns_the_trashed_copy_goes_at_the_next_empty ... FAILED`, `test result: FAILED. 27 passed; 2 failed`; reverted. Implementer's mutations: masking off on AI read → `planted_secret_never_leaks` failed for all 17 planted kinds; `get_prompt` unmasked → the new leak case failed for every kind |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewer 1: `core/ai.rs`, `store.rs`, `query.rs`, `context.rs`, the CLI files, `quick.rs` and their tests; reviewer 2: `mcp.rs`, `hook.rs`, `ai.rs`, `context.rs`, `tests/leak.rs` (binary driven with a temp HOME); Supervisor: entry point, stale wording, PROJECT_SPEC contract. Skipped: `format.rs`, `guard.rs`, `install.rs` (unchanged, on the guide's do-not-touch list) |
| Full smoke suite still green (no regression) | ☑ pass | `cargo test` after f644d0f: every suite `test result: ok`, 418 passed, 0 failed (lib 32, ai 29, cli 43, context 32, format 13, guard 37, hook 16, hotkey 4, inbox 19, install 23, leak 46, mcp 20, query 45, quick 26, readme 3, store 30) |
| **Companion score (easy-verifier)** | ☑ N/A | Not run in this auto run (no companion tool session); the merge gate blocks on ☑ fail only |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | No UI: CLI, MCP and hook output only, no screen |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | No UI: CLI, MCP and hook output only, no screen |
| **UI: Responsiveness at target viewports** | ☑ N/A | No UI: CLI, MCP and hook output only, no screen |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured at main 1836e22, before any HUN-262 implementation commit:

```
Tue Nov 17 03:29:22 UTC 2026
binary: release build of main 1836e22, temp HOME and EASY_NOTE_DIR
$ easy-note add before capture note #hun262
easy-note: created notes dir /tmp/tmp.DmuTdddKPf/notes
✓ noted 20261117-102922-5d2b · easy-note-mcp · HUN-262
exit=0
$ easy-note ai rm 20261117-102922-5d2b
error: unrecognized subcommand 'ai'

Usage: easy-note <COMMAND>

For more information, try '--help'.
exit=2
$ easy-note tag 20261117-102922-5d2b --accept-ai
error: unexpected argument '--accept-ai' found

  tip: to pass '--accept-ai' as a value, use '-- --accept-ai'

Usage: easy-note tag <ID> <TAGS>...

For more information, try '--help'.
exit=2
$ easy-note serve  (stdio: initialize, tools/list, prompts/list)
tools: ['note_add', 'note_list', 'note_read', 'note_search', 'note_stats']
prompts: {'prompts': []}
files under .easy-note/ai: 0
```

**AFTER**: the same script on the branch build (f644d0f):

```
Tue Nov 17 04:06:19 UTC 2026
binary: debug build of feat/HUN-262-claude-detail f644d0f, temp HOME and EASY_NOTE_DIR
$ easy-note add before capture note #hun262
easy-note: created notes dir /tmp/tmp.TGCGRpRUug/notes
✓ noted 20261117-110619-9ad7 · easy-note-mcp · HUN-262
exit=0
$ easy-note ai rm 20261117-110619-9ad7
easy-note: no AI detail for 20261117-110619-9ad7
exit=1
$ easy-note tag 20261117-110619-9ad7 --accept-ai
easy-note: no AI detail for 20261117-110619-9ad7
exit=1
$ easy-note serve  (stdio: initialize, tools/list, prompts/list)
tools: ['note_add', 'note_enrich', 'note_list', 'note_pending_enrich', 'note_read', 'note_search', 'note_stats']
prompts: {'prompts': [{'name': 'enrich_notes', 'description': "Add AI detail (title, kind, summary, suggested tags) to the person's notes that have none, in one subagent. The notes never change."}]}
files under .easy-note/ai: 0
```

**DELTA**: A person's own Claude can now add AI detail (title, kind, summary, suggested tags) to their notes through `note_pending_enrich`, `note_enrich` and the `enrich_notes` prompt, kept in separate masked files that `show`, `search` and `stats` label and that `ai rm`, `tag --accept-ai`, `rm` and `restore` handle, without ever changing a note file.

**WITNESS**: the Supervisor (session e61f3928), in the HUN-262 worktree at 2026-11-17T03:51:33Z (tests) and 04:06:19Z (AFTER), separately from the implementing agent; two independent review agents drove the binary; trace `memory/event-trace/HUN-262.jsonl`
