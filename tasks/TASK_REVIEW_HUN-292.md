# TASK_REVIEW — HUN-292

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
Not done: merge (yours); <auto verify: running (T182) | auto verify: <result> | /verify (yours), with [admin] only after the one retry also ended blocked>

-->

## Evidence

| Check | Result | Notes / output snippet |
|-------|--------|------------------------|
| **New test(s) cover Acceptance Criteria (file paths pasted)** | ☑ pass | `tests/overrides.rs` (12 tests, AC 1-8 and edge cases), `tests/leak.rs` (new surface `overrides`, all planted secrets); `src/core/ai.rs` unit tests unchanged |
| Verification command run | ☑ pass | `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` -> exit=0, 627 passed, 0 failed (summed over all `test result:` lines), run 2026-11-25 |
| Negative cases hold | ☑ pass | Mutation: in `ai::effective` the override and AI value swapped (`git diff --stat -- src/core/ai.rs` non-empty) -> `tests/overrides.rs`: 10 passed, 2 FAILED (`effective_applies_the_override_...`, `undo_restores_...`); reverted. Pin test `tests/format.rs` note_format_doc... failed on the word `draft` in NOTE_FORMAT.md until narrowed to `**Status**: Draft`. |
| verify | ☐ pass / ☐ fail / ☐ N/A | [leave it: the auto verify hook writes this row, with its run record. If you ran /verify yourself, tick pass and make the whole Notes cell "/verify by the user, session <id>, commit <sha>" (the full 40-character commit). The merge gate counts no other row, even with the word "pass" in it] |
| Review scope bounded to the change's blast radius (affected set, not whole repo) | ☐ pass / ☐ fail | [what was reviewed vs. skipped, and why] |
| Full smoke suite still green (no regression) | ☐ pass / ☐ fail | |
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
