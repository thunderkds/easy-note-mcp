# T001 — How Claude Code, Codex and Cursor start a stdio MCP server, and what the Claude Code prompt hook sees

Date: 2026-11-14. Task: HUN-251 (plan ID T001). Readers: T004 (context capture), T011 (Claude Code capture), T009 (README).

## Method

* Probe server: `scripts/probe_server.py` (stdio, MCP Python SDK `mcp` 1.30.0). It logs its process cwd, the `initialize` handshake (`clientInfo`, `protocolVersion`, the `roots` capability) and the answer to a `roots/list` request.
* Probe hook: `scripts/probe_hook.py` (stdlib). It dumps the raw `UserPromptSubmit` event and prints the block JSON when the prompt starts with `/note `.
* Run: `uv run --with 'mcp<2' python scripts/probe_server.py`. Plain `--with mcp` now resolves to `mcp` 2.x, whose server API differs (`MCPServer`, new `lowlevel`). Pin the SDK major version in T002 and T008.
* Every run started headless from a throwaway git repo `<proj>` and from a linked worktree `<proj-wt>` of it (branch `fix/t120-probe`), both under a temp dir.
* Claude Code 2.1.325: `claude -p` with `--mcp-config <tmp> --strict-mcp-config --settings <tmp> --setting-sources project`. The probe hook came only from the `--settings` file.
* Codex `codex-cli` 0.163.0: `codex exec -s read-only -c mcp_servers.<name>.…` overrides. No config file was edited.
* Cursor `cursor-agent` 2025.09.18-7ae6800: project-scope `.cursor/mcp.json` inside `<proj>`. See "Not measured".
* No user-level config file was edited. Checksums before and after are in the review Evidence of `tasks/TASK_REVIEW_HUN-251.md`.

## Per-client facts (AC 1)

| Client | Process cwd of the stdio server | `roots/list` | `clientInfo.name` | Notes |
|---|---|---|---|---|
| Claude Code 2.1.325 | The folder `claude` was started in: `<proj>`, `<proj-wt>`, and `<proj>/sub/deep` when started there. It is not the repo root. | Declared `roots` capability (`listChanged: true`). Answer: one root, `file://` + the same folder as the cwd. | `claude-code` (title `Claude Code`, version `2.1.325`) | Protocol `2025-11-25`. The server env also holds `CLAUDE_PROJECT_DIR` (the launch folder). |
| Codex 0.163.0 | The folder `codex` was started in: `<proj>`, `<proj-wt>`, and `~` when started from `$HOME`. | No `roots` capability declared. A `roots/list` request got an answer with an empty list: `roots = []`. | `codex-mcp-client` (title `Codex`, version `0.163.0`) | Protocol `2025-06-18`. The server env held none of the probe's hint variables. Started from `$HOME`, the server cwd is `$HOME`: not a repo. |
| Cursor | Not measured | Not measured | Not measured | `cursor-agent` is not signed in, and sign-in needs a browser. See "Not measured" for the manual steps. |

In a linked worktree both measured clients report the worktree folder, never the main repo folder. T004 maps it to the main repo name.

## Decision 1: cwd order for FR-004

Keep the PRD order: **explicit `cwd` argument, then MCP roots, then server process cwd.**

* Treat the roots answer as absent when the client declares no `roots` capability or when the list is empty (Codex answers `[]`). Use the first root otherwise.
* Both measured clients give a usable folder: Claude Code through roots and process cwd (they agree), Codex through process cwd only. So the third step is not optional, and no measured client needs a fourth source.
* A process cwd that is not inside a git repo (for example `$HOME`) gives no context fields. The note is still saved (FR-004: absent, never guessed).
* README line for Codex: "Codex starts the server in the folder you started `codex` from. Started elsewhere, pass `cwd`."
* README line for Cursor, until the owner's manual run replaces it: "pass `cwd`". Cursor is the only client with no measured source, so the safe line is to ask for the argument.

## Claude Code hook facts (AC 3)

Claude Code 2.1.325, headless (`claude -p`) and `--input-format stream-json`.

| Question | Measured answer | Evidence |
|---|---|---|
| Does `/note …` reach `UserPromptSubmit` raw, with no `~/.claude/commands/note.md`? | Yes. The event `prompt` is exactly `/note retry bug on firefox`. No model turn, no "unknown command" first. | Fixture `note_prefix_linked_worktree.json`; run result `num_turns: 0`, `total_cost_usd: 0`. |
| Same, with a project command file `.claude/commands/note.md` present? | Yes. The hook runs first and its block wins (`num_turns: 0`). When the hook prints nothing for the prompt, the command file runs instead (one model turn). So the command file is a working fallback. | Run with and without a matching prefix. |
| Does `{"decision":"block","reason":…}` stop the prompt? | Yes. No model call (`num_turns: 0`, cost 0). | Same run. |
| Is the reason shown? | In headless output the `result` text is `UserPromptSubmit operation blocked by hook:\n<reason>\n\nOriginal prompt: <prompt>`. The reason shows and the original prompt is echoed. How the interactive screen shows it: not measured. | See "Not measured". |
| Any other prompt, no hook output? | The prompt goes to the model unchanged. `;n retry bug` with the probe's `/note ` prefix passed through; so did `/zzunknown foo`. | Fixture `unknown_slash_command.json`. |
| Does the hook run for a prompt sent while the agent works? | Measured only for the nearest headless case: a second `stream-json` message sent 4 s into a running 15 s Bash turn. The hook did not run at send time: the probe wrote the first prompt's event at t0 and the `/note` event 18.1 s after t0, although the message was sent at 4.0 s. It ran when Claude Code took the message, after the first turn ended, and it blocked with `num_turns: 0`. The interactive queue (typing while the agent works): not measured. | Fixture `note_prefix_queued_while_working.json`. |
| Which `cwd` does the event carry in a linked worktree? | The worktree folder (`<proj-wt>`). In a subfolder it is the subfolder. It is not the main repo. | Fixtures `note_prefix_linked_worktree.json`, `note_prefix_from_subdir.json`. |

Event keys (all fixtures): `session_id`, `transcript_path`, `cwd`, `prompt_id`, `permission_mode`, `hook_event_name` (`UserPromptSubmit`), `prompt`.

## Decision 2: default capture prefix (D18)

Default prefix: **`/note `**. The fallback `;n ` is not needed.

* Reason: the hook sees `/note …` raw, with and without a command file, and the block JSON stops the prompt with no model turn.
* T011 must also write `~/.claude/commands/note.md` (DDR 0003). It covers the case where the hook is missing, and it is the safe path for interactive mode, where Claude Code may reject an unknown slash command before the hook runs (not measured, see below).
* Hook input to expect in T011: the keys above. Use `cwd` for context. Do not use `transcript_path` for anything stored in a note (FR-004: no paths in notes).

## Fixtures (AC 5)

`tests/fixtures/claude_hook/*.json`: six real events, scrubbed (`/work/...` for folders, `/CLAUDE_CONFIG_DIR/...` for the transcript path). `tests/test_T001_fixtures.py` checks each for `prompt`, `cwd`, `session_id`, and for no `/home/` and no `token`.

## Not measured (needs a manual run by the owner)

1. **Cursor, all three facts.** `cursor-agent` is not signed in: `cursor-agent -p …` shows "Press any key to sign in…", and `mcp list` reports no server. Signing in needs a browser and writes to `~/.cursor`, so the spike did not do it. Manual steps:
   1. In a throwaway git repo, create `.cursor/mcp.json`: `{"mcpServers":{"easy-note-probe":{"command":"uv","args":["run","--quiet","--with","mcp<2","python","<path to scripts/probe_server.py>"],"env":{"EASY_NOTE_PROBE_LOG":"/tmp/cursor-probe.jsonl"}}}}`.
   2. Open that folder in Cursor, approve and enable the `easy-note-probe` server, then start a chat (Agent mode).
   3. Read `/tmp/cursor-probe.jsonl`: `start.cwd`, `initialize.clientInfo.name`, `initialize.roots_capability`, `roots`. Repeat in a linked worktree (`git worktree add`).
   4. Remove `.cursor/mcp.json` and `/tmp/cursor-probe.jsonl`.
2. **Interactive Claude Code, hook rows.** Headless cannot show (a) whether the interactive prompt box rejects an unknown `/note` before the hook runs, (b) how the interactive screen shows the block reason, (c) whether the hook runs for a prompt typed while the agent works. Manual steps: use a throwaway repo; add `--settings <file>` with the probe hook as in `scripts/probe_hook.py`; run `claude --settings <file> --setting-sources project`; (a) type `/note hello` with no `note.md` anywhere and note whether Claude shows "Unknown command" and whether an `event-*.json` appears; (b) read what the screen shows; (c) ask for a 20 s task, then type `/note queued` and press Enter while it runs, and note when the event file appears and whether the reason shows.
3. Hook latency: out of scope (T010).
