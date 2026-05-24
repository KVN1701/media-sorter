#!/bin/sh
set -e

REPO="KVN1701/media-sorter"
BINARY="media-sorter"
INSTALL_DIR="/usr/local/bin"

# get latest version
VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" \
  | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

# detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

case "$OS" in
  linux)  TARGET="${ARCH}-unknown-linux-gnu" ;;
  darwin) TARGET="${ARCH}-apple-darwin" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

URL="https://github.com/$REPO/releases/download/$VERSION/${BINARY}-${TARGET}.tar.gz"

echo "Installing $BINARY $VERSION for $TARGET..."
curl -sL "$URL" | tar xz
chmod +x "$BINARY"
sudo mv "$BINARY" "$INSTALL_DIR/"
echo "Done! Run: $BINARY --version"