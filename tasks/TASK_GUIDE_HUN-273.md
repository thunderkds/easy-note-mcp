**Date**: 2026-11-19
**Complexity Level**: C1
**Risk Level**: Low
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*, *Context*, *Guard*)
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read `agents/backend.md`
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in `agents/backend.md`
6. Read `docs/ddr/0006-obsidian-as-the-dashboard.md`, `docs/design/README.md` and `docs/NOTE_FORMAT.md` (§ Location, § Front matter keys)
7. Tests only: temp `HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/easy-note/` or `~/.local/share/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-19: "refer to this design, and update the application … application can open the dashboard, not only the command, we should support the native way." Then: "how about apply the obsidian as brain" and "let create the tasks and start the parallel \[admin\]". The design's dashboard (`docs/design/Main.dc.html`) is served by Obsidian (DDR 0006).

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> One command (and later one click in the launcher and tray) opens a dashboard of the notes dir in Obsidian: notes by day, project, task, tag and source, with no window of easy-note's own.

**Out of scope** (what this task explicitly does NOT do):

* The launcher `Exec=` / icon and the tray "Open dashboard" item (FR-039, a separate task)
* AI detail views in the vault (FR-036, a separate task)
* Creating an Obsidian vault or editing `.obsidian/` config
* macOS `open` / Windows

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-034: `easy-note dashboard`
* FR-035: `install obsidian` / `uninstall obsidian`, the `easy-note.base` file
* NFR-001: `xdg-open` and `xdg-mime query` in `dashboard` only
* NFR-003: the `.base` file is inside the notes dir
* US-017

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent) — auto (Supervisor) 2026-11-19
- [X] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy) — auto (Supervisor) 2026-11-19
- [X] Every Acceptance Criterion below traces to a line in the Requirement — auto (Supervisor) 2026-11-19
- [X] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above — auto (Supervisor) 2026-11-19

---

## Dependencies & Reachability

**Depends on**: None

**Entry point**: `dashboard` (subcommand `Command::Dashboard` in `src/surfaces/cli.rs`); `install obsidian` / `uninstall obsidian` (`Target::Obsidian`, handled by `install::obsidian`)

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | `install obsidian` writes exactly `<notes dir>/easy-note.base` (creates the notes dir if missing, like the first write does); prints the path; a second run says `already installed` and writes nothing; a file there that is not byte-identical to ours is kept and named, exit 1. `uninstall obsidian` removes it only when byte-identical. Same rules and helper as `install launcher` (DDR 0003) | FR-035 |
| 2 | The `.base` file is valid Obsidian Bases YAML: a global filter limiting to notes (`file.folder` matches `^\d{4}/\d{2}$` or the property `easy-note` exists), and at least these views, named as in the design: `Today` (created today, newest first), `All notes` (table: created, first line/`file.name`, repo, task, tags, source), `By project` (grouped by `repo`), `By task` (grouped by `task`), `By tag`. A unit test parses it with `saphyr` and checks the view names and that every property it names is a key of `docs/NOTE_FORMAT.md` | FR-035 |
| 3 | `dashboard` with the file present and an `obsidian` scheme handler: runs `xdg-open obsidian://open?path=<percent-encoded absolute path of easy-note.base>` as an argv list, exit 0. The URI is built by a pure function `obsidian_uri(path) -> String` (unit-tested with spaces and non-ASCII in the path) | FR-034, NFR-001 |
| 4 | `dashboard` with no `easy-note.base`: stderr `no dashboard yet: run easy-note install obsidian`, exit 1, nothing started | FR-034 |
| 5 | `dashboard` with the file present but no handler (`xdg-mime query default x-scheme-handler/obsidian` prints nothing or `xdg-mime` is missing): stderr says Obsidian is not installed, then `xdg-open <notes dir>`, exit 0. Decision is a pure function of the handler string (unit test) | FR-034 |
| 6 | `starts_mcp(&Command::Dashboard)` is false; `every_command` lists `dashboard` and `install obsidian`; no other command writes `easy-note.base` (fs snapshot like the launcher test) | NFR-003 |
| 7 | The store still lists the same notes and `stats.skipped` is unchanged when `easy-note.base` is present (it is not a note) | FR-035, NOTE_FORMAT § Location |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | temp notes dir, `install obsidian` twice, then `uninstall obsidian` | file written, then `already installed`, then removed; nothing else changed | `tests/install.rs` |
| 2 | foreign `easy-note.base` | kept, named on stderr, exit 1 | `tests/install.rs` |
| 3 | `PATH` with fake `xdg-mime` (prints `obsidian.desktop`) and fake `xdg-open` (records argv) | `xdg-open` got one arg `obsidian://open?path=…easy-note.base` | `tests/cli.rs` or new `tests/dashboard.rs` |
| 4 | fake `xdg-mime` prints nothing | stderr names Obsidian; `xdg-open` got the notes dir | same |
| 5 | no `.base` file | exit 1, the install hint, fake `xdg-open` not called | same |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features tray -- -D warnings \
  && cargo test --locked --features tray
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-273.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-273.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

> The dashboard window is Obsidian's, not ours (DDR 0006). The only UI this task owns is the content of the `.base` views. Screenshot of the real Obsidian window is the owner's (AC owner-run); the agent proves the views by the parse test (AC 2).

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| `easy-note.base` in Obsidian | owner screenshot, read with LLM vision | views Today, All notes, By project, By task, By tag show the temp notes |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| Colors match design tokens | N/A | N/A — Obsidian's theme draws the view |
| Typography matches spec | N/A | N/A — Obsidian's theme |
| Spacing / layout matches spec | parse test (AC 2) | view names and groupings match `docs/design/Main.dc.html` (day groups, by project) |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| Mobile (320–480px) | N/A | N/A — Obsidian mobile draws it; easy-note is Linux only (NFR-006) |
| Tablet (768px) | N/A | N/A |
| Desktop (1024px+) | owner screenshot | Bases table readable |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-017 | open the dashboard | `easy-note dashboard` with Obsidian installed | Obsidian opens `easy-note.base` |
| US-017 | not set up yet | `easy-note dashboard` before `install obsidian` | one line says to run `easy-note install obsidian` |
| US-017 | no Obsidian | `easy-note dashboard`, no handler | one line says Obsidian is not installed; the notes folder opens |

---

## Approach

**Pattern reference**: `src/surfaces/install.rs` — `launcher()` / `tray()` with `own_desktop_file(rel, content, action)` (exact-file install/uninstall); `src/surfaces/tray.rs` — `argv()` and spawning `xdg-open` with argv lists; `src/surfaces/cli.rs` — `Target`, `starts_mcp`, `every_command`.

**Vital slice**: `install obsidian` writing a correct `.base` file plus `dashboard` opening it; the no-handler fallback is one branch.

* New `src/surfaces/dashboard.rs`: `const BASE_FILE: &str` (the YAML, kept in code like `LAUNCHER_DESKTOP`), `obsidian_uri(path)`, `pub fn run() -> Result<(), String>`, pure `choose(handler: &str, base_exists: bool) -> Plan`.
* `install.rs`: `pub fn obsidian(action)` on `<notes dir>/easy-note.base`; generalise `own_desktop_file` to take the absolute path only if needed (keep the launcher/tray call sites unchanged in behaviour).
* `percent-encoding` is already a dependency; use it for the `path` query value.

**Cut list**: a `--vault` flag; writing `.obsidian/` config or bookmarks; macOS `open`; a Bases view of AI detail (FR-036's task adds it if needed).

---

## Edge Case Checklist

- [ ] Notes dir path with spaces or non-ASCII → URI percent-encoded
- [ ] Notes dir not inside any Obsidian vault → Obsidian opens the file anyway or shows its own error; README says to open the notes dir as a vault (or a parent folder)
- [ ] `xdg-open` missing → stderr line, exit 1
- [ ] `easy-note.base` hand-edited by the person → `uninstall` keeps it and names it
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: grep README and `docs/` for "No main application window" / "no dashboard"

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/dashboard.rs` | new: `.base` text, URI, `run()` |
| `src/surfaces/mod.rs` | `pub mod dashboard` |
| `src/surfaces/install.rs` | `pub fn obsidian(action)` |
| `src/surfaces/cli.rs` | `Command::Dashboard`, `Target::Obsidian`, `installed()`, `starts_mcp`, `every_command` |
| `tests/install.rs`, `tests/dashboard.rs` (new) | AC 1–7 |
| `README.md` | "Obsidian dashboard" section |
| Pre-existing pin tests | `tests/cli.rs` help/command list, `tests/install.rs` snapshots, `tests/readme.rs` — list each and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*` | Supervisor-owned |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| `src/core/*` | The dashboard only reads the notes dir path (`config::notes_dir`) |
| `src/surfaces/tray.rs`, `LAUNCHER_DESKTOP` in `install.rs` | FR-039 task owns them (parallel task; avoid merge conflicts) |

---

## Test Plan

`cargo test` (both feature sets) with temp `HOME`/`EASY_NOTE_DIR` and fake `xdg-open`/`xdg-mime` scripts on a temp `PATH` (pattern: `tests/quick.rs` fakes for `zenity`). Owner-only: open the real dashboard in Obsidian and paste a screenshot.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_<ID>.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-19 DECIDED the dashboard is an Obsidian Bases file, not a window of ours (owner, chat; DDR 0006) — RULE: `docs/ddr/0006-obsidian-as-the-dashboard.md` § Decision
* 2026-11-19 DECIDED the `.base` text lives in code as a const, like `LAUNCHER_DESKTOP` (Supervisor, RULE: `src/surfaces/install.rs` pattern)