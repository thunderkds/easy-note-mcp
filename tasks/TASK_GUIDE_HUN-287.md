# TASK_GUIDE — HUN-287: Debian package — `scripts/build_deb.sh` builds an installable `.deb` (window + tray)
**Date**: 2026-11-21
**Complexity Level**: C2
**Risk Level**: Low
**Priority**: P2
**Assigned agent**: Common-Infrastructure-Agent (packaging: `scripts/build_deb.sh`, `tests/deb.rs`)
**Agent guide**: `agents/common-infrastructure.md`

> **Renumbering (owner approved, chat 2026-11-21).** The work was started under the ID `HUN-285`
> before a Linear issue existed. Linear gave `HUN-285` to the tray task, so this issue took the
> work over. The branch is `feat/HUN-287-deb` (renamed from `feat/HUN-285-deb`). The `(HUN-285)`
> mentions in `scripts/build_deb.sh` and `tests/deb.rs` now read `HUN-287`. The event trace of the
> session that wrote the code stays `memory/event-trace/HUN-285.jsonl`: that file holds the tray
> events too, and it is a hash-chained log, so it is not split.

---

## Mandatory Startup (Do Not Skip)

Before writing any code:
1. Read `PROJECT_SPEC.md`
2. Read the memory slice in your spawn prompt (`<!-- memory-slice -->`). Read `memory/MEMORY.md` in full only if your work reaches a file, hook, skill or decision the slice does not cover
3. Read this file completely
4. Read `agents/common-infrastructure.md`
5. Note the **Complexity Level** above (C2: plan before coding; `code-review` + `verify`)
6. Read `memory/codebase-map.md` for directory layout and entry points
7. **easy-memory first (saves tokens)**: call `mcp__easy-memory__memory_search` with `deb`, `package`, `install`, `HUN-287` before reading source files

---

## Requirement (Pillar 1 — Adapt the requirement)

Owner's request, as the board row states it:

> Debian package: scripts/build_deb.sh builds an installable .deb (window + tray)

**Restated intent** (Supervisor's interpretation, in the project's domain language):
> A person on Debian or Ubuntu installs easy-note with `apt` instead of `cargo install`, and gets the
> dashboard window and the tray icon in that one binary. The package touches no file below any home
> folder: each person still runs `easy-note setup` themselves.

**Out of scope** (what this task explicitly does NOT do):
- A repository, a PPA, a signed release, or any upload. The script writes a local file.
- An `n` symlink in the package (`cargo install` keeps that job).
- A maintainer script (`postinst`, `postrm`). The launcher, the tray and the Obsidian vault are per-person files, and a maintainer script runs as root.
- Any other package format (RPM, AUR, Flatpak, `.dmg`, `.msi`).
- A CI job that builds or publishes the package.

**Requirement Refs** (FR/NFR IDs from `PRD.md` this task satisfies):
- NFR-006: one Rust binary, Linux x86_64 in v1 — this task adds the `.deb` install path beside `cargo install`.
- NFR-003: writes only the files the PRD lists — the package itself writes nothing below a home folder.

### Requirement Fidelity Gate (sign off BEFORE implementation)

- [x] Restated intent confirmed to match the user's request — signed `auto (Supervisor)` (`docs/claude-md/tracks.md` `### Every user stop and its auto rule`, row 13)
- [x] Domain terms align with `PROJECT_SPEC.md` glossary (notes dir, setup, tray, dashboard window)
- [x] Every Acceptance Criterion below traces to a line in the Requirement
- [x] All Requirement Refs exist in `PRD.md` and are fully covered by the Acceptance Criteria above

---

## Dependencies & Reachability

**Depends on**: `None` — the script reads the repo and `Cargo.toml` only.

**Entry point**: `scripts/build_deb.sh`

---

## Acceptance Criteria

| # | Criterion (testable) | Traces to requirement |
|---|----------------------|-----------------------|
| 1 | `scripts/build_deb.sh` writes `dist/easy-note_<version>_<arch>.deb` and prints its path; `<version>` comes from `Cargo.toml`, `<arch>` from `dpkg --print-architecture` | "builds an installable .deb" |
| 2 | The package holds `/usr/bin/easy-note`, `/usr/share/doc/easy-note/README.md`, `.../copyright` and `.../changelog.gz`, and nothing else | "installable" (Debian policy), NFR-003 |
| 3 | The package ships no maintainer script, so the install writes nothing below any home folder | NFR-003 |
| 4 | The release build uses the features `window,tray` by default, and `--features LIST` overrides them | "(window + tray)" |
| 5 | `Depends` is read from the built binary with `dpkg-shlibdeps`, so a `window` build names the WebKitGTK library and a build without it does not | "installable" (apt resolves the libraries) |
| 6 | `EASY_NOTE_BIN=<path>` packages that prebuilt binary and runs no cargo build | testability of AC 1–5 |
| 7 | A missing `dpkg`, `dpkg-deb` or `dpkg-shlibdeps` prints which tool to install and exits 1, and writes no package | negative case |
| 8 | `README.md` and `RUNBOOK.md` name the build command, the `apt install` command and the `easy-note setup` step after the install | "installable" for a reader |

---

## Evaluation & Acceptance (How we know the agent worked correctly)

### Success Criteria (observable, pass/fail)

| # | Given (input/state) | Expect (output/behavior) | How it's checked |
|---|---------------------|--------------------------|------------------|
| 1 | `EASY_NOTE_BIN=/bin/true scripts/build_deb.sh --out <tmp>` | a `.deb` at `<tmp>/easy-note_0.1.0_<arch>.deb`; `dpkg-deb --contents` lists the four files and no more | automated test (`tests/deb.rs`) |
| 2 | the same package | `dpkg-deb --field <deb> Depends` is not empty, and `--field Package/Version/Architecture` match `Cargo.toml` and `dpkg --print-architecture` | automated test |
| 3 | the same package | `dpkg-deb --contents` lists no `DEBIAN/postinst`, `preinst`, `postrm` or `prerm` | automated test |
| 4 | `PATH` without `dpkg-deb` | exit 1, stderr names the tool and `sudo apt install dpkg-dev`, no file in `--out` | automated test |
| 5 | a machine with no `dpkg-deb` at all | the test prints `skipped: dpkg-deb is not on PATH` and passes | automated test (CI without dpkg) |

### Verification Command (exact, runnable)

```bash
cargo test --locked --test deb -- --nocapture
cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings
cargo test --locked                       # full suite, no regression
EASY_NOTE_BIN=/bin/true bash scripts/build_deb.sh --out /tmp/deb-ac && dpkg-deb --contents /tmp/deb-ac/easy-note_0.1.0_*.deb
```

### Evidence (filled by reviewer at Stage 4/5)

> **Moved.** Filled by the reviewer at Stage 4/5 in `tasks/TASK_REVIEW_HUN-287.md`.

---

## Demonstration

> **Moved.** See `tasks/TASK_REVIEW_HUN-287.md`.

---

## Approach

**Pattern reference**: `tests/install.rs` and `tests/setup.rs` — a shell-visible surface tested through its real commands in a temp dir, with the slow part replaced by a stand-in. `tests/deb.rs` packages `/bin/true` the same way, so it runs the real `dpkg-shlibdeps` and the real `dpkg-deb --build` in under a second.

**Vital slice**: one `dpkg-deb` staging script that produces a policy-clean package from a release build, plus the test that proves its contents.
**Cut list**:
- No `debian/` source package, no `debhelper`, no `dh_make` layout: the script stages a tree and calls `dpkg-deb --build`.
- No CI job: the owner builds the package by hand for now.
- No `n` symlink, no maintainer script, no repository or signature (see **Out of scope**).

A `.deb` only has to be a staged tree plus a `DEBIAN/control`, so the whole packaging fits in one
script with no new dependency. `dpkg-shlibdeps` reads `Depends` from the binary, so the features the
build used decide the dependency list, with nothing to keep in step by hand.

---

## Edge Case Checklist

- [x] `dpkg-shlibdeps` needs a `debian/control` beside its working directory: the script writes a throwaway one outside the package root, so it never lands in the package
- [x] Debian policy wants no debug symbols: the script strips the staged copy, never `target/release/easy-note`, so a cargo backtrace keeps its symbols
- [x] `strip` may be absent: the strip is best-effort and the build goes on
- [x] `LICENSE` must become a policy-shaped `copyright` file (blank lines become `.`, every line indented)
- [x] The changelog must be gzipped with no timestamp (`gzip -9n`), so two builds of one commit give the same bytes
- [x] `md5sums` must be sorted and stable (`find -print0 | sort -z`)
- [x] The package must be owned by root, not by the person who built it (`dpkg-deb --root-owner-group`)
- [x] A build in a repo with no git history still works (`git rev-parse` falls back to `unknown`)
- [x] `dist/` is already in `.gitignore`, so no package is ever committed
- [x] Stale references: `README.md` and `RUNBOOK.md` now name the `.deb` path; `PRD.md` NFR-006 named `cargo install` only

---

## Files to Change (Predicted)

| File | Change |
|------|--------|
| `scripts/build_deb.sh` | new: stage the tree, read `Depends` from the binary, write `DEBIAN/control` and `md5sums`, call `dpkg-deb --build` |
| `tests/deb.rs` | new: package `/bin/true` and assert the contents, the control fields, the absent maintainer scripts and the missing-tool error |
| `README.md` | new "A Debian package" section under the install part |
| `RUNBOOK.md` | new "Debian package" section: build, install, what the package holds, how `Depends` is read |
| `PRD.md` | NFR-006 names the `.deb` install path beside `cargo install` |
| Pre-existing pin tests | `tests/readme.rs` greps every `easy-note <cmd>` the README names against the real subcommand list — the new README section names `setup` and `doctor` only, both real; no line-count or size pin covers `README.md` or `RUNBOOK.md` |

## Files Must NOT Touch

| File | Reason |
|------|--------|
| `src/**` | packaging changes no app behaviour (Surgical Changes) |
| `Cargo.toml`, `Cargo.lock` | the package reads the version; it does not set it |
| `.github/workflows/**` | a CI packaging job is out of scope |
| `.gitignore` | `dist/` is already there (line 11) |

---

## Test Plan

- `tests/deb.rs` runs the real script with `EASY_NOTE_BIN=/bin/true`, so it exercises the real
  staging, the real `dpkg-shlibdeps` and the real `dpkg-deb --build` without a release build.
  It asserts: the four package files and no fifth, the `Package`/`Version`/`Architecture`/`Depends`
  control fields, no maintainer script, and the named error with exit 1 when `dpkg-deb` is off `PATH`.
- The test skips itself with a printed reason where `dpkg-deb` is absent, so CI without `dpkg-dev` stays green.
- A real release build and a real `sudo apt install` of the built package stay the owner's run: the
  install needs root, which the pipeline never takes.

---

## Completion Checklist

- [x] Implementation done
- [x] Self-review: `Skill({ skill: "code-review" })` run
- [ ] **easy-verifier**: Stage 4 `Skill({ skill: "score" })` — N/A with a reason when the verifier is unreachable
- [x] Security review: not required (Risk Low)
- [x] Lint passes
- [x] Tests written AND pass — output pasted into `tasks/TASK_REVIEW_HUN-287.md`'s Evidence table (Hard-Stop Gate 5)
- [ ] `Skill({ skill: "verify" })` run — the headless auto verify run fills the row
- [x] `memory/MEMORY.md` updated (diff-driven pass)
- [x] Supervisor notified: task ready for Stage 4 review
