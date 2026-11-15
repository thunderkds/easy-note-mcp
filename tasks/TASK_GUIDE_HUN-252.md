**Date**: 2026-11-14
**Complexity Level**: C1
**Risk Level**: Low
**Priority**: P0
**Assigned agent**: Common-Infrastructure-Agent
**Agent guide**: `agents/common-infrastructure.md`
**Plan ID**: T002 (`linear-tasks.md`, `PROJECT_SPEC.md` Tasks)

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*, *Context*, *Guard*, *Trash*, *Skipped*)
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read `agents/common-infrastructure.md`
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in `agents/common-infrastructure.md`
6. **C2/C3 or multi-file tasks only**: read `memory/codebase-map.md` if it exists
7. Tests only: temp `HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/easy-note/` or `~/.claude/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Plan T002: "`uv tool install .` gives `easy-note`; `easy-note --version` and `easy-note serve` start; `pytest` runs green on an empty suite; repo holds CLAUDE.md (`@AGENTS.md`), AGENTS.md, PRD.md, PROJECT_SPEC.md, PROJECT_KANBAN.md, RUNBOOK.md, BRAINSTORMING_LOG.md, GRILL_requirement.md, `tasks/`, `memory/`, `reports/`, `docs/ddr/`." The repo `thunderkds/easy-note-mcp` exists and the kit is installed (files not yet committed).

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> A clean Python package every later task builds on: one core, two console scripts (`easy-note`, `n`), an MCP `serve` stub, a green test run locally and in CI, and the kit files tracked in git.

**Out of scope** (what this task explicitly does NOT do):

* Any note logic (T003+)
* The MCP tools (T008)
* Publishing to PyPI

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-013: CLI and MCP share one core
* FR-020: console script `n`
* NFR-006: Python 3.11+, `uv tool install`
* NFR-010: the CLI never imports the MCP SDK

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent)
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy)
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

> An agent must NOT start implementing until this gate is checked. If anything here is unclear,
> STOP and ask the Supervisor (Karpathy: Think Before Coding).

---

## Dependencies & Reachability

**Depends on**: None

**Entry point**: `easy-note --version`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `pyproject.toml` (hatchling or uv build backend) declares package `easy-note-mcp`, Python `>=3.11`, deps `mcp`, `pyyaml`, extra `[tray]` = `pystray`, `pillow`, and console scripts `easy-note` and `n` | NFR-006, FR-020 |
| 2 | `src/easy_note/{__init__,core/__init__,surfaces/__init__}.py` and `surfaces/cli.py`, `surfaces/mcp.py` exist; `cli.py` dispatches subcommands with `argparse` | PROJECT_SPEC layout |
| 3 | `uv tool install .` then `easy-note --version` prints the package version; `n --version` prints the same | entry points |
| 4 | `easy-note serve` starts a stdio MCP server with zero tools and exits cleanly on EOF | MCP entry |
| 5 | A test imports `easy_note.surfaces.cli` in a subprocess and asserts `mcp` is not in `sys.modules` | NFR-010 |
| 6 | `uv run pytest -q` passes; `.github/workflows/ci.yml` runs it on Python 3.11 and 3.12 (Linux, macOS) | CI |
| 7 | `RUNBOOK.md` (release steps stub) and `reports/.gitkeep` exist; the kit files and planning docs are tracked in git | repo artifacts |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

> The reviewer fills **Evidence** at Stage 4/5 in `tasks/TASK_REVIEW_<ID>.md` (from `templates/TASK_REVIEW_template.md`). The implementing agent must NOT be the sole author of its own acceptance test.

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | Fresh clone, `uv tool install .` | `easy-note --version` prints `0.1.0` | manual + CI step |
| 2 | `python -c 'import easy_note.surfaces.cli, sys; print("mcp" in sys.modules)'` | `False` | automated test |
| 3 | `echo '' \| easy-note serve` | exit 0, no traceback | automated test |

### Verification Command (exact, runnable)

```bash
uv run pytest -q
uv tool install --force . && easy-note --version && n --version
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-252.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-252.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

N/A — pure backend / CLI task; no UI component. The three UI Evidence rows are ☐ N/A.

---

## Approach

**Pattern reference**: None — no code in this repo yet

**Vital slice**: `pyproject.toml` + the two console scripts + a green pytest; everything else builds on it

**Cut list**: Lint config beyond `ruff` defaults; Release automation (RUNBOOK stub only)

Standard `src/` layout with `uv`. Keep `cli.py` free of `mcp` imports: `serve` imports `surfaces.mcp` lazily inside its handler. `n` points at the same `main` with `add` pre-selected.

---

## Edge Case Checklist

- [ ] `n` may clash with another `n` on PATH (e.g. the Node version manager) — `doctor` reports it later; note it in RUNBOOK
- [ ] `.board/` holds a local triage marker — must be git-ignored
- [ ] The kit's `.claude/skills` and `.claude/agents` are relative symlinks — commit them as symlinks, never replace them
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: for each fact this task changes, grep README, `docs/`, comments and docstrings for the old wording

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `pyproject.toml` | new |
| `src/easy_note/**` | new skeleton |
| `tests/test_skeleton.py` | new |
| `.github/workflows/ci.yml` | new |
| `RUNBOOK.md`, `reports/.gitkeep` | new |
| `.gitignore` | Python + `.board/` + `.venv/` |
| Pre-existing pin tests | List each test in `tests/` that pins text, paths or counts of a file this task edits, and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md` | Supervisor-owned; report needed changes instead |
| `docs/ddr/*` | Decisions are owner-approved; a change needs a new DDR from the Supervisor |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/` | Kit files; Supervisor-only |
| The real `~/easy-notes/`, `~/.config/easy-note/`, `~/.claude/`, `~/.codex/`, `~/.cursor/` | Tests use a temp `HOME` and `EASY_NOTE_DIR` only (NFR-003) |

---

## Test Plan

`tests/test_skeleton.py`: version flag, `n` = `add` dispatch, no `mcp` import from the CLI, `serve` exits on EOF.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI / client
- [ ] Supervisor notified: task ready for Stage 4 review