# TASK_REVIEW — HUN-275: AI detail from the local claude CLI: ai enrich + [ai] settings

> Sibling of `tasks/TASK_GUIDE_HUN-275.md`. Filled by the reviewer at Stage 4/5.

---

## Auto decisions

Mode: auto
1. Dashboard = an Obsidian Bases file, no window of our own; zenity kept for `quick` — RULE: `docs/ddr/0006-obsidian-as-the-dashboard.md` § Decision (owner's chat answers, 2026-11-19)
2. Task breakdown: 4 tasks (HUN-273 dashboard, HUN-274 AI views, HUN-275 claude enrich, HUN-276 icons+launcher+tray), run in parallel worktrees as the owner asked — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 12)
3. Spawn model opus for C2 at Risk High — RULE: `skills/craft-spawn-prompt/SKILL.md` § 5. Recommend spawn model ("A two-model cell (`sonnet or opus`) isn't ambiguity — pick per Risk")
4. `claude -p` receives the masked note text only; `after_save` kept, default false — DEBATED: text only + after_save default off (Advocate 1: text plus repo/branch/task/created; Advocate 2: text only, after_save default off; Advocate 3: text only, no after_save) — settled by RULE: `PRD.md` § Non-Functional Requirements NFR-001 and § Functional Requirements FR-038 ⚠ decided for you — please review
5. The save paths read config.toml once through `config::notes_dir_and_after_save()`, also when `EASY_NOTE_DIR` is set (today 0 reads) — RULE: `tasks/TASK_GUIDE_HUN-275.md` § Acceptance Criteria AC 8 ("the save path does not read more than today (config read once, as it is now)") and FR-038 (after_save must be known at save)
Checks: tests `cargo fmt --check && cargo clippy … -D warnings && cargo test --locked` + `--features tray` at 6b6965a — exit 0, 42 `test result: ok`, 0 failed; 4 mutations caught; reviews code-review — 0 P0/P1, 1 P2 (stale PROJECT_SPEC wording, fixed on main 3a24145), 1 P3 (`</note>` in a note closes the prompt delimiter; child has no tools); security-review — no findings ≥ 8 (P3: Bedrock/Vertex env still reaches the child; billing, not security); real run one `claude -p --model haiku` from a temp dir — exit 0, `structured_output` + `modelUsage` keys (auto-checked, not /verify)
Memory: nothing new for HUN-275 alone; the round-4 memory pass runs after the last merge
Not done: merge (admin: after the auto verify pass); auto verify: ☑ pass (session 47b2d8b2-f695-492a-8a41-b0ab5117844a, commit 73d6fa671a68ad6f696004f8cc3aefaf7bd5f7e1, log .claude/verify-runs/HUN-275-73d6fa671a68.log); owner run `easy-note ai enrich --pending --limit 3` with the account dir (yours); html-report for code-review and security-review (not rendered this run)
Closed: row moved to Done 2026-11-19 — auto verify pass (session 47b2d8b2-f695-492a-8a41-b0ab5117844a, commit 73d6fa671a68ad6f696004f8cc3aefaf7bd5f7e1); Gates 5 and 6 checked

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass / ☐ fail | New: `tests/enrich.rs` (24 tests, fake `claude` on a temp PATH: AC 1 settings/defaults/bad keys/`init` keeps `[ai]`; AC 2 off; AC 3–5 exact argv, env, stdin, temp cwd, AI file + model id; AC 5–6 garbage, exit 1 + login hint, `is_error`, wrong shape, over cap, bad kind, timeout kill+reap; AC 7 ids, unknown id, `--pending --limit`, usage errors; AC 8 after_save via `add`, `hook-prompt`, `quick`, MCP `note_add` (real `serve` child), off = no spawn, bad `[ai]` never fails a save, no-HOME save as today; AC 9 doctor line). `src/surfaces/enrich.rs` unit tests (7: argv, child_env, schema, parse real shape / result fallback / odd model id, fixed failure reasons, doctor text, resolve, temp cwd 0700 + removed). `tests/leak.rs` new surface `enrich` (AC 10–11: fake echoes the planted text in every answer field, raw, `is_error`, stderr; planted text as an id). Run (commit `6b6965a`): `Running tests/enrich.rs` → `test result: ok. 24 passed; 0 failed` (both feature sets); leak → `test planted_secret_never_leaks ... ok`, `the_registry_holds_the_core_the_cli_and_the_inbox_surface ... ok` |
| Verification command run | ☑ pass / ☐ fail | Guide command on clean tree at `6b6965a`, 05:29:48–05:35:57 UTC (backend agent): fmt --check ok; clippy -D warnings `Finished` (default and `--features tray`); 42 `test result: ok` lines, none failed; tail: `Running tests/tray.rs … test result: ok. 2 passed; 0 failed` / `test result: ok. 0 passed; 0 failed` (doc-tests) / `VERIFY EXIT 0` |
| Negative cases hold | ☑ pass / ☐ fail | Red first: `tests/enrich.rs` did not compile before the change (`error[E0425]: cannot find function \`ai_settings\` in module \`config\``); the no-HOME test was red before its fix (`assertion failed: easy-note: HOME is not set`). Mutations on `6b6965a`, each restored after: M1 `src/surfaces/enrich.rs \| 2 +-` (API key kept) → `one_note_is_enriched_with_the_exact_argv_env_and_stdin` FAILED (`ANTHROPIC_API_KEY=x-api`), unit `child_env_drops_the_api_env_and_sets_the_account_dir` FAILED; M2 `src/surfaces/enrich.rs \| 3 ++-` (failure quotes raw output) → leak `planted_secret_never_leaks` FAILED (`"enrich x password_assign:correct hors: secret reached stderr"`); M3 `src/surfaces/hook.rs 1 insertion(+), 3 deletions(-)` (hook skips after_save) → `after_save_enriches_a_note_saved_by_the_hook` FAILED; M4 `src/core/config.rs 1 insertion(+), 1 deletion(-)` (save_dir writes whole file) → `init_keeps_an_existing_ai_section` FAILED |
| verify | ☑ pass | auto-run by hook, session 47b2d8b2-f695-492a-8a41-b0ab5117844a, commit 73d6fa671a68ad6f696004f8cc3aefaf7bd5f7e1, log .claude/verify-runs/HUN-275-73d6fa671a68.log — pass |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed: `src/surfaces/enrich.rs` (argv, child_env, call, parse_output, enrich_one, after_save), `src/core/config.rs` `[ai]` + `save_dir` + `notes_dir_and_after_save`, the 4 save-path call sites (`write.rs` add, `quick.rs`, `hook.rs`, `mcp.rs` note_add), `read.rs` doctor line, `cli.rs` arm; writer `src/core/ai.rs::enrich` read for the untrusted-answer path. Skipped: other core modules (untouched). Lane: `C2, 13 files, 1673 lines, EXCEEDS (lines 1673 > 500); unpredicted: src/core/ai.rs` (one doc line) — warn only |
| Full smoke suite still green (no regression) | ☑ pass / ☐ fail | Full `cargo test --locked` and `--features tray` green in the run above (pin tests unchanged: `tests/cli.rs` help/doctor, `tests/readme.rs` 3 passed with the new README section, `tests/perf.rs`, `tests/hook.rs`) |
| **Companion score (easy-verifier)** | ☑ N/A | not run in this session: the `score` skill was not started for the round-4 tasks (time); the merge gate does not block on N/A |
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☑ N/A | CLI only, no UI (guide § UI / Design Acceptance Criteria) |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ pass / ☐ fail / ☑ N/A | CLI only, no UI |
| **UI: Responsiveness at target viewports** | ☐ pass / ☐ fail / ☑ N/A | CLI only, no UI |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured on branch `feat/HUN-275-claude-enrich` at `2be87be`, before any implementation commit, with a temp `HOME` and `EASY_NOTE_DIR` (`$S` = the agent's scratchpad; `RUSTUP_HOME`/`CARGO_HOME` pinned so cargo runs under the temp `HOME`):

```
$ date -u
Thu Nov 19 05:11:56 UTC 2026
$ HOME=$S/before/home EASY_NOTE_DIR=$S/before/notes cargo run -q -- ai enrich --pending; echo "exit $?"
error: unrecognized subcommand 'enrich'

Usage: easy-note ai <COMMAND>

For more information, try '--help'.
exit 2
$ HOME=$S/before/home EASY_NOTE_DIR=$S/before/notes cargo run -q -- doctor; echo "exit $?"
notes dir: /tmp/claude-1000/-home-thunderkds-workspace-pets-hungnguyen111-easy-note-mcp/787574a0-3c07-428c-b66c-57bbc928438d/scratchpad/before/notes
0 notes checked
no problems found
exit 0
```

**AFTER**: same commands on `6b6965a`, temp `HOME` and `EASY_NOTE_DIR` (`$S` = scratchpad). First with no config (enrich off), then with `[ai] enrich = "claude"` and a stand-in `claude` script on `PATH` that prints the real output shape (not the real CLI):

```
$ date -u
Thu Nov 19 07:14:45 UTC 2026
$ cargo run -q -- ai enrich --pending; echo "exit $?"
easy-note: AI enrich is off: set [ai] enrich = "claude" in $S/after/home/.config/easy-note/config.toml
exit 1
$ cargo run -q -- doctor; echo "exit $?"
notes dir: $S/after/notes
0 notes checked
no problems found
ai: off
exit 0

$ date -u
Thu Nov 19 07:15:35 UTC 2026
$ cargo run -q -- add buy a usb-c hub
✓ noted 20261119-141535-659a · easy-note-mcp · HUN-275
$ cargo run -q -- ai enrich --pending; echo "exit $?"
✓ enriched 20261119-141535-659a
1 enriched, 0 failed
exit 0
$ cargo run -q -- doctor; echo "exit $?"
notes dir: $S/after/notes
1 note checked
no problems found
ai: claude via claude ($S/after/bin/claude), account dir default, model haiku, after_save false
exit 0
$ cat $S/after/notes/.easy-note/ai/*.json
{"easy-note-ai":1,"id":"20261119-141535-659a","title":"Buy a USB-C hub","kind":"todo","summary":"A reminder to buy a USB-C hub.","tags_suggested":["hardware","shopping"],"based_on":"sha256:7d002fe17259e675bc00d7c2dcffcff216865c2ec8e7cd9fc3f7d011d213106e","model":"claude-haiku-5-5","at":"2026-11-19T14:15:36+07:00"}
```

**DELTA**: The person can turn on `[ai] enrich = "claude"` and run `easy-note ai enrich <id>` or `--pending` (or set `after_save = true`), and their own `claude` CLI writes the AI detail with no Claude session, no `enrich_notes` run and no API key; before, `ai enrich` did not exist and `doctor` had no `ai:` line.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]
