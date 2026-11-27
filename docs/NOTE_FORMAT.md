# Note file format — `easy-note: 1`

**Status**: Final for `easy-note: 1` (HUN-253, 2026-11-15) · **Contract owner**: easy-note-mcp · DDR 0002

A breaking change needs `easy-note: 2`, a new DDR, and a reader that still reads v1 files.

## Location

`<notes dir>/YYYY/MM/<id>.md`, where `YYYY/MM` is the local date of `created`.
Only `.md` files in a `YYYY/MM` folder (4 digits / 2 digits) are notes. Files under `<notes dir>/.easy-note/`
(trash, AI detail), the Obsidian views in `<notes dir>/ai/` (below) and the writer's temp files (`.*.tmp` in a month folder) are not notes. Other files in the dir are ignored.

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
| `updated` | no | string, ISO 8601 with offset | Set by `easy-note edit` / `tag` and by the window's `Edit` and `Add as tags`. Read-only for AI: no AI run sets or changes it. Shown as `edited <when>` in `ls`, `search` and the window. |
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
An inbox note goes to `<notes dir>/.easy-note/trash/inbox__<mtime>__<file name>` (`<mtime>` = the file's original mtime in Unix seconds; extension kept); `restore inbox/<stem>` moves it back to `inbox/<file name>` and sets that mtime back.
`easy-note trash --empty --older-than 30d` deletes trash files whose mtime is older than the cutoff.

A trashed note's AI detail file (below) moves with it to `<notes dir>/.easy-note/trash/ai/<note trash file name>.json`
(`<id>.md.json`; `inbox__<mtime>__<file name>.json` for an inbox note) and back on `restore`; neither move replaces a file.
When a file of that name is in the way on `rm`, the AI file stays in `.easy-note/ai/`. When one is in the way on
`restore`, the live AI file is kept and the trashed copy stays in `trash/ai/` only until the next `trash --empty`.
`trash --empty` deletes a trashed AI file with its note, decided by the note's mtime only, and deletes any `trash/ai/`
file whose note is no longer in the trash.

## Inbox files (round 2)

Any `.md` or `.txt` file directly in `<notes dir>/inbox/` is a note even without front matter. Readers derive:
`id` = `inbox/<file stem>`; `created` = a `YYYYMMDD-HHMMSS` prefix of the file name (local time), else the file mtime;
`source` = `inbox`; `tags` = `#words` in the text; no context fields. If the file has valid `easy-note: 1` front matter,
its keys win, except `id`, which is always the inbox id above (`show`, `rm` and `restore` route by it). easy-note never rewrites inbox files. Phone example: an iOS Shortcut "Save File" to
`easy-notes/inbox/20261114-183002.txt` through iCloud Drive or Dropbox.

## AI detail files (round 3, DDR 0004)

`<notes dir>/.easy-note/ai/<id>.json` (inbox ids use `inbox__<stem>.json`). Not a note; never changes the note.

```json
{"easy-note-ai": 1, "id": "20261114-164713-a1b2", "title": "Firefox-only retry failure in ui_wait_for",
 "kind": "bug", "summary": "…≤ 600 chars…", "tags_suggested": ["firefox", "ui_wait_for"],
 "based_on": "sha256:…", "model": "claude-…", "at": "2026-11-15T09:00:00+07:00"}
```

`kind` ∈ `bug`, `idea`, `question`, `todo`, `draft`, `other`; `title` ≤ 120 chars, `summary` ≤ 600 chars, at most 5
`tags_suggested`, each a valid tag. `based_on` is `sha256:<hex>` of the note body exactly as stored (the text after
the closing `---`; the whole file for an inbox note without front matter); `based_on` and `at` are set by easy-note.
Stale when `based_on` ≠ that hash. Unknown keys are ignored. A file that is not valid JSON, has a wrong type or is over
a cap is treated as no AI file. easy-note writes it to a temp file in `.easy-note/ai/` and renames it over the old one.

An inbox note's AI file is keyed by its id, so when a second inbox file with the same stem appears (`a.md` next to
`a.txt`) the ids become `inbox/a.md` and `inbox/a.txt`, the old `inbox__a.json` no longer matches, and the note shows as
pending again.

A file written by `note_add(detail)` has `model: "capture"`, `kind: "other"`, `tags_suggested: []`, `title` = the
note's title and `summary` = the detail; it counts as enriched until the note changes.

Round 6 adds five optional keys to the AI file: `project` ≤ 60 chars, `topic` ≤ 60, `draft_reason` ≤ 200,
`draft_suggestion` ≤ 600, `organized_at` (ISO 8601). A file without them is valid. Every string is masked. Over a cap,
the file is no AI file.

The batch organizer (`ai run`, FR-047) writes all of these keys for each note of a batch: `title`, `summary`,
`tags_suggested`, `model` and `based_on` as above, then `kind`, `project` (the note's `repo` when it has one), `topic`,
`draft_reason` and `draft_suggestion` (kind `draft` only), and `organized_at`. A key with a person's override is left as
it is. When the kind shown (the person's override, else the model's) is not `draft`, the batch removes `draft_reason`
and `draft_suggestion` and logs the removal. A note whose AI file has `organized_at` missing or older than `at` is not yet organized.

### Overrides and the change log (round 6)

Two files in `<notes dir>/.easy-note/`, neither a note, neither read by easy-personal.

`ai-overrides.json`, written whole (temp file, then rename). A person's fix wins over the AI file; the AI file is not changed.
`project` and `topic` ≤ 60 chars, `kind` one of the AI kinds. `topics` renames a topic wherever it shows.

```json
{"easy-note-overrides": 1,
 "notes": {"20261114-164713-a1b2": {"topic": "firefox flaky", "kind": "bug"}},
 "topics": {"ff": "firefox"}}
```

`ai-log.jsonl`, append only (created with mode 0600), one line per change: `id`, `field`, `old`, `new` (`null` = no value), `who` (`ai` or
`person`), `at`; an undo line adds `"undo": true`; an AI line adds `op`. Values are masked.

A person's change is one line, its values cut to 200 chars. Each write of an AI file (`ai enrich`, `note_enrich`, the
batch) first appends, in one write: a line with `field: "ai_file"` whose `old` is the whole earlier AI file as JSON text
(`null`: there was none), then one line for each of `title`, `summary`, `tags_suggested` (JSON text such as `["usb"]`),
`kind`, `project`, `topic`, `draft_reason` and `draft_suggestion` that changes. The `old` value of an AI line is whole;
`new` is cut to 200 chars. If the lines cannot be written, the AI file is not written. The two writes of one batch item
share one `op`.

`easy-note ai set <id> project|topic|kind VALUE` writes `ai-overrides.json` and one log line. `easy-note ai undo <id>`
takes back the latest step of a note that no later undo took back. A step is every line of one `op`, or one line without
`op`. For a person's change it restores the override. For an AI item it puts back the AI file as it was before the item,
with its earlier `based_on`, `at`, `model` and `organized_at`, or removes the file and its Obsidian view when the item
created it; overrides stay. A file removed since with `ai rm` stays removed, and an AI file that is not valid is not
replaced. The undo first logs the file it replaces, whole, on a line with `field: "ai_replaced"` (never undone
itself); it writes the undo lines only after the AI file is back, so a failed undo can be run again. An item whose
`ai_file` line cannot be read is put back key by key. After an undo of an AI item the note is held from `ai run`
until its text changes (see "Run files").

Lines written before `op` existed are one step each: an earlier `ai enrich` replacement takes one undo per key it
logged. An AI file created then has no line, so it cannot be undone, and `ai undo` takes back the change logged before
it, which can be a person's override. If an AI write fails after its lines are logged, that step changes nothing.
The log keeps old AI text after `ai rm` and after the note is trashed.

An override of a trashed or missing note is kept and ignored; `doctor` lists it as an orphan. An invalid file is
ignored, named by `doctor`, and never overwritten.

Readers that must not hold note text (easy-personal) must not read AI files either.

### Run files (`ai run`, round 6)

Four more files in `<notes dir>/.easy-note/`. None holds note text.

- `ai-hold.json`: `{"<id>": "sha256:…"}`, written whole. `ai undo` of an AI item adds the note with the hash of its
  text. `ai run` skips the note while its text has that hash. Any new `title` / `summary` write (`ai enrich`,
  `note_enrich`, `Run AI`) removes the entry. An invalid file makes `ai run` and `ai undo` refuse; it is never
  overwritten.
- `ai-write.lock`: an empty file, locked while an AI file, `ai-overrides.json` or `ai-hold.json` is changed. Never
  locked while `claude` runs.
- `ai-run.lock`: an empty file, locked for the length of one `ai run`.
- `ai-run-state.json`: `last_run`, `organized`, `skipped`, `left`, `calls`, `failures`, `next_after` (when an
  unattended run may try again) and `last_error` (one masked line).

## Reader rules

- Read-only. Never rename, move or rewrite a note.
- A reader that must not hold note text reads only the front matter (stop at the second `---`).

## Obsidian views of AI detail (FR-036, DDR 0006)

When the notes dir, or a parent of it, holds a `.obsidian` folder (a vault), each AI file also shows as
`<notes dir>/ai/<id>.md` (inbox ids: `inbox__<stem>.md`, so a name never holds `/`). A view is **not a note** and is
**never read back**: it is output only, written by easy-note with a temp file and a rename, and only a file that starts
with `easy-note-ai-view: 1` is replaced or removed, so the person's own files in `ai/` stay. Outside a vault nothing is
written under `ai/`. A view write error never fails the AI write.

```markdown
---
easy-note-ai-view: 1
note: "[[20261114-164713-a1b2]]"
model: "claude-…"
at: "2026-11-15T09:00:00+07:00"
kind: "bug"
stale: false
tags_suggested: ["firefox", "ui_wait_for"]
---

# AI detail (not your words)

## Firefox-only retry failure in ui_wait_for

…summary…

Suggested tags: #firefox #ui_wait_for

Accept with: easy-note tag 20261114-164713-a1b2 --accept-ai
```

Every string is masked. For an inbox note `note` links the file part (`[[phone]]`). `stale` is true when the AI file's
`based_on` is not the hash of the note body. `ai rm`, `rm` remove the view, `restore` writes it again, `edit` and `tag`
refresh it, and `easy-note ai render` rewrites every view (it prints `N views written`; outside a vault it exits 1).
