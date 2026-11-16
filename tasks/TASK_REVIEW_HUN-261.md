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
6. Fix the verify fail (restore changed `created` of an inbox note with no time in its name) with option B of the Paused line: the trash name carries the file's original mtime, `inbox__<unix seconds>__<file name>`, and `restore` sets that mtime back; also mask the id in the `rm`/`restore` success lines — RULE: tasks/TASK_REVIEW_HUN-261.md § Auto decisions (the Paused line; the user replied `continue [admin] [auto]` to the Supervisor's recommended option, 2026-11-16)
Checks: tests `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` at 8f20713 (2026-11-16) — exit 0, 302 passed (inbox 19 incl. restore-mtime, rm/restore masking and bad-trash-name tests; mcp 20); code-review (session 2) of 8199f7b..d24bead — no P0/P1; and of 8f20713 — no code findings; P1 process: the 8f20713 agent's `Startup reads` line says PROJECT_SPEC.md not read in full and agents/backend.md not re-read (recorded, no code change possible); P2: tests/mcp.rs:578 assertion allows a note_read error that never happens; P3: lane EXCEEDS (mostly tests); security-review (session 2, main...HEAD at 57ee9e0) — no findings ≥ 8/10, session-1 MCP id leak FIXED by d24bead; 8f20713 parses trash names strictly (`split_trash_inbox`: `inbox__<digits>__<name>`, no `/`, no NUL); auto verify at 9058f3b ☑ fail (restore changed `created` of an undated inbox note) — fixed by 8f20713, red runs: `left: SystemTime { tv_sec: 1794826947 … } right: SystemTime { tv_sec: 1790000000 … }` and `✓ removed inbox/ghp_…` unmasked; real run (auto-checked, not /verify)
Paused: 4th Debate cap (docs/claude-md/tracks.md ### Stops that stay) — auto verify ☑ fail at 9058f3b: `rm` then `restore` of an inbox note with no time in its file name changes `created` to today, because `rm` sets the trash file's mtime to now (src/core/store.rs:350) and `restore` never sets it back; the fix needs a choice and this task already used 3 Debates; options: A keep the mtime on `rm` for inbox files and age inbox trash entries by another clock / B put the original mtime in the trash name (`inbox__<mtime>__<file name>`) and set it back on `restore` / C a sidecar file with the original mtime — answered: option B, decision 6
Memory: memory/learnings.md — 2026-11-16 board pull from a worktree rewrites that worktree's guides and board cache (HUN-261)
Not done: merge (admin: after the auto verify pass); auto verify: running (T182); html-report pages for code-review/security-review not generated; follow-up: front-matter `source` in an inbox file wins over `source: inbox` (docs/NOTE_FORMAT.md:55 vs :100); readiness incomplete: `Codebase map` file `none`, `Approved by` empty (yours)
Closed: row moved to Done 2026-11-16 — auto verify pass (session 95a59b99-9598-46f5-b1e1-db37fdc49937, commit 57ee9e01bce02eb7445907a2c8382c1e1c18fe5f); Gates 5 and 6 checked

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/inbox.rs` (16 tests, AC 1-7, 9, 10), `tests/leak.rs` inbox surface (AC 8), `tests/mcp.rs::inbox_file_names_with_a_secret_are_masked_in_every_read_tool` (security fix). Run 2026-11-16 at 8a6dfa9: `test result: ok. 16 passed` (inbox), `ok. 46 passed` (leak), `ok. 20 passed` (mcp) |
| Verification command run | ☑ pass | `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` at 8f20713: fmt and clippy clean; lib 20, cli 43, context 29, format 13, guard 37, inbox 19, leak 46, mcp 20, query 45, store 30 passed, 0 failed |
| Negative cases hold | ☑ pass | Security fix red run on unfixed code (before d24bead): `test inbox_file_names_with_a_secret_are_masked_in_every_read_tool ... FAILED` / `note_list leaked a token: {"items":[{"id":"inbox/ghp_EXAMPLE…` / `test result: FAILED. 0 passed; 1 failed`. Inbox feature red: Demonstration BEFORE (`show inbox/…` exit 2 `not a note id`). AC 9/10 refusal tests in tests/inbox.rs |
| verify | ☑ pass | auto-run by hook, session 95a59b99-9598-46f5-b1e1-db37fdc49937, commit 57ee9e01bce02eb7445907a2c8382c1e1c18fe5f, log .claude/verify-runs/HUN-261-57ee9e01bce0.log — pass |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | code-review: session 1 on 679e8da (src/core/store.rs, src/surfaces/cli/write.rs, tests); session 2 re-review of 8199f7b..HEAD (src/surfaces/mcp.rs, tests/mcp.rs, docs/NOTE_FORMAT.md) plus callers in src/core/query.rs. security-review: main...HEAD, src + tests |
| Full smoke suite still green (no regression) | ☑ pass | full `cargo test` at 8f20713, 302 passed, 0 failed |
| **Companion score (easy-verifier)** | ☑ N/A | not run this session: easy-verifier `score` skill not invoked for HUN-261 (optional tool) |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | pure backend / CLI task, no UI (Hard-Stop Gate 6 N/A) |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | pure backend / CLI task, no UI (Hard-Stop Gate 6 N/A) |
| **UI: Responsiveness at target viewports** | ☑ N/A | pure backend / CLI task, no UI (Hard-Stop Gate 6 N/A) |

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

**WITNESS**: Supervisor (session 1) ran BEFORE at 10:15:02 UTC and AFTER at 10:23:51 UTC on 2026-11-16, recorded in `memory/event-trace/HUN-261.jsonl`; Supervisor (session 2) reran the full suite at 8a6dfa9 on 2026-11-16 (299 passed).