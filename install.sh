#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="${1:-/usr/local/bin}"
PROFILE="${2:-release}"

echo "Building easytodo ($PROFILE)..."
cargo build --profile "$PROFILE"

echo "Installing to $BIN_DIR..."
sudo install -m 755 "target/$PROFILE/easytodo" "$BIN_DIR/easytodo"
sudo install -m 755 "target/$PROFILE/mcp" "$BIN_DIR/easytodo-mcp"

echo "Done! Run 'easytodo' to start."
