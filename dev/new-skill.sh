#!/usr/bin/env bash
# Scaffolds a new skill definition in skills/<name>/SKILL.md
# Usage: ./new-skill.sh <skill-name>
#
# This is the PREFERRED format for new workflows (over commands).

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
    err "Usage: $0 <skill-name>"
    echo "  Example: $0 deploy-check"
    echo "  Creates: skills/deploy-check/SKILL.md"
    exit 1
fi

NAME="$1"
SKILL_DIR="$SCRIPT_DIR/skills/${NAME}"
TARGET="$SKILL_DIR/SKILL.md"

# --- Check for existing file ---
if [ -e "$TARGET" ]; then
    err "File already exists: $TARGET"
    warn "Refusing to overwrite. Delete the file first if you want to recreate it."
    exit 1
fi

# --- Ensure directory exists ---
mkdir -p "$SKILL_DIR"

# --- Write template ---
cat > "$TARGET" << 'TEMPLATE'
---
name: SKILL_NAME_PLACEHOLDER
# description: Rich description including WHEN to use and keyword triggers.
#   Claude uses this to decide whether to auto-invoke the skill.
#   Include keywords at the end for discoverability.
description: "TODO: describe what this skill does and when to use it. Keywords: TODO"
# argument-hint: Shown in autocomplete to guide the user on what to pass.
#   Example: "<target: staged changes, commit, range, PR#>"
argument-hint: "<TODO: describe expected arguments>"
# disable-model-invocation: Controls whether Claude can auto-invoke this skill.
#   true  = user must explicitly call /skill-name (use for destructive or heavyweight operations)
#   false = Claude can invoke it when the description matches the user's intent
disable-model-invocation: true
# user-invocable: Whether the skill shows up in /slash-command autocomplete.
#   Should almost always be true for user-facing skills.
user-invocable: true
# allowed-tools: Restrict which tools the skill can use.
#   Use Bash(prefix:*) to allow only specific bash command prefixes.
#   Examples:
#     Read-only:  Read, Grep, Glob, Bash(git:*), Bash(bd:*)
#     Read-write: Read, Grep, Glob, Bash, Write, Edit
#     With tasks:  Read, Grep, Glob, Bash(bd:*), Bash(git:*), Task, SendMessage
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*)
# context: How the skill runs relative to the main conversation.
#   fork   = runs in a separate context (does not pollute main conversation)
#   inline = runs in the current conversation context
#   Prefer "fork" for heavyweight skills, "inline" for lightweight diagnostics.
# context: fork
---

# SKILL_TITLE_PLACEHOLDER

You are running **SKILL_NAME_PLACEHOLDER** -- TODO: brief description of what this skill does. Target: **$ARGUMENTS**

## When to Use

- <!-- When should a user invoke this skill? -->
- <!-- What problem does it solve? -->

## Overview

SKILL_TITLE_PLACEHOLDER works in N phases:

```
Phase 1: Gather information
  -> Phase 2: Process / analyze
    -> Phase 3: Produce output
      -> Phase 4: Session close
```

---

## Phase 1: Gather Information

### 1a. TODO

<!-- Collect the information needed -->

---

## Phase 2: Process

### 2a. TODO

<!-- Core logic -->

---

## Phase 3: Output

### 3a. TODO

<!-- What the skill produces -->

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

- <!-- Key principles for this skill -->
- Commit work before session transitions
- When in doubt, checkpoint (commit + sync) before proceeding
TEMPLATE

# Replace placeholders with actual name
sed -i "s/SKILL_NAME_PLACEHOLDER/${NAME}/g" "$TARGET"

TITLE=$(echo "$NAME" | sed 's/-/ /g' | sed 's/\b\(.\)/\u\1/g')
sed -i "s/SKILL_TITLE_PLACEHOLDER/${TITLE}/g" "$TARGET"

echo ""
log "Created: $TARGET"
echo ""
info "Next steps:"
echo "  1. Edit $TARGET to fill in the TODO sections"
echo "  2. Choose a context mode: uncomment 'context: fork' in frontmatter if this is a heavyweight skill"
echo "  3. Run ./install.sh to create symlinks"
echo "  4. Verify with: ls -la ~/.claude/skills/${NAME}"
echo ""
info "Skill frontmatter reference:"
echo "  name:                     Skill identifier (used in /slash-command)"
echo "  description:              When to use + keywords for auto-discovery"
echo "  argument-hint:            Autocomplete hint for user input"
echo "  disable-model-invocation: true = manual only, false = Claude can auto-invoke"
echo "  allowed-tools:            Restrict available tools (security boundary)"
echo "  context:                  fork (separate context) or inline (current context)"
echo ""
