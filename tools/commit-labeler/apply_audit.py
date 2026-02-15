"""Apply audit verdicts to relabel training data.

Reads the audit JSONL (with verdict field filled in), patches the source
labeled JSONL, and reports what changed.

Usage:
    # Apply verdicts from audit
    uv run python apply_audit.py ../data/audit.jsonl ../data/labeled-all.jsonl

    # Dry run (show what would change)
    uv run python apply_audit.py ../data/audit.jsonl ../data/labeled-all.jsonl --dry-run

Verdict values:
    relabel  - Change label to the model's prediction
    keep     - Keep original label (model was wrong)
    skip     - Skip this sample (remove from dataset)
    <empty>  - Not yet reviewed (ignored)
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description="Apply audit verdicts to labeled data")
    parser.add_argument("audit", type=Path, help="Audit JSONL with verdicts")
    parser.add_argument("labeled", type=Path, help="Source labeled JSONL to patch")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show changes without writing",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Output path (default: overwrite source)",
    )
    parser.add_argument(
        "--reset-test",
        action="store_true",
        help="Delete test-hashes.json to force re-freeze on next train",
    )
    args = parser.parse_args()

    if args.output is None:
        args.output = args.labeled

    # Load audit verdicts
    verdicts: dict[str, dict] = {}
    verdict_counts: Counter[str] = Counter()
    with open(args.audit) as f:
        for line in f:
            entry = json.loads(line.strip())
            v = entry.get("verdict", "").strip().lower()
            if not v:
                continue
            verdict_counts[v] += 1
            verdicts[entry["hash"]] = entry

    print(f"Loaded {len(verdicts)} verdicts from {args.audit}")
    for v, count in verdict_counts.most_common():
        print(f"  {v:12s}: {count}")

    unreviewed_count = 0
    with open(args.audit) as f:
        for line in f:
            entry = json.loads(line.strip())
            if not entry.get("verdict", "").strip():
                unreviewed_count += 1
    if unreviewed_count:
        print(f"  (unreviewed): {unreviewed_count}")

    # Process labeled data
    relabeled = 0
    skipped = 0
    kept = 0
    output_lines = []

    with open(args.labeled) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            obj = json.loads(line)
            h = obj.get("hash", "")

            if h in verdicts:
                v = verdicts[h]["verdict"].strip().lower()
                if v == "relabel":
                    old_label = obj.get("label", obj.get("labels", [{}])[0].get("label", "?"))
                    new_label = verdicts[h]["predicted"]
                    if not args.dry_run:
                        # Update single-label format
                        obj["label"] = new_label
                        # Update multi-label format if present
                        if "labels" in obj and obj["labels"]:
                            obj["labels"][0] = {"label": new_label, "confidence": 1.0}
                    print(f"  RELABEL {h[:8]}: {old_label} -> {new_label}  \"{obj['message'][:60]}\"")
                    relabeled += 1
                elif v == "skip":
                    print(f"  SKIP    {h[:8]}: \"{obj['message'][:60]}\"")
                    skipped += 1
                    continue  # Don't write this record
                elif v == "keep":
                    kept += 1

            output_lines.append(json.dumps(obj))

    print(f"\nSummary:")
    print(f"  Relabeled: {relabeled}")
    print(f"  Skipped:   {skipped}")
    print(f"  Kept:      {kept}")
    print(f"  Output:    {len(output_lines)} records")

    if args.dry_run:
        print("\n(dry run — no files modified)")
        return

    # Write output
    with open(args.output, "w") as f:
        for line in output_lines:
            f.write(line + "\n")
    print(f"\nWritten to {args.output}")

    # Optionally reset test set
    if args.reset_test:
        test_file = args.labeled.parent / "test-hashes.json"
        if test_file.exists():
            test_file.unlink()
            print(f"Deleted {test_file} (will re-freeze on next train --reset-test)")


if __name__ == "__main__":
    main()
