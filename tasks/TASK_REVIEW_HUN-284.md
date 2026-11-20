# TASK_REVIEW — HUN-284

> Sibling of `tasks/TASK_GUIDE_HUN-284.md`. Everything here is **filled by the reviewer at Stage
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
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/window_api.rs`: AC 1 `the_csp_allows_only_the_app_scheme` (pin changed: `CSP == "default-src easy-note:"`, no `unsafe`, equals the `<meta>` policy in `index.html`) + `every_served_asset_is_scanned_and_safe` (new inline-style scan: `style=` attribute, `<style`, `setAttribute('style'`, `.cssText`, `.style =`, inline `<script>`; a CSSOM property write such as `fill.style.width` is not blocked by CSP and stays); AC 3 `navigation_stays_on_the_app_scheme` (pin extended: `easy-note://other/`, `easy-note://localhost.evil/`, `…localhost.evil`, `…localhost:8080/`, `…localhost@evil/`, `easy-note://evil/localhost/`, `easy-note:///index.html`, `easy-note://`, `https://x/` refused); AC 4 `ipc_is_answered_only_from_the_app_origin` (new: `ipc_from` answers `easy-note://localhost/…`, returns `None` for 5 other sources, and a refused `trash` leaves the notes dir byte-identical); AC 5 `a_row_click_brings_the_detail_into_view_only_when_it_wraps` (new: row click is `select(n.id, true)`, `select` calls `revealDetail()` after both the detail and the error render, `revealDetail` has the `offsetTop` wrap guard, `scrollIntoView(`, `block: 'start'`, `(prefers-reduced-motion: reduce)`). Window build: `shell::tests::every_response_carries_the_csp` (unchanged, now asserts the new string). Run: `test result: ok. 25 passed; 0 failed` for `tests/window_api.rs` in default and `--features window`. |
| Verification command run | ☑ pass | Guide command (fmt --check, clippy default -D warnings, test, clippy `--features window` -D warnings, test `--features window`), log `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun284/verify2.log`: started `2026-11-20T03:53:08Z`, ended `EXIT=0` `2026-11-20T04:02:42Z`; 48 `test result: ok` lines; `grep -cE 'FAILED|^warning|^error|No space left|signal 7'` = 0; `Running tests/window_api.rs` → `test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s` (default) and `… finished in 0.18s` (window). Not hit by the disk-full window: 0 disk errors in this log. |
| Negative cases hold | ☑ pass | Red run against the prior code (see Demonstration BEFORE, 2026-11-20T03:24:39Z): 4 FAILED — CSP left `"default-src easy-note:; style-src easy-note: 'unsafe-inline'"`; `allowed_url("easy-note://other/")` true; `ipc_from` with source `easy-note://localhost.evil/` returned `Some(<board>)`; row click without `revealDetail`. Second red during the change: the first AC 5 version scrolled before the detail rendered, so at 800 px only the panel head showed (real window); the test was tightened to require the reveal after render, failed (`panicked at tests/window_api.rs:701:5`), then passed after `select(id, reveal)`. |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☐ pass / ☐ fail | [what was reviewed vs. skipped, and why] |
| Full smoke suite still green (no regression) | ☑ pass | The whole `cargo test --locked` and `cargo test --locked --features window` in `verify2.log`: 48 `test result: ok`, 0 FAILED. |
| **Companion score (easy-verifier)** | ☐ pass / ☐ fail / ☐ N/A | [`overall` and `change_risk` as the tool returned them, the scope, the ref, and the absolute path of the verifier report after a literal `report:` label — e.g. "overall 86 (>= 80), change_risk pass, scope changes, ref `main...HEAD`, report: `/home/me/.easy-verifier-reports/kit/score_changes_20261028T0530Z.html`". The gate reads the path that follows `report:` and nothing else, so with no label, or a path that is absent, the row is unscored — write ☑ N/A instead. ☑ N/A with a reason when neither companion tool was run: the merge gate blocks on ☑ fail only, never on an absent or N/A row] |
| **UI: Visual regression (diff or verdict pasted)** | ☑ pass | Real window under the new CSP (window build, private Xvfb `:184`, `GDK_BACKEND=x11`, temp HOME / XDG_CONFIG_HOME / EASY_NOTE_DIR, 11 sample notes): `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun284/shots/after-1280-click3.png` vs the HUN-279 board: same header + notes-dir chip, counts strip, `by project` bars at different widths (the CSSOM `fill.style.width` still applies: 100 % / 50 % / 50 %), search, filters `All 11 · Bugs 1 · Ideas 1 · Questions 1 · To-dos 0 · No AI detail 8 · Stale 0`, day groups, rows, detail card with `Open in editor` / `Copy id` / `Move to trash`. `after-1280-toast.png`: `Copied` toast with its glow border. `after-1280-filter.png`: `Ideas` pressed, 1 row. `after-800-q-4.png`: AI detail box with the sparkle SVG icon, `Add as tags`. The board loading at all proves the IPC origin check accepts the real page URL. LLM verdict: no visible change. |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ pass | No style moved: the only inline style write in the page was `fill.style.width` (CSSOM, allowed by CSP without `'unsafe-inline'`) and it stays; `git diff --quiet -- src/surfaces/window/assets/app.css src/surfaces/window/assets/index.html` → unchanged; `grep -c 'style="' index.html` → 0. The page's own `<meta>` CSP was already `default-src easy-note:`, so the page already ran without inline styles. Bar values match `docs/design/Main.dc.html:43` (`width: {{r.pct}}%`, `#7EDDB0`). |
| **UI: Responsiveness at target viewports** | ☑ pass | 800 px (window resized with `XMoveResizeWindow`, the minimum width): click 3rd row → `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun284/shots/after-800-click3-later.png` the page scrolled, the whole detail card (id, `Your words`, context, tags, no-AI box, actions) is in view; click the `question` row → `after-800-q-4.png` whole card with AI detail in view (the page end limits the scroll, so the panel sits at the bottom with its top visible, not at the top edge). No horizontal scrollbar in any 800 px shot. 1280 px: `after-1280-click3.png` two columns, no scroll jump (header still at top after the click). Note: in this debug build the scroll comes ~2–3 s after the click, because it waits for the `detail` IPC answer (`after-800-click3.png` at +2 s still unscrolled, `after-800-q-1..4.png` changes at +3 s); load time is HUN-282. |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured by the frontend agent on branch `feat/HUN-284-harden` at `f2e77d9` (no implementation commit yet; only the new tests and a no-op `ipc_from` stub that keeps today's behaviour — the shell answers every IPC message — were in the working tree).

```
$ date -u +%FT%TZ; grep -n "pub const CSP\|fn allowed_url" -A2 src/surfaces/window/mod.rs; grep -n "Content-Security-Policy" src/surfaces/window/assets/index.html; grep -n "select(n.id)\|scrollIntoView" src/surfaces/window/assets/app.js; grep -n "with_ipc_handler" -A3 src/surfaces/window/mod.rs
2026-11-20T03:24:30Z
14:pub const CSP: &str = "default-src easy-note:; style-src easy-note: 'unsafe-inline'";
57:pub fn allowed_url(url: &str) -> bool {
58-    url.starts_with("easy-note://")
59-}
5:<meta http-equiv="Content-Security-Policy" content="default-src easy-note:">
156:    b.addEventListener('click', function () { select(n.id); });
160:            .with_ipc_handler(move |request| {
161-                // The answer goes back through the event loop, where the webview lives.
162-                let _ = proxy.send_event(ipc(&store, request.body()));
163-            })

$ date -u +%FT%TZ; cargo test --locked --test window_api   (new HUN-284 tests, red)
2026-11-20T03:24:39Z
test a_row_click_brings_the_detail_into_view_only_when_it_wraps ... FAILED
test navigation_stays_on_the_app_scheme ... FAILED
test the_csp_allows_only_the_app_scheme ... FAILED
test ipc_is_answered_only_from_the_app_origin ... FAILED
thread 'the_csp_allows_only_the_app_scheme' panicked at tests/window_api.rs:479:5:
  left: "default-src easy-note:; style-src easy-note: 'unsafe-inline'"
 right: "default-src easy-note:"
thread 'navigation_stays_on_the_app_scheme' panicked at tests/window_api.rs:536:9:
easy-note://other/                       <- allowed_url("easy-note://other/") is true today
thread 'ipc_is_answered_only_from_the_app_origin' panicked at tests/window_api.rs:684:9:
assertion `left == right` failed: easy-note://localhost.evil/
  left: Some("{\"notes_dir\":...,\"total\":4,...,\"req\":1}")   <- a message from easy-note://localhost.evil/ gets the board
 right: None
thread 'a_row_click_brings_the_detail_into_view_only_when_it_wraps' panicked at tests/window_api.rs:696:5:
the row click does not call revealDetail
test result: FAILED. 21 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

Real window (pre-change `cargo build --locked --features window` binary, private Xvfb `:184`, `GDK_BACKEND=x11`, temp `HOME` / `XDG_CONFIG_HOME` / `EASY_NOTE_DIR` with 11 sample notes, window resized to 800×820 with `XMoveResizeWindow`, click by XTest at (400, 487) = the third row `write the release notes for 0.2 #docs`, 2026-11-20T03:24:14Z): `<scratchpad>/hun284/shots/before-800-click3.png` — the third row is highlighted (`aria-current`), the page did not scroll: the view still shows the header, counts and the `// today` rows; the detail panel (below the list, at the end of the page) is not in view. Scratchpad = `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad`.

**AFTER**: frontend agent, 2026-11-20, working tree on top of `70b94f4`.

```
$ grep -n "pub const CSP\|pub const ORIGIN\|fn allowed_url\|fn ipc_from" -A3 src/surfaces/window/mod.rs
15:pub const CSP: &str = "default-src easy-note:";
17:pub const ORIGIN: &str = "easy-note://localhost";
61:pub fn allowed_url(url: &str) -> bool {
62:    url.strip_prefix(ORIGIN)
63:        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
100:pub fn ipc_from(store: &Store, source: &str, body: &str) -> Option<String> {
101:    allowed_url(source).then(|| ipc(store, body))
shell: let source = request.uri().to_string();
       if let Some(answer) = ipc_from(&store, &source, request.body()) { let _ = proxy.send_event(answer); }

$ cargo test --locked --test window_api   (verify2.log, default and --features window)
test a_row_click_brings_the_detail_into_view_only_when_it_wraps ... ok
test ipc_is_answered_only_from_the_app_origin ... ok
test navigation_stays_on_the_app_scheme ... ok
test the_csp_allows_only_the_app_scheme ... ok
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Real window, same setup as BEFORE (new binary, 800×820, click (400, 487) on the third row at 2026-11-20T03:51:11Z): `/tmp/claude-1000/-home-hungnguyenhuu-workspace-pets-hungnguyen111-easy-note-mcp/8458b2db-536a-4fdc-a075-2bb8f587ea77/scratchpad/hun284/shots/after-800-click3-later.png` — the page scrolled down and the detail card of `write the release notes for 0.2 #docs` is fully in view (id `20261120-102349-7c4d`, `Your words`, `no project · today 10:23`, `#docs`, actions). 1280×820 (2026-11-20T03:52:27Z): `after-1280-click3.png` — two columns, detail beside the list, no scroll.

**DELTA**: The page now runs under `default-src easy-note:` with no `'unsafe-inline'`, the window navigates to and answers IPC only from the exact origin `easy-note://localhost` (`easy-note://localhost.evil/` and other hosts are refused), and at 800 px a click on a note brings its detail into view without scrolling by hand.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/HUN-284.jsonl`, never the
implementing agent alone]
