# Minimal-track Decision records

> One Decision record per minimal-track task (`docs/claude-md/tracks.md`), **newest first**: insert each
> new record directly under this note.

## HUN-281 — Dashboard window: an unusable display makes tao panic instead of falling back to Obsidian · minimal
**Asked:** "start all the tasks as parallel" (HUN-281: Linear issue from the HUN-278 code review, P2 reliability)
**Decisions:**
- Track minimal — C1, Risk Low, Rust only, no UI, no user data.
- Catch the one tao panic at the event-loop build with `std::panic::catch_unwind`, with a silent panic hook for that call only — tao 0.37.3 has no fallible builder: `platform_impl/linux/event_loop.rs:219` `.expect("Failed to initialize gtk backend!")`; the panic is raised in Rust after `gtk::init()` returned `Err`, so no FFI frame is unwound.
- `window::open` returns the new `window::NO_DISPLAY` error; `dashboard::run` then takes the FR-034 Obsidian path, so the person still gets a dashboard (or the install hint) instead of a crash.
- Assumed: a display that GTK cannot open is the only case to catch — any other window error still returns as before.
**Checked:**
- New test `hun281_unusable_display_falls_back_to_obsidian_without_a_panic` (tests/dashboard.rs, `window` build, `DISPLAY=:99317`) — failed before the change (`thread 'main' panicked at .../tao-0.37.3/src/platform_impl/linux/event_loop.rs:219:53: Failed to initialize gtk backend!`), passes after.
- Suite: `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`, then clippy + test with `--features tray` and `--features window` — EXIT=0, 72 `test result: ok`, 1613 passed, 0 failed (the first run died on `No space left on device`; rerun after freeing disk).
- Real run: `env -i PATH=/usr/bin:/bin HOME=<tmp> EASY_NOTE_DIR=<tmp>/notes DISPLAY=:99317 ./target/debug/easy-note dashboard` — printed `easy-note: no dashboard yet: run easy-note install obsidian`, exit=1, no panic.
- Simplify: skipped: under 20 lines outside tests and docs.
- verify: ☑ pass — auto-run by hook, session 32c2c9f3-8854-4ab9-8d4f-01fbd9738954, commit bef099a958473b511e75dbcd9e27a08593445ab2, log .claude/verify-runs/HUN-281-bef099a95847.log
**Files:** `src/surfaces/window/mod.rs`, `src/surfaces/dashboard.rs`, `tests/dashboard.rs`, `tasks/MINIMAL_LOG.md`
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
