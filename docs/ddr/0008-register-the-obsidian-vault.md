# 0008. `install obsidian` registers the notes dir as an Obsidian vault

**Status**: Accepted (owner, chat 2026-11-19: "can we setup this one by script … we can not require the user do the tech jobs"; picks: ask to close Obsidian when it is open; one `easy-note setup` command)
**Date**: 2026-11-19
**Deciders**: thunderkds
**Related**: PRD US-017, FR-034, FR-040, FR-041, NFR-003 · amends DDR 0001 (write targets), DDR 0006

---

## Context

`obsidian://open?path=` opens a file only inside a vault Obsidian knows. On the owner's machine the only vault
was a new `~/easy-notes/notes` folder ("Create new vault" makes a subfolder), so the dashboard did not open.
Obsidian has no command or URI that adds a vault; its list is `obsidian.json` in its config dir. Obsidian keeps
that list in memory while it runs and can save over an outside change.

DDR gate: surprising (we edit another app's config) ✅, genuine trade-off ✅, hard to reverse — no. 2 of 3 → DDR.

## Decision

1. `install obsidian` creates `<notes dir>/.obsidian/` and adds one entry for the notes dir to `vaults` in
   `$XDG_CONFIG_HOME/obsidian/obsidian.json` (else `~/.config/obsidian/obsidian.json`; native and snap). It only
   adds: every other key and vault is kept, the old file is backed up, the write is atomic.
2. It writes nothing when a vault is already the notes dir or a parent of it, when the file is not valid JSON, or
   when a process named `obsidian` runs (it asks the person to close Obsidian; owner's pick).
3. `uninstall obsidian` never edits the vault list and never removes `.obsidian/`: once added, the vault is the person's.
4. `easy-note setup` runs `install launcher`, `install obsidian` and `install tray`, not `install claude`. *(2026-11-26, HUN-303: it also runs `install hotkey`, DDR 0003 item 8.)*

## Alternatives Considered

| Alternative | Why not chosen |
|---|---|
| Tell the person to open the folder as a vault (README) | A manual step; the owner's goal is no tech steps |
| Close and reopen Obsidian ourselves | Closes the person's window without asking |
| Write while Obsidian runs | Obsidian may save over it; the setup fails silently |
| Remove our vault on uninstall | The person may keep using the vault; removing it is a surprise |

## Consequences

- Positive: `setup` then a launcher click opens the dashboard with the notes.
- Negative: we depend on the `obsidian.json` shape (`vaults` → id → `path`, `ts`, `open`), which Obsidian does
  not document; a bad or changed file is left alone and named. Flatpak's config dir is not covered.
