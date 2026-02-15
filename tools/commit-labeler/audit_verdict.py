#!/usr/bin/env python3
"""
Apply verdicts to audit entries based on heuristics.

For each disagreement, decide:
  - relabel: model prediction is correct
  - keep: original label is correct
  - skip: sample is ambiguous/garbage
"""

import json
import sys
from pathlib import Path
from typing import Literal


Verdict = Literal["relabel", "keep", "skip"]


def decide_verdict(message: str, label: str, predicted: str, confidence: float) -> Verdict:
    """
    Apply heuristics to decide verdict.

    Rules:
    1. High confidence (>=0.8): Model is likely right unless message strongly contradicts
    2. Medium confidence (0.5-0.8): Look at message text carefully
    3. Low confidence (<0.5): Default to keep unless obvious error
    """
    msg_lower = message.lower().strip()

    # Skip obvious garbage/merges
    if msg_lower.startswith("merge "):
        return "skip"

    # High confidence threshold
    if confidence >= 0.8:
        # Pattern: "Fix/Fixed/Fixes ..." should be 'fix', not 'style' or 'refactor'
        if predicted == "fix" and label in ["style", "refactor", "chore", "docs"]:
            if any(msg_lower.startswith(x) for x in ["fix", "fixed", "fixes", "fixed:", "fix:"]):
                return "relabel"

        # Pattern: "Add/Added ..." should be 'feat'
        if predicted == "feat" and label in ["docs", "style", "chore"]:
            if any(msg_lower.startswith(x) for x in ["add", "added", "adds", "add:", "added:"]):
                return "relabel"

        # Pattern: dependency updates should be 'chore', not 'build'
        if predicted == "chore" and label == "build":
            if any(x in msg_lower for x in ["deps:", "update deps", "bump", "update dependencies", "upgrade deps", "update everything"]):
                return "relabel"

        # Pattern: "Refactor ..." should be 'refactor'
        if predicted == "refactor" and label in ["feat", "style", "docs", "chore"]:
            if any(msg_lower.startswith(x) for x in ["refactor", "refactor:", "refactored"]):
                return "relabel"

        # Pattern: "docs:", "documentation:", "update README" should be 'docs'
        if predicted == "docs" and label in ["feat", "chore", "style"]:
            if any(x in msg_lower for x in ["docs:", "documentation:", "readme", "doc:", "update docs"]):
                return "relabel"

        # Pattern: CI/CD pipeline changes should be 'ci', not 'chore' or 'build'
        if predicted == "ci" and label in ["chore", "build"]:
            if any(x in msg_lower for x in [".github/workflows", "jenkinsfile", "gitlab-ci", ".circleci", "travis", "ci:", "pipeline"]):
                return "relabel"

        # Pattern: test changes should be 'test'
        if predicted == "test" and label in ["feat", "refactor", "chore"]:
            if any(x in msg_lower for x in ["test:", "tests:", "add tests", "update tests", "fix test"]):
                return "relabel"

        # Otherwise, high confidence prediction overrides original
        # unless there's a strong semantic match to the original label
        return "relabel"

    # Medium confidence (0.5-0.8): look more carefully at message
    elif confidence >= 0.5:
        # Check if message strongly supports the predicted label
        if predicted == "fix" and any(msg_lower.startswith(x) for x in ["fix", "fixed", "fixes"]):
            return "relabel"
        elif predicted == "feat" and any(msg_lower.startswith(x) for x in ["add", "added", "new", "adds"]):
            return "relabel"
        elif predicted == "chore" and any(x in msg_lower for x in ["deps:", "bump", "update dep", "update everything"]):
            return "relabel"
        elif predicted == "refactor" and msg_lower.startswith("refactor"):
            return "relabel"
        elif predicted == "docs" and any(x in msg_lower for x in ["docs:", "readme", "documentation"]):
            return "relabel"

        # If no strong semantic match, keep original
        return "keep"

    # Low confidence (<0.5): default to keep unless obvious error
    else:
        # Only relabel if there's a very clear pattern mismatch
        if predicted == "fix" and msg_lower.startswith(("fix", "fixed", "fixes")):
            return "relabel"
        elif predicted == "feat" and msg_lower.startswith(("add", "new")):
            return "relabel"

        return "keep"


def main():
    audit_path = Path("/home/ty/workspace/meta-agent-defs/tools/data/audit.jsonl")

    verdicts_by_decision: dict[str, int] = {"relabel": 0, "keep": 0, "skip": 0}
    entries_processed = 0
    chunk_size = 100

    # First pass: read and apply verdicts
    entries = []
    with open(audit_path) as f:
        for line in f:
            if not line.strip():
                continue
            entry = json.loads(line)

            # Only process entries without verdict
            if entry.get("verdict", "").strip():
                entries.append(entry)
                continue

            # Apply verdict
            message = entry.get("message", "")
            label = entry.get("label", "")
            predicted = entry.get("predicted", "")
            confidence = entry.get("confidence", 0.0)

            verdict = decide_verdict(message, label, predicted, confidence)
            entry["verdict"] = verdict
            entries.append(entry)

            verdicts_by_decision[verdict] += 1
            entries_processed += 1

            # Print progress for each chunk
            if entries_processed % chunk_size == 0:
                print(f"Processed {entries_processed} entries...", file=sys.stderr)

    # Write back
    with open(audit_path, "w") as f:
        for entry in entries:
            f.write(json.dumps(entry) + "\n")

    print(f"\nVerdicts applied to audit.jsonl:")
    print(f"  relabel: {verdicts_by_decision['relabel']}")
    print(f"  keep:    {verdicts_by_decision['keep']}")
    print(f"  skip:    {verdicts_by_decision['skip']}")
    print(f"  Total:   {sum(verdicts_by_decision.values())}")


if __name__ == "__main__":
    main()
