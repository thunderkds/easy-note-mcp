**Date**: 2026-11-19
**Complexity Level**: C2
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md` (glossary: *Note*, *Notes dir*)
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read `agents/backend.md`
5. Note the **Complexity Level** above and apply the matching process from the Complexity matrix in `agents/backend.md`
6. Read `docs/ddr/0006-obsidian-as-the-dashboard.md`, `docs/ddr/0008-register-the-obsidian-vault.md`, `docs/ddr/0003-claude-code-capture-hook.md` (exact-file install rules), PRD FR-034, FR-035, FR-039, FR-040, FR-041, NFR-003
7. Tests only: temp `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR`; never open the real `~/easy-notes/`, `~/.config/obsidian/`, `~/.config/easy-note/` or `~/.local/share/`

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner, chat 2026-11-19, after the dashboard opened Obsidian but not the notes (Obsidian's only vault was a new `~/easy-notes/notes` folder, so `~/easy-notes/easy-note.base` was outside every vault): "can we setup this one by script, cause I think that, we can not require the user do the tech jobs". Owner picks: when Obsidian is open, ask to close it and write nothing; scope is one `easy-note setup` command.

**Restated intent** (Supervisor's interpretation, in the project's domain language):

> A person runs one command, `easy-note setup`, and the launcher, the tray login item, the dashboard file and the Obsidian vault are all in place: the launcher click opens the dashboard with the notes, with no clicks inside Obsidian.

**Out of scope** (what this task explicitly does NOT do):

* `install claude` inside `setup` (it edits Claude Code's settings.json and shows a diff; a developer step, stays separate)
* Removing or changing any vault already in Obsidian's list (for example the stray `~/easy-notes/notes` vault)
* Closing or starting Obsidian
* A .deb `postinst` (runs as root, not per user)
* Flatpak vault list; macOS/Windows

**Requirement Refs** (FR/NFR/US IDs from `PRD.md` this task satisfies):

* FR-040: `install obsidian` registers the notes dir as an Obsidian vault
* FR-041: `easy-note setup`
* FR-034: `dashboard` hint when the vault is not registered
* NFR-003: new write target (Obsidian's vault list), DDR 0008
* US-017

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request (by Supervisor / user — not the implementing agent) — Supervisor 2026-11-19, owner answered the two forced choices
- [X] Domain terms align with `PROJECT_SPEC.md` glossary (`grill-with-docs` run if terminology was fuzzy) — Supervisor 2026-11-19
- [X] Every Acceptance Criterion below traces to a line in the Requirement — Supervisor 2026-11-19
- [X] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above — Supervisor 2026-11-19

---

## Dependencies & Reachability

**Depends on**: HUN-273 (merged), HUN-276 (merged)

**Entry point**: `setup` (new `Command::Setup` in `src/surfaces/cli.rs`); `install obsidian` (`install::obsidian`, extended); `dashboard` (`dashboard::run`, one new hint)

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | Vault list path: `$XDG_CONFIG_HOME/obsidian/obsidian.json`, else `$HOME/.config/obsidian/obsidian.json` (native and snap builds use this path). Pure function, unit-tested | FR-040 |
| 2 | `install obsidian`, Obsidian not open: after writing `easy-note.base` (FR-035, unchanged), it creates `<notes dir>/.obsidian/` if missing (empty dir; Obsidian fills it) and adds `{"path": "<absolute notes dir>", "ts": <now ms>, "open": true}` under a new random 16-hex-char key in `vaults`, keeping every other key and vault byte-for-byte equal in value (parse → insert → write, `serde_json` with `preserve_order`), written atomically with the old file's mode; a copy of the old file is kept as a backup, the same helper as `install claude`'s settings.json backup. Prints `✓ added <notes dir> to Obsidian's vaults` | FR-040, DDR 0008 |
| 3 | Already covered: when any vault's `path` equals the notes dir or is a parent of it (path-component compare, after `std::path::absolute`), nothing is written to obsidian.json and it prints `already an Obsidian vault (<vault path>)`. A vault *inside* the notes dir (like `~/easy-notes/notes`) does not count | FR-040 |
| 4 | Obsidian open: when a process named `obsidian` runs (`/proc/*/comm` equals `obsidian`, read by a pure function over a list of names, unit-tested) and the vault is not yet covered, nothing is written to obsidian.json and stderr says `Close Obsidian, then run easy-note setup again` (exit 1). The `.base` file step still runs first | FR-040, owner pick |
| 5 | Obsidian never started: no obsidian.json and no `obsidian` scheme handler (`dashboard::choose` input) → skip the vault step with one line `Obsidian is not installed: the vault step is skipped`, exit 0. Handler present but no obsidian.json → create the file with `{"vaults":{…ours…}}` (dir mode 0700 if created) | FR-040 |
| 6 | Bad obsidian.json (not JSON, or `vaults` not an object): write nothing, stderr names the file and says it was left as it is, exit 1. The error text never quotes file content | FR-040 |
| 7 | `uninstall obsidian` removes only `easy-note.base` as today; it never edits obsidian.json and never removes `.obsidian/` (the vault is the person's now). README says so | FR-040, DDR 0008 |
| 8 | `easy-note setup` runs, in order, `install launcher`, `install obsidian`, and `install tray` (only in a `tray` feature build; otherwise one line `tray not built in: skipped`). Each step prints its own output under a `launcher:` / `obsidian:` / `tray:` heading; a failing step does not stop the next; exit 1 if any step failed, else 0. A second run writes nothing and every step says already installed. `setup` never runs `install claude`; its last line suggests `easy-note install claude` for Claude Code users | FR-041 |
| 9 | `dashboard` with the base file present and a handler, but the notes dir not covered by any vault in obsidian.json (when that file exists): stderr one line `the notes dir is not an Obsidian vault yet: run easy-note setup`, then still opens the URI as today (exit 0) | FR-034 |
| 10 | `starts_mcp(&Command::Setup)` is false; `every_command` lists `setup`; the help text lists `setup` first among the setup commands; no command other than `install obsidian` / `setup` writes obsidian.json or `.obsidian/` (fs snapshot like the launcher test) | NFR-003 |
| 11 | The store still lists the same notes and `stats.skipped` is unchanged with `.obsidian/` present in the notes dir | NOTE_FORMAT § Location |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

### Success Criteria (observable, pass/fail)

| \# | Given (input/state) | Expect (output/behavior) | How it's checked |
| -- | -- | -- | -- |
| 1 | temp HOME, fake `xdg-mime` prints `obsidian.desktop`, obsidian.json with one other vault + an extra top-level key | `install obsidian` adds one vault, keeps the other vault and key, writes a backup, makes `.obsidian/`; second run writes nothing | `tests/install.rs` or new `tests/setup.rs` |
| 2 | obsidian.json vault = parent of notes dir | `already an Obsidian vault`, file unchanged (bytes) | same |
| 3 | obsidian.json vault = `<notes dir>/notes` (child) | ours is added | same |
| 4 | fake process list contains `obsidian` (pure fn) / integration via an env seam only if needed | no write, the close-Obsidian line, exit 1 | unit + same |
| 5 | no handler, no obsidian.json | vault step skipped, exit 0, no obsidian dir created | same |
| 6 | obsidian.json = `not json` | unchanged, error names the file, exit 1 | same |
| 7 | `setup` twice in a temp HOME (both feature sets) | first run writes launcher, icons, base, vault (+tray file in tray build); second run writes nothing | same |
| 8 | `dashboard`, base present, handler, obsidian.json without our dir | the hint line on stderr, fake `xdg-open` still called with the URI | `tests/dashboard.rs` |

### Verification Command (exact, runnable)

```bash
cargo fmt --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --locked \
  && cargo clippy --all-targets --locked --features tray -- -D warnings \
  && cargo test --locked --features tray
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** See `tasks/TASK_REVIEW_HUN-277.md` (review sections are never taken from the board).

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-277.md` (review sections are never taken from the board).

## UI / Design Acceptance Criteria

> No UI of ours: Obsidian draws the window (DDR 0006). The three design Evidence rows are N/A with this reason; the owner-run check is the UX row below.

### 1. Visual Regression

| Screen / Component | Verification method | Expected result |
| -- | -- | -- |
| N/A | N/A | N/A — terminal output only; Obsidian's window is not ours |

### 2. Design-System Compliance

| Criterion | Verification method | Expected result |
| -- | -- | -- |
| N/A | N/A | N/A — no UI of ours |

### 3. Layout / Responsiveness

| Viewport | Verification method | Expected result |
| -- | -- | -- |
| N/A | N/A | N/A — no UI of ours |

### 4. UX Acceptance Criteria

| User story | User goal | Flow / state | Pass condition (observable) |
| -- | -- | -- | -- |
| US-017 | set up with no tech steps | Obsidian closed, `easy-note setup`, then click the launcher | Obsidian opens the dashboard with the notes (owner-run on the real machine) |
| US-017 | Obsidian is open | `easy-note setup` | one line says to close Obsidian and run it again; nothing in Obsidian's list changed |

---

## Approach

**Pattern reference**: `src/surfaces/install.rs` — `claude()` (read + check everything before writing, `write_backup`, `write_atomic` with the old mode), `own_file` (exact-file install), `launcher()` / `tray()`; `src/surfaces/dashboard.rs` — `choose()`, handler query; `src/surfaces/cli.rs` — `Target`, `starts_mcp`, `every_command`.

**Vital slice**: vault registration inside `install obsidian` (AC 2–6) plus `setup` calling the three existing install functions (AC 8).

* New `src/surfaces/obsidian_vault.rs` (or a section in `dashboard.rs`; pick the smaller diff): `vault_list_path()`, `covered(vaults, notes_dir) -> Option<PathBuf>`, `obsidian_running(comms: &[String]) -> bool`, `register(...)`.
* Random key: 8 bytes from `/dev/urandom` (or `getrandom` if already in the tree; do not add a crate for it), hex.
* `setup`: a function in `install.rs` (or `cli.rs`) that calls `launcher(Action::Install)`, `obsidian(Action::Install)`, `tray(Action::Install)` under `#[cfg(feature = "tray")]`.

**Cut list**: `uninstall` of the vault entry; Flatpak `~/.var/app/md.obsidian.Obsidian/config/obsidian/`; closing/reopening Obsidian; `install claude` in `setup`; a .deb postinst; a `--yes`/`--dry-run` flag.

---

## Edge Case Checklist

- [ ] Notes dir with spaces / non-ASCII → stored as the plain absolute path in JSON (serde escapes)
- [ ] `vaults` missing in an existing obsidian.json → add the object, keep other keys
- [ ] Notes dir path given relative or via `~` in config → absolute before compare and write
- [ ] Trailing slash on a stored vault path → compare by components
- [ ] obsidian.json is a symlink → write to its target (same as the settings.json rule in `claude()`)
- [ ] Key clash with an existing vault id → draw again
- [ ] Pin-test pre-flight: the `Pre-existing pin tests` row below is filled before work starts
- [ ] Stale references: README "Open the notes folder as a vault in Obsidian once" → now `easy-note setup`

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/obsidian_vault.rs` (new) or `dashboard.rs` | vault list path, cover check, running check, register |
| `src/surfaces/install.rs` | `obsidian()` calls register; `pub fn setup()` |
| `src/surfaces/cli.rs` | `Command::Setup`, `starts_mcp`, `every_command`, help order |
| `src/surfaces/dashboard.rs` | AC 9 hint |
| `src/surfaces/mod.rs` | module line if a new file |
| `tests/setup.rs` (new), `tests/install.rs`, `tests/dashboard.rs` | AC 1–11 |
| `README.md` | "Set up" section: `easy-note setup` first; vault rule; uninstall keeps the vault |
| Pre-existing pin tests | `tests/cli.rs` help/command list, `tests/install.rs` obsidian snapshots, `tests/readme.rs` — list each and how it is handled |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `PRD.md`, `PROJECT_SPEC.md`, `PROJECT_KANBAN.md`, `docs/ddr/*` | Supervisor-owned (FR-040, FR-041, DDR 0008 are written by the Supervisor before the spawn) |
| `memory/*`, `AGENTS.md`, `CLAUDE.md`, `agents/`, `skills/`, `.claude/`, `.github/workflows/*` | Kit / CI files |
| `src/core/*` | Only reads `config::notes_dir` |
| `src/surfaces/tray.rs` | Not needed |

---

## Test Plan

`cargo test` (both feature sets) with temp `HOME`, `XDG_CONFIG_HOME`, `EASY_NOTE_DIR` and fake `xdg-mime`/`xdg-open` on a temp `PATH` (pattern: `tests/dashboard.rs`). The running-Obsidian check is a pure function over names; if an integration seam is needed, use an env var read only under `cfg(test)`-free code path documented in the test, never a real Obsidian. Owner-only: run `easy-note setup` on the real machine with Obsidian closed, click the launcher.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Lint passes
- [ ] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_HUN-277.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — feature confirmed working in the real CLI
- [ ] Supervisor notified: task ready for Stage 4 review

## Questions Log

* 2026-11-19 DECIDED Obsidian open → ask to close it, write nothing (owner, forced choice)
* 2026-11-19 DECIDED scope is one `easy-note setup` command (owner, forced choice)
* 2026-11-19 DECIDED `setup` excludes `install claude`; `uninstall obsidian` never edits the vault list (Supervisor; DDR 0008)