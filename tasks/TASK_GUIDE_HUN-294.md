**Date**: 2026-11-25
**Complexity Level**: C2
**Risk Level**: High
**Priority**: P0
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
 7. **easy-memory first**: call `mcp__easy-memory__memory_search` with `enrich`, `schema`, `argv`, `child_env`, `claude -p`, `HUN-275`, `HUN-288` before reading source files
 8. Read `PRD.md` (US-020, FR-037, FR-047, NFR-012, NFR-014, NFR-015), `GRILL_round6.md`, `BRAINSTORMING_LOG_round6.md` (Path B), `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0007-ai-detail-from-the-local-claude-cli.md`, `src/surfaces/enrich.rs`
 9. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/` or `~/.local/share/`. A fake `claude` script stands in for the model; no test starts the real CLI
10. Round 6 rule: **a raw note file is never changed by the AI** (NFR-012, NFR-015). If your change needs one, stop and write a `Question:`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-25: "easy-note is the place where I only take notes; the AI does the rest — arranges the notes, checks drafts, enriches them on a schedule." Decisions: `GRILL_round6.md` G1–G5, Path B in `BRAINSTORMING_LOG_round6.md`.

**Restated intent**:

> Given up to `batch_size` pending notes, the engine asks the person's `claude` ONE question and gets back a title, kind, summary, project, topic, tags and (for a draft) a reason and a suggested completion for each note. The server checks each item alone and writes the good ones to AI files. A bad item leaves only that note pending.

**Out of scope**:

* The `ai run` command, limits, lock and back-off (next slice)
* Digest
* Any UI
* Changing `ai enrich <id>` (it keeps working as today)

**Requirement Refs**:

* FR-047 (Path B, batch)
* FR-037: child process rules (`--tools ""`, no API key in env, timeout, masked stdin)
* FR-050: overrides win; one log line per applied item
* NFR-012, NFR-014, NFR-015
* US-020

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [ ] Restated intent confirmed to match the user's request
- [ ] Domain terms align with `PROJECT_SPEC.md` glossary
- [ ] Every Acceptance Criterion below traces to a line in the Requirement
- [ ] All Requirement Refs exist in `PRD.md` and are covered

---

## Dependencies & Reachability

**Depends on**: HUN-292 (AI file keys, overrides, log), HUN-293 (`[ai]` keys)

**Entry point**: `surfaces::organize::run_batch(store, settings, ids)`; pure parts in `core::organize` (prompt, schema, answer check)

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | The prompt holds the current project and topic lists (names + counts, at most 50 of each, most used first) and the batch's notes: masked, each cut to 1,500 chars, the whole prompt at most 12,000 chars (notes beyond the cap stay pending) | FR-047 |
| 2 | The call reuses the FR-037 child: `claude -p --output-format json --json-schema <array schema> --tools "" --strict-mcp-config --no-session-persistence --model <model>`, `ANTHROPIC_API_KEY` / `ANTHROPIC_AUTH_TOKEN` removed, `CLAUDE_CONFIG_DIR` from config, the timeout scaled by batch size (timeout_secs × min(batch, 5)) | FR-037, FR-047 |
| 3 | The answer is a JSON array with one item per note id. Each item is checked alone: id was in the batch and appears once, caps and kinds valid, strings masked, `based_on` still equals the note's hash. A bad item is skipped with a reason; the rest are written | FR-047 |
| 4 | A failed call, a timeout, invalid JSON, or an answer that fails the array schema writes nothing and leaves the whole batch pending; the result says which | FR-047 |
| 5 | `project` = the note's `repo` when it has one (the model's value is ignored for that note); otherwise the model's value | FR-047 |
| 6 | A field with a person's override is not changed; the item's other fields are written; the skipped field is reported | FR-050 |
| 7 | Each applied field change is one `ai-log.jsonl` line (`who: ai`), so `ai undo` works per note | FR-050 |
| 8 | A note that changed while the call ran (hash differs at write time) is skipped and stays pending | FR-047 |
| 9 | Fixture test: raw note bytes identical after a batch; a planted secret never reaches the child's stdin, an AI file or the log | NFR-015, NFR-002 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | fake `claude` returns a good array for 3 notes | 3 AI files with the new keys, 3+ log lines | tests/organize.rs (fake claude script) |
| 2 | fake returns 2 items, one with kind `lunch` | 1 written, 1 pending with a reason | same |
| 3 | fake exits 1 / prints junk / sleeps past the timeout | nothing written, batch pending | same |
| 4 | note edited between read and write | that note skipped | same |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-294.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-294.md` (review sections are never taken from the board).

## Approach

**Pattern reference**: `src/surfaces/enrich.rs` (`schema`, `argv`, `child_env`, the child spawn and timeout, the `Run AI` write path) and its fake-`claude` tests in `tests/enrich.rs`.

**Vital slice**: Prompt builder, array schema, per-item checker, writer; the process code is called from `enrich.rs`, not copied.

**Cut list**: Retrying a failed batch inside one run; streaming output; model choice per job.

---

## Edge Case Checklist

- [ ] The model returns an id not in the batch: dropped
- [ ] Two items for one id, or one item for two ids: both dropped
- [ ] The array is shorter than the batch: the rest stay pending
- [ ] Valid JSON but the schema check fails: the batch stays pending, one log line, no partial write
- [ ] A 20,000-char note: cut at 1,500 chars in the prompt, the AI file still based on the full text hash
- [ ] A note whose text is only `[masked:…]` markers
- [ ] Topic or project containing a path or a secret: masked and capped before the write
- [ ] The child writes a huge answer: read at most 1 MiB
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README, RUNBOOK, `docs/`, CLI help for text this change makes wrong

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| new `src/core/organize.rs` | prompt builder, array schema, item checker |
| new `src/surfaces/organize.rs` | run one batch, write results |
| `src/surfaces/enrich.rs` | expose the child spawn for reuse (no behavior change) |
| `src/core/mod.rs`, `src/surfaces/mod.rs` | module lines |
| new `tests/organize.rs`, fake `claude` fixture | AC 1–9 |
| `docs/NOTE_FORMAT.md` | AI file keys written by the batch |
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

A fake `claude` shell script that prints canned JSON, junk, or sleeps; temp notes dir; table-driven item checker tests; the byte-compare fixture and the leak suite.

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

* 2026-11-25 DECIDED scope and order: owner, chat 2026-11-25 ("let go")