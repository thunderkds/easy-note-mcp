**Date**: 2026-11-15
**Complexity Level**: C1
**Risk Level**: Medium
**Priority**: P0
**Assigned agent**: Common-Infrastructure-Agent
**Agent guide**: `agents/common-infrastructure.md`
**Plan ID**: R1 (DDR 0005; replaces HUN-252 / T002)

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*, *Context*, *Guard*, *Trash*, *Skipped*)
2. Read `docs/ddr/0005-rust-single-binary.md` in full
3. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
4. Read this file completely
5. Read `agents/common-infrastructure.md`
6. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in `agents/common-infrastructure.md`
7. Tests only: temp `HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/easy-note/` or `~/.claude/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, 2026-11-15: "could we convert to rust", "linux only, delete all the python related"; DDR 0005 (one Rust binary, `cargo install`, Linux only, all Python app code removed in the first Rust task; behaviour unchanged).

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> A clean cargo project every later Rust task builds on: one binary `easy-note`, `n` as the same binary, an MCP `serve` stub over stdio, a green `cargo test` locally and in CI on Linux, and no Python app code left in the repo.

**Out of scope** (what this task explicitly does NOT do):

* Any note logic (R2+): no format, store, guard, context or query code
* The MCP tools (later surface task)
* Prebuilt release binaries (GitHub Releases)
* The kit's own Python tooling (`.claude/hooks/*.py`, `scripts/board_sync.py`, `scripts/pull_moment.py`, `scripts/lane_check.py`, `scripts/gen_sandbox_profile.py`, `scripts/verify_trace.py`): keep it

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-013: CLI and MCP share one core (one crate, `core` and `surfaces` modules)
* FR-020: `n` equals `easy-note add` (dispatch on `argv[0]`; `add` itself is a later task — here `n` and `easy-note add` both print a "not implemented yet" line and exit 2)
* NFR-006: one Rust binary, `cargo install`, Linux only
* NFR-010: the CLI path never starts the MCP server or its async runtime

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [x] Restated intent confirmed to match the user's request (Supervisor, auto; owner accepted DDR 0005 and the R1–R5 plan on 2026-11-15)
- [x] Domain terms align with `PROJECT_SPEC.md` glossary
- [x] Every Acceptance Criterion below traces to a line in the Requirement
- [x] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

---

## Dependencies & Reachability

**Depends on**: None (DDR 0005 accepted)

**Entry point**: `easy-note --version`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `Cargo.toml` at the repo root declares package `easy-note-mcp`, version `0.1.0`, edition 2021 or later, one `[[bin]]` named `easy-note`; `Cargo.lock` is committed | NFR-006 |
| 2 | `src/main.rs` plus `src/core/mod.rs` and `src/surfaces/{mod,cli,mcp}.rs` exist; the CLI uses `clap` with subcommands `serve` and `add` (stubs) and `--version` | FR-013, PROJECT_SPEC layout |
| 3 | `easy-note --version` prints `easy-note 0.1.0`; the same binary invoked as `n` (symlink) prints the same for `n --version`, and `n hello` behaves as `easy-note add hello` | FR-020 |
| 4 | `easy-note serve` starts a stdio MCP server (official `rmcp` crate) with zero tools: it answers `initialize` and exits 0 on stdin EOF | MCP entry |
| 5 | The MCP server and its async runtime are started only inside `serve`; a test shows `easy-note --version` and `n --version` do not start them (e.g. a unit test on dispatch, or a feature-free code path) | NFR-010 |
| 6 | All Python app code is removed: `src/easy_note/`, `tests/*.py`, `tests/fixtures/` Python-only data that no Rust test uses yet may stay only if a later R task names it (keep `tests/fixtures/` for R2–R5 oracles), `pyproject.toml`, `uv.lock`, `scripts/probe_hook.py`, `scripts/probe_server.py`; kit tooling listed in Out of scope stays | DDR 0005 decision 4 |
| 7 | `.github/workflows/ci.yml` runs on `ubuntu-latest` only: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, then `cargo install --path .` and `easy-note --version`; no Python steps | CI, NFR-006 |
| 8 | `RUNBOOK.md` install section says `cargo install --path .` and how to make the `n` link (`ln -s "$(command -v easy-note)" ~/.cargo/bin/n`); no `uv` instruction left in it | NFR-006 |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

**Locked oracle**: `tests/cli.rs` (integration test, binary run with `std::process::Command`)

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | `cargo build`, run `target/debug/easy-note --version` | prints `easy-note 0.1.0`, exit 0 | automated test |
| 2 | symlink `n` -> the binary, run `n --version` | prints `easy-note 0.1.0` | automated test (temp dir) |
| 3 | `easy-note serve` with an `initialize` request then EOF on stdin | a JSON-RPC result, then exit 0 | automated test |
| 4 | `git ls-files '*.py'` | only kit tooling paths listed in Out of scope | automated check in review |

### Verification Command (exact, runnable)

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cargo install --path . --root "$(mktemp -d)" && echo ok
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-268.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-268.md` (review sections are never taken from the board).

BEFORE command: `date -u; git ls-files '*.py' 'pyproject.toml' 'uv.lock' | grep -v '^\.claude/\|^skills/\|^packs/' ; ls Cargo.toml`

## UI / Design Acceptance Criteria

N/A — pure backend / CLI task; no UI component. The three UI Evidence rows are ☐ N/A.

---

## Approach

**Pattern reference**: None — first Rust code in this repo

**Vital slice**: `Cargo.toml` + one binary with `--version`, `n` dispatch and a `serve` stub + green `cargo test`, and the Python removal

**Cut list**: Release automation; tray/quick features; any note logic

`main.rs` reads `argv[0]`'s file name: `n` → prepend `add`. `serve` builds the tokio runtime inside its handler only. Pick current stable versions of `clap` (derive) and `rmcp` (server + stdio transport features) and `tokio` (only the features `rmcp` needs); record the versions in the review file. Search Before You Build: no hand-written JSON-RPC.

---

## Edge Case Checklist

- [ ] `n` may clash with another `n` on PATH (Node version manager) — say so in RUNBOOK
- [ ] The kit's `.claude/skills` and `.claude/agents` are relative symlinks — never replace them
- [ ] `.gitignore`: add `/target`; drop Python-only entries that no longer apply (`.venv/`, `__pycache__/`) only if no kit tooling needs them
- [ ] Do not delete `tests/fixtures/` data that the R2–R5 oracles reuse (notes_basic, secrets) — move nothing
- [ ] Pin-test pre-flight: no pre-existing Rust tests; Python tests are deleted by this task (that is the point)
- [ ] Stale references: grep README, `docs/`, `RUNBOOK.md` for `uv`, `pytest`, `python` install wording and fix what this task owns (RUNBOOK); list the rest in the report

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `Cargo.toml`, `Cargo.lock` | new |
| `src/main.rs`, `src/core/mod.rs`, `src/surfaces/{mod,cli,mcp}.rs` | new |
| `tests/cli.rs` | new |
| `.github/workflows/ci.yml` | rewrite for Rust, Linux only |
| `RUNBOOK.md` | install section |
| `.gitignore` | `/target` |
| `src/easy_note/**`, `tests/*.py`, `pyproject.toml`, `uv.lock`, `scripts/probe_hook.py`, `scripts/probe_server.py` | delete |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md` | Supervisor-owned; report needed changes instead |
| `docs/ddr/*` | Decisions are owner-approved |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `templates/`, `packs/` | Kit files; Supervisor-only |
| Kit tooling scripts listed in Out of scope | They run the pipeline |
| The real `~/easy-notes/`, `~/.config/easy-note/`, `~/.claude/`, `~/.codex/`, `~/.cursor/` | Tests use a temp `HOME` and `EASY_NOTE_DIR` only (NFR-003) |

---

## Test Plan

`tests/cli.rs`: `--version`, `n` symlink dispatch, `add` stub exit code, `serve` answers `initialize` and exits on EOF.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Lint passes (`cargo fmt --check`, `cargo clippy -D warnings`)
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log
