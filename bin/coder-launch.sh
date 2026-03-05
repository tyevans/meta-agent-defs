#!/usr/bin/env bash
# coder-launch.sh — Launch an agent in a Coder workspace
#
# Usage:
#   coder-launch.sh [options]
#
# Options:
#   --name NAME          Workspace name (default: auto-generated)
#   --repo URL           Git repository to clone
#   --agent-type TYPE    claude-code | seam-agent | custom (default: claude-code)
#   --template NAME      Coder template name (default: agent-workspace)
#   --ttl HOURS          Auto-stop after N hours (default: 2)
#   --task TEXT           Create a Coder Task with this prompt
#   --wait               Wait for workspace to be ready before returning
#   --exec CMD           Execute command after workspace is ready
#
# Examples:
#   coder-launch.sh --repo https://github.com/org/repo --agent-type claude-code
#   coder-launch.sh --name review-pr-42 --repo git@github.com:org/repo --exec "claude 'review the last PR'"
#   coder-launch.sh --task "Fix the failing test in src/auth.py"

set -euo pipefail

# Defaults
WS_NAME=""
REPO_URL=""
AGENT_TYPE="claude-code"
TEMPLATE="agent-workspace"
TTL_HOURS=2
TASK_PROMPT=""
WAIT=false
EXEC_CMD=""

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --name)       WS_NAME="$2"; shift 2 ;;
    --repo)       REPO_URL="$2"; shift 2 ;;
    --agent-type) AGENT_TYPE="$2"; shift 2 ;;
    --template)   TEMPLATE="$2"; shift 2 ;;
    --ttl)        TTL_HOURS="$2"; shift 2 ;;
    --task)       TASK_PROMPT="$2"; shift 2 ;;
    --wait)       WAIT=true; shift ;;
    --exec)       EXEC_CMD="$2"; shift 2 ;;
    -h|--help)
      sed -n '2,/^$/{ s/^# //; s/^#//; p }' "$0"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

# Verify coder CLI is available
if ! command -v coder &>/dev/null; then
  echo "Error: coder CLI not found. Install from https://coder.com/docs/install" >&2
  exit 1
fi

# Generate workspace name if not provided
if [ -z "$WS_NAME" ]; then
  WS_NAME="agent-$(date +%s | tail -c 7)"
fi

# Validate workspace name (max 32 chars, alphanumeric + hyphens)
WS_NAME=$(echo "$WS_NAME" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9-]/-/g' | head -c 32)

echo "Creating workspace: $WS_NAME"
echo "  Template:   $TEMPLATE"
echo "  Agent type: $AGENT_TYPE"
echo "  Repo:       ${REPO_URL:-none}"
echo "  TTL:        ${TTL_HOURS}h"

# Create workspace with parameters
PARAMS=(
  --template "$TEMPLATE"
  --parameter "agent_type=$AGENT_TYPE"
  --parameter "ttl_hours=$TTL_HOURS"
)

if [ -n "$REPO_URL" ]; then
  PARAMS+=(--parameter "repo_url=$REPO_URL")
fi

coder create "$WS_NAME" "${PARAMS[@]}" --yes

# If --task was provided, create a Coder Task
if [ -n "$TASK_PROMPT" ]; then
  echo "Creating task..."
  coder task create --workspace "$WS_NAME" --prompt "$TASK_PROMPT"
  echo "Task created. Monitor with: coder task logs $WS_NAME"
  exit 0
fi

# Wait for workspace to be ready
if [ "$WAIT" = true ] || [ -n "$EXEC_CMD" ]; then
  echo "Waiting for workspace to be ready..."
  coder ssh "$WS_NAME" --wait yes -- echo "Workspace ready"
fi

# Execute command if provided
if [ -n "$EXEC_CMD" ]; then
  echo "Executing: $EXEC_CMD"
  coder ssh "$WS_NAME" -- bash -c "$EXEC_CMD"
else
  echo ""
  echo "Workspace ready. Connect with:"
  echo "  coder ssh $WS_NAME"
  echo "  coder ssh $WS_NAME -- <command>"
  echo ""
  echo "Stop workspace:"
  echo "  coder stop $WS_NAME"
fi
