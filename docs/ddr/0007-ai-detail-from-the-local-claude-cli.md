# 0007. AI detail from the person's own `claude` CLI, opt-in

**Status**: Accepted (owner, chat 2026-11-19: "we can use the claude subscription from the local, with `claude -p`"; "settings section to cover the configured claude account")
**Date**: 2026-11-19
**Deciders**: thunderkds
**Related**: PRD US-018, FR-037, FR-038, NFR-001, NFR-007 · amends DDR 0004 items 1–2

---

## Context

DDR 0004 adds AI detail only when the person runs the `enrich_notes` prompt in a Claude session. The owner wants
it without that step, paid by their Claude subscription, through the Claude Code CLI already on the machine, with
a setting that picks the account. `claude -p` (print mode) has `--output-format json`, `--json-schema`,
`--tools ""`, `--strict-mcp-config`, `--no-session-persistence` and `--model` (checked with `claude --help`,
2026-11-19). Claude Code picks the account by its config folder (`CLAUDE_CONFIG_DIR`).

DDR gate: surprising (a model-free tool starts a model) ✅, genuine trade-off ✅, hard to reverse — partly (text sent). 2 of 3 → DDR.

## Decision

1. easy-note still links no model, SDK or HTTP client. `ai enrich` starts the person's `claude` CLI as a child (argv list, no shell), only when `[ai] enrich = "claude"` (default `"off"`).
2. The child gets the masked note text on stdin and no tools (`--tools ""`), no MCP servers (`--strict-mcp-config`), no saved session, a JSON schema and a timeout. Its answer is untrusted input: it goes through the `note_enrich` writer (caps, kinds, masking, `based_on`).
3. `[ai] claude_config_dir` sets `CLAUDE_CONFIG_DIR` for the child; `ANTHROPIC_API_KEY` and `ANTHROPIC_AUTH_TOKEN` are removed so the subscription, not an API key, pays.
4. `after_save = true` starts `ai enrich <id>` detached after a save; capture never waits and never fails because of it.
5. The `enrich_notes` MCP prompt (DDR 0004) stays.

## Alternatives Considered

| Alternative | Why not chosen |
|---|---|
| Anthropic API with a key | Network code, key storage, separate billing; the owner asked for the subscription |
| Keep only `enrich_notes` | The owner wants no manual step |
| Give the child the easy-note MCP server | A note's text could steer tool calls; JSON on stdout is enough |

## Consequences

- Positive: detail without a manual run; no key in any file; account choice in one setting.
- Negative: masked note text reaches the person's Claude account (opt-in); calls count against subscription limits; each call takes seconds; the CLI's flags are an outside contract (`doctor` checks the command).
