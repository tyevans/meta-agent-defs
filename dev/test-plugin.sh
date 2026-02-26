#!/usr/bin/env bash
# Launch Claude Code with tackline loaded as a local plugin (no caching).
# Edits to skills, agents, hooks are reflected immediately.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Starting Claude Code with tackline plugin from: $SCRIPT_DIR"
echo "Skills will be namespaced as /tl:<name>"
echo ""

claude --plugin-dir "$SCRIPT_DIR" "$@"
