"""Shared evaluation utilities for commit classifier models."""

from __future__ import annotations

import json
from pathlib import Path

import numpy as np
from sklearn.metrics import classification_report, f1_score, confusion_matrix


def evaluate(y_true: list[str], y_pred: list[str], label_names: list[str] | None = None, messages: list[str] | None = None) -> dict:
    """Run standard evaluation and return metrics dict.

    Args:
        y_true: True labels
        y_pred: Predicted labels
        label_names: Optional ordered list of label names. If None, inferred from data.
        messages: Optional list of commit messages (same length as y_true) for confusion examples.

    Returns:
        dict with keys:
        - report: sklearn classification_report as dict (output_dict=True)
        - f1_macro: float
        - f1_weighted: float
        - accuracy: float
        - top_confusions: list of (count, true_label, pred_label, examples) tuples, top 10
    """
    # Infer label names if not provided
    if label_names is None:
        label_names = sorted(set(y_true) | set(y_pred))

    # Classification report as dict
    report = classification_report(
        y_true, y_pred,
        labels=label_names,
        output_dict=True,
        zero_division=0
    )

    # F1 scores
    f1_macro = f1_score(y_true, y_pred, average="macro", zero_division=0)
    f1_weighted = f1_score(y_true, y_pred, average="weighted", zero_division=0)

    # Accuracy
    accuracy = sum(a == b for a, b in zip(y_true, y_pred)) / len(y_true) if len(y_true) > 0 else 0.0

    # Confusion matrix and top confusions
    cm = confusion_matrix(y_true, y_pred, labels=label_names)

    # Collect example messages per confusion pair
    confusion_examples: dict[tuple[str, str], list[str]] = {}
    if messages is not None:
        for true, pred, msg in zip(y_true, y_pred, messages):
            if true != pred:
                key = (true, pred)
                if key not in confusion_examples:
                    confusion_examples[key] = []
                if len(confusion_examples[key]) < 3:
                    confusion_examples[key].append(msg)

    confusions = []
    for i, true_label in enumerate(label_names):
        for j, pred_label in enumerate(label_names):
            if i != j and cm[i][j] > 0:
                examples = confusion_examples.get((true_label, pred_label), [])
                confusions.append((cm[i][j], true_label, pred_label, examples))
    top_confusions = sorted(confusions, reverse=True)[:10]

    return {
        "report": report,
        "f1_macro": f1_macro,
        "f1_weighted": f1_weighted,
        "accuracy": accuracy,
        "top_confusions": top_confusions,
    }


def print_report(metrics: dict) -> None:
    """Pretty-print evaluation metrics to stdout.

    Args:
        metrics: dict returned by evaluate()
    """
    # Print classification report (reconstruct text format)
    report = metrics["report"]

    print("\nClassification Report:")
    print("-" * 60)

    # Header
    print(f"{'':12s} {'precision':>10s} {'recall':>10s} {'f1-score':>10s} {'support':>10s}")
    print("-" * 60)

    # Per-class metrics
    for label, m in sorted(report.items()):
        if label in ("accuracy", "macro avg", "weighted avg"):
            continue
        print(f"{label:12s} {m['precision']:10.2f} {m['recall']:10.2f} {m['f1-score']:10.2f} {m['support']:10.0f}")

    print("-" * 60)

    # Summary metrics
    print(f"{'accuracy':12s} {metrics['accuracy']:10.2f} {' ':>10s} {' ':>10s} {report['macro avg']['support']:10.0f}")
    print(f"{'macro avg':12s} {report['macro avg']['precision']:10.2f} {report['macro avg']['recall']:10.2f} {report['macro avg']['f1-score']:10.2f} {report['macro avg']['support']:10.0f}")
    print(f"{'weighted avg':12s} {report['weighted avg']['precision']:10.2f} {report['weighted avg']['recall']:10.2f} {report['weighted avg']['f1-score']:10.2f} {report['weighted avg']['support']:10.0f}")

    # Summary line
    print(f"\n  acc={metrics['accuracy']:.3f}  F1w={metrics['f1_weighted']:.3f}  F1m={metrics['f1_macro']:.3f}")

    # Top confusions
    if metrics["top_confusions"]:
        print("\nTop confusions (true -> predicted, count):")
        for entry in metrics["top_confusions"]:
            count, true, pred = entry[0], entry[1], entry[2]
            examples = entry[3] if len(entry) > 3 else []
            print(f"  {true:12s} -> {pred:12s}: {count}")
            for ex in examples:
                # Truncate long messages
                display = ex[:80] + "..." if len(ex) > 80 else ex
                print(f"    e.g. {display}")


def run_eval(
    y_true: list[str],
    y_pred: list[str],
    label_names: list[str] | None = None,
    y_proba: np.ndarray | None = None,
    output_path: Path | str | None = None,
    model_name: str | None = None,
    messages: list[str] | None = None,
) -> dict:
    """Standardized evaluation harness for all commit classifiers.

    This is the canonical evaluation function that all models should use.
    Runs standard metrics, prints results, and optionally saves to JSON for
    cross-model comparison.

    Args:
        y_true: True labels (list of strings)
        y_pred: Predicted labels (list of strings)
        label_names: Optional ordered list of label names. If None, inferred from data.
        y_proba: Optional probability matrix (n_samples x n_classes) for probability-based metrics
        output_path: Optional path to save JSON results for cross-model comparison
        model_name: Optional model name to include in saved results (e.g., "tfidf-logreg")
        messages: Optional list of commit messages for confusion examples.

    Returns:
        dict with keys:
        - report: sklearn classification_report as dict
        - f1_macro: float
        - f1_weighted: float
        - accuracy: float
        - top_confusions: list of (count, true_label, pred_label, examples) tuples
        - proba_metrics: dict (only if y_proba provided) with avg_confidence, etc.

    Side effects:
        - Prints formatted report to stdout
        - Saves JSON to output_path if provided
    """
    # Run standard evaluation
    metrics = evaluate(y_true, y_pred, label_names, messages=messages)

    # Add probability-based metrics if available
    if y_proba is not None:
        # Average confidence of predictions
        pred_confidences = np.max(y_proba, axis=1)
        avg_confidence = float(np.mean(pred_confidences))
        min_confidence = float(np.min(pred_confidences))
        max_confidence = float(np.max(pred_confidences))

        # Confidence on correct vs incorrect predictions
        correct_mask = np.array(y_true) == np.array(y_pred)
        avg_conf_correct = float(np.mean(pred_confidences[correct_mask])) if correct_mask.any() else 0.0
        avg_conf_incorrect = float(np.mean(pred_confidences[~correct_mask])) if (~correct_mask).any() else 0.0

        metrics["proba_metrics"] = {
            "avg_confidence": avg_confidence,
            "min_confidence": min_confidence,
            "max_confidence": max_confidence,
            "avg_conf_correct": avg_conf_correct,
            "avg_conf_incorrect": avg_conf_incorrect,
        }

    # Print report
    print_report(metrics)

    # Print probability metrics if available
    if "proba_metrics" in metrics:
        pm = metrics["proba_metrics"]
        print("\nProbability Metrics:")
        print(f"  Avg confidence: {pm['avg_confidence']:.3f}")
        print(f"  Confidence on correct: {pm['avg_conf_correct']:.3f}")
        print(f"  Confidence on incorrect: {pm['avg_conf_incorrect']:.3f}")
        print(f"  Range: [{pm['min_confidence']:.3f}, {pm['max_confidence']:.3f}]")

    # Save to JSON if output path provided
    if output_path is not None:
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Prepare JSON-serializable dict
        result = {
            "model": model_name or "unknown",
            "metrics": {
                "accuracy": metrics["accuracy"],
                "f1_macro": metrics["f1_macro"],
                "f1_weighted": metrics["f1_weighted"],
            },
            "report": metrics["report"],
            "top_confusions": [
                {"count": int(entry[0]), "true": entry[1], "pred": entry[2], "examples": entry[3] if len(entry) > 3 else []}
                for entry in metrics["top_confusions"]
            ],
        }

        if "proba_metrics" in metrics:
            result["proba_metrics"] = metrics["proba_metrics"]

        with open(output_path, "w") as f:
            json.dump(result, f, indent=2)

        print(f"\nResults saved to {output_path}")

    return metrics
