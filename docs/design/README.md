# Design source: easy-note — Graphite Mint

Copied 2026-11-19 from the owner's design canvas (claude.ai artifact `4TjJKSBDDNRR7KSUwz2REe`, version 1795063646-6bf9).
The `.dc.html` files are design artboards, not app code: they hold sample data and run only in the design editor.

| File | What it shows | Used by |
|---|---|---|
| `Icons.dc.html` | App icon (64-unit SVG, 16 px variant) and tray icon (16-unit outline, "just saved" badge) | FR-039 — copy the SVG paths exactly; `{{accent}}` = `#3CF5A8` |
| `TrayMenu.dc.html` | Tray menu order: New note · Open dashboard · recent 5 · Open notes folder · Quit | FR-039 |
| `Main.dc.html` | Dashboard layout and states (counts, filters by kind, day groups, AI detail panel) | FR-034/FR-035: the Bases views mirror its groupings; the window itself is not built (DDR 0006) |
| `Capture.dc.html` | Capture box states | `quick` opens this dialog in its own small window in a `window` build (HUN-289, FR-022); zenity/kdialog stay as the fallback |
| `graphite-mint-tokens.json` | Colour, type and spacing tokens | Icon colours |
