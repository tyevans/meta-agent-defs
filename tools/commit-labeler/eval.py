"""Shared evaluation utilities for commit classifier models."""

from __future__ import annotations

from sklearn.metrics import classification_report, f1_score, confusion_matrix


def evaluate(y_true: list[str], y_pred: list[str], label_names: list[str] | None = None) -> dict:
    """Run standard evaluation and return metrics dict.

    Args:
        y_true: True labels
        y_pred: Predicted labels
        label_names: Optional ordered list of label names. If None, inferred from data.

    Returns:
        dict with keys:
        - report: sklearn classification_report as dict (output_dict=True)
        - f1_macro: float
        - f1_weighted: float
        - accuracy: float
        - top_confusions: list of (count, true_label, pred_label) tuples, top 10
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
    confusions = []
    for i, true_label in enumerate(label_names):
        for j, pred_label in enumerate(label_names):
            if i != j and cm[i][j] > 0:
                confusions.append((cm[i][j], true_label, pred_label))
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
        for count, true, pred in metrics["top_confusions"]:
            print(f"  {true:12s} -> {pred:12s}: {count}")
