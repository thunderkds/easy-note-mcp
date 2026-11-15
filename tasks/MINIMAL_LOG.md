# Minimal-track Decision records

> One Decision record per minimal-track task (`docs/claude-md/tracks.md`), **newest first**: insert each
> new record directly under this note.

## HUN-267 — Align context task pattern with PRD FR-005 · minimal
**Asked:** "you can start automate" (after the Supervisor offered the HUN-254 ticket-pattern fix as the next small item)
**Decisions:**
- Track minimal — one regex in one file, Risk Low: no user data or storage format changes; only adds a `task` where PRD FR-005 says one belongs.
- Use the PRD pattern verbatim, `[A-Z][A-Z0-9]+-\d+`, keeping the existing no-letter-or-digit boundaries — FR-005 declares it; the boundaries are HUN-254's tested `art1234` guard.
- Tracker keys stay case-sensitive (`feat/hun-221` gives no task) — FR-005 says only `T` is case-insensitive.
- Did not touch DDR 0002's "draft" wording — DDRs are owner-approved and its Status is still "Proposed (owner to accept)"; left for the owner.
**Checked:**
- New test `test_task_from_branch[feat/AB2-12-x-AB2-12]`, `[fix/PROJ-1234567-PROJ-1234567]`, `[feat/A-12-None]` — the first two failed before the change (`2 failed, 12 passed`), all pass after
- Suite: `.venv/bin/pytest -q tests` — `205 passed in 11.09s`; `uvx ruff check --isolated` on both files — `All checks passed!`
- Real run: `context.capture(cwd=<temp repo on feat/AB2-12-demo>)` then `store.add` — printed `{'repo': 'rr', 'branch': 'feat/AB2-12-demo', 'task': 'AB2-12'}` and a note file with `task: "AB2-12"`
- Simplify: skipped: under 20 lines
**Files:** `src/easy_note/core/context.py`, `tests/test_context.py`, `tasks/MINIMAL_LOG.md`
**Not done:** merge (the user decides)
