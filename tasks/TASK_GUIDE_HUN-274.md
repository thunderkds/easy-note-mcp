**Date**: 2026-11-19
**Complexity Level**: C1
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*, *Guard*, *Trash*, *AI file*)
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read `agents/backend.md`
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in `agents/backend.md`
6. Read `docs/ddr/0004-ai-detail-in-separate-files.md`, `docs/ddr/0006-obsidian-as-the-dashboard.md`, `docs/NOTE_FORMAT.md` (AI file section) and `src/core/ai.rs`
7. Tests only: temp `HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-19: "how about apply the obsidian as brain", then "with the AI detail the note, …". The Supervisor's reply named the gap: Obsidian does not index `<notes dir>/.easy-note/`, so AI detail is invisible there. DDR 0006 item 3 and PRD FR-036 close it.

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> When the notes dir is in an Obsidian vault, each note's AI detail also shows in Obsidian as a separate, clearly labelled, read-only page linked to the note, and the note file is never touched.

**Out of scope** (what this task explicitly does NOT do):

* Any change to note files or to the JSON AI file shape (DDR 0002, DDR 0004)
* Reading a view back (views are output only)
* The `.base` dashboard (FR-035 task) and `ai enrich` (FR-037 task)

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-036: views, `ai render`, rm/restore/trash handling
* FR-032: label `AI detail (not your words)` with model and date
* NFR-002: no secret in any file written (views are masked)
* NFR-012: AI detail never changes a note file
* US-017, US-016

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent) — auto (Supervisor) 2026-11-19
- [X] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy) — auto (Supervisor) 2026-11-19
- [X] Every Acceptance Criterion below traces to a line in the Requirement — auto (Supervisor) 2026-11-19
- [X] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above — auto (Supervisor) 2026-11-19

---

## Dependencies & Reachability

**Depends on**: None

**Entry point**: `ai render` (`AiAction::Render` in `src/surfaces/cli.rs`); `ai_view::write_view` called from `ai::enrich` and `ai::capture` in `src/core/ai.rs`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `vault_root(notes_dir) -> Option<PathBuf>`: the nearest of the notes dir and its parents holding a `.obsidian` directory (stop at `/`; a `.obsidian` file does not count) | FR-036 |
| 2 | In a vault, every AI file write (`ai::enrich`, `ai::capture`, so also `note_enrich` and `note_add(detail)`) also writes `<notes dir>/ai/<id>.md` atomically (temp + rename, like the AI file). Outside a vault nothing is written under `ai/`, and no `ai/` dir is created | FR-036 |
| 3 | View shape: front matter `easy-note-ai-view: 1`, `note: "[[<id>]]"`, `model`, `at`, `kind`, `stale` (bool), `tags_suggested` (list); body `# AI detail (not your words)`, then the title, the summary, `Suggested tags: #a #b`, and `Accept with: easy-note tag <id> --accept-ai`. Every string goes through the guard mask; front matter strings JSON-quoted like NOTE_FORMAT | FR-036, FR-032, NFR-002 |
| 4 | A view file is not a note: the store lists the same notes, `stats.skipped` is unchanged, search does not return it | FR-036, NOTE_FORMAT § Location |
| 5 | `ai rm <id>` and `rm <id>` delete the view; `restore <id>` re-renders it from the restored AI file; `trash --empty` leaves no view of a gone note | FR-036 |
| 6 | `edit <id>` and `tag <id>` re-render an existing view so `stale` is true after a text edit (stale = `based_on` ≠ sha256 of the note text, as `ai::read` computes) | FR-036 |
| 7 | `ai render` rewrites every view from the AI files (backfill for detail written before this task, or after the vault was created) and prints `N views written`; outside a vault it says `not in an Obsidian vault (no .obsidian folder found)`, exit 1, writes nothing | FR-036 |
| 8 | Every note file is byte-identical before and after AC 2–7 (snapshot test) | NFR-012 |
| 9 | The view writer is registered in `tests/leak.rs` `SURFACES` (AI detail holding the planted secret) and `planted_secret_never_leaks` passes | NFR-002 |
| 10 | A view write error never fails the AI write: the JSON file is written, the error goes to stderr once | FR-036 (robustness) |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | temp vault (`.obsidian/` in a parent), note + `note_enrich` | `ai/<id>.md` with the AC 3 shape | `tests/ai.rs` |
| 2 | no `.obsidian` anywhere | no `ai/` dir | `tests/ai.rs` |
| 3 | `edit` the note text | view `stale: true` | `tests/ai.rs` |
| 4 | `rm`, `restore`, `trash --empty` | view gone, back, gone | `tests/ai.rs` |
| 5 | AI summary with planted secret | view masked | `tests/leak.rs` |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features tray -- -D warnings \
  && cargo test --locked --features tray
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-274.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-274.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

> No UI of ours: the view is a Markdown file that Obsidian renders. All three UI Evidence rows are ☐ N/A — Markdown output checked by tests (AC 3); the owner may paste an Obsidian screenshot.

---

## Approach

**Pattern reference**: `src/core/ai.rs` — the AI file writer (temp file, fsync, rename; mask on write); `src/core/format.rs` — JSON-quoted front matter rendering; `src/core/store.rs` `rm_with_ai` / `restore_with_ai` (`AiCarry`).

**Vital slice**: write the view on every AI write in a vault (AC 2–3); `ai render` and the rm/restore/edit hooks are small.

* New `src/core/ai_view.rs`: `vault_root`, `render(detail, note_id, stale) -> String` (pure), `write_view(store, id)`, `remove_view(store, id)`, `render_all(store) -> Result<usize, _>`.
* Call sites: end of `ai::enrich` / `ai::capture`; `ai::remove`; `cli/write.rs` `rm`, `restore`, `edit`, `tag`; trash empty. Keep each call one line.
* `stale` comes from the same `based_on` comparison `ai::read` uses; do not duplicate it — call `ai::read`.

**Cut list**: a Bases view over `ai/` (the dashboard task may add it); watching for `.obsidian` created later (use `ai render`); a config switch.

---

## Edge Case Checklist

- [ ] `.obsidian` in a parent several levels up (vault root above the notes dir) → still a vault
- [ ] Notes dir is the vault root and `ai/` already holds the person's own files → only `<id>.md` names of existing AI files are written or removed; nothing else in `ai/` is touched
- [ ] Note ID with a clash suffix (`-2`) → view name matches
- [ ] Inbox notes (`inbox/` prefix ids) → view name safe (no `/`), documented
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep `docs/NOTE_FORMAT.md` and README for "Files under `.easy-note/`" and add the `ai/` view as a non-note

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/core/ai_view.rs` (new), `src/core/mod.rs` | view writer |
| `src/core/ai.rs` | call `write_view` / `remove_view` |
| `src/surfaces/cli.rs` | `AiAction::Render` |
| `src/surfaces/cli/write.rs` | rm / restore / edit / tag / trash refresh calls |
| `tests/ai.rs`, `tests/leak.rs` | AC tests, leak surface |
| `docs/NOTE_FORMAT.md` | one section: the `ai/<id>.md` view (not a note, not read back) |
| `README.md` | "AI detail in Obsidian" paragraph |
| Pre-existing pin tests | `tests/ai.rs`, `tests/cli.rs` help text, `tests/readme.rs`, `tests/format.rs` — list each and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*` | Supervisor-owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| `src/core/format.rs` note render/parse, any note file | NFR-012, DDR 0002 |
| `src/surfaces/install.rs`, `src/surfaces/tray.rs`, `src/surfaces/dashboard.rs`, `src/core/config.rs` | Parallel tasks own them |

---

## Test Plan

`cargo test` (both feature sets): temp vault and non-vault notes dirs; view shape; lifecycle (rm/restore/edit/tag/trash); byte-identical note snapshot; leak suite.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Security review: `Skill({ skill: "security-review" })` run (Medium risk)
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-19 DEBATE views are written automatically when a `.obsidian` folder is found, not only after `install obsidian` — DEBATED: automatic (Advocate 1: automatic, PRD FR-036 says "when the notes dir is inside an Obsidian vault"; Advocate 2: only after `install obsidian`, so a vault user is not surprised by an `ai/` folder). Decided by RULE: `PRD.md` § Functional Requirements FR-036, cited first under `docs/claude-md/tracks.md` `### Admin tag` (domain first). Advocate 2's mitigation kept: `ai render` backfills ⚠ decided for you — please review
* 2026-11-19 DECIDED `stale` in the view is refreshed on `edit`/`tag` rather than dropped (Supervisor, RULE: `PRD.md` FR-036 lists `stale` in the front matter)