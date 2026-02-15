"""Enrich commits with diff metadata from bare git clones.

Extracts file paths changed, insertions/deletions, and file extension
distribution for each commit. Outputs enriched JSONL.

Supports both input formats:
- Pipe-delimited text (repo|||hash|||author|||email|||date|||message)
- JSONL (labeled commits)

Usage:
    # Enrich raw strided commits
    uv run python enrich_diffs.py ../data/strided-all.txt ../repos/ -o ../data/enriched-all.jsonl

    # Enrich already-labeled commits
    uv run python enrich_diffs.py ../data/labeled-all.jsonl ../repos/ -o ../data/labeled-enriched.jsonl
"""

from __future__ import annotations

import argparse
import json
import subprocess
from collections import Counter
from pathlib import Path


def get_diff_stats(repo_dir: Path, commit_hash: str) -> dict | None:
    """Extract diff stats from a bare git clone."""
    try:
        result = subprocess.run(
            ["git", "-C", str(repo_dir), "diff-tree", "--no-commit-id", "-r",
             "--numstat", commit_hash],
            capture_output=True, text=True, timeout=10,
        )
        if result.returncode != 0:
            return None

        files = []
        total_add = 0
        total_del = 0
        for line in result.stdout.strip().split("\n"):
            if not line:
                continue
            parts = line.split("\t")
            if len(parts) < 3:
                continue
            add = int(parts[0]) if parts[0] != "-" else 0
            delete = int(parts[1]) if parts[1] != "-" else 0
            filepath = parts[2]
            files.append(filepath)
            total_add += add
            total_del += delete

        if not files:
            return None

        # Extract file extensions
        extensions = []
        for f in files:
            if "." in f.split("/")[-1]:
                ext = f.rsplit(".", 1)[-1].lower()
                extensions.append(ext)

        # Extract directory paths (first component)
        top_dirs = [f.split("/")[0] if "/" in f else "." for f in files]

        return {
            "files": files,
            "file_count": len(files),
            "insertions": total_add,
            "deletions": total_del,
            "extensions": extensions,
            "top_dirs": top_dirs,
        }
    except (subprocess.TimeoutExpired, Exception):
        return None


def parse_commit_line(line: str) -> dict | None:
    """Parse pipe-delimited commit line into dict."""
    parts = line.strip().split("|||")
    if len(parts) < 6:
        return None
    return {
        "repo": parts[0],
        "hash": parts[1],
        "author": parts[2],
        "email": parts[3],
        "date": parts[4],
        "message": "|||".join(parts[5:]),
    }


def detect_format(path: Path) -> str:
    """Detect input format: 'jsonl' or 'pipe'."""
    with open(path) as f:
        first_line = f.readline().strip()
        if not first_line:
            return "pipe"
        # Try parsing as JSON
        try:
            json.loads(first_line)
            return "jsonl"
        except json.JSONDecodeError:
            return "pipe"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path, help="Input file (pipe-delimited or JSONL)")
    parser.add_argument("repos_dir", type=Path, help="Directory with bare git clones")
    parser.add_argument("-o", "--output", type=Path, required=True)
    args = parser.parse_args()

    # Map repo names to bare clone paths
    repo_dirs = {}
    for d in args.repos_dir.iterdir():
        if d.is_dir() and d.name.endswith(".git"):
            repo_dirs[d.name[:-4]] = d

    print(f"Found {len(repo_dirs)} repo clones: {sorted(repo_dirs.keys())}")

    # Detect input format
    input_format = detect_format(args.input)
    print(f"Detected input format: {input_format}")

    # Process commits
    enriched = 0
    skipped = 0
    missing_repo = Counter()

    with open(args.input) as fin, open(args.output, "w") as fout:
        for line in fin:
            line = line.strip()
            if not line:
                continue

            # Parse based on format
            if input_format == "jsonl":
                obj = json.loads(line)
            else:  # pipe-delimited
                obj = parse_commit_line(line)
                if not obj:
                    skipped += 1
                    continue

            repo = obj["repo"]
            commit_hash = obj["hash"]

            if repo not in repo_dirs:
                missing_repo[repo] += 1
                obj["diff"] = None
                fout.write(json.dumps(obj) + "\n")
                skipped += 1
                continue

            stats = get_diff_stats(repo_dirs[repo], commit_hash)
            obj["diff"] = stats
            fout.write(json.dumps(obj) + "\n")

            if stats:
                enriched += 1
            else:
                skipped += 1

            total = enriched + skipped
            if total % 500 == 0:
                print(f"  Processed {total}... ({enriched} enriched, {skipped} skipped)")

    total = enriched + skipped
    print(f"\nDone: {enriched}/{total} enriched ({100*enriched/total:.1f}%)")
    if missing_repo:
        print(f"Missing repos: {dict(missing_repo)}")
    print(f"Output: {args.output}")


if __name__ == "__main__":
    main()
