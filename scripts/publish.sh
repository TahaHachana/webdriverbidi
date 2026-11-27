#!/usr/bin/env bash
# Publish the webdriverbidi crate to crates.io and tag the release.
#
# Usage: scripts/publish.sh [--dry-run] [--yes] [--with-integration] [--allow-branch]
#
#   --dry-run           Run all checks and `cargo publish --dry-run`, then stop.
#   --yes               Don't ask for confirmation before publishing.
#   --with-integration  Also run the integration tests in tests/ (they need a
#                       WebDriver BiDi server listening on localhost:4444).
#   --allow-branch      Allow publishing from a branch other than main.
#
# Requires `cargo login` (or CARGO_REGISTRY_TOKEN) for the actual publish.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
cd "$ROOT"

DRY_RUN=0
ASSUME_YES=0
WITH_INTEGRATION=0
ALLOW_BRANCH=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --yes|-y) ASSUME_YES=1 ;;
    --with-integration) WITH_INTEGRATION=1 ;;
    --allow-branch) ALLOW_BRANCH=1 ;;
    -h|--help) sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $arg" >&2; exit 2 ;;
  esac
done

step() { printf '\n\033[1;34m==> %s\033[0m\n' "$*"; }
die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

# 1. Git state ---------------------------------------------------------------
step "Checking git state"
BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$BRANCH" != "main" && $ALLOW_BRANCH -eq 0 ]]; then
  die "on branch '$BRANCH'; releases are cut from main (use --allow-branch to override)"
fi
if [[ -n "$(git status --porcelain)" ]]; then
  if [[ $DRY_RUN -eq 1 ]]; then
    echo "warning: working tree is dirty (allowed for --dry-run)"
  else
    die "working tree is dirty; commit or stash your changes first"
  fi
fi
if git ls-remote --exit-code origin >/dev/null 2>&1; then
  git fetch --quiet origin "$BRANCH" || true
  if git rev-parse --verify --quiet "origin/$BRANCH" >/dev/null; then
    LOCAL="$(git rev-parse HEAD)"
    REMOTE="$(git rev-parse "origin/$BRANCH")"
    if [[ "$LOCAL" != "$REMOTE" ]]; then
      if git merge-base --is-ancestor "$REMOTE" "$LOCAL"; then
        echo "note: $BRANCH is ahead of origin/$BRANCH; push it before tagging"
        [[ $DRY_RUN -eq 1 ]] || die "push $BRANCH to origin first"
      else
        die "$BRANCH is behind or has diverged from origin/$BRANCH; pull first"
      fi
    fi
  fi
else
  echo "warning: remote 'origin' not reachable; skipping up-to-date check"
fi

# 2. Version -----------------------------------------------------------------
step "Reading crate version"
read -r NAME VERSION < <(cargo metadata --no-deps --format-version 1 \
  | python3 -c 'import json,sys; p=json.load(sys.stdin)["packages"][0]; print(p["name"], p["version"])')
TAG="v$VERSION"
echo "$NAME $VERSION (tag $TAG)"

if git rev-parse --verify --quiet "refs/tags/$TAG" >/dev/null; then
  die "tag $TAG already exists; bump the version in Cargo.toml"
fi

STATUS="$(curl -s -o /dev/null -w '%{http_code}' -A 'webdriverbidi-release-script' \
  "https://crates.io/api/v1/crates/$NAME/$VERSION" || echo 000)"
case "$STATUS" in
  200) die "$NAME $VERSION is already published on crates.io; bump the version" ;;
  404) echo "$NAME $VERSION is not on crates.io yet" ;;
  *) echo "warning: could not query crates.io (HTTP $STATUS); continuing" ;;
esac

# 3. README and CHANGELOG ----------------------------------------------------------
step "Checking README version"
if ! grep -q "^$NAME = \"$VERSION\"" README.md; then
  die "README.md installation snippet does not reference $NAME = \"$VERSION\""
fi
echo "README.md references $VERSION"

step "Checking CHANGELOG"
if ! grep -q "^## \[$VERSION\]" CHANGELOG.md; then
  die "CHANGELOG.md has no '## [$VERSION]' section"
fi
echo "CHANGELOG.md has an entry for $VERSION"

# 4. Quality checks ----------------------------------------------------------
step "cargo fmt --check"
cargo fmt --check

step "cargo clippy"
# Warnings are reported but not fatal (the codebase has some old ones); errors fail.
cargo clippy --all-targets

step "cargo build"
cargo build --all-targets

step "cargo test (unit, doc and CDDL conformance)"
cargo test --lib
cargo test --doc
cargo test --test cddl_conformance

if [[ $WITH_INTEGRATION -eq 1 ]]; then
  step "cargo test (integration; needs a WebDriver on localhost:4444)"
  cargo test --tests -- --test-threads=1
fi

# 5. Package -----------------------------------------------------------------
step "Packaged files"
cargo package --list --allow-dirty

step "cargo publish --dry-run"
if [[ $DRY_RUN -eq 1 ]]; then
  cargo publish --dry-run --allow-dirty
  printf '\nDry run complete. Nothing was published.\n'
  exit 0
fi
cargo publish --dry-run

# 6. Publish -----------------------------------------------------------------
if [[ $ASSUME_YES -eq 0 ]]; then
  read -r -p "Publish $NAME $VERSION to crates.io and push tag $TAG? [y/N] " answer
  [[ "$answer" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 1; }
fi

step "cargo publish"
cargo publish

step "Tagging $TAG"
git tag -a "$TAG" -m "$NAME $VERSION"
git push origin "$TAG"

printf '\nPublished %s %s: https://crates.io/crates/%s/%s\n' "$NAME" "$VERSION" "$NAME" "$VERSION"
