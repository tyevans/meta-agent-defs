#!/usr/bin/env python3
"""Difficulty calibration tool for agent challenges.

Tracks challenge outcomes and fits logistic regression to predict
the optimal difficulty level targeting ~70% success rate.
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import numpy as np
from sklearn.linear_model import LogisticRegression
from sklearn.preprocessing import StandardScaler

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

DIFFICULTY_LABELS: dict[int, str] = {
    1: "novice",
    2: "intermediate",
    3: "competent",
    4: "expert",
    5: "adversarial",
}

TARGET_SUCCESS_RATE: float = 0.70
MIN_RECORDS_FOR_MODEL: int = 10
DEFAULT_DIFFICULTY: int = 2
MEMORY_ROOT: Path = Path("memory/agents")


# ---------------------------------------------------------------------------
# Data I/O
# ---------------------------------------------------------------------------

def history_path(agent: str) -> Path:
    """Return the JSONL history path for an agent."""
    return MEMORY_ROOT / agent / "challenge-history.jsonl"


def load_history(agent: str) -> list[dict[str, Any]]:
    """Load challenge history for an agent from JSONL."""
    path = history_path(agent)
    if not path.exists():
        return []
    records: list[dict[str, Any]] = []
    for line_no, line in enumerate(path.read_text().splitlines(), start=1):
        line = line.strip()
        if not line:
            continue
        try:
            records.append(json.loads(line))
        except json.JSONDecodeError:
            print(f"warning: skipping malformed line {line_no} in {path}", file=sys.stderr)
    return records


def append_record(agent: str, record: dict[str, Any]) -> Path:
    """Append a single record to the agent's challenge history JSONL."""
    path = history_path(agent)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a") as f:
        f.write(json.dumps(record, sort_keys=True) + "\n")
    return path


def discover_agents() -> list[str]:
    """Return agents that have challenge history files."""
    agents: list[str] = []
    if not MEMORY_ROOT.exists():
        return agents
    for d in sorted(MEMORY_ROOT.iterdir()):
        if d.is_dir() and (d / "challenge-history.jsonl").exists():
            agents.append(d.name)
    return agents


# ---------------------------------------------------------------------------
# Feature extraction
# ---------------------------------------------------------------------------

def extract_features(records: list[dict[str, Any]]) -> tuple[np.ndarray, np.ndarray]:
    """Extract feature matrix X and binary target y from records.

    Features: [difficulty, learnings_count, category_coverage]
    Target: 1 if outcome == "pass", 0 otherwise.
    """
    X_rows: list[list[float]] = []
    y_rows: list[int] = []
    for r in records:
        X_rows.append([
            float(r["difficulty"]),
            float(r.get("learnings_count", 0)),
            float(r.get("category_coverage", 0.0)),
        ])
        y_rows.append(1 if r.get("outcome") == "pass" else 0)
    return np.array(X_rows), np.array(y_rows)


# ---------------------------------------------------------------------------
# Model fitting
# ---------------------------------------------------------------------------

class CalibrationModel:
    """Wraps logistic regression for difficulty calibration."""

    def __init__(self) -> None:
        self.clf = LogisticRegression(solver="lbfgs", max_iter=1000)
        self.scaler = StandardScaler()
        self.fitted = False
        self.r2: float | None = None

    def fit(self, X: np.ndarray, y: np.ndarray) -> None:
        """Fit the model. Requires both classes present in y."""
        if len(np.unique(y)) < 2:
            # Cannot fit logistic regression with a single class.
            return
        X_scaled = self.scaler.fit_transform(X)
        self.clf.fit(X_scaled, y)
        self.fitted = True
        # Pseudo-R² (McFadden) for goodness-of-fit indication.
        probs = self.clf.predict_proba(X_scaled)[:, 1]
        eps = 1e-15
        probs = np.clip(probs, eps, 1 - eps)
        ll_model = np.sum(y * np.log(probs) + (1 - y) * np.log(1 - probs))
        p_null = np.mean(y)
        ll_null = np.sum(y * np.log(p_null + eps) + (1 - y) * np.log(1 - p_null + eps))
        self.r2 = 1.0 - (ll_model / ll_null) if ll_null != 0 else None

    def predict_success_prob(self, difficulty: int, learnings_count: int, category_coverage: float) -> float:
        """Predict P(success) for a given feature vector."""
        if not self.fitted:
            raise RuntimeError("Model not fitted")
        X = np.array([[float(difficulty), float(learnings_count), float(category_coverage)]])
        X_scaled = self.scaler.transform(X)
        return float(self.clf.predict_proba(X_scaled)[0, 1])

    def find_target_difficulty(
        self,
        learnings_count: int,
        category_coverage: float,
        target: float = TARGET_SUCCESS_RATE,
    ) -> tuple[int, float]:
        """Find integer difficulty closest to target success rate.

        Returns (recommended_difficulty, predicted_success_at_that_level).
        """
        best_diff = DEFAULT_DIFFICULTY
        best_gap = float("inf")
        best_prob = 0.5
        for d in range(1, 6):
            p = self.predict_success_prob(d, learnings_count, category_coverage)
            gap = abs(p - target)
            if gap < best_gap:
                best_gap = gap
                best_diff = d
                best_prob = p
        return best_diff, best_prob


# ---------------------------------------------------------------------------
# Heuristic fallback
# ---------------------------------------------------------------------------

def heuristic_recommend(records: list[dict[str, Any]]) -> tuple[int, str]:
    """Simple heuristic when < MIN_RECORDS_FOR_MODEL data points exist.

    Returns (recommended_difficulty, reason).
    """
    if not records:
        return DEFAULT_DIFFICULTY, "no history — starting at default"

    current = records[-1]["difficulty"]

    if len(records) >= 3:
        last3 = [r.get("outcome") for r in records[-3:]]
        if all(o == "pass" for o in last3):
            new = min(current + 1, 5)
            return new, "last 3 all passed — increasing"
        if all(o == "fail" for o in last3):
            new = max(current - 1, 1)
            return new, "last 3 all failed — decreasing"

    return current, "mixed recent results — staying"


# ---------------------------------------------------------------------------
# Subcommands
# ---------------------------------------------------------------------------

def cmd_record(args: argparse.Namespace) -> None:
    """Record a challenge outcome."""
    record: dict[str, Any] = {
        "challenge_id": args.challenge_id,
        "agent": args.agent,
        "difficulty": args.difficulty,
        "outcome": args.outcome,
        "learnings_count": args.learnings_count,
        "category_coverage": args.category_coverage,
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }
    if args.trap_detected:
        record["trap_detected"] = args.trap_detected
    if args.time_taken is not None:
        record["time_taken"] = args.time_taken
    if args.rework_count is not None:
        record["rework_count"] = args.rework_count

    path = append_record(args.agent, record)
    total = len(load_history(args.agent))
    print(f"Recorded challenge {args.challenge_id} for {args.agent} (difficulty={args.difficulty}, outcome={args.outcome})")
    print(f"History: {path} ({total} records)")


def cmd_recommend(args: argparse.Namespace) -> None:
    """Recommend difficulty for next challenge."""
    records = load_history(args.agent)

    if len(records) < MIN_RECORDS_FOR_MODEL:
        diff, reason = heuristic_recommend(records)
        print(f"Agent: {args.agent}")
        print(f"Method: heuristic ({len(records)}/{MIN_RECORDS_FOR_MODEL} records for model)")
        print(f"Recommended difficulty: {diff} ({DIFFICULTY_LABELS[diff]})")
        print(f"Reason: {reason}")
        return

    X, y = extract_features(records)
    model = CalibrationModel()
    model.fit(X, y)

    if not model.fitted:
        diff, reason = heuristic_recommend(records)
        print(f"Agent: {args.agent}")
        print(f"Method: heuristic (model fit failed — single-class data)")
        print(f"Recommended difficulty: {diff} ({DIFFICULTY_LABELS[diff]})")
        print(f"Reason: {reason}")
        return

    # Use current agent state for prediction.
    current_learnings = args.learnings_count if args.learnings_count is not None else records[-1].get("learnings_count", 0)
    current_coverage = args.category_coverage if args.category_coverage is not None else records[-1].get("category_coverage", 0.0)

    diff, prob = model.find_target_difficulty(current_learnings, current_coverage)
    print(f"Agent: {args.agent}")
    print(f"Method: logistic regression ({len(records)} records)")
    print(f"Recommended difficulty: {diff} ({DIFFICULTY_LABELS[diff]})")
    print(f"Predicted success rate: {prob:.0%}")
    print(f"Target success rate: {TARGET_SUCCESS_RATE:.0%}")
    if model.r2 is not None:
        print(f"McFadden R²: {model.r2:.3f}")


def cmd_report(args: argparse.Namespace) -> None:
    """Show calibration report."""
    agents = [args.agent] if args.agent else discover_agents()

    if not agents:
        print("No agents with challenge history found.")
        return

    for agent in agents:
        _print_report(agent)
        if agent != agents[-1]:
            print("\n---\n")


def _print_report(agent: str) -> None:
    """Print markdown report for a single agent."""
    records = load_history(agent)

    print(f"# Difficulty Calibration: {agent}\n")

    if not records:
        print("No challenge history.\n")
        return

    # --- Challenge History table ---
    print("## Challenge History")
    print("| Date | Difficulty | Outcome | Trap | Learnings | Coverage |")
    print("|------|-----------|---------|------|-----------|----------|")
    for r in records:
        ts = r.get("timestamp", "")[:10]
        diff = r.get("difficulty", "?")
        outcome = r.get("outcome", "?")
        trap = r.get("trap_detected", "-")
        lc = r.get("learnings_count", "-")
        cc = r.get("category_coverage", "-")
        if isinstance(cc, float):
            cc = f"{cc:.2f}"
        print(f"| {ts} | {diff} | {outcome} | {trap} | {lc} | {cc} |")

    # --- Pass rate by difficulty ---
    print("\n## Pass Rate by Difficulty")
    print("| Difficulty | Attempts | Pass Rate |")
    print("|-----------|----------|-----------|")
    for d in range(1, 6):
        subset = [r for r in records if r.get("difficulty") == d]
        if not subset:
            print(f"| {d} ({DIFFICULTY_LABELS[d]}) | 0 | - |")
        else:
            passes = sum(1 for r in subset if r.get("outcome") == "pass")
            rate = passes / len(subset)
            print(f"| {d} ({DIFFICULTY_LABELS[d]}) | {len(subset)} | {rate:.0%} |")

    # --- Model status ---
    print("\n## Model Status")
    if len(records) < MIN_RECORDS_FOR_MODEL:
        print(f"Model: insufficient data ({len(records)}/{MIN_RECORDS_FOR_MODEL} records)")
        diff, reason = heuristic_recommend(records)
        print(f"Current recommended difficulty: {diff} ({DIFFICULTY_LABELS[diff]})")
        print(f"Recommendation basis: heuristic — {reason}")
    else:
        X, y = extract_features(records)
        model = CalibrationModel()
        model.fit(X, y)

        if not model.fitted:
            print("Model: fit failed (single-class data)")
            diff, reason = heuristic_recommend(records)
            print(f"Current recommended difficulty: {diff} ({DIFFICULTY_LABELS[diff]})")
            print(f"Recommendation basis: heuristic — {reason}")
        else:
            r2_str = f"{model.r2:.3f}" if model.r2 is not None else "N/A"
            print(f"Model: fitted ({len(records)} records)")
            print(f"McFadden R²: {r2_str}")

            current_learnings = records[-1].get("learnings_count", 0)
            current_coverage = records[-1].get("category_coverage", 0.0)
            diff, prob = model.find_target_difficulty(current_learnings, current_coverage)

            print(f"Current recommended difficulty: {diff} ({DIFFICULTY_LABELS[diff]})")
            print(f"Target success rate: {TARGET_SUCCESS_RATE:.0%}")
            print(f"Predicted success at recommended difficulty: {prob:.0%}")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="calibration",
        description="Track challenge outcomes and calibrate agent difficulty.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    # -- record --
    rec = sub.add_parser("record", help="Record a challenge outcome")
    rec.add_argument("--agent", required=True, help="Agent name")
    rec.add_argument("--challenge-id", required=True, help="Challenge identifier")
    rec.add_argument("--difficulty", required=True, type=int, choices=range(1, 6), help="Difficulty 1-5")
    rec.add_argument("--outcome", required=True, choices=["pass", "partial", "fail"], help="Challenge result")
    rec.add_argument("--learnings-count", required=True, type=int, help="Agent learnings count")
    rec.add_argument("--category-coverage", required=True, type=float, help="Domain coverage 0-1")
    rec.add_argument("--trap-detected", choices=["caught", "partial", "missed"], help="Trap detection result")
    rec.add_argument("--time-taken", type=float, help="Time taken in minutes")
    rec.add_argument("--rework-count", type=int, help="Number of rework iterations")

    # -- recommend --
    rcm = sub.add_parser("recommend", help="Recommend difficulty for next challenge")
    rcm.add_argument("--agent", required=True, help="Agent name")
    rcm.add_argument("--learnings-count", type=int, help="Override current learnings count")
    rcm.add_argument("--category-coverage", type=float, help="Override current category coverage")

    # -- report --
    rpt = sub.add_parser("report", help="Show calibration report")
    rpt.add_argument("--agent", help="Agent name (omit for all agents)")

    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    dispatch = {
        "record": cmd_record,
        "recommend": cmd_recommend,
        "report": cmd_report,
    }
    dispatch[args.command](args)


if __name__ == "__main__":
    main()
