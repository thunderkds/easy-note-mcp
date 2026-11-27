# PROJECT_KANBAN.md
**Last updated**: 2026-11-27

> Board cache, pulled from the board by `python3 scripts/board_sync.py pull`. Do not edit it by hand:
> the board is the source of truth, and the next pull overwrites this file.

---

## Board

### Todo
- [ ] **HUN-266** — Manual run: measure how Cursor starts a stdio MCP server (HUN-251 follow-up) | Owner | C0 | Risk: Low | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-266.md`
- [ ] **HUN-297** — Daily digest per project and topic, and the read-only \`note_context\` MCP tool | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-297.md`
- [ ] **HUN-298** — Window views: group by project, topic or kind; Drafts view; AI status; \`Run now\` | Frontend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-298.md`
- [ ] **HUN-299** — Window fixes: change project, topic or kind, rename or merge a topic, Undo | Frontend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-299.md`
- [ ] **HUN-301** — UX pass: keyboard keys, first-run empty state, and a \`ux-evaluate\` check of every screen | Frontend-Implementer | C1 | Risk: Low | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-301.md`

### In Progress
- [ ] **HUN-305** — Dashboard redesign: full-height list and note panes, simpler list rows, name Easy Taking Shot | Frontend-Implementer | C2 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-305.md`

### Ready for Review
- [ ] **HUN-295** — \`easy-note ai run\`: caps, batches, a run lock, back-off and \`--dry-run\` | Backend-Implementer | C2 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-295.md`
- [ ] **HUN-296** — Tray timer: run \`ai run\` on the schedule while the tray is up | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-296.md`
- [ ] **HUN-304** — Dashboard window layout: centred container on large screens, fit every allowed window size | Frontend-Implementer | C2 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-304.md`

### Done
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
- [x] **HUN-273** — Obsidian dashboard: easy-note dashboard + install/uninstall obsidian (easy-note.base) | Backend-Implementer | C1 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-273.md`
- [x] **HUN-274** — AI detail visible in Obsidian: <notes dir>/ai/<id>.md views + easy-note ai render | Backend-Implementer | C1 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-274.md`
- [x] **HUN-275** — AI detail from the local claude CLI: easy-note ai enrich (claude -p) + [ai] settings section | Backend-Implementer | C2 | Risk: High | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-275.md`
- [x] **HUN-276** — Icons from the design: app icon, launcher opens the dashboard, tray Open dashboard + saved badge | Backend-Implementer | C2 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-276.md`
- [x] **HUN-277** — easy-note setup: one command sets up launcher, tray and the Obsidian vault (registers the notes dir in obsidian.json) | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-277.md`
- [x] **HUN-278** — Dashboard window, read-only board: easy-note dashboard opens our own window (wry webview, feature window) | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-278.md`
- [x] **HUN-279** — Dashboard window actions: Add as tags, Move to trash with Undo, Open in editor, Copy id | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-279.md`
- [x] **HUN-280** — Dashboard window capture dialog: + New note in the window (Enter save, Esc cancel) | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-280.md`
- [x] **HUN-281** — Dashboard window: an unusable display makes tao panic instead of falling back to Obsidian | ? | ? | Risk: ? | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-281.md`
- [x] **HUN-282** — Dashboard window page: load the board once at start, render search faster, slimmer board payload | ? | ? | Risk: ? | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-282.md`
- [x] **HUN-283** — Docs out of date after the dashboard window: CLI help and PRD non-goal | ? | ? | Risk: ? | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-283.md`
- [x] **HUN-284** — Dashboard window hardening: stricter CSP and a pinned origin; sticky detail panel at 800 px | ? | ? | Risk: ? | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-284.md`
- [x] **HUN-285** — Tray: starts at login and with the app, menu texts from the design | Backend-Implementer | C1 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-285.md`
- [x] **HUN-286** — Dashboard window: read and edit a note in the app, not in Obsidian | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-286.md`
- [x] **HUN-287** — Debian package: scripts/build_deb.sh builds an installable .deb (window + tray) | ? | ? | Risk: ? | P2 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-287.md`
- [x] **HUN-288** — Dashboard window: Run AI button in the detail panel enriches that note now | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-288.md`
- [x] **HUN-289** — quick: the design's capture dialog in its own small window, not the zenity box | Backend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-289.md`
- [x] **HUN-290** — One dashboard window: a second open focuses the window that is open | Backend-Implementer | C1 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-290.md`
- [x] **HUN-291** — Show the time a note was last edited: \`updated\` in lists, MCP and the window | Backend-Implementer | C1 | Risk: Low | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-291.md`
- [x] **HUN-292** — AI file keys, overrides, change log, \`ai set\` and \`ai undo\` (core) | Backend-Implementer | C2 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-292.md`
- [x] **HUN-293** — \`[ai]\` config keys for the scheduler and batch run, and a \`doctor\` line | Backend-Implementer | C1 | Risk: Low | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-293.md`
- [x] **HUN-294** — Batch organizer engine: one model call per batch, checked item by item | Backend-Implementer | C2 | Risk: High | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-294.md`
- [x] **HUN-300** — Settings screen: AI switches, schedule and limits, with the one-time "what is sent" notice | Frontend-Implementer | C2 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-300.md`
- [x] **HUN-302** — Undo for the AI title, summary, tags and draft keys | Backend-Implementer | C2 | Risk: Medium | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-302.md`
- [x] **HUN-303** — \`easy-note setup\` binds Ctrl+Alt+N on GNOME, and the install guide says so | Backend-Implementer | C1 | Risk: Medium | P1 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-303.md`

### Closed (will not do)
- [~] **HUN-255** — Secret guard: declared patterns, mask on write and read, leak test suite | Backend-Implementer | C1 | Risk: High | P0 | @hungnh1110 | Guide: `tasks/TASK_GUIDE_HUN-255.md`

## Blocked

---

## Stage Tracker
