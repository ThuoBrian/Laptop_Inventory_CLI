#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# start.sh — Build (release) and start the Laptop Inventory API in the background.
#
# Usage:
#   ./scripts/start.sh           # release build (default)
#   ./scripts/start.sh --dev     # debug build (faster compile, larger binary)
# ─────────────────────────────────────────────────────────────────────────────
set -eo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PID_FILE="$PROJECT_ROOT/.server.pid"
LOG_FILE="$PROJECT_ROOT/server.log"
BUILD_MODE="release"

# ── Parse flags ──────────────────────────────────────────────────────────────
for arg in "$@"; do
    case $arg in
        --dev) BUILD_MODE="debug" ;;
        *) echo "Unknown flag: $arg"; exit 1 ;;
    esac
done

# ── Guard: already running? ──────────────────────────────────────────────────
if [[ -f "$PID_FILE" ]]; then
    PID=$(cat "$PID_FILE")
    if kill -0 "$PID" 2>/dev/null; then
        echo "Server is already running (PID $PID)."
        echo "Use scripts/restart.sh to restart, or scripts/stop.sh to stop."
        exit 1
    else
        echo "Stale PID file found — cleaning up."
        rm -f "$PID_FILE"
    fi
fi

# ── Load .env ────────────────────────────────────────────────────────────────
if [[ -f "$PROJECT_ROOT/.env" ]]; then
    set -a
    source "$PROJECT_ROOT/.env"
    set +a
else
    echo "ERROR: .env file not found. Copy .env.example to .env and configure it."
    exit 1
fi

# ── Build ────────────────────────────────────────────────────────────────────
echo "Building ($BUILD_MODE)..."
cd "$PROJECT_ROOT"

if [[ "$BUILD_MODE" == "release" ]]; then
    cargo build --release
    BINARY="$PROJECT_ROOT/target/release/laptop_inventory_cli"
else
    cargo build
    BINARY="$PROJECT_ROOT/target/debug/laptop_inventory_cli"
fi

# Windows: append .exe if the binary was produced as an .exe
[[ -f "${BINARY}.exe" ]] && BINARY="${BINARY}.exe"

# ── Start ─────────────────────────────────────────────────────────────────────
echo "Starting server..."
RUST_LOG="${RUST_LOG:-info}" "$BINARY" >> "$LOG_FILE" 2>&1 &
PID=$!

# Brief pause to let the process fail fast if there's a startup error.
sleep 1
if ! kill -0 "$PID" 2>/dev/null; then
    echo "ERROR: Server failed to start. Check $LOG_FILE for details."
    exit 1
fi

echo "$PID" > "$PID_FILE"
HOST_DISPLAY="${HOST:-127.0.0.1}"
PORT_DISPLAY="${PORT:-5342}"
echo ""
echo "  Server running at http://${HOST_DISPLAY}:${PORT_DISPLAY}"
echo "  PID         : $PID"
echo "  Log file    : $LOG_FILE"
echo "  Build mode  : $BUILD_MODE"
echo ""
echo "  Tail logs   : tail -f $LOG_FILE"
echo "  Stop server : ./scripts/stop.sh"
