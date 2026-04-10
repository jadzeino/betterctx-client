#!/usr/bin/env bash
set -euo pipefail

EXTENSION_ID="${1:-}"
if [[ -z "$EXTENSION_ID" ]]; then
  echo "Usage: ./install.sh <chrome-extension-id>"
  echo ""
  echo "The extension ID is shown in the better-ctx popup or at chrome://extensions"
  exit 1
fi

LEAN_CTX=$(command -v better-ctx 2>/dev/null || echo "$HOME/.cargo/bin/better-ctx")
if [[ ! -x "$LEAN_CTX" ]]; then
  echo "Error: better-ctx not found in PATH or ~/.cargo/bin/"
  echo "Install: cargo install better-ctx"
  exit 1
fi

HOST_NAME="com.betterctx.bridge"

case "$(uname)" in
  Darwin)
    TARGET_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
    ;;
  Linux)
    TARGET_DIR="$HOME/.config/google-chrome/NativeMessagingHosts"
    ;;
  *)
    echo "Unsupported platform: $(uname)"
    exit 1
    ;;
esac

mkdir -p "$TARGET_DIR"

INSTALL_DIR="$HOME/.better-ctx/chrome-bridge"
mkdir -p "$INSTALL_DIR"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cp "$SCRIPT_DIR/bridge.py" "$INSTALL_DIR/bridge.py"
chmod +x "$INSTALL_DIR/bridge.py"

PYTHON="/usr/bin/python3"
if [[ ! -x "$PYTHON" ]]; then
  PYTHON="$(command -v python3 2>/dev/null || true)"
fi
if [[ -z "$PYTHON" ]]; then
  echo "Error: python3 not found"
  exit 1
fi

cat > "$INSTALL_DIR/bridge.sh" <<BRIDGE
#!/bin/bash
exec "$PYTHON" -u "$INSTALL_DIR/bridge.py" 2>>"\$HOME/.better-ctx/bridge-stderr.log"
BRIDGE
chmod +x "$INSTALL_DIR/bridge.sh"

cat > "$TARGET_DIR/$HOST_NAME.json" <<MANIFEST
{
  "name": "$HOST_NAME",
  "description": "better-ctx native messaging bridge for Chrome",
  "path": "$INSTALL_DIR/bridge.sh",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://$EXTENSION_ID/"
  ]
}
MANIFEST

echo "Native messaging host installed successfully."
echo "  Manifest: $TARGET_DIR/$HOST_NAME.json"
echo "  Bridge:   $INSTALL_DIR/bridge.sh"
echo "  Python:   $PYTHON"
echo ""
echo "Restart Chrome completely (Cmd+Q) to activate."
