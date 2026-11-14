# Note file format — `easy-note: 1` (draft)

**Status**: Draft, finalized in T003 · **Contract owner**: easy-note-mcp · DDR 0002

## Location

`<notes dir>/YYYY/MM/<id>.md`, where `YYYY/MM` is the local date of `created`.
Files under `<notes dir>/.easy-note/` (trash, lock files) are not notes. Other files in the dir are ignored.

## ID

`YYYYMMDD-HHMMSS-xxxx`: local time of `created` to the second, then 4 lower-case hex chars of
sha256(created in ns, hostname, text). On a clash the writer adds `-2`, `-3`, …

## Example

```markdown
---
easy-note: 1
id: 20261114-164713-a1b2
created: 2026-11-14T16:47:13+07:00
repo: easy-ui-mcp
remote: thunderkds/easy-ui-mcp
branch: fix/t120-firefox
task: T120
tags: [flaky, firefox]
source: mcp
client: claude-code
---
Retry bug only on Firefox #flaky

ui_wait_for times out after the second navigation. Check after T120 lands.
```

## Front matter keys

| Key | Required | Type | Meaning |
|---|---|---|---|
| `easy-note` | yes | int | Format version. Readers skip files with an unknown version. |
| `id` | yes | string | As above; equals the file name stem. |
| `created` | yes | ISO 8601 with offset | Local time of capture. |
| `updated` | no | ISO 8601 with offset | Set by `easy-note edit` / `tag`. |
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

## Body

Markdown. The first line is the title (≤ 120 chars). Whole text (title + body) ≤ 20,000 chars.

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
