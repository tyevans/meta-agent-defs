"""Audit labeled data by finding disagreements between model and labels.

Trains a model (or loads a saved one), predicts on ALL data, and writes
a JSONL log of every disagreement sorted by model confidence descending.

Usage:
    # Train fresh and audit
    uv run python audit.py ../data/labeled-all.jsonl --model tfidf-logreg

    # Load saved transformer and audit
    uv run python audit.py ../data/labeled-all.jsonl --model transformer --load ../data/transformer-model

    # Only log high-confidence disagreements
    uv run python audit.py ../data/labeled-all.jsonl --model tfidf-logreg --min-confidence 0.7
"""

from __future__ import annotations

import argparse
import json
import pickle
from collections import Counter
from pathlib import Path

import numpy as np

import models.embed_mlp
import models.tfidf_logreg
import models.transformer
from data import load_labeled
from models import MODELS
from models.embed_mlp import format_input
from models.transformer import TransformerClassifier


def main():
    parser = argparse.ArgumentParser(description="Audit labels against model predictions")
    parser.add_argument("input", type=Path, help="Labeled JSONL file")
    parser.add_argument(
        "--model",
        type=str,
        default="tfidf-logreg",
        choices=list(MODELS.keys()),
        help="Model to train or load",
    )
    parser.add_argument(
        "--load",
        type=Path,
        default=None,
        help="Load saved model instead of training (pickle path or transformer dir)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Output JSONL path (default: <input-dir>/audit.jsonl)",
    )
    parser.add_argument(
        "--min-confidence",
        type=float,
        default=0.0,
        help="Only log disagreements where model confidence >= this (default: all)",
    )
    parser.add_argument(
        "--min-class-count",
        type=int,
        default=5,
        help="Drop labels with fewer than N samples before training",
    )
    parser.add_argument(
        "--embed-model",
        type=str,
        default="all-MiniLM-L6-v2",
        help="Sentence transformer model name (for embed-mlp)",
    )
    args = parser.parse_args()

    if args.output is None:
        args.output = args.input.parent / "audit.jsonl"

    # Load all data
    records = load_labeled(args.input)
    print(f"Loaded {len(records)} labeled commits")

    # Filter rare labels
    counts = Counter(r["label"] for r in records)
    rare = {l for l, c in counts.items() if c < args.min_class_count}
    if rare:
        print(f"Dropping rare labels ({args.min_class_count}+ required): {rare}")
        records = [r for r in records if r["label"] not in rare]

    # Prepare inputs
    if args.model == "embed-mlp":
        X = np.array([format_input(r) for r in records])
    else:
        X = np.array([r["message"] for r in records])
    y = np.array([r["label"] for r in records])

    # Load or train model
    if args.load:
        print(f"Loading model from {args.load}")
        if args.model == "transformer":
            model = TransformerClassifier.load(args.load)
        else:
            with open(args.load, "rb") as f:
                model = pickle.load(f)
    else:
        print(f"Training {args.model} on all {len(records)} samples...")
        if args.model == "tfidf-logreg":
            model = MODELS[args.model]()
        elif args.model == "embed-mlp":
            model = MODELS[args.model](model_name=args.embed_model)
        elif args.model == "transformer":
            model = MODELS[args.model]()
        else:
            model = MODELS[args.model]()
        model.train(X, y)

    # Predict on everything
    print("Predicting on all samples...")
    y_pred = model.predict(X)

    probas = None
    if hasattr(model, "predict_proba"):
        probas = model.predict_proba(X)

    # Find disagreements
    disagreements = []
    for i, (rec, true, pred) in enumerate(zip(records, y, y_pred)):
        if true == pred:
            continue

        entry = {
            "hash": rec["hash"],
            "message": rec["message"],
            "label": true,
            "predicted": pred,
        }

        if probas is not None:
            confidence = float(np.max(probas[i]))
            # Top 3 predictions
            sorted_idx = np.argsort(probas[i])[::-1]
            top3 = [
                {"label": model.classes_[j], "prob": round(float(probas[i][j]), 3)}
                for j in sorted_idx[:3]
            ]
            entry["confidence"] = round(confidence, 3)
            entry["top3"] = top3
        else:
            entry["confidence"] = None
            entry["top3"] = []

        entry["verdict"] = ""  # User fills: keep | relabel | skip

        if entry["confidence"] is None or entry["confidence"] >= args.min_confidence:
            disagreements.append(entry)

    # Sort by confidence descending (highest confidence disagreements first)
    disagreements.sort(key=lambda x: x["confidence"] or 0, reverse=True)

    # Write output
    with open(args.output, "w") as f:
        for d in disagreements:
            f.write(json.dumps(d) + "\n")

    # Summary
    agree = sum(1 for t, p in zip(y, y_pred) if t == p)
    print(f"\nAgreements:    {agree}/{len(y)} ({100*agree/len(y):.1f}%)")
    print(f"Disagreements: {len(disagreements)}/{len(y)} ({100*len(disagreements)/len(y):.1f}%)")
    print(f"\nAudit log written to {args.output}")

    # Show top confusion pairs
    pair_counts: Counter[tuple[str, str]] = Counter()
    for d in disagreements:
        pair_counts[(d["label"], d["predicted"])] += 1

    print(f"\nTop disagreement pairs:")
    for (true, pred), count in pair_counts.most_common(15):
        print(f"  {true:12s} -> {pred:12s}: {count}")

    if probas is not None:
        confs = [d["confidence"] for d in disagreements if d["confidence"] is not None]
        if confs:
            high = sum(1 for c in confs if c >= 0.8)
            med = sum(1 for c in confs if 0.5 <= c < 0.8)
            low = sum(1 for c in confs if c < 0.5)
            print(f"\nConfidence breakdown of disagreements:")
            print(f"  High (>=0.8): {high:4d}  <- review these first (likely label errors)")
            print(f"  Med  (0.5-0.8): {med:4d}  <- ambiguous, worth checking")
            print(f"  Low  (<0.5): {low:4d}  <- model unsure, label probably fine")


if __name__ == "__main__":
    main()
