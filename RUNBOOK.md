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

### Debian package

`scripts/build_deb.sh` builds `dist/easy-note_<version>_<arch>.deb` from the checkout. It needs
`dpkg-dev` and the WebKitGTK headers (`sudo apt install dpkg-dev libwebkit2gtk-4.1-dev`).

```bash
scripts/build_deb.sh                                  # default features: window,tray
sudo apt install ./dist/easy-note_0.1.0_amd64.deb     # apt resolves Depends
easy-note --version
```

The package holds `/usr/bin/easy-note` and `/usr/share/doc/easy-note/`, and nothing else: the
icons, the window page and its fonts are embedded in the binary. It ships no maintainer script, so
it writes nothing below any home folder. Each person runs `easy-note setup` themselves. It gives
no `n` symlink. `sudo apt remove easy-note` removes it; the notes dir stays.

`dpkg-shlibdeps` reads `Depends` from the built binary, so a build without `--features window`
drops the WebKitGTK dependency. `tests/deb.rs` covers the packaging with `/bin/true` in place of
the binary, so `cargo test` checks it without a release build.

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

## Rollback of `easy-note setup` (HUN-277)

`install obsidian` changes one file outside easy-note: Obsidian's vault list. To undo it, close Obsidian, then:

1. `cp ~/.config/obsidian/obsidian.json.easy-note-<YYYYMMDD-HHMMSS>.bak ~/.config/obsidian/obsidian.json` (the newest backup).
2. Optional: `rm -r <notes dir>/.obsidian` (it holds only Obsidian's own settings for this vault).

## Release Log

| Version | Date | Scope | Deployer | Outcome | Approved (by / when / answer) |
|---|---|---|---|---|---|
| (no tag) `0885cb5` on `main` | 2026-11-19 | HUN-251–HUN-277 (no release tagged yet; round 4 + `easy-note setup`) | — | planned only, not released: production approval not asked (auto mode) | — |
