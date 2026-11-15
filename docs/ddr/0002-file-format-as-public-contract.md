# 0002. Files are the store, and the file format is the public contract

**Status**: Accepted (owner, 2026-11-15)
**Date**: 2026-11-14
**Deciders**: thunderkds
**Related**: PRD FR-002, FR-015, FR-017, US-004, US-009 · GRILL D2, G12 · easy-personal DDR 0002, DDR 0003

---

## Context

Notes must be easy for the person to open, edit and sync, and easy-personal wants counts per repo and task.
easy-personal's DDR 0002 chose an MCP client of easy-context over reading easy-context's SQLite, because the
SQLite schema is private. easy-note has no private schema if the files themselves are the store.

DDR gate: hard to reverse (other tools and the person's own files depend on the format) ✅, surprising (a plug-in with no database) ✅, genuine trade-off ✅. 3 of 3.

---

## Decision

1. The notes dir is the **only** store. No database, no cache in v1.
2. One note = one file `<dir>/YYYY/MM/<id>.md` in format `easy-note: 1`, specified in `docs/NOTE_FORMAT.md` (final since HUN-253). The spec is versioned; a breaking change needs `easy-note: 2`, a new DDR, and continued reading of v1 files.
3. Unknown front matter keys (for example ones Obsidian adds) are kept unchanged on edit and ignored on read.
4. **Readers** (easy-personal follow-up F1, any script) may read the files read-only. A reader that must not hold note text (easy-personal NFR-002) reads the front matter only and never the body. easy-note does not know its readers and never depends on them.
5. `easy-note stats --json` gives the same counts for readers that prefer a command.

---

## Alternatives Considered

| Alternative | Pros | Cons | Why not chosen |
|-------------|------|------|----------------|
| Files as store, format public (chosen) | No lock-in; human-editable; readers work with easy-note not running | Scan cost; format frozen early | **Selected** |
| SQLite store + Markdown export | Fast queries | Two copies drift; export is extra work; hand edits lost | Files would stop being the truth |
| Files + easy-personal reads via MCP `note_stats` over HTTP | Same pattern as easy-personal DDR 0002 | easy-note runs on stdio, started per agent session; no service to call when the dashboard runs | Data only while some session runs |
| One daily file with line IDs | Journal feel | Line-ID syntax; edits and sync merges touch other notes | Per-note atomic writes are safer (GRILL D2) |

---

## Consequences

### Positive
- easy-personal shows note counts without a new NFR-002 exception (front matter holds no note text).
- Notes outlive the tool: plain Markdown.

### Negative (accepted trade-offs)
- Every query scans the folder; T010 decides if a rebuildable cache is ever needed (that would be a new DDR, cache only, files stay the truth).
- `title` is not in front matter (it is the body's first line), so front-matter-only readers cannot show titles; they link to the file instead.

### Follow-up
- [ ] T003 ships `docs/NOTE_FORMAT.md` final and a round-trip test (parse → render → byte-identical for files the tool wrote).
- [ ] F1 on easy-personal's board, with its own DDR.
