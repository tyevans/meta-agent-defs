"""Sentence-transformer + MLP commit classifier."""

from __future__ import annotations

import numpy as np
from sentence_transformers import SentenceTransformer
from sklearn.neural_network import MLPClassifier
from sklearn.preprocessing import LabelEncoder

from . import MODELS, ModelProtocol


def format_input(record: dict) -> str:
    """Build a single string from message + diff metadata for embedding."""
    msg = record["message"]
    diff = record.get("diff")
    if not diff:
        return msg

    parts = [msg]

    # File paths (truncate long lists)
    files = diff.get("files", [])
    if files:
        file_str = ", ".join(files[:6])
        if len(files) > 6:
            file_str += f" (+{len(files) - 6} more)"
        parts.append(f"files: {file_str}")

    # Extensions
    exts = sorted(set(diff.get("extensions", [])))
    if exts:
        parts.append(f"ext: {' '.join(exts)}")

    # Size
    parts.append(f"+{diff.get('insertions', 0)} -{diff.get('deletions', 0)}")

    return " | ".join(parts)


class EmbedMLP:
    """Sentence-transformer embedding + MLP classifier.

    Implements ModelProtocol. This model works in two stages:
    1. Encode text using a sentence-transformer model
    2. Classify embeddings with an MLP

    X for train/predict can be either:
    - Pre-computed embeddings (np.ndarray of floats)
    - Text strings (will be encoded using the sentence-transformer)
    """

    def __init__(self, model_name: str = "all-MiniLM-L6-v2"):
        self.model_name = model_name
        self.encoder: SentenceTransformer | None = None
        self.mlp = MLPClassifier(
            hidden_layer_sizes=(128, 64),
            max_iter=500,
            early_stopping=True,
            validation_fraction=0.1,
            random_state=42,
        )
        self.label_encoder = LabelEncoder()

    def _ensure_encoder(self):
        """Lazy-load the sentence transformer."""
        if self.encoder is None:
            self.encoder = SentenceTransformer(self.model_name)

    def encode(self, texts: list[str], batch_size: int = 128) -> np.ndarray:
        """Encode text into embeddings."""
        self._ensure_encoder()
        return self.encoder.encode(texts, show_progress_bar=True, batch_size=batch_size)

    def train(self, X: np.ndarray, y: np.ndarray) -> EmbedMLP:
        """Train on embeddings (or text) and labels.

        If X contains strings, encodes them first.
        y should be string labels — LabelEncoder handles encoding.
        """
        # Encode if text
        if X.dtype.kind in ('U', 'O'):  # unicode or object (strings)
            X = self.encode(X.tolist())

        self.label_encoder.fit(y)
        y_encoded = self.label_encoder.transform(y)
        self.mlp.fit(X, y_encoded)
        return self

    def predict(self, X: np.ndarray) -> np.ndarray:
        """Predict labels. Returns string labels."""
        if X.dtype.kind in ('U', 'O'):
            X = self.encode(X.tolist())
        y_encoded = self.mlp.predict(X)
        return self.label_encoder.inverse_transform(y_encoded)

    def predict_proba(self, X: np.ndarray) -> np.ndarray:
        """Predict probabilities for all classes."""
        if X.dtype.kind in ('U', 'O'):
            X = self.encode(X.tolist())
        return self.mlp.predict_proba(X)

    @property
    def classes_(self) -> np.ndarray:
        """Expose fitted class labels (decoded)."""
        return self.label_encoder.classes_


# Register with model registry
MODELS["embed-mlp"] = EmbedMLP
