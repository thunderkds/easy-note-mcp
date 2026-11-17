Start the Rust rewrite (DDR 0005). One cargo project builds one Linux binary, `easy-note`. Running it as `n` gives the one-line capture entry point. A GitHub Actions job runs `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test`. All Python app code is removed in the same change.

Replaces [HUN-252](https://linear.app/thunderkds/issue/HUN-252/repo-skeleton-uv-project-srceasy-notecoresurfaces-cli-and-mcp-entry) (Python skeleton).

+++ Agent notes

Guide: `tasks/TASK_GUIDE_<this id>.md`. Delete: `src/easy_note/`, `tests/*.py`, `pyproject.toml`, `uv.lock`, `scripts/probe_hook.py`, `scripts/probe_server.py`, Python CI. Keep kit tooling (`.claude/hooks/*.py`, `scripts/board_sync.py`, `scripts/pull_moment.py`, `scripts/lane_check.py`, `scripts/gen_sandbox_profile.py`, `scripts/verify_trace.py`).

+++