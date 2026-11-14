# BRAINSTORMING_LOG_capture.md
**Generated**: 2026-11-14
**Task / Context**: Pressure test of the plan's core promise — "taking a note is easy". Owner request 2026-11-14: "investigate to make sure we will be easy to note, brainstorm the solution". Scope: the capture path only, from the moment a thought appears to the moment the note is saved.
**Skill**: `Skill({ skill: "brainstorming" })` — tier **Standard** · **Mode**: autonomous (decisions made and recorded, owner reviews later)
**Changes**: GRILL G19–G28 and D17–D26 (D8, D13 revised) · PRD FR-018–FR-024, NFR-009/010 · DDR 0003 · tasks T011–T013

---

## Verdict first

**The first plan is not easy enough.** It made capture *possible* everywhere a terminal or a local agent runs,
but each path has a step or a failure that makes a person skip the note. Five problems, worst first:

1. **In an agent chat, a note costs a model turn.** "note: X" goes to the model, which must decide to call
   `note_add`, may ask for tool permission the first time, spends tokens, and adds the note to the task's context.
   Mid-task this derails the agent. This is the most used path, and it is the slowest.
2. **The first note fails.** FR-001 refused every write until `easy-note init` ran. The first thought is lost.
3. **A secret refusal loses the thought.** D8 refused the whole note. The person must retype it.
4. **The terminal command is long and fragile.** `easy-note add "..."` is 15 characters before the text, and the shell
   eats it: an unquoted `#flaky` becomes a comment, `!` triggers history expansion, `'` breaks quotes.
5. **No path from the editor or from a phone.** Mid-task in VS Code/JetBrains the person must switch to a terminal; away
   from the desk there is no capture at all.

## What makes quick capture easy (principles)

From how well-liked quick-capture tools behave (Drafts opens to a blank page; Things / Raycast quick entry is one
global hotkey; `jrnl` takes the words after the command with no quotes; Obsidian daily-note append). This is general
product knowledge, not a measured study.

| # | Principle | Test for each path |
|---|---|---|
| P1 | One gesture from where you already are | ≤ 1 prefix or 1 hotkey, no app or window switch |
| P2 | Zero decisions at capture | no title, folder, tag or ID to choose; context is automatic |
| P3 | Never lose the input | no refusal and no setup step can drop the typed text |
| P4 | Instant confirmation | one line back in < 1 s saying it is saved and where it went |
| P5 | Organize later | edit, tag, delete happen after, never at capture |
| P6 | Do not disturb the task | an agent mid-task keeps its context; no model call for a note |

## Friction map — first plan vs. goal

Overhead = what the person types or does besides the note text itself.

| Situation | First plan | Overhead | Breaks | Goal |
|---|---|---|---|---|
| S1 Claude Code chat | "note: X" → model → `note_add` | model turn, maybe a permission prompt, tokens, context noise | P4, P6 | prefix → saved by a hook, no model call |
| S2 Codex / Cursor chat | same as S1 | same | P4, P6 | same as S1 (no hooks there, accepted) but a short, fixed phrase |
| S3 Terminal | `easy-note add "X"` | 15 chars + quoting | P1, P3 | `n X` |
| S4 First use | `easy-note init` first, or refused | 1 extra command | P3 | first note just works |
| S5 Editor mid-task | switch to terminal | window switch + cd | P1 | one editor key → one-line box, repo filled |
| S6 Anywhere on desktop | not covered | — | P1 | one global hotkey → one-line box |
| S7 Phone / other device | not covered | — | P1 | drop a file in the synced folder's `inbox/` |
| S8 Secret in text | whole note refused | retype | P3 | save it masked, say so |
| S9 claude.ai cloud session | not reachable | — | — | still out of scope (local-first) |

## Alternative Paths

| Option | Name | Summary | Invasiveness | Code Volume | Regression Risk | Recommended? |
|--------|------|---------|-------------|------------|----------------|--------------|
| A | Keep MCP + CLI, polish docs | Better tool description, README aliases | Low | ~0 | Low | |
| B | Capture layer around the same core | Claude Code prompt hook (no model call), `n` short command, zero-setup first note, mask-not-refuse, `quick` one-line box for hotkeys/editors, read-only `inbox/` | Medium (one new write target: user-level Claude Code settings, by person-run `install`) | ~+600 lines | Low–Med | ✅ Yes |
| C | Always-on capture daemon | Background service with global hotkey, tray icon, HTTP endpoint for phone | High | ~+2k lines, OS-specific | High | |
| D | Ride on easy-personal's tray / watch service | Add a "New note" item to easy-personal's tray | Medium | ~300 lines in easy-personal | Med | |

### Option A — Polish docs only
**Pros**: no new surface. **Cons**: S1's model turn stays; S4, S5, S7, S8 stay broken.
**Why it might fail**: the owner asked for "easy", and the most used path is still the slowest.

### Option B — Capture layer (chosen)
**Approach**:
1. **S1 prompt hook.** `easy-note hook-prompt` is a Claude Code `UserPromptSubmit` hook. A prompt that starts with the
   capture prefix (default `/note `; see D18) is saved and **blocked**: the model never sees it, and Claude Code shows the
   block reason, e.g. `✓ noted 20261114-164713-a1b2 · easy-ui-mcp · T120`. The hook gets the session's `cwd` in its
   input, so the repo tag is exact (no T001 cwd guess for this path). Every other prompt passes untouched; the hook
   imports only the stdlib on that path, so it adds little latency (NFR-009). Same pattern as easy-memory's
   `hook-prompt` (`easy-memory-mcp/easy_memory.py:850`, UserPromptSubmit, timeout 5 s).
2. **`easy-note install claude`** adds that hook to user-level `~/.claude/settings.json`: shows the diff, keeps a
   timestamped backup, writes atomically, `uninstall claude` removes exactly its entry. Same pattern as easy-memory's
   installer (`easy_memory.py:1054`–`1113`). DDR 0003.
3. **S2.** No hooks in Codex/Cursor (`easy-kit/AGENTS.md` "Non-Claude CLIs"). The `note_add` description names the
   phrase "note:" so the model calls it at once and replies in one line. Accepted cost: one model turn.
4. **S3 `n`.** A second console script `n` = `easy-note add`. All words after it are the note (`n retry bug on firefox`).
   With no words, it reads one line from a prompt (no shell parsing, so `#`, `!`, `'` are safe). Cold start < 150 ms:
   the CLI never imports the MCP SDK (NFR-010).
5. **S4 zero setup.** No config → use `~/easy-notes`, create it, print where once. `init` only to choose another folder.
6. **S8 mask, not refuse.** A secret-looking span is replaced by `[masked:<kind>]` before the write, and the confirmation
   says `1 secret masked`. The thought is kept; the secret never reaches disk.
7. **S5/S6 `easy-note quick`.** Opens a native one-line box (macOS `osascript`, Linux `zenity` or `kdialog`); if none
   exists, a terminal prompt. Saves, shows a notification line, exits. The person binds it once: to a global hotkey
   (macOS Shortcuts/Raycast, GNOME/KDE custom shortcut), and in editors with the snippet that passes the workspace
   folder (`easy-note quick --cwd ${workspaceFolder}` in a VS Code task + keybinding; JetBrains External Tool with
   `$ProjectFileDir$`) so the repo tag is exact. Without `--cwd` the note has no repo (never guessed).
8. **S7 `inbox/`.** Any `.md` or `.txt` file the person drops into `<notes dir>/inbox/` (iOS Shortcuts "Save File",
   Obsidian mobile, any sync app) is read as a note: `source: inbox`, created from a `YYYYMMDD-HHMMSS` file-name prefix
   or else the file mtime, tags from `#words`. easy-note never rewrites inbox files (read tools stay read-only, DDR 0001);
   `rm` moves them to trash like any note. Sync is the person's tool, so local-first holds.

**Pros**: every situation except cloud sessions gets ≤ 1 gesture; the most used path (S1) drops the model turn;
no input is ever lost; no daemon; core unchanged.
**Cons**: one new write target (user-level Claude Code settings) and a hook on every prompt; OS dialog code per platform;
inbox notes have no repo context.
**Why it might fail**: (a) the hook may not see `/note ...` raw if Claude Code handles unknown slash commands first
→ T001 checks; fallback prefix `;n ` (D18). (b) Two prompt hooks (easy-memory + easy-note) both run on every prompt →
budget each < 50 ms on the pass path. (c) `zenity` missing on minimal Linux → terminal fallback.

### Option C — Capture daemon
**Pros**: one hotkey with no OS setup; HTTP endpoint for the phone.
**Cons**: a long-running service, OS-specific hotkey code, a listening port for phone capture (breaks 127.0.0.1-only
unless LAN exposure, which breaks local-first).
**Why it might fail**: large, fragile, and the phone path needs network exposure.

### Option D — easy-personal tray
**Pros**: a tray icon may already exist (easy-personal `BRAINSTORMING_LOG_WATCH.md`, still in grilling).
**Cons**: makes capture depend on another plug-in (microkernel break: easy-note must work alone, NFR-008).
**Why it might fail**: easy-personal's tray is not decided yet; coupling two plug-ins for one menu item.
Kept as a possible *easy-personal* follow-up that only shells out to `easy-note quick` (F3), never required.

---

## 50% Rule Check

Half of B: hook + `n` + zero setup + mask (~300 lines). That fixes S1, S3, S4, S8 — the paths used most. `quick` and
`inbox/` cover S5–S7 for ~300 lines more. Kept both, but ranked them P2/P1 so the vital slice ships first:
T011 (hook, P1) → T007 changes (`n`, zero setup) → T013 (inbox, P1) → T012 (quick, P2).

## Adversarial Review (Option B)

| Attack | Answer |
|---|---|
| "A hook on every prompt slows every prompt" | Pass path = read stdin JSON, check a prefix, exit; stdlib only. NFR-009: p95 < 50 ms. T011 measures. |
| "`/note` may clash with a real command" | It is a reserved name only if the person has another `/note`; `install` checks `~/.claude/commands/` and the prefix is configurable. |
| "Blocking the prompt hides it from the transcript" | Intended: a note must not enter the agent's context (P6). The saved file is the record. |
| "Masking may cut a real word" | Only declared patterns (keys, tokens, private keys, JWT, `password=`); the confirmation names what was masked; `edit` can fix. |
| "Inbox files without an ID break DDR 0002" | Inbox is a named, documented part of the format: id = `inbox/<file stem>`; never rewritten. |
| "Writing `~/.claude/settings.json` breaks DDR 0001" | It does, so DDR 0003 records it, with easy-memory's installer as the family precedent; person-run, diffed, backed up, reversible. |

## Decision

**Option B**, Supervisor, 2026-11-14 (autonomous; owner to confirm). Details in GRILL D17–D26, DDR 0003.

> **Round 3 (2026-11-14):** one press / one click and the tray were re-examined in `BRAINSTORMING_LOG_oneclick-enrich.md` Part 1. Hotkey + launcher icon in v1; tray as an easy-personal item (F3), own tray v1.1 fallback. *Superseded by round 3c: own system tray in v1 (GRILL D39).*

## Cut List

- A daemon, tray icon or OS hotkey code of its own (OS shortcut settings call `easy-note quick`).
- Phone capture over network, email-to-note, chat bots.
- Rewriting or normalizing inbox files.
- Voice notes, screenshots, clipboard auto-capture.
- Codex/Cursor zero-model-call capture (no hook system there).
- Note on the current file/line from the editor (path storage is banned, FR-004; revisit with repo-relative paths later).
