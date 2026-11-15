# decisions.md — Cold Tier: Architectural & Infrastructure Decisions

> **Rules**: Supervisor-only writes. Each entry: `### YYYY-MM-DD — Title`, then **Decision**, **Why**, and **Files** (cite paths — the diff-driven pass greps this file by changed file path).

## Architecture

### 2026-11-14 — v1 scope kept at all 15 tasks (T001–T015)
**Decision**: Owner keeps every planned task in v1, including T012 (`quick`, hotkey, launcher), T013 (`inbox/`), T014 (Claude detail), T015 (tray) and the streamable-HTTP mode. The Supervisor's proposal to move T012–T015 to v1.1 and drop HTTP was rejected.
**Why**: Owner call ("keep all in v1"). Ship order still follows the vital slice in PROJECT_SPEC.md (T011 + T007 `n` + T005 first).
**Files**: PROJECT_SPEC.md, PRD.md, linear-tasks.md

## Infrastructure

### 2026-11-14 — MCP SDK pinned to mcp>=1.2,<2 (FastMCP v1 API)
**Decision**: easy-note uses the official SDK's v1 FastMCP API (`mcp.server.fastmcp`) and pins `mcp>=1.2,<2`. Moving to the v2 `MCPServer` API is a separate, owner-approved change.
**Why**: HUN-252 found that `mcp` 2.3.0 removed `mcp.server.fastmcp`; PROJECT_SPEC names FastMCP (D11).
**Files**: pyproject.toml, src/easy_note/surfaces/mcp.py

### 2026-11-15 — Linear "Todo" status stands for Ready for Review
**Decision**: `.claude/board.json` maps the Linear status `Todo` to the kit's Ready for Review section (`"status_map": {"Todo": "Ready for Review"}`). Open tasks sit in Linear `Backlog`, which the kit shows as Todo.
**Why**: Team HUN has no "In Review" status and the Linear tools cannot create one; the owner chose a mapping over adding the status. `Backlog` was tried first and is wrong: all open tasks use it. Revert the map if an "In Review" status is added.
**Files**: .claude/board.json, PROJECT_KANBAN.md
