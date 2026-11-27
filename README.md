# easy-note

Capture one-line notes from the terminal, from Claude Code, Codex and Cursor, and from a desktop
key press, into a folder of plain Markdown files. Each note gets its context for free: the repo,
the branch and the task ID of the folder you were in. easy-note is one Linux binary with a CLI and
an MCP server. It masks secrets on every write and read. It runs no model and makes no network
call, with opt-in exceptions: `easy-note ai enrich` and `easy-note ai run` (need `[ai] enrich`), the tray's timed `ai run`
(also needs `schedule = "tray"`) and the window's `Run AI`
button (your click, one note) ask your own `claude` CLI for AI detail
(see "AI detail with your Claude subscription").

## Install

Linux only, Rust stable:

```
git clone https://github.com/thunderkds/easy-note-mcp && cd easy-note-mcp
cargo install --path . --locked
ln -s "$(command -v easy-note)" ~/.cargo/bin/n   # `n` is the same binary: `n <text>` = `easy-note add <text>`
```

Another `n` (for example the Node version manager) may already be on `PATH`. `easy-note doctor`
reports the clash; `easy-note add` always works.

### A Debian package

`scripts/build_deb.sh` makes a .deb for Debian and Ubuntu. The build needs `dpkg-dev` and the
WebKitGTK headers:

```
sudo apt install dpkg-dev libwebkit2gtk-4.1-dev
scripts/build_deb.sh                                  # writes dist/easy-note_<version>_<arch>.deb
sudo apt install ./dist/easy-note_0.1.0_amd64.deb     # apt pulls the libraries it needs
```

The package holds the binary and the docs. It builds the `window` and `tray` features, so the
dashboard window and the tray icon both work. It gives you `easy-note`, not `n`. It writes nothing
below your home folder, so run `easy-note setup` yourself after the install (on GNOME it also
binds Ctrl+Alt+N to the new-note box). `sudo apt remove easy-note` removes the package; it does
not remove what `setup` added (`easy-note uninstall <target>` does).

## Set up

One command puts the desktop parts in place: the launcher and its icons, the Obsidian dashboard
and vault, (in a build with the tray) the tray login item, and on GNOME the Ctrl+Alt+N shortcut
for a new note.

```
easy-note setup
```

Close Obsidian first: Obsidian keeps its vault list in memory and can save over a change. If
Obsidian is open, `setup` says so, leaves Obsidian's vault list as it is and exits with 1; close
Obsidian and run it again. Each step prints under its own heading (`launcher:`, `obsidian:`,
`tray:`). A step that fails does not stop the next one. A second run writes nothing. `setup` does
not touch Claude Code: run `easy-note install claude` for that (see "Clients").

On GNOME, the `hotkey:` step adds one custom shortcut, so Ctrl+Alt+N opens `easy-note quick`:

```
easy-note install hotkey      # only the shortcut
easy-note uninstall hotkey    # remove it
```

Your other shortcuts stay. The step changes nothing when Ctrl+Alt+N is already in use, or when a
shortcut of your own already runs `easy-note quick`. To check it, open GNOME Settings, then
Keyboard, then the custom shortcuts: the entry is named `easy-note quick`. On another desktop,
see "Bind a key".

## First note

No setup is needed. The first write creates `~/easy-notes`:

```
n retry bug only on Firefox #flaky
```

prints `✓ noted 20261114-164713-a1b2 · easy-ui-mcp · T120`. Then:

```
easy-note ls                 # newest first
easy-note search firefox     # every word must match
easy-note show 20261114-164713-a1b2
easy-note tag 20261114-164713-a1b2 ui
easy-note rm 20261114-164713-a1b2       # to the trash; `easy-note restore` brings it back
```

To keep notes somewhere else (a synced folder, for example), run `easy-note init` once, or set
`EASY_NOTE_DIR`. `easy-note path` prints the folder in use.

## Clients

easy-note never writes a client's config file. Add it yourself, once, with one of these.

**Claude Code**:

```
claude mcp add easy-note -- easy-note serve
```

This adds it for the current project only. Add `--scope user` after `add` to have it in every project.

Then `/note <text>` in Claude Code saves a note with no model turn. That needs the prompt hook:

```
easy-note install claude
```

It shows a diff of `settings.json`, keeps a timestamped backup, and adds one hook entry and
`commands/note.md` in `$CLAUDE_CONFIG_DIR` (else `~/.claude`). `easy-note uninstall claude`
removes exactly those.

**Codex**, in `~/.codex/config.toml`:

```toml
[mcp_servers.easy-note]
command = "easy-note"
args = ["serve"]
```

Codex starts the server in the folder you started `codex` from. Started elsewhere, the note has no
repo context unless the call passes `cwd`.

**Cursor**, in `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "easy-note": { "command": "easy-note", "args": ["serve"] }
  }
}
```

Cursor's start folder is not measured yet, so pass `cwd` when the context matters.

In Codex and Cursor, write `note: <text>` and the model calls `note_add` (one model turn). The MCP
tools are `note_add`, `note_search`, `note_list`, `note_read`, `note_stats`, `note_pending_enrich`
and `note_enrich`, plus the prompt `enrich_notes`, which has your own Claude add AI detail to your
notes in separate files (`easy-note ai rm <id>` deletes it). No tool edits, deletes or moves a note.

## AI detail in Obsidian

If your notes folder is inside an Obsidian vault (a `.obsidian` folder in it or above it), each note's AI detail also
shows in Obsidian as a separate page, `ai/<id>.md`, labelled "AI detail (not your words)" and linked to the note. Your
note files are never changed. Run `easy-note ai render` once to write the pages for detail you already have.

## AI detail with your Claude subscription

`easy-note ai enrich` writes the AI detail without a Claude session: it starts your own Claude
Code CLI (`claude -p`) once per note, signed in to the account you pick. No API key is used:
easy-note removes `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN` and `ANTHROPIC_BASE_URL` from the
CLI's environment, so your subscription pays. The CLI path (`ai enrich`, `--pending`, `after_save`) is
off until you turn it on in `~/.config/easy-note/config.toml`; the window's `Run AI` click is its own
opt-in for one note and needs no `[ai]` section:

```
[ai]
enrich = "claude"                        # "off" (the default) or "claude"
claude_config_dir = "~/.claude-personal" # the account; leave it out for Claude Code's default
model = "haiku"                          # the default
timeout_secs = 120                       # 1 to 600
after_save = false                       # true: enrich each note just after you save it
command = "claude"                       # the CLI, on PATH or a full path
schedule = "off"                         # "off" (the default) or "tray"; "tray" needs enrich = "claude"
interval_min = 60                        # minutes between scheduled runs, 5 or more
max_per_run = 20                         # notes per run, 1 to 200
batch_size = 10                          # notes per model call, 1 to 50, not above max_per_run
jobs = ["organize", "drafts", "digest"]  # which scheduled work runs
digest_hour = 3                          # hour of day for the digest, 0 to 23
```

Log in once for each account dir: run `CLAUDE_CONFIG_DIR=~/.claude-personal claude` and type
`/login`. Then:

```
easy-note ai enrich <id> <id>          # those notes
easy-note ai enrich --pending          # notes with no AI detail or a stale one (at most 20)
easy-note ai enrich --pending --limit 3
easy-note doctor                       # the "ai:" line shows the command, account and model
```

Only the note text, with secrets masked, goes to the CLI: no repo, branch, task, date or
transcript. The CLI runs in an empty temp folder with no tools and no MCP servers, and its answer
goes through the same caps and masking as `note_enrich`. Each call counts against your Claude
usage limits and takes a few seconds. With `after_save = true`, `n`, `/note`, the quick box and
`note_add` start `easy-note ai enrich <id>` in the background; the save never waits for it.

### Organize the notes that wait

`easy-note ai run` asks the same CLI to organize the notes: title, kind, summary, project, topic, tags and, for a
draft, why it looks unfinished. One call covers a batch of notes.

```
easy-note ai run                       # at most max_per_run notes, newest first
easy-note ai run --limit 5             # at most 5 (never more than max_per_run)
easy-note ai run --dry-run             # how many notes, calls and which model; starts and writes nothing
```

It needs `enrich = "claude"`, and `jobs` with `organize` or `drafts`. It picks notes with no AI detail, detail for
older text, or detail that was never organized. The last line is `organized N, skipped M, left K pending, calls C`.
Only one run works on a notes folder at a time: a second one prints `already running`. A run stops at the first
failed call and exits 1 when no call worked. `.easy-note/ai-run-state.json` keeps the last run, its counts and a
wait that doubles after each failed run (from `interval_min`, at most 4 hours). The wait is for unattended runs:
`ai run` by hand does not wait.

After `easy-note ai undo` takes back a change of the AI, `ai run` leaves that note alone until you edit its text.
`ai enrich <id>`, `ai enrich --pending`, `Run AI` and `note_enrich` still write it, and that ends the hold.

The tray runs it for you. While `easy-note tray` runs (a build with the `tray` feature), `schedule = "tray"` and
`enrich = "claude"`, the tray starts `ai run` every `interval_min` minutes. The first run comes one interval after
the tray starts, not at once. A change in config.toml or in Settings applies within a minute, with no tray restart.
No run starts while another one works, or before the wait of a failed run has passed. With the tray closed, or with
either switch off, nothing runs by itself. A tray started at login may not have your shell's settings: its `PATH` may be
short and `CLAUDE_CONFIG_DIR` unset. Set `command` in `[ai]` to the full path of `claude`, and set
`claude_config_dir` to the account folder you want.

Without a tray, use cron (`crontab -e`) and leave `schedule = "off"`. cron has a short `PATH` too:

```
0 * * * * $HOME/.cargo/bin/easy-note ai run >> $HOME/.cache/easy-note-ai.log 2>&1
```

## Notes are plain files

Each note is one Markdown file, `<notes dir>/YYYY/MM/<id>.md`: a short front matter block
(`id`, `created`, `repo`, `branch`, `task`, `tags`, `source`) and your text, exactly as typed.
Read them, grep them, sync them or back them up with any tool. The format is a public contract:
`docs/NOTE_FORMAT.md`. `easy-note edit` opens a note in `$EDITOR`; secrets are masked on save.

Files dropped in `<notes dir>/inbox/` (`.md` or `.txt`, for example from a phone) are read as notes
and never rewritten.

## easy-note and easy-memory

They do different jobs. easy-memory saves agent turns automatically (a "memory note"). easy-note
saves only what you ask it to save (a "note"): your words, typed with `n`, `/note`, `note:` or the
quick box. Neither reads the other's folder.

## Dashboard window

A build with the `window` feature has its own dashboard window: counts, search, kind filters, notes by day and a
detail panel with the AI detail. The page is inside the binary and loads nothing from the network.

Window actions on the selected note: `Add as tags` does what `easy-note tag <id> --accept-ai` does; `Edit` changes
the text and tags in the window and saves them as `easy-note edit` does (an inbox note is read-only); `Run AI` (`Re-run`
when the note has AI detail) runs `easy-note ai enrich <id>` on the note without freezing the window, and the click is the
opt-in, so it needs no `[ai]` section (it starts your own `claude`, `command` and `timeout_secs` of `[ai]` apply; an error shows
in a toast); `Copy id` copies the id; `Move to trash` does what `easy-note rm <id>` does, and its
toast offers `Undo` (`easy-note restore <id>`) for 6 seconds. The AI file moves with the note. Nothing else in the
window changes a note file.

Window capture: `+ New note` opens a box. Enter saves the note as `easy-note add` does (`source: quick`), with the
context of your last note as `easy-note quick` takes it; Shift+Enter adds a line, Esc closes without saving, and
Backspace in the empty box drops the context. The new note is selected at the top of today. Building it needs the WebKitGTK headers (Debian/Ubuntu: `sudo apt install libwebkit2gtk-4.1-dev`).

Window settings: the gear button opens `Settings`, which shows and saves these `[ai]` keys of
`~/.config/easy-note/config.toml`: `enrich` (the AI switch), `schedule` (`Organize automatically`),
`interval_min`, `max_per_run`, `batch_size`, `jobs`, `model` and `after_save` (`Run after each save`).
`command` and `claude_config_dir` are shown only: change them in the file. A save changes only the
keys the screen owns, so your other keys, tables and comments stay. If the file changed after the
window read it, nothing is saved and the screen reads the file again. Before
`Organize automatically` turns on, the window shows what is sent (masked note text, your own
`claude` CLI, the account folder and the model) with `Confirm` and `Cancel`. `Confirm` saves the
screen and records `notice_seen`, so the same text is not shown again; another account folder or
model shows it again. The switch needs the AI on. It saves `schedule = "tray"`, and the running tray
then starts `ai run` on that schedule (see "Organize the notes that wait"). `easy-note
doctor` shows the saved values (the model and `after_save` only while the AI is on).

```
cargo install --path . --locked --features "tray window"
easy-note dashboard
```

`dashboard` opens the window when a desktop is running (`WAYLAND_DISPLAY` or `DISPLAY` is set). In a build without
`window`, or with no desktop, it opens the Obsidian dashboard below instead. `easy-note dashboard --note <id>` opens
the window on that note. Only one window opens per notes folder: a second `dashboard` (a tray click, the launcher)
brings the open window to the front, and shows the note when you give `--note`.

## Obsidian dashboard

In a build without the `window` feature, Obsidian shows the notes: by day, project, task, tag and source.
`easy-note setup` (or `easy-note install obsidian` alone) writes the dashboard file, `easy-note.base`, in your notes
folder (the path `easy-note path` prints). It also makes the notes folder an Obsidian vault: it creates
`<notes folder>/.obsidian/` and adds the folder to Obsidian's vault list,
`$XDG_CONFIG_HOME/obsidian/obsidian.json` (else `~/.config/obsidian/obsidian.json`). It only adds: every other vault
and setting stays, and a backup of the old file is kept beside it. Nothing is added when a vault already is the notes
folder or a parent of it, or when the file is not valid JSON (it names the file and exits with 1). With no Obsidian at
all, the vault step is skipped. Flatpak's Obsidian is not covered.

```
easy-note setup
easy-note dashboard
```

`dashboard` opens the file in Obsidian. If the notes folder is not a vault yet, it says to run `easy-note setup`.
If Obsidian is not installed, `dashboard` says so and opens the notes folder instead.
`easy-note uninstall obsidian` removes only `easy-note.base`: it never edits Obsidian's vault list and never removes
`.obsidian/`, because the vault is yours now. Remove the vault in Obsidian's vault switcher if you want it gone.

## One key press

From anywhere on the Linux desktop or in an editor, one key press or one click opens a small
input box. Type, press Enter, and the note is saved with `source: quick`. A `✓ noted ...` desktop
notification confirms it. Esc, or an empty box, saves nothing.

In a build with the window (the `.deb`, or `--features window`), the box is a small easy-note
window that holds only the dashboard's `+ New note` dialog, with the same look and keys: Enter
saves, Shift+Enter adds a line, Esc cancels. It closes when you save or cancel. In other builds, or
when that window cannot open, the box is the native `zenity` box, or `kdialog` when `zenity` is
missing. Either box shows only when `$DISPLAY` or `$WAYLAND_DISPLAY` is set. Without a display,
`quick` asks on the terminal. With no terminal either, it exits with `no dialog tool and no terminal`. The notification uses
`notify-send` when it is installed.

### Context in the box

The box shows the context it will save: in the easy-note window as a chip above the text, for
example `easy-ui-mcp · T120`; in the `zenity` box in its label, for example
`Note — easy-ui-mcp · T120 (start with - to drop)`.

- From an editor, pass `--cwd` with the project folder. The context is that folder's repo, branch and task.
- Without `--cwd`, the context is that of your newest terminal (`n`) or `/note` note made less
  than 2 hours ago. An older note gives no context.
- In the easy-note window, the context is a chip above the text: press Backspace in the empty box,
  or click its ×, to save with no context.
- In the `zenity` or `kdialog` box, start the answer with `-` and a space (`- remember the milk`) to
  save with no context. The `-` and one space after it are removed. `-x` and `--x` are saved as typed.

### Bind a key

```
easy-note hotkey
```

prints the commands for your desktop (GNOME, KDE, or any other). It changes nothing: you run them.
On GNOME, `easy-note install hotkey` (part of `easy-note setup`) does it for you.

For a click instead of a key, add the launcher. It writes five files: the app entry
`~/.local/share/applications/easy-note.desktop` and four icons under
`~/.local/share/icons/hicolor/`. `easy-note uninstall launcher` removes them. Clicking the app opens
the dashboard; a right-click offers New note. A file there that is not ours
is kept: the install names it, writes nothing and exits with 1.

```
easy-note install launcher
```

### System tray

An optional icon in the system tray puts "New note" two clicks away. It is not in the default
build, so the default `easy-note` links no tray code. Install a build that has it:

```
cargo install --path . --locked --features tray
```

Then run `easy-note tray`. The menu holds New note · Ctrl+Alt+N (opens the quick box with the last
context), Open dashboard, your 5 newest notes (click one to open the window on it; without the window, the file opens), Open notes folder and Quit. It needs a desktop with a
StatusNotifier host: KDE has one; GNOME needs the AppIndicator extension. Without a host,
`easy-note tray` exits with 1 and prints the hotkey hint, and `easy-note doctor` says
`tray: no tray host`. At login the panel may start after the tray, so the tray checks again every
second for up to 60 seconds before it gives up. In a build with `tray` and `window`, opening the
dashboard window starts the tray when it is not running. Only one tray runs at a time. After `install launcher` the tray shows the
easy-note outline icon, and a check badge for 3 seconds after a note is saved.

To start it at login, `easy-note install tray` writes one file,
`~/.config/autostart/easy-note-tray.desktop`, and `easy-note uninstall tray` removes it. Nothing
starts at login unless you run `install tray`. If your session does not find `easy-note` (for
example it sits in `~/.cargo/bin`), edit the `Exec=` line to the full path from `command -v easy-note`.

The tray is also the timer of `Organize automatically` (see "Organize the notes that wait"). After you install a
new version, quit the tray and start it again.

If you also use easy-personal's tray, keep one tray: use that one and do not run `easy-note tray`. Timed AI runs
then need cron.

### VS Code

Add a task, in the workspace `.vscode/tasks.json` or in your user tasks:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "easy-note quick",
      "type": "process",
      "command": "easy-note",
      "args": ["quick", "--cwd", "${workspaceFolder}"],
      "presentation": { "reveal": "never", "close": true },
      "problemMatcher": []
    }
  ]
}
```

Bind a key in your user `keybindings.json`:

```json
{
  "key": "ctrl+alt+n",
  "command": "workbench.action.tasks.runTask",
  "args": "easy-note quick"
}
```

### JetBrains IDEs

Open Settings, then Tools, then External Tools, and add a tool:

| Field | Value |
|---|---|
| Name | `easy-note quick` |
| Program | `easy-note` |
| Arguments | `quick --cwd $ProjectFileDir$` |
| Working directory | `$ProjectFileDir$` |

Clear "Open console for tool output". Then open Settings, Keymap, External Tools, and give the
tool a key.

### GNOME and KDE

`easy-note hotkey` prints the exact steps. On GNOME it prints `gsettings` commands for a custom
shortcut (Ctrl+Alt+N). On KDE it names the System Settings pages (Keyboard, Shortcuts, Add New,
Command or Script).

If your desktop does not find `easy-note` (for example it sits in `~/.cargo/bin`), use the full
path from `command -v easy-note` in the key binding, the task and the tool.
