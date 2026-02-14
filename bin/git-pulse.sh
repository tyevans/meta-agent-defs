#!/usr/bin/env bash
# git-pulse.sh: Session commit metrics in plaintext format
# Usage: git-pulse.sh [--since="time spec"]
set -euo pipefail
# Parse arguments
SINCE_FLAG=""
[[ $# -gt 0 && $1 == --since=* ]] && SINCE_FLAG="--since=${1#*=}"
# Fail gracefully if not in a git repo
if ! git rev-parse --git-dir > /dev/null 2>&1; then
  echo "commits: 0" && echo "Not a git repository" >&2 && exit 0
fi
# Get commits with optional time filter
COMMITS=$(git log --oneline ${SINCE_FLAG:+"$SINCE_FLAG"} 2>/dev/null || true)
TOTAL=$(echo "$COMMITS" | wc -l)
[[ -z "$COMMITS" ]] && TOTAL=0
# Count by type (conventional commit prefixes)
FEAT=$(echo "$COMMITS" | grep -c "^[a-f0-9]* feat:" || true)
FIX=$(echo "$COMMITS" | grep -c "^[a-f0-9]* fix:" || true)
CHORE=$(echo "$COMMITS" | grep -c "^[a-f0-9]* chore:" || true)
DOCS=$(echo "$COMMITS" | grep -c "^[a-f0-9]* docs:" || true)
REFACTOR=$(echo "$COMMITS" | grep -c "^[a-f0-9]* refactor:" || true)
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
