#!/usr/bin/env bash
# pipeline.sh — Clone, extract, stride-sample, enrich, label, and train commit classifier.
# Fire and forget. Resumable at every stage.
#
# Usage:
#   ./pipeline.sh                    # full run (clone + extract + enrich + label + train)
#   ./pipeline.sh --skip-clone       # reuse existing clones
#   ./pipeline.sh --skip-extract     # reuse existing extracts
#   ./pipeline.sh --skip-enrich      # skip enrichment, label raw strided data
#   ./pipeline.sh --label-only       # just run labeler on existing strided/enriched data
#   ./pipeline.sh --train-only       # just run training on existing labeled data
#
# Requirements:
#   - git, python3, uv
#   - Ollama server reachable (default: 192.168.1.14:11434)

set -euo pipefail

# --- Config ---
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="$SCRIPT_DIR/../data"
REPOS_DIR="$SCRIPT_DIR/../repos"
COMMITS_PER_REPO=2000
SAMPLES_PER_BUCKET=100   # per repo-year
BATCH_SIZE=25
OLLAMA_SERVER="http://192.168.1.14:11434/v1"
OLLAMA_MODEL="gpt-oss:20b"

# Repos: org/repo across diverse ecosystems
REPOS=(
  # C# — media automation (existing dataset)
  "Radarr/Radarr"
  "Sonarr/Sonarr"
  "Lidarr/Lidarr"
  "Prowlarr/Prowlarr"
  "tidusjar/Ombi"
  "Jackett/Jackett"
  # Python — web frameworks, high contributor diversity
  "django/django"
  "pallets/flask"
  "tiangolo/fastapi"
  # Ruby — mature, ticket-heavy commit styles
  "rails/rails"
  "jekyll/jekyll"
  # Rust — conventional-commit heavy
  "BurntSushi/ripgrep"
  "tokio-rs/tokio"
  # PHP — massive contributor pools
  "laravel/framework"
  "WordPress/WordPress"
  # Go — enterprise-style messages
  "gohugoio/hugo"
  "junegunn/fzf"
  # Perl — old-school terse messages
  "Perl/perl5"
)

# --- Parse flags ---
SKIP_CLONE=false
SKIP_EXTRACT=false
SKIP_ENRICH=false
LABEL_ONLY=false
TRAIN_ONLY=false

for arg in "$@"; do
  case "$arg" in
    --skip-clone)   SKIP_CLONE=true ;;
    --skip-extract) SKIP_EXTRACT=true; SKIP_CLONE=true ;;
    --skip-enrich)  SKIP_ENRICH=true ;;
    --label-only)   LABEL_ONLY=true; SKIP_EXTRACT=true; SKIP_CLONE=true; SKIP_ENRICH=true ;;
    --train-only)   TRAIN_ONLY=true; SKIP_EXTRACT=true; SKIP_CLONE=true; SKIP_ENRICH=true ;;
    --help|-h)
      head -14 "$0" | tail -12
      exit 0
      ;;
    *)
      echo "Unknown flag: $arg"
      exit 1
      ;;
  esac
done

mkdir -p "$DATA_DIR" "$REPOS_DIR"

# --- Phase 1: Clone ---
if [ "$SKIP_CLONE" = false ]; then
  echo "=== Phase 1: Cloning ${#REPOS[@]} repos ==="
  for repo in "${REPOS[@]}"; do
    name=$(basename "$repo")
    dest="$REPOS_DIR/$name.git"
    if [ -d "$dest" ]; then
      echo "  [skip] $name (already cloned)"
    else
      echo "  [clone] $name ..."
      git clone --bare --filter=blob:none "https://github.com/$repo.git" "$dest" 2>&1 | tail -1
    fi
  done
  repo_count=$(find "$REPOS_DIR" -maxdepth 1 -name '*.git' -type d | wc -l)
  echo "Done: $repo_count repos cloned"
fi

# --- Phase 2: Extract commits ---
RAW_FILE="$DATA_DIR/all-commits.txt"

if [ "$SKIP_EXTRACT" = false ]; then
  echo ""
  echo "=== Phase 2: Extracting commits ==="
  > "$RAW_FILE"  # truncate
  for repo in "${REPOS[@]}"; do
    name=$(basename "$repo")
    dest="$REPOS_DIR/$name.git"
    if [ ! -d "$dest" ]; then
      echo "  [warn] $name not found, skipping"
      continue
    fi
    git -C "$dest" log --no-merges \
      --format="$name|||%H|||%an|||%ae|||%aI|||%s" \
      -n "$COMMITS_PER_REPO" 2>/dev/null >> "$RAW_FILE"
    echo "  $name: extracted"
  done
  total=$(wc -l < "$RAW_FILE")
  echo "Done: $total total commits in $RAW_FILE"
fi

# --- Phase 3: Time-strided sampling ---
STRIDED_FILE="$DATA_DIR/strided-all.txt"

if [ "$LABEL_ONLY" = false ] && [ "$TRAIN_ONLY" = false ]; then
  echo ""
  echo "=== Phase 3: Time-strided sampling ($SAMPLES_PER_BUCKET per repo-year) ==="
  python3 - "$RAW_FILE" "$STRIDED_FILE" "$SAMPLES_PER_BUCKET" <<'PYEOF'
import sys, random
from collections import defaultdict

raw_file, out_file, samples = sys.argv[1], sys.argv[2], int(sys.argv[3])
random.seed(42)

buckets = defaultdict(list)
with open(raw_file) as f:
    for line in f:
        line = line.strip()
        if not line:
            continue
        parts = line.split("|||")
        if len(parts) < 6:
            continue
        repo, date = parts[0], parts[4]
        year = date[:4] if len(date) >= 4 else "unknown"
        buckets[(repo, year)].append(line)

selected = []
for key in sorted(buckets.keys()):
    pool = buckets[key]
    n = min(samples, len(pool))
    selected.extend(random.sample(pool, n))

with open(out_file, "w") as f:
    for line in selected:
        f.write(line + "\n")

repos = set(k[0] for k in buckets.keys())
print(f"  {len(buckets)} buckets across {len(repos)} repos")
print(f"  {len(selected)} commits sampled -> {out_file}")
PYEOF
fi

# --- Phase 4: Enrich with diffs ---
ENRICHED_FILE="$DATA_DIR/enriched-all.jsonl"

if [ "$SKIP_ENRICH" = false ] && [ "$LABEL_ONLY" = false ] && [ "$TRAIN_ONLY" = false ]; then
  echo ""
  echo "=== Phase 4: Enriching with diff metadata ==="

  # Skip if enriched file exists and is newer than strided
  if [ -f "$ENRICHED_FILE" ] && [ "$ENRICHED_FILE" -nt "$STRIDED_FILE" ]; then
    echo "  [skip] Enriched file exists and is up to date"
  else
    echo "  Input:  $STRIDED_FILE"
    echo "  Output: $ENRICHED_FILE"
    echo "  Repos:  $REPOS_DIR"

    cd "$SCRIPT_DIR"
    uv run python enrich_diffs.py "$STRIDED_FILE" "$REPOS_DIR" -o "$ENRICHED_FILE"
  fi
fi

# --- Phase 5: Label via Ollama ---
LABELED_FILE="$DATA_DIR/labeled-all.jsonl"

if [ "$TRAIN_ONLY" = false ]; then
  echo ""
  echo "=== Phase 5: Labeling via Ollama ($OLLAMA_MODEL) ==="

  # Determine input file based on enrichment
  if [ "$SKIP_ENRICH" = false ] && [ -f "$ENRICHED_FILE" ]; then
    LABEL_INPUT="$ENRICHED_FILE"
    LABEL_FLAGS="--enriched"
    echo "  Input:  $LABEL_INPUT (enriched)"
  else
    LABEL_INPUT="$STRIDED_FILE"
    LABEL_FLAGS=""
    echo "  Input:  $LABEL_INPUT (raw)"
  fi

  echo "  Output: $LABELED_FILE"
  echo "  Batch:  $BATCH_SIZE"

  # Check Ollama is reachable
  if ! curl -sf "${OLLAMA_SERVER%/v1}/api/tags" > /dev/null 2>&1; then
    echo "  ERROR: Ollama server not reachable at $OLLAMA_SERVER"
    echo "  Skipping labeling. Run again when server is up."
    exit 1
  fi

  cd "$SCRIPT_DIR"
  uv run python label.py "$LABEL_INPUT" \
    -o "$LABELED_FILE" \
    $LABEL_FLAGS \
    --model "$OLLAMA_MODEL" \
    --server "$OLLAMA_SERVER" \
    --batch-size "$BATCH_SIZE" \
    --think low
fi

# --- Phase 6: Train ---
MODEL_FILE="$DATA_DIR/classifier.pkl"

echo ""
echo "=== Phase 6: Training classifier ==="
echo "  Input:  $LABELED_FILE"
echo "  Output: $MODEL_FILE"

cd "$SCRIPT_DIR"
uv run python train.py "$LABELED_FILE" --model tfidf-logreg

echo ""
echo "Note: Run with --model embed-mlp to train the embedding-based model"

# --- Phase 7: Summary ---
echo ""
echo "=== Phase 7: Summary ==="
python3 - "$LABELED_FILE" <<'PYEOF'
import json, sys
from collections import Counter

path = sys.argv[1]
primary_labels = Counter()
repos = Counter()
all_labels = []
label_counts = []

with open(path) as f:
    for line in f:
        obj = json.loads(line.strip())

        # Handle multi-label format
        if "labels" in obj and obj["labels"]:
            # Count primary (first) label
            primary = obj["labels"][0]["label"]
            primary_labels[primary] += 1

            # Track all labels for this commit
            label_counts.append(len(obj["labels"]))

            # Collect all label confidences
            for lbl in obj["labels"]:
                all_labels.append((lbl["label"], lbl["confidence"]))
        # Fallback for old single-label format
        elif "label" in obj:
            primary_labels[obj["label"]] += 1
            label_counts.append(1)
            if "confidence" in obj:
                all_labels.append((obj["label"], obj["confidence"]))

        repos[obj["repo"]] += 1

total = sum(primary_labels.values())
print(f"Total: {total} commits labeled")

print(f"\nPrimary label distribution:")
for k, v in primary_labels.most_common():
    print(f"  {k:12s}: {v:4d} ({100*v/total:.1f}%)")

if label_counts:
    avg_labels = sum(label_counts) / len(label_counts)
    print(f"\nMulti-label stats:")
    print(f"  Average labels per commit: {avg_labels:.2f}")
    print(f"  Single-label commits: {sum(1 for c in label_counts if c == 1)} ({100*sum(1 for c in label_counts if c == 1)/len(label_counts):.1f}%)")
    print(f"  Multi-label commits:  {sum(1 for c in label_counts if c > 1)} ({100*sum(1 for c in label_counts if c > 1)/len(label_counts):.1f}%)")

print(f"\nBy repo:")
for k, v in repos.most_common():
    print(f"  {k:15s}: {v:4d}")

if all_labels:
    confs = [c for _, c in all_labels]
    print(f"\nConfidence stats:")
    print(f"  min={min(confs):.2f}  avg={sum(confs)/len(confs):.2f}  max={max(confs):.2f}")
    print(f"  Low confidence (<0.7): {sum(1 for c in confs if c < 0.7)} ({100*sum(1 for c in confs if c < 0.7)/len(confs):.1f}%)")
PYEOF

echo ""
echo "Done. Results at: $LABELED_FILE"
echo "      Model at:   $MODEL_FILE"
