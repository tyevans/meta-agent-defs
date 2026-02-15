"""Sentence-transformer + MLP commit classifier."""

from __future__ import annotations

import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
from sentence_transformers import SentenceTransformer
from sklearn.preprocessing import LabelEncoder
from sklearn.utils.class_weight import compute_class_weight
from torch.utils.data import DataLoader, TensorDataset

from . import MODELS, ModelProtocol
from losses import FocalLoss


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


class PyTorchMLP(nn.Module):
    """Simple PyTorch MLP for classification."""

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


class EmbedMLP:
    """Sentence-transformer embedding + MLP classifier.

    Implements ModelProtocol. This model works in two stages:
    1. Encode text using a sentence-transformer model
    2. Classify embeddings with an MLP (PyTorch-based for class weighting support)

    X for train/predict can be either:
    - Pre-computed embeddings (np.ndarray of floats)
    - Text strings (will be encoded using the sentence-transformer)

    Args:
        model_name: Sentence transformer model name
        class_weight_mode: 'none', 'balanced', or 'auto' (inverse frequency)
        use_focal_loss: Whether to use focal loss instead of cross-entropy
        focal_gamma: Focal loss gamma parameter (default 2.0)
    """

    def __init__(
        self,
        model_name: str = "all-MiniLM-L6-v2",
        class_weight_mode: str = "none",
        use_focal_loss: bool = False,
        focal_gamma: float = 2.0,
    ):
        self.model_name = model_name
        self.encoder: SentenceTransformer | None = None
        self.mlp: PyTorchMLP | None = None
        self.label_encoder = LabelEncoder()
        self.class_weight_mode = class_weight_mode
        self.use_focal_loss = use_focal_loss
        self.focal_gamma = focal_gamma
        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        self.class_weights: torch.Tensor | None = None

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

        # Encode labels
        self.label_encoder.fit(y)
        y_encoded = self.label_encoder.transform(y)
        num_classes = len(self.label_encoder.classes_)

        # Compute class weights if requested
        if self.class_weight_mode == "balanced":
            weights = compute_class_weight(
                "balanced",
                classes=np.arange(num_classes),
                y=y_encoded,
            )
            self.class_weights = torch.FloatTensor(weights).to(self.device)
        elif self.class_weight_mode == "auto":
            # Manual inverse frequency
            class_counts = np.bincount(y_encoded)
            weights = len(y_encoded) / (num_classes * class_counts)
            self.class_weights = torch.FloatTensor(weights).to(self.device)

        # Initialize MLP
        input_dim = X.shape[1]
        self.mlp = PyTorchMLP(
            input_dim=input_dim,
            hidden_dims=(128, 64),
            num_classes=num_classes,
        ).to(self.device)

        # Prepare data
        X_tensor = torch.FloatTensor(X).to(self.device)
        y_tensor = torch.LongTensor(y_encoded).to(self.device)
        dataset = TensorDataset(X_tensor, y_tensor)
        dataloader = DataLoader(dataset, batch_size=128, shuffle=True)

        # Setup loss function
        if self.use_focal_loss:
            criterion = FocalLoss(
                alpha=self.class_weights,
                gamma=self.focal_gamma,
            )
        else:
            if self.class_weights is not None:
                criterion = nn.CrossEntropyLoss(weight=self.class_weights)
            else:
                criterion = nn.CrossEntropyLoss()

        # Setup optimizer
        optimizer = optim.Adam(self.mlp.parameters(), lr=0.001)

        # Training loop
        self.mlp.train()
        num_epochs = 100
        best_loss = float('inf')
        patience = 10
        patience_counter = 0

        for epoch in range(num_epochs):
            epoch_loss = 0.0
            for batch_X, batch_y in dataloader:
                optimizer.zero_grad()
                outputs = self.mlp(batch_X)
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

        return self

    def predict(self, X: np.ndarray) -> np.ndarray:
        """Predict labels. Returns string labels."""
        if X.dtype.kind in ('U', 'O'):
            X = self.encode(X.tolist())

        self.mlp.eval()
        X_tensor = torch.FloatTensor(X).to(self.device)
        with torch.no_grad():
            outputs = self.mlp(X_tensor)
            y_encoded = outputs.argmax(dim=1).cpu().numpy()

        return self.label_encoder.inverse_transform(y_encoded)

    def predict_proba(self, X: np.ndarray) -> np.ndarray:
        """Predict probabilities for all classes."""
        if X.dtype.kind in ('U', 'O'):
            X = self.encode(X.tolist())

        self.mlp.eval()
        X_tensor = torch.FloatTensor(X).to(self.device)
        with torch.no_grad():
            outputs = self.mlp(X_tensor)
            probas = torch.softmax(outputs, dim=1).cpu().numpy()

        return probas

    @property
    def classes_(self) -> np.ndarray:
        """Expose fitted class labels (decoded)."""
        return self.label_encoder.classes_


# Register with model registry
MODELS["embed-mlp"] = EmbedMLP
