**Date**: 2026-11-26
**Complexity Level**: C1
**Risk Level**: Medium
**Priority**: P1
**Assigned agent**: Backend-Implementer
**Agent guide**: `agents/backend.md`
**Mode**: auto (`[admin]` session tag)

---

## Mandatory Startup (Do Not Skip)

Before writing any code:

1. Read `PROJECT_SPEC.md`
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file or decision the slice does not cover
3. Read this file completely
4. Read `agents/backend.md`
5. Read `src/surfaces/install.rs`, `setup()` in `src/surfaces/cli.rs`, `hotkey()` in `src/surfaces/quick.rs`, `tests/setup.rs`

---

## Requirement (Pillar 1 — Adapt the requirement)

User, 2026-11-26: "I using the control + alt + N but got no way to take new note", then "should add it to the guide, or attach it when install the application". The Supervisor proposed both (guide step, and `easy-note setup` adds the GNOME shortcut) and named the written rule it reverses; the user replied "yes".

**Restated intent** (Supervisor's interpretation):

> After a person installs easy-note and runs `easy-note setup` on GNOME, Ctrl+Alt+N opens `easy-note quick`, without losing any shortcut the person already has. The guide says this where the person installs.

**Out of scope**:

* Binding a key during the `.deb` install (runs as root; the package writes nothing below home).
* Writing shortcuts on KDE, macOS, Windows, sway or i3 (`easy-note hotkey` prints the steps).
* Choosing another key; changing the tray menu label.

**Requirement Refs** (added to `PRD.md` by this task):

* FR-054: `install hotkey` / `uninstall hotkey`, and the `hotkey` step of `setup`.
* FR-041 (amended): `setup` also runs the `hotkey` step.
* FR-025 (unchanged): `easy-note hotkey` writes nothing.

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [X] Restated intent confirmed to match the user's request (auto (Supervisor); the user answered "yes" to this proposal on 2026-11-26)
- [X] Domain terms align with `PROJECT_SPEC.md` glossary
- [X] Every Acceptance Criterion below traces to a line in the Requirement
- [X] All Requirement Refs exist in `PRD.md` (after the first commit of this task) and are covered by the Acceptance Criteria

---

## Dependencies & Reachability

**Depends on**: None
**Entry point**: `easy-note install hotkey`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | On GNOME (`XDG_CURRENT_DESKTOP` holds `gnome`) with no custom shortcut, `install hotkey` creates one entry at `/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/easy-note/` with name `easy-note quick`, command `easy-note quick`, binding `<Control><Alt>n`, and lists that path. The three keys are written before the list. It prints what it did. | Ctrl+Alt+N opens `quick` |
| 2 | Every other path already in the list stays in it, unchanged and in order. A list that cannot be read or parsed: nothing is written, exit 1. | "without losing any shortcut" |
| 3 | A second run writes nothing. A custom shortcut (any path) whose command runs `easy-note quick` already exists with a binding: nothing is written, and the output names the binding. This keeps a person's own key choice. Our listed entry with a cleared binding (turned off by the person) also stays. | never overwrite the person's choice |
| 4 | Ctrl+Alt+N already used by another shortcut (a custom entry, or any GNOME setting, in any modifier order or spelled `<Primary>`): nothing is written, the output names what uses it, exit 0. | "skip if already used" |
| 5 | Our path exists but holds a command that is not ours: nothing is written there; the output says it was kept. | never change what is not ours |
| 6 | `uninstall hotkey` removes our path from the list and resets its keys, only when its command is `easy-note quick`; other entries stay. Nothing ours: prints `not installed`, writes nothing. | "`uninstall` removes it" |
| 7 | Not GNOME: `install hotkey` and `uninstall hotkey` run no command, write nothing, tell the person to run `easy-note hotkey`, exit 0. | other desktops get steps |
| 8 | `setup` runs the step under a `hotkey:` heading. A failed `gsettings` call (missing or non-zero) fails only that step: later output still appears, exit 1, and `setup` names `hotkey` as failed. | "have `easy-note setup` add the shortcut" |
| 9 | `easy-note hotkey` still runs no command and writes nothing; its GNOME text also names `easy-note install hotkey`. | FR-025 stays |
| 10 | `README.md` says, in the install / "Set up" part, that `setup` adds Ctrl+Alt+N on GNOME, how to check it and how to remove it. "Bind a key" matches. `PRD.md` (FR-054, FR-041, NFR-003) and `docs/ddr/0003-claude-code-capture-hook.md` item 7 record the owner-approved change of 2026-11-26. | "add it to the guide" |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | AC 1-9 against a stub `gsettings` that keeps its state in a temp folder | the stated state and output | automated test (`tests/hotkey.rs` or `tests/setup.rs`) |
| 2 | A stub that fails, a malformed list, a foreign entry, a used key | no write | automated test |

### Verification Command (exact, runnable)

```bash
cargo test --locked --test hotkey --test setup
```

---

## Approach

**Pattern reference**: `src/surfaces/install.rs` (`tray`, `own_file`: act only on what is ours) and `tests/setup.rs` (cleared environment, stub tools on a temp `PATH`).
**Vital slice**: GNOME `install hotkey` / `uninstall hotkey`, called by `setup`.
**Cut list**: other desktops; a custom key option; `doctor` reporting the shortcut.

Run `gsettings` found on `PATH`, with argument lists (no shell). Add `Target::Hotkey` beside the other install targets.

**Safety rule for every test and agent**: a test that runs `setup` or `install hotkey` clears the environment and puts only stub tools on `PATH`. The real `gsettings` is never run: it writes the person's desktop settings through D-Bus even with a temp `HOME`.

---

## Edge Case Checklist

- [ ] Empty list is printed as `@as []`
- [ ] A list that already holds our path but whose keys are empty (half-done earlier run) is completed
- [ ] `gsettings` exits 0 when dconf cannot save: read each value back, and fail if it did not change
- [ ] An empty binding on another entry is not a conflict
- [ ] Binding compare ignores modifier order and case, and treats `<Primary>`, `<Ctrl>`, `<Control>` as one
- [ ] Existing `setup` tests pass: with no `XDG_CURRENT_DESKTOP` the step only prints
- [ ] Pin-test pre-flight: grep `tests/` for the `setup` output, README text and `hotkey` text before editing
- [ ] Stale references: grep README, `PRD.md`, `PROJECT_SPEC.md`, `docs/ddr/`, comments for "writes nothing" / "never written" / "changes nothing" about shortcuts

---

## Files to Change (Predicted)

| File | Change |
| -- | -- |
| `src/surfaces/install.rs` | the `hotkey` install / uninstall |
| `src/surfaces/cli.rs` | `Target::Hotkey`, the `setup` step, help text |
| `src/surfaces/quick.rs` | GNOME text names `install hotkey` |
| `tests/hotkey.rs`, `tests/setup.rs` | new tests; adjust pinned output |
| `README.md`, `PRD.md`, `docs/ddr/0003-claude-code-capture-hook.md` | guide step and the recorded decision |
| Pre-existing pin tests | `tests/hotkey.rs`, `tests/readme.rs`, `tests/setup.rs`, `tests/deb.rs`: check before editing |

## Files Must NOT Touch

| File | Reason |
| -- | -- |
| `scripts/build_deb.sh` | the package keeps no maintainer script |
| `.claude/`, `AGENTS.md`, `skills/` | kit files, not part of this task |

---

## Test Plan

Test first against a stub `gsettings`; then `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, and both with `--features window,tray`.

---

## Completion Checklist

- [ ] Implementation done
- [ ] Self-review: `Skill({ skill: "code-review" })` run
- [ ] Security review: `Skill({ skill: "security-review" })` run
- [ ] Lint passes
- [ ] Tests written AND pass — output in the review file
- [ ] Auto verify run