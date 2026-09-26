#!/bin/sh
# Install the syscribe CLI from a GitHub release.
#
#   curl -fsSL https://raw.githubusercontent.com/sjames/syscribe/main/install.sh | sh
#
# Options (flags or environment variables):
#   -d, --dir <path>   install directory   (SYSCRIBE_INSTALL_DIR, default: ~/.local/bin)
#   -h, --help
#
# Always installs the latest release, and verifies the download against the SHA-256
# published alongside it. Set SYSCRIBE_DOWNLOAD_BASE to install from a mirror.
#
# Linux gets the fully static musl build, which runs on any distribution. Windows: download
# syscribe-x86_64-pc-windows-msvc.exe from the releases page.
set -eu

REPO="sjames/syscribe"
INSTALL_DIR="${SYSCRIBE_INSTALL_DIR:-$HOME/.local/bin}"

say() { printf '%s\n' "$*"; }
die() { printf 'install.sh: error: %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    -d|--dir)     [ $# -ge 2 ] || die "$1 needs a value"; INSTALL_DIR="$2"; shift 2 ;;
    -h|--help)    sed -n '2,/^set -eu/p' "$0" | sed '$d; s/^# \{0,1\}//'; exit 0 ;;
    *) die "unknown option: $1 (try --help)" ;;
  esac
done

case "$(uname -s)" in
  Linux)  os=linux ;;
  Darwin) os=darwin ;;
  *) die "unsupported OS: $(uname -s). Download a binary from https://github.com/$REPO/releases" ;;
esac
case "$(uname -m)" in
  x86_64|amd64)  arch=x86_64 ;;
  aarch64|arm64) arch=aarch64 ;;
  *) die "unsupported architecture: $(uname -m)" ;;
esac
case "$os" in
  linux)  target="$arch-unknown-linux-musl" ;;
  darwin) target="$arch-apple-darwin" ;;
esac

asset="syscribe-$target"
base="${SYSCRIBE_DOWNLOAD_BASE:-https://github.com/$REPO/releases/latest/download}"
url="$base/$asset"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# fetch <url> <file>: exit 0 ok, 44 = HTTP error status (e.g. 404), anything else = failure.
fetch() {
  if command -v curl >/dev/null 2>&1; then
    rc=0; curl -fsL "$1" -o "$2" || rc=$?
    [ "$rc" -eq 0 ] && return 0
    [ "$rc" -eq 22 ] && return 44
    return 1
  elif command -v wget >/dev/null 2>&1; then
    rc=0; wget -qO "$2" "$1" || rc=$?
    [ "$rc" -eq 0 ] && return 0
    [ "$rc" -eq 8 ] && return 44
    return 1
  fi
  die "need curl or wget"
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1
  elif command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | cut -d' ' -f1
  fi
}
command -v sha256sum >/dev/null 2>&1 || command -v shasum >/dev/null 2>&1 \
  || die "need sha256sum or shasum to verify the download"

say "Downloading $url"
fetch "$url" "$tmp/syscribe" || die "download failed: $url"

rc=0; fetch "$url.sha256" "$tmp/syscribe.sha256" || rc=$?
case "$rc" in
  0)
    expected="$(cut -d' ' -f1 "$tmp/syscribe.sha256" | tr 'A-F' 'a-f')"
    actual="$(sha256_of "$tmp/syscribe")"
    [ -n "$expected" ] && [ "$expected" = "$actual" ] \
      || die "checksum mismatch for $asset (expected ${expected:-<empty>}, got $actual); not installing"
    say "Verified SHA-256 $actual"
    ;;
  44)
    # Releases cut before checksums were published have no .sha256 asset.
    say "Warning: no checksum is published for this release; skipping verification." >&2
    ;;
  *) die "could not download the checksum file" ;;
esac
chmod +x "$tmp/syscribe"

installed="$("$tmp/syscribe" --version 2>&1)" || die "the downloaded binary does not run on this system"

mkdir -p "$INSTALL_DIR"
mv "$tmp/syscribe" "$INSTALL_DIR/syscribe"
say "Installed $installed to $INSTALL_DIR/syscribe"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) say "Note: $INSTALL_DIR is not on your PATH. Add it, e.g.:  export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
say "Next: claude mcp add syscribe -- $INSTALL_DIR/syscribe -m /abs/path/to/model mcp"
