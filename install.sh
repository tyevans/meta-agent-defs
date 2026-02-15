#!/usr/bin/env bash
# meta-agent-defs installer
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
SKIP_RUST=false

show_help() {
    cat << EOF
Usage: ./install.sh [project_dir] [--hardlink] [--skip-rust] [--help]

Options:
  project_dir   Install to project_dir/.claude/ instead of ~/.claude/
  --hardlink    Use hardlinks instead of symlinks
  --skip-rust   Skip optional git-intel Rust CLI build
  --help, -h    Show this help message
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
        --skip-rust)
            SKIP_RUST=true
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
MANIFEST_FILE="$TARGET_DIR/.meta-agent-defs.manifest"
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
echo "Workbench installer"
echo "==================="
echo "Source: $SCRIPT_DIR"
echo "Target: $TARGET_DIR"
if [ "$USE_HARDLINKS" = true ]; then
    echo "Mode:   hardlink"
else
    echo "Mode:   symlink"
fi
if [ -n "$PROJECT_DIR" ]; then
    echo "Scope:  project-local (agents + skills only)"
else
    echo "Scope:  global"
fi
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
mkdir -p "$TARGET_DIR/agents"
mkdir -p "$TARGET_DIR/skills"
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
info "Installing agents..."
for agent in "$SCRIPT_DIR"/agents/*.md; do
    [ -f "$agent" ] || continue
    name="$(basename "$agent")"
    link_file "$agent" "$TARGET_DIR/agents/$name"
done

# --- Skills ---
info "Installing skills..."
for skill_dir in "$SCRIPT_DIR"/skills/*/; do
    [ -d "$skill_dir" ] || continue
    name="$(basename "$skill_dir")"
    link_dir "$skill_dir" "$TARGET_DIR/skills/$name"
done

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

# --- Settings ---
if [ -z "$PROJECT_DIR" ]; then
    info "Installing settings..."
    link_file "$SCRIPT_DIR/settings.json" "$TARGET_DIR/settings.json"
else
    info "Skipping settings.json (project-local mode - would cause duplicate hooks)"
fi

# --- MCP Servers ---
if [ -z "$PROJECT_DIR" ]; then
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
else
    info "Skipping MCP servers (project-local mode - global-only)"
fi

# --- Write manifest ---
info "Writing installation manifest..."
{
    echo "# meta-agent-defs manifest - installed files"
    if [ "$USE_HARDLINKS" = true ]; then
        LINK_MODE="hardlink"
    else
        LINK_MODE="symlink"
    fi
    echo "# mode=$LINK_MODE target=$TARGET_DIR date=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo ""
    # Sort and deduplicate
    printf '%s\n' "${INSTALLED_FILES[@]}" | sort -u
} > "$MANIFEST_FILE"
log "Manifest written: $MANIFEST_FILE ($(wc -l < "$MANIFEST_FILE") entries)"

echo ""

# --- git-intel Rust CLI (optional) ---
if [ "$SKIP_RUST" = false ]; then
    GIT_INTEL_DIR="$SCRIPT_DIR/tools/git-intel"
    if [ -d "$GIT_INTEL_DIR" ] && [ -f "$GIT_INTEL_DIR/Cargo.toml" ]; then
        info "Checking for git-intel build prerequisites..."

        if command -v cargo &>/dev/null; then
            # cargo is available - offer to build
            log "Found cargo — git-intel can be built"
            info "git-intel provides: Rust CLI for churn analysis, pattern detection, and file lifecycle tracking"
            echo ""
            read -p "Build git-intel now? [y/N] " -n 1 -r
            echo ""

            if [[ $REPLY =~ ^[Yy]$ ]]; then
                info "Building git-intel in $GIT_INTEL_DIR..."
                if (cd "$GIT_INTEL_DIR" && cargo build 2>&1); then
                    log "git-intel built successfully"
                    info "Binary available at: $GIT_INTEL_DIR/target/debug/git-intel"
                else
                    warn "git-intel build failed (see output above)"
                    warn "Skills using git-intel will fall back gracefully"
                fi
            else
                info "Skipping git-intel build (you can build later with: cd $GIT_INTEL_DIR && cargo build)"
            fi
        else
            # cargo not available - print friendly skip message
            warn "git-intel (optional Rust CLI for churn/pattern analysis) requires cargo"
            info "Install rustup (https://rustup.rs) to enable it. All skills work without it."
        fi
        echo ""
    fi
fi

log "Installation complete!"
echo ""
info "What was installed:"
echo "  Agents:   $(ls -1 "$SCRIPT_DIR"/agents/*.md 2>/dev/null | wc -l) agent definitions"
echo "  Skills:   $(ls -d "$SCRIPT_DIR"/skills/*/ 2>/dev/null | wc -l) skills"
if [ -z "$PROJECT_DIR" ]; then
    if [ -d "$SCRIPT_DIR/rules" ]; then
        echo "  Rules:    $(ls -1 "$SCRIPT_DIR"/rules/*.md 2>/dev/null | wc -l) global rules"
    fi
    if [ -d "$SCRIPT_DIR/templates" ]; then
        echo "  Templates: $(find "$SCRIPT_DIR/templates" -type f -name "*.yaml" 2>/dev/null | wc -l) team templates"
    fi
    echo "  Settings: settings.json (hooks + env)"
    MCP_CONFIG="$SCRIPT_DIR/mcp-servers.json"
    if [ -f "$MCP_CONFIG" ] && command -v claude &>/dev/null; then
        echo "  MCP:      $(python3 -c "import json; print(len(json.load(open('$MCP_CONFIG'))))" 2>/dev/null || echo '?') server(s)"
    fi
else
    echo "  Mode:     Project-local (agents + skills only)"
fi
echo ""
info "To verify:"
echo "  ls -la $TARGET_DIR/agents/"
echo "  ls -la $TARGET_DIR/skills/"
if [ -z "$PROJECT_DIR" ]; then
    if [ -d "$SCRIPT_DIR/rules" ]; then
        echo "  ls -la $TARGET_DIR/rules/"
    fi
    if [ -d "$SCRIPT_DIR/templates" ]; then
        echo "  ls -la $TARGET_DIR/templates/"
    fi
    echo "  ls -la $TARGET_DIR/settings.json"
fi
echo ""
info "To uninstall:"
echo "  xargs rm -f < $MANIFEST_FILE && rm $MANIFEST_FILE"
echo ""
