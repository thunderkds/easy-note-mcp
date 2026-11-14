# Design brief: easy-note

You are designing the user experience and visual design of **easy-note**, a small note-taking app for one person who works all day with AI coding agents (Claude Code, Codex, Cursor) across many code projects. You need no prior context; everything you need is below.

## What it is

easy-note saves short notes the moment a thought appears ("retry bug only on firefox", "ask about the 8 KB limit", "idea: i18n check"). Each note is a plain Markdown file in a folder the person picks (default `~/easy-notes`). easy-note adds context automatically: the project (repo), the git branch, the task ID (for example `T120`) and the time. Later, if the person asks, their own AI assistant adds a short "AI detail" to each note: a clear title, a type (bug, idea, question, to-do, other), a 1–3 sentence summary and suggested tags.

## The core promise

1. **One press.** A note is one key press, one click, or one short typed command away, from wherever the person is.
2. **Never lose your words.** Capture never fails and never asks a question. No setup step blocks the first note. Secret-looking text (API keys, passwords) is masked, not rejected.
3. **Out of the way.** Capture takes a few seconds, then disappears. Organizing happens later, never at capture.

## Who uses it

One person, a solo software builder. Primary desktop: **Linux with GNOME** (Ubuntu), keyboard-first, dark theme likely. Also macOS later; Windows best effort. Expert user: dislikes clutter, modal dialogs and decoration.

## Surfaces to design

Design each surface with all listed states.

### 1. Quick capture box (the main screen)
Opens from a global hotkey, the dock icon, or the tray menu. One line of text, Enter saves, Esc cancels.
- States: empty; typing; with **pre-filled context** (repo · branch · task taken from the last note under 2 hours old, shown as a removable chip, removed with one key); long text (wraps or grows to a few lines; `Shift+Enter` for a new line is your call); saving; saved; cancelled (nothing saved).
- Must be visible in under 300 ms; first keystroke must land in the field (no click needed).
- Implemented with a native system dialog or a minimal custom window: design for a single small window, no title bar chrome if possible, centred or near the top of the screen (your call).

### 2. Confirmation feedback
After every capture, from any path, one line: `✓ noted · easy-ui-mcp · T120` (repo and task only if known; plus `· 1 secret masked` when it applies). As a desktop notification for the box/tray paths; as one terminal line for the command line; as a one-line message inside the AI chat app. Design the wording and the notification look. It must never need a click to dismiss.

### 3. System tray icon and menu (opt-in)
A small icon in the OS tray (GNOME needs the AppIndicator extension; clicking opens the menu). Menu: **New note** (opens the capture box), the **5 newest notes** (title + relative time; click opens the file), **Open notes folder**, **Quit**.
- Design: the icon (monochrome, legible at 16–22 px, light and dark panels), optional "saved" pulse or badge, menu labels and order, empty state (no notes yet).

### 4. Dock / app launcher icon
The app icon for the dock and app menu (`easy-note`). Same mark as the tray icon or a fuller version. Sizes 16–512 px.

### 5. Note display (terminal)
The command line is a first-class surface. Design the text layout (monospace, ANSI colours that work on dark and light terminals, readable with colour off) for:
- `easy-note show <id>`: the person's text first, then a context line (`easy-ui-mcp · fix/t120-firefox · T120 · Tue 16:47`), then the **AI detail block**, clearly separated and labelled `AI detail (not your words)` with model and date, title, type, summary, suggested tags and how to accept them.
- `easy-note ls` / `search`: a dense list (one line per note: time, repo, task, title or first line, type marker, an "AI" or "stale" marker).
- `easy-note stats`: counts by project, task, tag and month.
- Error and edge messages: no tray host found ("use the hotkey instead"), a stale AI detail (the note was edited after the AI wrote it).

### 6. Optional: a note rendered in a Markdown editor
Notes are opened in the person's editor or Obsidian. Show how the file reads there (a small metadata header, then the person's text). You may propose the header field order; you may not add visual elements that require a special app.

## Fixed constraints (do not change)

- Notes are plain Markdown files with a small metadata header. No database, no account, no cloud, no sync service of our own.
- **The person's words are never changed.** AI detail is always shown separately, below the note, and always labelled as not the person's words. It must never look like the person wrote it.
- Capture never asks a question (no title, folder, tag or type picker at capture).
- Nothing leaves the machine; no sign-in screens; no network features.
- Tray icon is opt-in; the hotkey is the main one-press path.
- No main application window or dashboard in this version. Do not design one. (A separate app may show note counts later.)
- Accessible: keyboard only must work everywhere; contrast WCAG AA; do not rely on colour alone (the AI block needs a text label, not only a colour).

## Open for you to decide

- Name treatment and the icon mark (the product name is "easy-note").
- Capture box size, position, typography, how the context chip looks and is removed.
- Notification wording and tone (short, calm, no exclamation marks suggested).
- Terminal colour palette and symbols (prefer plain Unicode that works in common terminal fonts).
- How the AI block is visually set apart (indent, rule, dim colour, prefix), as long as the text label stays.
- How the "type" (bug, idea, question, to-do, other) is shown: words, letters or symbols.

## Deliverables

1. Mockups of every surface and state above (light and dark where it applies), at real sizes.
2. The icon set: tray (16, 22, 24 px; light/dark panel variants) and app icon (16–512 px), as SVG.
3. Text and terminal specs: exact strings for confirmations, menu labels, empty states and errors; ANSI colour and symbol map, with a no-colour fallback.
4. A one-page spec of the capture box: size, spacing, font, keyboard behaviour (Enter, Esc, Shift+Enter, removing the context chip), timing.
5. A short rationale (under one page) for the main choices, and anything in this brief you would push back on.

Keep it minimal. If a choice adds a step to capture, leave it out.
