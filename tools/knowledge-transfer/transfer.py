#!/usr/bin/env python3
"""Cross-agent knowledge transfer detector.

Uses sentence embeddings to find when agent A's learnings are semantically
relevant to agent B's ownership scope but absent from B's existing knowledge.

The "transfer value" is:
    scope_sim - knowledge_sim
where scope_sim measures how relevant A's learning is to B's domain (via commit
messages on B's owned files), and knowledge_sim measures how much B already knows
something similar.

Usage:
    python transfer.py
    python transfer.py --source skill-author
    python transfer.py --since 180d --json-output results.json
    python transfer.py --json-only
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
from sklearn.metrics.pairwise import cosine_similarity


# ---------------------------------------------------------------------------
# Data types
# ---------------------------------------------------------------------------

@dataclass
class TransferSuggestion:
    from_agent: str
    to_agent: str
    learning: str
    scope_similarity: float
    knowledge_similarity: float
    transfer_value: float
    rationale: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "from_agent": self.from_agent,
            "to_agent": self.to_agent,
            "learning": self.learning,
            "scope_similarity": round(self.scope_similarity, 3),
            "knowledge_similarity": round(self.knowledge_similarity, 3),
            "transfer_value": round(self.transfer_value, 3),
            "rationale": self.rationale,
        }


@dataclass
class TransferResult:
    transfers: list[TransferSuggestion]

    def to_dict(self) -> dict[str, Any]:
        return {
            "transfers": [t.to_dict() for t in self.transfers],
        }

    def to_markdown(self) -> str:
        if not self.transfers:
            return (
                "# Knowledge Transfer Suggestions\n\n"
                "No transfer suggestions found. All agents' learnings are either "
                "already known by relevant peers or not relevant to other scopes.\n"
            )

        # Group by (from_agent, to_agent) pair
        pairs: dict[tuple[str, str], list[TransferSuggestion]] = {}
        for t in self.transfers:
            key = (t.from_agent, t.to_agent)
            pairs.setdefault(key, []).append(t)

        lines = ["# Knowledge Transfer Suggestions", ""]

        for (from_a, to_a), suggestions in pairs.items():
            lines.append(
                f"## {from_a} -> {to_a} ({len(suggestions)} suggestion{'s' if len(suggestions) != 1 else ''})"
            )
            lines.append("")
            for i, s in enumerate(suggestions, 1):
                lines.append(f'{i}. **"{_truncate(s.learning, 100)}"**')
                lines.append(f"   - Relevance to {to_a}'s scope: {s.scope_similarity:.2f}")
                lines.append(f"   - Already known by {to_a}: {s.knowledge_similarity:.2f}")
                lines.append(f"   - Transfer value: {s.transfer_value:.2f}")
            lines.append("")

        pair_count = len(pairs)
        lines.append("---")
        lines.append(
            f"Total: {len(self.transfers)} transfer suggestion{'s' if len(self.transfers) != 1 else ''} "
            f"across {pair_count} agent pair{'s' if pair_count != 1 else ''}"
        )
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


@dataclass
class AgentInfo:
    name: str
    role: str
    owns: list[str]


def get_all_agents(team: dict[str, Any]) -> list[AgentInfo]:
    """Extract all agents with their roles and ownership patterns."""
    agents: list[AgentInfo] = []
    for member in team.get("members", []):
        agents.append(AgentInfo(
            name=member["name"],
            role=member.get("role", ""),
            owns=member.get("owns", []),
        ))
    return agents


def parse_learnings(repo_root: Path, agent_name: str) -> list[str]:
    """Parse learnings.md into individual entries (one per bullet).

    Strips provenance metadata (added/dispatch/triaged dates) before returning
    so embeddings reflect semantic content only.
    """
    learnings_path = repo_root / "memory" / "agents" / agent_name / "learnings.md"
    if not learnings_path.exists():
        return []

    entries: list[str] = []
    text = learnings_path.read_text()

    for line in text.splitlines():
        line = line.strip()
        if line.startswith("- ") and not line.startswith("- (none"):
            entry = line[2:].strip()
            # Remove provenance suffixes for cleaner embedding
            entry = re.sub(r"\s*\(added:.*?\)", "", entry)
            entry = re.sub(r"\s*\(dispatch:.*?\)", "", entry)
            entry = re.sub(r"\s*\(triaged:.*?\)", "", entry)
            entry = re.sub(r"\s*\(updated:.*?\)", "", entry)
            entry = re.sub(r"\s*\(consolidated:.*?\)", "", entry)
            if entry:
                entries.append(entry)
    return entries


def get_commit_messages(
    repo_root: Path, owns_patterns: list[str], since: str,
) -> list[str]:
    """Get commit messages from git log filtered by ownership patterns."""
    if not owns_patterns:
        return []

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
        print(f"Warning: git log failed for {owns_patterns}: {result.stderr}", file=sys.stderr)
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


def parse_since(since_str: str) -> str:
    """Convert shorthand like '90d', '6m', '1y' to git-friendly format."""
    match = re.match(r"^(\d+)([dmy])$", since_str)
    if not match:
        return since_str
    value, unit = match.groups()
    unit_map = {"d": "days", "m": "months", "y": "years"}
    return f"{value} {unit_map[unit]}"


# ---------------------------------------------------------------------------
# Embedding & analysis
# ---------------------------------------------------------------------------

def embed_texts(model: SentenceTransformer, texts: list[str]) -> np.ndarray:
    """Embed a list of texts, returning an (N, D) array."""
    if not texts:
        return np.empty((0, 0))
    return model.encode(texts, show_progress_bar=False, convert_to_numpy=True)


def build_scope_texts(
    agent: AgentInfo,
    commit_messages: list[str],
    learnings: list[str],
) -> list[str]:
    """Build the scope embedding texts for a target agent.

    Combines ownership pattern descriptions, recent commit messages on owned
    files, and existing learnings. Commit messages are the strongest signal
    of actual work semantics.
    """
    texts: list[str] = []

    # Ownership patterns as natural-language descriptions
    for pattern in agent.owns:
        texts.append(f"Owns and maintains: {pattern}")

    # Role description
    if agent.role:
        texts.append(f"Role: {agent.role}")

    # Commit messages (strongest signal of actual work)
    texts.extend(commit_messages)

    # Existing learnings (also define the agent's domain)
    texts.extend(learnings)

    return texts


def find_transfers(
    source_agent: AgentInfo,
    source_learnings: list[str],
    source_embeddings: np.ndarray,
    target_agent: AgentInfo,
    target_scope_embeddings: np.ndarray,
    target_learnings_embeddings: np.ndarray,
    target_learnings: list[str],
    scope_threshold: float,
    knowledge_threshold: float,
    transfer_threshold: float,
) -> list[TransferSuggestion]:
    """Find transfer candidates from source to target agent."""
    if source_embeddings.size == 0 or target_scope_embeddings.size == 0:
        return []

    # Compute scope similarity: how relevant is each source learning to target's domain?
    scope_sim_matrix = cosine_similarity(source_embeddings, target_scope_embeddings)
    scope_max_sims = scope_sim_matrix.max(axis=1)

    # Compute knowledge similarity: does target already know something similar?
    if target_learnings_embeddings.size > 0:
        knowledge_sim_matrix = cosine_similarity(source_embeddings, target_learnings_embeddings)
        knowledge_max_sims = knowledge_sim_matrix.max(axis=1)
    else:
        # Target has no learnings — everything is novel
        knowledge_max_sims = np.zeros(len(source_learnings))

    suggestions: list[TransferSuggestion] = []

    for i, learning in enumerate(source_learnings):
        scope_sim = float(scope_max_sims[i])
        knowledge_sim = float(knowledge_max_sims[i])
        transfer_value = scope_sim - knowledge_sim

        if (
            scope_sim >= scope_threshold
            and knowledge_sim < knowledge_threshold
            and transfer_value >= transfer_threshold
        ):
            suggestions.append(TransferSuggestion(
                from_agent=source_agent.name,
                to_agent=target_agent.name,
                learning=learning,
                scope_similarity=scope_sim,
                knowledge_similarity=knowledge_sim,
                transfer_value=transfer_value,
                rationale=(
                    f"Relevant to {target_agent.name}'s "
                    f"{', '.join(target_agent.owns)} scope "
                    f"but not in their learnings"
                ),
            ))

    # Sort by transfer value descending
    suggestions.sort(key=lambda s: s.transfer_value, reverse=True)
    return suggestions


# ---------------------------------------------------------------------------
# Main analysis
# ---------------------------------------------------------------------------

def analyze(
    source_filter: str | None = None,
    scope_threshold: float = 0.4,
    knowledge_threshold: float = 0.6,
    transfer_threshold: float = 0.15,
    since: str = "90d",
    model_name: str = "all-mpnet-base-v2",
) -> TransferResult:
    """Run full transfer analysis across all agent pairs."""
    repo_root = find_repo_root()
    team = load_team_yaml(repo_root)
    agents = get_all_agents(team)

    if not agents:
        print("Error: no agents found in team.yaml", file=sys.stderr)
        sys.exit(1)

    since_friendly = parse_since(since)

    # Pre-load all data
    agent_learnings: dict[str, list[str]] = {}
    agent_commits: dict[str, list[str]] = {}

    for agent in agents:
        agent_learnings[agent.name] = parse_learnings(repo_root, agent.name)
        agent_commits[agent.name] = get_commit_messages(
            repo_root, agent.owns, since_friendly,
        )

    # Filter source agents if specified
    source_agents = agents
    if source_filter:
        source_agents = [a for a in agents if a.name == source_filter]
        if not source_agents:
            available = [a.name for a in agents]
            print(
                f"Error: agent '{source_filter}' not found. Available: {', '.join(available)}",
                file=sys.stderr,
            )
            sys.exit(1)

    # Collect all texts that need embedding (batch for efficiency)
    all_texts: list[str] = []
    text_registry: dict[str, tuple[int, int]] = {}  # key -> (start_idx, count)

    for agent in agents:
        learnings = agent_learnings[agent.name]
        if learnings:
            key = f"learnings:{agent.name}"
            text_registry[key] = (len(all_texts), len(learnings))
            all_texts.extend(learnings)

        scope_texts = build_scope_texts(
            agent, agent_commits[agent.name], agent_learnings[agent.name],
        )
        if scope_texts:
            key = f"scope:{agent.name}"
            text_registry[key] = (len(all_texts), len(scope_texts))
            all_texts.extend(scope_texts)

    if not all_texts:
        return TransferResult(transfers=[])

    # Single batch embedding
    print(f"Loading model: {model_name}", file=sys.stderr)
    model = SentenceTransformer(model_name)

    print(f"Embedding {len(all_texts)} texts across {len(agents)} agents...", file=sys.stderr)
    all_embeddings = model.encode(all_texts, show_progress_bar=False, convert_to_numpy=True)

    def get_embeddings(key: str) -> np.ndarray:
        if key not in text_registry:
            return np.empty((0, 0))
        start, count = text_registry[key]
        return all_embeddings[start : start + count]

    # Find transfers across all pairs
    all_transfers: list[TransferSuggestion] = []

    for source in source_agents:
        source_learnings = agent_learnings[source.name]
        if not source_learnings:
            continue

        source_emb = get_embeddings(f"learnings:{source.name}")

        for target in agents:
            if target.name == source.name:
                continue  # skip self-comparison

            target_scope_emb = get_embeddings(f"scope:{target.name}")
            target_learnings_emb = get_embeddings(f"learnings:{target.name}")

            transfers = find_transfers(
                source_agent=source,
                source_learnings=source_learnings,
                source_embeddings=source_emb,
                target_agent=target,
                target_scope_embeddings=target_scope_emb,
                target_learnings_embeddings=target_learnings_emb,
                target_learnings=agent_learnings[target.name],
                scope_threshold=scope_threshold,
                knowledge_threshold=knowledge_threshold,
                transfer_threshold=transfer_threshold,
            )
            all_transfers.extend(transfers)

    # Sort all transfers by transfer value descending
    all_transfers.sort(key=lambda t: t.transfer_value, reverse=True)

    return TransferResult(transfers=all_transfers)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _truncate(text: str, max_len: int) -> str:
    if len(text) <= max_len:
        return text
    return text[: max_len - 3] + "..."


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(
        description=(
            "Detect cross-agent knowledge transfer opportunities using "
            "embedding similarity between learnings and ownership scopes."
        ),
    )
    parser.add_argument(
        "--source",
        help="Analyze only this agent as source (default: all agents)",
    )
    parser.add_argument(
        "--scope-threshold", type=float, default=0.4,
        help="Minimum scope similarity to consider relevant (default: 0.4)",
    )
    parser.add_argument(
        "--knowledge-threshold", type=float, default=0.6,
        help="Maximum knowledge similarity before considering 'already known' (default: 0.6)",
    )
    parser.add_argument(
        "--transfer-threshold", type=float, default=0.15,
        help="Minimum transfer value (scope_sim - knowledge_sim) to suggest (default: 0.15)",
    )
    parser.add_argument(
        "--since", default="90d",
        help="Commit history window for scope embeddings (default: 90d)",
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
        source_filter=args.source,
        scope_threshold=args.scope_threshold,
        knowledge_threshold=args.knowledge_threshold,
        transfer_threshold=args.transfer_threshold,
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
