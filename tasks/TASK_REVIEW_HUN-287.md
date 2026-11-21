# TASK_REVIEW — HUN-287: Debian package — `scripts/build_deb.sh` builds an installable `.deb`

> Sibling of `tasks/TASK_GUIDE_HUN-287.md`. Filled by the reviewer at Stage 4/5.

---

## Auto decisions

Mode: auto
1. Extended track, C2 / Risk Low / P2 — RULE: `docs/claude-md/tracks.md` `## Track rule` (C2 = 3+ files or a new component) with `agents/backend.md:93`. The session that wrote the code ran it on the minimal track; a task only ever moves up, so it gets a guide, a Stage 4 review and a verify run.
2. The work moved from the ID `HUN-285` to `HUN-287`: the branch is `feat/HUN-287-deb` (renamed from `feat/HUN-285-deb`), and the `(HUN-285)` mentions in `scripts/build_deb.sh:2` and `tests/deb.rs:1` now read `HUN-287` — FACT: `tasks/TASK_GUIDE_HUN-287.md` "Owner approved the renumbering (chat 2026-11-21)".
3. The event trace of the implementing session stays `memory/event-trace/HUN-285.jsonl` — SAFER OPTION: that file holds the tray task's events too (its records 38-66 run in `easy-note-mcp-HUN-285`), and every record carries `prev` and `hash`, so a rename or a split would break the chain and lose the tray audit trail. The guide names the file instead. ⚠ decided for you — please review
4. Restored `PROJECT_KANBAN.md` and the five `TASK_GUIDE_HUN-278/279/280/282/284.md` files that an earlier board pull had rewritten in this checkout, after copying all six to the session scratchpad — RULE: `memory/learnings.md` § `2026-11-16 — rmcp argument errors echo escaped values; spawn pull rewrites the worktree's board cache (HUN-258)` ("git restore board cache before review"). The pull had moved seven merged rows back to Todo and cut 188 lines from two guides.
5. Wrote three more test cases rather than lower Acceptance Criteria 2, 4 and 7 to what the one existing test covered — RULE: `AGENTS.md` § Hard-Stop Gates, gate 5 ("No tests = not done"). The Supervisor wrote this oracle, not the implementing session, as `templates/TASK_GUIDE_template.md` § Evaluation & Acceptance asks.
6. Recorded the `.deb` install path in `PRD.md` NFR-006 and `PROJECT_SPEC.md` `Deployment target`, instead of adding a new FR — RULE: `AGENTS.md` § Karpathy Engineering Principles (Simplicity First): packaging adds no app behaviour, so it belongs in the portability line that already names `cargo install`. ⚠ decided for you — please review
7. The `memory/event-trace/*.jsonl` files stay uncommitted — RULE: the practice of every task since HUN-278 (`main` holds traces for HUN-251, HUN-252, HUN-277 and HUN-278 only), because `_untagged.jsonl` is append-only and conflicts on every parallel merge.
8. The six P3 review findings ship unfixed, as follow-ups — RULE: `skills/code-review/SKILL.md` Phase 4 (apply P0/P1 fixes; present P2/P3 as suggestions).
Checks: tests `cargo test --locked` — 25 suites, 568 passed, 0 failed; `cargo test --locked --features window,tray` — 25 suites, 576 passed, 0 failed; `cargo test --locked --test deb` — 3 passed; lint `cargo fmt --check` clean, `cargo clippy --all-targets --locked -- -D warnings` exit 0, `shellcheck -S style scripts/build_deb.sh` clean; reviews code-review — 0 P0, 0 P1, 1 P2 fixed (`8b54222`), 6 P3 open; security-review not run (Risk Low, and the package ships no maintainer script, so nothing of ours runs as root); real run `bash scripts/build_deb.sh` — see Demonstration (auto-checked, not /verify)
Memory: `memory/decisions.md` — 2026-11-21 HUN-287 Debian package: renumbered from HUN-285, `.deb` beside `cargo install`; `memory/learnings.md` — 2026-11-21 a `.deb` test can run the real dpkg tools in under a second with `EASY_NOTE_BIN=/bin/true`, and a PATH override breaks `Command::new("bash")`
Not done: merge (admin: after the auto verify pass); auto verify: ☐ blocked (session 29d7c789-7418-4636-9c59-5d955da312a9, commit dfd317d9d0cf3694fb7c6ff35d5f3fc86e39f8c5, log .claude/verify-runs/HUN-287-dfd317d9d0cf.log)

---

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/deb.rs` — `build_deb_packages_the_binary_and_the_docs` (AC 1, 2, 3, 5, 6), `missing_dpkg_tooling_names_the_tool_and_writes_nothing` (AC 7), `the_release_build_takes_window_and_tray_unless_features_say_otherwise` (AC 4). AC 8 is checked by `tests/readme.rs` plus the README/RUNBOOK diff. |
| Verification command run | ☑ pass | `cargo test --locked --test deb -- --nocapture`:<br>`running 3 tests`<br>`test missing_dpkg_tooling_names_the_tool_and_writes_nothing ... ok`<br>`test build_deb_packages_the_binary_and_the_docs ... ok`<br>`test the_release_build_takes_window_and_tray_unless_features_say_otherwise ... ok`<br>`test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.19s` |
| Negative cases hold | ☑ pass | Red run against the prior content (`scripts/build_deb.sh` moved out of the tree, 2026-11-21T19:32:45+07:00):<br>`$ ls scripts/build_deb.sh` → `"scripts/build_deb.sh": No such file or directory (os error 2)`<br>`failures: build_deb_packages_the_binary_and_the_docs, missing_dpkg_tooling_names_the_tool_and_writes_nothing, the_release_build_takes_window_and_tray_unless_features_say_otherwise`<br>`test result: FAILED. 0 passed; 3 failed` — then the script was moved back. Inside the suite, AC 7's own negative case (a `PATH` with no dpkg) asserts exit 1, the `need dpkg` line, the `sudo apt install dpkg-dev` hint and an empty output directory. |
| verify | ☐ blocked | auto-run by hook, session 29d7c789-7418-4636-9c59-5d955da312a9, commit dfd317d9d0cf3694fb7c6ff35d5f3fc86e39f8c5, log .claude/verify-runs/HUN-287-dfd317d9d0cf.log — blocked (no single verdict line) |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed: `scripts/build_deb.sh`, `tests/deb.rs`, `README.md`, `RUNBOOK.md`, `PRD.md`, `PROJECT_SPEC.md`. Skipped: all of `src/**` — the diff changes no Rust code, so no caller set exists. `lane: C2, 6 files, 574 lines, EXCEEDS (lines 574 > 500); unpredicted: tasks/TASK_GUIDE_HUN-287.md` — warn only; 193 lines are the guide and ~200 the docs. |
| Full smoke suite still green (no regression) | ☑ pass | `cargo test --locked` — 25 suites, 568 passed, 0 failed. `cargo test --locked --features window,tray` — 25 suites, 576 passed, 0 failed. Both lint runs exit 0: `cargo clippy --all-targets --locked -- -D warnings` and the same with `--features window,tray`. |
| **Companion score (easy-verifier)** | ☑ N/A | easy-verifier was not run: the companion MCP tool is not available in this session, and the known false positives on this repo's `.claude/harness-lock.json` hashes make its `security` dimension unusable (`memory/learnings.md` § 2026-11-15). |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | Pure packaging task: no UI component, so the guide's UI/Design AC section was deleted (Hard-Stop Gate 6). |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | As above. |
| **UI: Responsiveness at target viewports** | ☑ N/A | As above. |

---

## Demonstration

**BEFORE** (2026-11-21T19:30:01+07:00, before the first implementation commit — `619151c` was HEAD):

```
$ git show HEAD:scripts/build_deb.sh
fatal: path 'scripts/build_deb.sh' exists on disk, but not in 'HEAD'
$ git show HEAD:tests/deb.rs
fatal: path 'tests/deb.rs' exists on disk, but not in 'HEAD'
```

**AFTER** (2026-11-21T19:39:37+07:00, on `8b54222`, a real release build of features `window,tray`):

```
$ bash scripts/build_deb.sh
building the release binary (features: window,tray)
    Finished `release` profile [optimized] target(s) in 0.32s
dist/easy-note_0.1.0_amd64.deb                       # 4.1 MB, exit=0

$ dpkg-deb --contents dist/easy-note_0.1.0_amd64.deb
-rwxr-xr-x root/root  14700240 ./usr/bin/easy-note
-rw-r--r-- root/root     14305 ./usr/share/doc/easy-note/README.md
-rw-r--r-- root/root       176 ./usr/share/doc/easy-note/changelog.gz
-rw-r--r-- root/root      1231 ./usr/share/doc/easy-note/copyright

$ dpkg-deb --field dist/easy-note_0.1.0_amd64.deb Package Version Architecture Depends Installed-Size
Package: easy-note
Version: 0.1.0
Architecture: amd64
Depends: libc6 (>= 2.39), libcairo2 (>= 1.10.0), libdbus-1-3 (>= 1.10), libgcc-s1 (>= 4.2), libgdk-pixbuf-2.0-0 (>= 2.22.0), libglib2.0-0t64 (>= 2.66.0), libgtk-3-0t64 (>= 3.21.4), libjavascriptcoregtk-4.1-0, libsoup-3.0-0 (>= 3.0.3), libwebkit2gtk-4.1-0 (>= 2.35.1)
Installed-Size: 14380

$ dpkg-deb --extract dist/easy-note_0.1.0_amd64.deb <tmp> && <tmp>/usr/bin/easy-note --version
easy-note 0.1.0
$ <tmp>/usr/bin/easy-note --help | grep -E '^  (tray|dashboard|setup)'
  tray         Show an icon in the system tray (needs a build with `--features tray`)
  dashboard    Open the dashboard: easy-note's own window in a `window` build on a desktop, else `easy-note.base` in Obsidian (run `install obsidian` first)
  setup        Set up the launcher, the Obsidian dashboard and vault, and the tray login item in one go
```

The `Depends` list names `libwebkit2gtk-4.1-0` and `libgtk-3-0t64`, so `dpkg-shlibdeps` did read the dependencies of a `window` build (AC 5), and `apt` will pull them.

**DELTA**: A person on Debian or Ubuntu installs easy-note — dashboard window and tray included — with `sudo apt install ./dist/easy-note_0.1.0_amd64.deb`, instead of installing a Rust toolchain for `cargo install`.

**WITNESS**: The code was written in session `1cc6cbb3-aba4-499f-a74b-593c4344e943` (`memory/event-trace/HUN-285.jsonl`, records 0-37, 2026-11-20). The renumbering, the three added tests, the Stage 4 review and the runs above are session `b0f44c22-81de-4d45-9216-7960c2753abd` (`memory/event-trace/HUN-287.jsonl`, 2026-11-21).
