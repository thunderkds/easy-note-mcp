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
Checks: pending
Memory: pending
Not done: merge (admin: after the auto verify pass); auto verify: not started; AC 10 token count over 20 notes (yours); readiness incomplete: `Approved by` empty, no codebase map (yours)

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☐ pass / ☐ fail | [test file path(s) — required before Done] |
| Verification command run | ☐ pass / ☐ fail | [paste actual output] |
| Negative cases hold | ☐ pass / ☐ fail | [paste the red run against the prior content, or the mutation diff (`git diff --stat`, non-empty) before each verdict] |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☐ pass / ☐ fail | [what was reviewed vs. skipped, and why] |
| Full smoke suite still green (no regression) | ☐ pass / ☐ fail | |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
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

**AFTER**: [same command, post-change] OR [verbatim excerpt of the new content]

**DELTA**: [one sentence — what a user can now do that they could not before]

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-262.jsonl`, never the
implementing agent alone]
