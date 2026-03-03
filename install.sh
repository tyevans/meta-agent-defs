#!/usr/bin/env bash
# Tackline installer
# Creates symlinks from ~/.claude/ to this repo's files.
# Re-running is idempotent — existing symlinks are refreshed, regular files are backed up.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log()  { echo -e "${GREEN}[+]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
info() { echo -e "${BLUE}[i]${NC} $1"; }

# --- Argument parsing ---
PROJECT_DIR=""
USE_HARDLINKS=false
RULES_ONLY=false
USE_TACKS=false
USE_BEADS=false
show_help() {
    cat << EOF
Usage: ./install.sh [project_dir] [--hardlink] [--rules-only] [--beads] [--tacks] [--help]

Options:
  project_dir      Install to project_dir/.claude/ instead of ~/.claude/
  --hardlink       Use hardlinks instead of symlinks
  --rules-only     Install only rules and templates (use with plugin install)
  --beads          Require beads (bd) for task management (opt-in; tacks is default)
  --tacks          No-op (tacks is already the default); kept for backward compatibility
  --help, -h       Show this help message

Install modes:
  Full install (default)  Symlinks agents, skills, rules, templates, MCP servers
  Plugin companion        Use --rules-only to install what plugins can't provide
  Project-local           Provide project_dir for agents + skills only

Task management:
  Default: auto-detects tacks (tk) first, then beads (bd)
  --beads: require beads and prefer it over tacks
EOF
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h)
            show_help
            ;;
        --hardlink)
            USE_HARDLINKS=true
            shift
            ;;
        --rules-only)
            RULES_ONLY=true
            shift
            ;;
        --tacks)
            USE_TACKS=true
            shift
            ;;
        --beads)
            USE_BEADS=true
            shift
            ;;
        -*)
            echo "Unknown option: $1"
            show_help
            ;;
        *)
            # Positional argument: project_dir
            if [ -z "$PROJECT_DIR" ]; then
                PROJECT_DIR="$1"
            else
                echo "Error: Multiple positional arguments provided"
                show_help
            fi
            shift
            ;;
    esac
done

# Set TARGET_DIR based on parsed arguments
if [ -n "$PROJECT_DIR" ]; then
    # Validate that PROJECT_DIR exists and is a directory
    if [ ! -d "$PROJECT_DIR" ]; then
        echo "Error: PROJECT_DIR '$PROJECT_DIR' does not exist or is not a directory"
        exit 1
    fi
    TARGET_DIR="$PROJECT_DIR/.claude"
else
    TARGET_DIR="$HOME/.claude"
fi

# Define manifest file location and tracking array
MANIFEST_FILE="$TARGET_DIR/.tackline.manifest"
INSTALLED_FILES=()

# --- Cross-filesystem check for hardlinks ---
if [ "$USE_HARDLINKS" = true ]; then
    # Get parent directory of TARGET_DIR (since TARGET_DIR may not exist yet)
    TARGET_PARENT="$(dirname "$TARGET_DIR")"

    # Get device IDs using stat
    SRC_DEV=$(stat -c %d "$SCRIPT_DIR" 2>/dev/null)
    DST_DEV=$(stat -c %d "$TARGET_PARENT" 2>/dev/null)

    if [ "$SRC_DEV" != "$DST_DEV" ]; then
        echo "Error: Cannot use hardlinks across filesystems. Source ($SCRIPT_DIR) and target ($TARGET_DIR) are on different devices."
        exit 1
    fi
fi

link_file() {
    local src="$1"
    local dst="$2"

    # Create parent directory if needed
    mkdir -p "$(dirname "$dst")"

    if [ "$USE_HARDLINKS" = true ]; then
        # Hardlink mode
        if [ -e "$dst" ]; then
            # Check if already hardlinked (same inode)
            SRC_INODE=$(stat -c %i "$src" 2>/dev/null)
            DST_INODE=$(stat -c %i "$dst" 2>/dev/null)

            if [ "$SRC_INODE" = "$DST_INODE" ]; then
                log "Already linked: $dst"
                INSTALLED_FILES+=("$dst")
                return
            else
                # Different inode — back up and hardlink
                local backup="${dst}.bak.$(date +%Y%m%d%H%M%S)"
                mv "$dst" "$backup"
                warn "Backed up existing file: $dst -> $backup"
                ln "$src" "$dst"
                log "Hardlinked: $dst -> $src"
                INSTALLED_FILES+=("$dst")
            fi
        else
            # Nothing exists — create fresh hardlink
            ln "$src" "$dst"
            log "Hardlinked: $dst -> $src"
            INSTALLED_FILES+=("$dst")
        fi
    else
        # Symlink mode (original behavior)
        if [ -L "$dst" ]; then
            # Already a symlink — update it
            rm "$dst"
            ln -s "$src" "$dst"
            log "Updated: $dst -> $src"
            INSTALLED_FILES+=("$dst")
        elif [ -e "$dst" ]; then
            # Regular file exists — back it up
            local backup="${dst}.bak.$(date +%Y%m%d%H%M%S)"
            mv "$dst" "$backup"
            warn "Backed up existing file: $dst -> $backup"
            ln -s "$src" "$dst"
            log "Linked: $dst -> $src"
            INSTALLED_FILES+=("$dst")
        else
            # Nothing exists — create fresh
            ln -s "$src" "$dst"
            log "Linked: $dst -> $src"
            INSTALLED_FILES+=("$dst")
        fi
    fi
}

link_dir() {
    local src="$1"
    local dst="$2"

    if [ "$USE_HARDLINKS" = true ]; then
        # Hardlink mode: create target dir and hardlink each file
        mkdir -p "$dst"

        local file_count=0
        for file in "$src"/*; do
            [ -f "$file" ] || continue
            local name="$(basename "$file")"
            link_file "$file" "$dst/$name"
            file_count=$((file_count + 1))
        done

        if [ "$file_count" -gt 0 ]; then
            log "Hardlinked directory: $dst ($file_count files)"
            INSTALLED_FILES+=("$dst")
        fi
    else
        # Symlink mode: symlink the whole directory
        link_file "$src" "$dst"
    fi
}

echo ""
echo "Tackline installer"
echo "=================="
echo "Source: $SCRIPT_DIR"
echo "Target: $TARGET_DIR"
if [ "$USE_HARDLINKS" = true ]; then
    echo "Mode:   hardlink"
else
    echo "Mode:   symlink"
fi
if [ -n "$PROJECT_DIR" ]; then
    echo "Scope:  project-local (agents + skills only)"
elif [ "$RULES_ONLY" = true ]; then
    echo "Scope:  rules-only (plugin companion mode)"
else
    echo "Scope:  global"
fi
if [ "$USE_BEADS" = true ]; then
    echo "Backlog: beads (--beads)"
fi
echo ""

# --- Plugin conflict detection ---
if [ "$RULES_ONLY" = false ] && [ -z "$PROJECT_DIR" ]; then
    # Check if the tl plugin is already installed
    PLUGIN_ACTIVE=false
    if [ -f "$HOME/.claude/settings.json" ] && command -v python3 &>/dev/null; then
        if python3 -c "
import json, sys
try:
    s = json.load(open('$HOME/.claude/settings.json'))
    plugins = s.get('enabledPlugins', [])
    if any('tl' in str(p) for p in plugins):
        sys.exit(0)
except Exception:
    pass
sys.exit(1)
" 2>/dev/null; then
            PLUGIN_ACTIVE=true
        fi
    fi
    if [ "$PLUGIN_ACTIVE" = true ]; then
        warn "The 'tl' plugin appears to be installed."
        warn "Full install will create duplicate skills/agents in both global (/gather) and plugin (/tl:gather) namespaces."
        warn "Consider using --rules-only instead to install only what the plugin can't provide."
        echo ""
        read -p "Continue with full install anyway? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            info "Aborting. Run with --rules-only for plugin companion mode."
            exit 0
        fi
    fi
fi

# --- Dependency checks ---
if ! command -v git &>/dev/null; then
    warn "REQUIRED: git is not installed. Cannot proceed."
    exit 1
fi

# --- Task manager detection ---
if [ "$USE_BEADS" = true ]; then
    if ! command -v bd &>/dev/null; then
        warn "REQUIRED: bd (beads) not found but --beads was specified."
        warn "Install beads: see beads documentation"
        exit 1
    fi
    BACKLOG_TOOL="bd"
    log "Task manager: beads (bd)"
elif command -v tk &>/dev/null; then
    BACKLOG_TOOL="tk"
    log "Task manager: tacks (tk)"
elif command -v bd &>/dev/null; then
    BACKLOG_TOOL="bd"
    log "Task manager: beads (bd)"
else
    warn "RECOMMENDED: Neither tk (tacks) nor bd (beads) found."
    warn "Session hooks depend on a task manager for backlog management."
    warn "Install tacks: cargo install tacks"
    warn "Install beads: see beads documentation"
    BACKLOG_TOOL=""
fi

if ! command -v claude &>/dev/null; then
    info "OPTIONAL: claude CLI not found. Install it to use these definitions."
fi

echo ""

# Ensure target directories exist
if [ "$RULES_ONLY" = false ]; then
    mkdir -p "$TARGET_DIR/agents"
    mkdir -p "$TARGET_DIR/skills"
fi
if [ -z "$PROJECT_DIR" ]; then
    if [ -d "$SCRIPT_DIR/rules" ]; then
        mkdir -p "$TARGET_DIR/rules"
    fi
    if [ -d "$SCRIPT_DIR/templates" ]; then
        mkdir -p "$TARGET_DIR/templates"
    fi
fi

# --- Stale cleanup ---
info "Checking for stale installations..."
STALE_COUNT=0
if [ -f "$MANIFEST_FILE" ]; then
    # Manifest-based cleanup: remove entries whose source no longer exists
    while IFS= read -r entry; do
        [[ "$entry" =~ ^# ]] && continue
        [ -z "$entry" ] && continue
        # Only remove if the installed file/link still exists but its source is gone
        if [ -e "$entry" ] || [ -L "$entry" ]; then
            # For symlinks, check if target exists; for hardlinks, the entry itself is the check
            if [ -L "$entry" ]; then
                target="$(readlink "$entry")"
                if [ ! -e "$target" ]; then
                    rm "$entry"
                    warn "Removed stale link: $entry -> $target"
                    STALE_COUNT=$((STALE_COUNT + 1))
                fi
            fi
        fi
    done < "$MANIFEST_FILE"
else
    # Legacy fallback: symlink-based detection (no manifest from prior install)
    for dir in "$TARGET_DIR/agents" "$TARGET_DIR/skills" "$TARGET_DIR/rules"; do
        [ -d "$dir" ] || continue
        for link in "$dir"/*; do
            [ -L "$link" ] || continue
            target="$(readlink "$link")"
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
fi
if [ "$STALE_COUNT" -gt 0 ]; then
    log "Cleaned up $STALE_COUNT stale installation(s)"
else
    log "No stale installations found"
fi

echo ""

# --- Agents ---
if [ "$RULES_ONLY" = false ]; then
    info "Installing agents..."
    for agent in "$SCRIPT_DIR"/agents/*.md; do
        [ -f "$agent" ] || continue
        name="$(basename "$agent")"
        link_file "$agent" "$TARGET_DIR/agents/$name"
    done
else
    info "Skipping agents (--rules-only mode)"
fi

# --- Skills ---
if [ "$RULES_ONLY" = false ]; then
    # Skills may be organized in category subdirectories (skills/core/gather/, skills/workflows/blossom/)
    # or directly under skills/ (skills/gather/). Both layouts are flattened on install.
    info "Installing skills..."
    for entry in "$SCRIPT_DIR"/skills/*/; do
        [ -d "$entry" ] || continue
        if [ -f "$entry/SKILL.md" ]; then
            # Direct skill directory (skills/foo/SKILL.md)
            name="$(basename "$entry")"
            link_dir "$entry" "$TARGET_DIR/skills/$name"
        else
            # Category directory (skills/core/foo/SKILL.md) — flatten into skills/foo/
            for skill_dir in "$entry"*/; do
                [ -d "$skill_dir" ] || continue
                [ -f "$skill_dir/SKILL.md" ] || continue
                name="$(basename "$skill_dir")"
                link_dir "$skill_dir" "$TARGET_DIR/skills/$name"
            done
        fi
    done
else
    info "Skipping skills (--rules-only mode)"
fi

# --- Rules ---
if [ -z "$PROJECT_DIR" ]; then
    if [ -d "$SCRIPT_DIR/rules" ]; then
        info "Installing rules..."
        link_dir "$SCRIPT_DIR/rules" "$TARGET_DIR/rules"
    fi
else
    info "Skipping global rules (project-local mode - use .claude/rules/ instead)"
fi

# --- Templates ---
if [ -z "$PROJECT_DIR" ]; then
    if [ -d "$SCRIPT_DIR/templates" ]; then
        info "Installing templates..."
        link_dir "$SCRIPT_DIR/templates" "$TARGET_DIR/templates"
    fi
else
    info "Skipping templates (project-local mode - global feature)"
fi

# --- MCP Servers ---
if [ -z "$PROJECT_DIR" ] && [ "$RULES_ONLY" = false ]; then
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
elif [ "$RULES_ONLY" = true ]; then
    info "Skipping MCP servers (--rules-only mode)"
else
    info "Skipping MCP servers (project-local mode - global-only)"
fi

# --- Write manifest ---
info "Writing installation manifest..."
{
    echo "# tackline manifest - installed files"
    if [ "$USE_HARDLINKS" = true ]; then
        LINK_MODE="hardlink"
    else
        LINK_MODE="symlink"
    fi
    echo "# mode=$LINK_MODE backend=${BACKLOG_TOOL:-none} target=$TARGET_DIR date=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo ""
    # Sort and deduplicate
    printf '%s\n' "${INSTALLED_FILES[@]}" | sort -u
} > "$MANIFEST_FILE"
log "Manifest written: $MANIFEST_FILE ($(wc -l < "$MANIFEST_FILE") entries)"

echo ""

log "Installation complete!"
echo ""
info "What was installed:"
if [ "$RULES_ONLY" = true ]; then
    echo "  Mode:     Plugin companion (--rules-only)"
    if [ -d "$SCRIPT_DIR/rules" ]; then
        echo "  Rules:    $(ls -1 "$SCRIPT_DIR"/rules/*.md 2>/dev/null | wc -l) global rules"
    fi
    if [ -d "$SCRIPT_DIR/templates" ]; then
        echo "  Templates: $(find "$SCRIPT_DIR/templates" -type f -name "*.yaml" 2>/dev/null | wc -l) team templates"
    fi
elif [ -n "$PROJECT_DIR" ]; then
    echo "  Mode:     Project-local (agents + skills only)"
    echo "  Agents:   $(ls -1 "$SCRIPT_DIR"/agents/*.md 2>/dev/null | wc -l) agent definitions"
    echo "  Skills:   $(find "$SCRIPT_DIR/skills" -name "SKILL.md" 2>/dev/null | wc -l) skills"
else
    echo "  Agents:   $(ls -1 "$SCRIPT_DIR"/agents/*.md 2>/dev/null | wc -l) agent definitions"
    echo "  Skills:   $(find "$SCRIPT_DIR/skills" -name "SKILL.md" 2>/dev/null | wc -l) skills"
    if [ -d "$SCRIPT_DIR/rules" ]; then
        echo "  Rules:    $(ls -1 "$SCRIPT_DIR"/rules/*.md 2>/dev/null | wc -l) global rules"
    fi
    if [ -d "$SCRIPT_DIR/templates" ]; then
        echo "  Templates: $(find "$SCRIPT_DIR/templates" -type f -name "*.yaml" 2>/dev/null | wc -l) team templates"
    fi
    MCP_CONFIG="$SCRIPT_DIR/mcp-servers.json"
    if [ -f "$MCP_CONFIG" ] && command -v claude &>/dev/null; then
        echo "  MCP:      $(python3 -c "import json; print(len(json.load(open('$MCP_CONFIG'))))" 2>/dev/null || echo '?') server(s)"
    fi
fi
if [ -n "$BACKLOG_TOOL" ]; then
    echo "  Backlog: $BACKLOG_TOOL ($(command -v $BACKLOG_TOOL))"
fi
echo ""
info "To verify:"
if [ "$RULES_ONLY" = false ]; then
    echo "  ls -la $TARGET_DIR/agents/"
    echo "  ls -la $TARGET_DIR/skills/"
fi
if [ -z "$PROJECT_DIR" ]; then
    if [ -d "$SCRIPT_DIR/rules" ]; then
        echo "  ls -la $TARGET_DIR/rules/"
    fi
    if [ -d "$SCRIPT_DIR/templates" ] && [ "$RULES_ONLY" = true ]; then
        echo "  ls -la $TARGET_DIR/templates/"
    elif [ -d "$SCRIPT_DIR/templates" ] && [ "$RULES_ONLY" = false ]; then
        echo "  ls -la $TARGET_DIR/templates/"
    fi
fi
echo ""
info "To uninstall:"
echo "  xargs rm -f < $MANIFEST_FILE && rm $MANIFEST_FILE"
echo ""
