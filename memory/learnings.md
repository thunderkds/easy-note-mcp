# learnings.md — Cold Tier: Clarifications, Patterns & Gotchas

> **Rules**: Supervisor-only writes. Each entry dated (`YYYY-MM-DD`) and citing the file/task it came from (the diff-driven pass greps this file by changed file path).

## Requirement Clarifications

## Patterns

## Gotchas

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
