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

### 2026-11-15 — HUN-251 merged by the owner over a failing Companion score
**Decision**: The owner merged HUN-251 (client launch spike) into `main` (093f5cc) by hand, overriding the ☑ fail Companion score row (easy-verifier overall 67 < 80). Verify passed at 5116549; 27 tests pass on main.
**Why**: Security 30 came from 152 hex hashes read as secrets (147 in `.claude/harness-lock.json`, 5 sha256 checksums used as AC 6 evidence); the rest of the gap is repo-wide (supply-chain 55: no SBOM/provenance; test-strategy 65). An SBOM CI step is the follow-up that would lift every branch.
**Files**: tasks/TASK_REVIEW_HUN-251.md, .claude/harness-lock.json, .github/workflows/ci.yml

### 2026-11-15 — HUN-253 merged by the owner over a failing Companion score
**Decision**: The owner merged HUN-253 (note format and store) into `main` (225ea7c) by hand and pushed, overriding the ☑ fail Companion score row (easy-verifier 0.2.0 overall 68 < 80). Verify passed at 703cdcb (hook accepted it for review-only tip 64d1a09); code-review P0/P1 0; security-review pass; 99 tests pass on merged main.
**Why**: Rescored on an image built from easy-verifier-mcp `develop` (fix b958920 exempts harness-lock.json hashes). Security 30 now comes from one `high_entropy_string` hit: the verify-run log name `HUN-253-703cdcbbc189.log` in the review file's verify row (commit-hash prefix, not covered by b958920). Real gaps left as follow-ups: CCN 16 in `format._validate`, 9% requirement tracing, no SBOM CI step, no formatter config, 18% tests without a direct assert. Local easy-verifier `main` (de74623) lacks the fix; build from `develop`.
**Files**: tasks/TASK_REVIEW_HUN-253.md, src/easy_note/core/format.py, .github/workflows/ci.yml

### 2026-11-15 — HUN-254 merged by the owner over a failing Companion score
**Decision**: The owner merged HUN-254 (context capture) into `main` (aacfcdb) by hand and pushed, overriding the ☑ fail Companion score row (easy-verifier 0.2.0 overall 72 < 80). Verify passed at faa3cec; code-review P0/P1 0; security-review pass; 146 tests pass on merged main.
**Why**: Security 30 came from two `key_material_segment` false positives in the review file (mutation test id `fix/art1234-None`; the template's placeholder text in the Companion row). Open follow-up (P2): `context.py` tracker pattern `[A-Z]{2,}-\d{1,6}` differs from PRD FR-005 `[A-Z][A-Z0-9]+-\d+` (`AB2-12` gives no task); owner merged without aligning it.
**Files**: tasks/TASK_REVIEW_HUN-254.md, src/easy_note/core/context.py
