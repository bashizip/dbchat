#!/usr/bin/env bash
set -euo pipefail

REPO="pbash/dbchat"
VERSION="${1:-latest}"
INSTALL_DIR="${2:-/usr/local/bin}"

BOLD='\033[1m'
GREEN='\033[32m'
CYAN='\033[36m'
DIM='\033[2m'
NC='\033[0m'

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

# ── Download ───────────────────────────────────────────────
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "${CYAN}ℹ${NC} Downloading ${BOLD}dbchat ${VERSION}${NC} for ${OS}/${ARCH}..."
curl -sSfL "$URL" -o "$TMPDIR/$ARCHIVE"

# ── Extract ────────────────────────────────────────────────
tar -xzf "$TMPDIR/$ARCHIVE" -C "$TMPDIR"

# ── Install ────────────────────────────────────────────────
BINARY="$TMPDIR/dbchat"

if [ ! -f "$BINARY" ]; then
  echo "${RED}✗${NC} Binary not found in archive"
  exit 1
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
