# MEMORY.md — Hot-Tier Memory Index

> **Rules**: Supervisor-only writes. Max 42,000 characters — a ratchet: `/compact-memory` may lower
> it, never raise it to fit growth. One-line summaries + links to cold files.
> Each spawn prompt carries a slice of these lines (craft-spawn-prompt's memory_slice.py) and this
> path to read in full only when the slice falls short; the whole file is not pasted.
> Updated by the Supervisor — prompted by the PostToolUse hook on `git push` / `git merge` (diff-driven pass), or via the `/compact-memory` skill.

---

## Memory Architecture

- [decisions.md](decisions.md) — code + infra architectural decisions (the "why")
- [glossary.md](glossary.md) — canonical biz domain terms and core domain models
- [learnings.md](learnings.md) — specs/requirement clarifications, patterns, gotchas

---

## Index

<!-- Format: - [Title](cold-file.md#section) — one-line summary.
     Target ≤150 chars/entry. Advisory: reported by the size test, never enforced.
     The enforced gate is the 42,000-character whole-file budget above. -->
- [v1 scope kept at all 15 tasks](decisions.md#2026-11-14--v1-scope-kept-at-all-15-tasks-t001t015) — owner keeps T001–T015 + HTTP mode in v1; no v1.1 split
- [MCP SDK pinned <2](decisions.md#2026-11-14--mcp-sdk-pinned-to-mcp12-2-fastmcp-v1-api) — FastMCP v1 API; v2 removed mcp.server.fastmcp (HUN-252)
- [HUN-251 merged over failing score](decisions.md#2026-11-15--hun-251-merged-by-the-owner-over-a-failing-companion-score) — hash false positives; SBOM CI step is the follow-up
- [Linear Todo = Ready for Review](decisions.md#2026-11-15--linear-todo-status-stands-for-ready-for-review) — no In Review status in team HUN; open tasks live in Backlog
- [`[admin]` merge gate behaviour](learnings.md#2026-11-15--admin-merge-gate-behaviour-hun-251-hun-252) — name the task; whole-board check; no main merge-in; branch must hold task ID
- [easy-verifier scores of early branches](learnings.md#2026-11-15--easy-verifier-scores-of-early-branches-hun-251-hun-252) — merge skeleton first; uv.lock makes change_risk abstain; pin CI SHAs + uv sync --locked
