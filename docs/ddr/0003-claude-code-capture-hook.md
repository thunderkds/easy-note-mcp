# 0003. Claude Code capture hook in user-level settings

**Status**: Proposed (Supervisor, autonomous run; owner to accept)
**Date**: 2026-11-14
**Deciders**: thunderkds
**Related**: PRD FR-018, FR-019, NFR-009, US-013 · GRILL G19, G20, G26, G27, D17, D18, D25 · `BRAINSTORMING_LOG_capture.md` · amends DDR 0001

---

## Context

In Claude Code the most common note path went through the model: "note: X" → the model calls `note_add`. That costs a
model turn, maybe a permission prompt, tokens, and it puts the note into the running task's context. A
`UserPromptSubmit` hook can save the note and block the prompt, so the model never sees it. Installing the hook means
writing user-level `~/.claude/settings.json`, which DDR 0001 did not allow.

Family precedent: easy-memory-mcp already runs a Python `UserPromptSubmit` hook (`easy_memory.py:850`, timeout 5 s) and
installs it into user-level Claude Code settings with a diff, a timestamped backup and an `uninstall`
(`easy_memory.py:1054`–`1113`; easy-memory `PROJECT_SPEC.md` Critical Constraints: "the installer's reversible edit of
user-level Claude Code settings").

DDR gate: hard to reverse (the hook runs on every prompt in every session) ✅, surprising (a note tool edits Claude Code settings) ✅, genuine trade-off ✅. 3 of 3.

---

## Decision

1. `easy-note hook-prompt` is a `UserPromptSubmit` hook. Prompt starts with the prefix (default `/note `) → save with `source: hook`, context from the event's `cwd`, return `{"decision":"block","reason":"✓ noted <id> · <repo> · <task>"}`. Otherwise no output, exit 0.
2. The pass path imports only the stdlib and must never block or fail a prompt; any error lets the prompt through.
3. `easy-note install claude` adds exactly one hook entry (command `easy-note hook-prompt`, timeout 5 s) to `settings.json` and writes `commands/note.md`, both in the Claude Code config dir (`$CLAUDE_CONFIG_DIR` when set, else `~/.claude`) (fallback that asks the model to call `note_add`, used only if the hook cannot see `/note`). It shows the diff, keeps a timestamped backup, writes atomically, keeps a symlinked settings file a symlink, and never changes another key or hook (easy-memory's entries stay).
4. `easy-note uninstall claude` removes exactly those two things.
5. Only the person runs `install`/`uninstall`. No MCP tool and no other command touches Claude Code settings. Project-scope settings are never written (the kernel's `setup.sh` owns them).
6. If T001 shows the hook does not receive `/note …` raw, the default prefix becomes `;n ` and `note.md` is not written. *(T001/HUN-251, 2026-11-14: the hook receives `/note …` raw and can block it; the default stays `/note `.)*
7. *(Amended 2026-11-15, owner-approved)* Claude Code reads its config from `$CLAUDE_CONFIG_DIR` when that is set (the owner's machine uses `~/.claude-personal`). `install`/`uninstall claude` resolve the target dir the same way and print it in the diff header; with neither set nor present they create `~/.claude`.

---

7. *(round 3)* The same rules cover `easy-note install launcher` / `uninstall launcher`: exactly one file, `~/.local/share/applications/easy-note.desktop`, diff shown, removable. OS shortcut settings are never written; `easy-note hotkey` only prints commands. *(Round 3c)* Same rules for `install tray` / `uninstall tray`: exactly `~/.config/autostart/easy-note-tray.desktop`.
8. *(Amended 2026-11-26, owner-approved in chat, HUN-303)* Item 7's "OS shortcut settings are never written" no longer holds for GNOME: `easy-note install hotkey`, also a step of `setup`, adds exactly one custom shortcut that runs `easy-note quick` with Ctrl+Alt+N, and `uninstall hotkey` removes it. Reason: after an install the owner pressed Ctrl+Alt+N and nothing opened. It never changes another entry, never takes a used key, and keeps an entry that is not ours. `easy-note hotkey` still only prints. Other desktops' settings are never written.

## Alternatives Considered

| Alternative | Pros | Cons | Why not chosen |
|-------------|------|------|----------------|
| Hook + person-run install (chosen) | Zero model turns; one setup command; reversible | Writes Claude Code settings; runs on every prompt | **Selected** |
| Print the JSON snippet, person pastes it | No settings write by the tool | Hand-editing JSON is the friction we are removing; paste errors break settings | Fails "easy" at setup |
| Model + `note_add` only (round 1) | No settings write | Model turn per note; derails tasks | Main friction G19 |
| Slash command file only | No hook | Still a model turn | Same as above |

---

## Consequences

### Positive
- A Claude Code note costs no model turn and never enters the task's context.
- Repo tag is exact (the hook input carries `cwd`).

### Negative (accepted trade-offs)
- A second family tool edits user-level Claude Code settings.
- Two prompt hooks (easy-memory + easy-note) may run on every prompt; each must stay fast (NFR-009).

### Follow-up
- [ ] T001: verify raw `/note` visibility, `decision: block` display, and behaviour for a prompt queued while the agent works.
- [ ] T011: tests for install idempotence (twice = once), uninstall leaves easy-memory's entries byte-identical, backup never overwritten.
