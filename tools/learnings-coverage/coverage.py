#!/usr/bin/env python3
"""Learnings coverage analyzer.

Embeds agent learnings and commit messages using sentence-transformers,
then computes a knowledge coverage score showing what percentage of an
agent's commit-message semantic space is covered by their learnings.

Usage:
    python coverage.py --agent skill-author
    python coverage.py --agent ml-eng --threshold 0.4 --since 180d
    python coverage.py --agent rust-dev --json-output results.json
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import numpy as np
import yaml
from sentence_transformers import SentenceTransformer
from sklearn.cluster import DBSCAN
from sklearn.metrics.pairwise import cosine_similarity


# ---------------------------------------------------------------------------
# Data types
# ---------------------------------------------------------------------------

@dataclass
class SparseRegion:
    theme: str
    uncovered_count: int
    representative_commits: list[str]


@dataclass
class DenseCluster:
    learning: str
    covered_commits: int
    similarity_mean: float


@dataclass
class UncoveredCommit:
    message: str
    nearest_similarity: float


@dataclass
class CoverageResult:
    agent: str
    coverage_score: float
    total_commits: int
    covered_commits: int
    total_learnings: int
    threshold: float
    sparse_regions: list[SparseRegion]
    dense_clusters: list[DenseCluster]
    uncovered_commits: list[UncoveredCommit]

    def to_dict(self) -> dict[str, Any]:
        return {
            "agent": self.agent,
            "coverage_score": round(self.coverage_score, 3),
            "total_commits": self.total_commits,
            "covered_commits": self.covered_commits,
            "total_learnings": self.total_learnings,
            "threshold": self.threshold,
            "sparse_regions": [
                {
                    "theme": r.theme,
                    "uncovered_count": r.uncovered_count,
                    "representative_commits": r.representative_commits,
                }
                for r in self.sparse_regions
            ],
            "dense_clusters": [
                {
                    "learning": c.learning,
                    "covered_commits": c.covered_commits,
                    "similarity_mean": round(c.similarity_mean, 3),
                }
                for c in self.dense_clusters
            ],
            "uncovered_commits": [
                {
                    "message": u.message,
                    "nearest_similarity": round(u.nearest_similarity, 3),
                }
                for u in self.uncovered_commits
            ],
        }

    def to_markdown(self) -> str:
        lines = [
            f"# Knowledge Coverage: {self.agent}",
            "",
            f"Coverage: {self.coverage_score:.0%} ({self.covered_commits}/{self.total_commits} commits within {self.threshold} similarity threshold)",
            f"Learnings: {self.total_learnings} entries",
            "",
        ]

        if self.sparse_regions:
            lines.append("## Sparse Regions (Knowledge Gaps)")
            for i, r in enumerate(self.sparse_regions, 1):
                lines.append(f"{i}. **{r.theme}** — {r.uncovered_count} uncovered commits")
                for msg in r.representative_commits[:3]:
                    lines.append(f'   - "{msg}"')
            lines.append("")

        if self.dense_clusters:
            lines.append("## Dense Clusters (Well-Learned Areas)")
            for i, c in enumerate(self.dense_clusters, 1):
                lines.append(
                    f"{i}. **{_truncate(c.learning, 80)}** — covers {c.covered_commits} commits (mean similarity: {c.similarity_mean:.2f})"
                )
            lines.append("")

        if self.uncovered_commits:
            lines.append("## Top Uncovered Commits")
            for i, u in enumerate(self.uncovered_commits[:15], 1):
                lines.append(f'{i}. "{u.message}" (nearest: {u.nearest_similarity:.2f})')
            lines.append("")

        return "\n".join(lines)


# ---------------------------------------------------------------------------
# Parsing
# ---------------------------------------------------------------------------

def find_repo_root() -> Path:
    """Walk up from cwd to find the git repo root."""
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True, text=True, check=True,
    )
    return Path(result.stdout.strip())


def load_team_yaml(repo_root: Path) -> dict[str, Any]:
    """Load .claude/team.yaml and return parsed YAML."""
    team_path = repo_root / ".claude" / "team.yaml"
    if not team_path.exists():
        print(f"Error: {team_path} not found", file=sys.stderr)
        sys.exit(1)
    with open(team_path) as f:
        return yaml.safe_load(f)


def get_agent_owns(team: dict[str, Any], agent_name: str) -> list[str]:
    """Extract the owns patterns for a given agent name."""
    for member in team.get("members", []):
        if member["name"] == agent_name:
            return member.get("owns", [])
    available = [m["name"] for m in team.get("members", [])]
    print(
        f"Error: agent '{agent_name}' not found in team.yaml. Available: {', '.join(available)}",
        file=sys.stderr,
    )
    sys.exit(1)


def parse_learnings(repo_root: Path, agent_name: str) -> list[str]:
    """Parse learnings.md into individual entries (one per bullet)."""
    learnings_path = repo_root / "memory" / "agents" / agent_name / "learnings.md"
    if not learnings_path.exists():
        return []

    entries: list[str] = []
    text = learnings_path.read_text()

    for line in text.splitlines():
        line = line.strip()
        if line.startswith("- ") and not line.startswith("- (none"):
            # Strip leading "- " and any trailing provenance markers
            entry = line[2:].strip()
            # Remove (added: ...) and (dispatch: ...) suffixes for cleaner embedding
            entry = re.sub(r"\s*\(added:.*?\)", "", entry)
            entry = re.sub(r"\s*\(dispatch:.*?\)", "", entry)
            entry = re.sub(r"\s*\(triaged:.*?\)", "", entry)
            if entry:
                entries.append(entry)
    return entries


def get_commit_messages(
    repo_root: Path, owns_patterns: list[str], since: str
) -> list[str]:
    """Get commit messages from git log filtered by ownership patterns."""
    cmd = [
        "git", "log",
        f"--since={since} ago",
        "--format=%s",
        "--",
    ] + owns_patterns

    result = subprocess.run(
        cmd, capture_output=True, text=True, cwd=repo_root,
    )
    if result.returncode != 0:
        print(f"Warning: git log failed: {result.stderr}", file=sys.stderr)
        return []

    messages = [m.strip() for m in result.stdout.strip().splitlines() if m.strip()]
    # Deduplicate while preserving order
    seen: set[str] = set()
    unique: list[str] = []
    for m in messages:
        if m not in seen:
            seen.add(m)
            unique.append(m)
    return unique


# ---------------------------------------------------------------------------
# Embedding & analysis
# ---------------------------------------------------------------------------

def embed_texts(model: SentenceTransformer, texts: list[str]) -> np.ndarray:
    """Embed a list of texts, returning an (N, D) array."""
    if not texts:
        return np.empty((0, 0))
    return model.encode(texts, show_progress_bar=False, convert_to_numpy=True)


def compute_coverage(
    learnings_embeddings: np.ndarray,
    commit_embeddings: np.ndarray,
    threshold: float,
) -> tuple[np.ndarray, np.ndarray]:
    """Compute coverage metrics.

    Returns:
        sim_matrix: (n_commits, n_learnings) cosine similarity matrix
        max_sims: (n_commits,) best similarity per commit
    """
    sim_matrix = cosine_similarity(commit_embeddings, learnings_embeddings)
    max_sims = sim_matrix.max(axis=1)
    return sim_matrix, max_sims


def detect_sparse_regions(
    uncovered_messages: list[str],
    uncovered_embeddings: np.ndarray,
    max_regions: int = 5,
) -> list[SparseRegion]:
    """Cluster uncovered commits to find thematic gaps."""
    if len(uncovered_messages) < 2:
        if uncovered_messages:
            return [SparseRegion(
                theme=_summarize_theme(uncovered_messages),
                uncovered_count=len(uncovered_messages),
                representative_commits=uncovered_messages[:3],
            )]
        return []

    # Use DBSCAN — no need to specify k, finds natural clusters
    clustering = DBSCAN(eps=0.5, min_samples=2, metric="cosine").fit(uncovered_embeddings)
    labels = clustering.labels_

    regions: list[SparseRegion] = []
    unique_labels = set(labels)

    for label in sorted(unique_labels):
        if label == -1:
            continue  # noise points handled separately
        mask = labels == label
        cluster_msgs = [uncovered_messages[i] for i in range(len(uncovered_messages)) if mask[i]]
        if cluster_msgs:
            regions.append(SparseRegion(
                theme=_summarize_theme(cluster_msgs),
                uncovered_count=len(cluster_msgs),
                representative_commits=cluster_msgs[:3],
            ))

    # Add noise points as an "other" region if any
    noise_mask = labels == -1
    noise_msgs = [uncovered_messages[i] for i in range(len(uncovered_messages)) if noise_mask[i]]
    if noise_msgs:
        regions.append(SparseRegion(
            theme="Miscellaneous uncovered commits",
            uncovered_count=len(noise_msgs),
            representative_commits=noise_msgs[:3],
        ))

    # Sort by uncovered count descending, take top N
    regions.sort(key=lambda r: r.uncovered_count, reverse=True)
    return regions[:max_regions]


def detect_dense_clusters(
    learnings: list[str],
    sim_matrix: np.ndarray,
    threshold: float,
    max_clusters: int = 10,
) -> list[DenseCluster]:
    """Find learnings entries that cover the most commits."""
    clusters: list[DenseCluster] = []
    for i, learning in enumerate(learnings):
        sims = sim_matrix[:, i]
        covered_mask = sims >= threshold
        covered_count = int(covered_mask.sum())
        if covered_count > 0:
            clusters.append(DenseCluster(
                learning=learning,
                covered_commits=covered_count,
                similarity_mean=float(sims[covered_mask].mean()),
            ))
    clusters.sort(key=lambda c: c.covered_commits, reverse=True)
    return clusters[:max_clusters]


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _summarize_theme(messages: list[str]) -> str:
    """Extract a theme label from a cluster of commit messages.

    Uses the most common conventional-commit prefix as the theme,
    falling back to the shortest message as representative.
    """
    prefixes: dict[str, int] = {}
    for msg in messages:
        match = re.match(r"^(feat|fix|docs|chore|refactor|test|style|ci|perf|build):", msg)
        if match:
            prefixes[match.group(1)] = prefixes.get(match.group(1), 0) + 1

    if prefixes:
        dominant = max(prefixes, key=prefixes.get)  # type: ignore[arg-type]
        # Collect the subject parts after the prefix
        subjects = []
        for msg in messages:
            if msg.startswith(f"{dominant}:"):
                subjects.append(msg[len(dominant) + 1:].strip())
        if subjects:
            # Use the shortest subject as the theme label
            shortest = min(subjects, key=len)
            return f"{dominant}: {shortest}" if len(shortest) < 60 else dominant
        return dominant

    # No conventional prefix — use shortest message
    shortest = min(messages, key=len)
    return _truncate(shortest, 60)


def _truncate(text: str, max_len: int) -> str:
    if len(text) <= max_len:
        return text
    return text[: max_len - 3] + "..."


def parse_since(since_str: str) -> str:
    """Convert shorthand like '90d', '6m', '1y' to git-friendly format."""
    match = re.match(r"^(\d+)([dmy])$", since_str)
    if not match:
        return since_str  # pass through as-is (e.g., "3 months")
    value, unit = match.groups()
    unit_map = {"d": "days", "m": "months", "y": "years"}
    return f"{value} {unit_map[unit]}"


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def analyze(
    agent_name: str,
    threshold: float = 0.5,
    since: str = "90d",
    model_name: str = "all-mpnet-base-v2",
) -> CoverageResult:
    """Run full coverage analysis for an agent."""
    repo_root = find_repo_root()
    team = load_team_yaml(repo_root)
    owns = get_agent_owns(team, agent_name)

    learnings = parse_learnings(repo_root, agent_name)
    since_friendly = parse_since(since)
    commits = get_commit_messages(repo_root, owns, since_friendly)

    if not commits:
        return CoverageResult(
            agent=agent_name,
            coverage_score=0.0,
            total_commits=0,
            covered_commits=0,
            total_learnings=len(learnings),
            threshold=threshold,
            sparse_regions=[],
            dense_clusters=[],
            uncovered_commits=[],
        )

    if not learnings:
        return CoverageResult(
            agent=agent_name,
            coverage_score=0.0,
            total_commits=len(commits),
            covered_commits=0,
            total_learnings=0,
            threshold=threshold,
            sparse_regions=[],
            dense_clusters=[],
            uncovered_commits=[
                UncoveredCommit(message=m, nearest_similarity=0.0) for m in commits[:15]
            ],
        )

    # Load model and embed
    print(f"Loading model: {model_name}", file=sys.stderr)
    model = SentenceTransformer(model_name)

    print(f"Embedding {len(learnings)} learnings + {len(commits)} commits...", file=sys.stderr)
    learnings_emb = embed_texts(model, learnings)
    commits_emb = embed_texts(model, commits)

    # Compute coverage
    sim_matrix, max_sims = compute_coverage(learnings_emb, commits_emb, threshold)
    covered_mask = max_sims >= threshold
    covered_count = int(covered_mask.sum())
    coverage_score = covered_count / len(commits)

    # Uncovered commits
    uncovered_indices = np.where(~covered_mask)[0]
    uncovered_messages = [commits[i] for i in uncovered_indices]
    uncovered_sims = max_sims[uncovered_indices]

    uncovered_list = sorted(
        [
            UncoveredCommit(message=uncovered_messages[i], nearest_similarity=float(uncovered_sims[i]))
            for i in range(len(uncovered_messages))
        ],
        key=lambda u: u.nearest_similarity,
    )

    # Sparse regions (cluster uncovered commits)
    sparse_regions: list[SparseRegion] = []
    if uncovered_messages:
        uncovered_emb = commits_emb[uncovered_indices]
        sparse_regions = detect_sparse_regions(uncovered_messages, uncovered_emb)

    # Dense clusters (learnings that cover the most)
    dense_clusters = detect_dense_clusters(learnings, sim_matrix, threshold)

    return CoverageResult(
        agent=agent_name,
        coverage_score=coverage_score,
        total_commits=len(commits),
        covered_commits=covered_count,
        total_learnings=len(learnings),
        threshold=threshold,
        sparse_regions=sparse_regions,
        dense_clusters=dense_clusters,
        uncovered_commits=uncovered_list[:15],
    )


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Analyze knowledge coverage: what %% of commit-message semantic space is captured in agent learnings?",
    )
    parser.add_argument(
        "--agent", required=True,
        help="Agent name (must exist in .claude/team.yaml)",
    )
    parser.add_argument(
        "--threshold", type=float, default=0.5,
        help="Cosine similarity threshold for 'covered' (default: 0.5)",
    )
    parser.add_argument(
        "--since", default="90d",
        help="Commit history window (default: 90d). Accepts 90d, 6m, 1y or git-friendly strings.",
    )
    parser.add_argument(
        "--model", default="all-mpnet-base-v2",
        help="Sentence-transformers model name (default: all-mpnet-base-v2)",
    )
    parser.add_argument(
        "--json-output", metavar="PATH",
        help="Write JSON results to file (in addition to markdown on stdout)",
    )
    parser.add_argument(
        "--json-only", action="store_true",
        help="Output JSON to stdout instead of markdown",
    )

    args = parser.parse_args()

    result = analyze(
        agent_name=args.agent,
        threshold=args.threshold,
        since=args.since,
        model_name=args.model,
    )

    if args.json_only:
        print(json.dumps(result.to_dict(), indent=2))
    else:
        print(result.to_markdown())

    if args.json_output:
        output_path = Path(args.json_output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "w") as f:
            json.dump(result.to_dict(), f, indent=2)
        print(f"\nJSON written to {output_path}", file=sys.stderr)


if __name__ == "__main__":
    main()
