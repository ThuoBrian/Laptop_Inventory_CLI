#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# stop.sh — Gracefully stop the Laptop Inventory API server.
# ─────────────────────────────────────────────────────────────────────────────
set -eo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PID_FILE="$PROJECT_ROOT/.server.pid"

# ── No PID file ───────────────────────────────────────────────────────────────
if [[ ! -f "$PID_FILE" ]]; then
    echo "No PID file found. The server is probably not running."
    exit 0
fi

PID=$(cat "$PID_FILE")

# ── Process already gone ─────────────────────────────────────────────────────
if ! kill -0 "$PID" 2>/dev/null; then
    echo "No process found with PID $PID. Removing stale PID file."
    rm -f "$PID_FILE"
    exit 0
fi

# ── Graceful shutdown (SIGTERM) ───────────────────────────────────────────────
echo "Stopping server (PID $PID)..."
kill -SIGTERM "$PID"

# Wait up to 10 seconds for clean exit.
WAIT=0
while kill -0 "$PID" 2>/dev/null; do
    if [[ $WAIT -ge 10 ]]; then
        echo "Server did not stop within 10 s — force killing..."
        kill -SIGKILL "$PID" 2>/dev/null || true
        break
    fi
    sleep 1
    (( WAIT++ )) || true
done

rm -f "$PID_FILE"
echo "Server stopped."
