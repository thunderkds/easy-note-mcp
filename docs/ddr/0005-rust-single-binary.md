# 0005. Rewrite easy-note in Rust as one Linux binary; remove all Python

**Status**: Accepted (owner, 2026-11-15: "yes, go ahead with R1")
**Date**: 2026-11-15
**Deciders**: thunderkds
**Related**: PRD NFR-004, NFR-006, NFR-009 · amends DDR 0001 (install method) · DDR 0002 (file format, unchanged) · DDR 0003 (hook, unchanged in behaviour)

---

## Context

v1 was planned and partly built in Python ≥ 3.11 (FastMCP, PyYAML, `uv`). Merged on `main`: HUN-251 (spike), HUN-252 (skeleton), HUN-253 (format + store), HUN-254 (context), HUN-256 (query), HUN-267 (task pattern). HUN-255 (secret guard) is built and verified on `feat/HUN-255-secret-guard` but not merged.

On 2026-11-15 the owner said the Python choice was a mistake and asked to convert to Rust. The owner's reasons:

1. **One binary to install.** No Python, no `uv`, no virtual env on the machine.
2. **Startup speed.** The capture hook (NFR-009 pass path p95 < 50 ms), `n` and the stdio MCP server start on every use.
3. **The owner prefers Rust** to own and read.

Scope picked by the owner: **Linux only** in v1, and **delete all Python** from the repo.

DDR gate: surprising (a stack change after six merged tasks) ✅, genuine trade-off ✅, hard to reverse ✅ → DDR.

---

## Decision

1. easy-note is one Rust binary, `easy-note`, built with `cargo`. `n` is the same binary (a hard link or symlink made at install, dispatched on `argv[0]`), so capture stays one word.
2. **Linux only** in v1 (x86_64). macOS and Windows are out of scope; NFR-006 changes to match.
3. Install: `cargo install --path .` (from a checkout) and `cargo install --git https://github.com/thunderkds/easy-note-mcp`. A prebuilt Linux binary on GitHub Releases is a later, optional task. ⚠ The owner did not pick an install method; this is the Supervisor's default.
4. **All Python is removed** in the first Rust task: `src/easy_note/`, `tests/*.py`, `pyproject.toml`, `uv.lock`, `scripts/probe_*.py`, and any Python CI. The kit's own scripts (`.claude/hooks/*.py`, `scripts/board_sync.py`, `scripts/pull_moment.py`, `scripts/lane_check.py`) are tooling, not the app, and stay.
5. Behaviour does not change. The PRD's requirements, DDR 0002 (the note file format), the frozen MCP tool contract (PROJECT_SPEC.md) and the CLI command set stay as they are. The Python test suite is the behaviour oracle: each Rust task ports the tests of the Python task it replaces before the code (red first).
6. Proposed crates (Stage 2 confirms each in the task guide; Search Before You Build):
   - MCP: `rmcp` (official Rust MCP SDK; stdio and streamable HTTP).
   - CLI: `clap`.
   - Front matter: a small hand-written reader/writer for the DDR 0002 subset, or a maintained YAML crate. `serde_yaml` is archived and is not used.
   - Secret guard: the patterns in `guard.py` use look-ahead (`(?!...)`), which the `regex` crate does not support. Either restate them without look-around, or use `fancy-regex` for those patterns only. The leak oracle decides.
   - Time: `time` or `chrono`; hashing: `sha2`.

---

## Consequences

- **Lost work:** about 11 Python source files and 928 tests are rewritten. Their git history stays, and the tests carry over as the spec.
- **Board:** HUN-252 to HUN-256 and HUN-267 reopen as Rust tasks, and HUN-255 is not merged. HUN-257 onward are re-planned in Rust before pickup.
- **Faster start:** a native binary meets NFR-009 without the "stdlib imports only" workaround. HUN-265 measures it.
- **Narrower reach:** Linux only.
- **Kit tooling stays Python:** the hooks and board scripts are kit code and are out of this decision.

---

## Alternatives considered

1. **Finish v1 in Python, then port.** Rejected by the owner ("Pause, re-plan, rewrite").
2. **Keep Python and ship a frozen executable (PyInstaller, shiv).** This gives one file, but the start is slow and the owner prefers Rust.
3. **Rewrite now and keep the Python as a branch.** Rejected: the owner asked to delete all Python. The history keeps it anyway.
