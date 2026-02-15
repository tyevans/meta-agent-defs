"""Benchmark embedding models for commit classification.

Compares multiple sentence-transformer models with identical MLP heads
on a frozen test set. This helps select the best embedding model before
fine-tuning or expanding the architecture.

Usage:
    uv run python benchmark.py ../data/labeled-all.jsonl
    uv run python benchmark.py ../data/labeled-all.jsonl --output results/benchmark-2024.json
    uv run python benchmark.py ../data/labeled-all.jsonl --quick  # Skip slow models
"""

from __future__ import annotations

import argparse
import json
import time
from collections import Counter
from pathlib import Path

import numpy as np
from sentence_transformers import SentenceTransformer
from sklearn.neural_network import MLPClassifier
from sklearn.preprocessing import LabelEncoder

from data import freeze_test_set, load_labeled, load_test_set
from eval import evaluate, print_report
from models.embed_mlp import format_input

# Embedding models to benchmark
# Format: (model_id, embedding_dim, approx_params_millions, description)
MODELS = [
    ("sentence-transformers/all-MiniLM-L6-v2", 384, 22, "Current baseline"),
    ("sentence-transformers/all-MiniLM-L12-v2", 384, 33, "Larger MiniLM"),
    ("BAAI/bge-small-en-v1.5", 384, 33, "BGE small"),
    ("thenlper/gte-small", 384, 33, "GTE small"),
    ("intfloat/e5-small-v2", 384, 33, "E5 small"),
    ("nomic-ai/nomic-embed-text-v1.5", 768, 137, "Nomic large"),
]

# ModernBERT-embed-base may not be available yet on HF hub
# Uncomment when available:
# MODELS.append(("answerdotai/ModernBERT-embed-base", 768, 149, "ModernBERT embed"))


class BenchmarkRunner:
    """Runs embedding model benchmarks with identical MLP architectures."""

    def __init__(self, train_records: list[dict], test_records: list[dict]):
        self.train_records = train_records
        self.test_records = test_records
        self.label_encoder = LabelEncoder()

        # Prepare labels
        y_train = [r["label"] for r in train_records]
        y_test = [r["label"] for r in test_records]
        self.label_encoder.fit(y_train)
        self.y_train_encoded = self.label_encoder.transform(y_train)
        self.y_test_encoded = self.label_encoder.transform(y_test)
        self.y_test = y_test

        # Prepare text inputs (use enriched format like embed_mlp)
        self.X_train_text = [format_input(r) for r in train_records]
        self.X_test_text = [format_input(r) for r in test_records]

    def benchmark_model(self, model_id: str) -> dict:
        """Benchmark a single embedding model.

        Returns:
            dict with keys: model_id, embedding_dim, encode_time_s,
            train_time_s, inference_time_s, f1_macro, f1_weighted, accuracy
        """
        print(f"\n{'='*60}")
        print(f"Benchmarking: {model_id}")
        print(f"{'='*60}")

        # Load encoder
        print("  Loading encoder...")
        t0 = time.time()
        encoder = SentenceTransformer(model_id, trust_remote_code=True)
        load_time = time.time() - t0
        print(f"  Loaded in {load_time:.1f}s")

        # Encode training data
        print("  Encoding train set...")
        t0 = time.time()
        X_train_emb = encoder.encode(
            self.X_train_text,
            show_progress_bar=True,
            batch_size=128,
        )
        encode_train_time = time.time() - t0

        # Encode test data
        print("  Encoding test set...")
        t0 = time.time()
        X_test_emb = encoder.encode(
            self.X_test_text,
            show_progress_bar=True,
            batch_size=128,
        )
        encode_test_time = time.time() - t0
        total_encode_time = encode_train_time + encode_test_time

        embedding_dim = X_train_emb.shape[1]
        print(f"  Embedding dim: {embedding_dim}")

        # Train MLP (same architecture as embed_mlp.py)
        print("  Training MLP...")
        t0 = time.time()
        mlp = MLPClassifier(
            hidden_layer_sizes=(128, 64),
            max_iter=500,
            early_stopping=True,
            validation_fraction=0.1,
            random_state=42,
        )
        mlp.fit(X_train_emb, self.y_train_encoded)
        train_time = time.time() - t0
        print(f"  Trained in {train_time:.1f}s")

        # Inference
        print("  Running inference...")
        t0 = time.time()
        y_pred_encoded = mlp.predict(X_test_emb)
        inference_time = time.time() - t0

        # Decode predictions
        y_pred = self.label_encoder.inverse_transform(y_pred_encoded)

        # Evaluate
        print("  Evaluating...")
        metrics = evaluate(self.y_test, y_pred.tolist())

        result = {
            "model_id": model_id,
            "embedding_dim": int(embedding_dim),
            "load_time_s": round(load_time, 2),
            "encode_time_s": round(total_encode_time, 2),
            "train_time_s": round(train_time, 2),
            "inference_time_s": round(inference_time, 2),
            "f1_macro": round(metrics["f1_macro"], 4),
            "f1_weighted": round(metrics["f1_weighted"], 4),
            "accuracy": round(metrics["accuracy"], 4),
        }

        print(f"\n  Results:")
        print(f"    F1 macro:    {result['f1_macro']:.4f}")
        print(f"    F1 weighted: {result['f1_weighted']:.4f}")
        print(f"    Accuracy:    {result['accuracy']:.4f}")
        print(f"    Total time:  {load_time + total_encode_time + train_time + inference_time:.1f}s")

        return result

    def run_all(self, model_configs: list[tuple]) -> list[dict]:
        """Run benchmarks for all models."""
        results = []
        for model_id, dim, params_m, desc in model_configs:
            try:
                result = self.benchmark_model(model_id)
                result["expected_dim"] = dim
                result["params_millions"] = params_m
                result["description"] = desc
                results.append(result)
            except Exception as e:
                print(f"\n  ERROR: Failed to benchmark {model_id}: {e}")
                results.append({
                    "model_id": model_id,
                    "expected_dim": dim,
                    "params_millions": params_m,
                    "description": desc,
                    "error": str(e),
                })
        return results


def print_comparison_table(results: list[dict]):
    """Print a comparison table of all benchmark results."""
    print("\n" + "=" * 100)
    print("BENCHMARK COMPARISON")
    print("=" * 100)

    # Header
    print(f"{'Model':<35s} {'Dim':>5s} {'Params':>7s} {'F1-M':>7s} {'F1-W':>7s} {'Acc':>7s} {'Encode':>8s} {'Train':>7s}")
    print("-" * 100)

    # Sort by F1 macro descending
    valid_results = [r for r in results if "error" not in r]
    sorted_results = sorted(valid_results, key=lambda x: x["f1_macro"], reverse=True)

    for r in sorted_results:
        model_short = r["model_id"].split("/")[-1][:34]
        print(
            f"{model_short:<35s} "
            f"{r['embedding_dim']:>5d} "
            f"{r['params_millions']:>6d}M "
            f"{r['f1_macro']:>7.4f} "
            f"{r['f1_weighted']:>7.4f} "
            f"{r['accuracy']:>7.4f} "
            f"{r['encode_time_s']:>7.1f}s "
            f"{r['train_time_s']:>6.1f}s"
        )

    # Print errors if any
    errors = [r for r in results if "error" in r]
    if errors:
        print("\nErrors:")
        for r in errors:
            print(f"  {r['model_id']}: {r['error']}")


def main():
    parser = argparse.ArgumentParser(
        description="Benchmark embedding models for commit classification"
    )
    parser.add_argument("input", type=Path, help="Labeled JSONL file")
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Path to save benchmark results JSON (default: data/benchmark-results.json)",
    )
    parser.add_argument(
        "--test-size",
        type=float,
        default=0.2,
        help="Test set fraction (default: 0.2)",
    )
    parser.add_argument(
        "--min-confidence",
        type=float,
        default=0.0,
        help="Min confidence to include (0.0 = all)",
    )
    parser.add_argument(
        "--min-class-count",
        type=int,
        default=5,
        help="Drop labels with fewer than N samples",
    )
    parser.add_argument(
        "--reset-test",
        action="store_true",
        help="Re-freeze test set (discards previous)",
    )
    parser.add_argument(
        "--quick",
        action="store_true",
        help="Skip slow models (>100M params)",
    )
    args = parser.parse_args()

    if args.output is None:
        args.output = Path("data/benchmark-results.json")

    test_file = args.input.parent / "test-hashes.json"

    # Load data
    print(f"Loading data from {args.input}")
    records = load_labeled(args.input, args.min_confidence)
    print(f"Loaded {len(records)} labeled commits")

    # Filter rare labels
    counts = Counter(r["label"] for r in records)
    rare = {l for l, c in counts.items() if c < args.min_class_count}
    if rare:
        print(f"Dropping rare labels ({args.min_class_count}+ required): {rare}")
        records = [r for r in records if r["label"] not in rare]

    # Freeze or load test set
    if args.reset_test or not test_file.exists():
        test_hashes = freeze_test_set(records, args.test_size, test_file)
    else:
        test_hashes = load_test_set(test_file)
        print(f"Loaded frozen test set: {len(test_hashes)} hashes from {test_file}")

    # Split using frozen hashes
    test_records = [r for r in records if r["hash"] in test_hashes]
    train_records = [r for r in records if r["hash"] not in test_hashes]

    print(f"\nTrain: {len(train_records)}  Test: {len(test_records)}")

    # Print class distribution
    all_labels = [r["label"] for r in records]
    print(f"\nClass distribution ({len(set(all_labels))} classes, {len(records)} samples):")
    for label, count in Counter(all_labels).most_common():
        print(f"  {label:12s}: {count:4d}")

    # Filter models if --quick
    models_to_test = MODELS
    if args.quick:
        models_to_test = [(m, d, p, desc) for m, d, p, desc in MODELS if p <= 100]
        print(f"\n--quick mode: testing {len(models_to_test)} models (skipping >100M params)")

    # Run benchmarks
    runner = BenchmarkRunner(train_records, test_records)
    results = runner.run_all(models_to_test)

    # Print comparison
    print_comparison_table(results)

    # Save results
    args.output.parent.mkdir(parents=True, exist_ok=True)
    output_data = {
        "metadata": {
            "input_file": str(args.input),
            "test_file": str(test_file),
            "train_size": len(train_records),
            "test_size": len(test_records),
            "num_classes": len(set(all_labels)),
            "min_class_count": args.min_class_count,
            "test_fraction": args.test_size,
        },
        "results": results,
    }
    with open(args.output, "w") as f:
        json.dump(output_data, f, indent=2)
    print(f"\nResults saved to {args.output}")


if __name__ == "__main__":
    main()
