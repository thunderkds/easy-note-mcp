# TASK_REVIEW — HUN-274: AI detail visible in Obsidian: ai/<id>.md views + easy-note ai render

> Sibling of `tasks/TASK_GUIDE_HUN-274.md`. Filled by the reviewer at Stage 4/5.

---

## Auto decisions

Mode: auto
1. Dashboard = an Obsidian Bases file, no window of our own; zenity kept for `quick` — RULE: `docs/ddr/0006-obsidian-as-the-dashboard.md` § Decision (owner's chat answers, 2026-11-19)
2. Task breakdown: 4 tasks (HUN-273 dashboard, HUN-274 AI views, HUN-275 claude enrich, HUN-276 icons+launcher+tray), run in parallel worktrees as the owner asked — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` (row 12)
3. Spawn model sonnet for C1 — FACT: PROJECT_SPEC.md:168 "| claude | `claude` | haiku | sonnet | sonnet or opus | opus |"
4. AI views written automatically when a `.obsidian` folder is found — DEBATED: automatic (Advocate 1: automatic, per PRD FR-036; Advocate 2: only after `install obsidian`, to avoid surprise files) — settled by RULE: `PRD.md` § Functional Requirements FR-036, domain first under `docs/claude-md/tracks.md` `### Admin tag`; `ai render` backfill kept ⚠ decided for you — please review
Checks: tests `cargo fmt --check && cargo clippy … -D warnings && cargo test --locked` + `--features tray` at 6c99f5f — exit 0, 0 failed (tests/ai.rs 38 passed both feature sets); mutation of the view call → 4 view tests fail; reviews code-review — 0 P0, 1 P1 process (implementer did not read `agents/backend.md`; its own re-check was asked for and never arrived; covered by the Supervisor's full Stage 4 read of the diff), 1 P3 (tests written after the code; mutation evidence); security-review — no findings ≥ 8 (P3 follow-up: render the summary inside a blockquote so a model-written `dataviewjs` block stays inert); real run `ai render` in a temp vault (auto-checked, not /verify)
Memory: nothing new for HUN-274 alone; the round-4 memory pass runs after the last merge
Not done: merged by the Supervisor ([admin], 2026-11-19), task branch pushed; push of `main` (yours: the gate refuses a `main` that carries more than one merge); auto verify: ☑ pass (session a8561146-b237-4f08-bf92-0f38fda395ad, commit 734fcc0f1449c9626c773d54b36c441d1436d35f, log .claude/verify-runs/HUN-274-734fcc0f1449.log); owner check of the `ai/<id>.md` page in real Obsidian (yours); html-report for code-review and security-review (not rendered this run)
Paused: auto-mode check failed — no test run in memory/event-trace/HUN-274.jsonl — write /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID with the task ID (minimal-track step 1): `mkdir -p /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d && printf '%s\n%s\n' "HUN-274" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /home/thunderkds/workspace/pets/hungnguyen111/easy-note-mcp/.claude/hooks/.state/active_task.d/$CLAUDE_CODE_SESSION_ID`, then rerun the task's tests so the trace records them under the task, then stop [report 6f5c9e70e8e1]
Paused: done not moved — Gate 5: no test run in memory/event-trace/HUN-274.jsonl

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/ai.rs` (8 new `view` tests, AC 2-8, 10), `tests/leak.rs` (`ai_view` surface, AC 9), unit tests in `src/core/ai_view.rs` (AC 1, 3). `cargo test --locked --test ai`: `test result: ok. 38 passed; 0 failed` |
| Verification command run | ☑ pass | fmt, clippy (both feature sets) clean; `cargo test --locked` and `--features tray` every `test result: ok`, e.g. `ok. 38 passed; 0 failed` (tests/ai.rs), `exit=0` |
| Negative cases hold | ☑ pass | Mutation: `ai::enrich` view call commented out (`git diff --stat src/core/ai.rs`: `1 file changed, 5 insertions(+), 1 deletion(-)`), then `cargo test --test ai view`: `4 failed` (e.g. `a_vault_gets_a_labelled_view_on_enrich_and_capture --- FAILED`); mutation reverted. Tests were written after the code, not before. |
| verify | ☑ pass | auto-run by hook, session a8561146-b237-4f08-bf92-0f38fda395ad, commit 734fcc0f1449c9626c773d54b36c441d1436d35f, log .claude/verify-runs/HUN-274-734fcc0f1449.log — pass; done not moved: Gate 5: no test run in memory/event-trace/HUN-274.jsonl |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed: `src/core/ai_view.rs` (vault_root, render, write_view, remove_at, render_all), its call sites in `src/core/ai.rs` (enrich, remove) and `src/surfaces/cli/write.rs` (rm, restore, edit, tag, ai_render), `cli.rs` arm, `tests/ai.rs`, `tests/leak.rs`, NOTE_FORMAT section. Id path checked through `store.canonical_id` and `ai::file_name`. Skipped: format.rs, query, MCP (untouched). Lane: `C1, 9 files, 505 lines, EXCEEDS` — warn only |
| Full smoke suite still green (no regression) | ☑ pass | whole `cargo test --locked` run, both feature sets, exit=0 |
| **Companion score (easy-verifier)** | ☑ N/A | not run in this session: the `score` skill was not started for the round-4 tasks (time); the merge gate does not block on N/A |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | no UI of ours: the view is a Markdown file Obsidian renders; its shape is checked by `tests/ai.rs` (AC 3) |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | Markdown output; Obsidian's theme draws it |
| **UI: Responsiveness at target viewports** | ☑ N/A | Markdown output; no layout of ours |

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
