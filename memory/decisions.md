# decisions.md — Cold Tier: Architectural & Infrastructure Decisions

> **Rules**: Supervisor-only writes. Each entry: `### YYYY-MM-DD — Title`, then **Decision**, **Why**, and **Files** (cite paths — the diff-driven pass greps this file by changed file path).

## Architecture

### 2026-11-14 — v1 scope kept at all 15 tasks (T001–T015)
**Decision**: Owner keeps every planned task in v1, including T012 (`quick`, hotkey, launcher), T013 (`inbox/`), T014 (Claude detail), T015 (tray) and the streamable-HTTP mode. The Supervisor's proposal to move T012–T015 to v1.1 and drop HTTP was rejected.
**Why**: Owner call ("keep all in v1"). Ship order still follows the vital slice in PROJECT_SPEC.md (T011 + T007 `n` + T005 first).
**Files**: PROJECT_SPEC.md, PRD.md, linear-tasks.md

## Infrastructure
