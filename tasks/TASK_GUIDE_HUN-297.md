**Date**: 2026-11-25
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

 1. Read `PROJECT_SPEC.md` (glossary: *Note*, *AI detail*, *Project*, *Topic*, *Kind*, *Override*)
 2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
 3. Read this file completely
 4. Read your agent guide (`agents/backend.md`)
 5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in your agent guide
 6. Read `memory/codebase-map.md`
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `note_stats`, `note_list`, `mcp`, `rmcp`, `HUN-258`, `HUN-262` before reading source files
 8. Read `PRD.md` (US-022, FR-012, FR-016, FR-051, NFR-014), `GRILL_round6.md`, `BRAINSTORMING_LOG_round6.md` (Path B), `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0007-ai-detail-from-the-local-claude-cli.md`
 9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`. A fake `claude` script stands in for the model; no test starts the real CLI
10. Round 6 rule: **a raw note file is never changed by the AI** (NFR-012, NFR-015). If your change needs one, stop and write a `Question:`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-25: "easy-note is the place where I only take notes; the AI does the rest — arranges the notes, checks drafts, enriches them on a schedule." Decisions: `GRILL_round6.md` G1–G5, Path B in `BRAINSTORMING_LOG_round6.md`.

**Restated intent**:

> Once a day the AI writes a short digest for each project and topic whose notes changed. An agent calls one read-only MCP tool, `note_context`, to get the digest and the newest notes of a project or topic, or the list of projects and topics.

**Out of scope**:

* Any MCP tool that writes, edits or moves a note
* A digest in the window (the views slice may show it later)
* Digests for a project or topic with fewer than 3 notes: their summaries are joined with no model call

**Requirement Refs**:

* FR-051: digest files and `note_context`
* FR-012: tool count 7 → 8 (Open Question 8: owner approval before merge)
* FR-016: item shape and bounds
* NFR-014: the digest job runs only under the same opt-in
* US-022

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are covered

---

## Dependencies & Reachability

**Depends on**: HUN-295 (`ai run` and `jobs`)

**Entry point**: MCP tool `note_context(project?, topic?, limit≤10)`; the `digest` job inside `ai run`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `jobs` containing `digest` makes `ai run` write `<notes dir>/.easy-note/ai/digest/<project|topic>.json` once a day (at or after `digest_hour`, when none was written today) for each project or topic with ≥ 3 notes whose member hash (`based_on` over member ids + their AI file hashes) changed; fields: `summary` (≤ 1,200 chars, masked), `based_on`, `at`, `model`, `members` | FR-051 |
| 2 | A project or topic with 1–2 notes gets a digest made by joining their AI summaries (or first lines), with `model: "joined"` and no model call | FR-051 |
| 3 | The digest call uses the same child rules as FR-037; a failed digest leaves the old one and records `last_error` | FR-037, FR-051 |
| 4 | MCP `note_context(project, topic?, limit)` returns `{digest, items, total, truncated}`; items use the FR-016 shape (id, title, created, repo, task, tags, ≤ 200-char snippet, plus `project`, `topic`, `kind`); `limit` ≤ 10; a note with an override shows the override | FR-051, FR-016 |
| 5 | `note_context()` with no argument returns the projects and topics with counts, at most 50 each | FR-051 |
| 6 | The tool never writes and never starts a model; it appears in the tool list with a description that tells the model when to use it; the tool count is 8 and the pin test says so | FR-012, FR-051 |
| 7 | All text out of the tool is masked, as every read output (FR-014) | FR-014 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | digest job over a fixture with 2 projects (one with 2 notes) | one model digest, one joined digest | tests/ai_run.rs (fake claude) |
| 2 | `note_context(project="easy-note-mcp")` over MCP stdio | digest + ≤ 10 items, truncated flag right | tests/mcp.rs |
| 3 | `note_context()` empty args | lists with counts | same |
| 4 | planted secret in a note and a digest | `[masked:…]` in the response | tests/mcp.rs + leak suite |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-297.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-297.md` (review sections are never taken from the board).

## Approach

**Pattern reference**: `note_list` / `note_search` in `src/surfaces/mcp.rs` for the item shape and bounds; `core::ai` for AI file IO.

**Vital slice**: Digest writer (model + joined) and the tool; one read function shared by both.

**Cut list**: A digest history; per-note digest links; a markdown export.

---

## Edge Case Checklist

- [ ] Digest `based_on` changes only by whitespace: still a change, one extra call is fine
- [ ] A topic renamed by the person: old digest file removed, new one built at the next digest run
- [ ] Digest hour passes while the machine is off: run at the next start of `ai run`
- [ ] `limit` = 0 or > 10: clamp to the range
- [ ] A project name with `/` or `..`: file name is a sanitized slug plus a short hash; lookup uses the same function
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for text this change makes wrong

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/organize.rs` | digest job |
| `src/core/organize.rs` | digest prompt, schema, slug |
| `src/surfaces/mcp.rs` | `note_context`, tool description |
| `tests/mcp.rs`, `tests/ai_run.rs`, `tests/leak.rs` | AC 1–7 |
| `PROJECT_SPEC.md` MCP contract is Supervisor-edited; list the change in the review file | — |
| `README.md`, `docs/` | the tool in the client config and tool list |
| Pre-existing pin tests | list each test that pins current behavior (help text, asset list, README, MCP tool count) and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*`, `docs/design/*` | Supervisor / owner owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| Note files and the note writer (`Store::update`, `Store::add` in `src/core/store.rs`) | The AI never changes a raw note (NFR-012, NFR-015); call existing public functions only; a missing one is a `Question:` to the Supervisor |
| `src/core/guard.rs` patterns | Secret guard rules are frozen; call them, never edit them |

---

## Test Plan

MCP stdio test as in `tests/mcp.rs`; fake `claude`; digest slug table test; byte-compare fixture for raw notes.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] **easy-verifier**: Stage 4 `Skill({ skill: "score" })` run by the Supervisor
- [ ] Security review: `Skill({ skill: "security-review" })` run (note text reaches the model, or a new write path)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md` (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-25 OPEN Owner approves the MCP surface growing from 7 to 8 tools (PRD Open Question 8) before this task merges