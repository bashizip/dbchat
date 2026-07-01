#!/usr/bin/env bash
set -euo pipefail

REPO="bashizip/dbchat"
VERSION="${1:-latest}"
INSTALL_DIR="${2:-/usr/local/bin}"

BOLD='\033[1m'
GREEN='\033[32m'
CYAN='\033[36m'
RED='\033[31m'
DIM='\033[2m'
NC='\033[0m'

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "${RED}✗${NC} Required command not found: $1"
    exit 1
  fi
}

need_cmd curl
need_cmd tar
need_cmd shasum

if [ -t 1 ]; then
  echo "" # blank line for spacing in terminal
fi

# ── Detect OS + arch ──────────────────────────────────────
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

case "$OS" in
  linux|darwin) ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

# ── Resolve version ───────────────────────────────────────
if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl -sSfL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name":' \
    | sed 's/.*"tag_name": "\(.*\)",/\1/')
  if [ -z "$VERSION" ]; then
    echo "Failed to resolve latest version"
    exit 1
  fi
fi

ARCHIVE="dbchat-${OS}-${ARCH}.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION/$ARCHIVE"
SUMS_URL="https://github.com/$REPO/releases/download/$VERSION/SHA256SUMS"

# ── Download ───────────────────────────────────────────────
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "${CYAN}ℹ${NC} Downloading ${BOLD}dbchat ${VERSION}${NC} for ${OS}/${ARCH}..."
if ! curl -sSfL "$URL" -o "$TMPDIR/$ARCHIVE"; then
  echo "${RED}✗${NC} Release asset not found: $ARCHIVE"
  echo "${DIM}URL: $URL${NC}"
  exit 1
fi

if curl -sSfL "$SUMS_URL" -o "$TMPDIR/SHA256SUMS"; then
  if grep -F "  $ARCHIVE" "$TMPDIR/SHA256SUMS" > "$TMPDIR/$ARCHIVE.sha256"; then
    (cd "$TMPDIR" && shasum -a 256 -c "$ARCHIVE.sha256")
  else
    echo "${RED}✗${NC} Checksum missing for $ARCHIVE"
    exit 1
  fi
else
  echo "${DIM}No SHA256SUMS file found for ${VERSION}; skipping checksum verification.${NC}"
fi

# ── Extract ────────────────────────────────────────────────
tar -xzf "$TMPDIR/$ARCHIVE" -C "$TMPDIR"

# ── Install ────────────────────────────────────────────────
BINARY="$TMPDIR/dbchat"

if [ ! -f "$BINARY" ]; then
  echo "${RED}✗${NC} Binary not found in archive"
  exit 1
fi

if [ ! -d "$INSTALL_DIR" ]; then
  if ! mkdir -p "$INSTALL_DIR" 2>/dev/null; then
    echo "${CYAN}ℹ${NC} Need sudo to create ${BOLD}$INSTALL_DIR${NC}"
    sudo mkdir -p "$INSTALL_DIR"
  fi
fi

if [ ! -w "$INSTALL_DIR" ]; then
  echo "${CYAN}ℹ${NC} Need sudo to install to ${BOLD}$INSTALL_DIR${NC}"
  sudo mv "$BINARY" "$INSTALL_DIR/dbchat"
  sudo chmod +x "$INSTALL_DIR/dbchat"
else
  mv "$BINARY" "$INSTALL_DIR/dbchat"
  chmod +x "$INSTALL_DIR/dbchat"
fi

echo "${GREEN}✓${NC} Installed ${BOLD}dbchat${NC} to ${BOLD}$INSTALL_DIR/dbchat${NC}"
echo ""
echo "  Run:  ${CYAN}dbchat --help${NC}"
