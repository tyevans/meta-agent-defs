"""SetFit few-shot commit classifier."""

from __future__ import annotations

import json
from pathlib import Path

import numpy as np
from sklearn.preprocessing import LabelEncoder

from . import MODELS, ModelProtocol


class SetFitClassifier:
    """SetFit contrastive fine-tuning + classification head.

    Implements ModelProtocol. SetFit is highly sample-efficient: it
    fine-tunes a sentence transformer via contrastive learning on
    sentence pairs, then trains a small classification head on the
    resulting embeddings.

    X for train/predict is an array of strings (commit messages).

    Args:
        model_name: Sentence-transformer backbone for SetFit
        samples_per_class: If set, subsample training data to K samples
            per class. Useful for data efficiency experiments (8/16/64).
            None means use all available data.
        num_iterations: Number of contrastive training iterations (default 20)
        num_epochs: Number of epochs for the classification head (default 1)
    """

    def __init__(
        self,
        model_name: str = "sentence-transformers/all-mpnet-base-v2",
        samples_per_class: int | None = None,
        num_iterations: int = 20,
        num_epochs: int = 1,
    ):
        self.model_name = model_name
        self.samples_per_class = samples_per_class
        self.num_iterations = num_iterations
        self.num_epochs = num_epochs
        self.model = None  # SetFitModel, loaded lazily
        self.label_encoder = LabelEncoder()

    def _subsample(self, X: np.ndarray, y: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
        """Subsample to K samples per class for few-shot experiments."""
        if self.samples_per_class is None:
            return X, y

        rng = np.random.RandomState(42)
        indices = []
        for label in np.unique(y):
            label_indices = np.where(y == label)[0]
            k = min(self.samples_per_class, len(label_indices))
            chosen = rng.choice(label_indices, size=k, replace=False)
            indices.extend(chosen)

        indices = sorted(indices)
        return X[indices], y[indices]

    def train(self, X: np.ndarray, y: np.ndarray) -> SetFitClassifier:
        """Train SetFit on commit messages.

        Args:
            X: Array of commit message strings
            y: Array of string labels

        Returns:
            self (trained model)
        """
        from setfit import SetFitModel, Trainer, TrainingArguments
        from datasets import Dataset

        # Encode labels to integers (SetFit Trainer expects int labels)
        self.label_encoder.fit(y)
        y_encoded = self.label_encoder.transform(y)

        # Subsample if doing few-shot experiment
        X_sub, y_sub = self._subsample(X, np.array(y_encoded))

        if self.samples_per_class is not None:
            print(f"  SetFit few-shot: {self.samples_per_class} samples/class "
                  f"-> {len(X_sub)} total training samples")

        # Build HuggingFace Dataset
        dataset = Dataset.from_dict({
            "text": X_sub.tolist(),
            "label": y_sub.tolist(),
        })

        # Split a small eval set (10%) for logging
        split = dataset.train_test_split(test_size=0.1, seed=42)

        # Load SetFit model
        self.model = SetFitModel.from_pretrained(
            self.model_name,
            labels=list(self.label_encoder.classes_),
        )

        # Training arguments
        training_args = TrainingArguments(
            num_iterations=self.num_iterations,
            num_epochs=self.num_epochs,
            batch_size=16,
            seed=42,
            report_to="none",
        )

        # Train
        trainer = Trainer(
            model=self.model,
            args=training_args,
            train_dataset=split["train"],
            eval_dataset=split["test"],
        )
        trainer.train()

        return self

    def predict(self, X: np.ndarray) -> np.ndarray:
        """Predict labels for commit messages."""
        if self.model is None:
            raise ValueError("Model not trained. Call train() first.")

        preds = self.model.predict(X.tolist())
        # SetFit returns integer labels; decode back to strings
        pred_indices = np.array(preds, dtype=int)
        return self.label_encoder.inverse_transform(pred_indices)

    def predict_proba(self, X: np.ndarray) -> np.ndarray:
        """Predict class probabilities."""
        if self.model is None:
            raise ValueError("Model not trained. Call train() first.")

        probas = self.model.predict_proba(X.tolist())
        return np.array(probas)

    @property
    def classes_(self) -> np.ndarray:
        """Expose fitted class labels."""
        return self.label_encoder.classes_

    def save(self, output_dir: Path):
        """Save SetFit model to directory.

        Args:
            output_dir: Directory to save model artifacts
        """
        if self.model is None:
            raise ValueError("Model not trained. Call train() first.")

        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

        # Save SetFit model (includes backbone + head)
        self.model.save_pretrained(output_dir)

        # Save label encoder mapping
        label_mapping = {
            i: label for i, label in enumerate(self.label_encoder.classes_)
        }
        with open(output_dir / "label_mapping.json", "w") as f:
            json.dump(label_mapping, f)

    @classmethod
    def load(cls, model_dir: Path) -> SetFitClassifier:
        """Load SetFit model from directory.

        Args:
            model_dir: Directory containing saved model artifacts

        Returns:
            Loaded SetFitClassifier instance
        """
        from setfit import SetFitModel

        model_dir = Path(model_dir)
        instance = cls()

        # Load SetFit model
        instance.model = SetFitModel.from_pretrained(str(model_dir))

        # Load label encoder
        with open(model_dir / "label_mapping.json") as f:
            label_mapping = json.load(f)

        labels = [label_mapping[str(i)] for i in range(len(label_mapping))]
        instance.label_encoder.fit(labels)

        return instance


# Register with model registry
MODELS["setfit"] = SetFitClassifier
