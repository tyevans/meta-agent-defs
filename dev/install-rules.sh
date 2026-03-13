#!/usr/bin/env bash
# install-rules.sh — install tackline global rules without a local checkout
#
# Usage (no local clone needed):
#   curl -fsSL https://raw.githubusercontent.com/tyevans/tackline/main/dev/install-rules.sh | bash
#
# Or from a local checkout:
#   bash dev/install-rules.sh

set -euo pipefail

REPO="tyevans/tackline"
BRANCH="main"
BASE_URL="https://raw.githubusercontent.com/${REPO}/${BRANCH}/rules"
RULES_DIR="${HOME}/.claude/rules"

RULES=(
  batch-safety.md
  context-trust.md
  delegation.md
  memory-layout.md
  pipe-format.md
  team-protocol.md
  test-conventions.md
)

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

log()  { echo -e "${GREEN}[+]${NC} $1"; }
fail() { echo -e "${RED}[x]${NC} $1"; }

mkdir -p "${RULES_DIR}"

PASS=0
FAIL=0

for rule in "${RULES[@]}"; do
  url="${BASE_URL}/${rule}"
  dest="${RULES_DIR}/${rule}"
  if curl -fsSL "${url}" -o "${dest}"; then
    log "installed ${rule}"
    PASS=$((PASS + 1))
  else
    fail "failed to fetch ${rule} from ${url}"
    FAIL=$((FAIL + 1))
  fi
done

echo ""
if [ "${FAIL}" -eq 0 ]; then
  log "${PASS} rules installed to ${RULES_DIR}"
else
  fail "${PASS} installed, ${FAIL} failed"
  exit 1
fi
