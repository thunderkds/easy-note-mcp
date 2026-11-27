# TASK_REVIEW — HUN-292

> Sibling of `tasks/TASK_GUIDE_T[NNN].md`. Everything here is **filled by the reviewer at Stage
> 4/5** — it is deliberately NOT in the guide, because the implementing agent re-reads the guide on
> every turn and never fills these two sections.
>
> Consumers resolve each section **guide first, this file second** (`.claude/hooks/lib/guide_sections.py`):
> a legacy guide that still carries these sections inline keeps working unchanged, and a stray
> review file can never override an inline section.

---


## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/overrides.rs` (12 tests, AC 1-8 and edge cases), `tests/leak.rs` (new surface `overrides`, all planted secrets); `src/core/ai.rs` unit tests unchanged |
| Verification command run | ☑ pass | `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` -> exit=0, 627 passed, 0 failed (summed over all `test result:` lines), run 2026-11-25 |
| Negative cases hold | ☑ pass | Implementer mutation run (first report): override and AI precedence swapped in `ai::effective`; 2 tests in `tests/overrides.rs` failed; reverted. Second reviewer tried to break the P2-1 fix with the real binary: a file with one good `project`, an over-cap `topic`, an unknown key and an orphan `ghost.kind=rant`; `ai set` and `ai undo` both exited 1 and named `<id>.<key>` without the value; md5 of `ai-overrides.json` and `ai-log.jsonl` identical before and after. |
| verify | ☑ pass | auto-run by hook, session e39a6137-f3bd-4b02-9428-f6b92e0d1d5d, commit 9df459047ad71b783743fb805660d3a98954f344, log .claude/verify-runs/HUN-292-9df459047ad7.log — pass; done not moved: Gate 5: no test run in memory/event-trace/HUN-292.jsonl |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☑ pass | Two fresh-context reviews (315b8b8, then 61624ce) over `main...61624ce`: `src/core/ai.rs`, `src/core/overrides.rs` (new), `src/core/ai_view.rs`, `src/surfaces/cli.rs`, `cli/read.rs`, `cli/write.rs`, `docs/NOTE_FORMAT.md`, `tests/overrides.rs` (new), `tests/leak.rs`, `tests/format.rs`. Callers of `ai::enrich` and `Detail` checked; `note_enrich` kind schema unchanged (`KINDS` stays 5). |
| Full smoke suite still green (no regression) | ☑ pass | `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` exit 0, 631 passed, 0 failed (implementer after the fixes); reviewer ran it again: every `test result` ok. |
| **Companion score (easy-verifier)** | ☑ N/A | Not run in this session: the easy-verifier tool is not available here. The merge gate blocks on a `fail` row only. |
| **UI: Visual regression (diff or verdict pasted)** | ☑ N/A | No UI in this task (core and CLI only). |
| **UI: Design-system compliance (tokens/colors/typography verified)** | ☑ N/A | No UI in this task. |
| **UI: Responsiveness at target viewports** | ☑ N/A | No UI in this task. |

---

## Demonstration

> Anchors what this task delivered to an observable before/after pair. BEFORE has no `N/A` path:
> if the task changes executable code, BEFORE is a pasted, timestamped terminal capture taken
> **before any implementation commit exists**; if it does not (docs, templates, skill-instruction
> text), BEFORE is the **verbatim prior content** of what changed — a quoted excerpt, not a command.

**BEFORE**: captured 2026-11-25T11:14:28+07:00 on main (74a290b), temp HOME/XDG_CONFIG_HOME/EASY_NOTE_DIR, `target/debug/easy-note add "retry bug on firefox"`, then `ai set <id> topic "firefox flaky"` and `ai undo <id>`:

```
$ easy-note ai set 20261125-111428-dd1e topic "firefox flaky"
error: unrecognized subcommand 'set'
Usage: easy-note ai <COMMAND>
exit=2
$ easy-note ai undo 20261125-111428-dd1e
error: unrecognized subcommand 'undo'
Usage: easy-note ai <COMMAND>
exit=2
```

**AFTER**: 2026-11-25T11:26:26+07:00, same sandbox method, branch feat/HUN-292-ai-overrides:

```
$ easy-note ai set <id> topic "firefox flaky"   -> ✓ topic of <id> set        exit=0
$ easy-note ai set <id> kind rant                -> easy-note: kind must be one of bug, idea, question, todo, draft, other   exit=1
$ easy-note ai undo <id>                         -> ✓ undone: topic of <id> is now no override   exit=0
$ easy-note ai undo <id>                         -> easy-note: nothing to undo   exit=1
ai-log.jsonl:
{"id":"…","field":"topic","old":null,"new":"firefox flaky","who":"person","at":"2026-11-25T11:26:26+07:00"}
{"id":"…","field":"topic","old":"firefox flaky","new":null,"who":"person","at":"2026-11-25T11:26:26+07:00","undo":true}
```

**DELTA**: A person can fix a note's project, topic or kind with `ai set`, the fix wins over the AI, every change is logged, and `ai undo` takes the latest one back.

**WITNESS**: [who ran it and when — derived from `memory/event-trace/Txxx.jsonl`, never the
implementing agent alone]

---

## Stage 4 review

Reviewer: fresh context, commit 315b8b8, `code-review` + `security-review` bounded to the diff. Gate run: `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` -> exit 0, every `test result` line ok, 0 failed.

### Acceptance Criteria trace

| # | Verdict | Where |
| -- | -- | -- |
| 1 Detail keys, caps, masking, `draft` kind | pass | `src/core/ai.rs` `Detail`, `check`, `mask_detail`, `ORGANIZER_KINDS`; tests/overrides.rs:105 |
| 2 overrides file, temp+rename, caps, masking, unknown id refused | pass (rename map write path: see P3-2) | `overrides.rs:138-149` (`save`), `:251-264` (`set`) |
| 3 `effective` + rename map | pass | `ai.rs` `effective` |
| 4 log line fields, append only, whole line | pass | `overrides.rs:170-180` |
| 5 `ai set`, exit 1 with allowed values | pass | `write.rs` `refused`, tests/overrides.rs:291 |
| 6 `ai undo`, `nothing to undo` exit 1 | pass | `overrides.rs:266-331`, tests/overrides.rs:312 |
| 7 override of trashed note kept, applies after restore | pass | `effective` ignores non-live id; test :347 |
| 8 note bytes identical | pass | tests/overrides.rs:367 |

### Findings

No P0, no P1.

- P2-1 `src/core/overrides.rs:80-103` + `:151-161` + `:259`: `parse` drops hand-edited entries that are over a cap or have a bad kind, and `set`/`undo` then write the filtered data back. One `ai set` therefore silently deletes the person's other hand-edited, invalid-looking entries (data loss, no warning). An invalid whole file is protected (refused), a single bad entry is not. Fix: keep the raw entry in the writeback, or refuse the write and name the entry.
- P2-2 `src/core/ai.rs` `effective` (calls `overrides::read` each time): the overrides file is read and parsed on every call, and an invalid file prints one stderr warning per call. A window or list that calls `effective` for N notes reads the file N times and prints N warnings. Not wrong today (no caller yet); fix before the window or `ai run` loops over notes: add a variant that takes an already-read `Overrides`, and warn once.
- P3-1 `src/core/overrides.rs:170-180` `append` ignores every error (`let _`). If the log write fails after the override was saved, the change has no log line and cannot be undone, with exit 0. NFR-015 says every AI change can be undone. Return or report the error (stderr line at least).
- P3-2 `overrides.rs` has no write function for the topic rename map `topics` (FR-050: "Renaming or merging a topic ... rewrites the override"). Out of this slice's vital slice only if a later task owns it; confirm the owner. Map keys are not masked on read (`:99-101` masks values only) and values are not cap-checked. Hand-edited only, low risk.
- P3-3 `overrides.rs:293-331` `undo` of an AI line when the AI file is gone: `ai::restore_field` returns `Ok` silently, then the undo is logged and the CLI prints "undone". Misleading. Also an AI `kind` change made by `enrich` (not logged by `enrich`) is overwritten by undoing an older logged AI kind change.
- P3-4 `ai.rs` `organize`: an empty or blank `Organized.project` / `topic` is accepted and stored as `Some("")`. `set` refuses empty; organize should too (treat as "no value").
- P3-5 `overrides.rs:317-325`: an undo of an AI change is logged `who: "person"`. Acceptable (a person ran it) but the log cannot tell it from a person's own `set` except by `undo: true`. Document in NOTE_FORMAT if the window will show it.

### Focus points

- Raw note bytes (NFR-015): no function in the diff opens a note path for write; `organize` and `restore_field` write only `.easy-note/ai/<id>.json` via `write`. Test :367 snapshots the whole tree. Pass.
- Masking and caps on every new string: `set` masks then caps; `organize`/`restore_field` mask, then `check`; log values masked then cut to 200 chars; `load` re-masks. Gap: rename map keys (P3-2). tests/leak.rs `surface_overrides` covers set/organize/undo/effective/CLI. Pass.
- Atomic whole-file write: `save` uses `create_temp` + fsync + `store.rename`, temp unlinked. Pass. Two writers race on read-modify-write (last wins), accepted in the guide edge cases.
- Log line size: ids come from `canonical_id` (bounded by `is_note_id` / inbox file name), values cut to 200 chars; worst case JSON escaping stays well under 4 KiB, below PIPE_BUF-style single-write atomicity of O_APPEND. Pass.
- Undo for the right note: `latest` filters `line.id == id`, undo lines pop the stack, non-undoable fields skipped, bad lines skipped. Pass.
- Invalid `ai-overrides.json` never overwritten: `load_for_write` refuses; `read` and `problems` do not write; test :389. Pass.
- Escaping the notes dir: `canonical_id` -> `resolve` -> `note_path` requires `is_note_id`; inbox ids come from the directory listing. The field name is checked against `FIELDS` before use. No user string builds a path. Pass.
- Security: no new process spawn, no network, new files live under `.easy-note/`. Secrets in values are masked before storage and before the log. Pass.

### Design point: FR-047 and `ai::organize` without an AI file

Yes, the API as built supports it, in two steps, with no change to the `enriched` count (NFR-012). `organize` refuses a note without a valid AI file because `Detail` needs `title`, `summary` and `kind`. For a note with no AI file the batch engine should call `ai::enrich` (title, kind, summary, tags, which FR-047 returns in the same answer item) and then `ai::organize` (project, topic, draft keys, kind `draft`). `enrich` already keeps the organizer keys on a re-run (stale notes). `enriched` counts only notes with a fresh valid AI file (`query.rs:191`), and FR-047 notes get that file anyway, so the count rule is unchanged: it counts AI files, nothing new feeds it. Caveats for the `ai run` task: (a) `enrich` accepts only the 5 `KINDS`, so a draft needs `enrich` with `other` and then `organize` with `draft`; the log then shows `other -> draft` as an AI change; (b) the two writes are not atomic, so a failure between them leaves an AI file with no `organized_at` (it stays pending; acceptable); (c) `enrich`'s own kind is not logged (P3-3). Do not add an organizer-only AI file: it would not parse and would raise the file count.

Verdict: pass with findings

## Fixes after review

- P2-1: `parse` now returns the dropped `<id>.<key>` names; `load_for_write` refuses when any entry was dropped (exit 1, names the file and the key, never the value). Reads still ignore the bad entry with one warning; `doctor` names each invalid entry. Test: `an_invalid_entry_blocks_set_and_undo_and_the_file_bytes_stay` (file bytes and log unchanged).
- P2-2: new `ai::effective_with(store, id, &Overrides)`; `effective` loads once and calls it. Test: `two_effective_with_calls_over_one_load_warn_once` (child process, stderr has one warning).
- P3-1: `set` and `undo` return an error `the change was saved, but ai-log.jsonl could not be written` (exit 1) when the log append fails. Test: `a_failed_log_append_after_a_saved_override_is_an_error`.
- Tests were added after the code in this round, not red first.
- Verification: `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` -> exit=0, 631 passed, 0 failed (summed over all `test result:` lines), 2026-11-25.

## Stage 4 review (second, 61624ce)

Reviewer: fresh context, `code-review` + `security-review` over `main...61624ce`, fix delta `315b8b8..61624ce` read line by line. Gate run: `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` -> exit 0, no warning or error, every `test result` line ok, 0 failed.

No P0, no P1, no P2.

### First-review P2s

- P2-1 (invalid entry silently dropped on write): fixed. `load_for_write` (`src/core/overrides.rs` ~151-170) refuses when `parse` returns any dropped entry. Break attempt by hand with the real binary: one note with a good `project`, plus `topic` of 70 chars, an unknown key `bogus`, and an orphan `ghost.kind=rant`. `ai set <id> kind idea` -> exit 1, `ai undo <id>` -> exit 1, md5 of `ai-overrides.json` and `ai-log.jsonl` identical before and after. The message names `<id>.<key>`, never the value. `doctor` lists each entry. Reads still use the good entry. Pass.
- P2-2 (re-parse and re-warn per call): fixed. `ai::effective_with(store, id, &Overrides)` (`src/core/ai.rs` ~378-392) does no I/O on the overrides; `effective` is `effective_with(.., &read(store))`. Test `two_effective_with_calls_over_one_load_warn_once` checks one warning for two calls. Use is correct: the in-tree caller passes a borrowed value loaded once; `canonical_id` still runs per call, as before. Pass.
- P3-1 (log failure ignored): fixed for `set` and `undo` (`log_line(..).map_err(log_failed)?`, exit 1, message says the change was saved). Pass; see P3-6.

### New findings

- P3-6 `overrides.rs` `set`/`undo`: when the log append fails, the override stays saved and the error is returned. The change is then live with no log line, so `ai undo` cannot take it back. The message is honest about it. Acceptable for this slice; a rollback (re-save the old data) would close NFR-015 fully. Optional.
- P3-7 `overrides.rs` `load_for_write` refusal message: `dropped` names are masked but not length-capped. A hand-edited key of megabytes is echoed to stderr whole, and in `doctor` lines. Cut each name to ~80 chars (`cut`) and cap the count.
- P3-8 `overrides.rs` `parse`: an orphan or trashed note's invalid entry (`ghost.kind`) blocks `set` for every note. Consistent with P2-1's rule and fixable by hand, but the person has no CLI way to remove an entry; the message says "fix or remove" without saying the file path. Consider naming the path.
- Still open from the first review, unchanged: P3-2 (rename map has no write path; map keys unmasked, values not cap-checked and not reported as invalid), P3-3, P3-4, P3-5. Not made worse by the fix.

### Security

No new input path, process spawn or network call. New stderr text carries only masked `<id>.<key>`. Files unchanged in location (`.easy-note/`). Notes bytes are not opened for write. Pass.

Verdict: pass with findings

## Auto decisions

Mode: auto
1. Fixed both review P2s on the branch (hand-edited entries dropped silently on write; `effective` re-parsing per call) — RULE: `tasks/TASK_GUIDE_HUN-292.md` Edge Case Checklist "Hand-edited, invalid `ai-overrides.json`: named by `doctor`, never overwritten silently"
2. Left P3-2 to P3-8 as follow-ups (no rename-map writer, undo of an AI change when the AI file is gone, empty project/topic in `organize`, undo logged as `person`, no rollback after a log failure, uncapped names in errors, one bad entry blocks every `set`) — RULE: `docs/claude-md/tracks.md` `### Every user stop and its auto rule` row 17 (P3 is reported, not a blocker); the rename-map writer belongs to the window-fixes slice (guide `## Requirement` of its task)
3. `ai::organize` refuses a note with no AI file; the batch engine must call `ai::enrich` first, then `ai::organize`, so the enriched count rule holds — FACT: `tasks/TASK_REVIEW_HUN-292.md` second-review design point "for a note with no AI file, the batch engine calls `ai::enrich` first (title, kind, summary, tags) and then `ai::organize`"; recorded for the batch-engine guide
4. The task was under-labelled: lane check printed `EXCEEDS (lines 1216 > 500)`, so C3 fits; no split now because the slice is merged-ready and reviewed twice — SAFER OPTION: re-labelling after review changes no code; the C3 label goes on the board with the merge ⚠ decided for you — please review
Checks: tests `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` — exit 0, 631 passed, 0 failed; reviews code-review and security-review by a fresh-context reviewer on 315b8b8 and again on 61624ce — pass with findings (P3 only after the fixes), code-review by the Supervisor over the diff — lane `C2, 11 files, 1216 lines, EXCEEDS`; real run `easy-note ai set` and `ai undo` on a temp notes dir by the second reviewer — exit 1 with a named entry on a bad file, bytes unchanged (auto-checked, not /verify)
Memory: nothing new on this branch; the HUN-292 decision entry is written on main after the merge
Not done: merge (admin: after the auto verify pass); the P3 follow-ups above; auto verify: ☑ pass (session e39a6137-f3bd-4b02-9428-f6b92e0d1d5d, commit 9df459047ad71b783743fb805660d3a98954f344, log .claude/verify-runs/HUN-292-9df459047ad7.log)

Code-review (Supervisor, `Skill({ skill: "code-review" })` over `main...61624ce`, bounded to the diff): no P0, P1 or P2 open. Lane check: `lane: C2, 11 files, 1216 lines, EXCEEDS (lines 1216 > 500); unpredicted: src/core/ai_view.rs, src/surfaces/cli/read.rs, src/surfaces/cli/write.rs, tests/format.rs` — C3 would fit. The unpredicted files are the test sample for the new fields, the `doctor` lines and `ai set`/`ai undo` wiring, and the pin test narrowed from the word "draft" to `**Status**: Draft`.
Closed: row moved to Done 2026-11-25 — auto verify pass (session 509e4a41-6be3-41a7-bd92-25f7589ec21c, commit 9b1593531257dec2bd65c6e78d9d88979dd6145b); Gates 5 and 6 checked
Paused: done not moved — Gate 5: no test run in memory/event-trace/HUN-292.jsonl
