# 0009. easy-note draws its own dashboard window (webview, feature `window`)

**Status**: Accepted (owner, chat 2026-11-19: "when I run all, I command easy-note dashboard, but it show the obsidian, not the note board that I expect"; "that is what I expected" + the design canvas link; pick "Build own window"; "let merge and start the dashboard"). Direction Option A taken under `[admin]` (`BRAINSTORMING_LOG_dashboard-window.md`), ⚠ owner may still say B or C.
**Date**: 2026-11-19
**Deciders**: thunderkds
**Related**: PRD US-017, FR-034, FR-042–FR-044, NFR-001, NFR-006, NFR-013 · supersedes DDR 0006 item 2 · design `docs/design/Main.dc.html` (board "Dashboard (clickable)")

---

## Context

DDR 0006 made Obsidian the dashboard. The owner ran `easy-note setup` and `easy-note dashboard` on the real
machine (HUN-277): Obsidian opened the Bases table, and the owner said it is "not the note board that I expect".
The expected board is the design canvas: counts strip, search, kind filters, notes grouped by day, a detail panel
with the AI detail, and actions. A Bases table cannot draw that layout.

The core already reads everything the board shows: `Store::iter_notes`, `query::list_notes_in`, `ai::read`
(`src/core/store.rs`, `src/core/query.rs`, `src/core/ai.rs`). The machine has WebKitGTK 4.1 (2.52.6) and GTK 3.24.

DDR gate: surprising (reverses an accepted DDR) ✅, genuine trade-off ✅, hard to reverse — no (feature-gated). 2 of 3 → DDR.

## Decision

1. A new optional Cargo feature `window = ["dep:wry", "dep:tao"]` adds a native window with a WebKitGTK webview.
   The default build has no GUI crate (same rule as `tray`).
2. The page is the design's HTML/CSS ported to plain JS (no framework, no build step), embedded in the binary and
   served only through a custom `easy-note://` protocol. IBM Plex Sans and Mono (SIL OFL) are bundled; nothing is
   loaded from the network. A strict CSP (`default-src easy-note:`) and a navigation handler refuse every other origin.
3. The page reaches the notes only through IPC (`window.ipc.postMessage` → a pure `window::api` handler over
   `Store`). Every string the handler returns passes `guard::mask_for_output`. The page renders note text with
   `textContent` only.
4. `easy-note dashboard` (and so the launcher and the tray's "Open dashboard") opens this window when the binary is
   built with `window` and a display is present (`WAYLAND_DISPLAY` or `DISPLAY` set). Otherwise it keeps the
   Obsidian path of DDR 0006 / FR-034 unchanged. Obsidian stays a working option; `install obsidian` is unchanged.
5. Delivered in three slices: read-only board (FR-042), actions (FR-043), capture dialog in the window (FR-044).

## Alternatives Considered

| Alternative | Why not chosen |
|---|---|
| Keep Obsidian Bases, restyle the `.base` | Cannot draw the designed board; the owner's complaint stays |
| Native GTK4 widgets (gtk4-rs) | About twice the code; the web layout of the design only gets close |
| Local page on `127.0.0.1` in a browser | Owner already rejected "a browser tab, not an app" (DDR 0006); opens a port that other local users can reach |

## Consequences

- Positive: the launcher opens the board the owner designed; one process, no port; the design's CSS carries over.
- Negative: building with `window` needs `libwebkit2gtk-4.1-dev` (CI gets one extra job); a second UI language (JS)
  to test; some GPU/Wayland setups draw a blank WebKitGTK window (we set `WEBKIT_DISABLE_DMABUF_RENDERER=1` when unset).
- A page bug could send any IPC request, so the Rust handler validates every command and id; IPC is an API.
