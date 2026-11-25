# GRILL — round 6 (AI does the jobs, scheduler, settings, UX)

Source: owner's request 2026-11-25. Direction: "the user just takes notes; the AI does all the jobs."

## Locked at step Requirements
- AI never changes a raw note (stays DDR 0004 / NFR-012). AI writes only AI files; the dashboard builds views from them; every AI change is logged and undoable.
- Scheduler: the tray runs `easy-note ai run` on an interval from Settings; cron or systemd can run the same command.
- Organizing axes: project (repo, else AI-guessed), topic/keyword, kind (idea, todo, bug, question, draft). "Place" dropped.
- Agent context: a rolling digest per project and topic, read through a new read-only MCP tool (to confirm in grilling).
- Tasks: 1 one dashboard window, 2 show `updated`, 3 settings, 4 `ai run` organizer, 5 tray timer, 6 views + digest, 7 UX pass.

## Resolved in grilling
| # | Question | Decision |
|---|---|---|
| G1 | Consent for scheduled AI | Settings switch "Organize automatically", off by default. It shows once what is sent (masked text, own `claude` CLI, account, model). NFR-001 and NFR-007 gain one named exception for scheduled runs. |
| G2 | MCP surface for organized view | One read-only `note_context` tool (digest + up to 10 notes; no args = project and topic lists). 7 to 8 tools; owner re-approves (open question 8). FR-051. |
| G3 | Your fix of an AI guess | Kept. Overrides in `.easy-note/ai-overrides.json`, never in a note or AI file; AI skips an overridden field; topic rename/merge covers all its notes; log + undo. FR-050. |
| G4 | Scheduled run limits | Interval (60 min) + `max_per_run` (20) + only new or stale notes; digest once a day; back-off on failure. No daily call cap. FR-047..049. |
| G5 | What "check the draft" does | AI flags kind=draft with reason and suggested completion in the AI file; Drafts view; you finish it by editing. No Apply button. FR-047, FR-052. |

## Derived without a question (from "raw note always kept")
- `updated` is set only by the person's edits; AI uses its own `at` (FR-046). Shown in list and window.
- Single window per notes dir via a lock file; a second call focuses it (FR-045).

PRD changed: US-020..023, FR-045..053, NFR-014..015, FR-012 amended, Out of Scope and Open Question 8 added.
