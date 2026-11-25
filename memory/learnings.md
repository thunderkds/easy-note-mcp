# learnings.md — Cold Tier: Clarifications, Patterns & Gotchas

> **Rules**: Supervisor-only writes. Each entry dated (`YYYY-MM-DD`) and citing the file/task it came from (the diff-driven pass greps this file by changed file path).

## Requirement Clarifications

## Patterns

## Gotchas

### 2026-11-16 — Auto close-out placeholder check; easy-verifier CLI reports dir (HUN-257)
- **Files**: `.claude/hooks/lib/auto_closeout.py`, `tasks/TASK_REVIEW_HUN-257.md`, `skills/score/SKILL.md`
- The auto close-out treats any Evidence Notes cell holding an angle-bracketed lower-case word (regex `<[a-z][^<>\n]*>`, e.g. a CLI usage like `-e <text>`) as unfilled, and reports `Gate 5: the tests row is not ☑ pass with pasted output`. Write usage placeholders in capitals (`-e TEXT`).
- The `Not done:` slot must already read `auto verify: ☑ ...` or the template placeholder for the hook to fill it; a free-text "auto verify: not started" is left alone and the row is not moved.
- easy-verifier MCP is not registered in Supervisor sessions; rung 2 works with `/home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-verifier-mcp/.venv/bin/python -m easy_verifier.adapters.cli`. `~/.easy-verifier-reports/` is owned by the container uid (10001), so set `EASY_VERIFIER_REPORTS_DIR=~/.easy-verifier-reports-local/<branch>` for the CLI.
- `stop_auto_verify.py --start` refuses from a linked worktree; run it from the main checkout.

### 2026-11-15 — easy-verifier scores of early branches (HUN-251, HUN-252)
- **Files**: `.github/workflows/ci.yml`, `uv.lock`, `scripts/probe_hook.py`, `scripts/probe_server.py`, `tests/test_T001_probes.py`
- A branch cut from `main` before the skeleton (HUN-252) merges has no pyproject or CI, so easy-verifier test-strategy flags `test_config_and_ci_missing` and overall stays under 80. Merge the skeleton first, then merge `main` into the branch and rescore.
- `uv.lock` (~1,100 lines) pushes a `changes` diff past easy-verifier's 200000-character ceiling, so `change_risk` abstains (`below_coverage_floor`). It is not a failed rule.
- Supply-chain rises when CI pins actions to full commit SHAs (resolve with `git ls-remote`, peel annotated tags with `^{}`) and installs with `uv sync --locked`.
- Score `findings` need `confidence` as `low|medium|high` and an `evidence_ref` that appears in that dimension's pack; `test_pairs[].ref` is `path:line`, not a range.

### 2026-11-15 — `[admin]` merge gate behaviour (HUN-251, HUN-252)
- **Files**: `.claude/hooks/pre_bash_block_unsafe_merge.py`, `.claude/hooks/stop_auto_verify.py`, `.claude/hooks/lib/auto_closeout.py`
- The `[admin]` request must name the task (`[admin] merge HUN-252`); a bare `[admin]` merges nothing.
- A merge checks the whole board: any task In Progress, or in Ready for Review with a `☑ fail` Evidence row, blocks every merge. Park a task no one works on before merging another.
- `[admin]` allows no merge of `main` into a task branch (`main names no single task`), and no compound `git commit ... && git merge`. The owner runs that merge-in.
- `stop_auto_verify.py --start` needs a branch name that holds the task ID (e.g. `feat/HUN-252-...`); `worktree-agent-<hex>` branches are ambiguous.
- Commit the hook-written verify row on the task branch (a review-only commit) before the admin merge; the gate reads it from the branch.
- A task branch cut from `main` before the kit exists has no guide, so the auto close-out reports `Gate 5: no ## Evidence section`.
- Merge in a separate worktree for `main` (`.claude/worktrees/main-integration`): `memory/event-trace/*.jsonl` are tracked, and switching the main checkout's branch would swap the gate's evidence.

### 2026-11-15 — easy-verifier counts `.claude/harness-lock.json` hashes as secrets (HUN-251, HUN-253)
- **Files**: `.claude/harness-lock.json`
- easy-verifier's lockfile role matches `**/*-lock.*` (`easy-verifier-mcp/src/easy_verifier/core/roles.py:208`), so the kit's `harness-lock.json` is read as repo context; its ~147 sha256 hashes trip `high_entropy_hex`, and rule `redaction_hits_observed` (weight 35, `judge.py:745`) drops security to ~30 on every branch once the kit is on `main`. No repo-side exclude exists. The fix belongs in easy-verifier; until then every Companion score row fails and needs an owner override to merge.

### 2026-11-15 — TASK_REVIEW template text trips the verifier; [admin] needs a named task and a traced test run
- **Pattern**: The Companion score row's template placeholder (it quotes an example report filename) is read by easy-verifier as `key_material_segment`, costing security 70 points (overall about 10). Replace that placeholder with plain "Pending" text before the first score call; HUN-256 went 72 → 81 from that alone.
- **Pattern**: `[admin] for all` is refused by the merge gate: each `[admin]` request must name its task. The gate also needs a direct pytest call recorded in `memory/event-trace/<ID>.jsonl` (write the session state file first); a run made before the state file named the task does not count.
- **Pattern**: The headless verify run may leave an untracked `skills/verify/SKILL.md` in the task worktree; move it out before merge.
**Files**: tasks/TASK_REVIEW_HUN-256.md, templates/TASK_REVIEW_template.md

### 2026-11-16 — Masking raw YAML misses escaped tokens (HUN-270)
- **Pattern**: Running the secret guard on the raw YAML source of a front-matter value is not enough. In a double-quoted scalar, `\n`/`\t`/`\r` before a token removes the `\b` boundary the anchored patterns need, and `\x41` escapes can spell a token. Mask the decoded scalar strings too (`store.rs` `mask_decoded`), and test each escape before each anchored kind.
**Files**: src/core/store.rs, src/core/guard.rs, tests/leak.rs

### 2026-11-16 — rmcp argument errors echo escaped values; spawn pull rewrites the worktree's board cache (HUN-258)
- **Pattern**: When an MCP tool argument has the wrong type, rmcp/serde quote the value in the error text, JSON-escaped, so `guard::mask_for_output` cannot see a secret in it. `src/surfaces/mcp.rs` overrides `call_tool`: a `failed to deserialize parameters:` error other than `missing field` becomes a fixed message, and `error.data` is cleared. The leak harness's `mcp` surface caught this; keep it for any new tool.
- **Pattern**: `scripts/pull_moment.py spawn <ID>` run from a task worktree rewrites that worktree's `PROJECT_KANBAN.md` and the `tasks/TASK_GUIDE_*.md` copies from the board. Restore them (`git restore <files>`) before review, so the task branch carries only its own work.
- **Pattern**: Two Supervisor sessions can run in parallel when their tasks share only small hunks: HUN-258 ran beside HUN-257 and touched `src/surfaces/cli.rs` (`Command::Serve` flags, `run_serve`) and one `tests/leak.rs` `SURFACES` entry. The second branch to merge resolves the conflict.
**Files**: src/surfaces/mcp.rs, tests/leak.rs, scripts/pull_moment.py

### 2026-11-17 — `board_sync.py update --guide` pushes the guide of the checkout the script lives in (HUN-265)
- **Pattern**: `scripts/board_sync.py` sets `ROOT = script_project_root()` (`scripts/board_sync.py:85`), so `python3 scripts/board_sync.py update --guide <ID>` run from the main checkout pushes main's `tasks/TASK_GUIDE_<ID>.md`, not the re-planned copy in the task worktree. Run it as `cd <worktree> && python3 scripts/board_sync.py update --guide <ID>`. A board round trip also rewrites list markers (`- [x]` → `- [X]`, `-` → `*`): content is the same, so `git restore` the cache in the worktree.
- **Pattern**: Under `[admin]` the Stop hook's auto-mode check fires while background implementers are still running, and its second fire writes a `Paused: auto-mode check failed` line into the worktree's `TASK_REVIEW_<ID>.md`. Replace that line when the report is filled; it is not the agent's.
**Files**: scripts/board_sync.py, tasks/TASK_REVIEW_HUN-265.md

### 2026-11-17 — The merge gate pulls the board before every merge; parallel task branches need the cache committed first (HUN-262, HUN-263, HUN-265)
- **Pattern**: `.claude/hooks/pre_bash_block_unsafe_merge.py:2741` runs `board_sync.py pull` before a merge, which rewrites every `tasks/TASK_GUIDE_<ID>.md` copy in the main checkout. A branch that changed its own guide (a re-plan) is then refused by git ("local changes would be overwritten"), and `git restore` does not help because the next merge pulls again. Workaround used: commit the board cache on `main` first (`47fac19`), merge, and resolve the guide conflict with `git checkout --ours` (the board's copy). Kit fix to make: restore the cache after the gate's pull, or skip guides the merge touches.
- **Pattern**: That cache commit makes `main` more than one merge commit ahead of `origin/main`, so the `[admin]` push of `main` is refused (only a fast-forward to the branch tip, or one merge commit on top of the remote `main`, passes). The owner pushed `main` by hand.
- **Pattern**: The merge gate checks the whole board: no merge while any task is In Progress, or in Ready for Review without a verify pass. A test-only task gets `SKIP` from the headless verify, and the hook's docs-only list has no `tests/*.rs`, so it ends `blocked` twice; only the owner's `/verify <ID>` (row `/verify by the user, session <id>, commit <sha>`) clears it. Run test-only tasks last, or alone.
- **Pattern**: Each `[admin]` merge needs its own request naming the task (`merge HUN-263 [admin]`); a bare "admin", "yes" without the tag, or "run all" binds nothing.
- **Pattern**: Two branches that both add an item to `tests/leak.rs` `SURFACES`, the registry test list, `cli.rs` `every_command` and `cli/read.rs` imports conflict on those lines only; rebase the second onto `main` in its worktree, keep both items, rerun the suite on both feature sets and start a new auto verify (the earlier pass is for other code).
**Files**: .claude/hooks/pre_bash_block_unsafe_merge.py, scripts/board_sync.py, tests/leak.rs


### 2026-11-19 — Obsidian Bases syntax; `claude -p` JSON shape; auto verify slot and merge-gate gotchas (HUN-273–HUN-276)
- **Pattern**: Obsidian Bases (obsidian.md/help/bases/functions): a String has no `matches()`; regex is `/re/.matches(text)`, e.g. `/(^|\/)\d{4}\/\d{2}$/.matches(file.folder)` (`file.folder` is vault-relative, so do not anchor at `^\d{4}`). `date()` parses only `YYYY-MM-DD HH:mm:ss`, so ISO `created` with an offset fails: filter "today" by the note ID, `file.name.startsWith(today().format("YYYYMMDD"))`. `sort:` is not in the documented view schema. A parse test does not prove Obsidian accepts a `.base`: check the docs.
- **Pattern**: `claude -p --output-format json --json-schema …` returns `structured_output` (the object), `result` (the same as a JSON string), `is_error`, and `modelUsage` keyed by model id. Bare note text on stdin is read as a request ("I can't buy things"), so the prompt must frame the note as data.
- **Pattern**: Write the report's slot as `auto verify: running (T182)` before `stop_auto_verify.py --start`; any other text ("not started") is "the user's" and the run leaves it, so close-out never moves the row, and the Supervisor may not fill the slot itself (auto-mode classifier: Undermining Oversight). The hook writes the `verify` row in the worktree but does not commit it: commit `tasks/TASK_REVIEW_<ID>.md` on the branch before the merge, or the gate says "no passed auto verify committed".
- **Pattern**: The merge gate counts a test run only from a `Bash` call whose command starts with the runner (`cargo test --manifest-path <worktree>/Cargo.toml …`); `printf … && cd … && cargo test` does not count. The headless `/verify` may end `blocked` with a PASS in prose ("no single verdict line"); the one `[admin]` retry passed. Parallel `[admin]` tasks need the owner to name them (`merge HUN-273 HUN-276 HUN-274 HUN-275 [admin]` worked as one request). Commit `memory/event-trace/_untagged.jsonl` on `main` before each merge: a branch that touched it otherwise aborts the merge.
**Files**: src/surfaces/dashboard.rs, src/surfaces/enrich.rs, .claude/hooks/stop_auto_verify.py, .claude/hooks/pre_bash_block_unsafe_merge.py, tasks/TASK_REVIEW_HUN-273.md

### 2026-11-19 — Merged tasks left in Ready for Review block every later merge; a hook verify row counts only at its own head commit (HUN-277)
**Learning**: HUN-273–276 were merged but stayed in Ready for Review (`Paused: done not moved — Gate 5: no test run in memory/event-trace/<ID>.jsonl`). The merge gate checks the whole board, and an `auto-run by hook` verify row counts only when its run record matches the checkout's head commit, so after any later main commit those rows read `(no evidence row)` and blocked the HUN-277 merge. Fix used: the owner approved moving the four rows to Done by hand. Prevent it: set `.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID` to the task ID before running its tests, and start the test command with the runner itself (`cargo test --manifest-path <worktree>/Cargo.toml …`), so the trace records the run under the task before auto verify starts.
**Files**: .claude/hooks/pre_bash_block_unsafe_merge.py, .claude/hooks/stop_auto_mode_check.py, PROJECT_KANBAN.md

### 2026-11-21 — A deb test runs the real dpkg tools in a second; an empty PATH hides bash from the test (HUN-287)
- **Pattern**: `EASY_NOTE_BIN=/bin/true scripts/build_deb.sh --out <tmp>` runs the real staging, the real `dpkg-shlibdeps` and the real `dpkg-deb --build` in under a second, so packaging has a fast integration test with no release build. `dpkg-deb --field <deb> Package Version Architecture Depends` reads the control fields in one call, and `--info` shows the control archive's members, so it proves a maintainer script is absent. Assert the *count* of `-`-prefixed lines in `--contents`, or "and nothing else" goes unchecked.
- **Gotcha**: a test that sets `.env("PATH", <empty dir>)` must name the interpreter by its full path: Rust resolves `Command::new("bash")` against the *child's* PATH, so the call fails with `NotFound` instead of running the script. Resolve it first (`sh -c "command -v bash"`).
- **Gotcha**: a `cargo` stand-in on PATH checks the script's feature list without a release build, but it cannot prove the script then stops — `target/release/easy-note` may still be there from an earlier real build, and the script happily packages it. Assert the recorded arguments only.
- **Pattern**: `dpkg-shlibdeps` wants a `debian/control` beside its working directory; write a throwaway one *outside* the staged package root, or it ships inside the `.deb`.
**Files**: scripts/build_deb.sh, tests/deb.rs

### 2026-11-22 — `[admin]` counts only the hook's own verify; a blocked deb verify needs a hand merge; merge resolutions can break Markdown tables (HUN-285/286/287)
- **Gotcha**: `[admin] merge <ID>` accepts only a hook-written `auto-run by hook` verify pass. A user `/verify` row (`/verify by the user, session <sid>, commit <sha>`) satisfies the plain merge gate but not `[admin]`, so HUN-287 (its headless verify blocked on Bash approval for `bash scripts/build_deb.sh`) needs the owner's hand merge, or an allow rule for that command and a fresh auto verify.
- **Gotcha**: the gate reads only the latest user message: an `[admin]` tag must name exactly one task ID in that message, one merge per message. `git merge --abort` and merging `main` into a task branch are refused under `[admin]` too; the owner runs them.
- **Gotcha**: a `ReviewerDone` record holds only for the commit it names; any code or doc commit after it (even a one-line PRD fix) needs another `Reviewer: <ID>` spawn. Commits that touch only review files keep it valid.
- **Pattern**: resolving a conflict in a Markdown table row with a string replace can leave a blank line inside the table, which ends it; check `PRD.md` tables after every merge (HUN-286 `bb6888f` → fixed `ad41bf5`).
- **Gotcha**: a `.deb` built from a task branch holds only that branch plus `main`; features in Ready for Review are not in it, and every build reports `0.1.0`, so `--version` cannot tell builds apart.
**Files**: .claude/hooks/pre_bash_block_unsafe_merge.py, scripts/build_deb.sh, PRD.md

- **Gotcha**: a `verify` Evidence row filled by hand (even `☑ pass`) makes `stop_auto_verify.py --start <ID>` print "its result is already filled for commit <sha> — nothing started", and the `[admin]` merge gate then has no hook-written row. Leave the row empty until the hook writes it; put your own real-run notes in a separate section (HUN-288, HUN-289).
- **Gotcha**: easy-verifier `score` rejects `findings` whose `evidence_ref` is not a `path:start-end` inside the dimension's own evidence pack, and returns a 100 KB payload that the harness saves to a file; read it with `jq` (`.overall`, `.change_risk`, `.needs_input`). A first call with no `agent_input` returns `change_kind`, `picks` and `test_pairs` questions; answer them on the second call.
- **Pattern**: to see a webview window work without touching the real desktop, run `Xvfb :77`, `GDK_BACKEND=x11`, unset `WAYLAND_DISPLAY`, `WEBKIT_DISABLE_DMABUF_RENDERER=1`, drive keys and clicks with `libXtst` through ctypes (`XTestFakeKeyEvent`), and capture with `import -window root`; no window manager, so move the pointer over the window first. A temp `HOME`, `EASY_NOTE_DIR` and a stub `claude` on `PATH` keep it away from real notes.
**Files**: tasks/TASK_REVIEW_HUN-288.md, tasks/TASK_REVIEW_HUN-289.md

### 2026-11-25 — Round 6 gotchas: gate binding, parked tasks, GTK, hashes (HUN-290 to HUN-293)
- **Gotcha**: with 4 tasks open at once the `[admin]` loop binding never holds (it needs no other task In Progress or Ready for Review, and `--start` typed by the session). Ask one question naming exactly one task with no other task ID in the message; the user's `merge <ID> [admin]` then binds it.
- **Gotcha**: the pipeline gate blocks any merge while any task is In Progress, or in Ready for Review with no hook-written verify pass in main. A task that cannot be verified (HUN-290: no GTK on this machine) must be parked in Backlog through the Linear connector; `board_sync.py push status` has no Backlog, and its `todo` lands in Ready for Review (`status_map`).
- **Gotcha**: `board_sync.py pull` rewrites the cache from the board and undoes the auto close-out's local Done move; the board itself keeps Ready for Review. The pass evidence stays in the branch's `TASK_REVIEW`.
- **Gotcha**: the merge command must be one git command alone (no `board_sync` before it in the same call); the pipeline gate and the `[admin]` gate each refuse a chained command.
- **Gotcha**: a reviewer spawn needs the full 40-hex commit. A guessed hash records nothing and wastes a review; get it with `git rev-parse <branch>`.
- **Gotcha**: the Stop hook wants a `code-review` Skill call attributed to each extended task in the Supervisor's own trace, even when a fresh reviewer ran it. Write `active_task.d/$CLAUDE_CODE_SESSION_ID` for that task, then call the skill.
- **Gotcha**: a commit after the reviewed commit may change only `TASK_REVIEW`, `PROJECT_KANBAN`, `MINIMAL_LOG` or `memory/*.md`. Restoring the guide counted as a code change and needed a third reviewer.
- **Pattern**: the window feature (`--features window,tray`) needs `pkg-config`, `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libdbus-1-dev`; without them every window task's guide command is `blocked` in the headless verify.
- **Pattern**: the `[admin]` attributes scan walks the whole checkout and refuses after 10 s; it failed once while cargo builds were running, and passed at 0.3 s when idle.
**Files**: .claude/hooks/pre_bash_block_unsafe_merge.py, scripts/board_sync.py, tasks/TASK_REVIEW_HUN-290.md, tasks/TASK_REVIEW_HUN-293.md
