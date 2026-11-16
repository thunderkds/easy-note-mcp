# easy-note

Capture one-line notes from the terminal, from an editor and from MCP clients into a folder of
Markdown files. This README holds the "One key press" section only; the full README comes with
HUN-264.

## One key press

From anywhere on the Linux desktop or in an editor, one key press or one click opens a one-line
box. Type, press Enter, and the note is saved with `source: quick`. A `✓ noted ...` desktop
notification confirms it. Esc, or an empty box, saves nothing.

The box is the native `zenity` box, or `kdialog` when `zenity` is missing. It shows only when
`$DISPLAY` or `$WAYLAND_DISPLAY` is set. Without a display, `quick` asks on the terminal. With no
terminal either, it exits with `no dialog tool and no terminal`. The notification uses
`notify-send` when it is installed.

### Context in the box

The box shows the context it will save, for example `Note — easy-ui-mcp · T120 (start with - to drop)`.

- From an editor, pass `--cwd` with the project folder. The context is that folder's repo, branch and task.
- Without `--cwd`, the context is that of your newest terminal (`n`) or `/note` note made less
  than 2 hours ago. An older note gives no context.
- Start the answer with `-` and a space (`- remember the milk`) to save with no context. The `-`
  and one space after it are removed. `-x` and `--x` are saved as typed.

### Bind a key

```
easy-note hotkey
```

prints the commands for your desktop (GNOME, KDE, or any other). It changes nothing: you run them.

For a click instead of a key, add the launcher. It writes one file,
`~/.local/share/applications/easy-note.desktop`, and `easy-note uninstall launcher` removes it.

```
easy-note install launcher
```

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
