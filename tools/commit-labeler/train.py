"""Train commit classifiers using the model registry.

Usage:
    uv run python train.py ../data/labeled-all.jsonl --model tfidf-logreg
    uv run python train.py ../data/relabeled.jsonl --model embed-mlp
    uv run python train.py ../data/labeled-all.jsonl --model tfidf-logreg --reset-test
    uv run python train.py ../data/labeled-all.jsonl --model ensemble --ensemble-models model1.pkl transformer-model/ --ensemble-strategy soft_vote

    # Track experiments in results registry
    uv run python train.py ../data/labeled-all.jsonl --model tfidf-logreg --eval-output results/eval.json --registry
    python registry.py  # compare all registered experiments
"""

from __future__ import annotations

import argparse
import pickle
from collections import Counter
from pathlib import Path

import numpy as np
from sklearn.model_selection import cross_val_score

# Import model modules to trigger registration
import models.embed_mlp
import models.ensemble
import models.setfit
import models.tfidf_logreg
import models.transformer
from data import freeze_test_set, load_labeled, load_test_set
from eval import run_eval
from models import MODELS
from models.embed_mlp import format_input
from registry import register_result


def main():
    parser = argparse.ArgumentParser(description="Train commit message classifier")
    parser.add_argument("input", type=Path, help="Labeled JSONL file")
    parser.add_argument(
        "--model",
        type=str,
        default="tfidf-logreg",
        choices=list(MODELS.keys()),
        help="Model to use for training",
    )
    parser.add_argument(
        "--min-confidence",
        type=float,
        default=0.0,
        help="Min confidence to include (0.0 = all)",
    )
    parser.add_argument(
        "--test-size",
        type=float,
        default=0.2,
        help="Fraction held out for test (default 0.2)",
    )
    parser.add_argument(
        "--output", type=Path, default=None, help="Path to save pickled model"
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
    # Embed-MLP specific args
    parser.add_argument(
        "--embed-model",
        type=str,
        default="all-MiniLM-L6-v2",
        help="Sentence transformer model name (for embed-mlp)",
    )
    parser.add_argument(
        "--top-k",
        type=int,
        default=3,
        help="Number of top labels to show per prediction (for embed-mlp)",
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=0.15,
        help="Minimum probability for top-k (for embed-mlp)",
    )
    parser.add_argument(
        "--eval-output",
        type=Path,
        default=None,
        help="Path to save evaluation results JSON for cross-model comparison",
    )
    # Class imbalance handling
    parser.add_argument(
        "--class-weight",
        type=str,
        default="none",
        choices=["none", "balanced", "auto"],
        help="Class weighting strategy: none (no weights), balanced (sklearn balanced), auto (inverse frequency)",
    )
    parser.add_argument(
        "--focal-loss",
        action="store_true",
        help="Use focal loss instead of cross-entropy (for embed-mlp and transformer)",
    )
    parser.add_argument(
        "--focal-gamma",
        type=float,
        default=2.0,
        help="Focal loss gamma parameter (default 2.0, higher = more focus on hard examples)",
    )
    parser.add_argument(
        "--export-onnx",
        type=Path,
        default=None,
        help="Export transformer model to ONNX format at this path (transformer only)",
    )
    # SetFit-specific args
    parser.add_argument(
        "--setfit-backbone",
        type=str,
        default="sentence-transformers/all-mpnet-base-v2",
        help="Sentence-transformer backbone for SetFit (default: all-mpnet-base-v2)",
    )
    parser.add_argument(
        "--samples-per-class",
        type=int,
        default=None,
        help="Limit training to K samples per class for SetFit few-shot experiments (8/16/64/None=all)",
    )
    parser.add_argument(
        "--setfit-iterations",
        type=int,
        default=20,
        help="Number of contrastive training iterations for SetFit (default 20)",
    )
    parser.add_argument(
        "--setfit-epochs",
        type=int,
        default=1,
        help="Number of epochs for SetFit classification head (default 1)",
    )
    # Ensemble-specific args
    parser.add_argument(
        "--ensemble-models",
        nargs="+",
        type=Path,
        default=None,
        help="Paths to pre-trained sub-models for ensemble (pickle files or model directories)",
    )
    parser.add_argument(
        "--ensemble-strategy",
        type=str,
        default="soft_vote",
        choices=["majority_vote", "soft_vote", "weighted_soft_vote"],
        help="Ensemble combination strategy (default: soft_vote)",
    )
    parser.add_argument(
        "--ensemble-weights",
        nargs="+",
        type=float,
        default=None,
        help="Weights for weighted_soft_vote strategy (one per sub-model)",
    )
    parser.add_argument(
        "--registry",
        action="store_true",
        help="Register experiment result in results registry (requires --eval-output)",
    )
    args = parser.parse_args()

    if args.output is None:
        args.output = args.input.parent / "classifier.pkl"

    test_file = args.input.parent / "test-hashes.json"

    # Handle standalone ONNX export (load existing model and export without training)
    if args.export_onnx and args.model == "transformer":
        # Check if saved model exists
        save_dir = args.output.parent / "transformer-model"
        if save_dir.exists():
            print(f"Loading existing transformer model from {save_dir}...")
            from models.transformer import TransformerClassifier
            model = TransformerClassifier.load(save_dir)
            print(f"Exporting to ONNX at {args.export_onnx}...")
            model.export_onnx(args.export_onnx)
            print("Standalone ONNX export complete. Exiting.")
            return
        else:
            print(f"Warning: No saved model found at {save_dir}. Will train and export.")

    # Load data
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
        print(
            f"Loaded frozen test set: {len(test_hashes)} hashes from {test_file}"
        )

    # Split using frozen hashes
    test_records = [r for r in records if r["hash"] in test_hashes]
    train_records = [r for r in records if r["hash"] not in test_hashes]

    # Prepare X based on model type
    if args.model == "embed-mlp":
        X_train = np.array([format_input(r) for r in train_records])
        X_test = np.array([format_input(r) for r in test_records])
        print(f"\nExample enriched input (embed-mlp):")
        print(f"  {X_train[0][:120]}")
    else:
        X_train = np.array([r["message"] for r in train_records])
        X_test = np.array([r["message"] for r in test_records])

    y_train = np.array([r["label"] for r in train_records])
    y_test = np.array([r["label"] for r in test_records])

    # Print class distribution
    all_labels = [r["label"] for r in records]
    print(
        f"\nClass distribution ({len(set(all_labels))} classes, {len(records)} samples):"
    )
    for label, count in Counter(all_labels).most_common():
        print(f"  {label:12s}: {count:4d}")

    print(f"\nTrain: {len(X_train)}  Test: {len(X_test)} (frozen)")

    # Instantiate model with class weighting parameters
    print(f"\nUsing model: {args.model}")
    print(f"  Class weight mode: {args.class_weight}")
    if args.focal_loss:
        print(f"  Focal loss: enabled (gamma={args.focal_gamma})")

    if args.model == "tfidf-logreg":
        # Map our CLI args to sklearn's class_weight parameter
        if args.class_weight == "none":
            cw = None
        elif args.class_weight in ("balanced", "auto"):
            cw = "balanced"  # sklearn only supports 'balanced' or None or dict
        else:
            cw = args.class_weight
        model = MODELS[args.model](class_weight=cw)
        if args.focal_loss:
            print("  Warning: focal loss not supported for tfidf-logreg (ignored)")
    elif args.model == "embed-mlp":
        model = MODELS[args.model](
            model_name=args.embed_model,
            class_weight_mode=args.class_weight,
            use_focal_loss=args.focal_loss,
            focal_gamma=args.focal_gamma,
        )
    elif args.model == "transformer":
        model = MODELS[args.model](
            class_weight_mode=args.class_weight,
            use_focal_loss=args.focal_loss,
            focal_gamma=args.focal_gamma,
        )
    elif args.model == "setfit":
        model = MODELS[args.model](
            model_name=args.setfit_backbone,
            samples_per_class=args.samples_per_class,
            num_iterations=args.setfit_iterations,
            num_epochs=args.setfit_epochs,
        )
    elif args.model == "ensemble":
        if not args.ensemble_models:
            parser.error("--ensemble-models is required when using --model ensemble")
        model = MODELS[args.model](
            model_paths=args.ensemble_models,
            weights=args.ensemble_weights,
            strategy=args.ensemble_strategy,
        )
    else:
        model = MODELS[args.model]()

    # Cross-validation on train set (for tfidf-logreg with sklearn pipeline)
    if args.model == "tfidf-logreg" and hasattr(model, "pipeline"):
        cv_scores = cross_val_score(
            model.pipeline, X_train, y_train, cv=5, scoring="f1_macro"
        )
        print(
            f"\n5-fold CV F1 (macro): {cv_scores.mean():.3f} +/- {cv_scores.std():.3f}"
        )

    # Train
    print("\nTraining...")
    model.train(X_train, y_train)

    # Predict
    print("Evaluating...")
    y_pred = model.predict(X_test)

    # Get probabilities if available
    probas = None
    if hasattr(model, "predict_proba"):
        probas = model.predict_proba(X_test)

    # Get raw messages for confusion examples
    test_messages = [r["message"] for r in test_records]

    # Run standardized evaluation harness
    metrics = run_eval(
        y_test.tolist(),
        y_pred.tolist(),
        label_names=model.classes_.tolist(),
        y_proba=probas,
        output_path=args.eval_output,
        model_name=args.model,
        messages=test_messages,
    )

    # Register result in registry if requested
    if args.registry:
        if args.eval_output is None:
            print("\nWarning: --registry requires --eval-output to be set (skipping registration)")
        else:
            # Build metadata from CLI args
            cli_metadata = {
                "model": args.model,
                "cli_args": {
                    "input": str(args.input),
                    "test_size": args.test_size,
                    "min_confidence": args.min_confidence,
                    "min_class_count": args.min_class_count,
                    "class_weight": args.class_weight,
                    "focal_loss": args.focal_loss,
                    "focal_gamma": args.focal_gamma if args.focal_loss else None,
                },
                "train_samples": len(X_train),
                "test_samples": len(X_test),
                "num_classes": len(model.classes_),
            }

            # Add model-specific args
            if args.model == "embed-mlp":
                cli_metadata["cli_args"]["embed_model"] = args.embed_model
            elif args.model == "transformer":
                cli_metadata["cli_args"]["export_onnx"] = str(args.export_onnx) if args.export_onnx else None
            elif args.model == "setfit":
                cli_metadata["cli_args"]["setfit_backbone"] = args.setfit_backbone
                cli_metadata["cli_args"]["samples_per_class"] = args.samples_per_class
                cli_metadata["cli_args"]["setfit_iterations"] = args.setfit_iterations
            elif args.model == "ensemble":
                cli_metadata["cli_args"]["ensemble_models"] = [str(p) for p in args.ensemble_models]
                cli_metadata["cli_args"]["ensemble_strategy"] = args.ensemble_strategy
                cli_metadata["cli_args"]["ensemble_weights"] = args.ensemble_weights

            # Merge metrics into metadata
            result_data = {**metrics, **cli_metadata}

            # Register
            registry_file = register_result(result_data, metadata=cli_metadata)
            print(f"\nResult registered to {registry_file}")

    # Model-specific outputs
    if probas is not None:
        # Top-k accuracy for embed-mlp
        if args.model == "embed-mlp":
            print("\n" + "=" * 60)
            print("TOP-K ACCURACY")
            print("=" * 60)
            y_test_indices = np.array(
                [np.where(model.classes_ == y)[0][0] for y in y_test]
            )
            for k in [1, 2, 3]:
                top_k_indices = np.argsort(probas, axis=1)[:, -k:]
                top_k_hit = [
                    y_test_indices[i] in top_k_indices[i]
                    for i in range(len(y_test))
                ]
                print(
                    f"  Top-{k}: {sum(top_k_hit)}/{len(y_test)} ({100*sum(top_k_hit)/len(y_test):.1f}%)"
                )

    # Top features for tfidf-logreg
    if args.model == "tfidf-logreg" and hasattr(model, "feature_names"):
        print("\nTop 5 features per class:")
        for i, label in enumerate(model.classes_):
            top_idx = model.coef_[i].argsort()[-5:][::-1]
            top_features = model.feature_names[top_idx]
            print(f"  {label:12s}: {', '.join(top_features)}")

    # Save model
    if args.model == "transformer":
        # Transformer uses custom save method
        save_dir = args.output.parent / "transformer-model"
        model.save(save_dir)
        print(f"\nTransformer model saved to {save_dir}")

        # Export to ONNX if requested
        if args.export_onnx:
            print(f"\nExporting to ONNX...")
            model.export_onnx(args.export_onnx)
    elif args.model == "setfit":
        # SetFit saves as directory (like transformer)
        save_dir = args.output.parent / "setfit-model"
        model.save(save_dir)
        print(f"\nSetFit model saved to {save_dir}")

        if args.export_onnx:
            print(f"\nWarning: --export-onnx not supported for SetFit models (ignored)")
    elif args.model == "ensemble":
        # Ensemble saves as directory with config.json
        save_dir = args.output.parent / "ensemble-model"
        model.save(save_dir)
        print(f"\nEnsemble config saved to {save_dir}")

        if args.export_onnx:
            print(f"\nWarning: --export-onnx not supported for ensemble models (ignored)")
    else:
        # Other models use pickle
        with open(args.output, "wb") as f:
            pickle.dump(model, f)
        print(f"\nModel saved to {args.output}")

        # Warn if ONNX export requested for non-transformer model
        if args.export_onnx:
            print(f"\nWarning: --export-onnx only supported for transformer models (ignored)")

    # Sanity check predictions
    test_samples = [
        {"message": "Fix null pointer in auth module"},
        {"message": "Add support for dark mode"},
        {"message": "Update dependencies"},
        {"message": "Refactor database connection pool"},
        {"message": "Add unit tests for parser"},
        {"message": "Update README with new API docs"},
        {"message": "Bump version to 2.0.0"},
        {"message": "Improve query performance with index"},
    ]

    if args.model == "embed-mlp":
        # Use enriched format for embed-mlp
        test_inputs = np.array([format_input(s) for s in test_samples])
    else:
        test_inputs = np.array([s["message"] for s in test_samples])

    print("\nSanity check predictions:")
    preds = model.predict(test_inputs)

    if hasattr(model, "predict_proba"):
        probas = model.predict_proba(test_inputs)
        if args.model == "embed-mlp":
            # Show top-k for embed-mlp
            for sample, proba in zip(test_samples, probas):
                sorted_idx = np.argsort(proba)[::-1]
                top_k = [
                    (model.classes_[j], proba[j])
                    for j in sorted_idx[: args.top_k]
                    if proba[j] >= args.threshold
                ]
                labels_str = ", ".join(f"{l}:{p:.2f}" for l, p in top_k)
                print(f"  [{labels_str}]  {sample['message']}")
        else:
            # Show single prediction with confidence for tfidf-logreg
            for sample, pred, proba in zip(test_samples, preds, probas):
                conf = max(proba)
                print(f"  [{pred:8s} {conf:.2f}] {sample['message']}")
    else:
        for sample, pred in zip(test_samples, preds):
            print(f"  [{pred:8s}] {sample['message']}")


if __name__ == "__main__":
    main()
