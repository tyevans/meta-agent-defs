"""Commit classifier model protocol and registry."""

from __future__ import annotations

from typing import Protocol, runtime_checkable

import numpy as np


@runtime_checkable
class ModelProtocol(Protocol):
    """Minimal protocol for commit classifiers.

    Any model that implements train() and predict() can be used
    with the unified training pipeline. This is intentionally minimal —
    just the shape, not a framework.
    """

    def train(self, X: np.ndarray, y: np.ndarray) -> ModelProtocol:
        """Train on feature matrix X and label array y. Returns self."""
        ...

    def predict(self, X: np.ndarray) -> np.ndarray:
        """Predict labels for feature matrix X."""
        ...


# Model registry — populated by model modules
MODELS: dict[str, type[ModelProtocol]] = {}


# Import model modules to trigger registration
# These imports happen after MODELS is defined so models can register themselves
def _register_models():
    """Import all model modules to populate MODELS registry."""
    from . import tfidf_logreg, embed_mlp, transformer, setfit


_register_models()


__all__ = ["ModelProtocol", "MODELS"]
