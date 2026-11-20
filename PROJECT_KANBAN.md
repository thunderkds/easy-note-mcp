# PROJECT_KANBAN.md
**Last updated**: 2026-11-19

> Board cache, pulled from the board by `python3 scripts/board_sync.py pull`. Do not edit it by hand:
> the board is the source of truth, and the next pull overwrites this file.

---

## Board

### Todo
- [ ] **HUN-266** — Manual run: measure how Cursor starts a stdio MCP server (HUN-251 follow-up) | Owner | C0 | Risk: Low | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-266.md`

### In Progress
- [ ] **HUN-280** — Dashboard window capture dialog: + New note in the window (Enter save, Esc cancel) | Backend-Implementer + Frontend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-280.md` | In Progress 2026-11-20: backend slice, worktree ../easy-note-mcp-HUN-280 (from feat/HUN-279-actions)
- [ ] **HUN-282** — Dashboard window page: load the board once at start, faster search render, slimmer board payload (HUN-278 review P2) | Frontend-Implementer + Backend-Implementer | C1 | Risk: Low | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-282.md` | In Progress 2026-11-20: worktree ../easy-note-mcp-HUN-282 (from feat/HUN-279-actions)
- [ ] **HUN-284** — Dashboard window hardening: stricter CSP, pinned origin, sticky detail panel at 800 px (HUN-278 reviews) | Frontend-Implementer + Backend-Implementer | C1 | Risk: Medium | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-284.md` | In Progress 2026-11-20: worktree ../easy-note-mcp-HUN-284 (from feat/HUN-279-actions)

### Ready for Review
- [ ] **HUN-281** — Dashboard window: an unusable display makes tao panic instead of falling back to Obsidian (HUN-278 review P2) | Backend-Implementer | C1 | Risk: Low | P1 | @hungnh1110 | Branch fix/HUN-281-display | Track: minimal | Record: tasks/MINIMAL_LOG.md
- [ ] **HUN-283** — Docs out of date after the dashboard window: CLI help and PRD non-goal (HUN-278 review P2) | Backend-Implementer | C0 | Risk: Low | P2 | @hungnh1110 | Branch docs/HUN-283-stale | Track: minimal | Record: tasks/MINIMAL_LOG.md
- [ ] **HUN-279** — Dashboard window actions: Add as tags, Move to trash with Undo, Open in editor, Copy id | Backend-Implementer + Frontend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-279.md` | Ready for Review 2026-11-20: branch feat/HUN-279-actions @ d592093; code-review 0 P0/P1 (4 P2), security-review no findings

### Done
- [x] **HUN-278** — Dashboard window, read-only board: easy-note dashboard opens our own window (wry webview, feature window) | Backend-Implementer + Frontend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-278.md` | Done 2026-11-20: merged 996fe74 by the owner (auto verify pass; companion score 74 false-positive fail, owner override)
- [x] **HUN-273** — Obsidian dashboard: easy-note dashboard + install/uninstall obsidian (easy-note.base) | Backend-Implementer | C1 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-273.md` | Done 2026-11-19 (owner moved: merged, auto verify pass)
- [x] **HUN-274** — AI detail visible in Obsidian: <notes dir>/ai/<id>.md views + easy-note ai render | Backend-Implementer | C1 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-274.md` | Done 2026-11-19 (owner moved: merged, auto verify pass)
- [x] **HUN-275** — AI detail from the local claude CLI: easy-note ai enrich (claude -p) + [ai] settings section | Backend-Implementer | C2 | Risk: High | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-275.md` | Done 2026-11-19 (owner moved: merged, auto verify pass)
- [x] **HUN-276** — Icons from the design: app icon, launcher opens the dashboard, tray Open dashboard + saved badge | Backend-Implementer | C2 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-276.md` | Done 2026-11-19 (owner moved: merged, auto verify pass)
- [x] **HUN-277** — easy-note setup: one command sets up launcher, tray and the Obsidian vault (registers the notes dir in obsidian.json) | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-277.md` | Done 2026-11-19: auto verify pass (session 57adcf06-698d-4fd2-ae01-d55a73e5839c, commit c00391b7f352d0d7f2626d9c9bc5fc09744e7de2)
- [x] **HUN-251** — Spike: how Claude Code, Codex and Cursor start stdio MCP servers, and what the prompt hook sees | QA-Automation-Agent | C1 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-251.md`
- [x] **HUN-252** — Repo skeleton: uv project, src/easy_note/{core,surfaces}, CLI and MCP entry points over one core, CI | Common-Infrastructure-Agent | C1 | Risk: Low | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-252.md`
- [x] **HUN-253** — Note format and store: parse/render, ID, atomic write, clash suffix, trash/restore, skipped files | Backend-Implementer | C2 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-253.md`
- [x] **HUN-254** — Context capture: cwd order, main-worktree repo name, remote sanitizing, task patterns | Backend-Implementer | C1 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-254.md`
- [x] **HUN-256** — Query engine: list, search, stats, bounded results | Backend-Implementer | C1 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-256.md`
- [x] **HUN-257** — CLI commands (Rust): init, add/n, ls, search, show, edit, tag, rm, restore, trash, stats, doctor, path | Backend-Implementer | C1 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-257.md`
- [x] **HUN-258** — MCP surface: 5 tools over stdio and streamable HTTP on 127.0.0.1:8769 | Backend-Implementer | C1 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-258.md`
- [x] **HUN-259** — Claude Code capture: hook-prompt, install/uninstall claude, note.md fallback | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-259.md`
- [x] **HUN-260** — One press / one click: easy-note quick, hotkey helper, install launcher, editor snippets | Backend-Implementer | C1 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-260.md`
- [x] **HUN-261** — Read-only inbox/ notes for phones and other devices | Backend-Implementer | C1 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-261.md`
- [x] **HUN-262** — Claude detail: session/commit fields, AI files, note_pending_enrich, note_enrich, enrich_notes prompt | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-262.md`
- [x] **HUN-263** — System tray icon: easy-note tray and install/uninstall tray (XDG autostart) | Backend-Implementer | C2 | Risk: Low | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-263.md`
- [x] **HUN-264** — README, RUNBOOK, client config snippets, install guide | Backend-Implementer | C0 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-264.md`
- [x] **HUN-265** — Perf check over 10,000 notes: search, note_add, hook and n timings | QA-Automation-Agent | C0 | Risk: Low | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-265.md`
- [x] **HUN-267** — Align context task pattern with PRD FR-005 | ? | ? | Risk: ? | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-267.md`
- [x] **HUN-268** — R1 Rust skeleton: cargo binary easy-note (+ n link), CI, remove all Python app code | ? | C1 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-268.md`
- [x] **HUN-269** — R2 Note format and store in Rust: parse/render, ID, atomic write, clash suffix, trash/restore, skipped files | ? | C2 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-269.md`
- [x] **HUN-270** — R3 Secret guard in Rust: declared patterns, mask on write and read, leak test suite | ? | C1 | Risk: High | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-270.md`
- [x] **HUN-271** — R4 Context capture in Rust: cwd order, main-worktree repo name, remote sanitizing, task patterns | ? | C1 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-271.md`
- [x] **HUN-272** — R5 Query engine in Rust: list, search, stats, bounded results | ? | C1 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-272.md`

### Closed (will not do)
- [~] **HUN-255** — Secret guard: declared patterns, mask on write and read, leak test suite | Backend-Implementer | C1 | Risk: High | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-255.md`

## Blocked

---

## Stage Tracker
