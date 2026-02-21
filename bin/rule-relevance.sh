#!/usr/bin/env bash
# Advisory rule relevance signal for SessionStart hook.
# Reads recent git activity and beads backlog to determine which
# rules are most relevant to the current session's likely work domains.
# Output goes to stdout and appears as session context.

set -euo pipefail

# Collect recent file activity signals
RECENT_FILES=$(git diff --name-only HEAD~5..HEAD 2>/dev/null || true)
STAGED_FILES=$(git diff --name-only --cached 2>/dev/null || true)
DIRTY_FILES=$(git diff --name-only 2>/dev/null || true)
ALL_FILES=$(printf '%s\n%s\n%s' "$RECENT_FILES" "$STAGED_FILES" "$DIRTY_FILES" | sort -u | grep -v '^$' || true)

if [ -z "$ALL_FILES" ]; then
  exit 0  # No file signals, skip advisory
fi

# Check each rule file for paths: frontmatter and match against active files
RELEVANT=""
for rule_file in rules/*.md .claude/rules/*.md; do
  [ -f "$rule_file" ] || continue

  # Extract paths: patterns from YAML frontmatter
  PATHS=$(sed -n '/^---$/,/^---$/{ /^  - "/{ s/^  - "//; s/"$//; p; } }' "$rule_file" 2>/dev/null || true)
  [ -z "$PATHS" ] && continue

  # Extract rule title
  TITLE=$(grep -m1 '^# ' "$rule_file" 2>/dev/null | sed 's/^# //' || echo "$rule_file")

  # Extract strength
  STRENGTH=$(sed -n '/^---$/,/^---$/{ /^strength:/{ s/^strength: //; p; } }' "$rule_file" 2>/dev/null || echo "unknown")

  # Check if any active files match any paths: pattern
  MATCHED=false
  while IFS= read -r pattern; do
    [ -z "$pattern" ] && continue
    # Use bash glob matching against each active file
    while IFS= read -r file; do
      [ -z "$file" ] && continue
      # Convert glob to regex for matching
      REGEX=$(echo "$pattern" | sed 's/\*\*/DOUBLESTAR/g; s/\*/[^\/]*/g; s/DOUBLESTAR/.*/g')
      if echo "$file" | grep -qE "^${REGEX}$" 2>/dev/null; then
        MATCHED=true
        break 2
      fi
    done <<< "$ALL_FILES"
  done <<< "$PATHS"

  if [ "$MATCHED" = true ]; then
    RELEVANT="${RELEVANT}  - ${TITLE} (${rule_file}, strength: ${STRENGTH})\n"
  fi
done

if [ -n "$RELEVANT" ]; then
  echo "Rules relevant to current file activity:"
  printf "$RELEVANT"
fi
