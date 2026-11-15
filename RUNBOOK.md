# RUNBOOK

Operational notes for easy-note-mcp. This is a stub; HUN-264 completes it.

## Install (Linux, Rust stable)

From a checkout:

```bash
cargo install --path .
ln -s "$(command -v easy-note)" ~/.cargo/bin/n   # `n` is the same binary
easy-note --version && n --version
```

Remove it again:

```bash
rm ~/.cargo/bin/n
cargo uninstall easy-note-mcp
```

## Local checks

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cargo install --path . --root "$(mktemp -d)" && echo ok   # leaves PATH as it was
```

## Release steps (stub)

1. Bump `version` in `Cargo.toml` (the only place).
2. Run the local checks above. CI must be green on Linux (`ubuntu-latest`).
3. Tag `vX.Y.Z` on `main`.
4. Prebuilt binaries are not part of v1. Install is `cargo install --path .` from a clone.

Release automation is deliberately not built yet.

## Known clash: `n`

Another `n` may already be on PATH (for example the Node version manager `n`). `which n` shows which one wins.
`easy-note add` does the same job as `n`. The `doctor` command (later task) will report the clash.
