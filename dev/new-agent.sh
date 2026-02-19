#!/usr/bin/env bash
# Scaffolds a new agent definition file in agents/
# Usage: ./new-agent.sh <agent-name>

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
    err "Usage: $0 <agent-name>"
    echo "  Example: $0 test-runner"
    echo "  Creates: agents/test-runner.md"
    exit 1
fi

NAME="$1"
TARGET="$SCRIPT_DIR/agents/${NAME}.md"

# --- Check for existing file ---
if [ -e "$TARGET" ]; then
    err "File already exists: $TARGET"
    warn "Refusing to overwrite. Delete the file first if you want to recreate it."
    exit 1
fi

# --- Ensure directory exists ---
mkdir -p "$SCRIPT_DIR/agents"

# --- Write template ---
cat > "$TARGET" << 'TEMPLATE'
---
name: AGENT_NAME_PLACEHOLDER
# description: WHEN to use this agent, not just what it does.
#   Start with a verb phrase. Example:
#   "Reviews staged diffs for correctness and security. Use when reviewing code
#    before merging or after an implementation agent completes work."
description: TODO - describe when to invoke this agent
# tools: Only list tools the agent actually needs.
#   Read-only agents should NOT have Write or Edit.
#   Common sets:
#     Read-only:  Read, Grep, Glob, Bash
#     Read-write: Read, Grep, Glob, Bash, Write, Edit
tools: Read, Grep, Glob, Bash
# model: Match task complexity.
#   opus   - deep reasoning, complex analysis
#   sonnet - most tasks (default)
#   haiku  - trivial/fast tasks
model: sonnet
# permissionMode: (optional) default|acceptEdits|dontAsk|bypassPermissions|plan
#   Omit to use the default. Never use bypassPermissions without justification.
---

# AGENT_TITLE_PLACEHOLDER

<!-- One paragraph: what this agent does and how it operates. -->

## Inputs

<!-- What does this agent receive? A bead ID, file path, arguments, etc. -->

## Process

### Step 1: Understand the Scope

<!-- How the agent determines what to work on -->

### Step 2: Execute

<!-- The core work the agent performs -->

### Step 3: Validate

<!-- How the agent checks its own work -->

## Output Format

<!-- What the agent produces: structured report, code changes, etc. -->

## Investigation Protocol

<!-- How this agent verifies findings rather than guessing.
     Required section per definition-of-done. Examples:
     - Read full implementations, not just names
     - Trace callers of modified functions
     - State confidence levels: CONFIRMED > LIKELY > POSSIBLE -->

1. **Read before concluding.** Do not make claims about code without reading the actual implementation.
2. **Trace call sites.** Use Grep to find every caller of modified functions and verify compatibility.
3. **State confidence levels** for non-obvious findings:
   - CONFIRMED: Verified by reading the implementation
   - LIKELY: Pattern suggests an issue but full path was not traced
   - POSSIBLE: Suspicious but could be intentional

## Context Management

<!-- How this agent avoids filling its context window.
     Required section per definition-of-done. Examples:
     - Read changed files fully before reviewing
     - Limit scope to the actual diff
     - Use Grep for targeted lookups, not broad exploration -->

- **Read files fully before acting.** Understand surrounding code before making changes or judgments.
- **Limit scope.** Focus on the specific task rather than exploring the entire codebase.
- **Use Grep for targeted lookups.** Search for specific identifiers, not open-ended patterns.

## Knowledge Transfer

<!-- How this agent picks up context and hands off results.
     Required section per definition-of-done. -->

**Before starting work:**
1. Ask the orchestrator for the bead ID you are working on
2. Run `bd show <id>` to read notes on the task and parent epic

**After completing work:**
Report back to the orchestrator:
- What was done and key findings
- Any issues that affect downstream tasks
- Patterns to follow in future work
TEMPLATE

# Replace placeholders with actual name
sed -i "s/AGENT_NAME_PLACEHOLDER/${NAME}/g" "$TARGET"

# Create a title from the name (hyphen-separated to Title Case)
TITLE=$(echo "$NAME" | sed 's/-/ /g' | sed 's/\b\(.\)/\u\1/g')
sed -i "s/AGENT_TITLE_PLACEHOLDER/${TITLE}/g" "$TARGET"

echo ""
log "Created: $TARGET"
echo ""
info "Next steps:"
echo "  1. Edit $TARGET to fill in the TODO sections"
echo "  2. Run ./install.sh to create symlinks"
echo "  3. Verify with: ls -la ~/.claude/agents/${NAME}.md"
echo ""
info "Definition of done checklist (from .claude/rules/definition-of-done.md):"
echo "  [ ] YAML frontmatter includes: name, description, tools, model"
echo "  [ ] Description says WHEN to use the agent"
echo "  [ ] Investigation Protocol, Context Management, Knowledge Transfer sections filled"
echo "  [ ] Model selection matches task complexity"
echo "  [ ] Tools list is minimal"
echo "  [ ] install.sh re-run to verify symlink creation"
echo ""
