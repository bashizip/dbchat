#!/usr/bin/env bash
set -euo pipefail

# Build dbchat binaries for distribution and package them
# Usage: ./scripts/build-release.sh [version]

VERSION="${1:-$(grep '^version =' dbchat-cli/Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')}"
DIST_DIR="dist"
TARGETS=(
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
)

echo "${DIM}Building dbchat v${VERSION} for:${NC}"
for target in "${TARGETS[@]}"; do
  echo "  - $target"
done
echo ""

mkdir -p "$DIST_DIR"

for target in "${TARGETS[@]}"; do
  echo "${CYAN}Building${NC} $target ..."

  rustup target add "$target" 2>/dev/null || true

  CARGO_TARGET_DIR="target-release" cargo build --release --target "$target" 2>/dev/null

  BIN="target-release/$target/release/dbchat"
  if [ ! -f "$BIN" ]; then
    echo "${YELLOW}⚠ Skipping $target (build failed)${NC}"
    continue
  fi

  OS_ARCH="${target//unknown-/}"
  OS_ARCH="${OS_ARCH//-gnu/}"
  ARCHIVE="dbchat-${OS_ARCH}.tar.gz"

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
