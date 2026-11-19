# BRAINSTORMING_LOG.md — easy-note's own dashboard window
**Generated**: 2026-11-19
**Task / Context**: Stage 0 step Direction for the dashboard window (round 5). The owner tried HUN-277 on the real machine; the Obsidian Bases table is "not the note board that I expect". The expected board is the design canvas board "Dashboard (clickable)" (`docs/design/Main.dc.html`). Owner pick 2026-11-19: "Build own window" (over a restyled `.base` or a localhost browser page). Supersedes DDR 0006 item 2.
**Skill**: `Skill({ skill: "brainstorming" })` — tier **Deep** (new GUI dependency, new process model, reverses an accepted DDR). Run under `[admin]`: no stop; direction taken by `docs/claude-md/tracks.md` `### Every user stop and its auto rule` row 4.

---

## The Problem Space

The person wants one window, opened by the launcher, the tray's "Open dashboard" and `easy-note dashboard`, that looks and behaves like `docs/design/Main.dc.html`:

- Header: app mark, notes dir chip, status chips (tray on, hotkey, `N without AI detail`), `+ New note`.
- Counts strip: notes today, notes in total, projects, top 3 projects as bars.
- Notes column: search ("every word must match: text, project, task or tag"), kind filters with counts (All, Bugs, Ideas, Questions, To-dos, No AI detail, Stale), day groups (today / yesterday / earlier), rows with time, first line, where-line, kind chip, AI / stale mark.
- Detail panel: id, source, "Your words", masked notice, context line, tags, AI detail section (model · at, stale warning, title, kind, summary, suggested tags, `Add as tags`), actions `Open in editor`, `Copy id`, `Move to trash` with an Undo toast.
- Capture dialog: Enter save, Shift+Enter newline, Esc cancel, Backspace on empty drops the context chip.

Non-negotiable constraints, verified in the repo:
- One Rust binary, Linux only (DDR 0005). The default build has no GUI crate; GUI parts sit behind a Cargo feature, like `tray = ["dep:ksni", "dep:zbus"]` (`Cargo.toml`).
- No network (README: "It runs no model and makes no network call" except opt-in `ai enrich`). The design loads IBM Plex from Google Fonts (`docs/design/Main.dc.html:11-12`): the window must not.
- Every text that leaves the core is masked (`src/core/guard.rs:268` `mask_for_output`).
- The person's words are never changed except by an explicit action (`Add as tags` changes front-matter tags only; trash moves the file).
- The core already has what the board reads: `Store::iter_notes`, `read`, `update`, `rm_with_ai`, `restore_with_ai`, `add` (`src/core/store.rs`), `query::list_notes_in` with `Filters` (`src/core/query.rs:134-138`), `ai::read` → `Detail { title, kind, summary, tags_suggested, model, at }` plus a stale flag (`src/core/ai.rs:44-55,274`).
- The machine has `libwebkit2gtk-4.1-dev` 2.52.6 and GTK 3.24 (checked with `pkg-config`), so a WebKitGTK webview builds here.

---

## Questions for the User

Decided under `[admin]` (no stop); each is in the report as a taste decision:

1. Fonts: bundle IBM Plex Sans + Mono (SIL OFL, about 200 KB of woff2 in the binary) or use system fonts? → **bundle** (the design is set in Plex; no network allowed).
2. Hotkey chip: the app cannot know which key the desktop bound to `easy-note quick` → show the chip only when `[ui] hotkey = "<keys>"` is set in config; otherwise hide it. Cut from the first slice.
3. Without the `window` feature (or with no display), `easy-note dashboard` keeps today's Obsidian path (DDR 0006) → yes; Obsidian stays a working option (owner, HUN-277 decision).

---

## Alternative Paths

| Option | Name | Summary | Invasiveness | Code Volume | Regression Risk | Recommended? |
|--------|------|---------|-------------|------------|----------------|--------------|
| A | Webview window (wry + tao) | A native window with a WebKitGTK webview. The page is the design's HTML/CSS ported to plain JS, served from the binary through a custom `easy-note://` protocol; JS calls Rust through wry IPC for list / read / actions | Medium (new optional feature `window`) | ~900 Rust + ~700 HTML/CSS/JS | Low for the default build (feature-gated); Medium for the window build | ✅ Yes |
| B | Native GTK4 widgets | gtk4-rs (or relm4) widgets styled with GTK CSS to look like the design | High (new toolkit, all UI in Rust) | ~2,000 Rust | Medium | |
| C | Local page in an app-mode browser | Reuse `axum` (already a dependency) to serve the page on `127.0.0.1:<random>` with a token; open it with `xdg-open` or a Chromium `--app` window | Low | ~600 Rust + ~700 JS | Medium (open port, token, browser differences) | |

### Option A — Webview window (wry + tao)
- **How**: `easy-note dashboard` (feature `window`) opens a tao window with a wry webview. Assets (`index.html`, `app.css`, `app.js`, Plex woff2) are `include_bytes!`. The page talks to Rust only through `window.ipc.postMessage(JSON)`; Rust answers with `evaluate_script`. A strict CSP (`default-src easy-note:`), no remote navigation, no devtools in release.
- **Pros**: the design's HTML/CSS carries over almost line for line, so it looks exactly like the board; one process; no port; WebKitGTK is already on the machine (and on most GNOME desktops).
- **Cons**: build needs `libwebkit2gtk-4.1-dev` (only for `--features window`); WebKitGTK rendering quirks on some GPUs / Wayland (`WEBKIT_DISABLE_DMABUF_RENDERER=1` is the known workaround); a second UI language (JS) to keep tested.

### Option B — Native GTK4 widgets
- **Pros**: native look and accessibility; no JS.
- **Cons**: the design is a web layout (flex-wrap, grid, dashed borders, glow); GTK CSS gets close, not exact; roughly double the code; every visual change is a Rust change.

### Option C — Local page in an app-mode browser
- **Pros**: smallest new dependency set (axum is already in the tree); easy to test with HTTP calls.
- **Cons**: the owner already rejected "a browser tab, not an app" (DDR 0006 Alternatives; owner pick 2026-11-19); opens a local port, so it needs a token and other local users are a threat (the HTTP-mode lesson in `memory/decisions.md`); the window depends on which browser is installed.

---

## 50% Rule Check

The same goal with half the code: **Option A, read-only first**. Slice 1 ships the window with counts, search, filters, day groups and the detail panel (no writes). That alone fixes the owner's complaint ("not the note board that I expect"). Writes (`Add as tags`, trash + Undo, `Open in editor`, `Copy id`) and the capture dialog are later slices on the same IPC. The status chips and the hotkey chip are cut from slice 1.

---

## Adversarial Review

| Option | Why this might fail |
|---|---|
| A | WebKitGTK draws a blank window on some GPU/Wayland setups → set `WEBKIT_DISABLE_DMABUF_RENDERER=1` when unset, and a test that the page reports "ready" over IPC. A note text rendered with `innerHTML` is a script injection into a window that can call trash → render every note string with `textContent` only, CSP forbids inline script except the bundled one. IPC is an API: a page bug could send any id → Rust validates the id shape and resolves only through `Store`. Thousands of notes → `iter_notes` once per refresh, then filter in JS; measure with 5,000 synthetic notes. |
| B | Two weeks of widget work before the owner sees the board; the result still does not match the design. |
| C | Owner already said no; a port is a new attack surface; the browser may restore the tab or block the custom font. |

---

## Recommended Path

**Option A — wry + tao webview behind a new `window` Cargo feature, read-only first.**

`easy-note dashboard` opens the window when built with `window` and a display is present; otherwise it keeps the Obsidian path (DDR 0006 stays as the fallback, DDR 0009 supersedes its item 2). The launcher (`Exec=easy-note dashboard`) and the tray's "Open dashboard" need no change.

Alternatives not taken: B (native GTK4 widgets), C (local page in an app-mode browser).

---

## Surgical Scope

**Should touch**: `Cargo.toml` (feature `window = ["dep:wry", "dep:tao"]`), new `src/surfaces/window/` (`mod.rs` window + IPC, `api.rs` pure request → response over `Store`, `assets/` html/css/js/fonts), `src/surfaces/dashboard.rs` (choose window vs Obsidian), `src/surfaces/mod.rs`, `tests/window_api.rs` (the pure API, no display), README, PRD (new FRs), `docs/ddr/0009-*.md`, `.github/workflows/ci.yml` (one job with `--features window` and the apt package).

**Must not touch**: `src/core/format.rs` and the note file format (DDR 0002); `src/core/guard.rs` patterns; `src/surfaces/tray.rs` menu order; `install.rs` file sets (no new write target in slice 1).

---

## Edge Case Checklist for TASK_GUIDE

- [ ] No display (`DISPLAY` and `WAYLAND_DISPLAY` unset) or a build without `window` → Obsidian path, exit as today
- [ ] Every string sent to the page passes `guard::mask_for_output`; a masked note shows "secret masked"
- [ ] Note text is rendered with `textContent`, never `innerHTML`; a note containing `<script>` shows as text
- [ ] CSP: no network loads; Plex fonts come from the binary; navigation away from `easy-note://` is refused
- [ ] IPC request with an unknown command or a bad id → an error object, never a panic
- [ ] Day groups use the local time zone (today / yesterday / earlier), same as `query` `day`
- [ ] Notes added while the window is open appear on window focus (refresh on focus), without a restart
- [ ] AI detail `stale` matches `ai::read`'s flag; a note without AI detail shows the design's "No AI detail yet" text
- [ ] 5,000 notes: the first paint in under 1 s on the owner's machine (measured, not assumed)
- [ ] A second `easy-note dashboard` while a window is open: opens a second window (single-instance is cut from slice 1)

---

## Next Actions

1. Write `docs/ddr/0009-own-dashboard-window.md` (supersedes DDR 0006 item 2; Obsidian stays as fallback).
2. Add PRD FRs: window (read-only board), window actions, capture dialog in the window; US-017 restated.
3. `to-issues`: slice 1 read-only window (C2, Medium), slice 2 actions (C2, Medium), slice 3 capture dialog (C2, Medium). Each with a TASK_GUIDE.
4. Stage 3: slice 1 first (Backend-Implementer for the Rust window + API; Frontend-Implementer for the page port), tests first on `window::api`.

---

## User Selection

Under `[admin]`: Option A taken by `docs/claude-md/tracks.md` `### Every user stop and its auto rule` row 4 — ⚠ decided for you, please review. The owner may say "B" or "C" to change it before Stage 3 starts.
