#!/usr/bin/env bash
# nightly.sh - Build a debug app bundle and publish it as a GitHub prerelease.
#
# Usage:
#   ./scripts/nightly.sh                     # build + upload
#   ./scripts/nightly.sh --upload-only       # skip build, re-upload existing dist/Kaku.app
#   ./scripts/nightly.sh --features=remote   # enable optional cargo features
#
# Published as a prerelease under the fixed tag "nightly"; each run clobbers
# the previous artifact. Concurrent runs are blocked via a PID lock file.
#
# Requirements: gh CLI authenticated (gh auth login)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

GITHUB_REPO="${GITHUB_REPO:-tw93/Kaku}"
NIGHTLY_TAG="${NIGHTLY_TAG:-nightly}"
OUT_DIR="${OUT_DIR:-$REPO_ROOT/dist}"
ZIP_NAME="Kaku-nightly.zip"
ZIP_PATH="$OUT_DIR/$ZIP_NAME"
UPLOAD_ONLY=0
FEATURES=""

for arg in "$@"; do
    case "$arg" in
        --upload-only) UPLOAD_ONLY=1 ;;
        --features=*) FEATURES="${arg#--features=}" ;;
    esac
done

# Concurrency guard: only one nightly build at a time per checkout.
mkdir -p "$OUT_DIR"
LOCK="$OUT_DIR/.nightly.lock"
if ! (set -o noclobber; echo $$ > "$LOCK") 2>/dev/null; then
    echo "nightly.sh already running (PID $(cat "$LOCK" 2>/dev/null)); remove $LOCK if stale" >&2
    exit 1
fi
trap 'rm -f "$LOCK"' EXIT

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'
log() { echo -e "${GREEN}[nightly]${NC} $*"; }
warn() { echo -e "${YELLOW}[nightly]${NC} $*"; }

# Build
if [[ "$UPLOAD_ONLY" -eq 0 ]]; then
    log "Building debug app bundle..."
    # Filter the noisy ranlib warning, but preserve build.sh's exit code via
    # PIPESTATUS — `| grep ... || true` would mask any build failure.
    set +e
    PROFILE=debug CARGO_FEATURES="$FEATURES" ./scripts/build.sh --app-only 2>&1 \
        | grep -v 'ranlib: warning:.*has no symbols'
    BUILD_STATUS=${PIPESTATUS[0]}
    set -e
    if [[ "$BUILD_STATUS" -ne 0 ]]; then
        echo "build.sh failed with exit code $BUILD_STATUS" >&2
        exit "$BUILD_STATUS"
    fi
    log "Build complete: $OUT_DIR/Kaku.app"
fi

# Zip
log "Packaging $ZIP_NAME..."
rm -f "$ZIP_PATH"
cd "$OUT_DIR"
zip -qr "$ZIP_NAME" Kaku.app
cd "$REPO_ROOT"
SIZE=$(du -sh "$ZIP_PATH" | cut -f1)
log "Package ready: $ZIP_PATH ($SIZE)"

# Draft release
if ! command -v gh >/dev/null 2>&1; then
    echo "gh CLI not found. Install from https://cli.github.com/" >&2
    exit 1
fi
if ! gh auth status >/dev/null 2>&1; then
    echo "gh CLI not authenticated. Run: gh auth login" >&2
    exit 1
fi

SHORT_SHA=$(git rev-parse --short HEAD)
TIMESTAMP=$(date -u "+%Y-%m-%d %H:%M UTC")
BODY="Debug build for testing. Commit: \`$SHORT_SHA\` ($TIMESTAMP)

**Install:** Download and unzip, then drag \`Kaku.app\` to \`/Applications\`.
> Ad-hoc signed (debug build). macOS may require: System Settings → Privacy & Security → Open Anyway."

if gh release view "$NIGHTLY_TAG" -R "$GITHUB_REPO" >/dev/null 2>&1; then
    log "Updating existing nightly release '$NIGHTLY_TAG'..."
    gh release edit "$NIGHTLY_TAG" \
        -R "$GITHUB_REPO" \
        --prerelease \
        --draft=false \
        --title "Nightly ($SHORT_SHA)" \
        --notes "$BODY"
    gh release upload "$NIGHTLY_TAG" \
        -R "$GITHUB_REPO" \
        "$ZIP_PATH" \
        --clobber
else
    log "Creating nightly release '$NIGHTLY_TAG'..."
    gh release create "$NIGHTLY_TAG" \
        -R "$GITHUB_REPO" \
        "$ZIP_PATH" \
        --prerelease \
        --title "Nightly ($SHORT_SHA)" \
        --notes "$BODY"
fi

DOWNLOAD_URL="https://github.com/$GITHUB_REPO/releases/download/$NIGHTLY_TAG/$ZIP_NAME"
log "Done."
echo ""
echo "  Download: $DOWNLOAD_URL"
echo ""
