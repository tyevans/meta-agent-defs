#!/usr/bin/env bash
# session-start.sh: Session orientation card for Claude Code SessionStart hook
# Outputs a compact status card with tree state, backlog, hot files, and suggestions
set -euo pipefail

# Derive paths from this script's location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
META_AGENT_DIR="$(dirname "$SCRIPT_DIR")"

echo '=== Session Start ==='

# --- Tree state ---
DIRTY=$(git status --porcelain 2>/dev/null || true)
if [ -n "$DIRTY" ]; then
  DCOUNT=$(echo "$DIRTY" | wc -l | tr -d ' ')
  echo "Tree:    $DCOUNT uncommitted files"
else
  echo 'Tree:    clean'
fi

# --- Recent commits ---
LOGS=$(git log --oneline -3 --format='%s' 2>/dev/null | cut -c1-30 | tr '\n' ',' | sed 's/,/, /g; s/, $//')
echo "Last:    ${LOGS:-no commits}"

# --- Beads backlog ---
STATS=$(bd stats 2>/dev/null || true)
IP=$(echo "$STATS" | grep 'In Progress' | awk '{print $NF}')
RD=$(echo "$STATS" | grep 'Ready' | awk '{print $NF}')
echo "Open:    ${IP:-0} in-progress, ${RD:-0} ready"

# --- Hot files (file volatility) ---
# Find git-intel: try project-local first, then derive from script location
GI=''
if [ -f tools/git-intel/target/release/git-intel ]; then
  GI=tools/git-intel/target/release/git-intel
elif [ -f "$META_AGENT_DIR/tools/git-intel/target/release/git-intel" ]; then
  GI="$META_AGENT_DIR/tools/git-intel/target/release/git-intel"
fi

if [ -n "$GI" ]; then
  HOT=$($GI churn --repo . --limit 3 2>/dev/null | \
    python3 -c "import sys,json; data=json.load(sys.stdin)['files']; print(', '.join(f'{d[\"path\"].split(\"/\")[-1]}({d[\"commit_count\"]})' for d in data))" 2>/dev/null || true)
else
  HOT=$(git log --pretty=format: --name-only -10 2>/dev/null | \
    grep -v '^$' | sort | uniq -c | sort -rn | head -3 | \
    awk '{name=$2; split(name,a,"/"); print a[length(a)]"("$1")"}' | \
    paste -sd ',' - | sed 's/,/, /g' || true)
fi
if [ -n "$HOT" ]; then
  echo "Hot:     $HOT"
fi

# --- Team info ---
if [ -f .claude/team.yaml ]; then
  TN=$(grep '^team:' .claude/team.yaml 2>/dev/null | head -1 | sed 's/^team: *//')
  MC=$(grep '^ *- name:' .claude/team.yaml 2>/dev/null | wc -l | tr -d ' ')
  echo "Team:    ${TN:-unknown} ($MC members)"
fi

# --- Next suggestion ---
READY_TITLE=""
if [ -n "$DIRTY" ]; then
  echo 'Next:    review uncommitted changes from previous session'
else
  READY_TITLE=$(bd ready --limit 1 2>/dev/null | grep '^1\.' | sed 's/^[^:]*: //' | head -1 || true)
  if [ -n "$READY_TITLE" ]; then
    echo "Next:    $READY_TITLE"
  else
    echo 'Next:    backlog clear — create new work or explore'
  fi
fi

# --- Skill suggestion ---
if [ -n "$DIRTY" ]; then
  echo 'Skill:   /review (uncommitted changes detected)'
elif [ -n "$READY_TITLE" ]; then
  echo 'Skill:   /sprint (ready work available)'
else
  echo 'Skill:   /blossom or /status (explore or orient)'
fi

echo '=== ==='

# --- Unpushed commits warning ---
UNPUSHED=$(git log @{u}.. --oneline 2>/dev/null || true)
if [ -n "$UNPUSHED" ]; then
  UPCOUNT=$(echo "$UNPUSHED" | wc -l | tr -d ' ')
  echo "NOTE: $UPCOUNT unpushed commits"
fi

# --- Last session context ---
if [ -f memory/sessions/last.md ]; then
  echo '--- Last Session ---'
  cat memory/sessions/last.md
  echo '---'
fi

# --- Prime beads ---
bd prime 2>/dev/null || true
