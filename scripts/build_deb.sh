#!/usr/bin/env bash
# Build an installable .deb of easy-note (HUN-287).
#
# Usage: scripts/build_deb.sh [--out DIR] [--features LIST]
#
#   --out DIR        where the .deb goes (default: dist/)
#   --features LIST  cargo features for the release build (default: window,tray)
#
# EASY_NOTE_BIN packages a prebuilt binary and skips the cargo build; tests/deb.rs uses it to
# package /bin/true in under a second. EASY_NOTE_MAINTAINER sets the Maintainer control field.
#
# The package carries the binary and the docs, and nothing else: the icons, the window page and
# its fonts are embedded in the binary. It ships no maintainer script, because a postinst runs as
# root and the launcher, the tray and the Obsidian vault are per-person files. After the install,
# the person runs `easy-note setup` themselves.
set -euo pipefail
umask 022

usage='usage: scripts/build_deb.sh [--out DIR] [--features LIST]'
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
out=$root/dist
features=window,tray

while [ $# -gt 0 ]; do
  case $1 in
    --out) out=$2; shift 2 ;;
    --features) features=$2; shift 2 ;;
    -h|--help) echo "$usage"; exit 0 ;;
    *) echo "unknown option: $1" >&2; echo "$usage" >&2; exit 2 ;;
  esac
done

for tool in dpkg dpkg-deb dpkg-shlibdeps; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "need $tool — install it with: sudo apt install dpkg-dev" >&2
    exit 1
  }
done

version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$root/Cargo.toml" | head -1)
[ -n "$version" ] || { echo "no version in Cargo.toml" >&2; exit 1; }
arch=$(dpkg --print-architecture)
maintainer=${EASY_NOTE_MAINTAINER:-"easy-note <easy-note@localhost>"}

bin=${EASY_NOTE_BIN:-}
if [ -z "$bin" ]; then
  echo "building the release binary (features: $features)" >&2
  ( cd "$root" && cargo build --release --locked --features "$features" )
  bin=$root/target/release/easy-note
fi
[ -x "$bin" ] || { echo "not an executable file: $bin" >&2; exit 1; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
pkg=$work/pkg

install -D -m 0755 "$bin" "$pkg/usr/bin/easy-note"
# Debian policy wants no debug symbols in the shipped binary. Strip the staged copy, never the
# one in target/, so a backtrace from a cargo build keeps its symbols.
if command -v strip >/dev/null 2>&1; then
  strip --strip-unneeded "$pkg/usr/bin/easy-note" 2>/dev/null || true
fi
install -D -m 0644 "$root/README.md" "$pkg/usr/share/doc/easy-note/README.md"

{
  cat <<'COPYRIGHT'
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: easy-note

Files: *
License: MIT

License: MIT
COPYRIGHT
  sed 's/^$/./; s/^/ /' "$root/LICENSE"
} > "$pkg/usr/share/doc/easy-note/copyright"

commit=$(cd "$root" && git rev-parse --short HEAD 2>/dev/null || echo unknown)
cat <<CHANGELOG | gzip -9n > "$pkg/usr/share/doc/easy-note/changelog.gz"
easy-note ($version) unstable; urgency=medium

  * Build from the easy-note source tree at commit $commit.
  * Cargo features: ${features:-none}.

 -- $maintainer  $(date -R)
CHANGELOG

# Depends, read from the binary itself. dpkg-shlibdeps wants a debian/control beside its cwd, so
# it gets a throwaway one outside the package root.
mkdir -p "$work/debian"
printf 'Source: easy-note\n\nPackage: easy-note\nArchitecture: any\n' > "$work/debian/control"
if ! depends=$(cd "$work" && dpkg-shlibdeps -O pkg/usr/bin/easy-note 2>"$work/shlibdeps.err"); then
  grep -v 'binaries to analyze should already be installed' "$work/shlibdeps.err" >&2 || true
  echo "dpkg-shlibdeps failed on $bin" >&2
  exit 1
fi
depends=${depends#shlibs:Depends=}

size=$(du -ks "$pkg" | cut -f1)
mkdir -p "$pkg/DEBIAN"
cat > "$pkg/DEBIAN/control" <<CONTROL
Package: easy-note
Version: $version
Architecture: $arch
Maintainer: $maintainer
Installed-Size: $size
Depends: $depends
Section: utils
Priority: optional
Description: capture one-line notes from the terminal and from MCP clients
 easy-note saves a one-line note into a folder of Markdown files. It captures from the
 terminal, from a hotkey, from its own dashboard window and from MCP clients such as
 Claude Code.
 .
 Run "easy-note setup" once as your own user to add the launcher, the tray and the
 Obsidian vault entry. The package itself writes nothing below your home folder.
CONTROL

( cd "$pkg" && find usr -type f -print0 | sort -z | xargs -0 md5sum > DEBIAN/md5sums )

mkdir -p "$out"
deb=$out/easy-note_${version}_${arch}.deb
dpkg-deb --root-owner-group --build "$pkg" "$deb" >/dev/null
echo "$deb"
