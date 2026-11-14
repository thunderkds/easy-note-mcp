# BRAINSTORMING_LOG_oneclick-enrich.md
**Generated**: 2026-11-14
**Task / Context**: Round 3. Owner asked (2026-11-14): "how about the shortcut, the icon in the tray, or any way to just click or press to note … we suggest to use claude to note for the auto detail, or just a simple text line and claude sub will scan to add more detail for each note. is it good, let check it more clearly".
Reading of "claude sub": a Claude subagent (or a separate Claude pass) that runs after capture and adds detail. Not a model inside easy-note.
**Skill**: `Skill({ skill: "brainstorming" })` — tier **Standard** · **Mode**: autonomous
**Changes**: GRILL G29–G38, D27–D38 · PRD US-014–US-016, FR-025–FR-032 · DDR 0003 amended, DDR 0004 new · tasks T012 widened, T014, T015

---

## Verdict first

1. **One press: yes, in v1, with no background process.** A global hotkey bound to `easy-note quick` is the one-press
   path. A launcher icon (pin it to the dock) is the one-click path. The OS keeps both resident, so easy-note runs only
   while you type. **A tray icon: yes, but not a second one.** easy-personal already plans a tray (Tauri,
   `easy-personal-mcp/BRAINSTORMING_LOG_WATCH.md` D1). The best tray is a "New note" item there, which runs
   `easy-note quick` when easy-note is installed. easy-note gets its own tray only if that tray does not ship (v1.1, opt-in).
2. **Claude detail: mode (b) is good, mode (a) is not.** Capture stays plain and instant (no model call). Later, your own
   Claude session runs an `enrich_notes` prompt that scans new notes and adds detail. Detail goes into a **separate,
   labelled AI file**. Your words are never changed. easy-note itself still runs no model. Mode (a) — every note through
   Claude — brings back the model turn, the delay and the derailed task that round 2 removed, and it lets the model
   rewrite your words. One exception keeps the free win of (a): where a model turn already happens (Codex/Cursor
   `note:`), `note_add` may take an optional `detail`, stored the same separate way.

---

## Part 1 — One press or one click

Facts that limit the options:
- A global hotkey needs something resident that listens for keys. The OS shortcut service already is (GNOME/KDE custom
  shortcuts, macOS Shortcuts/Raycast, Windows `.lnk` shortcut key). A tool of our own would need a daemon plus OS
  permissions (macOS Accessibility/Input Monitoring; Wayland blocks global key grabs by apps).
- A tray icon needs a resident GUI process. GNOME shows tray icons only with the AppIndicator extension (Ubuntu ships it;
  stock GNOME does not).
- The owner works on Linux/GNOME (easy-personal `BRAINSTORMING_LOG_WATCH.md`: "GNOME logout kills the tray") and is
  already building one tray in easy-personal.

| Option | Name | Summary | Resident process | Code Volume | Risk | Recommended? |
|---|---|---|---|---|---|---|
| H | OS hotkey → `quick` | Person binds a key to `easy-note quick`; `easy-note hotkey` prints the exact commands for the detected desktop | None of ours | ~80 lines + docs | Low | ✅ v1 |
| L | Launcher icon | `easy-note install launcher` writes one `.desktop` file (Linux); person pins it to the dock; macOS via a Shortcuts action | None | ~60 lines | Low | ✅ v1 |
| P | easy-personal tray item | "New note" item in easy-personal's planned tray that runs `easy-note quick` if found | easy-personal's | ~30 lines there | Low | ✅ F3 (easy-personal board) |
| T | easy-note's own tray | `easy-note tray`: menu New note / Recent / Open folder | Ours | ~300 lines + GUI deps | Med | v1.1 opt-in, only if P does not ship |
| D | Own hotkey daemon | Listen for keys ourselves | Ours, with OS input permissions | ~800 lines, per OS | High | ❌ |

Adversarial notes:
- *"Binding a key by hand is not easy."* It is once, ~30 s; `easy-note hotkey` prints a copy-paste command
  (`gsettings set …` for GNOME). Writing dconf ourselves would be one more write target for a one-time step — not worth it.
- *"Two trays for one person is clutter."* Hence P first; T only as a fallback.
- *"`quick` must open fast or the thought is gone."* Box visible in < 300 ms (NFR-011): the `quick` path imports neither
  the MCP SDK nor YAML before the box shows; save happens after Enter.
- *"Wayland focus."* A window started by a hotkey may open unfocused on some compositors; `zenity` takes focus; T012 checks.
- Windows: the box falls back to stdlib `tkinter`; a `.lnk` with a shortcut key is the hotkey. Best effort (NFR-006).

Context: a hotkey or launcher has no project folder, so the note has no repo — unless it comes from an editor snippet
(`--cwd ${workspaceFolder}`). New in round 3: `quick` may use the **last context** — repo/branch/task of your most recent
CLI or hook note if it is under 2 hours old — shown in the box as `easy-ui-mcp · T120 (Tab to clear)`. Deterministic,
visible, removable. Alternative: no repo at all (round 2). Chosen because most hotkey notes happen while working in
the repo you just used.

> **Round 3c (2026-11-14):** owner confirmed a real OS system tray matters. easy-personal's tray is a real Tauri system tray but only at Stage 0.5 (path pending, no task; commit `331532b`). Depending on it would put a wanted v1 feature behind another plug-in's unscheduled work, so option **T moves into v1** (opt-in, pystray, XDG autostart), P stays an optional follow-up. GRILL D39–D41.

## Part 2 — Detail by Claude

What deterministic code already adds at capture (no model): time, repo, remote, branch, task, tags, source. Round 3 adds
two cheap deterministic fields: `session` (Claude Code session id from the hook event) and `commit` (short HEAD sha).
What only a model can add: a clear title, "what this refers to" (from the session around the note), kind (bug / idea /
question / todo), suggested tags, links to related notes.

| Option | Name | Summary | Capture cost | Words kept? | Deterministic core? | Recommended? |
|---|---|---|---|---|---|---|
| E1 | Enrich at capture | Every note goes through Claude, which writes the detailed note | 1 model turn per note, seconds, task context | Model may rewrite | No | ❌ |
| E2 | Later pass, separate AI file | Plain capture; `enrich_notes` prompt in your own Claude session (subagent) adds detail into `.easy-note/ai/<id>.json` | 0 at capture | Yes, byte-identical | Yes | ✅ |
| E3 | Later pass, edit the note | Same pass, but writes detail into the note file | 0 at capture | Note file changed | Partly | |
| E4 | Model inside easy-note | Server calls an API to enrich | 0 at capture | Yes | No; needs network + key | ❌ |
| E5 | Detail only where a turn is paid | `note_add(detail=…)` in Codex/Cursor; stored like E2 | 0 extra | Yes | Yes | ✅ with E2 |

### E2 — how it works
1. Capture as round 2. Hook notes also store `session` (opaque id) so the pass can find the conversation.
2. The person runs the MCP prompt `enrich_notes` in Claude Code (`/mcp__easy-note__enrich_notes`), any time or from a
   scheduled `claude -p` they set up themselves. Precedent: easy-personal's `investigate` prompt — "the prompt text is
   fixed in the code; the server runs no model … the owner's own Claude Code session does the reasoning"
   (`easy-personal-mcp/PRD.md` FR-075) and its `save_report` write tool (DDR 0003 there).
3. The prompt tells Claude to start one subagent, call `note_pending_enrich` (notes with no AI file, or an AI file whose
   `based_on` hash no longer matches the note), and for each note: read the note, read the session transcript around
   the note time if `session` is set and the transcript still exists (Claude Code deletes them after 30 days), and call
   `note_enrich(id, title, kind, summary, tags_suggested)`.
4. `note_enrich` writes only `<notes dir>/.easy-note/ai/<id>.json`: caps (title ≤ 120, summary ≤ 600 chars, ≤ 5 tags),
   secrets masked, `based_on` = sha256 of the note text, `model` and `at`. It never opens the
   note file for writing.
5. Reads show it labelled: `AI detail (not your words) · claude-… · 2026-11-15`. Search matches AI detail too, and marks
   the hit `matched_in: ai`. `stats` never counts anything from AI detail except "enriched: N of M".
6. Accepting is a person action: `easy-note tag <id> --accept-ai` copies suggested tags into the note; `easy-note ai rm <id>`
   deletes the AI file.

Cost estimate (supervisor guess, T014 measures): ~1–3k tokens per note with a transcript read; at 15 notes/week that is
~15–45k tokens/week, run in one batch. On a subscription plan this is usage-window cost, not dollars.

When it runs: only when the person runs the prompt (or their own schedule). Never at capture, never by easy-note.

How it squares with the family rules: the server stays deterministic and model-free (NFR-007 unchanged); model text is
kept apart, labelled and capped, the same narrow-exception shape as easy-personal DDR 0003; capture keeps zero model
turns (D17). Recorded as DDR 0004 because `note_enrich` is a second MCP write tool and AI text enters the notes dir.

Adversarial notes:
- *"AI detail is wrong."* It is labelled, never replaces your words, never feeds counts, and one command removes it.
- *"Transcript text leaks into a synced folder."* Summary cap 600 chars, masked; the prompt says "explain, do not quote";
  `enrich_notes` is opt-in.
- *"Two places for one note."* The AI file is keyed by note id and hash; `rm` of a note moves its AI file to trash too.
- *"Why not let the model fix typos in the note?"* Your words are the record (P3). The AI title covers readability.

### Worked example (round 3b, written for the owner)

1. Tuesday 16:47, Claude Code in `easy-ui-mcp`, branch `fix/t120-firefox`, the agent is chasing a flaky test. The owner types `/note retry bug only on firefox`. The hook saves `~/easy-notes/2026/11/20261114-164713-a1b2.md` (front matter: repo, branch, task `T120`, session, commit; body: the one line). The agent never sees it.
2. Friday, any Claude Code window: `/mcp__easy-note__enrich_notes`. The owner starts it; nothing runs by itself.
3. Claude starts one subagent. It asks easy-note for notes without detail and gets the line + repo, branch, task, time, session.
4. The subagent opens that session's transcript and reads only the 10 minutes before 16:47 (≤ 40 entries).
5. It calls `note_enrich`; easy-note writes `.easy-note/ai/20261114-164713-a1b2.json` (title, kind, summary, suggested tags). The note file is untouched.
6. `easy-note show 20261114-164713-a1b2` prints the line, then the labelled AI block.

Gaps this exposed, fixed in FR-031: (a) how much transcript to read → 10 minutes, ≤ 40 entries (cost cap); (b) terminal and hotkey notes have no session → look for exactly one transcript of the same repo in that window, else enrich from the line only. First run: Claude Code may ask permission to read `~/.claude/projects/`.

## 50% Rule Check

Vital slice: H + L (one press, one click) and E2 with only `title`, `kind`, `summary` (cut `related` and transcript
reading) is ~half the work. Kept transcript reading because "what did this note mean" is the main value the owner asked
for ("auto detail"); cut `related` to v1.1.

## Decision

H + L in v1 (T012), P as easy-personal follow-up F3 (promoted from optional), T in v1.1 only if P does not ship (T015).
E2 + E5 in v1 (T014), DDR 0004. E1, E3, E4, D rejected. Supervisor, 2026-11-14, autonomous; owner to confirm.

## Cut List

- Own hotkey daemon; writing dconf/OS shortcut settings for the person.
- Two trays; easy-note's own tray before v1.1.
- Any model call inside easy-note; any edit of the note file by AI.
- `related` links in AI detail (v1.1).
- Automatic background enrichment run by easy-note.
