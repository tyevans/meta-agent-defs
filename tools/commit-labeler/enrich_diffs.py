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
import time
from collections import Counter
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

from data import parse_commit_line


def parse_numstat_block(lines: list[str]) -> dict | None:
    """Parse numstat lines for a single commit into diff stats."""
    files = []
    total_add = 0
    total_del = 0
    for line in lines:
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

    extensions = []
    for f in files:
        if "." in f.split("/")[-1]:
            ext = f.rsplit(".", 1)[-1].lower()
            extensions.append(ext)

    top_dirs = [f.split("/")[0] if "/" in f else "." for f in files]

    return {
        "files": files,
        "file_count": len(files),
        "insertions": total_add,
        "deletions": total_del,
        "extensions": extensions,
        "top_dirs": top_dirs,
    }


def batch_diff_stats(repo_dir: Path, hashes: list[str]) -> dict[str, dict | None]:
    """Get diff stats for many commits in one git call.

    Uses git log --stdin --no-walk --numstat with a delimiter to split output
    per commit. Returns a dict mapping hash -> stats.
    """
    if not hashes:
        return {}

    # Use a format that gives us hash + delimiter, then numstat follows
    try:
        result = subprocess.run(
            ["git", "-C", str(repo_dir), "log", "--stdin", "--no-walk",
             "--format=COMMIT_DELIM %H", "--numstat"],
            input="\n".join(hashes) + "\n",
            capture_output=True, text=True, timeout=120,
        )
    except subprocess.TimeoutExpired:
        return {}

    if result.returncode != 0:
        return {}

    # Parse output: split by COMMIT_DELIM lines
    stats = {}
    current_hash = None
    current_lines: list[str] = []

    for line in result.stdout.split("\n"):
        if line.startswith("COMMIT_DELIM "):
            # Flush previous commit
            if current_hash is not None:
                stats[current_hash] = parse_numstat_block(current_lines)
            current_hash = line.split(" ", 1)[1].strip()
            current_lines = []
        elif line.strip():
            current_lines.append(line)

    # Flush last commit
    if current_hash is not None:
        stats[current_hash] = parse_numstat_block(current_lines)

    return stats


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

    # Load all commits and group by repo
    commits = []
    parse_skipped = 0
    with open(args.input) as fin:
        for line in fin:
            line = line.strip()
            if not line:
                continue
            if input_format == "jsonl":
                commits.append(json.loads(line))
            else:
                obj = parse_commit_line(line)
                if obj:
                    commits.append(obj)
                else:
                    parse_skipped += 1

    print(f"Loaded {len(commits)} commits ({parse_skipped} parse failures)")

    # Group by repo
    by_repo: dict[str, list[int]] = {}
    for i, obj in enumerate(commits):
        by_repo.setdefault(obj["repo"], []).append(i)

    print(f"Repos to process: {len(by_repo)}")

    # Batch fetch diff stats per repo (parallel)
    enriched = 0
    skipped = 0
    missing_repo = Counter()

    # Separate missing repos from processable ones
    processable = {}
    for repo, indices in by_repo.items():
        if repo not in repo_dirs:
            missing_repo[repo] += len(indices)
            for i in indices:
                commits[i]["diff"] = None
            skipped += len(indices)
        else:
            processable[repo] = indices

    def process_repo(repo: str, indices: list[int]) -> tuple[str, dict[str, dict | None], list[int]]:
        hashes = [commits[i]["hash"] for i in indices]
        stats = batch_diff_stats(repo_dirs[repo], hashes)
        return repo, stats, indices

    t0 = time.monotonic()
    workers = min(6, len(processable))
    print(f"  Processing {len(processable)} repos with {workers} workers")

    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures = {
            pool.submit(process_repo, repo, indices): repo
            for repo, indices in processable.items()
        }
        done_count = 0
        for future in as_completed(futures):
            repo, stats, indices = future.result()
            done_count += 1
            elapsed = time.monotonic() - t0
            print(f"  [{done_count}/{len(processable)}] {repo}: {len(stats)}/{len(indices)} enriched ({elapsed:.1f}s)")

            for i in indices:
                commit_stats = stats.get(commits[i]["hash"])
                commits[i]["diff"] = commit_stats
                if commit_stats:
                    enriched += 1
                else:
                    skipped += 1

    # Write output
    with open(args.output, "w") as fout:
        for obj in commits:
            fout.write(json.dumps(obj) + "\n")

    total = enriched + skipped
    print(f"\nDone: {enriched}/{total} enriched ({100*enriched/total:.1f}%)")
    if missing_repo:
        print(f"Missing repos: {dict(missing_repo)}")
    print(f"Output: {args.output}")


if __name__ == "__main__":
    main()
