#!/usr/bin/env bash
# Install geckodriver (Firefox's WebDriver server) from its GitHub releases
# (https://github.com/mozilla/geckodriver/releases).
#
# Unlike chromedriver, one geckodriver release supports a range of Firefox
# versions, so the latest release is the right default.
#
# Usage: scripts/install-geckodriver.sh [options]
#
#   --version <v>   Install a specific release, e.g. 0.37.1 (default: latest).
#   --dir <path>    Install directory (default: <repo>/drivers, git-ignored).
#   --force         Reinstall even if the same version is already present.
#
# Requires curl, tar (or unzip on Windows) and python3.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
REPO="mozilla/geckodriver"

VERSION=""
DEST="$ROOT/drivers"
FORCE=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version) VERSION="${2:?--version needs a value}"; VERSION="${VERSION#v}"; shift 2 ;;
    --dir) DEST="${2:?--dir needs a value}"; shift 2 ;;
    --force) FORCE=1; shift ;;
    -h|--help) sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

die() { printf 'error: %s\n' "$*" >&2; exit 1; }

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) PLATFORM=linux64; EXT=tar.gz ;;
  Linux-aarch64|Linux-arm64) PLATFORM=linux-aarch64; EXT=tar.gz ;;
  Darwin-arm64) PLATFORM=macos-aarch64; EXT=tar.gz ;;
  Darwin-x86_64) PLATFORM=macos; EXT=tar.gz ;;
  MINGW*-x86_64|MSYS*-x86_64|CYGWIN*-x86_64) PLATFORM=win64; EXT=zip ;;
  *) die "unsupported platform: $(uname -s) $(uname -m)" ;;
esac
EXE=geckodriver
[[ "$PLATFORM" == win64 ]] && EXE=geckodriver.exe

if [[ -z "$VERSION" ]]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["tag_name"].lstrip("v"))')" \
    || die "could not look up the latest geckodriver release"
fi

TARGET="$DEST/$EXE"
if [[ $FORCE -eq 0 && -x "$TARGET" ]]; then
  CURRENT="$("$TARGET" --version 2>/dev/null | head -n1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)"
  if [[ "$CURRENT" == "$VERSION" ]]; then
    echo "geckodriver $VERSION is already installed at $TARGET"
    exit 0
  fi
fi

URL="https://github.com/$REPO/releases/download/v$VERSION/geckodriver-v$VERSION-$PLATFORM.$EXT"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading geckodriver $VERSION ($PLATFORM)..."
curl -fSL --progress-bar -o "$TMP/geckodriver.$EXT" "$URL" || die "download failed: $URL"
if [[ "$EXT" == zip ]]; then
  unzip -q "$TMP/geckodriver.$EXT" -d "$TMP"
else
  tar -xzf "$TMP/geckodriver.$EXT" -C "$TMP"
fi
[[ -f "$TMP/$EXE" ]] || die "archive did not contain $EXE"

mkdir -p "$DEST"
install -m 0755 "$TMP/$EXE" "$TARGET"
[[ "$PLATFORM" == macos* ]] && xattr -d com.apple.quarantine "$TARGET" 2>/dev/null || true

echo "Installed: $("$TARGET" --version | head -n1)"
echo "Location:  $TARGET"
echo
echo "Start it for this crate's tests with:"
echo "  $TARGET --port=4444"
echo "  WEBDRIVERBIDI_BROWSER=firefox cargo test --tests -- --test-threads=1"
