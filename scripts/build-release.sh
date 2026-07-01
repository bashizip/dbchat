#!/usr/bin/env bash
set -euo pipefail

# Build dbchat binaries for distribution and package them
# Usage: ./scripts/build-release.sh [version]

VERSION="${1:-$(grep '^version =' dbchat-cli/Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')}"
DIST_DIR="dist"
CYAN='\033[36m'
GREEN='\033[32m'
YELLOW='\033[33m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'
TARGETS=(
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
)

asset_name_for_target() {
  case "$1" in
    x86_64-unknown-linux-gnu) echo "dbchat-linux-x86_64.tar.gz" ;;
    aarch64-unknown-linux-gnu) echo "dbchat-linux-aarch64.tar.gz" ;;
    x86_64-apple-darwin) echo "dbchat-darwin-x86_64.tar.gz" ;;
    aarch64-apple-darwin) echo "dbchat-darwin-aarch64.tar.gz" ;;
    *) return 1 ;;
  esac
}

echo "${DIM}Building dbchat v${VERSION} for:${NC}"
for target in "${TARGETS[@]}"; do
  echo "  - $target"
done
echo ""

mkdir -p "$DIST_DIR"
: > "$DIST_DIR/SHA256SUMS"

for target in "${TARGETS[@]}"; do
  echo "${CYAN}Building${NC} $target ..."

  rustup target add "$target" 2>/dev/null || true

  CARGO_TARGET_DIR="target-release" cargo build --release --target "$target" 2>/dev/null

  BIN="target-release/$target/release/dbchat"
  if [ ! -f "$BIN" ]; then
    echo "${YELLOW}⚠ Skipping $target (build failed)${NC}"
    continue
  fi

  ARCHIVE="$(asset_name_for_target "$target")"

  TMPDIR=$(mktemp -d)
  cp "$BIN" "$TMPDIR/dbchat"
  cp README.md "$TMPDIR/" 2>/dev/null || true

  tar -czf "$DIST_DIR/$ARCHIVE" -C "$TMPDIR" .

  HASH=$(shasum -a 256 "$DIST_DIR/$ARCHIVE" | cut -d' ' -f1)
  echo "  → ${GREEN}$DIST_DIR/$ARCHIVE${NC} (${DIM}SHA256: ${HASH}${NC})"

  echo "$HASH  $ARCHIVE" >> "$DIST_DIR/SHA256SUMS"

  rm -rf "$TMPDIR"
done

echo ""
echo "${GREEN}Done.${NC} Packages in ${BOLD}$DIST_DIR/${NC}"
ls -lh "$DIST_DIR/"*.tar.gz 2>/dev/null
