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
