# TASK_REVIEW — HUN-258: Rust MCP surface (5 tools, stdio + HTTP)

> Sibling of `tasks/TASK_GUIDE_HUN-258.md`. Everything here is **filled by the reviewer at Stage
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
1. Picked HUN-258 as the ticket to run beside HUN-257: the only Todo ticket whose `Depends on` tasks are all Done and that no other session claims — RULE: `docs/claude-md/tracks.md` `### Session tags` (the loop, step 1)
2. Re-planned the guide for Rust and signed the Requirement Fidelity Gate as Supervisor — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 13)
3. Spawned the Backend-Implementer on `sonnet` — FACT: PROJECT_SPEC.md:168 "haiku | sonnet | sonnet or opus | opus"
4. Restored the board-cache files the spawn pull rewrote in the worktree (`PROJECT_KANBAN.md`, `tasks/TASK_GUIDE_HUN-258.md`, `-268` to `-272`), so the branch holds only HUN-258 work — RULE: `tasks/TASK_GUIDE_HUN-258.md:159` "`PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md` | Supervisor-owned"
5. Kept the new `axum` dependency (default features off, `tokio` + `http1`) for the HTTP server; rmcp 3.10 ships no HTTP server, so either option adds a crate, and the Simplicity First rule (AGENTS.md § Karpathy Engineering Principles) picks the 2-line version — DEBATED: keep axum (Advocate 1: keep axum — 2 lines vs 15-20 for a hand-written loop, rmcp's own dev-dependency uses the same axum config; Advocate 2: replace with hyper + hyper-util — saves 6 lock packages, adds a 2nd direct dependency and ~20 lines of accept-loop and routing code, called it a coin flip) ⚠ decided for you — please review
6. Did not add auth to `serve --http`; the security-review candidate (other local OS users can call 127.0.0.1:8769) was filtered at 3/10 as accepted by design — RULE: `tasks/TASK_GUIDE_HUN-258.md:127` "**Cut list**: Resources and resource templates; Auth on HTTP (localhost-only bind instead)". Follow-up: one README/RUNBOOK line in HUN-264 ("`--http` is reachable by every local account; use stdio on shared machines")
7. Left the two P3 code-review findings as suggestions (cli.rs `run()` no longer dispatches through `starts_mcp`; lane check EXCEEDS on line count, mostly tests and Cargo.lock) — RULE: `skills/code-review/SKILL.md:148` "Present P2/P3 fixes as suggestions only."
8. Companion score row set to N/A — RULE: `skills/score/SKILL.md:28` "**Neither**: write `☑ N/A`, naming the missing tool and the rung reached"
9. HUN-257 merged first (main 31fd91b), so this branch resolved the conflict: `Serve` keeps `--http`/`--host`/`--port` and now dispatches through HUN-257's exhaustive `starts_mcp`; serve error text goes through `safe()`; `SURFACES` = core, cli, mcp — RULE: `tasks/TASK_GUIDE_HUN-258.md:36` "the second branch to merge resolves the conflict"
10. The rebase stalled because the merge gate blocks `git rebase --continue`/`--abort` while the worktree is detached; the resolved commit was put on a new branch `feat/HUN-258-rust-mcp-2` (worktree `.claude/worktrees/hun-258-mcp-2`); the old branch `feat/HUN-258-rust-mcp` (2084229) and its stuck worktree are kept untouched for the owner — RULE: `docs/claude-md/tracks.md` `### Stops that stay` (no delete or overwrite without a backup), and the verify row was reset to unwritten so a fresh auto verify judges the new code — RULE: `docs/claude-md/tracks.md` `### Admin tag` (One retry after blocked, step 1: the unwritten row of the template)
Checks: tests `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` — exit 0, 233 passed before the rebase and 282 passed after it (on main 31fd91b with HUN-257), 0 failed; reviews code-review — P0 0 / P1 0 / P2 0 / P3 2, security-review — 0 findings at confidence ≥ 8 (1 filtered); real run `easy-note serve` over stdio with `initialize` + `tools/list`, and `serve --http --host 0.0.0.0` — printed the 5 tool names, and `--host "0.0.0.0" is refused: HTTP mode binds 127.0.0.1 only` exit 2 (auto-checked, not /verify)
Memory: memory/learnings.md — 2026-11-16 rmcp argument errors echo escaped values; spawn pull rewrites the worktree's board cache (HUN-258); memory/MEMORY.md — index line for it
Not done: merge (admin: after the auto verify pass); auto verify: running (T182) on the rebased branch (the first run passed at 4c696af, session 7e683245-e69c-48b1-91bf-cc802a22afc9, on the pre-rebase code); readiness incomplete: `Codebase map` file `none` does not exist, `Approved by` empty (yours)

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/mcp.rs` (new, 19 tests: AC 1 `exactly_the_five_tools_are_registered`; AC 2 `read_tools_leave_the_notes_dir_byte_identical`, `note_read_masks_a_hand_edited_token_and_leaves_the_file_alone`; AC 3 `note_add_writes_one_file_with_source_mcp_and_the_client_name`; AC 4 `cwd_arg_wins_over_roots`, `roots_are_used_when_there_is_no_cwd_arg`, `no_roots_support_or_no_roots_falls_back_to_the_process_cwd`; AC 5 `list_and_search_hold_at_most_50_items_with_short_snippets`; AC 6 `note_add_description_carries_the_consent_and_note_prefix_rules`; AC 7 `http_mode_binds_loopback_and_answers_initialize`, `http_host_other_than_127_0_0_1_exits_2_before_binding`, `http_port_in_use_exits_1_with_a_clear_message`; AC 8 `smoke_real_binary_over_stdio_calls_every_tool`; AC 9 `tool_error_text_never_echoes_a_secret`, `protocol_error_text_never_echoes_a_secret`), `tests/leak.rs` (`surface_mcp` in `SURFACES`, `planted_secret_never_leaks`), `src/surfaces/mcp.rs` unit tests (3). Supervisor re-run 2026-11-16 at 21af165: `test result: ok. 19 passed; 0 failed` (tests/mcp.rs), `test result: ok. 46 passed; 0 failed` (tests/leak.rs) |
| Verification command run | ☑ pass | Supervisor run at 21af165: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` exit 0. Per suite: lib 9, main 0, cli 5, context 29, format 13, guard 37, leak 46, mcp 19, query 45, store 30 passed; 0 failed. No clippy warnings |
| Negative cases hold | ☑ pass | Supervisor mutation 2026-11-16: `check_host` changed to accept any IPv4 (`git diff --stat`: ` src/surfaces/mcp.rs \| 2 +-`, 1 file changed) → `test http_host_other_than_127_0_0_1_exits_2_before_binding ... FAILED` and `test surfaces::mcp::tests::only_127_0_0_1_passes_the_host_check ... FAILED`; restored with `git checkout -- src/surfaces/mcp.rs`, `test result: ok. 1 passed`. Agent mutation (reported): `note_list` writing a file made the no-write test fail |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | code-review: `src/surfaces/mcp.rs`, `src/surfaces/cli.rs`, `Cargo.toml`, `tests/mcp.rs`, `tests/leak.rs`, plus callees read in `src/core/query.rs` (MAX_LIMIT clamp, :290), `src/core/context.rs` (`capture`, roots), `src/core/store.rs` (`is_note_id`, mask on read). Skipped: rest of `src/core/` (unchanged). Result: P0 0, P1 0, P2 0, P3 2 (cli.rs `run()` dispatches on `Command::Serve` directly so `starts_mcp` is test-only — merge note for HUN-257; lane_check `EXCEEDS (lines 1414 > 250)`, mostly tests and Cargo.lock). security-review: 1 candidate (no auth on `--http` for other local OS users), filtered by the false-positive pass at 3/10 as accepted by design (guide cut list `tasks/TASK_GUIDE_HUN-258.md:127`, PRD NFR-001); docs follow-up for HUN-264. Rest of the HTTP surface (rmcp 3.10 Host check, Origin allow-list, JSON content-type, OPTIONS 405), id traversal, git args checked safe |
| Full smoke suite still green (no regression) | ☑ pass | full `cargo test` above: 233 passed, 0 failed; pinned `tests/cli.rs` `serve_answers_initialize_then_exits_on_eof` passes |
| **Companion score (easy-verifier)** | ☑ N/A | MCP not registered in this session; `easy_verifier` not importable (`ModuleNotFoundError: No module named 'easy_verifier'`); stopped at rung 3 |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | pure backend / CLI task, no UI component |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | pure backend / CLI task, no UI component |
| **UI: Responsiveness at target viewports** | ☑ N/A | pure backend / CLI task, no UI component |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured 2026-11-16T07:33:42Z on branch `feat/HUN-258-rust-mcp` at bd1e627 (no implementation commit yet), `cargo build`, then `initialize` + `tools/list` sent to `easy-note serve` over stdio (temp HOME and EASY_NOTE_DIR), then `easy-note serve --http`:

```
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-03-26","capabilities":{},"serverInfo":{"name":"easy-note","version":"0.1.0"}}}
{"jsonrpc":"2.0","id":2,"result":{"tools":[]}}
exit=0
---
error: unexpected argument '--http' found

Usage: easy-note serve

For more information, try '--help'.
exit=2
```

**AFTER**: captured 2026-11-16T07:58:30Z at commit 47d2368, same commands (the tools/list reply is shown as its sorted tool names):

```
initialize -> serverInfo {'name': 'easy-note', 'version': '0.1.0'}
tools/list -> ['note_add', 'note_list', 'note_read', 'note_search', 'note_stats']
---
easy-note: --host "0.0.0.0" is refused: HTTP mode binds 127.0.0.1 only
exit=2
```

**DELTA**: An MCP client can now add, search, list, read and count the person's notes through `easy-note serve` (stdio, or HTTP on 127.0.0.1 only), and no tool can edit, delete or move a note.

**WITNESS**: `memory/event-trace/HUN-258.jsonl` (main checkout) records 81 calls attributed to HUN-258, from the spawn at 2026-11-16T07:31:19Z through the Supervisor's Stage 4 checks. The Supervisor (not the implementing agent) re-ran the full verification command at 21af165, ran the host-check mutation, and re-captured the test results pasted in Evidence.
