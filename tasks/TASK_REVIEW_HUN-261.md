# TASK_REVIEW — HUN-261: Read-only inbox notes (Rust)

> Sibling of `tasks/TASK_GUIDE_HUN-261.md`. Everything here is **filled by the reviewer at Stage
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
1. Kept HUN-261 (already claimed In Progress and built in session 1, which took it over HUN-259 although HUN-259 was first in board order); the loop's next pick follows the literal rule — first ready ticket in board order, claimed on the board at once, board re-read before work — DEBATED: first in board order, claim then re-check (Advocate 1: first in board order, claim at once, re-read, back off if another session claimed it; Advocate 2: first in board order, claim at once, re-read, loser of a fixed tie-break moves on — both: a deliberate second pick is symmetric and collides too) ⚠ decided for you — please review
2. Re-planned the guide for Rust (Rust paths, cargo verification command, AC 9 path escape and AC 10 read-only refusal, a binding Design list) and signed the Requirement Fidelity Gate as Supervisor — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 13)
3. Spawned the Backend-Implementer on `sonnet` — FACT: PROJECT_SPEC.md:169 "haiku | sonnet | sonnet or opus | opus"
4. Inbox ids keep `id` fixed to `inbox/<stem>` even when the file has valid `easy-note: 1` front matter (every other key wins), so `show`/`rm`/`restore` always route to the file; docs/NOTE_FORMAT.md Inbox sentence narrowed to say so — DEBATED: id stays `inbox/<stem>`, doc-only change (Advocate 1: keep code, narrow the doc, since every route keys on the `inbox/` prefix, src/core/store.rs:259,342,355,521; Advocate 2: same, the code and tests/inbox.rs:260 already check it) ⚠ decided for you — please review
5. Trash keeps the origin in the trash file name (`inbox__<file name>`) instead of a sidecar file; Trash section of docs/NOTE_FORMAT.md now documents it — DEBATED: file-name prefix (Advocate 1: prefix, one atomic move, no orphaned sidecar, src/core/store.rs:341; Advocate 2: prefix, keeps the extension and shared stems, matches `inbox__` AI file names at docs/NOTE_FORMAT.md:104) ⚠ decided for you — please review
Checks: tests `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` at 679e8da (2026-11-16 10:26 UTC) — exit 0: lib 20, cli 43, context 29, format 13, guard 37, inbox 16, leak 46, mcp 19, query 45, store 30 passed; code-review (Supervisor, inline) — no P0/P1/P2; P3: lane check EXCEEDS (1194 lines > 250, mostly tests), `edit` calls `ensure_writable` after `read`; security-review — 1 finding, Medium, confidence 8/10, NOT FIXED: inbox file names become ids, and `id` is never masked, so a secret in an inbox file name (`inbox/ghp_….md`, `inbox/password=x.md`) reaches MCP `note_list`/`note_search`/`note_read` JSON and `note_stats` `skipped[].path` unmasked (src/core/store.rs:824 `inbox_ids`; src/surfaces/mcp.rs:137; src/core/query.rs:187, :300); CLI is safe via `safe()`. Fix next session: mask ids and skipped paths at the MCP/query output boundary, plus an MCP test; this blocks merge; real run — pending
Memory: nothing new (task in progress; the memory pass runs after merge)
Not done: security-review finding fix (session 2 running, 2026-11-16), Evidence rows, auto verify, merge (admin: after the auto verify pass)

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
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☐ N/A | [screenshot path or LLM verdict — required for UI tasks, Hard-Stop Gate 6] |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ pass / ☐ fail / ☐ N/A | [method used + output] |
| **UI: Responsiveness at target viewports** | ☐ pass / ☐ fail / ☐ N/A | [viewports tested, any overflow findings] |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured before any implementation commit, on `main` @ 5b05e1a code, temp `HOME` and `EASY_NOTE_DIR` holding `inbox/20261114-183002.txt` with text `phone idea #mobile`, after `cargo build`:

```
Mon Nov 16 10:15:02 AM UTC 2026   (date -u)
$ target/debug/easy-note ls
exit=0                            (no output: the inbox note is not listed)
$ target/debug/easy-note show inbox/20261114-183002
easy-note: not a note id: inbox/20261114-183002
exit=2
```

**AFTER**: same fixture and commands after the change (`cargo build`, temp `HOME` and `EASY_NOTE_DIR`):

```
Mon Nov 16 10:23:51 AM UTC 2026   (date -u)
$ target/debug/easy-note ls
inbox/20261114-183002  phone idea #mobile  #mobile
exit=0
$ target/debug/easy-note show inbox/20261114-183002
---
easy-note: 1
id: "inbox/20261114-183002"
created: "2026-11-14T18:30:02+07:00"
tags: ["mobile"]
source: "inbox"
---
phone idea #mobile
exit=0
inbox file sha256 before and after both: 1771b6cff0b5654a3e9fe23e60acc88e8ea706360b55b8f45f0a9ed28eeea2e1
```

**DELTA**: A note a phone sync app saved in `inbox/` now shows up in `ls`, `search`, `stats`, `show` and the MCP read tools, and easy-note never rewrites the file.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-261.jsonl`, never the
implementing agent alone]
