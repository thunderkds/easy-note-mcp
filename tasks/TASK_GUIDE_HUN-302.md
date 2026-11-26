**Date**: 2026-11-26
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P0
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`

---

## Mandatory Startup (Do Not Skip)

1. Read `PROJECT_SPEC.md`, this file, `agents/backend.md`, `memory/codebase-map.md`
2. Read `PRD.md` (FR-047, FR-050, NFR-002, NFR-015), `src/core/ai.rs`, `src/core/overrides.rs`, `docs/NOTE_FORMAT.md`
3. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; no test starts the real `claude` CLI

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-26, answer to the HUN-294 question: "A: log now, undo later". HUN-294 keeps the old title, summary and tags in `ai-log.jsonl`. This task makes `ai undo` restore them, before `ai run` ships.

**Restated intent**:

> `easy-note ai undo <id>` takes back every AI change of a note, also the title, summary, tags and draft keys. It never removes a person's override when the latest change was an AI change.

**Out of scope**:

* The `ai run` command
* Any window UI

**Requirement Refs**:

* NFR-015, FR-047, FR-050

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [x] Restated intent confirmed to match the user's request
- [x] Domain terms align with `PROJECT_SPEC.md` glossary
- [x] Every Acceptance Criterion below traces to a line in the Requirement
- [x] All Requirement Refs exist in `PRD.md` and are covered

Signed: auto (Supervisor), 2026-11-26.

---

## Dependencies & Reachability

**Depends on**: HUN-294 (old values in the log)

**Entry point**: `easy-note ai undo <id>`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `ai undo` restores the old title, summary and tags that a batch or `ai enrich` replaced, with the full old value | NFR-015 |
| 2 | `ai undo` restores cleared draft reason and suggestion | NFR-015 |
| 3 | The changes of one applied item are taken back together, in one call | FR-047 |
| 4 | An applied item that only created an AI file has a log line too, so undo never removes a person's override in its place | FR-047, FR-050 |
| 5 | Raw note files stay byte-identical through any number of runs and undos | NFR-015 |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | a stale AI file replaced by a batch | one undo returns the old title, summary, tags and kind | tests/overrides.rs |
| 2 | a new note with project and topic overridden, then a batch | undo changes the AI file and keeps both overrides | tests/overrides.rs |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-302.md` (review sections are never taken from the board).

## Approach

**Pattern reference**: `overrides::undo`, `ai::restore_field`, `log_ai_diff`.

**Vital slice**: Group the lines of one applied item; restore every field of the group.

**Cut list**: Redo; undo history view.

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/core/overrides.rs` | grouped undo |
| `src/core/ai.rs` | restore title, summary, tags and draft keys |
| `tests/overrides.rs` | AC 1 to 5 |
| `docs/NOTE_FORMAT.md` | undo rules |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| Note files and the note writer | The AI never changes a raw note |
| `src/core/guard.rs` patterns | Frozen |

## Questions Log

* 2026-11-26 DECIDED scope: owner chose "log now, undo later" for HUN-294
* 2026-11-26 DEBATE How should the log lines of one applied item be grouped for one undo? → an optional `op` id on each line of one item; lines without it stay single steps (Advocate 1: `op` id, one core apply, marker on creation; Advocate 2: `op` id, one core entry over the two writers, marker on creation) ⚠ decided for you — please review
* 2026-11-26 DEBATE What does the AI file look like after undo? → a file the AI created is removed; a replaced file comes back whole, with its old `based_on`, `at`, `model` and `organized_at` (Advocate 1: the same; Advocate 2: the same, plus a hold so `ai run` does not redo it) ⚠ decided for you — please review
