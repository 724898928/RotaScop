#!/usr/bin/env bash
# RotaScope USB Display Startup Script (macOS/Linux)
set -euo pipefail

PORT="${1:-8083}"
DISPLAY_INDEX="${2:-0}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SERVER_DIR="$ROOT/rotascope-server"

echo "RotaScope USB display"
echo "Workspace: $ROOT"
echo "Port: $PORT"
echo "Display index: $DISPLAY_INDEX"
echo ""

# Check adb
if ! command -v adb &>/dev/null; then
    echo "Error: adb was not found. Install Android platform-tools and add adb to PATH."
    exit 1
fi

# Check cargo
if ! command -v cargo &>/dev/null; then
    echo "Error: cargo was not found. Install Rust from https://rustup.rs/"
    exit 1
fi

# Check for connected Android device
DEVICES=$(adb devices)
if ! echo "$DEVICES" | grep -q "device$"; then
    echo "Warning: No authorized Android device found."
    echo "Connect the phone by USB, enable USB debugging, and accept the authorization prompt."
fi

# Set up ADB reverse tunnel
adb reverse "tcp:$PORT" "tcp:$PORT"
echo "ADB reverse is ready: phone 127.0.0.1:$PORT -> PC 127.0.0.1:$PORT"

# Start the server
export ROTASCOPE_DISPLAY_INDEX="$DISPLAY_INDEX"
cd "$SERVER_DIR"
cargo run
