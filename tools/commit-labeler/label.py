"""Label git commit messages using a local Ollama model via pydantic-ai.

Supports both single-commit and batched modes. Batched mode packs multiple
commits into one request to maximize throughput within the model's context window.

Supports two input formats:
- Raw pipe-delimited commits (repo|||hash|||author|||email|||date|||message)
- Enriched JSONL with diff metadata (--enriched flag)

Usage:
    # Raw commits
    uv run python label.py ../data/strided-commits.txt --batch-size 20
    uv run python label.py ../data/strided-commits.txt --limit 50  # test run

    # Enriched commits with diff context
    uv run python label.py ../data/labeled-enriched.jsonl --enriched --batch-size 10
    uv run python label.py ../data/labeled-enriched.jsonl --enriched  # resume from last position
"""

from __future__ import annotations

import argparse
import json
import time
from collections import Counter
from pathlib import Path

from pydantic_ai import Agent, PromptedOutput
from pydantic_ai.models.openai import OpenAIChatModel, OpenAIChatModelSettings
from pydantic_ai.providers.ollama import OllamaProvider

from data import load_done_hashes, parse_commit_line, print_distribution
from schemas import BatchClassification, CommitClassification


# --- Prompts ---


SYSTEM_PROMPT_BASIC = """\
You are a git commit message classifier. Classify each commit — assign up to \
3 labels if the commit clearly spans multiple categories, but prefer 1 label \
for focused commits. Be precise — read each message carefully.

Categories:
- feat: Adds new functionality, features, capabilities, or support for new things
- fix: Corrects a bug, error, or broken behavior
- refactor: Restructures code without changing behavior
- chore: Maintenance, dependencies, config, routine updates
- docs: Documentation changes only
- test: Test additions or modifications only
- perf: Performance improvements
- style: Formatting, whitespace, naming (no logic change)
- ci: CI/CD pipeline changes
- build: Build system or external dependency changes
- i18n: Internationalization, translations, locale changes
- revert: Reverts a previous change
- other: Doesn't fit any category above

Focus on the primary intent. If ambiguous, pick the most likely category and \
reflect uncertainty in the confidence score. Keep reasoning to one sentence."""


SYSTEM_PROMPT_ENRICHED = """\
You are a git commit message classifier. Classify each commit — assign up to \
3 labels if the commit clearly spans multiple categories, but prefer 1 label \
for focused commits. Use BOTH the commit message AND the diff metadata provided.

Categories:
- feat: Adds new functionality, features, capabilities, or support for new things
- fix: Corrects a bug, error, or broken behavior
- refactor: Restructures code without changing behavior
- chore: Maintenance, dependencies, config, routine updates
- docs: Documentation changes only (README, docs/, comments-only changes)
- test: Test additions or modifications only (test files, specs, fixtures)
- perf: Performance improvements
- style: Formatting, whitespace, naming (no logic change)
- ci: CI/CD pipeline changes (workflows, pipelines, CI config)
- build: Build system or external dependency changes
- i18n: Internationalization, translations, locale changes
- revert: Reverts a previous change
- other: Doesn't fit any category above

Use the diff metadata to disambiguate ambiguous messages:
- File extensions reveal intent: .md/.rst/.txt -> docs, .test.* / *_test.* / spec.* -> test
- File paths reveal intent: docs/ -> docs, .github/workflows/ -> ci, locales/ -> i18n
- Pure additions (0 deletions) with new files often indicate feat
- Small edits to config files (yml, json, toml) are usually chore
- Large insertions with 0 deletions to a new file = likely feat

Focus on the primary intent. Keep reasoning to one sentence."""


def format_diff(diff: dict | None) -> str:
    """Format diff metadata into a readable string for the prompt."""
    if not diff:
        return "  (no diff data available)"

    parts = [
        f"  Files ({diff['file_count']}): {', '.join(diff['files'][:8])}",
    ]
    if diff["file_count"] > 8:
        parts[0] += f" ... (+{diff['file_count'] - 8} more)"

    parts.append(f"  Changes: +{diff['insertions']} -{diff['deletions']}")

    exts = list(set(diff.get("extensions", [])))
    if exts:
        parts.append(f"  Extensions: {', '.join(sorted(exts))}")

    dirs = list(set(diff.get("top_dirs", [])))
    if dirs:
        parts.append(f"  Top dirs: {', '.join(sorted(dirs)[:6])}")

    return "\n".join(parts)


def make_batch_prompt(items: list[str | dict], enriched: bool = False) -> str:
    """Build a user prompt for a batch of commits.

    Args:
        items: Either list of message strings (enriched=False) or commit dicts (enriched=True)
        enriched: Whether to include diff metadata
    """
    if not enriched:
        # Basic mode: items are message strings
        lines = []
        for i, msg in enumerate(items, 1):
            lines.append(f"{i}. {msg}")
        return (
            f"Classify each of these {len(items)} commit messages. "
            f"Return exactly {len(items)} classifications in the same order.\n\n"
            + "\n".join(lines)
        )
    else:
        # Enriched mode: items are commit dicts with diff metadata
        lines = []
        for i, c in enumerate(items, 1):
            lines.append(f"{i}. Message: {c['message']}")
            lines.append(f"   Diff:")
            lines.append(format_diff(c.get("diff")))
            lines.append("")
        return (
            f"Classify each of these {len(items)} commits. "
            f"Return exactly {len(items)} classifications in the same order.\n\n"
            + "\n".join(lines)
        )


def make_single_prompt(item: str | dict, enriched: bool = False) -> str:
    """Build a user prompt for a single commit.

    Args:
        item: Either message string (enriched=False) or commit dict (enriched=True)
        enriched: Whether to include diff metadata
    """
    if not enriched:
        return f"Classify this commit message:\n\n{item}"
    else:
        return (
            f"Classify this commit:\n\n"
            f"Message: {item['message']}\n"
            f"Diff:\n{format_diff(item.get('diff'))}"
        )


# --- I/O helpers ---


def load_enriched(path: Path) -> list[dict]:
    """Load enriched JSONL records."""
    records = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            records.append(json.loads(line))
    return records


# --- Main ---


def main():
    parser = argparse.ArgumentParser(description="Label git commits via Ollama")
    parser.add_argument("input", type=Path, help="Input file (raw commits or enriched JSONL)")
    parser.add_argument("-o", "--output", type=Path, default=None, help="Output JSONL")
    parser.add_argument(
        "--enriched",
        action="store_true",
        help="Input is enriched JSONL with diff metadata (enables diff-aware prompts)",
    )
    parser.add_argument("--model", default="gpt-oss:20b", help="Ollama model name")
    parser.add_argument(
        "--server",
        default="http://192.168.1.14:11434/v1",
        help="Ollama server URL",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=20,
        help="Commits per request (1=single mode, >1=batch mode)",
    )
    parser.add_argument("--limit", type=int, default=0, help="Max commits (0=all)")
    parser.add_argument("--retry", type=int, default=3, help="Retries per batch")
    parser.add_argument("--delay", type=float, default=0.5, help="Seconds between requests")
    parser.add_argument(
        "--think",
        default="low",
        choices=["low", "medium", "high", "none"],
        help="Reasoning effort (low=fastest, none=attempt to disable)",
    )
    args = parser.parse_args()

    if args.output is None:
        if args.enriched:
            args.output = args.input.parent / "relabeled.jsonl"
        else:
            args.output = args.input.with_suffix(".labeled.jsonl")

    # Load input
    if args.enriched:
        commits = load_enriched(args.input)
        print(f"Loaded {len(commits)} enriched commits from {args.input}")
    else:
        with open(args.input) as f:
            raw_lines = [l.strip() for l in f if l.strip()]
        commits = [c for line in raw_lines if (c := parse_commit_line(line)) is not None]
        print(f"Loaded {len(commits)} commits from {args.input}")

    # Resume support
    done_hashes = load_done_hashes(args.output)
    remaining = [c for c in commits if c["hash"] not in done_hashes]
    if done_hashes:
        print(f"Resuming: {len(done_hashes)} done, {len(remaining)} remaining")

    if args.limit > 0:
        remaining = remaining[: args.limit]

    if not remaining:
        print("Nothing to label.")
        return

    # Set up pydantic-ai
    ollama_model = OpenAIChatModel(
        model_name=args.model,
        provider=OllamaProvider(base_url=args.server),
    )

    # Reasoning effort: "low" is fastest, "none" maps to "low" (Ollama floor)
    effort = args.think if args.think != "none" else "low"
    model_settings = OpenAIChatModelSettings(openai_reasoning_effort=effort)

    # Select prompt based on enriched flag
    system_prompt = SYSTEM_PROMPT_ENRICHED if args.enriched else SYSTEM_PROMPT_BASIC

    use_batch = args.batch_size > 1

    if use_batch:
        agent = Agent(
            model=ollama_model,
            output_type=PromptedOutput(
                BatchClassification,
                name="BatchClassification",
                description="Classification results for a batch of git commits",
            ),
            system_prompt=system_prompt,
        )
    else:
        agent = Agent(
            model=ollama_model,
            output_type=PromptedOutput(
                CommitClassification,
                name="CommitClassification",
                description="Multi-label classification of a git commit",
            ),
            system_prompt=system_prompt,
        )

    # Process
    labeled = 0
    errors = 0
    start_time = time.time()

    # Build batches
    batches: list[list[dict]] = []
    for i in range(0, len(remaining), args.batch_size):
        batches.append(remaining[i : i + args.batch_size])

    print(f"Processing {len(remaining)} commits in {len(batches)} batches of ~{args.batch_size}")

    with open(args.output, "a") as out_f:
        for batch_idx, batch in enumerate(batches):
            for attempt in range(args.retry):
                try:
                    if use_batch:
                        # Build prompt based on enriched flag
                        if args.enriched:
                            prompt = make_batch_prompt(batch, enriched=True)
                        else:
                            prompt = make_batch_prompt([c["message"] for c in batch], enriched=False)

                        result = agent.run_sync(prompt, model_settings=model_settings)
                        classifications = result.output.classifications

                        # Validate count matches
                        if len(classifications) != len(batch):
                            raise ValueError(
                                f"Expected {len(batch)} classifications, got {len(classifications)}"
                            )

                        for commit, classification in zip(batch, classifications):
                            # Convert LabelEntry list to dict list for JSON
                            labels_list = [
                                {"label": lbl.label.value, "confidence": lbl.confidence}
                                for lbl in classification.labels
                            ]

                            record = {
                                "repo": commit["repo"],
                                "hash": commit["hash"],
                                "author": commit["author"],
                                "email": commit["email"],
                                "date": commit["date"],
                                "message": commit["message"],
                                "labels": labels_list,
                                "reasoning": classification.reasoning,
                            }

                            # Include diff and old_label if enriched
                            if args.enriched:
                                record["diff"] = commit.get("diff")
                                record["old_label"] = commit.get("label")

                            out_f.write(json.dumps(record) + "\n")
                            labeled += 1
                    else:
                        # Single mode
                        commit = batch[0]
                        if args.enriched:
                            prompt = make_single_prompt(commit, enriched=True)
                        else:
                            prompt = make_single_prompt(commit["message"], enriched=False)

                        result = agent.run_sync(prompt, model_settings=model_settings)
                        classification = result.output

                        # Convert LabelEntry list to dict list for JSON
                        labels_list = [
                            {"label": lbl.label.value, "confidence": lbl.confidence}
                            for lbl in classification.labels
                        ]

                        record = {
                            "repo": commit["repo"],
                            "hash": commit["hash"],
                            "author": commit["author"],
                            "email": commit["email"],
                            "date": commit["date"],
                            "message": commit["message"],
                            "labels": labels_list,
                            "reasoning": classification.reasoning,
                        }

                        # Include diff and old_label if enriched
                        if args.enriched:
                            record["diff"] = commit.get("diff")
                            record["old_label"] = commit.get("label")

                        out_f.write(json.dumps(record) + "\n")
                        labeled += 1

                    out_f.flush()
                    break  # success

                except Exception as e:
                    if attempt < args.retry - 1:
                        print(
                            f"  Retry {attempt + 1}/{args.retry} batch {batch_idx}: {e}"
                        )
                        time.sleep(2)
                    else:
                        print(f"  FAILED batch {batch_idx} ({len(batch)} commits): {e}")
                        errors += len(batch)

            # Progress
            elapsed = time.time() - start_time
            commits_done = labeled + errors
            rate = labeled / elapsed if elapsed > 0 else 0
            commits_left = len(remaining) - commits_done
            eta = commits_left / rate if rate > 0 else 0

            print(
                f"[batch {batch_idx + 1}/{len(batches)}] "
                f"labeled={labeled} errors={errors} "
                f"rate={rate:.1f}/s "
                f"eta={eta / 60:.1f}m"
            )

            if args.delay > 0:
                time.sleep(args.delay)

    # Summary
    elapsed = time.time() - start_time
    print(f"\nDone: {labeled} labeled, {errors} errors in {elapsed:.0f}s")
    print(f"Output: {args.output}")
    print_distribution(args.output)

    # Label change summary (enriched mode only)
    if args.enriched and args.output.exists():
        changed = 0
        total = 0
        with open(args.output) as f:
            for line in f:
                obj = json.loads(line.strip())
                total += 1
                old_label = obj.get("old_label")
                if old_label and obj.get("labels"):
                    # Compare old single label to new primary (first) label
                    new_primary = obj["labels"][0]["label"]
                    if old_label != new_primary:
                        changed += 1
        if total:
            print(f"\nLabel changes: {changed}/{total} ({100*changed/total:.1f}%)")


if __name__ == "__main__":
    main()
