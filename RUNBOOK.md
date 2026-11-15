# RUNBOOK

Operational notes for easy-note-mcp. This is a stub; T009 completes it.

## Local checks

```bash
uv run pytest -q
uv tool install --force . && easy-note --version && n --version
uv tool uninstall easy-note-mcp   # leave PATH as it was
```

## Release steps (stub)

1. Bump `__version__` in `src/easy_note/__init__.py` (the only place; `pyproject.toml` reads it).
2. Run the local checks above. CI must be green on Python 3.11 and 3.12 (Linux, macOS).
3. Tag `vX.Y.Z` on `main`.
4. Publishing to PyPI is not part of v1. Install is `uv tool install .` from a clone.

Release automation is deliberately not built yet.

## Known clash: `n`

Another `n` may already be on PATH (for example the Node version manager). `which n` shows which one wins.
`easy-note add` does the same job as `n`. The `doctor` command (later task) will report the clash.
