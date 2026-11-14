# 0004. Claude detail as separate AI files, written by the person's own Claude session

**Status**: Proposed (Supervisor, autonomous run; owner to accept)
**Date**: 2026-11-14
**Deciders**: thunderkds
**Related**: PRD US-016, FR-028–FR-032, NFR-007, NFR-012 · GRILL G34–G38, D32–D38 · `BRAINSTORMING_LOG_oneclick-enrich.md` Part 2 · amends DDR 0001 · precedent easy-personal FR-075 and its DDR 0003

---

## Context

One-line notes lose meaning after a few weeks ("retry bug on firefox" — which bug?). The owner asked whether Claude
should add detail at capture or scan plain notes later. Capture through Claude costs a model turn per note (round 2
removed exactly that, DDR 0003) and lets the model rewrite the person's words. The family prefers deterministic code
over AI judgement, and easy-note's NFR-007 says no LLM in the server.

DDR gate: surprising (a deterministic, model-free plug-in stores model text) ✅, genuine trade-off ✅, hard to reverse —
partly (AI files accumulate in a synced folder). 2 of 3 → DDR.

---

## Decision

1. Capture never calls a model. Detail is added **later**, only when the person runs the MCP prompt `enrich_notes` in their own Claude session (or their own schedule of it).
2. easy-note runs no model and makes no network call. The prompt text is fixed in code; the client's model does the work, using one subagent.
3. The model writes detail only through `note_enrich`, into `<notes dir>/.easy-note/ai/<id>.json`, with caps (title ≤ 120, summary ≤ 600, ≤ 5 suggested tags, `kind` from a fixed list), secrets masked, and server-set `based_on` hash and `at`.
4. Note files are never written by this path. A person accepts suggested tags with `easy-note tag <id> --accept-ai`.
5. Every read shows AI detail labelled "AI detail (not your words)" with model and date. Counts never use AI fields except `enriched`.
6. Where a model turn is already paid (Codex/Cursor `note:` → `note_add`), the optional `detail` argument is stored the same way with `model: capture`.

---

## Alternatives Considered

| Alternative | Pros | Cons | Why not chosen |
|-------------|------|------|----------------|
| Later pass, separate AI file (chosen) | Zero-cost capture; words kept; server deterministic | Two files per enriched note; detail only after a run | **Selected** |
| Enrich at capture via Claude | Detail at once; full context of the moment | Model turn per note; slow; derails the task; may rewrite words | Undoes DDR 0003's gain |
| Later pass writes into the note | One file | Person's file changes; sync conflicts; hard to tell AI from person | Breaks "your words are the record" |
| Model inside easy-note (API) | Fully automatic | Network + API key + cost; breaks local-first and NFR-007 | Family rules |
| No AI at all | Simplest | Old notes stay cryptic | Owner asked for auto detail |

---

## Consequences

### Positive
- Capture stays instant and free; notes get readable titles and context later.
- easy-note stays deterministic and model-free; AI text is easy to find, ignore or delete.

### Negative (accepted trade-offs)
- A second MCP write tool (`note_enrich`, AI files only).
- AI summaries may be wrong; they are labelled and never replace the note.
- Transcript-based detail works only within Claude Code's 30-day transcript life.

### Follow-up
- [ ] T014: tests that `note_enrich` leaves every note file byte-identical; caps and masking; stale detection after `edit`; `rm`/`restore` carry the AI file.
- [ ] T014: measure tokens per note on 20 real notes; record in PRD open question 7.
