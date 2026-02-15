"""ModernBERT-based transformer commit classifier."""

from __future__ import annotations

import json
from pathlib import Path

import numpy as np
import torch
from sklearn.preprocessing import LabelEncoder
from transformers import (
    AutoModelForSequenceClassification,
    AutoTokenizer,
    Trainer,
    TrainingArguments,
    EvalPrediction,
)
from datasets import Dataset

from . import MODELS, ModelProtocol


class TransformerClassifier:
    """Fine-tuned ModernBERT commit classifier.

    Implements ModelProtocol using HuggingFace transformers.
    Fine-tunes ModernBERT-base (149M params) for multi-class commit classification.
    """

    def __init__(self, model_name: str = "answerdotai/ModernBERT-base"):
        self.model_name = model_name
        self.tokenizer = None
        self.model = None
        self.label_encoder = LabelEncoder()
        self.device = "cuda" if torch.cuda.is_available() else "cpu"

    def _ensure_tokenizer(self):
        """Lazy-load tokenizer."""
        if self.tokenizer is None:
            self.tokenizer = AutoTokenizer.from_pretrained(self.model_name)

    def train(self, X: np.ndarray, y: np.ndarray) -> TransformerClassifier:
        """Train ModernBERT on commit messages.

        Args:
            X: Array of commit message strings
            y: Array of string labels

        Returns:
            self (trained model)
        """
        # Ensure tokenizer is loaded
        self._ensure_tokenizer()

        # Encode labels
        self.label_encoder.fit(y)
        y_encoded = self.label_encoder.transform(y)
        num_labels = len(self.label_encoder.classes_)

        # Create HuggingFace dataset
        dataset_dict = {
            "text": X.tolist(),
            "label": y_encoded.tolist(),
        }
        dataset = Dataset.from_dict(dataset_dict)

        # Split into train/eval (90/10 split for early stopping)
        dataset = dataset.train_test_split(test_size=0.1, seed=42)
        train_dataset = dataset["train"]
        eval_dataset = dataset["test"]

        # Tokenize
        def tokenize_function(examples):
            return self.tokenizer(
                examples["text"],
                padding="max_length",
                truncation=True,
                max_length=128,  # Most commit messages are short
            )

        train_dataset = train_dataset.map(tokenize_function, batched=True)
        eval_dataset = eval_dataset.map(tokenize_function, batched=True)

        # Load model
        self.model = AutoModelForSequenceClassification.from_pretrained(
            self.model_name,
            num_labels=num_labels,
        )

        # Move to device
        self.model.to(self.device)

        # Training arguments
        training_args = TrainingArguments(
            output_dir="./results",
            eval_strategy="epoch",
            save_strategy="epoch",
            learning_rate=2e-5,
            per_device_train_batch_size=16,
            per_device_eval_batch_size=16,
            num_train_epochs=3,
            weight_decay=0.01,
            warmup_steps=100,
            logging_steps=50,
            load_best_model_at_end=True,
            metric_for_best_model="eval_loss",
            save_total_limit=2,
            fp16=torch.cuda.is_available(),  # Use mixed precision if CUDA available
            report_to="none",  # Disable wandb/tensorboard
        )

        # Compute metrics function for evaluation
        def compute_metrics(pred: EvalPrediction):
            preds = pred.predictions.argmax(-1)
            labels = pred.label_ids
            acc = (preds == labels).mean()
            return {"accuracy": acc}

        # Create trainer
        trainer = Trainer(
            model=self.model,
            args=training_args,
            train_dataset=train_dataset,
            eval_dataset=eval_dataset,
            compute_metrics=compute_metrics,
        )

        # Train
        trainer.train()

        return self

    def predict(self, X: np.ndarray) -> np.ndarray:
        """Predict labels for commit messages.

        Args:
            X: Array of commit message strings

        Returns:
            Array of predicted string labels
        """
        if self.model is None:
            raise ValueError("Model not trained. Call train() first.")

        self._ensure_tokenizer()

        # Tokenize inputs
        inputs = self.tokenizer(
            X.tolist(),
            padding="max_length",
            truncation=True,
            max_length=128,
            return_tensors="pt",
        )

        # Move to device
        inputs = {k: v.to(self.device) for k, v in inputs.items()}

        # Predict
        self.model.eval()
        with torch.no_grad():
            outputs = self.model(**inputs)
            logits = outputs.logits
            preds = logits.argmax(dim=-1).cpu().numpy()

        # Decode labels
        return self.label_encoder.inverse_transform(preds)

    def predict_proba(self, X: np.ndarray) -> np.ndarray:
        """Predict class probabilities.

        Args:
            X: Array of commit message strings

        Returns:
            Array of shape (n_samples, n_classes) with probabilities
        """
        if self.model is None:
            raise ValueError("Model not trained. Call train() first.")

        self._ensure_tokenizer()

        # Tokenize inputs
        inputs = self.tokenizer(
            X.tolist(),
            padding="max_length",
            truncation=True,
            max_length=128,
            return_tensors="pt",
        )

        # Move to device
        inputs = {k: v.to(self.device) for k, v in inputs.items()}

        # Predict
        self.model.eval()
        with torch.no_grad():
            outputs = self.model(**inputs)
            logits = outputs.logits
            probs = torch.softmax(logits, dim=-1).cpu().numpy()

        return probs

    @property
    def classes_(self) -> np.ndarray:
        """Expose fitted class labels."""
        return self.label_encoder.classes_

    def save(self, output_dir: Path):
        """Save model and tokenizer to directory.

        Args:
            output_dir: Directory to save model artifacts
        """
        if self.model is None:
            raise ValueError("Model not trained. Call train() first.")

        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

        # Save model and tokenizer
        self.model.save_pretrained(output_dir)
        self.tokenizer.save_pretrained(output_dir)

        # Save label encoder
        label_mapping = {
            i: label for i, label in enumerate(self.label_encoder.classes_)
        }
        with open(output_dir / "label_mapping.json", "w") as f:
            json.dump(label_mapping, f)

    @classmethod
    def load(cls, model_dir: Path) -> TransformerClassifier:
        """Load model from directory.

        Args:
            model_dir: Directory containing saved model artifacts

        Returns:
            Loaded TransformerClassifier instance
        """
        model_dir = Path(model_dir)

        # Load tokenizer and model
        instance = cls()
        instance.tokenizer = AutoTokenizer.from_pretrained(model_dir)
        instance.model = AutoModelForSequenceClassification.from_pretrained(model_dir)
        instance.model.to(instance.device)

        # Load label encoder
        with open(model_dir / "label_mapping.json") as f:
            label_mapping = json.load(f)

        # Reconstruct label encoder
        labels = [label_mapping[str(i)] for i in range(len(label_mapping))]
        instance.label_encoder.fit(labels)

        return instance


# Register with model registry
MODELS["transformer"] = TransformerClassifier
