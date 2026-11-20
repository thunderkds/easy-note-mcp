# TASK_REVIEW — T[NNN]: [Short Title]

> Sibling of `tasks/TASK_GUIDE_T[NNN].md`. Everything here is **filled by the reviewer at Stage
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
Not done: auto verify: not started (the Supervisor starts it at Ready for Review); the P2/P3 review findings above (follow-ups)

-->

## Auto decisions

Mode: auto
1. (Frontend slice) `fonts/OFL.txt` is kept in the repo next to the five woff2 faces but is not in the `asset()` table, so `easy-note://localhost/fonts/OFL.txt` answers 404; the five fonts are served as `font/woff2` and fall under the CSP `default-src easy-note:` (no separate `font-src` needed) — RULE: `tasks/TASK_GUIDE_HUN-278.md:77` "IBM Plex Sans + Mono woff2 (OFL, licence file next to them) bundled; no `http:`/`https:` URL in any asset (test)"; tested by `every_served_asset_is_scanned_and_safe` (`asset("/fonts/OFL.txt").is_none()`).
2. (Frontend slice) The focus refresh is the shell's `REFRESH` script calling `window.__easyNote.refresh = loadBoard`, not a DOM `focus` listener (the two together would load the board twice per focus) — FACT: src/surfaces/window/mod.rs:18 "pub const REFRESH: &str"; `ipc.js` (loaded before `app.js`) creates `window.__easyNote`; tested by `the_focus_refresh_calls_a_function_the_page_defines`.
3. (Frontend slice) The masked notice reads `secret masked`, not the design's `1 secret masked`, because `detail` sends a boolean, not a count — FACT: src/surfaces/window/api.rs:111 "json!(note.body.contains(" and tasks/TASK_GUIDE_HUN-278.md:168 "a masked note shows".
4. (Frontend slice, changed by the Supervisor after a Debate) An empty notes dir shows `No notes yet. Your first one appears here.`, the design set's only empty-notes text; the board design has only `No note matches “{{query}}”.`, which would read `No note matches “”.` — DEBATED: use the tray design's empty text (Advocate 1: use `No notes yet. Your first one appears here.` from docs/design/TrayMenu.dc.html:42; Advocate 2: same option, for AC row tasks/TASK_GUIDE_HUN-278.md:177 "the design's empty text") ⚠ decided for you — please review.
5. (Frontend slice, changed by the Supervisor after a Debate) A failed `board` load shows `Could not load notes: <error>` with no hint: `board` cannot fail on the notes dir, so "Check the notes dir" pointed to the wrong cause; a focus still re-runs `board` — DEBATED: the short text (Advocate 1: `'Could not load notes: ' + msg`, citing src/surfaces/window/api.rs:50 "fn board(store: &Store) -> Value {"; Advocate 2: same option) ⚠ decided for you — please review.
6. (Frontend slice) At 800 px the detail panel wraps below the note list (the design's own flex rule, `docs/design/Main.dc.html:50` `flex: 999 1 520px` and `:91` `flex: 1 1 380px`); a click does not scroll it into view. Kept as designed — RULE: tasks/TASK_GUIDE_HUN-278.md:139 "detail panel below or beside without overlap; no horizontal scroll"; checked in overflow-check-800.png.
7. (Frontend slice) Screenshots: headless Chrome on a scratch copy of `assets/` with a stub `window.ipc` fed the real `window::api::handle` answers over 6 sample notes, plus one capture of the real window (`GDK_BACKEND=x11`, `import -window`). The scratch copy drops the page's meta CSP because a `file://` page cannot load scripts under `default-src easy-note:`; the real-window capture shows the page under the real CSP — RULE: `tasks/TASK_GUIDE_HUN-278.md:125` "screenshot of the page (`assets/index.html` with a stub `window.ipc` fed the `board` JSON of the test fixture)".
8. (Frontend slice) ux-evaluate ran at 1280 and 800 px, not 375: the window cannot be narrower than 800 — FACT: src/surfaces/window/mod.rs:150 ".with_min_inner_size(LogicalSize::new(800.0, 560.0))" and `tasks/TASK_GUIDE_HUN-278.md:139` "800 px (smallest window)".
Checks: tests: the guide's full Verification Command (fmt, clippy + test in default / `tray` / `window`, `! cargo tree … (wry|tao)`) — EXIT=0, 72 `test result: ok`, 0 FAILED, 0 warnings; reviews: ux-evaluate (frontend agent) — 3 of 3 UX rows pass, baseline 6 pass / 3 not checked (375-only checks); real run `easy-note dashboard` (window build, temp HOME / XDG_CONFIG_HOME / EASY_NOTE_DIR) — the window stayed open and showed the 6 sample notes, nothing printed (auto-checked, not /verify); Supervisor Stage 4: code-review — 0 P0, 0 P1, 7 P2, 5 P3 (none fixed in this task; P2 follow-ups: tao panic when DISPLAY is set but unusable bypasses the FR-034 fallback, double board load at start-up, full list rebuild per keystroke, full body in board payload, detail panel not sticky at 800 px, stale `src/surfaces/cli.rs:198` dashboard help, stale PRD.md:159 non-goal); security-review — pass with findings (2 P3: drop `unsafe-inline` from the header CSP, pin `easy-note://localhost`)
Memory: nothing new (frontend agent; memory writes are Supervisor-only)
Not done: merge (yours; blocked by the merge gate: Companion score `☑ fail`, overall 74 < 80; both hits behind security 30 are placeholder text in this file); auto verify: pass (commit 6c04b4244a52, log .claude/verify-runs/HUN-278-6c04b4244a52.log); the P2/P3 review findings above (follow-ups)
Paused: done not moved — Gate 6: the `UI: Visual regression` row has no evidence; report incomplete: auto verify slot; report incomplete: Not done: merge

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | New: `tests/window_api.rs` (13 tests: AC 3–7 board/detail/counts/top 3/empty dir/inbox/bad requests incl. `../../etc/passwd`/planted-secret leak; AC 2 `use_window` truth table; AC 9 CSP, asset table, navigation guard, reply quoting, asset scan for DOM sinks and remote URLs; IPC `req` echo); `src/surfaces/window/mod.rs` `shell::tests` (`--features window`: AC 11 `dmabuf_is_set_only_when_unset`, AC 9 `every_response_carries_the_csp`); `tests/perf.rs` `perf_window_board` (AC 12, ignored by default). `cargo test --locked --features window`: `Running tests/window_api.rs` … `test result: ok. 13 passed; 0 failed; 0 ignored` · `test surfaces::window::shell::tests::dmabuf_is_set_only_when_unset ... ok` · `test surfaces::window::shell::tests::every_response_carries_the_csp ... ok`. AC 12, `cargo test --release --locked --test perf perf_window_board -- --ignored --nocapture`: `perf window_board: n=30 first 140.7 ms p50 120.5 ms p95 123.9 ms max 124.0 ms` / `test result: ok. 1 passed` (budget 300 ms, 5,000 notes). AC 8 and 10 (the page) are not covered here: the placeholder page is replaced by the Frontend slice (feat/HUN-278-page). Frontend slice (2026-11-20) adds 2 tests to `tests/window_api.rs` (now 15): `every_reference_in_the_page_is_served` (AC 8/9: every `src=`/`href=` in `index.html` and `url(...)` in `app.css` resolves through `asset()`, the 5 Plex faces included) and `the_focus_refresh_calls_a_function_the_page_defines` (AC 10: `REFRESH` calls `window.__easyNote.refresh`, `app.js` sets it to `loadBoard`, `ipc.js` loads before `app.js`); `every_served_asset_is_scanned_and_safe` now also checks each woff2 is served as `font/woff2` and `fonts/OFL.txt` is not served; `shell::tests::every_response_carries_the_csp` covers `/ipc.js`. Run: `Running tests/window_api.rs` … `test result: ok. 15 passed; 0 failed; 0 ignored`. |
| Verification command run | ☑ pass | Full guide command (fmt, clippy + test in default / `tray` / `window`, `! cargo tree … (wry\|tao)`) run in the worktree, exit `EXIT=0`; 72 `test result: ok` lines, no `FAILED`, no clippy warning. Tail: `Running tests/window_api.rs (target/debug/deps/window_api-6b2c8de9b77d2b51)` … `test no_planted_secret_leaves_through_board_or_detail ... ok` / `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s` / `Doc-tests easy_note_mcp` / `test result: ok. 0 passed; 0 failed` / `EXIT=0`. Default build: `tests/leak.rs` `test result: ok. 46 passed`; `tests/dashboard.rs` `test result: ok. 5 passed` (unchanged pin test); `tests/enrich.rs` `test result: ok. 24 passed` in all three runs (the known timing test did not fail this time). Re-run by the Frontend-Implementer 2026-11-20 after the page slice: `EXIT=0`, 72 `test result: ok` lines, 0 `FAILED`, 0 clippy warnings; tail: `test no_planted_secret_leaves_through_board_or_detail ... ok` / `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s` / `Doc-tests easy_note_mcp` / `test result: ok. 0 passed; 0 failed; …` / `EXIT=0`; `--features window`: `test surfaces::window::shell::tests::dmabuf_is_set_only_when_unset ... ok`, `test surfaces::window::shell::tests::every_response_carries_the_csp ... ok`. |
| Negative cases hold | ☑ pass | Mutation (Supervisor, 2026-11-20): appended `// fetch("https://example.com/x"); el.innerHTML = x;` to `src/surfaces/window/assets/app.js` (`git diff --stat`: `src/surfaces/window/assets/app.js | 5 +++--`), ran `cargo test --locked --features window --test window_api` → `test every_served_asset_is_scanned_and_safe ... FAILED` / `test result: FAILED. 14 passed; 1 failed`; restored the file → `test result: ok. 15 passed; 0 failed`. Also in the suite: `../../etc/passwd` detail request refused, planted-secret leak test. |
| verify | ☑ pass | auto-run by hook, session 6f23c09d-e04d-47f2-be28-cecf40e38ed9, commit 6c04b4244a52c75bd67fae7552e3b968ba446b18, log .claude/verify-runs/HUN-278-6c04b4244a52.log — pass; done not moved: Gate 6: the `UI: Visual regression` row has no evidence |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Reviewed `main...HEAD` only: `src/surfaces/window/**` (mod.rs, api.rs, assets), `src/surfaces/dashboard.rs`, `src/surfaces/mod.rs`, `Cargo.toml`/`Cargo.lock` (wry/tao behind feature `window`), `tests/window_api.rs`, `tests/perf.rs`. Skipped the store, MCP and tray code: the window reads them through the existing `Store` API and changes none of it. |
| Full smoke suite still green (no regression) | ☑ pass | Supervisor re-run 2026-11-20 at `9d88901` (after the two Debate text changes): the full Verification Command (default, `tray`, `window` builds) → `EXIT=0`, 72 `test result: ok`, 0 `FAILED`, 0 warnings. |
| **Companion score (easy-verifier)** | ☑ fail | overall 74 (floor missed: 74 < 80), change_risk 100 pass (feature, confirmed; 7 test pairs confirmed), scope changes, ref `main...HEAD` at `9d88901`, abstentions: solution-fit (no_static_rule, then agent-rated 80 at the evaluate gate), reliability (insufficient_measured_weight), api, data, operations, infrastructure (below_coverage_floor), gates: solution-fit answered (80, confidence 0.6, BRAINSTORMING_LOG_dashboard-window.md:46-50 + :77-86), none abstained; divergence: no findings submitted (the verifier rejected the code-review evidence refs as outside its packs). Low dimensions: security 30, driven only by `redaction_hits_observed` = 2 (weight 35), both hits in this review file (`tasks/TASK_REVIEW_HUN-278.md:72`, the template placeholder text of this row) — not in source code; test-strategy 61; requirement-fidelity 67; blast-radius 65. report: `/home/hungnguyenhuu/.easy-verifier-reports/easy-note-mcp-HUN-278/evidence-report-changes-20261119T235457-643234Z.html` |
| **UI: Visual regression (diff or verdict pasted)** | ☑ pass | LLM verdict against `docs/design/Main.dc.html` board "Dashboard (clickable)", read from screenshots (not committed). Method: headless Chrome (`google-chrome --headless --window-size=W,H --screenshot`) on a scratch copy of `assets/` with a stub `window.ipc` fed the real `window::api::handle` answers over 6 sample notes (2 today with AI detail, 1 yesterday stale, 1 yesterday question, 2 earlier without AI detail, one with a masked token and a `<script>` text); plus the real window: `easy-note dashboard` (window build, temp env, `GDK_BACKEND=x11`) captured with `import -window` → `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/dc887785-69f4-4baa-b7b7-cc4c1c76eca7/scratchpad/shots/real-window-x11.png`. Same layout, colours, fonts and labels: header mark + `easy-note` + notes dir chip; counts strip `notes today` / `notes in total` / `projects` / `by project` bars; search placeholder `Search notes — every word must match`; filters `All 6 · Bugs 1 · Ideas 1 · Questions 1 · To-dos 1 · No AI detail 2 · Stale 1`; groups `// today`, `// yesterday`, `// earlier this week`; rows time / first line / where-line / kind chip / `AI` or amber `stale`. Detail with AI detail (`/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/dc887785-69f4-4baa-b7b7-cc4c1c76eca7/scratchpad/shots/board-ai-1280.png`): `Your words`, context `easy-ui-mcp · fix/t120-firefox · T120 · today 05:40`, tags, dashed box `AI detail (not your words)` `claude-sonnet-4-5 · 20 Nov`, title, `type: bug`, summary, `suggested #firefox #flaky #ui_wait_for`. Stale (`/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/dc887785-69f4-4baa-b7b7-cc4c1c76eca7/scratchpad/shots/detail-stale-1280.png`): amber `stale: you edited the note after the AI wrote this`. Without AI detail + masked (`/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/dc887785-69f4-4baa-b7b7-cc4c1c76eca7/scratchpad/shots/detail-noai-masked-1280.png`): `No AI detail yet. In Claude Code, run the enrich_notes prompt; your own AI writes it in a separate file.`, amber `secret masked`, a note body holding a script tag with `alert(1)` shown as plain text, not run. Also `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/dc887785-69f4-4baa-b7b7-cc4c1c76eca7/scratchpad/shots/search-filter-1280.png` (`firefox flaky` + Bugs → 1 row), `nomatch-800.png`, `empty-800.png`, `error-1280.png`. Differences (all by the guide's out-of-scope list or Auto decisions 3–5): no status chips, no `+ New note`, no action buttons / `Add as tags`, `secret masked` without a count, an empty-dir text and an error text the design does not have. Note (Supervisor, 2026-11-20): `empty-800.png` and `error-1280.png` show the text from before Auto decisions 4–5; after the Debates only those two strings changed (`app.js` `no-results-text` and `showError`), layout unchanged. |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ pass | Method: every hex colour in `src/surfaces/window/assets/app.css` grepped in `docs/design/Main.dc.html`: `#0E100F #121413 #1F2321 #232826 #262B29 #2C3230 #3CF5A8 #6E7772 #7EDDB0 #8C9590 #ABB3AE #B5BCB7 #DCE0DD #E3B66B #E8968F #EAEDEA` → 16 of 16 `in design` (accent `#3CF5A8` with glow `#3CF5A859` / `#3CF5A814` = the design's default `accent` + `glow: true`; `#E8968F` is the design's trash-button red, used for the error line). Sizes and spacing grepped in both files: `26px/1.1`, `12.5px`, `11.5px`, `padding: 11px 14px`, `minmax(160px, 1fr)`, `max-width: 1440px`, `padding: 24px 28px`, `padding: 10px 28px` each present in both. Fonts: IBM Plex Sans 400/500/600 + Mono 400/500 woff2 bundled (`@font-face` → `fonts/*.woff2`, served as `font/woff2` by `asset()`), the design's weights exactly; the real-window capture `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/dc887785-69f4-4baa-b7b7-cc4c1c76eca7/scratchpad/shots/real-window-x11.png` shows Plex (not the system fallback) under the real CSP, so the fonts load from `easy-note://` with no network. |
| **UI: Responsiveness at target viewports** | ☑ pass | 1280 px (`/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/dc887785-69f4-4baa-b7b7-cc4c1c76eca7/scratchpad/shots/board-ai-1280.png`, `detail-stale-1280.png`, `search-filter-1280.png`): two columns, notes list left, detail panel right. 800 px = the window's minimum (`mod.rs:150` `with_min_inner_size(800, 560)`) (`/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/dc887785-69f4-4baa-b7b7-cc4c1c76eca7/scratchpad/shots/board-ai-800.png`, `detail-stale-800.png`, `detail-noai-masked-800.png`, `empty-800.png`, `nomatch-800.png`): the detail panel wraps below the list (the design's flex rule), no overlap; `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/dc887785-69f4-4baa-b7b7-cc4c1c76eca7/scratchpad/shots/overflow-check-800.png` taken with scrollbars on shows only a vertical scrollbar, no horizontal one; the notes dir chip ends in `…` and long project names in the bars end in `…` (`easy-note-m…`), as the design's `text-overflow: ellipsis`. 375 px not tested: the window cannot be that narrow (Auto decision 8). |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE** (captured 2026-11-19 by the Backend-Implementer before any implementation commit, worktree at `4b5c38a`, display present):

```
Thu Nov 19 11:05:54 AM UTC 2026
$ cargo build --locked --features window 2>&1 | tail -3
error: the package 'easy-note-mcp' does not contain this feature: window

Thu Nov 19 11:07:05 AM UTC 2026
$ EASY_NOTE_DIR=$(mktemp -d) HOME=<tmp> XDG_CONFIG_HOME=<tmp> ./target/debug/easy-note dashboard; echo exit=$?
easy-note: no dashboard yet: run easy-note install obsidian
exit=1
DISPLAY=:0 WAYLAND_DISPLAY=wayland-0
```

(`cargo run` with a temp HOME cannot find the rustup toolchain, so the second command runs the built binary.)

**AFTER** (captured 2026-11-20 by the Frontend-Implementer, worktree `feat/HUN-278-window` with the page slice, display present; the temp notes dir holds 6 sample notes):

```
Thu Nov 19 11:15:25 PM UTC 2026
$ cargo build --locked --features window 2>&1 | tail -3
   Compiling easy-note-mcp v0.1.0 (/home/hungnguyenhuu/workspace/pets/hungnguyen111/easy-note-mcp-HUN-278)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 34.07s

Fri Nov 20 06:14:00 AM +07 2026
$ easy-note dashboard   (window build; WAYLAND_DISPLAY=wayland-0 DISPLAY=:0 GDK_BACKEND=unset)
pid 973748 alive after 6 s: window open
exit=143 (closed by the test after the check)
stdout/stderr of easy-note:

Fri Nov 20 06:14:09 AM +07 2026
$ easy-note dashboard   (window build; WAYLAND_DISPLAY=wayland-0 DISPLAY=:0 GDK_BACKEND=x11)
pid 975515 alive after 6 s: window open
window id: 0x60003f
  Width: 1330
  Height: 907
captured /tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/dc887785-69f4-4baa-b7b7-cc4c1c76eca7/scratchpad/shots/real-window-x11.png
exit=143 (closed by the test after the check)
stdout/stderr of easy-note:
```

(Both runs: `HOME`, `XDG_CONFIG_HOME` and `EASY_NOTE_DIR` point into a scratch dir; the binary runs directly, as in BEFORE. Where BEFORE printed `easy-note: no dashboard yet: run easy-note install obsidian` and exited 1, the window opens and prints nothing; the capture shows the board with counts 2 / 6 / 3, the 7 filters and the 6 notes in 3 day groups.)

**DELTA**: In a `window` build on a desktop, `easy-note dashboard` (and so the launcher and the tray's "Open dashboard") now opens easy-note's own board window with the person's real notes, counts, search, kind filters, day groups and the AI detail panel, where before it needed Obsidian and failed without `install obsidian`.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-278.jsonl`, never the
implementing agent alone]
