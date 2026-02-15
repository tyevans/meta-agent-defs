"""Results registry for tracking model experiment outcomes.

Simple file-based registry that stores experiment results as JSON files
with run metadata (timestamp, git commit, CLI args).

Usage as module:
    from registry import register_result, list_results, best_result, compare_results

    # Register a result
    register_result(metrics_dict, metadata={"model": "tfidf-logreg", ...})

    # Query results
    all_results = list_results()
    best = best_result(metric="macro_f1")
    compare_results()  # prints table to stdout

Usage as CLI:
    python registry.py                     # print comparison table
    python registry.py --metric accuracy   # show best by accuracy
"""

from __future__ import annotations

import argparse
import json
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Any


def register_result(
    result: dict[str, Any],
    registry_dir: str | Path = "results/",
    metadata: dict[str, Any] | None = None,
) -> Path:
    """Register an experiment result to the registry.

    Args:
        result: Result dict from eval.run_eval() containing metrics, report, etc.
        registry_dir: Directory to store registry files (default: "results/")
        metadata: Optional dict with additional metadata (model name, CLI args, etc.)

    Returns:
        Path to the saved JSON file

    Side effects:
        Creates registry_dir if it doesn't exist
        Writes a JSON file named {model}_{timestamp}.json
    """
    registry_dir = Path(registry_dir)
    registry_dir.mkdir(parents=True, exist_ok=True)

    # Merge metadata into result
    full_result = {**result}
    if metadata:
        full_result.update(metadata)

    # Add timestamp
    timestamp = datetime.now().isoformat(timespec="seconds")
    full_result["timestamp"] = timestamp

    # Add git commit hash if in a git repo
    try:
        commit_hash = subprocess.check_output(
            ["git", "rev-parse", "HEAD"],
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
        full_result["git_commit"] = commit_hash
    except (subprocess.CalledProcessError, FileNotFoundError):
        full_result["git_commit"] = None

    # Generate filename: {model}_{timestamp}.json
    model_name = full_result.get("model", "unknown")
    # Make timestamp filesystem-safe
    safe_timestamp = timestamp.replace(":", "-")
    filename = f"{model_name}_{safe_timestamp}.json"
    filepath = registry_dir / filename

    # Write to file
    with open(filepath, "w") as f:
        json.dump(full_result, f, indent=2)

    return filepath


def list_results(registry_dir: str | Path = "results/") -> list[dict[str, Any]]:
    """List all experiment results sorted by timestamp (newest first).

    Args:
        registry_dir: Directory where registry files are stored

    Returns:
        List of result dicts, sorted by timestamp (newest first)
    """
    registry_dir = Path(registry_dir)
    if not registry_dir.exists():
        return []

    results = []
    for filepath in registry_dir.glob("*.json"):
        # Skip checkpoint directories
        if filepath.is_dir():
            continue

        try:
            with open(filepath) as f:
                result = json.load(f)
                result["_filepath"] = str(filepath)  # track source file
                results.append(result)
        except (json.JSONDecodeError, KeyError):
            # Skip malformed files
            continue

    # Sort by timestamp, newest first
    results.sort(key=lambda r: r.get("timestamp", ""), reverse=True)
    return results


def best_result(
    metric: str = "macro_f1",
    registry_dir: str | Path = "results/",
) -> dict[str, Any] | None:
    """Get the result with the highest value for a given metric.

    Args:
        metric: Metric key to maximize (default: "macro_f1")
                Should be a key in result["metrics"] dict
        registry_dir: Directory where registry files are stored

    Returns:
        Result dict with highest metric value, or None if no results
    """
    results = list_results(registry_dir)
    if not results:
        return None

    # Filter results that have the requested metric
    def get_metric_value(result: dict[str, Any]) -> float:
        if "metrics" in result and metric in result["metrics"]:
            return result["metrics"][metric]
        # Fallback: check if metric is at top level
        if metric in result:
            val = result[metric]
            return val if isinstance(val, (int, float)) else -1.0
        return -1.0

    valid_results = [r for r in results if get_metric_value(r) >= 0]
    if not valid_results:
        return None

    return max(valid_results, key=get_metric_value)


def compare_results(registry_dir: str | Path = "results/") -> None:
    """Print comparison table of all experiment results.

    Args:
        registry_dir: Directory where registry files are stored

    Side effects:
        Prints formatted table to stdout
    """
    results = list_results(registry_dir)
    if not results:
        print("No results found in registry.")
        return

    print("\nExperiment Results Comparison")
    print("=" * 100)
    print(
        f"{'Model':20s} {'Timestamp':20s} {'Accuracy':>10s} {'F1-Macro':>10s} {'F1-Weight':>10s} {'Commit':10s}"
    )
    print("-" * 100)

    for result in results:
        model = result.get("model", "unknown")[:20]
        timestamp = result.get("timestamp", "")[:20]

        # Extract metrics
        metrics = result.get("metrics", {})
        accuracy = metrics.get("accuracy", result.get("accuracy", 0.0))
        f1_macro = metrics.get("f1_macro", result.get("f1_macro", 0.0))
        f1_weighted = metrics.get("f1_weighted", result.get("f1_weighted", 0.0))

        # Git commit (short hash)
        git_commit = result.get("git_commit", "")
        git_short = git_commit[:8] if git_commit else "n/a"

        print(
            f"{model:20s} {timestamp:20s} {accuracy:10.3f} {f1_macro:10.3f} {f1_weighted:10.3f} {git_short:10s}"
        )

    print("=" * 100)
    print(f"Total experiments: {len(results)}\n")


def main():
    """CLI entrypoint for registry operations."""
    parser = argparse.ArgumentParser(
        description="Query experiment results registry"
    )
    parser.add_argument(
        "--registry-dir",
        type=Path,
        default="results/",
        help="Registry directory (default: results/)",
    )
    parser.add_argument(
        "--metric",
        type=str,
        default=None,
        help="Show best result by metric (e.g., macro_f1, accuracy)",
    )
    args = parser.parse_args()

    if args.metric:
        # Show best result for metric
        best = best_result(metric=args.metric, registry_dir=args.registry_dir)
        if best is None:
            print(f"No results found with metric '{args.metric}'")
            return

        print(f"\nBest result by {args.metric}:")
        print(f"  Model: {best.get('model', 'unknown')}")
        print(f"  Timestamp: {best.get('timestamp', 'n/a')}")

        metrics = best.get("metrics", {})
        metric_value = metrics.get(args.metric, best.get(args.metric, 0.0))
        print(f"  {args.metric}: {metric_value:.3f}")

        if "git_commit" in best:
            print(f"  Git commit: {best['git_commit'][:8]}")

        # Show all metrics
        print("\n  All metrics:")
        for k, v in sorted(metrics.items()):
            if isinstance(v, (int, float)):
                print(f"    {k}: {v:.3f}")
    else:
        # Default: print comparison table
        compare_results(registry_dir=args.registry_dir)


if __name__ == "__main__":
    main()
