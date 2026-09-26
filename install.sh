#!/bin/sh
# Install the syscribe CLI from a GitHub release.
#
#   curl -fsSL https://raw.githubusercontent.com/sjames/syscribe/main/install.sh | sh
#
# Options (flags or environment variables):
#   -d, --dir <path>   install directory   (SYSCRIBE_INSTALL_DIR, default: ~/.local/bin)
#   -h, --help
#
# Always installs the latest release.
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
url="https://github.com/$REPO/releases/latest/download/$asset"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

say "Downloading $url"
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$url" -o "$tmp/syscribe" || die "download failed"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$tmp/syscribe" "$url" || die "download failed"
else
  die "need curl or wget"
fi
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
