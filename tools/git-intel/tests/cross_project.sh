#!/usr/bin/env bash
# cross_project.sh — Run git-intel subcommands against all bare repos in tools/repos/
# Produces a pass/fail summary table. Exits nonzero if any test failed.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GI_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPOS_DIR="$(cd "$GI_DIR/../repos" && pwd)"
GI="$GI_DIR/target/release/git-intel"
CMDS=(metrics churn patterns hotspots authors trends)
SINCE="365d"

# Colors (disabled if not a terminal)
if [[ -t 1 ]]; then
  G='\033[32m' R='\033[31m' B='\033[1m' Z='\033[0m'
else
  G='' R='' B='' Z=''
fi

# Build if needed
if [[ ! -x "$GI" ]]; then
  echo "Building git-intel..."
  (cd "$GI_DIR" && cargo build --release --features ml 2>/dev/null || cargo build --release)
fi

# Discover repos
mapfile -t REPOS < <(find "$REPOS_DIR" -maxdepth 1 -name '*.git' -type d | sort)
[[ ${#REPOS[@]} -eq 0 ]] && { echo "No bare repos found in $REPOS_DIR" >&2; exit 1; }

echo -e "${B}Cross-project test harness${Z}  (${#REPOS[@]} repos x ${#CMDS[@]} cmds, --since $SINCE)"
echo ""

declare -A res errs
total=0; passed=0; failed=0

for repo in "${REPOS[@]}"; do
  name=$(basename "$repo" .git)
  echo -e "${B}--- $name ---${Z}"
  for cmd in "${CMDS[@]}"; do
    key="$name:$cmd"; total=$((total + 1))
    tmpout=$(mktemp); tmperr=$(mktemp)
    t0=$(($(date +%s%N)/1000000))
    "$GI" --repo "$repo" --since "$SINCE" "$cmd" >"$tmpout" 2>"$tmperr" && ec=0 || ec=$?
    ms=$(( $(date +%s%N)/1000000 - t0 ))
    sz=$(wc -c < "$tmpout")
    if [[ $ec -ne 0 ]]; then
      err=$(head -1 "$tmperr" | cut -c1-80)
      res[$key]="FAIL"; errs[$key]="exit=$ec $err"; failed=$((failed+1))
      echo -e "  $cmd: ${R}FAIL${Z} (${ms}ms) $err"
    elif ! jq . "$tmpout" >/dev/null 2>&1; then
      res[$key]="FAIL"; errs[$key]="invalid JSON"; failed=$((failed+1))
      echo -e "  $cmd: ${R}FAIL${Z} (${ms}ms) invalid JSON"
    else
      res[$key]="PASS"; passed=$((passed+1)); extra=""
      if [[ "$cmd" == "patterns" ]]; then
        sc=$(jq '[.fix_after_feat//[],.fix_after_refactor//[],.multi_edit_chains//[],.temporal_clusters//[]]|map(length)|add//0' "$tmpout" 2>/dev/null||echo "?")
        extra=" signals=$sc"
      fi
      echo -e "  $cmd: ${G}PASS${Z} (${ms}ms) ${sz}B${extra}"
    fi
    rm -f "$tmpout" "$tmperr"
  done
done

# Summary table
echo ""
echo -e "${B}=== SUMMARY ===${Z}"
printf "%-16s" "REPO"
for c in "${CMDS[@]}"; do printf "%-12s" "$c"; done; echo ""
printf '%0.s-' {1..88}; echo ""
for repo in "${REPOS[@]}"; do
  name=$(basename "$repo" .git); printf "%-16s" "$name"
  for c in "${CMDS[@]}"; do
    k="$name:$c"
    [[ "${res[$k]:-}" == "PASS" ]] && printf "${G}%-12s${Z}" "PASS" || printf "${R}%-12s${Z}" "FAIL"
  done; echo ""
done

echo ""
echo -e "${B}Total: $total  Passed: ${G}$passed${Z}  Failed: ${R}$failed${Z}"

if [[ $failed -gt 0 ]]; then
  echo -e "\n${B}Failed tests:${Z}"
  for repo in "${REPOS[@]}"; do
    name=$(basename "$repo" .git)
    for c in "${CMDS[@]}"; do
      k="$name:$c"
      [[ "${res[$k]:-}" == "FAIL" ]] && echo "  $name/$c: ${errs[$k]:-unknown}"
    done
  done
  exit 1
fi
