#!/usr/bin/env python3
"""Benchmark multiple sentence transformer models with identical MLP heads.

This is a STANDALONE benchmark script (not a model implementation).
Compares embedding backbones by training the same MLP architecture on each.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
from sentence_transformers import SentenceTransformer
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import LabelEncoder
from torch.utils.data import DataLoader, TensorDataset

from data import load_labeled
from eval import run_eval
from models.embed_mlp import format_input


# Default embedding models to benchmark
DEFAULT_MODELS = [
    "all-MiniLM-L6-v2",
    "nomic-ai/nomic-embed-text-v1.5",
    "BAAI/bge-small-en-v1.5",
    "thenlper/gte-small",
    "intfloat/e5-small-v2",
]


class PyTorchMLP(nn.Module):
    """Simple PyTorch MLP for classification.

    Identical to models/embed_mlp.py:PyTorchMLP for fair comparison.
    """

    def __init__(self, input_dim: int, hidden_dims: tuple[int, ...], num_classes: int):
        super().__init__()
        layers = []
        prev_dim = input_dim
        for hidden_dim in hidden_dims:
            layers.append(nn.Linear(prev_dim, hidden_dim))
            layers.append(nn.ReLU())
            layers.append(nn.Dropout(0.2))
            prev_dim = hidden_dim
        layers.append(nn.Linear(prev_dim, num_classes))
        self.network = nn.Sequential(*layers)

    def forward(self, x):
        return self.network(x)


def train_mlp(
    X_train: np.ndarray,
    y_train: np.ndarray,
    num_classes: int,
    epochs: int = 100,
    batch_size: int = 128,
    device: str = "cuda",
) -> PyTorchMLP:
    """Train MLP on embeddings.

    Args:
        X_train: Embeddings (n_samples x embedding_dim)
        y_train: Encoded labels (n_samples,)
        num_classes: Number of output classes
        epochs: Maximum number of epochs
        batch_size: Batch size for training
        device: torch device ("cuda" or "cpu")

    Returns:
        Trained MLP model
    """
    input_dim = X_train.shape[1]
    mlp = PyTorchMLP(
        input_dim=input_dim,
        hidden_dims=(128, 64),
        num_classes=num_classes,
    ).to(device)

    # Prepare data
    X_tensor = torch.FloatTensor(X_train).to(device)
    y_tensor = torch.LongTensor(y_train).to(device)
    dataset = TensorDataset(X_tensor, y_tensor)
    dataloader = DataLoader(dataset, batch_size=batch_size, shuffle=True)

    # Setup training
    criterion = nn.CrossEntropyLoss()
    optimizer = optim.Adam(mlp.parameters(), lr=0.001)

    # Training loop with early stopping
    mlp.train()
    best_loss = float('inf')
    patience = 10
    patience_counter = 0

    for epoch in range(epochs):
        epoch_loss = 0.0
        for batch_X, batch_y in dataloader:
            optimizer.zero_grad()
            outputs = mlp(batch_X)
            loss = criterion(outputs, batch_y)
            loss.backward()
            optimizer.step()
            epoch_loss += loss.item()

        epoch_loss /= len(dataloader)

        # Early stopping
        if epoch_loss < best_loss:
            best_loss = epoch_loss
            patience_counter = 0
        else:
            patience_counter += 1
            if patience_counter >= patience:
                break

    return mlp


def predict_with_mlp(mlp: PyTorchMLP, X: np.ndarray, device: str = "cuda") -> tuple[np.ndarray, np.ndarray]:
    """Predict with MLP.

    Args:
        mlp: Trained MLP model
        X: Embeddings (n_samples x embedding_dim)
        device: torch device

    Returns:
        (predictions, probabilities) where predictions are class indices (n_samples,)
        and probabilities are (n_samples x n_classes)
    """
    mlp.eval()
    X_tensor = torch.FloatTensor(X).to(device)
    with torch.no_grad():
        outputs = mlp(X_tensor)
        probas = torch.softmax(outputs, dim=1).cpu().numpy()
        preds = outputs.argmax(dim=1).cpu().numpy()
    return preds, probas


def benchmark_model(
    model_name: str,
    X_train: np.ndarray,
    y_train: np.ndarray,
    X_test: np.ndarray,
    y_test: np.ndarray,
    label_encoder: LabelEncoder,
    epochs: int,
    batch_size: int,
    device: str,
    messages_test: list[str],
) -> dict:
    """Benchmark a single embedding model.

    Args:
        model_name: Name of the sentence-transformers model
        X_train: Training embeddings (already encoded)
        y_train: Training labels (encoded as integers)
        X_test: Test embeddings (already encoded)
        y_test: Test labels (encoded as integers)
        label_encoder: LabelEncoder for decoding predictions
        epochs: Max training epochs
        batch_size: Training batch size
        device: torch device
        messages_test: Test commit messages for confusion examples

    Returns:
        dict with model_name, embedding_dim, and metrics
    """
    print(f"\n{'=' * 80}")
    print(f"Training MLP for: {model_name}")
    print(f"{'=' * 80}")

    num_classes = len(label_encoder.classes_)
    embedding_dim = X_train.shape[1]

    # Train MLP
    mlp = train_mlp(
        X_train=X_train,
        y_train=y_train,
        num_classes=num_classes,
        epochs=epochs,
        batch_size=batch_size,
        device=device,
    )

    # Predict on test set
    y_pred_encoded, y_proba = predict_with_mlp(mlp, X_test, device=device)

    # Decode predictions
    y_pred = label_encoder.inverse_transform(y_pred_encoded)
    y_test_decoded = label_encoder.inverse_transform(y_test)

    # Evaluate
    metrics = run_eval(
        y_true=y_test_decoded.tolist(),
        y_pred=y_pred.tolist(),
        label_names=label_encoder.classes_.tolist(),
        y_proba=y_proba,
        model_name=model_name,
        messages=messages_test,
    )

    return {
        "model_name": model_name,
        "embedding_dim": embedding_dim,
        "accuracy": metrics["accuracy"],
        "f1_macro": metrics["f1_macro"],
        "f1_weighted": metrics["f1_weighted"],
    }


def print_comparison_table(results: list[dict]):
    """Print a formatted comparison table.

    Args:
        results: List of result dicts from benchmark_model()
    """
    print("\n" + "=" * 100)
    print("BENCHMARK RESULTS")
    print("=" * 100)

    # Header
    print(f"{'Model':<45s} {'Dim':>6s} {'Accuracy':>10s} {'F1 Macro':>10s} {'F1 Weighted':>10s}")
    print("-" * 100)

    # Sort by F1 macro descending
    sorted_results = sorted(results, key=lambda x: x["f1_macro"], reverse=True)

    for r in sorted_results:
        print(
            f"{r['model_name']:<45s} "
            f"{r['embedding_dim']:>6d} "
            f"{r['accuracy']:>10.3f} "
            f"{r['f1_macro']:>10.3f} "
            f"{r['f1_weighted']:>10.3f}"
        )

    print("=" * 100)


def main():
    parser = argparse.ArgumentParser(
        description="Benchmark multiple sentence transformer models with identical MLP heads"
    )
    parser.add_argument(
        "--data",
        type=Path,
        required=True,
        help="Path to labeled JSONL file",
    )
    parser.add_argument(
        "--models",
        type=str,
        nargs="+",
        default=DEFAULT_MODELS,
        help=f"List of sentence-transformers model names (default: {len(DEFAULT_MODELS)} models)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Optional path to save JSON results",
    )
    parser.add_argument(
        "--epochs",
        type=int,
        default=100,
        help="Max training epochs (default: 100)",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=128,
        help="Training batch size (default: 128)",
    )
    parser.add_argument(
        "--test-size",
        type=float,
        default=0.2,
        help="Test set fraction (default: 0.2)",
    )
    parser.add_argument(
        "--random-state",
        type=int,
        default=42,
        help="Random seed for train/test split (default: 42)",
    )

    args = parser.parse_args()

    # Device selection
    device = "cuda" if torch.cuda.is_available() else "cpu"
    print(f"Using device: {device}")

    # Load data
    print(f"\nLoading data from {args.data}")
    records = load_labeled(args.data)
    print(f"Loaded {len(records)} records")

    # Format texts
    texts = [format_input(r) for r in records]
    labels = [r["label"] for r in records]
    messages = [r["message"] for r in records]

    # Encode labels
    label_encoder = LabelEncoder()
    y_encoded = label_encoder.fit_transform(labels)

    print(f"Classes: {len(label_encoder.classes_)}")
    print(f"Labels: {', '.join(label_encoder.classes_)}")

    # Split data
    texts_train, texts_test, y_train, y_test, messages_train, messages_test = train_test_split(
        texts,
        y_encoded,
        messages,
        test_size=args.test_size,
        random_state=args.random_state,
        stratify=y_encoded,
    )

    print(f"Train size: {len(texts_train)}")
    print(f"Test size: {len(texts_test)}")

    # Benchmark each model
    results = []

    for model_name in args.models:
        try:
            print(f"\n{'=' * 80}")
            print(f"Encoding with: {model_name}")
            print(f"{'=' * 80}")

            # Load encoder
            encoder = SentenceTransformer(model_name)

            # Encode texts (do this once per model)
            print("Encoding train set...")
            X_train = encoder.encode(texts_train, show_progress_bar=True, batch_size=args.batch_size)

            print("Encoding test set...")
            X_test = encoder.encode(texts_test, show_progress_bar=True, batch_size=args.batch_size)

            # Benchmark this model
            result = benchmark_model(
                model_name=model_name,
                X_train=X_train,
                y_train=y_train,
                X_test=X_test,
                y_test=y_test,
                label_encoder=label_encoder,
                epochs=args.epochs,
                batch_size=args.batch_size,
                device=device,
                messages_test=messages_test,
            )

            results.append(result)

        except Exception as e:
            print(f"ERROR: Failed to benchmark {model_name}: {e}")
            continue

    # Print comparison table
    if results:
        print_comparison_table(results)

        # Save results to JSON if requested
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            with open(args.output, "w") as f:
                json.dump(results, f, indent=2)
            print(f"\nResults saved to {args.output}")
    else:
        print("\nERROR: No models were successfully benchmarked")
        sys.exit(1)


if __name__ == "__main__":
    main()
