#!/usr/bin/env bash
# Scaffolds a new command definition file in commands/
# Usage: ./new-command.sh <command-name>
#
# NOTE: Commands are the LEGACY format. Prefer skills (./new-skill.sh) for new workflows.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors (matching install.sh style)
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

log()  { echo -e "${GREEN}[+]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
info() { echo -e "${BLUE}[i]${NC} $1"; }
err()  { echo -e "${RED}[x]${NC} $1"; }

# --- Validate arguments ---
if [ $# -lt 1 ] || [ -z "$1" ]; then
    err "Usage: $0 <command-name>"
    echo "  Example: $0 deploy-check"
    echo "  Creates: commands/deploy-check.md"
    exit 1
fi

NAME="$1"
TARGET="$SCRIPT_DIR/commands/${NAME}.md"

# --- Check for existing file ---
if [ -e "$TARGET" ]; then
    err "File already exists: $TARGET"
    warn "Refusing to overwrite. Delete the file first if you want to recreate it."
    exit 1
fi

# --- Ensure directory exists ---
mkdir -p "$SCRIPT_DIR/commands"

# --- Write template ---
cat > "$TARGET" << 'TEMPLATE'
# COMMAND_TITLE_PLACEHOLDER

<!-- NOTE: Commands are the legacy format. For new workflows, prefer skills
     (see skills/ directory and ./new-skill.sh). Commands remain supported
     as fallbacks but skills offer richer metadata and discoverability. -->

You are running **COMMAND_NAME_PLACEHOLDER** -- BRIEF_DESCRIPTION_PLACEHOLDER. Focus area (optional): **$ARGUMENTS**

## When to Use

- <!-- Describe the situations where this command is appropriate -->

## Overview

COMMAND_TITLE_PLACEHOLDER works in N phases:

```
Phase 1: Gather information
  -> Phase 2: Process / analyze
    -> Phase 3: Produce output
      -> Phase 4: Session close
```

---

## Phase 1: Gather Information

### 1a. TODO

<!-- Collect the information needed for this command -->

```bash
# Example commands
git status
```

---

## Phase 2: Process

### 2a. TODO

<!-- Core logic of the command -->

---

## Phase 3: Output

### 3a. TODO

<!-- What the command produces -->

---

## Phase 4: Session Close

### 4a. Sync State

```bash
bd sync --flush-only
```

### 4b. Final Status

```bash
git status
```

Report to the user:
- What was accomplished
- Any follow-up actions needed

---

## Guidelines

- <!-- Key principles for this command -->
- Commit work before session transitions
- When in doubt, checkpoint (commit + sync) before proceeding
TEMPLATE

# Replace placeholders with actual name
sed -i "s/COMMAND_NAME_PLACEHOLDER/${NAME}/g" "$TARGET"

TITLE=$(echo "$NAME" | sed 's/-/ /g' | sed 's/\b\(.\)/\u\1/g')
sed -i "s/COMMAND_TITLE_PLACEHOLDER/${TITLE}/g" "$TARGET"
sed -i "s/BRIEF_DESCRIPTION_PLACEHOLDER/TODO: describe what this command does/g" "$TARGET"

echo ""
log "Created: $TARGET"
echo ""
warn "Commands are the LEGACY format. Consider using ./new-skill.sh instead."
echo ""
info "Next steps:"
echo "  1. Edit $TARGET to fill in the TODO sections"
echo "  2. Run ./install.sh to create symlinks"
echo "  3. Verify with: ls -la ~/.claude/commands/${NAME}.md"
echo ""
info "Definition of done checklist (from .claude/rules/definition-of-done.md):"
echo "  [ ] Uses \$ARGUMENTS for user input where applicable"
echo "  [ ] Has clear phase structure with numbered steps"
echo "  [ ] Includes session close reminder (bd sync, git status)"
echo "  [ ] Tested by running the slash command in a live session"
echo ""
