**Date**: 2026-11-15
**Complexity Level**: C0
**Risk Level**: Low
**Priority**: P2
**Assigned agent**: Owner (manual run) + QA-Automation-Agent (records the result)
**Agent guide**: `agents/qa.md`
**Plan ID**: T016 (follow-up of HUN-251 / T001)

---

## Requirement (Pillar 1 — Adapt the requirement)

HUN-251 measured Claude Code and Codex but not Cursor: `cursor-agent` is not signed in, and sign-in needs a browser and writes to `~/.cursor`. Owner decision 2026-11-15: accept HUN-251 with the Cursor row open and track the Cursor run as this follow-up.

**Restated intent**:

> Fill the Cursor row of `docs/experiments/T001-client-launch.md` (process cwd, `roots/list`, `clientInfo.name`, in a repo and in a linked worktree) so the README's Cursor line can say more than "pass `cwd`".

**Out of scope**:

* Any product code change; FR-004's order already covers every answer Cursor can give.

**Requirement Refs**:

* FR-004, FR-007, PRD Open Question #1

---

## Dependencies & Reachability

**Depends on**: HUN-251 (plan T001) — `scripts/probe_server.py` and the manual steps in `docs/experiments/T001-client-launch.md` § Not measured

**Entry point**: Standalone — N/A: a manual measurement; output is one table row in `docs/experiments/T001-client-launch.md`

---

## Acceptance Criteria

| \# | Criterion (testable) | Traces to requirement |
| -- | -- | -- |
| 1 | Owner runs the 4 manual steps in `docs/experiments/T001-client-launch.md` § Not measured item 1 (signed-in Cursor, throwaway repo, project-scope `.cursor/mcp.json`) | Cursor row |
| 2 | The Cursor row is filled from `/tmp/cursor-probe.jsonl`: `start.cwd`, `clientInfo.name`, roots capability and answer, for the repo and a linked worktree | FR-004, FR-007 |
| 3 | The README line for Cursor (HUN-264) is updated if Cursor gives roots or a project cwd | FR-004 |
| 4 | The throwaway `.cursor/mcp.json` and `/tmp/cursor-probe.jsonl` are removed; `~/.cursor/mcp.json` checksum unchanged | config safety |

---

## Evaluation & Acceptance

### Success Criteria (observable, pass/fail)

| \# | Given | Expect | How it's checked |
| -- | -- | -- | -- |
| 1 | The probe log from a Cursor chat | a filled Cursor row | manual, pasted |

### Verification Command (exact, runnable)

```bash
# ci: skip — manual run in the Cursor desktop app
uv run --with pytest pytest tests/test_T001_fixtures.py -q
```

---

## UI / Design Acceptance Criteria

N/A — no UI component.

---

## Completion Checklist

- [ ] Manual run done and pasted
- [ ] Cursor row filled
- [ ] Supervisor notified