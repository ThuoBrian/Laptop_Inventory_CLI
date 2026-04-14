#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# restart.sh — Stop and restart the Laptop Inventory API server.
#
# Usage:
#   ./scripts/restart.sh           # release build (default)
#   ./scripts/restart.sh --dev     # debug build
# ─────────────────────────────────────────────────────────────────────────────
set -eo pipefail

SCRIPTS_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Restarting server..."
echo ""
"$SCRIPTS_DIR/stop.sh"
echo ""
"$SCRIPTS_DIR/start.sh" "$@"
