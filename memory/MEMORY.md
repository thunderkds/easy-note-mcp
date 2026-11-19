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
- [Codebase Map](codebase-map.md) — structural snapshot: directory tree, entry points, blast-radius hotspots. Refresh via /map-codebase.
- [v1 scope kept at all 15 tasks](decisions.md#2026-11-14--v1-scope-kept-at-all-15-tasks-t001t015) — owner keeps T001–T015 + HTTP mode in v1; no v1.1 split
- [MCP SDK pinned <2](decisions.md#2026-11-14--mcp-sdk-pinned-to-mcp12-2-fastmcp-v1-api) — FastMCP v1 API; v2 removed mcp.server.fastmcp (HUN-252)
- [easy-verifier flags harness-lock.json hashes](learnings.md#2026-11-15--easy-verifier-counts-claudeharness-lockjson-hashes-as-secrets-hun-251-hun-253) — security ~30 on every branch; fix in easy-verifier
- [HUN-251 merged over failing score](decisions.md#2026-11-15--hun-251-merged-by-the-owner-over-a-failing-companion-score) — hash false positives; SBOM CI step is the follow-up
- [HUN-253 merged over failing score](decisions.md#2026-11-15--hun-253-merged-by-the-owner-over-a-failing-companion-score) — 68 on fixed verifier; verify-log name false positive; build verifier from develop
- [HUN-254 merged over failing score](decisions.md#2026-11-15--hun-254-merged-by-the-owner-over-a-failing-companion-score) — 72; review-file false positives; FR-005 tracker pattern mismatch left open (P2)
- [Rust rewrite; HUN-268 merged over failing score](decisions.md#2026-11-15--app-rewritten-in-rust-hun-268-merged-over-a-failing-companion-score) — DDR 0005; Python app gone; 73 from id-token false positive; Python entries are history
- [HUN-269 merged over failing score](decisions.md#2026-11-15--hun-269-merged-by-the-owner-over-a-failing-companion-score) — 67; parse CCN 23; YAML 1.2 diffs accepted; admin gate needs hand merge on fail row
- [HUN-271/270/272 merged; Rust core complete](decisions.md#2026-11-16--hun-271-hun-270-hun-272-merged-by-the-owner-over-failing-companion-scores-rust-port-of-the-core-complete) — scores 67/71/72; surfaces HUN-257–265 need Rust re-plan; follow-ups listed
- [Mask decoded YAML, not only raw](learnings.md#2026-11-16--masking-raw-yaml-misses-escaped-tokens-hun-270) — `\n` before a token defeats `\b`; mask_decoded
- [Review template trips verifier; admin needs named task](learnings.md#2026-11-15--task_review-template-text-trips-the-verifier-admin-needs-a-named-task-and-a-traced-test-run) — clear Companion placeholder before scoring; `[admin] merge <ID>` only
- [Linear Todo = Ready for Review](decisions.md#2026-11-15--linear-todo-status-stands-for-ready-for-review) — no In Review status in team HUN; open tasks live in Backlog
- [Close-out placeholder check; verifier CLI reports dir](learnings.md#2026-11-16--auto-close-out-placeholder-check-easy-verifier-cli-reports-dir-hun-257) — no `<word>` in Evidence Notes; verifier CLI needs a writable reports dir
- [`[admin]` merge gate behaviour](learnings.md#2026-11-15--admin-merge-gate-behaviour-hun-251-hun-252) — name the task; whole-board check; no main merge-in; branch must hold task ID
- [easy-verifier scores of early branches](learnings.md#2026-11-15--easy-verifier-scores-of-early-branches-hun-251-hun-252) — merge skeleton first; uv.lock makes change_risk abstain; pin CI SHAs + uv sync --locked
- [rmcp arg errors echo values; spawn pull rewrites worktree board](learnings.md#2026-11-16--rmcp-argument-errors-echo-escaped-values-spawn-pull-rewrites-the-worktrees-board-cache-hun-258) — mask via call_tool override; git restore board cache before review
- [board_sync update pushes its own checkout's guide](learnings.md#2026-11-17--board_syncpy-update---guide-pushes-the-guide-of-the-checkout-the-script-lives-in-hun-265) — run update from the task worktree; stop-hook Paused line in review is not the agent's
- [Merge gate pulls the board; commit the cache before parallel merges](learnings.md#2026-11-17--the-merge-gate-pulls-the-board-before-every-merge-parallel-task-branches-need-the-cache-committed-first-hun-262-hun-263-hun-265) — guide copies dirty main; [admin] push of main refused after a cache commit; test-only tasks need the owner's /verify
