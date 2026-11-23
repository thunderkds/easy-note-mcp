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
| v0.2.0 @ `8057f5d` on `main` (tag not created yet) | 2026-11-23 | HUN-277–HUN-289 (own dashboard window with Run AI, tray, `.deb`, styled quick window) | operator | planned, GO recommended; the operator runs the steps below | thunderkds / 2026-11-23 / "Approve v0.2.0 @ 8057f5d" (AskUserQuestion answer, copied verbatim) |

## Release v0.2.0 plan (operator runs it)

Install target: the owner's machine, Linux, `cargo install` or the `.deb` (PROJECT_SPEC Deployment target). No server, no migration: the note format (`easy-note: 1`) does not change.

**Smoke test and pass condition (fixed before the steps):** with a temp notes dir (`EASY_NOTE_DIR="$(mktemp -d)"`), the installed binary prints `easy-note 0.2.0`; `easy-note add release check` prints `✓ noted …`; `easy-note doctor` exits 0; `easy-note dashboard` opens the window on a desktop session and shows the note; `easy-note quick` opens the small capture window, and Enter saves a note with `source: quick`. Any of these failing is a NO-GO for the install.

1. On a clean `main` at `8057f5d` (`git status --short` empty in a fresh clone), set `version = "0.2.0"` in `Cargo.toml`, run `cargo build` so `Cargo.lock` follows, commit `release: v0.2.0`.
2. Run the local checks above and the verification command with `--features window,tray`; CI (`.github/workflows/ci.yml`) must be green on the release commit.
3. `git tag -a v0.2.0 -m v0.2.0 && git push origin main v0.2.0`.
4. Install check from a fresh clone of the tag: `cargo install --path . --locked --root "$(mktemp -d)" --features window,tray`, then the smoke test above against that binary.
5. For real: `cargo install --path . --locked --features window,tray` or `scripts/build_deb.sh && sudo apt install ./dist/easy-note_0.2.0_amd64.deb`; then `easy-note setup` if the launcher and tray are wanted.

**Health check:** `easy-note --version` prints `easy-note 0.2.0` and `easy-note doctor` exits 0 (it prints `ai: off` while the window can still run AI: known follow-up).

**Rollback triggers:** the smoke test fails, `doctor` exits non-zero, or the window or `quick` does not open on the owner's display. **Reverse:** the Rollback section above (`git checkout v<previous>`, `cargo install --path . --locked --force`); for the `.deb`, `sudo apt remove easy-note`. No tag exists before v0.2.0, so the previous good state is commit `0885cb5` (the last runbook row). The notes dir is untouched by any step.

**Owner-only checks before relying on it:** one `Run AI` with the real `claude`; the 20-run `quick` timing; tray New note and `Ctrl+Alt+N`; a planted secret typed into the window; GNOME Wayland focus.

**Release notes (v0.2.0)**
- Features: easy-note's own dashboard window (HUN-278), with the note board, actions (HUN-279), `+ New note` capture dialog (HUN-280), edit mode (HUN-286) and a `Run AI` button for one note (HUN-288); a tray with New note and Open dashboard (HUN-285); `easy-note quick` in its own styled window, also for the tray and `Ctrl+Alt+N` (HUN-289); a `.deb` package build (HUN-287).
- Fixes: an unusable display falls back to the Obsidian dashboard instead of a panic (HUN-281); the window starts with one board request, debounced search and a lighter board payload (HUN-282); strict CSP and an exact `easy-note://localhost` origin for navigation and IPC (HUN-284).
- Internal: stale docs cleaned after the window decision (HUN-283); `easy-note setup` and the Obsidian vault registration (HUN-277). Known limits: `doctor` prints `ai: off` while the window can run AI; the in-flight `claude` child is not killed when the window closes; the dashboard and `quick` windows are untested on GNOME Wayland.
