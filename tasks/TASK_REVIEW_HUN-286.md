# TASK_REVIEW — HUN-286: Dashboard window: read and edit a note in the app, not in Obsidian

> Sibling of `tasks/TASK_GUIDE_HUN-286.md`. Everything here is **filled by the reviewer at Stage
> 4/5** — it is deliberately NOT in the guide, because the implementing agent re-reads the guide on
> every turn and never fills these two sections.
>
> Consumers resolve each section **guide first, this file second** (`.claude/hooks/lib/guide_sections.py`):
> a legacy guide that still carries these sections inline keeps working unchanged, and a stray
> review file can never override an inline section.

---

<!-- Optional: the final report, only for a task run in auto mode (the user's `[auto]` or `[admin]` tag; with `[admin]`, the `Not done:` line names `merge (admin: after the auto verify pass)`,
`docs/claude-md/tracks.md` `## Auto mode`). To use it, copy the block below out of this comment, to this
place, and replace every <...>. Keep `Mode: auto` as its first line, exactly as written: T182's hook reads
it. Write it to this file before the run ends; the chat message is a copy, not the report. The same
block is the Supervisor's final chat message. It is not the `delivery-report` page and not
Evidence. A taste decision (what the user wants) also ends with the flag. Write `DEBATED` only when ≥2
Advocate `Agent` calls ran for that question, naming each Advocate's option. A decision with no Advocates is
labelled `RULE`, `FACT` or `SAFER OPTION`, never `DEBATED`. A decision flagged `⚠ decided for you` is a choice, and is never labelled `FACT`. A
safety action a rule requires (for example the backup before a delete) is `RULE` without the flag.
At Risk Medium or High, a `RULE:` counts only when it names a written rule (`path:line`,
`path § <heading>`, or `docs/claude-md/tracks.md` `### <section>`), and a `FACT:` needs a Citation
(`path:line "quoted text"`); without one it is a choice and must be `DEBATED`. Good: `2. backed up the CSV
— RULE: `docs/claude-md/tracks.md` `### Stops that stay``. Bad: `1. kept recurring.csv — RULE: Surgical Changes`.
`Memory:` records the diff-driven memory pass you run before you stop: each entry written, or `nothing new`.
After an auto verify pass the hook, never you, adds `Closed: row moved to Done <date> — …` and moves the row,
or adds `Paused: done not moved — <reason>`. It moves the row only when this block holds `Mode: auto`, a
numbered decision, `Checks:`, `Memory:`, the `Not done:` auto verify slot with its result, and `Not done:`
naming the merge — no `<...>` left (`docs/claude-md/tracks.md` `### Auto close-out`).
With `/verify (yours)`, your pass goes in the `verify` row below: Result `☑ pass`, Notes `/verify by the user, session <id>, commit <sha>`.

## Auto decisions

Mode: auto
1. <what was decided> — RULE: <the rule>
2. <what was decided> — FACT: <path>:<line> "<quoted text>"
3. <what was decided> — DEBATED: <option> (Advocate 1: <its proposal>; Advocate 2: <its proposal>) ⚠ decided for you — please review
4. <what was decided> — SAFER OPTION: <why> ⚠ decided for you — please review
Paused: <only when a stop that stays paused the task: which one, and why>
Checks: tests `<command>` — <result>; reviews <skill — result>; real run `<command>` — printed `<what it printed>` (auto-checked, not /verify)
Memory: <each entry the memory pass wrote, as `file — entry` | nothing new>
Not done: merge (yours); <auto verify: running (T182) | auto verify: <result> | /verify (yours), with [admin] only after the one retry also ended blocked>

-->

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/window_api.rs` (edit AC 1–3, 9 incl. `edit_errors_write_nothing`, `another_note_is_not_selected_while_editing`, `the_focus_refresh_keeps_edit_mode_and_the_draft`, `esc_and_cancel_leave_edit_mode_without_writing`), `tests/fixtures/window/page_harness.js` (14 edit scenarios), `tests/dashboard.rs` (`--note` refusals, AC 7), `src/surfaces/tray.rs` `a_recent_note_opens_the_window_when_it_can_else_the_file` (AC 8). Supervisor rerun 2026-11-21: `cargo test --locked --test window_api` → `test result: ok. 60 passed; 0 failed`. Frontend agent's full Verification Command 19:01–19:09: exit 0, 1768 passed, 0 failed |
| Verification command run | ☐ pass / ☐ fail | [paste actual output] |
| Negative cases hold | ☐ pass / ☐ fail | [paste the red run against the prior content, or the mutation diff (`git diff --stat`, non-empty) before each verdict] |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed `window/api.rs` (`edit`), `window/mod.rs` (`initial_note_script`), `cli.rs` + `dashboard.rs` (`--note`, `canonical_id` → `resolve` validates the id; `NotFound` only after a valid id), `tray.rs` (`open_note`), `assets/{app.js,ipc.js,app.css}`. Not reviewed: unchanged core (`Store::update` reused as is) |
| Full smoke suite still green (no regression) | ☐ pass / ☐ fail | |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☑ pass | headless Chrome on a scratch copy of `assets/` with a stub IPC (easy-ui not available this session): `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/f7aa1818-7158-435d-8b50-41e885be095a/scratchpad/hun286/shots/after-{read,edit,saved,changed,switch,empty,inbox,unknown,cancel}-1280.png`, BEFORE `before-read-1280.png`. ux-evaluate: 11 pass, 0 fail, 3 not checked (tray click: owner machine; 375 px and touch: N/A for an 800 px-min desktop window). Supervisor looked at `after-edit-1280.png`: text box, Tags field, Save/Cancel, key hints in the panel |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ pass | `app.css` edit rules use existing tokens only (agent CSS audit vs `docs/design/graphite-mint-tokens.json`); text box 16px panel body font; Save is the one mint primary button |
| **UI: Responsiveness at target viewports** | ☑ pass | 800 px and 1280 px: `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/f7aa1818-7158-435d-8b50-41e885be095a/scratchpad/hun286/shots/after-*-800.png`, `*-800-full.png`. No horizontal scroll at 800; in a short 800 px window Save/Cancel sit below the fold and the panel scrolls to them |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (backend-developer, worktree `easy-note-mcp-HUN-286` at `38d0f01`, no implementation
commit yet; temp `HOME`, `XDG_CONFIG_HOME`, `EASY_NOTE_DIR`):

```
$ date -Iseconds; easy-note dashboard --help; easy-note dashboard --note ../x
2026-11-21T18:16:59+07:00
Open the dashboard: easy-note's own window in a `window` build on a desktop, else `easy-note.base` in Obsidian (run `install obsidian` first)

Usage: easy-note dashboard

Options:
  -h, --help  Print help
exit=0
error: unexpected argument '--note' found

Usage: easy-note dashboard

For more information, try '--help'.
exit=2
```

```
$ date -Iseconds; cargo test --locked --test window_api edit_refuses_when_the_note_changed
2026-11-21T18:18:38+07:00
thread 'edit_refuses_when_the_note_changed_outside_the_window' (77904) panicked at tests/window_api.rs:1787:5:
  left: Object {"error": String("unknown command")}
 right: Object {"error": String("the note changed outside the window")}
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.10s
```

The window API answers `edit` with `unknown command`; `dashboard` has no `--note`; the tray's
recent-note click runs `xdg-open <file>` (`src/surfaces/tray.rs` `argv`, `Action::Open(path)`).

**BEFORE, page side** (frontend-developer, worktree at `57215d3`, no frontend commit yet;
2026-11-21T18:42:01+07:00; headless Chrome on a scratch copy of `src/surfaces/window/assets/` with a
stub `window.ipc`, meta CSP dropped for `file://`):
`/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/f7aa1818-7158-435d-8b50-41e885be095a/scratchpad/hun286/shots/before-read-1280.png`
— detail panel of a `cli` note: the actions row is `Copy id` and `Move to trash` only (the backend
commit `d9b84fb` already removed `Open in editor`); there is no `Edit` button and no way to change
the text or tags in the window. `before-inbox-1280.png` (same folder): an inbox note shows the same
row, with no read-only line.

**AFTER**: [same command, post-change] OR [verbatim excerpt of the new content]

**DELTA**: [one sentence — what a user can now do that they could not before]

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]

## Stage 4 — code-review and security pass (Supervisor, 2026-11-21)

Scope: `main...feat/HUN-286-edit`. Personas: correctness, testing, maintainability, standards, security (input handling: a webview IPC now writes note files), over-engineering, api (`dashboard --note`, IPC contract), ux-evaluate (agent run, verdicts above).

Security pass (done inline: the built-in `security-review` reviews the current branch of the main checkout, which is the other session's `.deb` branch):
- `edit` writes only through `Store::update` (guard masking, re-validate, atomic rename, `updated` stamp); `ensure_writable` refuses inbox notes before any write.
- The id goes through `store.read` / `resolve` (path-safe id check); `dashboard --note` refuses a bad id with exit 2 before the window opens.
- The initial note id reaches the page only as a serde_json string in an initialization script; no raw splice. CSP and pinned origin (HUN-284) unchanged.
- No command left that runs `xdg-open` from the window (`the_window_api_runs_no_xdg_open`). The tray click is an argv list.
- No P0/P1 security finding.

- P0: None.
- P1: None.
- P2: None.
- P3: `window/api.rs` `edit` — read, compare with `base`, then `Store::update` is not one atomic step; a write by another app in that gap of milliseconds is overwritten. Same window as `easy-note edit`; accepted.
- P3: merge order — `src/surfaces/tray.rs` overlaps with HUN-285 (`argv` / `icon::start`). Rebase onto `feat/HUN-285-tray` was blocked by the pipeline gate while this task was In Progress; resolve at merge time (HUN-285 first).

Verdict: no P0/P1; 0 safe fixes needed.
