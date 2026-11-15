# Note file format — `easy-note: 1`

**Status**: Final for `easy-note: 1` (HUN-253, 2026-11-15) · **Contract owner**: easy-note-mcp · DDR 0002

A breaking change needs `easy-note: 2`, a new DDR, and a reader that still reads v1 files.

## Location

`<notes dir>/YYYY/MM/<id>.md`, where `YYYY/MM` is the local date of `created`.
Only `.md` files in a `YYYY/MM` folder (4 digits / 2 digits) are notes. Files under `<notes dir>/.easy-note/`
(trash, AI detail) and the writer's temp files (`.*.tmp` in a month folder) are not notes. Other files in the dir are ignored.

The notes dir is `EASY_NOTE_DIR` if set, else `dir` in `~/.config/easy-note/config.toml`, else `~/easy-notes`
(created by the first write).

## ID

`YYYYMMDD-HHMMSS-xxxx`: local time of `created` to the second, then the first 4 lower-case hex chars of
sha256 over the UTF-8 bytes of `<created, ns since the epoch>\n<hostname>\n<text>`.
On a clash the writer adds `-2`, `-3`, … (`20261114-164713-a1b2-2`). The `id` key always equals the file name stem.

## Example

```markdown
---
easy-note: 1
id: "20261114-164713-a1b2"
created: "2026-11-14T16:47:13+07:00"
repo: "easy-ui-mcp"
remote: "thunderkds/easy-ui-mcp"
branch: "fix/t120-firefox"
task: "T120"
tags: ["flaky", "firefox"]
source: "mcp"
client: "claude-code"
---
Retry bug only on Firefox #flaky

ui_wait_for times out after the second navigation. Check after T120 lands.
```

## Front matter keys

| Key | Required | Type | Meaning |
|---|---|---|---|
| `easy-note` | yes | int | Format version. Readers skip files with an unknown version. |
| `id` | yes | string | As above; equals the file name stem. |
| `created` | yes | string, ISO 8601 with offset | Local time of capture, to the second. |
| `updated` | no | string, ISO 8601 with offset | Set by `easy-note edit` / `tag`. |
| `repo` | no | string | Folder name of the main worktree. Absent outside git. |
| `remote` | no | string | `owner/name` of origin, userinfo stripped. |
| `branch` | no | string | Branch at capture. |
| `task` | no | string | Task ID from branch pattern or explicit. |
| `tags` | yes | list of strings | May be empty `[]`. Lower-case `[a-z0-9_/-]+`. |
| `source` | yes | `cli` \| `mcp` \| `hook` \| `quick` | Which surface wrote it (`inbox` is implied for inbox files, never written). |
| `client` | no | string | MCP client name, if the client sent one. |
| `session` | no | string | *(round 3)* Claude Code session id, hook notes only. |
| `commit` | no | string | *(round 3)* Short HEAD sha at capture. |

Unknown keys are kept on edit and ignored on read. Paths are never stored.

## How easy-note writes a file

- The file starts with a `---` line, then the known keys in the order of the table above, one per line, then `---`.
- Every string is double-quoted (JSON escapes), so YAML never reads `no`, `on` or a date as another type.
  `easy-note` is a bare integer; `tags` is a one-line list of quoted strings.
- Unknown keys come after the known keys, as their original YAML text (comments on their lines included).
- The body follows the closing `---` line exactly as typed, with one `\n` added if it does not end with one.
- Encoding UTF-8, line ends `\n`. For a file easy-note wrote, parse then render gives the same bytes.
- A new note goes to a temp file in its month folder, is fsynced, then hard-linked to its final name.
  The link fails if the name exists, so a write never replaces a file. Where the filesystem has no hard links,
  the writer reserves the name with an exclusive create, then renames the temp file over it.
- `edit` / `tag` write a temp file in the same folder and rename it over the note.

## How a reader must read a file

- CRLF line ends are accepted. YAML timestamps are read as strings (`created`, `updated` keep their text).
- A file is **skipped** (counted with its path, never changed) when: it has no `---` front matter, the front matter
  is not valid YAML or not a mapping, `easy-note` is missing, not an integer or not `1`, a required key is missing,
  a key has the wrong type, `source` is not a listed value, `created` / `updated` is not ISO 8601 with an offset,
  the file is not UTF-8, or `id` differs from the file name stem.

## Body

Markdown. The title is the first line, cut to 120 chars for display; the file keeps the whole line.
Whole text (title + body) ≤ 20,000 chars: a longer text is refused on write, the only refusal.

## Trash

`easy-note rm <id>` moves the file to `<notes dir>/.easy-note/trash/<id>.md` and sets its mtime to the time of the move.
`easy-note restore <id>` moves it back with the same bytes; neither move replaces an existing file.
`easy-note trash --empty --older-than 30d` deletes trash files whose mtime is older than the cutoff.

## Inbox files (round 2)

Any `.md` or `.txt` file directly in `<notes dir>/inbox/` is a note even without front matter. Readers derive:
`id` = `inbox/<file stem>`; `created` = a `YYYYMMDD-HHMMSS` prefix of the file name (local time), else the file mtime;
`source` = `inbox`; `tags` = `#words` in the text; no context fields. If the file has valid `easy-note: 1` front matter,
its keys win. easy-note never rewrites inbox files. Phone example: an iOS Shortcut "Save File" to
`easy-notes/inbox/20261114-183002.txt` through iCloud Drive or Dropbox.

## AI detail files (round 3, DDR 0004)

`<notes dir>/.easy-note/ai/<id>.json` (inbox ids use `inbox__<stem>.json`). Not a note; never changes the note.

```json
{"easy-note-ai": 1, "id": "20261114-164713-a1b2", "title": "Firefox-only retry failure in ui_wait_for",
 "kind": "bug", "summary": "…≤ 600 chars…", "tags_suggested": ["firefox", "ui_wait_for"],
 "based_on": "sha256:…", "model": "claude-…", "at": "2026-11-15T09:00:00+07:00"}
```

`kind` ∈ `bug`, `idea`, `question`, `todo`, `other`. Stale when `based_on` ≠ sha256 of the note text. Readers that
must not hold note text (easy-personal) must not read AI files either.

## Reader rules

- Read-only. Never rename, move or rewrite a note.
- A reader that must not hold note text reads only the front matter (stop at the second `---`).
