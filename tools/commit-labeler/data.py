"""Shared data loading utilities for the commit-labeler pipeline."""

from __future__ import annotations

import json
import random
from collections import Counter
from pathlib import Path


def load_labeled(path: Path, min_confidence: float = 0.0) -> list[dict]:
    """Load labeled commits as list of dicts."""
    records = []
    skipped = 0
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            obj = json.loads(line)
            if obj.get("confidence", 1.0) < min_confidence:
                skipped += 1
                continue
            records.append(obj)
    if skipped:
        print(f"Skipped {skipped} low-confidence samples (< {min_confidence})")
    return records


def freeze_test_set(
    records: list[dict],
    test_fraction: float,
    test_file: Path,
) -> set[str]:
    """Pick a stratified random test set and save hashes to disk."""
    by_label: dict[str, list[str]] = {}
    for r in records:
        by_label.setdefault(r["label"], []).append(r["hash"])

    rng = random.Random(42)
    test_hashes: set[str] = set()
    for label, hashes in by_label.items():
        n = max(1, int(len(hashes) * test_fraction))
        test_hashes.update(rng.sample(hashes, min(n, len(hashes))))

    with open(test_file, "w") as f:
        json.dump(sorted(test_hashes), f)
    print(f"Frozen test set: {len(test_hashes)} hashes -> {test_file}")
    return test_hashes


def load_test_set(test_file: Path) -> set[str]:
    """Load frozen test hashes from disk."""
    with open(test_file) as f:
        return set(json.load(f))


def load_labels(labels_json_path: Path) -> list[str]:
    """Load label vocabulary from labels.json."""
    with open(labels_json_path) as f:
        data = json.load(f)
    return data["labels"]


def parse_commit_line(line: str) -> dict | None:
    """Parse repo|||hash|||author|||email|||date|||message format."""
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


def load_done_hashes(output_path: Path) -> set[str]:
    """Load already-labeled commit hashes for resume support."""
    done = set()
    if output_path.exists():
        with open(output_path) as f:
            for line in f:
                try:
                    obj = json.loads(line.strip())
                    done.add(obj["hash"])
                except (json.JSONDecodeError, KeyError):
                    continue
    return done


def print_distribution(output_path: Path):
    """Print label distribution from output file (multi-label aware)."""
    dist: Counter[str] = Counter()
    with open(output_path) as f:
        for line in f:
            try:
                obj = json.loads(line.strip())
                # Handle both old single-label and new multi-label formats
                if "labels" in obj:
                    # Multi-label: count each label
                    for lbl in obj["labels"]:
                        dist[lbl["label"]] += 1
                elif "label" in obj:
                    # Old single-label format
                    dist[obj["label"]] += 1
            except (json.JSONDecodeError, KeyError):
                pass

    total = sum(dist.values())
    if total == 0:
        return

    print("\nLabel distribution:")
    for label, count in dist.most_common():
        print(f"  {label:12s}: {count:4d} ({100 * count / total:.1f}%)")
