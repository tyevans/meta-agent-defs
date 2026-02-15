"""Ensemble inference combining multiple trained commit classifiers."""

from __future__ import annotations

import json
import pickle
from collections import Counter
from pathlib import Path
from typing import Literal

import numpy as np

from . import MODELS, ModelProtocol

EnsembleStrategy = Literal["majority_vote", "soft_vote", "weighted_soft_vote"]


class EnsembleClassifier:
    """Ensemble that combines predictions from multiple trained models.

    Implements ModelProtocol. For ensemble, "training" means loading
    pre-trained sub-models -- no actual gradient updates happen.

    Strategies:
        majority_vote:    Each model predicts, take most common label (hard voting)
        soft_vote:        Average predict_proba across models, take argmax
        weighted_soft_vote: Weighted average of predict_proba, take argmax

    X for train/predict is an array of strings (commit messages).
    Sub-models that need special input formatting (like embed-mlp's
    format_input) receive raw messages -- they must handle string input.
    """

    STRATEGIES = ("majority_vote", "soft_vote", "weighted_soft_vote")

    def __init__(
        self,
        model_paths: list[str | Path] | None = None,
        weights: list[float] | None = None,
        strategy: EnsembleStrategy = "soft_vote",
    ):
        self.model_paths: list[Path] = [Path(p) for p in model_paths] if model_paths else []
        self.weights = weights
        self.strategy: EnsembleStrategy = strategy
        self.sub_models: list[ModelProtocol] = []
        self._classes: np.ndarray | None = None

    def _load_single_model(self, path: Path) -> ModelProtocol:
        """Load a single sub-model from path (pickle file or directory).

        Detection logic for directory-based models:
        - Ensemble: config.json with "type": "ensemble"
        - Transformer: label_mapping.json + tokenizer_config.json (HF tokenizer marker)
        - SetFit: label_mapping.json without tokenizer_config.json at top level
        - Pickle: any non-directory file (tfidf-logreg, embed-mlp)
        """
        path = Path(path)

        if path.is_dir():
            # Check for ensemble config first
            if (path / "config.json").exists():
                config_data = json.loads((path / "config.json").read_text())
                if config_data.get("type") == "ensemble":
                    return EnsembleClassifier.load(path)

            # Both transformer and setfit save label_mapping.json
            if (path / "label_mapping.json").exists():
                # Transformer saves a tokenizer alongside the model;
                # setfit nests its sentence-transformer internally
                has_tokenizer = (path / "tokenizer_config.json").exists()
                if has_tokenizer:
                    from .transformer import TransformerClassifier
                    return TransformerClassifier.load(path)
                else:
                    from .setfit import SetFitClassifier
                    return SetFitClassifier.load(path)

            raise ValueError(
                f"Cannot determine model type for directory: {path}. "
                f"Expected label_mapping.json (transformer or setfit) "
                f"or config.json with type=ensemble."
            )
        else:
            # Pickle-based models: tfidf-logreg, embed-mlp
            with open(path, "rb") as f:
                return pickle.load(f)

    def train(self, X: np.ndarray, y: np.ndarray) -> EnsembleClassifier:
        """Load pre-trained sub-models. No actual training happens.

        X and y are ignored -- ensemble doesn't train, it loads.
        They're accepted to satisfy ModelProtocol.
        """
        if not self.model_paths:
            raise ValueError(
                "No model paths provided. Use --ensemble-models to specify sub-model paths."
            )

        self.sub_models = []
        for path in self.model_paths:
            print(f"  Loading sub-model: {path}")
            model = self._load_single_model(path)
            self.sub_models.append(model)
            print(f"    -> {type(model).__name__}, classes: {list(model.classes_)}")

        # Validate all models share the same label set
        base_classes = set(self.sub_models[0].classes_)
        for i, model in enumerate(self.sub_models[1:], 1):
            model_classes = set(model.classes_)
            if model_classes != base_classes:
                missing = base_classes - model_classes
                extra = model_classes - base_classes
                raise ValueError(
                    f"Sub-model {i} ({self.model_paths[i]}) has different classes. "
                    f"Missing: {missing}, Extra: {extra}"
                )

        # Use the first model's class ordering as canonical
        self._classes = self.sub_models[0].classes_

        # Validate weights
        if self.strategy == "weighted_soft_vote":
            if self.weights is None:
                # Default to equal weights
                self.weights = [1.0 / len(self.sub_models)] * len(self.sub_models)
            elif len(self.weights) != len(self.sub_models):
                raise ValueError(
                    f"Weight count ({len(self.weights)}) != model count ({len(self.sub_models)})"
                )
            else:
                # Normalize weights to sum to 1
                total = sum(self.weights)
                self.weights = [w / total for w in self.weights]

        # Validate soft strategies require predict_proba
        if self.strategy in ("soft_vote", "weighted_soft_vote"):
            for i, model in enumerate(self.sub_models):
                if not hasattr(model, "predict_proba"):
                    raise ValueError(
                        f"Sub-model {i} ({type(model).__name__}) lacks predict_proba, "
                        f"required for {self.strategy}. Use majority_vote instead."
                    )

        print(f"\n  Ensemble ready: {len(self.sub_models)} models, strategy={self.strategy}")
        return self

    def _align_probas(self, model: ModelProtocol, probas: np.ndarray) -> np.ndarray:
        """Align a model's probability columns to the canonical class order."""
        model_classes = list(model.classes_)
        canonical_classes = list(self._classes)

        if model_classes == canonical_classes:
            return probas

        # Reorder columns to match canonical ordering
        aligned = np.zeros_like(probas)
        for i, cls in enumerate(canonical_classes):
            j = model_classes.index(cls)
            aligned[:, i] = probas[:, j]
        return aligned

    def predict(self, X: np.ndarray) -> np.ndarray:
        """Predict labels by combining sub-model predictions."""
        if not self.sub_models:
            raise ValueError("No sub-models loaded. Call train() first.")

        if self.strategy == "majority_vote":
            return self._majority_vote(X)
        else:
            # soft_vote and weighted_soft_vote both use probability averaging
            probas = self.predict_proba(X)
            indices = np.argmax(probas, axis=1)
            return self._classes[indices]

    def _majority_vote(self, X: np.ndarray) -> np.ndarray:
        """Hard voting: each model predicts, take most common label."""
        all_preds = [model.predict(X) for model in self.sub_models]

        results = []
        for i in range(len(X)):
            votes = [preds[i] for preds in all_preds]
            # Most common label; ties broken by first model's vote
            counts = Counter(votes)
            winner = counts.most_common(1)[0][0]
            results.append(winner)

        return np.array(results)

    def predict_proba(self, X: np.ndarray) -> np.ndarray:
        """Average probabilities across sub-models (optionally weighted)."""
        if not self.sub_models:
            raise ValueError("No sub-models loaded. Call train() first.")

        all_probas = []
        for model in self.sub_models:
            if not hasattr(model, "predict_proba"):
                raise ValueError(
                    f"{type(model).__name__} lacks predict_proba. "
                    f"Use majority_vote strategy for models without probability support."
                )
            probas = model.predict_proba(X)
            aligned = self._align_probas(model, probas)
            all_probas.append(aligned)

        if self.strategy == "weighted_soft_vote" and self.weights is not None:
            # Weighted average
            combined = np.zeros_like(all_probas[0])
            for probas, weight in zip(all_probas, self.weights):
                combined += weight * probas
        else:
            # Uniform average
            combined = np.mean(all_probas, axis=0)

        return combined

    @property
    def classes_(self) -> np.ndarray:
        """Expose class labels (from first sub-model)."""
        if self._classes is None:
            raise ValueError("No sub-models loaded. Call train() first.")
        return self._classes

    def save(self, output_dir: Path):
        """Save ensemble config to directory.

        Saves config.json listing sub-model paths, weights, and strategy.
        Does NOT copy sub-model files -- they stay at their original paths.
        """
        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

        config = {
            "type": "ensemble",
            "strategy": self.strategy,
            "model_paths": [str(p) for p in self.model_paths],
            "weights": self.weights,
            "classes": list(self._classes) if self._classes is not None else None,
        }

        with open(output_dir / "config.json", "w") as f:
            json.dump(config, f, indent=2)

        print(f"Ensemble config saved to {output_dir / 'config.json'}")

    @classmethod
    def load(cls, model_dir: Path) -> EnsembleClassifier:
        """Load ensemble from config directory.

        Reads config.json and loads all referenced sub-models.
        """
        model_dir = Path(model_dir)
        with open(model_dir / "config.json") as f:
            config = json.load(f)

        if config.get("type") != "ensemble":
            raise ValueError(f"Not an ensemble config: {model_dir / 'config.json'}")

        instance = cls(
            model_paths=config["model_paths"],
            weights=config.get("weights"),
            strategy=config.get("strategy", "soft_vote"),
        )

        # Load sub-models (passing dummy X, y since train() ignores them for ensemble)
        dummy = np.array([])
        instance.train(dummy, dummy)

        return instance


# Register with model registry
MODELS["ensemble"] = EnsembleClassifier
