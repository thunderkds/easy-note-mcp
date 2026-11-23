# TASK_REVIEW — HUN-288: Run AI button in the dashboard window's detail panel

> Sibling of `tasks/TASK_GUIDE_HUN-288.md`. Stage 4 on branch `feat/HUN-288-run-ai`, HEAD at the time of the
> score: `4449c5c` (+ docs commit `6a5fd8f`). Review by a fresh-context reviewer (`Reviewer: HUN-288`) on `0fac452`,
> fixes in `48ab6b2` and `4449c5c`, runtime check by the Supervisor under Xvfb (below).

---

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/window_api.rs` (enrich success with no config, stale-then-fresh, 6 error cases, trashed/bad/missing ids, masking, slow-claude seam, wrong origin, edit-during-run refusal), page scenarios in `tests/fixtures/window/page_harness.js` |
| Verification command run | ☑ pass | implementer: exit 0, 1222 passed / 0 failed / 12 ignored (default + `--features window,tray`) at `48ab6b2`; rerun exit 0 after `4449c5c` (label change) |
| Negative cases hold | ☑ pass | each error leaves no AI file and starts no child where claude must not start; a note edited during the run is refused and nothing is written (test failed before the fix, passes after) |
| verify | ☐ pass / ☐ fail / ☐ N/A | left for the auto verify hook (the Supervisor's own Xvfb run is in "Runtime check" below; it is not the hook's verify) |
| Review scope bounded to the change's blast radius | ☑ pass | `enrich.rs`, `window/api.rs`, `window/mod.rs`, `assets/{app.js,ipc.js,app.css}`, `tests/window_api.rs`, harness |
| Full smoke suite still green (no regression) | ☑ pass | both builds, 0 failed |
| **Companion score (easy-verifier)** | ☑ fail | overall 76 (floor missed: 76 < 80), change_risk 100 pass (bugfix, no failed rule), scope changes, ref `main...HEAD`, abstentions: solution-fit (no_static_rule), reliability (insufficient_measured_weight), api, data, operations, infrastructure (below_coverage_floor), gates: solution-fit asked once, left abstained (listed refs are brainstorm-log lines, no held evidence stands behind them), ratings: architecture 100, supply-chain 100, test-strategy 72, blast-radius 65, code-quality 67, requirement-fidelity 67, security 59 (sink_hits 2 failed: child-process spawn, which is the feature; redaction hits 0), no findings sent, report: `/home/hungnguyenhuu/.easy-verifier-reports/easy-note-mcp-HUN-288/evidence-report-changes-20261123T105437-660895Z.html` |
| **UI: Visual regression** | ☑ pass | headless Chrome stub screenshots by the implementer, plus real-window screenshots under Xvfb by the Supervisor (idle, running, done, error toast) |
| **UI: Design-system compliance** | ☑ pass | `.act:disabled` reuses token `#8C9590`; no new colours |
| **UI: Responsiveness at target viewports** | ☑ pass (one note) | 1280 px: Edit, Re-run, Copy id, Move to trash on one row; 800 px one row. While a run is in progress on a note that already has AI detail the `AI running…` button is wider and `Move to trash` wraps at 1280 px until the answer (owner may shorten it to `Running…`) |

---

## Code review (fresh-context reviewer, `0fac452`)

No P0, no P1. Security verdict: pass with notes.

| # | Sev | Finding | Status |
|---|-----|---------|--------|
| 1 | P2 | AI detail stamped fresh when the note is edited during a run (`enrich_one`) | fixed in `48ab6b2`, test added |
| 2 | P2 | README said AI is off unless `[ai] enrich` is set | fixed in `48ab6b2` |
| 3 | P3 | Empty-panel text next to the new button | fixed in `48ab6b2` |
| 4 | P3 | No panic guard in the AI thread (button stuck at "AI running…") | fixed in `48ab6b2` (no test: needs a new seam) |
| 5 | P3 | Keyboard focus lost when the panel re-renders | not fixed (accepted) |
| 6 | UX | `Run AI again` wrapped `Move to trash` at 1280 px | fixed in `4449c5c`: label is `Re-run` |

Supervisor-owned edits made: DDR 0007 item 6 (the click is the opt-in), PRD FR-043 `Run AI` wording, guide label.
Follow-ups (not blocking): `doctor` still prints `ai: off` while the window can run AI; the in-flight `claude`
child is not killed when the window closes (ends on its own within `timeout_secs`).

## Security review

Origin check before any parse or child; one run per note (`Running`, slot freed on panic); same argv, env
(no `ANTHROPIC_*` auth variables) and stdin as `ai enrich`; AI file written only by the parent; every reply and
error text passes `mask_all`; ids validated through `Store::read`; page uses `textContent` only. Verdict: pass
with notes (above).

## Runtime check (`/verify`, Supervisor, Xvfb, stub `claude`)

Release build `--features window,tray`, temp HOME and notes dir, stub `claude` on PATH, no config file:

1. Select a note, click `Run AI` → button becomes disabled `AI running…`; window stays responsive.
2. After the stub answers → AI detail shown (title, summary, `#hardware`, `#shopping`, `Add as tags`), label `Re-run`,
   list row gets the `to-do` chip and `AI` mark, filter counts update (To-dos 1, No AI detail 1). AI file written under
   `.easy-note/ai/`; the note file is unchanged.
3. Stub exits 1 ("not logged in") → toast `Run AI failed: claude exited 1; if it is not logged in, log in once: claude`;
   no AI file; button back to `Run AI`.

Not driven: the edit-during-run race at the surface (covered by a Rust test), a real `claude`, Wayland.

## Verdict

Code review and security review pass with no open P0/P1. The Companion score is a `fail` on the floor (76 < 80),
driven by security 59 (the child-process spawn is the feature itself), blast-radius 65, code-quality 67 and
requirement-fidelity 67. Merge needs the owner's hand merge over the failing row, as for the earlier tasks.
Owner-only before or after merge: one `Run AI` with the real `claude`.

## Demonstration

BEFORE: not captured before the first implementation commit (the spawn warning for a blank BEFORE was raised and not
acted on); at `main`, the detail panel has no AI action in the window: AI detail could only be created from the CLI
(`easy-note ai enrich <id>`) or the `enrich_notes` prompt.
AFTER: see "Runtime check" above.

## Auto decisions

Mode: auto

1. Q1 (does `Run AI` run with no `[ai]` section): yes, the click is the opt-in; `after_save` stays off. RULE: Supervisor recommendation, owner said "go with your recommendations" and "spawn parallel" (chat 2026-11-23, `[admin]`); recorded in DDR 0007 item 6.
2. Button label with AI detail: `Re-run` (not `Run AI again` or `Re-run AI`). RULE: owner chose a shorter label; `Re-run AI` (88 px) and `Redo AI` still wrapped `Move to trash` at 1280 px, `Re-run` (71 px) fits one row.
3. Fresh-context reviewer on `0fac452`: P2 race fixed in the code (re-read the note after the child ends), README, empty-panel text and panic reply fixed; focus-after-render left. RULE: Stage 4, every P0/P1/P2 with an unambiguous fix is fixed; P3 focus left as accepted.

Checks:
- code-review: fresh-context reviewer, 0 P0, 0 P1, findings above (auto-checked, not /verify).
- security-review: Skill call made; the skill's own diff covered the main checkout, so the HUN-288 source diff (`enrich.rs`, `window/api.rs`, `window/mod.rs`) was read directly: origin check before any parse or child, one run per note with the slot freed on panic, `for_window` reuses `enrich_one` (same argv, env without `ANTHROPIC_*` auth variables, stdin), ids go through `Store::read`, replies pass `mask_all`; no finding at confidence 8 or more (auto-checked, not /verify).
- Tests: guide Verification Command exit 0, 1222 passed, 0 failed, 12 ignored, both builds (auto-checked, not /verify).
- Real run: Xvfb, release `--features window,tray`, stub `claude`: Run AI click, running state, AI detail and `Re-run`, failure toast, no AI file on failure (auto-checked, not /verify).
- Companion score: overall 76, floor missed (see the Evidence row).

Memory: wrote `memory/learnings.md` entries (hand-filled `verify` row blocks the hook's auto verify; easy-verifier `score` rejects findings whose refs are outside its packs and returns 100 KB payloads to read with jq; Xvfb plus XTEST recipe to drive the window under test).

Not done: merge (admin: after the auto verify pass), the delivery report, owner-only check (one `Run AI` with the real `claude`), `doctor` still prints `ai: off` while the window can run AI (follow-up), the in-flight `claude` child is not killed when the window closes (follow-up). auto verify: running (T182), commit fa16162.
