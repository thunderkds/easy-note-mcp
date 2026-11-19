# TASK_REVIEW — T[NNN]: [Short Title]

> Sibling of `tasks/TASK_GUIDE_T[NNN].md`. Everything here is **filled by the reviewer at Stage
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

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | New: `tests/window_api.rs` (13 tests: AC 3–7 board/detail/counts/top 3/empty dir/inbox/bad requests incl. `../../etc/passwd`/planted-secret leak; AC 2 `use_window` truth table; AC 9 CSP, asset table, navigation guard, reply quoting, asset scan for DOM sinks and remote URLs; IPC `req` echo); `src/surfaces/window/mod.rs` `shell::tests` (`--features window`: AC 11 `dmabuf_is_set_only_when_unset`, AC 9 `every_response_carries_the_csp`); `tests/perf.rs` `perf_window_board` (AC 12, ignored by default). `cargo test --locked --features window`: `Running tests/window_api.rs` … `test result: ok. 13 passed; 0 failed; 0 ignored` · `test surfaces::window::shell::tests::dmabuf_is_set_only_when_unset ... ok` · `test surfaces::window::shell::tests::every_response_carries_the_csp ... ok`. AC 12, `cargo test --release --locked --test perf perf_window_board -- --ignored --nocapture`: `perf window_board: n=30 first 140.7 ms p50 120.5 ms p95 123.9 ms max 124.0 ms` / `test result: ok. 1 passed` (budget 300 ms, 5,000 notes). AC 8 and 10 (the page) are not covered here: the placeholder page is replaced by the Frontend slice (feat/HUN-278-page). |
| Verification command run | ☑ pass | Full guide command (fmt, clippy + test in default / `tray` / `window`, `! cargo tree … (wry\|tao)`) run in the worktree, exit `EXIT=0`; 72 `test result: ok` lines, no `FAILED`, no clippy warning. Tail: `Running tests/window_api.rs (target/debug/deps/window_api-6b2c8de9b77d2b51)` … `test no_planted_secret_leaves_through_board_or_detail ... ok` / `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s` / `Doc-tests easy_note_mcp` / `test result: ok. 0 passed; 0 failed` / `EXIT=0`. Default build: `tests/leak.rs` `test result: ok. 46 passed`; `tests/dashboard.rs` `test result: ok. 5 passed` (unchanged pin test); `tests/enrich.rs` `test result: ok. 24 passed` in all three runs (the known timing test did not fail this time). |
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

**BEFORE** (captured 2026-11-19 by the Backend-Implementer before any implementation commit, worktree at `4b5c38a`, display present):

```
Thu Nov 19 11:05:54 AM UTC 2026
$ cargo build --locked --features window 2>&1 | tail -3
error: the package 'easy-note-mcp' does not contain this feature: window

Thu Nov 19 11:07:05 AM UTC 2026
$ EASY_NOTE_DIR=$(mktemp -d) HOME=<tmp> XDG_CONFIG_HOME=<tmp> ./target/debug/easy-note dashboard; echo exit=$?
easy-note: no dashboard yet: run easy-note install obsidian
exit=1
DISPLAY=:0 WAYLAND_DISPLAY=wayland-0
```

(`cargo run` with a temp HOME cannot find the rustup toolchain, so the second command runs the built binary.)

**AFTER**: [same command, post-change] OR [verbatim excerpt of the new content]

**DELTA**: [one sentence — what a user can now do that they could not before]

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-278.jsonl`, never the
implementing agent alone]
