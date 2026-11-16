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
