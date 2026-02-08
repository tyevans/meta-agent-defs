#!/usr/bin/env bash
# meta-agent-defs installer
# Creates symlinks from ~/.claude/ to this repo's files.
# Re-running is idempotent — existing symlinks are refreshed, regular files are backed up.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_DIR="$HOME/.claude"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log()  { echo -e "${GREEN}[+]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
info() { echo -e "${BLUE}[i]${NC} $1"; }

link_file() {
    local src="$1"
    local dst="$2"

    # Create parent directory if needed
    mkdir -p "$(dirname "$dst")"

    if [ -L "$dst" ]; then
        # Already a symlink — update it
        rm "$dst"
        ln -s "$src" "$dst"
        log "Updated: $dst -> $src"
    elif [ -e "$dst" ]; then
        # Regular file exists — back it up
        local backup="${dst}.bak.$(date +%Y%m%d%H%M%S)"
        mv "$dst" "$backup"
        warn "Backed up existing file: $dst -> $backup"
        ln -s "$src" "$dst"
        log "Linked: $dst -> $src"
    else
        # Nothing exists — create fresh
        ln -s "$src" "$dst"
        log "Linked: $dst -> $src"
    fi
}

echo ""
echo "meta-agent-defs V5 installer"
echo "============================"
echo "Source: $SCRIPT_DIR"
echo "Target: $CLAUDE_DIR"
echo ""

# Ensure ~/.claude/ exists
mkdir -p "$CLAUDE_DIR/agents"
mkdir -p "$CLAUDE_DIR/commands"

# --- Agents ---
info "Installing agents..."
for agent in "$SCRIPT_DIR"/agents/*.md; do
    [ -f "$agent" ] || continue
    name="$(basename "$agent")"
    link_file "$agent" "$CLAUDE_DIR/agents/$name"
done

# --- Commands ---
info "Installing commands..."
for cmd in "$SCRIPT_DIR"/commands/*.md; do
    [ -f "$cmd" ] || continue
    name="$(basename "$cmd")"
    link_file "$cmd" "$CLAUDE_DIR/commands/$name"
done

# --- Settings ---
info "Installing settings..."
link_file "$SCRIPT_DIR/settings.json" "$CLAUDE_DIR/settings.json"

echo ""
log "Installation complete!"
echo ""
info "What was installed:"
echo "  Agents:   $(ls -1 "$SCRIPT_DIR"/agents/*.md 2>/dev/null | wc -l) agent definitions"
echo "  Commands: $(ls -1 "$SCRIPT_DIR"/commands/*.md 2>/dev/null | wc -l) slash commands"
echo "  Settings: settings.json (hooks + env)"
echo ""
info "To verify:"
echo "  ls -la ~/.claude/agents/"
echo "  ls -la ~/.claude/commands/"
echo "  ls -la ~/.claude/settings.json"
echo ""
info "To uninstall, remove the symlinks:"
echo "  find ~/.claude -type l -lname '$SCRIPT_DIR/*' -delete"
echo ""
