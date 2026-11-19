# 0006. Obsidian is the dashboard; easy-note draws no window of its own

**Status**: Accepted; item 2 superseded by DDR 0009 when built with `window` and a display (owner, chat 2026-11-19: "how about apply the obsidian as brain"; "let create the tasks")
**Date**: 2026-11-19
**Deciders**: thunderkds
**Related**: PRD US-017, FR-034–FR-036, FR-039 · amends DDR 0001 (write targets), DDR 0004 (where AI detail is visible) · design `docs/design/`

---

## Context

The owner's design (`docs/design/Main.dc.html`) adds a dashboard: counts, search, filters, a note list and an
AI detail panel, opened from the tray and the app launcher, "the native way". A window of our own means a webview
(`wry` + WebKitGTK) or GTK widgets: a large new dependency, system build libraries, and a UI we maintain. The notes
are already plain Markdown with YAML front matter that Obsidian reads as properties (US-004, DDR 0002). Obsidian's
core Bases view gives tables, filters and groups over those properties with no plugin.

DDR gate: surprising (the dashboard is another app) ✅, genuine trade-off ✅, hard to reverse — no. 2 of 3 → DDR.

## Decision

1. The dashboard is `<notes dir>/easy-note.base`, an Obsidian Bases file written only by `install obsidian` (FR-035).
2. `easy-note dashboard`, the launcher and the tray's "Open dashboard" open it through `obsidian://open?path=…` with `xdg-open` (FR-034). Obsidian draws the window; easy-note draws none.
3. AI detail stays in `.easy-note/ai/<id>.json` (DDR 0004). When the notes dir is in a vault, easy-note also writes a read-only view `<notes dir>/ai/<id>.md` that links back with `[[<id>]]`, so Obsidian shows it in backlinks and Bases (FR-036). The note file is never touched.
4. `quick` keeps zenity/kdialog this round; the designed capture box is a later task.

## Alternatives Considered

| Alternative | Why not chosen |
|---|---|
| Own window, `wry` webview (the owner's first pick) | WebKitGTK build libs and a GUI toolkit for a read-mostly view Obsidian already gives; replaced by the owner's Obsidian choice |
| Browser tab on `127.0.0.1` | Opens a port and a tab, not an app |
| Add an `ai:` key to the note front matter | Changes the person's file (DDR 0004 item 4) |

## Consequences

- Positive: no GUI crate; the person browses, links and graphs notes in an app they use; the format contract is unchanged.
- Negative: Obsidian is a separate, closed-source install; trash/undo and "add AI tags" are Obsidian's own edits or the CLI; `ai/` holds one more file per enriched note in a synced folder.
