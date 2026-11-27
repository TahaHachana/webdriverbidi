#!/usr/bin/env bash
# Install chromedriver from Chrome for Testing
# (https://googlechromelabs.github.io/chrome-for-testing/).
#
# By default the driver matches the major version of the installed Chrome
# (chromedriver refuses to drive a Chrome with a different major version).
# If no Chrome is found, the latest Stable chromedriver is installed.
#
# Usage: scripts/install-chromedriver.sh [options]
#
#   --channel <name>   Install the latest driver of a channel instead:
#                      stable, beta, dev or canary.
#   --version <v>      Install an exact version (e.g. 158.0.8127.88) or the
#                      latest build of a milestone (e.g. 158).
#   --dir <path>       Install directory (default: <repo>/drivers, git-ignored).
#   --force            Reinstall even if the same version is already present.
#
# Requires curl and python3 (unzip is used when available).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
API="https://googlechromelabs.github.io/chrome-for-testing"

CHANNEL=""
VERSION=""
DEST="$ROOT/drivers"
FORCE=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --channel) CHANNEL="${2:?--channel needs a value}"; shift 2 ;;
    --version) VERSION="${2:?--version needs a value}"; shift 2 ;;
    --dir) DEST="${2:?--dir needs a value}"; shift 2 ;;
    --force) FORCE=1; shift ;;
    -h|--help) sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done
[[ -n "$CHANNEL" && -n "$VERSION" ]] && { echo "use either --channel or --version, not both" >&2; exit 2; }

die() { printf 'error: %s\n' "$*" >&2; exit 1; }

# Platform -------------------------------------------------------------------
case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) PLATFORM=linux64 ;;
  Linux-aarch64|Linux-arm64) PLATFORM=linux-arm64 ;;
  Darwin-arm64) PLATFORM=mac-arm64 ;;
  Darwin-x86_64) PLATFORM=mac-x64 ;;
  MINGW*-x86_64|MSYS*-x86_64|CYGWIN*-x86_64) PLATFORM=win64 ;;
  *) die "unsupported platform: $(uname -s) $(uname -m)" ;;
esac
EXE=chromedriver
[[ "$PLATFORM" == win64 ]] && EXE=chromedriver.exe

# Installed Chrome -----------------------------------------------------------
detect_chrome_version() {
  local bin
  for bin in google-chrome google-chrome-stable chromium chromium-browser \
             "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"; do
    if command -v "$bin" >/dev/null 2>&1 || [[ -x "$bin" ]]; then
      "$bin" --version 2>/dev/null | grep -oE '[0-9]+(\.[0-9]+){3}' | head -n1 && return 0
    fi
  done
  return 1
}

if [[ -z "$CHANNEL" && -z "$VERSION" ]]; then
  if CHROME_VERSION="$(detect_chrome_version)"; then
    VERSION="${CHROME_VERSION%%.*}"
    echo "Found Chrome $CHROME_VERSION; matching milestone $VERSION"
  else
    CHANNEL=stable
    echo "No Chrome found; using the latest Stable chromedriver"
  fi
fi

# Resolve version and download URL ------------------------------------------
# Prints "<version> <url>" for the chosen channel / milestone / exact version.
resolve() {
  local endpoint selector
  if [[ -n "$CHANNEL" ]]; then
    endpoint="last-known-good-versions-with-downloads.json"
    selector="channel"
  elif [[ "$VERSION" =~ ^[0-9]+$ ]]; then
    endpoint="latest-versions-per-milestone-with-downloads.json"
    selector="milestone"
  else
    endpoint="known-good-versions-with-downloads.json"
    selector="exact"
  fi
  curl -fsSL "$API/$endpoint" | python3 -I -c '
import json, sys
selector, key, platform = sys.argv[1:4]
data = json.load(sys.stdin)
if selector == "channel":
    entry = data["channels"].get(key.capitalize())
elif selector == "milestone":
    entry = data["milestones"].get(key)
else:
    entry = next((v for v in data["versions"] if v["version"] == key), None)
if entry is None:
    sys.exit(f"no Chrome for Testing release found for {selector} {key!r}")
for d in entry.get("downloads", {}).get("chromedriver", []):
    if d["platform"] == platform:
        print(entry["version"], d["url"])
        break
else:
    version = entry["version"]
    sys.exit(f"no chromedriver download for {platform} in {version} "
             "(chromedriver is only published for Chrome 115+)")
' "$selector" "${CHANNEL:-$VERSION}" "$PLATFORM"
}

read -r DRIVER_VERSION URL < <(resolve) || die "could not resolve a chromedriver download"
[[ -n "${URL:-}" ]] || die "could not resolve a chromedriver download"

TARGET="$DEST/$EXE"
if [[ $FORCE -eq 0 && -x "$TARGET" ]]; then
  CURRENT="$("$TARGET" --version 2>/dev/null | grep -oE '[0-9]+(\.[0-9]+){3}' | head -n1 || true)"
  if [[ "$CURRENT" == "$DRIVER_VERSION" ]]; then
    echo "chromedriver $DRIVER_VERSION is already installed at $TARGET"
    exit 0
  fi
fi

# Download and install -------------------------------------------------------
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading chromedriver $DRIVER_VERSION ($PLATFORM)..."
curl -fSL --progress-bar -o "$TMP/chromedriver.zip" "$URL"

mkdir -p "$TMP/out"
if command -v unzip >/dev/null 2>&1; then
  unzip -q "$TMP/chromedriver.zip" -d "$TMP/out"
else
  python3 -I -m zipfile -e "$TMP/chromedriver.zip" "$TMP/out"
fi

SRC="$TMP/out/chromedriver-$PLATFORM/$EXE"
[[ -f "$SRC" ]] || die "archive did not contain chromedriver-$PLATFORM/$EXE"

mkdir -p "$DEST"
install -m 0755 "$SRC" "$TARGET"
# Gatekeeper quarantines downloaded binaries on macOS.
[[ "$PLATFORM" == mac-* ]] && xattr -d com.apple.quarantine "$TARGET" 2>/dev/null || true

echo "Installed: $("$TARGET" --version)"
echo "Location:  $TARGET"
echo
echo "Start it for this crate's examples and tests with:"
echo "  $TARGET --port=4444"
