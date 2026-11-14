# Stage 0.5a — Requirement grill of PRD.md (easy-note-mcp)
**Date**: 2026-11-14 · **Skill**: `grill-with-docs` `mode=requirement` · **PRD gate**: PASS (all items resolved by the Supervisor in autonomous mode; owner review pending) · **Round 2 (capture ease)**: 2026-11-14, G19–G28, D17–D26, source `BRAINSTORMING_LOG_capture.md`

Checks run: FR → US traceability · scope clarity · ambiguity ("would two engineers build it differently?") ·
missing NFRs · open questions without owner/date · assumptions that trace only to a supervisor guess ·
microkernel boundary.

Traceability: FR-001…FR-017 each trace to a US ✅. US-008 (agent recall across repos) and US-009 (easy-personal counts) trace to FR-012/FR-015 ✅. Nothing in Out of Scope has an FR ✅.

## Findings

| # | Finding | Where | Kind | Resolution (Supervisor, 2026-11-14) | Status |
|---|---|---|---|---|---|
| G1 | "Somewhere to save" is open: DB, files, or a cloud tool | US-001, FR-002 | Ambiguity | Plain Markdown files in a folder the person picks (D2, DDR 0002) | ✅ Resolved |
| G2 | One file per note or per day? Two engineers would build it differently | FR-002 | Ambiguity | One file per note; day view is a query (D2) | ✅ Resolved |
| G3 | The stdio MCP server may not know which repo the agent works in; tags could be wrong | FR-004, US-002 | Risk / unknown | Fixed order: explicit `cwd` arg → MCP roots → server process cwd; T001 spike verifies per client before T004 | ✅ Resolved (spike-gated) |
| G4 | "Task ID" has no pattern | FR-005 | Ambiguity | From branch name by declared patterns: kit `T\d{3,4}` (kit example `fix/t120-x`, `easy-kit/docs/claude-md/tracks.md:329`) and tracker keys `[A-Z][A-Z0-9]+-\d+` (e.g. `HUN-221`); first match wins; `--task` overrides; none → field absent (D5) | ✅ Resolved |
| G5 | Worktrees: a note from `wt-t148` must count for the main repo | FR-004 | Missing | `repo` = folder name of the main worktree (from `git rev-parse --git-common-dir`), never the worktree folder (D4) | ✅ Resolved |
| G6 | Full repo paths and remote URLs are personal and may hold tokens; notes may be git-synced | FR-004, NFR-002 | Privacy | Store repo folder name only; `remote` as `owner/name` only after stripping userinfo; never the path or cwd (D4) | ✅ Resolved |
| G7 | May agents edit or delete notes? | FR-012 | Scope | No. MCP = add/read only; edit/delete from CLI or the editor; delete moves to trash (D7) | ✅ Resolved |
| G8 | Secrets pasted into a note end up in a synced repo | NFR-002 | Missing NFR | Refuse on add; mask on read (D8) | ✅ Resolved |
| G9 | Cloud sessions (claude.ai/code, like this project's threads) cannot reach a local MCP | US-001 | Scope gap | Out of Scope for v1, stated in the PRD | ✅ Resolved |
| G10 | Timezone of `created` and of folder dates | FR-002, FR-010 | Ambiguity | `created` is ISO 8601 with the local UTC offset; folder `YYYY/MM` and `--day` use local date; sort by UTC instant (D10) | ✅ Resolved |
| G11 | Several machines writing one synced folder can clash on IDs | FR-003, NFR-005 | Missing | ID = local time to the second + 4 hex chars of sha256(created ns, host, text); a clash on write adds a suffix (D3) | ✅ Resolved |
| G12 | How easy-personal reads notes is not defined; risk of a hidden coupling | US-009 | Boundary | Reads front matter only, read-only, from the documented format v1; never the body; needs its own easy-personal DDR (DDR 0002 here; follow-up F1) | ✅ Resolved |
| G13 | Search quality: "deterministic" but which algorithm? | FR-011 | Ambiguity | Case-insensitive AND of words over title + body + tags; filters; newest first; no ranking model (D9) | ✅ Resolved |
| G14 | No performance target for search | NFR-004 | Missing NFR | p95 < 500 ms over 10,000 notes on a laptop; provisional until T010 measures (D9) | ✅ Resolved (provisional) |
| G15 | Kernel coupling: should the kit get a `note` skill or a hook? | US-001 | Boundary | No kernel change in v1. Follow-up F2: list easy-note in the kernel's `companion-setup` (kernel board) | ✅ Resolved |
| G16 | Notion / other connectors | US list | Scope | Out of Scope (network). Obsidian works with no code because the folder is a vault | ✅ Resolved |
| G17 | Docker parity with easy-ui / easy-verifier | NFR-006 | Convention | Native only (`uv tool install`), like easy-personal DDR 0001 and easy-memory; DDR 0001 here | ✅ Resolved |
| G18 | Board: Linear or local? | Process | Missing | Local `PROJECT_KANBAN.md` in the new repo; `linear-tasks.md` is ready to sync to team HUN when the owner asks | ✅ Resolved |

## Decisions (D1–D16)

Each decision names the alternative so the owner can flip it. "Owner's call" marks the ones a person would normally decide.

| # | Decision | Alternative not chosen | Why | Owner's call? |
|---|---|---|---|---|
| D1 | Separate plug-in repo `easy-note-mcp`; server id `easy-note`; CLI `easy-note` | Feature of easy-personal; extend easy-memory | `BRAINSTORMING_LOG.md` Options A/C | Yes |
| D2 | One Markdown file per note, YAML front matter, path `<dir>/YYYY/MM/<id>.md` | One file per day with line IDs | Atomic per-note write/edit/delete; clean git merges; no line-ID syntax | Yes |
| D3 | ID `YYYYMMDD-HHMMSS-xxxx` (local time + 4 hex) | UUID; counter | Sortable, readable, unique across machines without a counter | No |
| D4 | Context fields: `repo` (main-worktree folder name), `remote` (`owner/name`, userinfo stripped), `branch`; never paths | Full path; nothing | Notes may be synced/shared; matches easy-personal G13 (folder name in outputs) | No |
| D5 | `task` from branch by declared patterns (`T\d{3,4}`, `[A-Z][A-Z0-9]+-\d+`); `--task` / `task` arg overrides | Read `PROJECT_KANBAN.md` In Progress row | Branch rule is deterministic and kernel-free; Kanban read is kernel coupling (cut) | No |
| D6 | Writes only inside the notes dir (notes, `.easy-note/trash/`) and `~/.config/easy-note/config.toml` (only by `easy-note init`); git is read-only | Auto-commit to the notes repo | DDR 0001 | Yes |
| D7 | MCP tools: `note_add`, `note_search`, `note_list`, `note_read`, `note_stats` (5). No MCP edit/delete | Full CRUD over MCP | An agent can never destroy a note; files are editable by hand | Yes |
| D8 | ~~Refuse on a match~~ **Revised round 2 (D21):** mask each match as `[masked:<kind>]` before the write and say so; also mask in every read output | Refuse the whole note (round 1); warn only | Refusing loses the thought (P3); masking keeps the note and still keeps the secret off disk | No |
| D9 | Search: AND of words, case-insensitive, filters `repo/task/tag/source/since/until`, newest first; no index in v1 | SQLite FTS cache; embeddings | Deterministic, no state to rebuild; T010 measures | No |
| D10 | `created` in local time with offset; local dates for folders and `--day` | UTC everywhere | The person reads the files; the offset keeps sorting exact | No |
| D11 | Runtime: Python ≥ 3.11, official MCP SDK (FastMCP), `uv tool install`, stdio default, streamable-http opt-in on `127.0.0.1:8769` | stdlib-only like easy-memory; Docker | Same stack as easy-verifier / easy-personal; 8765–8768 are taken (ui, verifier, context, personal) | No |
| D12 | Tags: explicit `tags` arg + `#word` tokens in the text; lower-case, `[a-z0-9_/-]+` | Auto-tags from an LLM | Deterministic | No |
| D13 | Notes dir default `~/easy-notes`; `EASY_NOTE_DIR` env overrides config. **Revised round 2 (D20):** no `init` needed; the first write creates the default dir | `~/.easy-note/notes` (hidden); refuse until `init` (round 1) | Notes are for the person, so the folder is visible; the first note must not fail | Yes |
| D14 | Title = first line (≤ 120 chars); body = the rest; a note's text ≤ 20,000 chars | No limit | Bounded outputs for agents | No |
| D15 | Every MCP response is bounded: list/search return ≤ 50 items with a 200-char snippet; `note_read` returns full text | Unbounded | Token cost, like easy-ui low-token work | No |
| D16 | No kernel change in v1; kernel follow-up F2 only; easy-personal follow-up F1 on its own board | Ship kernel skill now | Microkernel: plug-in first, kernel never depends | Yes |

Only D1, D2, D6, D7, D13 and D16 change what the owner feels; the rest have reversible defaults.

## Round 2 — capture ease (2026-11-14)

Owner asked to make sure noting is easy. Friction map and options: `BRAINSTORMING_LOG_capture.md`. Verdict: round 1 was not easy enough.

| # | Finding | Where | Kind | Resolution | Status |
|---|---|---|---|---|---|
| G19 | In Claude Code a note costs a model turn, maybe a permission prompt, and lands in the task's context | US-001 | Friction | Prompt hook saves and blocks; no model call (D17) | ✅ Resolved |
| G20 | `/note` may not reach a hook raw, or may clash with a command | FR-018 | Unknown | T001 checks; fallback prefix `;n ` (D18) | ✅ Resolved (spike-gated) |
| G21 | First write refused until `init` | FR-001 | Friction | Zero setup (D20) | ✅ Resolved |
| G22 | Secret refusal drops the note | FR-014 | Friction | Mask, keep, report (D21) | ✅ Resolved |
| G23 | `easy-note add "..."` is long; shell eats `#`, `!`, `'` | FR-013 | Friction | `n` command, words joined, no-arg line prompt (D19) | ✅ Resolved |
| G24 | No capture from editor or desktop without a terminal | US-011 | Gap | `easy-note quick` + hotkey/editor snippets (D22) | ✅ Resolved |
| G25 | No capture from a phone or other device | US-012 | Gap | Read-only `inbox/` folder in the synced notes dir (D23) | ✅ Resolved |
| G26 | A hook on every prompt adds latency; easy-memory already has one | NFR-009 | Missing NFR | Pass path stdlib-only, p95 < 50 ms (D24) | ✅ Resolved |
| G27 | Installing a hook writes `~/.claude/settings.json`, outside DDR 0001 | DDR 0001 | Boundary | DDR 0003: person-run `install claude`, diff + backup + `uninstall`, easy-memory precedent (D25) | ✅ Resolved |
| G28 | Codex/Cursor have no hooks | US-001 | Limit | `note_add` description keys on "note:"; one model turn accepted (D26) | ✅ Accepted |

| # | Decision | Alternative not chosen | Why | Owner's call? |
|---|---|---|---|---|
| D17 | Claude Code capture = `UserPromptSubmit` hook `easy-note hook-prompt`: prefix match → save → `{"decision":"block","reason":"✓ noted <id> · <repo> · <task>"}`; repo from the hook input's `cwd` | Model calls `note_add` | No model turn, no context noise (P4, P6) | Yes |
| D18 | Default prefix `/note ` plus a `~/.claude/commands/note.md` fallback that asks the model to call `note_add`; if T001 shows the hook cannot see `/note`, default becomes `;n `; prefix configurable | Text prefix `note:` | Slash grammar is familiar and autocompletes; `note:` would catch real prompts that start with "note:" | Yes |
| D19 | Console script `n` = `easy-note add`; args joined by spaces; no args → one-line prompt; `-e` opens `$EDITOR` for long notes | Alias in README only | Works without shell config; prompt avoids shell quoting | Yes |
| D20 | No config → `~/easy-notes`, created on first write, path printed once | Refuse until `init` | P3 | No |
| D21 | Mask secrets on write, confirmation says `N secret(s) masked` | Refuse | P3 | No |
| D22 | `easy-note quick [--cwd DIR]`: native one-line box (`osascript` / `zenity` / `kdialog`), else terminal prompt; README snippets for OS hotkeys, VS Code task + keybinding, JetBrains External Tool | Own hotkey daemon (Option C) | No daemon; OS and editors already bind keys | No |
| D23 | `<notes dir>/inbox/*.md|*.txt` read as notes (`source: inbox`, id `inbox/<stem>`), never rewritten | Normalize into dated files; network endpoint | Any sync app can drop a file; no new write path; local-first | Yes |
| D24 | Hook pass path p95 < 50 ms, stdlib only; save path p95 < 150 ms | No target | Two prompt hooks may run on every prompt | No |
| D25 | `easy-note install claude` / `uninstall claude` own exactly one hook entry and the `note.md` command file (DDR 0003) | Print snippet only | One-command setup; family precedent in easy-memory | Yes |
| D26 | Codex/Cursor: no hook; `note_add` description: "When the person's message starts with `note:` or `/note`, call this at once with the rest of the message, then reply with one line" | Skip those clients | Keeps one capture phrase across clients | No |

Source enum grows: `source` ∈ `cli`, `mcp`, `hook`, `quick`, `inbox`.


## Round 3 — one press and Claude detail (2026-11-14)

Owner asked for "just click or press to note" and whether Claude should add detail at capture or in a later scan. Options: `BRAINSTORMING_LOG_oneclick-enrich.md`.

| # | Finding | Where | Kind | Resolution | Status |
|---|---|---|---|---|---|
| G29 | One press needs a resident listener; ours would need a daemon and OS input permissions | US-014 | Constraint | Use the OS shortcut service; `easy-note hotkey` prints the binding (D27) | ✅ Resolved |
| G30 | One click with no resident process | US-014 | Gap | Launcher icon via `install launcher` (D28) | ✅ Resolved |
| G31 | A tray of our own would be a second tray next to easy-personal's planned one | US-015 | Duplication | easy-personal "New note" item (F3); own tray v1.1 fallback (D29) — superseded by G39/D39 | ✅ Resolved |
| G32 | Hotkey notes have no repo | FR-022 | Gap | Last context < 2 h, shown and clearable (D30) | ✅ Resolved |
| G33 | Box must open before the thought fades | NFR-011 | Missing NFR | < 300 ms to visible box (D31) | ✅ Resolved (T012 measures) |
| G34 | Enrich at capture brings back the model turn of G19 and can rewrite the person's words | US-016 | Conflict | Rejected (D32) | ✅ Resolved |
| G35 | Later enrichment adds model text: against "deterministic"? | NFR-007 | Boundary | Server stays model-free; AI text in a separate, labelled, capped file; DDR 0004 (D33, D34) | ✅ Resolved |
| G36 | What detail can the pass use? The note alone is thin | US-016 | Missing | Store `session` + `commit` deterministically; pass reads the transcript if it still exists (D35) | ✅ Resolved |
| G37 | When an edit makes AI detail stale | FR-029 | Missing | `based_on` hash; stale = pending again (D36) | ✅ Resolved |
| G38 | MCP tool surface was frozen at 5 tools | FR-012 | Change | +2 tools (`note_pending_enrich`, `note_enrich`) + 1 prompt; owner approval needed (D37) | ✅ Resolved (owner to approve) |

| # | Decision | Alternative not chosen | Why | Owner's call? |
|---|---|---|---|---|
| D27 | One press = OS hotkey → `easy-note quick`; `easy-note hotkey` prints exact commands for GNOME, KDE, macOS, Windows | Own key-listener daemon | No resident process of ours; no OS input permission | Yes |
| D28 | One click = `easy-note install launcher` writes `~/.local/share/applications/easy-note.desktop` (Linux); macOS uses a Shortcuts action (README) | Tray only | Pin to dock, no resident process | No |
| D29 | ~~Tray only via easy-personal (F3), own tray v1.1~~ **Revised round 3c (D39):** easy-note has its own opt-in system tray icon in v1 | Depend on easy-personal's tray | Owner says the system tray matters; easy-personal's tray is not scheduled | Yes |
| D30 | `quick` pre-fills last context (repo/branch/task of the newest `cli`/`hook` note, < 2 h old), shown in the box, cleared with one key | No repo for hotkey notes | Most hotkey notes follow work in the same repo | No |
| D31 | `quick` box visible in < 300 ms; save after Enter | No target | A slow box loses the thought | No |
| D32 | No enrichment at capture (mode a rejected), except optional `detail` on `note_add` where a model turn is already paid (Codex/Cursor) | Every note via Claude | Keeps D17 zero-turn capture; words kept | Yes |
| D33 | Later enrichment (mode b) by the person's own Claude session via MCP prompt `enrich_notes`, using one subagent; run on demand or by the person's own schedule | Model inside easy-note (needs network + key) | Server stays model-free and local; same pattern as easy-personal FR-075 | Yes |
| D34 | AI detail in `<notes dir>/.easy-note/ai/<id>.json` only: `title` ≤ 120, `kind` ∈ bug/idea/question/todo/other, `summary` ≤ 600, `tags_suggested` ≤ 5, `based_on`, `model`, `at`; masked; never written into the note | Write into the note's front matter/body | The person's file stays byte-identical | Yes |
| D35 | Deterministic extra fields at capture: `session` (hook notes) and `commit` (short HEAD sha) | None | Give the pass something to look up; no model needed | No |
| D36 | Stale AI detail (hash mismatch) counts as pending; shown with "(stale)" until re-run | Delete on edit | Never silently wrong; never silently lost | No |
| D37 | MCP surface v1 = 7 tools + 1 prompt: + `note_pending_enrich`, `note_enrich`; prompt `enrich_notes` | Keep 5, CLI-only enrichment | The pass runs inside Claude, which reaches easy-note over MCP | Yes |
| D38 | Search also matches AI detail, marked `matched_in: ai`; stats never use AI fields except `enriched` count | Ignore AI in search | Finding a note by its AI title is the point | No |

Source enum unchanged; AI detail has no `source` (it is not a note).


## Round 3c — system tray (2026-11-14)

Owner: "the tray come in the system I mean, not from the browser" — a real OS tray icon is wanted.

Verified (`easy-personal-mcp/BRAINSTORMING_LOG_WATCH.md`, commit `331532b` "Watch service + tray icon: Stage 0.5 grilling record (D1-D7), path choice pending"): easy-personal's tray is a real system tray (D1 "easy-personal-rs + Tauri (tray + notifications)"), but it is at Stage 0.5, the path is not chosen, autostart is an open question, and the tray menu design waits on the owner (D7). It has no task and no date.

| # | Finding | Kind | Resolution | Status |
|---|---|---|---|---|
| G39 | A wanted v1 feature would depend on another plug-in's unscheduled work | Dependency / microkernel | Own tray in v1 (D39) | ✅ Resolved |
| G40 | Two tray icons when easy-personal's ships | Clutter | Person keeps one: `easy-note uninstall tray`, or easy-personal's F3 item; documented (D41) | ✅ Resolved |
| G41 | GNOME shows tray icons only with the AppIndicator extension; AppIndicator menus open on left click (no click action) | Platform | Click → menu → "New note" (2 clicks); hotkey stays the 1-press path; `doctor` warns if no tray host (D40) | ✅ Accepted |

| # | Decision | Alternative not chosen | Why | Owner's call? |
|---|---|---|---|---|
| D39 | `easy-note tray`: own system tray icon in v1, opt-in, Python `pystray` as optional extra `easy-note-mcp[tray]`; menu: New note (opens `quick`), last 5 notes (open file), Open notes folder, Quit | Wait for easy-personal (F3); Tauri/Rust tray | Real tray now; same language as the rest; no cross-plug-in dependency | Yes |
| D40 | `easy-note install tray` writes one XDG autostart file `~/.config/autostart/easy-note-tray.desktop` (Linux; macOS LaunchAgent plist in v1.1); `uninstall tray` removes it (DDR 0003 rules) | Always-on by default | Resident process only if the person wants it (~30–50 MB RAM idle, estimate) | No |
| D41 | F3 stays: when easy-personal's tray ships it may add a "New note" item; README says to keep one tray | Merge trays | Each plug-in works alone | No |
