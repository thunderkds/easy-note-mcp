# TASK_REVIEW — HUN-274: AI detail visible in Obsidian: ai/<id>.md views + easy-note ai render

> Sibling of `tasks/TASK_GUIDE_HUN-274.md`. Filled by the reviewer at Stage 4/5.

---

## Auto decisions

Mode: auto
1. Dashboard = an Obsidian Bases file, no window of our own; zenity kept for `quick` — RULE: `docs/ddr/0006-obsidian-as-the-dashboard.md` § Decision (owner's chat answers, 2026-11-19)
2. Task breakdown: 4 tasks (HUN-273 dashboard, HUN-274 AI views, HUN-275 claude enrich, HUN-276 icons+launcher+tray), run in parallel worktrees as the owner asked — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 12)
3. Spawn model from the CLI Model Table — FACT: `PROJECT_SPEC.md:168` "| claude | `claude` | haiku | sonnet | sonnet or opus | opus |"
4. AI views written automatically when a `.obsidian` folder is found — DEBATED: automatic (Advocate 1: automatic, per PRD FR-036; Advocate 2: only after `install obsidian`, to avoid surprise files) — settled by RULE: `PRD.md` § Functional Requirements FR-036, domain first under `docs/claude-md/tracks.md` `### Admin tag`; `ai render` backfill kept ⚠ decided for you — please review
Checks: pending (Stage 4)
Memory: pending (Stage 5)
Not done: merge (admin: after the auto verify pass); auto verify: not started
Paused: auto-mode check failed — no test run in memory/event-trace/HUN-274.jsonl — write /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID with the task ID (minimal-track step 1): `mkdir -p /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d && printf '%s\n%s\n' "HUN-274" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID`, then rerun the task's tests so the trace records them under the task, then stop; Risk Medium: decision 3 (`FACT`) is a choice — its FACT has no Citation (`path:line "quoted text"`) — so it must be `DEBATED` with exactly 2 Advocates named; extended track, but the trace has no `code-review` Skill call (Stage 4 runs in auto mode); Risk Medium, but the trace has no `security-review` Skill call [report fe125eb56fdd]

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/ai.rs` (8 new `view` tests, AC 2-8, 10), `tests/leak.rs` (`ai_view` surface, AC 9), unit tests in `src/core/ai_view.rs` (AC 1, 3). `cargo test --locked --test ai`: `test result: ok. 38 passed; 0 failed` |
| Verification command run | ☑ pass | fmt, clippy (both feature sets) clean; `cargo test --locked` and `--features tray` every `test result: ok`, e.g. `ok. 38 passed; 0 failed` (tests/ai.rs), `exit=0` |
| Negative cases hold | ☑ pass | Mutation: `ai::enrich` view call commented out (`git diff --stat src/core/ai.rs`: `1 file changed, 5 insertions(+), 1 deletion(-)`), then `cargo test --test ai view`: `4 failed` (e.g. `a_vault_gets_a_labelled_view_on_enrich_and_capture --- FAILED`); mutation reverted. Tests were written after the code, not before. |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☐ pass / ☐ fail | [what was reviewed vs. skipped, and why] |
| Full smoke suite still green (no regression) | ☑ pass | whole `cargo test --locked` run, both feature sets, exit=0 |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☐ pass / ☐ fail / ☐ N/A | [screenshot path or LLM verdict — required for UI tasks, Hard-Stop Gate 6] |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☐ pass / ☐ fail / ☐ N/A | [method used + output] |
| **UI: Responsiveness at target viewports** | ☐ pass / ☐ fail / ☐ N/A | [viewports tested, any overflow findings] |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (captured 2026-11-19, `date -u` = Thu Nov 19 05:09:23 UTC 2026, before any implementation commit; temp vault with `.obsidian/`, one note, one hand-written AI file):

```
vault/notes/.easy-note/ai/20261119-120923-8eb2.json
vault/notes/2026/11/20261119-120923-8eb2.md
(no ai/ folder; nothing visible to Obsidian)
$ easy-note ai render
error: unrecognized subcommand 'render'
Usage: easy-note ai <COMMAND>
exit=2
```

**AFTER** (same temp vault, after the change):

```
$ easy-note ai render
1 view written
exit=0
(<notes dir>/ai/<id>.md now exists; outside a vault: "not in an Obsidian vault (no .obsidian folder found)", exit 1)
```

**DELTA**: In an Obsidian vault the person now sees each note's AI detail as a labelled, linked, read-only page under `ai/`, and can backfill with `easy-note ai render`.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]
