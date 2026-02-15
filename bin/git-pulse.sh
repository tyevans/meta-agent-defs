#!/usr/bin/env bash
# git-pulse.sh: Session commit metrics in plaintext format
# Usage: git-pulse.sh [--since="time spec"]
set -euo pipefail

# Parse arguments
SINCE_FLAG=""
SINCE_VALUE=""
[[ $# -gt 0 && $1 == --since=* ]] && SINCE_FLAG="--since=${1#*=}" && SINCE_VALUE="${1#*=}"

# Convert human-readable dates to git-intel relative format for the git-intel path
# git-intel accepts Nd/Nw/Nm/Ny but not "N hours ago" etc.
INTEL_SINCE_VALUE="$SINCE_VALUE"
if [[ "$SINCE_VALUE" =~ ^([0-9]+)\ hours?\ ago$ ]]; then
  INTEL_SINCE_VALUE="1d"  # Round up to 1 day for git-intel
elif [[ "$SINCE_VALUE" =~ ^([0-9]+)\ days?\ ago$ ]]; then
  INTEL_SINCE_VALUE="${BASH_REMATCH[1]}d"
elif [[ "$SINCE_VALUE" =~ ^([0-9]+)\ weeks?\ ago$ ]]; then
  INTEL_SINCE_VALUE="${BASH_REMATCH[1]}w"
elif [[ "$SINCE_VALUE" =~ ^([0-9]+)\ months?\ ago$ ]]; then
  INTEL_SINCE_VALUE="${BASH_REMATCH[1]}m"
fi

# Fail gracefully if not in a git repo
if ! git rev-parse --git-dir > /dev/null 2>&1; then
  echo "commits: 0" && echo "Not a git repository" >&2 && exit 0
fi

# Check if git-intel and jq are available
# Try relative path first, then derive from script's own location, then legacy hardcoded path
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
META_AGENT_DIR="$(dirname "$SCRIPT_DIR")"
GIT_INTEL_PATH="tools/git-intel/target/release/git-intel"
if [ ! -f "$GIT_INTEL_PATH" ]; then
  GIT_INTEL_PATH="$META_AGENT_DIR/tools/git-intel/target/release/git-intel"
fi
USE_GIT_INTEL=false
if [ -f "$GIT_INTEL_PATH" ] && command -v jq >/dev/null 2>&1; then
  # Use git-intel if --since is ISO format, relative, convertible from human-readable, or not provided
  if [ -z "$INTEL_SINCE_VALUE" ] || [[ "$INTEL_SINCE_VALUE" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || [[ "$INTEL_SINCE_VALUE" =~ ^[0-9]+[dwmy]$ ]]; then
    USE_GIT_INTEL=true
  fi
fi
if [ "$USE_GIT_INTEL" = true ]; then
  # Check for ONNX model and ML-capable git-intel
  ML_FLAGS=""
  REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || echo ".")
  ONNX_MODEL_DIR="$REPO_ROOT/tools/data/onnx-model"
  if [ ! -f "$ONNX_MODEL_DIR/model.onnx" ]; then
    ONNX_MODEL_DIR="$META_AGENT_DIR/tools/data/onnx-model"
  fi
  if [ -f "$ONNX_MODEL_DIR/model.onnx" ] && \
     $GIT_INTEL_PATH --help 2>&1 | grep -q '\-\-ml'; then
    # Runtime test: verify ML feature works (may fail if ONNX Runtime lib missing)
    TEST_JSON=$($GIT_INTEL_PATH metrics --repo . --ml --model-dir "$ONNX_MODEL_DIR" --since 365d 2>/dev/null || echo "{}")
    if echo "$TEST_JSON" | jq -e '.total_commits' >/dev/null 2>&1; then
      ML_FLAGS="--ml --model-dir $ONNX_MODEL_DIR"
    fi
  fi

  # Use git-intel for metrics and churn
  METRICS_JSON=$($GIT_INTEL_PATH metrics --repo . $ML_FLAGS ${INTEL_SINCE_VALUE:+--since "$INTEL_SINCE_VALUE"} 2>/dev/null || echo "{}")
  CHURN_JSON=$($GIT_INTEL_PATH churn --repo . --limit 5 $ML_FLAGS ${INTEL_SINCE_VALUE:+--since "$INTEL_SINCE_VALUE"} 2>/dev/null || echo '{"files":[]}')

  # Extract metrics with jq
  TOTAL=$(echo "$METRICS_JSON" | jq -r '.total_commits // 0')
  FEAT=$(echo "$METRICS_JSON" | jq -r '[.commit_types[] | select(.type == "feat")] | .[0].count // 0')
  FIX=$(echo "$METRICS_JSON" | jq -r '[.commit_types[] | select(.type == "fix")] | .[0].count // 0')
  CHORE=$(echo "$METRICS_JSON" | jq -r '[.commit_types[] | select(.type == "chore")] | .[0].count // 0')
  DOCS=$(echo "$METRICS_JSON" | jq -r '[.commit_types[] | select(.type == "docs")] | .[0].count // 0')
  REFACTOR=$(echo "$METRICS_JSON" | jq -r '[.commit_types[] | select(.type == "refactor")] | .[0].count // 0')

  # Calculate fix rate
  FIX_RATE=0
  [[ $TOTAL -gt 0 ]] && FIX_RATE=$(awk "BEGIN {printf \"%.0f\", ($FIX / $TOTAL) * 100}")

  # Output results
  echo "commits: $TOTAL"
  echo "feat: $FEAT"
  echo "fix: $FIX"
  echo "chore: $CHORE"
  echo "docs: $DOCS"
  echo "refactor: $REFACTOR"
  echo "fix_rate: ${FIX_RATE}%"

  # Output churn (top 5 from git-intel)
  CHURN_COUNT=$(echo "$CHURN_JSON" | jq -r '.files | length')
  if [ "$CHURN_COUNT" -gt 0 ]; then
    for i in $(seq 1 $CHURN_COUNT); do
      idx=$((i - 1))
      path=$(echo "$CHURN_JSON" | jq -r ".files[$idx].path")
      total=$(echo "$CHURN_JSON" | jq -r ".files[$idx].total_churn")
      echo "churn_$i: $path ($total)"
    done
  fi

  # Output signal count from patterns (if available)
  PATTERNS_JSON=$($GIT_INTEL_PATH patterns --repo . $ML_FLAGS ${INTEL_SINCE_VALUE:+--since "$INTEL_SINCE_VALUE"} 2>/dev/null || echo '{"signals":[]}')
  SIGNAL_COUNT=$(echo "$PATTERNS_JSON" | jq -r '.signals | length')
  if [ "$SIGNAL_COUNT" -gt 0 ]; then
    echo "signals: $SIGNAL_COUNT"
  fi
else
  # Fall back to raw git implementation
  # Get commits with optional time filter
  COMMITS=$(git log --oneline ${SINCE_FLAG:+"$SINCE_FLAG"} 2>/dev/null || true)
  TOTAL=$(echo "$COMMITS" | wc -l)
  [[ -z "$COMMITS" ]] && TOTAL=0
  # Count by type (conventional commit prefixes)
  FEAT=$(echo "$COMMITS" | grep -cE "^[a-f0-9]* feat(\(.*\))?:" || true)
  FIX=$(echo "$COMMITS" | grep -cE "^[a-f0-9]* fix(\(.*\))?:" || true)
  CHORE=$(echo "$COMMITS" | grep -cE "^[a-f0-9]* chore(\(.*\))?:" || true)
  DOCS=$(echo "$COMMITS" | grep -cE "^[a-f0-9]* docs(\(.*\))?:" || true)
  REFACTOR=$(echo "$COMMITS" | grep -cE "^[a-f0-9]* refactor(\(.*\))?:" || true)
  # Calculate fix rate
  FIX_RATE=0
  [[ $TOTAL -gt 0 ]] && FIX_RATE=$(awk "BEGIN {printf \"%.0f\", ($FIX / $TOTAL) * 100}")
  # Get top-5 churning files (lines added + removed)
  CHURN=$(git log --numstat ${SINCE_FLAG:+"$SINCE_FLAG"} 2>/dev/null | \
    awk 'NF==3 {added[$3]+=$1; removed[$3]+=$2} END {for(f in added) print added[f]+removed[f], f}' | \
    sort -rn | head -5)
  # Output results
  echo "commits: $TOTAL"
  echo "feat: $FEAT"
  echo "fix: $FIX"
  echo "chore: $CHORE"
  echo "docs: $DOCS"
  echo "refactor: $REFACTOR"
  echo "fix_rate: ${FIX_RATE}%"
  # Output churn (top 5)
  if [[ -n "$CHURN" ]]; then
    i=1
    while IFS= read -r line; do
      [[ -n "$line" ]] && echo "churn_$i: $(echo "$line" | cut -d' ' -f2-) ($(echo "$line" | awk '{print $1}'))" && i=$((i + 1))
    done <<< "$CHURN"
  fi
fi
