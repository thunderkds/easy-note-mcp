# RUNBOOK

Operational notes for easy-note-mcp: install, checks, release and rollback. Linux only (DDR 0005).

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

## Release steps

1. On a clean `main` (`git status --short` prints nothing), bump `version` in `Cargo.toml` (the only
   place), then `cargo build` so `Cargo.lock` follows. Commit both: `release: vX.Y.Z`.
2. Run the local checks above. CI must be green on Linux (`ubuntu-latest`).
3. Tag and push: `git tag -a vX.Y.Z -m vX.Y.Z && git push origin main vX.Y.Z`.
4. Install check, from a fresh clone of the tag:
   `cargo install --path . --locked --root "$(mktemp -d)"`, then run that binary's
   `easy-note --version` (prints `easy-note X.Y.Z`) and `EASY_NOTE_DIR="$(mktemp -d)" <root>/bin/easy-note add release check`
   (prints `✓ noted …`).
5. Install for real: `cargo install --path . --locked` from the tag.

Prebuilt binaries and release automation are not part of v1.

## Rollback

1. `git checkout v<previous>` in the clone.
2. `cargo install --path . --locked --force`.
3. `easy-note --version` prints the previous version; `easy-note doctor` exits 0.

No step touches the notes dir: the note format (`easy-note: 1`, DDR 0002) does not change between
releases, so an older binary reads every note a newer one wrote.

## Known clash: `n`

Another `n` may already be on PATH (for example the Node version manager `n`). `which n` shows which one wins.
`easy-note add` does the same job as `n`. `easy-note doctor` reports the clash.
