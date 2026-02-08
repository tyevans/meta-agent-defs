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

# --- Dependency checks ---
if ! command -v git &>/dev/null; then
    warn "REQUIRED: git is not installed. Cannot proceed."
    exit 1
fi

if ! command -v bd &>/dev/null; then
    warn "RECOMMENDED: bd (beads) not found. Session hooks depend on it for backlog management."
    warn "Install beads or hooks will degrade gracefully with fallback messages."
fi

if ! command -v claude &>/dev/null; then
    info "OPTIONAL: claude CLI not found. Install it to use these definitions."
fi

echo ""

# Ensure ~/.claude/ exists
mkdir -p "$CLAUDE_DIR/agents"
mkdir -p "$CLAUDE_DIR/commands"

# --- Stale symlink cleanup ---
info "Checking for stale symlinks..."
STALE_COUNT=0
for dir in "$CLAUDE_DIR/agents" "$CLAUDE_DIR/commands"; do
    [ -d "$dir" ] || continue
    for link in "$dir"/*; do
        [ -L "$link" ] || continue
        target="$(readlink "$link")"
        # Only consider symlinks pointing into this repo
        case "$target" in
            "$SCRIPT_DIR"/*) ;;
            *) continue ;;
        esac
        if [ ! -e "$target" ]; then
            rm "$link"
            warn "Removed stale symlink: $link -> $target"
            STALE_COUNT=$((STALE_COUNT + 1))
        fi
    done
done
if [ "$STALE_COUNT" -gt 0 ]; then
    log "Cleaned up $STALE_COUNT stale symlink(s)"
else
    log "No stale symlinks found"
fi

echo ""

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

# --- MCP Servers ---
MCP_CONFIG="$SCRIPT_DIR/mcp-servers.json"
if [ -f "$MCP_CONFIG" ]; then
    if command -v claude &>/dev/null; then
        info "Installing MCP servers..."
        # Parse server names from the JSON config
        SERVER_NAMES=$(python3 -c "import json; [print(k) for k in json.load(open('$MCP_CONFIG')).keys()]" 2>/dev/null)
        if [ -n "$SERVER_NAMES" ]; then
            while IFS= read -r name; do
                # Extract command and args for this server
                CMD=$(python3 -c "import json; print(json.load(open('$MCP_CONFIG'))['$name']['command'])" 2>/dev/null)
                ARGS=$(python3 -c "import json; print(' '.join(json.load(open('$MCP_CONFIG'))['$name']['args']))" 2>/dev/null)
                if [ -n "$CMD" ] && [ -n "$ARGS" ]; then
                    # claude mcp add exits 1 if server already exists -- that's fine
                    OUTPUT=$(claude mcp add "$name" --scope user -- $CMD $ARGS 2>&1) && \
                        log "MCP: $name" || \
                        { if echo "$OUTPUT" | grep -q "already exists"; then
                            log "MCP: $name (already installed)"
                          else
                            warn "MCP: Failed to add $name -- $OUTPUT"
                          fi; }
                fi
            done <<< "$SERVER_NAMES"
        else
            warn "MCP: Could not parse $MCP_CONFIG"
        fi
    else
        info "Skipping MCP server installation (claude CLI not found)"
    fi
fi

echo ""
log "Installation complete!"
echo ""
info "What was installed:"
echo "  Agents:   $(ls -1 "$SCRIPT_DIR"/agents/*.md 2>/dev/null | wc -l) agent definitions"
echo "  Commands: $(ls -1 "$SCRIPT_DIR"/commands/*.md 2>/dev/null | wc -l) slash commands"
echo "  Settings: settings.json (hooks + env)"
if [ -f "$MCP_CONFIG" ] && command -v claude &>/dev/null; then
    echo "  MCP:      $(python3 -c "import json; print(len(json.load(open('$MCP_CONFIG'))))" 2>/dev/null || echo '?') server(s)"
fi
echo ""
info "To verify:"
echo "  ls -la ~/.claude/agents/"
echo "  ls -la ~/.claude/commands/"
echo "  ls -la ~/.claude/settings.json"
echo ""
info "To uninstall, remove the symlinks:"
echo "  find ~/.claude -type l -lname '$SCRIPT_DIR/*' -delete"
echo ""
