# easy-note-mcp — tasks (synced to Linear 2026-11-14)

Team: **HUN** · Project: **easy-note-mcp** (create when the owner asks) · Assignee: Thunder
Labels: `complexity:<Cn>`, `risk:<level>` · Priority: P0 = Urgent, P1 = High, P2 = Medium
Each issue description: the Summary, the Acceptance Criteria, then "Source: PROJECT_SPEC.md / PRD.md in thunderkds/easy-note-mcp".
Status: all **Todo**. Synced 2026-11-14 to Linear project `easy-note-mcp` (P-HUN-8); the board is now the source of truth and each issue description is its `tasks/TASK_GUIDE_<ID>.md`. Plan ID → board ID: T001=HUN-251 · T002=HUN-252 · T003=HUN-253 · T004=HUN-254 · T005=HUN-255 · T006=HUN-256 · T007=HUN-257 · T008=HUN-258 · T009=HUN-264 · T010=HUN-265 · T011=HUN-259 · T012=HUN-260 · T013=HUN-261 · T014=HUN-262 · T015=HUN-263.

| Local | Title | Complexity | Risk | Priority | Depends on |
|---|---|---|---|---|---|
| T001 | Spike: how Claude Code, Codex, Cursor start stdio MCP servers | C1 | Med | P0 | — |
| T002 | Repo skeleton with kit artifacts and one core for CLI + MCP | C1 | Low | P0 | — |
| T003 | Note format + store | C2 | Med | P0 | T002 |
| T004 | Context capture (repo, remote, branch, task) | C1 | Med | P0 | T001, T003 |
| T005 | Secret guard, masking, leak test suite | C1 | High | P0 | T003 |
| T006 | Query engine: list, search, stats | C1 | Low | P1 | T003 |
| T007 | CLI commands | C1 | Low | P1 | T004, T005, T006 |
| T008 | MCP surface (5 tools) | C1 | Med | P1 | T004, T005, T006 |
| T009 | README, RUNBOOK, client config snippets | C0 | Low | P1 | T007, T008 |
| T010 | Perf check over 10,000 notes (incl. hook and `n` timings) | C0 | Low | P2 | T006, T011 |
| T011 | Claude Code capture hook + install/uninstall | C2 | Med | P1 | T001, T005, T007 |
| T012 | One press / one click: `quick`, `hotkey` helper, launcher icon, editor snippets | C1 | Low | P1 | T007 |
| T013 | Read-only `inbox/` notes | C1 | Med | P1 | T003 |
| T014 | Claude detail: AI files, `note_pending_enrich`, `note_enrich`, `enrich_notes` prompt | C2 | Med | P1 | T003, T008, T011 |
| T015 | System tray icon + autostart | C2 | Low | P2 | T012 |

## Acceptance Criteria per task

**T001** — For each of Claude Code, Codex, Cursor: record the stdio server's process cwd when started from a project, whether the client answers `roots/list`, and the `clientInfo.name` it sends. Output: a table in `docs/experiments/T001-client-launch.md` and a decision on the cwd order (FR-004). If a client gives neither roots nor a project cwd, the README says "pass `cwd`" for it. Round 2, Claude Code hook: does a `UserPromptSubmit` hook receive `/note …` raw (or does Claude Code reject an unknown command first); does `{"decision":"block","reason":…}` stop the prompt and show the reason; does the hook run for a prompt typed while the agent works; which `cwd` the event carries in a worktree. Output decides the default prefix (D18).

**T002** — `uv tool install .` gives `easy-note`; `easy-note --version` and `easy-note serve` start; `pytest` runs green on an empty suite; repo holds CLAUDE.md (`@AGENTS.md`), AGENTS.md, PRD.md, PROJECT_SPEC.md, PROJECT_KANBAN.md, RUNBOOK.md, BRAINSTORMING_LOG.md, GRILL_requirement.md, `tasks/`, `memory/`, `reports/`, `docs/ddr/` (copied from this plan), installed via the kernel's `setup.sh`.

**T003** — Tests: round-trip of a tool-written note is byte-identical; ID format and clash suffix (two writes in the same second with the same text); a write never replaces an existing file; temp file + rename (no partial file after a killed write, simulated); `rm` → trash, `restore` → back, same bytes; bad front matter and `easy-note: 2` counted as skipped with path, file unchanged; unknown keys kept on edit. `docs/NOTE_FORMAT.md` final.

**T004** — Tests on temp git repos: main repo, linked worktree (repo = main folder name), no git (fields absent), detached HEAD (branch absent), remote `https://user:tok@github.com/o/r.git` → `o/r` and no token anywhere in the file; branch `fix/t120-x` → `T120`, `feat/HUN-221-y` → `HUN-221`, `main` → absent; explicit task wins; cwd order per T001; git timeout 2 s → fields absent, add still succeeds; no path in any written file.

**T005** — Declared patterns (at least: AWS key, GitHub token, Slack token, OpenAI/Anthropic-style key, private key block, JWT, `password=`/`secret=` assignments, generic high-entropy bearer). Tests (round 2, mask not refuse): each planted secret on `add`, `n`, `quick`, hook, `edit`, `note_add` is written as `[masked:<kind>]`, the rest of the note is kept, the confirmation names the count and kinds, and the value is not echoed; hand-edited file with secrets is masked in `show`, `search`, `ls`, `note_read`, `note_search`, `note_list`; log capture holds no note text; a grep of every test output for planted values finds nothing.

**T006** — Tests: AND-of-words, case-insensitive; each filter; local-date `day`/`since`/`until` across a timezone offset; newest-first order stable for equal times (by ID); `limit` cap 50 with `total` + `truncated`; snippet ≤ 200 chars; stats counts by repo/task/tag/source/month and `skipped`; same output twice on the same files.

**T007** — Every FR-013 command has a test through the CLI runner; round 2: with no config the first `add` creates `~/easy-notes` (temp HOME in tests) and prints the path once; `n retry #flaky it's!` saves the exact words (args joined); `n` with no args reads one line; `-e` opens `$EDITOR`; every capture prints the FR-021 line; the CLI import graph has no `mcp` module (test); `edit` re-runs the guard and masks each secret before the save (FR-014, D21), naming the count; exit codes 0/1/2 documented.

**T008** — 5 tools registered and no others (test lists the names); each of the 4 read tools leaves the notes dir byte-identical (hash before/after); `note_add` adds exactly one file; responses respect FR-016 bounds; tool descriptions contain "only when the person asks"; HTTP mode refuses to bind anything but `127.0.0.1`; smoke run over stdio with the MCP SDK client.

**T009** — README: one-paragraph pitch, install, `init`, the three client snippets (Claude Code, Codex, Cursor), "notes are plain files" section, boundary vs easy-memory. RUNBOOK: release steps. Each snippet run once in T001's environment.

**T010** — Fixture generator for 10,000 notes; measured p95 for `search`, `note_add`, hook pass path, hook save path and `n` cold start pasted in the TASK_GUIDE Evidence; NFR-004, NFR-009, NFR-010 confirmed or reset in the PRD with the numbers.

**T011** — Hook tests with recorded event JSON: prefix → one file, `source: hook`, repo from event `cwd`, block JSON with id/repo/task; other prompt → no stdout, exit 0, no file; malformed stdin → prompt passes; masking applies. Install tests on a temp settings file: diff shown, backup created and never overwritten, run twice = once, existing easy-memory hook entries byte-identical, symlink kept; uninstall removes exactly its entry and `note.md`. Manual smoke in real Claude Code: `/note test` shows `✓ noted …` and the model does not answer.

**T012** — `quick` tests with the dialog tool stubbed: text → saved `source: quick`; cancel/empty → nothing saved, exit 0; `--cwd` repo tag; last context < 2 h pre-filled and clearable, ≥ 2 h not used; no dialog tool → `tkinter`, else terminal prompt; box visible p95 < 300 ms (measured on GNOME/Wayland and X11); `hotkey` prints correct commands per desktop and writes nothing; `install launcher` writes exactly one `.desktop` file, `uninstall launcher` removes it. README snippets: VS Code `tasks.json` + `keybindings.json` using `${workspaceFolder}`, JetBrains External Tool with `$ProjectFileDir$`, macOS Shortcuts, GNOME/KDE custom shortcut; each tried once.

**T013** — Tests: `inbox/20261114-183002.txt` → id `inbox/20261114-183002`, created from the name; a file with no time prefix → mtime; `#tags` parsed; file with front matter → its keys win; list/search/stats/read include inbox notes; the inbox dir is byte-identical after every read command; `rm inbox/…` → trash; subfolders and other extensions ignored.

**T014** — Tests: hook notes carry `session`, repo notes carry `commit`; `note_pending_enrich` lists un-enriched and stale notes only; `note_enrich` writes one AI file with caps enforced (over-long fields refused), secrets masked, `based_on`/`at` set by the server, unknown id refused; every note file byte-identical after an enrich run (hash before/after); reads show the "AI detail (not your words)" label; search hit in AI text marked `matched_in: ai`; stats unchanged except `enriched`; `edit` makes detail stale; `rm`/`restore` move the AI file; `tag --accept-ai` and `ai rm` work; `note_add(detail=…)` stores `model: capture`; the T008 tool-list test now expects exactly 7 tools + 1 prompt (D37). Manual run of `/mcp__easy-note__enrich_notes` on 20 real notes: tokens per note recorded.

**T015** — (round 3c, v1) `easy-note tray` with pystray: icon appears on GNOME with AppIndicator (Ubuntu) and KDE; menu New note opens `quick` with last context; Recent 5 opens files; Open folder; Quit. Without a tray host: exits 1 with the hotkey hint, `doctor` reports it. `install tray` writes exactly one autostart `.desktop`, `uninstall tray` removes it; no autostart by default. Idle RAM measured and pasted. CLI/MCP never import pystray (test).

## Follow-ups on other boards

| Ref | Board | Title | Notes |
|---|---|---|---|
| F1 | easy-personal-mcp | Read-only `notes` source: counts per repo/task + file links | Front matter only (DDR 0002 here); needs its own DDR next to easy-personal DDR 0002; brainstorm first |
| F2 | easy-kit | List easy-note in `companion-setup` | One row; kernel never depends on easy-note |
| F3 | easy-personal-mcp (optional) | Tray "New note" item that runs `easy-note quick` | In its planned Tauri tray; hidden when easy-note is not on PATH; easy-note never depends on it |
