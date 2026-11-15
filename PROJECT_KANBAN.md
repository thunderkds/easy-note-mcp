# PROJECT_KANBAN.md
**Last updated**: 2026-11-15

> Board cache, pulled from the board by `python3 scripts/board_sync.py pull`. Do not edit it by hand:
> the board is the source of truth, and the next pull overwrites this file.

---

## Board

### Todo
- [ ] **HUN-255** — Secret guard: declared patterns, mask on write and read, leak test suite | Backend-Implementer | C1 | Risk: High | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-255.md`
- [ ] **HUN-257** — CLI commands: init, add/n, ls, search, show, edit, tag, rm, restore, trash, stats, doctor, path | Backend-Implementer | C1 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-257.md`
- [ ] **HUN-258** — MCP surface: 5 tools over stdio and streamable HTTP on 127.0.0.1:8769 | Backend-Implementer | C1 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-258.md`
- [ ] **HUN-259** — Claude Code capture: hook-prompt, install/uninstall claude, note.md fallback | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-259.md`
- [ ] **HUN-260** — One press / one click: easy-note quick, hotkey helper, install launcher, editor snippets | Backend-Implementer | C1 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-260.md`
- [ ] **HUN-261** — Read-only inbox/ notes for phones and other devices | Backend-Implementer | C1 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-261.md`
- [ ] **HUN-262** — Claude detail: session/commit fields, AI files, note_pending_enrich, note_enrich, enrich_notes prompt | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-262.md`
- [ ] **HUN-263** — System tray icon: easy-note tray and install/uninstall tray (XDG autostart) | Backend-Implementer | C2 | Risk: Low | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-263.md`
- [ ] **HUN-264** — README, RUNBOOK, client config snippets, install guide | Backend-Implementer | C0 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-264.md`
- [ ] **HUN-265** — Perf check over 10,000 notes: search, note_add, hook and n timings | QA-Automation-Agent | C0 | Risk: Low | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-265.md`
- [ ] **HUN-266** — Manual run: measure how Cursor starts a stdio MCP server (HUN-251 follow-up) | Owner | C0 | Risk: Low | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-266.md`

### In Progress

### Ready for Review

### Done
- [x] **HUN-270** — R3 Rust secret guard: ten kinds, mask on write and read, leak suite | Backend-Implementer | C1 | Risk: High | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-270.md` | Branch: `feat/HUN-270-rust-guard` | Done 2026-11-16: merged 2d023e1 by the owner over a failing Companion score (71); auto verify pass (session 4820c8b8-f2e6-4327-9393-8416549bfa05, commit f49c09fd69fb)
- [x] **HUN-272** — R5 Rust query engine: list, search, stats, bounded results | Backend-Implementer | C1 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-272.md` | Branch: `feat/HUN-272-rust-query` | Done 2026-11-15: auto verify pass (session 8536879c-9e4d-4f5b-aee1-fff530a455d5, commit 18272656a19d23751f55cf6b836644c26a279f33)
- [x] **HUN-271** — R4 Rust context capture: cwd order, main-worktree repo name, remote sanitizing, task patterns | Backend-Implementer | C1 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-271.md` | Branch: `feat/HUN-271-rust-context` | Done 2026-11-15: auto verify pass (session 98bed120-ede1-42ae-ba63-910c1605f4f6, commit 5b9c5dabb4d83397506a8ba5ba776237085e7742)
- [x] **HUN-269** — R2 Rust note format and store: parse/render, ID, never-overwrite write, clash suffix, trash/restore, skipped files | Backend-Implementer | C2 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-269.md` | Branch: `feat/HUN-269-rust-store` | Done 2026-11-15: merged 5c223a0 by the owner over a failing Companion score (67); auto verify pass (session e978d4eb-4075-4dc8-87ed-0cecf5d1dadc, commit 1d4ef7681476fe6e4e4f3996fad4915349900bbd)
- [x] **HUN-268** — R1 Rust skeleton: cargo binary easy-note (+ n link), CI, remove all Python app code | Common-Infrastructure-Agent | C1 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-268.md` | Branch: `feat/HUN-268-rust-skeleton` | Done 2026-11-15: auto verify pass (session 112498b1-18d7-47b0-9c29-228891780e6c, commit bf07f5c92b516e48539051a66fc7e3eb17d97a91)
- [x] **HUN-251** — Spike: how Claude Code, Codex and Cursor start stdio MCP servers, and what the prompt hook sees | QA-Automation-Agent | C1 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-251.md`
- [x] **HUN-252** — Repo skeleton: uv project, src/easy_note/{core,surfaces}, CLI and MCP entry points over one core, CI | Common-Infrastructure-Agent | C1 | Risk: Low | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-252.md`
- [x] **HUN-253** — Note format and store: parse/render, ID, atomic write, clash suffix, trash/restore, skipped files | Backend-Implementer | C2 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-253.md`
- [x] **HUN-254** — Context capture: cwd order, main-worktree repo name, remote sanitizing, task patterns | Backend-Implementer | C1 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-254.md`
- [x] **HUN-256** — Query engine: list, search, stats, bounded results | Backend-Implementer | C1 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-256.md`
- [x] **HUN-267** — Align context task pattern with PRD FR-005 | ? | ? | Risk: ? | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-267.md`

### Closed (will not do)

## Blocked

---

## Stage Tracker
