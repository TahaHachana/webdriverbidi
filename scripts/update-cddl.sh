#!/usr/bin/env bash
# Regenerate cddl/{all,local,remote}.cddl from the latest WebDriver BiDi spec.
#
# Usage: scripts/update-cddl.sh [git-ref]   (default: main)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
REF="${1:-main}"
REPO="https://github.com/w3c/webdriver-bidi.git"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Fetching $REPO ($REF)..."
git clone --quiet --depth 1 --branch "$REF" "$REPO" "$TMP/webdriver-bidi"
SHA="$(git -C "$TMP/webdriver-bidi" rev-parse HEAD)"

python3 -I "$ROOT/scripts/extract_cddl.py" "$TMP/webdriver-bidi/index.bs" "$ROOT/cddl"

if command -v cddl >/dev/null 2>&1; then
  for f in local remote all; do
    cddl compile-cddl --cddl "$ROOT/cddl/$f.cddl"
  done
else
  echo "note: 'cddl' not installed (cargo install cddl); skipping validation"
fi

echo "CDDL updated from w3c/webdriver-bidi@$SHA"
git -C "$ROOT" --no-pager diff --stat -- cddl
