# TASK_REVIEW — HUN-264: README, RUNBOOK, client config snippets (minimal track)

> Sibling of `tasks/TASK_GUIDE_HUN-264.md`. Everything here is **filled by the reviewer at Stage
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
1. Track minimal — C0, Risk Low, docs plus one test, no UI, no user data; the Decision record is `tasks/MINIMAL_LOG.md` § HUN-264 — RULE: `docs/claude-md/tracks.md` `## Tracks`
2. Install is `cargo install --path . --locked` plus an `n` symlink, not `uv tool install` — FACT: docs/ddr/0005-rust-single-binary.md:30 "3. Install: `cargo install --path .` (from a checkout)"
3. The README lists the 5 current MCP tools only; the enrich tools wait for HUN-262 and the owner's D37 approval — SAFER OPTION: docs must match the binary ⚠ decided for you — please review
4. The command check scans only code spans and fenced blocks — SAFER OPTION: no false failures on prose, every shown command still checked
5. Cursor's snippet is not run (`cursor-agent` not signed in; sign-in writes `~/.cursor`) — RULE: tasks/TASK_GUIDE_HUN-264.md § Files Must NOT Touch
Checks: tests `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` — 379 passed, 0 failed (readme 3, red first `1 passed; 2 failed`); reviews code-review — 0 findings (lane EXCEEDS by the record and the test file); real run `claude mcp list` (temp HOME) — printed `easy-note: easy-note serve - ✔ Connected` (auto-checked, not /verify)
Memory: nothing new
Not done: merge (admin: after the auto verify pass); auto verify: ☑ pass (session 657e766b-020c-46dc-a747-262d0190c2b1, commit 5e090f49c85b8812fa182c27300b868b81f7918a, log .claude/verify-runs/HUN-264-5e090f49c85b.log); Cursor snippet run (yours)

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/readme.rs`: `test result: ok. 3 passed; 0 failed` (red first: `1 passed; 2 failed`). Full record: `tasks/MINIMAL_LOG.md` § HUN-264 (minimal track). |
| Verification command run | ☑ pass | `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` at 208ad51 — 379 passed, 0 failed |
| Negative cases hold | ☐ pass / ☐ fail | [paste the red run against the prior content, or the mutation diff (`git diff --stat`, non-empty) before each verdict] |
| verify | ☑ pass | auto-run by hook, session 657e766b-020c-46dc-a747-262d0190c2b1, commit 5e090f49c85b8812fa182c27300b868b81f7918a, log .claude/verify-runs/HUN-264-5e090f49c85b.log — pass |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☐ pass / ☐ fail | [what was reviewed vs. skipped, and why] |
| Full smoke suite still green (no regression) | ☐ pass / ☐ fail | |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | docs-only task, no UI |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | docs-only task, no UI |
| **UI: Responsiveness at target viewports** | ☑ N/A | docs-only task, no UI |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: [pasted timestamped command output showing the thing absent/failing, captured before the
first implementation commit] OR [verbatim excerpt of the prior content, for non-executable changes]

**AFTER**: [same command, post-change] OR [verbatim excerpt of the new content]

**DELTA**: [one sentence — what a user can now do that they could not before]

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]
