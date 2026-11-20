# Minimal-track Decision records

> One Decision record per minimal-track task (`docs/claude-md/tracks.md`), **newest first**: insert each
> new record directly under this note.

## HUN-283 — Docs out of date after the dashboard window: CLI help and PRD non-goal · minimal
**Asked:** "start all the tasks as parallel" (HUN-283: Linear issue from the HUN-278 code review, 2 P2 docs findings)
**Decisions:**
- Track minimal — C0, Risk Low, docs and one help string, no UI, no user data.
- The `dashboard` help names the own window first and Obsidian as the fallback — matches `src/surfaces/dashboard.rs` `run()`: window when `use_window`, else the Obsidian plan.
- PRD non-goal narrowed to "a web UI served on a local port" — DDR 0009 brought the own window in scope (FR-042); a port-served page stays out (BRAINSTORMING_LOG_dashboard-window.md, option C).
- README needed no change — its "Dashboard window" section already says the window opens when a desktop is running, else Obsidian.
**Checked:**
- New test `dashboard_help_names_the_own_window_and_the_obsidian_fallback` (tests/cli.rs) — failed before the change (`dashboard    Open the notes dashboard (`easy-note.base`) in Obsidian. Run `install obsidian` first`), passes after.
- Suite: `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` — EXIT=0, 24 `test result: ok`, 536 passed, 0 failed.
- Real run: `./target/debug/easy-note --help` — printed `dashboard    Open the dashboard: easy-note's own window in a `window` build on a desktop, else `easy-note.base` in Obsidian (run `install obsidian` first)`.
- Simplify: skipped: under 20 lines outside tests and docs.
**Files:** `src/surfaces/cli.rs`, `PRD.md`, `tests/cli.rs`, `tasks/MINIMAL_LOG.md`
**Not done:** merge (the user decides)

## HUN-264 — README, RUNBOOK, client config snippets, install guide · minimal
Mode: auto
**Asked:** "continue [admin]" (session tag loop; HUN-262 paused for owner approval, so the next ready ticket at P1)
**Decisions:**
1. Track minimal — C0, Risk Low, docs plus one test, no UI, no user data — RULE: `docs/claude-md/tracks.md` `## Tracks` (minimal unless C2+, Risk Medium+, a UI component, or more than one agent)
2. Install is `cargo install --path . --locked` plus an `n` symlink, not `uv tool install` (the guide's Python-era AC 1) — FACT: docs/ddr/0005-rust-single-binary.md:30 "3. Install: `cargo install --path .` (from a checkout)"
3. The README lists the 5 current MCP tools only; the 2 enrich tools wait for HUN-262 and the owner's D37 approval — SAFER OPTION: docs must match the binary ⚠ decided for you — please review
4. The command check scans only code spans and fenced blocks, so prose like "easy-note never writes" is not read as a command — SAFER OPTION: no false failures, every shown command still checked
5. Kept the HUN-260 "One key press" section unchanged under the new sections — SAFER OPTION: smallest change
6. Cursor's snippet is not run: `cursor-agent` is not signed in and sign-in writes `~/.cursor` — RULE: tasks/TASK_GUIDE_HUN-264.md § Files Must NOT Touch (the real `~/.cursor/` is off limits); owner run left open
**Checked:**
- New test `tests/readme.rs` (`every_command_the_readme_names_is_a_real_subcommand`, `the_readme_covers_install_first_note_clients_and_boundaries`, `the_word_finder_skips_names_that_are_not_commands`) — red before the README change: `test result: FAILED. 1 passed; 2 failed` (`README is missing "cargo install --path ."`); passes after: `test result: ok. 3 passed; 0 failed`. Mutation: a `easy-note bogus` span made it fail with `README names commands that do not exist: ["bogus"]`.
- Suite: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` — 379 passed, 0 failed
- Real run (temp HOME, 2026-11-16 15:32 UTC): `claude mcp add easy-note -- easy-note serve` printed `Added stdio MCP server easy-note with command: easy-note serve to local config`; `claude mcp list` printed `easy-note: easy-note serve - ✔ Connected`; Codex with the README `config.toml`: `codex mcp list` printed `easy-note  easy-note  serve  -    -    enabled`; first note `easy-note add retry bug only on Firefox #flaky` printed `✓ noted 20261116-223537-a64a`, `easy-note search firefox` found it
- Simplify: skipped: under 20 lines (0 changed lines outside tests and docs)
**Memory:** nothing new
**Files:** `README.md`, `RUNBOOK.md`, `tests/readme.rs`, `tasks/MINIMAL_LOG.md`
**Not done:** merge (admin: after the auto verify pass); auto verify: running (T182); Cursor snippet run (yours: needs `cursor-agent` sign-in); README enrich tools after HUN-262

## HUN-267 — Align context task pattern with PRD FR-005 · minimal
**Asked:** "you can start automate" (after the Supervisor offered the HUN-254 ticket-pattern fix as the next small item)
**Decisions:**
- Track minimal — one regex in one file, Risk Low: no user data or storage format changes; only adds a `task` where PRD FR-005 says one belongs.
- Use the PRD pattern verbatim, `[A-Z][A-Z0-9]+-\d+`, keeping the existing no-letter-or-digit boundaries — FR-005 declares it; the boundaries are HUN-254's tested `art1234` guard.
- Tracker keys stay case-sensitive (`feat/hun-221` gives no task) — FR-005 says only `T` is case-insensitive.
- Did not touch DDR 0002's "draft" wording — DDRs are owner-approved and its Status is still "Proposed (owner to accept)"; left for the owner.
**Checked:**
- New test `test_task_from_branch[feat/AB2-12-x-AB2-12]`, `[fix/PROJ-1234567-PROJ-1234567]`, `[feat/A-12-None]` — the first two failed before the change (`2 failed, 12 passed`), all pass after
- Suite: `.venv/bin/pytest -q tests` — `205 passed in 11.09s`; `uvx ruff check --isolated` on both files — `All checks passed!`
- Real run: `context.capture(cwd=<temp repo on feat/AB2-12-demo>)` then `store.add` — printed `{'repo': 'rr', 'branch': 'feat/AB2-12-demo', 'task': 'AB2-12'}` and a note file with `task: "AB2-12"`
- Simplify: skipped: under 20 lines
**Files:** `src/easy_note/core/context.py`, `tests/test_context.py`, `tasks/MINIMAL_LOG.md`
**Not done:** merge (the user decides)
