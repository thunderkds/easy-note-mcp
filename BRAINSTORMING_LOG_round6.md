# BRAINSTORMING_LOG.md — round 6: the AI organizer
**Generated**: 2026-11-25
**Context**: Stage 0 step Direction. PRD gate passed (`GRILL_round6.md`, FR-045 to FR-053, NFR-014, NFR-015). Tier: **Standard**.

## The Problem Space

The person only takes notes. Easy-note must then arrange them without help: project, topic, kind, drafts, a digest per project and topic. Fixed by the grilling: the raw note never changes; AI output lives in AI files; the person's fixes win; a timer in the tray runs `easy-note ai run`; limits are interval, `max_per_run` and only new or stale notes.

The open design question is how `ai run` talks to the model. It decides cost, topic quality and how much code we write.

Verified in the repo:
- `ai enrich` already starts the person's `claude` with a JSON schema, one call per note (`src/surfaces/enrich.rs`: `schema()` line 53, `argv()` line 72, `run()` line 322).
- AI files are JSON through `core/ai.rs` (`Detail`, `enrich`, `pending`, `based_on`), with a stale test already (`pending`).
- The window talks to the core through `{"cmd": …}` in `src/surfaces/window/api.rs`; MCP has 7 tools in `src/surfaces/mcp.rs`.
- There is no scheduler, no overrides file, no log, no digest, and no window lock.

## The Alternatives

### Path A — Per note, with topic hints (extend what exists)
`ai run` loops over pending notes. Each call is today's enrich call with a bigger schema (project, topic, draft fields). The prompt carries the current topic and project names (at most 50) as hints, so the model reuses a name instead of inventing a near copy. The digest is a second job: one call per changed project or topic, once a day.
- Pros: reuses `enrich.rs`, the schema check and the masking path. One bad note fails alone. Easy to test with the existing fake `claude`.
- Cons: N calls per run (bounded by `max_per_run`). Topic names can still drift between runs.

### Path B — Batch, taxonomy first
One call takes up to 20 notes and returns the whole arrangement at once, with topics chosen together. Fewer calls, topics are consistent inside a batch.
- Pros: best topic quality. Cheapest in calls.
- Cons: one bad answer or timeout loses the batch. A bigger prompt means more note text per call. Needs new batch parsing and a partial-failure path. The enrich code does not carry over.

### Path C — Minimal: tags and sums, no model topics
Kind and draft come from the existing enrich call. Project is the repo, else the first `#tag`. Topic is the most common tag. The digest joins summaries with no model call.
- Pros: about half the code. No new model risk.
- Cons: not "the AI does the job". No topic for a note with no tags. Misses the owner's direction.

**50% rule for the recommended path:** drop the model call for the digest of a project with fewer than 3 notes (join the summaries instead), and put organize and drafts in one call. That removes about half the calls.

## Adversarial Review

| Path | Why it might fail |
|---|---|
| A | The model renames the same topic ("firefox-flaky" and "flaky-firefox") on different days; the person ends up fixing names by hand. Mitigation: hints in the prompt, the override file, and a topic merge in the window. |
| B | A 20-note prompt costs more tokens per call and one timeout drops 20 notes at once. A bad answer may move many notes into one wrong topic, and Undo has to work per note. |
| C | The person writes "buy milk" with no tag and no repo: no project, no topic. The result looks empty and the feature is not felt. |

## Edge Case Checklist (for every TASK_GUIDE)
- A note changes while `ai run` works on it: write nothing if `based_on` no longer matches (as HUN-288).
- Two `ai run` at once (tray plus cron): `ai-run.lock`; the second exits quietly.
- `claude` missing, logged out, or over quota: back off, one line in Settings, never a crash loop.
- The AI returns a project or topic with a secret or a path: mask and cap before the write.
- A person's override names a topic that no note uses any more: keep it, hide it in the list.
- Topic merge while a run is in progress: the override is written first; the run reads it at the end.
- Clock set back or a note dated in the future: the stale test uses the text hash, never time.
- A sync tool copies `ai-overrides.json` from two machines: write whole file by temp plus rename; the log is append-only.
- 5,000 notes: `ai run` still takes at most `max_per_run` notes; the window groups by topic in under 1 s (NFR-013).
- Raw note bytes are identical after any run (NFR-015, a fixture test).

## Surgical Scope
- **Touch:** `src/core/ai.rs` (new fields), `src/core/config.rs` (`[ai]` keys), new `src/core/overrides.rs`, `src/surfaces/enrich.rs` (schema, hints, `run`), `cli.rs` (`ai run`, `ai set`, `ai undo`), `tray.rs` (timer), `window/*` (lock, views, settings, keys), `mcp.rs` (`note_context`), `docs/NOTE_FORMAT.md` (AI file keys), `PRD.md`.
- **Must not touch:** note files and their writer (`store.rs` `update` path), the secret guard patterns, the hook, `install claude`.

## Owner pick (2026-11-25): Path B — batch, taxonomy first
The owner chose B over the recommended A. Consequences taken into the PRD (FR-047, FR-048):
- One call per batch of `batch_size` notes (default 10, `max_per_run` 20 = at most 2 calls per run). A smaller batch than the 20 first discussed limits what one timeout can lose.
- The prompt carries the project and topic lists, so the model picks topics for the whole batch together and reuses names.
- The answer is an array; the server validates each item alone. A bad item stays pending; a failed call leaves the batch pending and counts toward back-off.
- Prompt size: each note cut to 1,500 chars, 12,000 chars per call, so a long note cannot crowd out the others.
- Undo stays per note: one log line per applied item.
- Not carried over from `enrich.rs`: the batch prompt, the array schema and the per-item check. The child process, env, timeout and masking code is reused.
Extra edge cases for B: the model returns an id that was not in the batch (drop it); two ids merged into one item (drop it); the array is shorter than the batch (rest stays pending); the answer is valid JSON but the schema check fails (batch stays pending, one log line, no partial write).

## Build order (Next Actions for Stage 2)
1. **Wave 1, no dependencies, parallel:** window lock (FR-045); show `updated` (FR-046).
2. **Wave 2, foundation:** AI file keys, overrides, log and undo (core); `[ai]` config keys.
3. **Wave 3:** batch `ai run` and digest (FR-047, FR-051 job); tray timer (FR-049); `note_context` tool.
4. **Wave 4:** window views, drafts, topic rename and merge, Settings screen (FR-052).
5. **Wave 5:** UX pass with `ux-evaluate` (FR-053).
Write one DDR at Stage 2: "AI organizer writes beside notes, batches its calls and runs unattended after one Settings opt-in".
