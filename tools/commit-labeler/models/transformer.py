"""ModernBERT-based transformer commit classifier."""

from __future__ import annotations

import json
import shutil
from pathlib import Path

import numpy as np
import onnx
import onnxruntime as ort
import torch
from sklearn.preprocessing import LabelEncoder
from sklearn.utils.class_weight import compute_class_weight
from transformers import (
    AutoModelForSequenceClassification,
    AutoTokenizer,
    Trainer,
    TrainingArguments,
    EvalPrediction,
)
from datasets import Dataset

from . import MODELS, ModelProtocol
from losses import FocalLoss


class TransformerClassifier:
    """Fine-tuned ModernBERT commit classifier.

    Implements ModelProtocol using HuggingFace transformers.
    Fine-tunes ModernBERT-base (149M params) for multi-class commit classification.

    Args:
        model_name: HuggingFace model name
        class_weight_mode: 'none', 'balanced', or 'auto' (inverse frequency)
        use_focal_loss: Whether to use focal loss instead of cross-entropy
        focal_gamma: Focal loss gamma parameter (default 2.0)
    """

    def __init__(
        self,
        model_name: str = "answerdotai/ModernBERT-base",
        class_weight_mode: str = "none",
        use_focal_loss: bool = False,
        focal_gamma: float = 2.0,
    ):
        self.model_name = model_name
        self.tokenizer = None
        self.model = None
        self.label_encoder = LabelEncoder()
        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        self.class_weight_mode = class_weight_mode
        self.use_focal_loss = use_focal_loss
        self.focal_gamma = focal_gamma
        self.class_weights: torch.Tensor | None = None

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

        # Compute class weights if requested
        if self.class_weight_mode == "balanced":
            weights = compute_class_weight(
                "balanced",
                classes=np.arange(num_labels),
                y=y_encoded,
            )
            self.class_weights = torch.FloatTensor(weights).to(self.device)
        elif self.class_weight_mode == "auto":
            # Manual inverse frequency
            class_counts = np.bincount(y_encoded)
            weights = len(y_encoded) / (num_labels * class_counts)
            self.class_weights = torch.FloatTensor(weights).to(self.device)

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
            per_device_train_batch_size=128,
            per_device_eval_batch_size=128,
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

        # Custom trainer with loss override
        class WeightedTrainer(Trainer):
            def __init__(self, *args, class_weights=None, use_focal=False, focal_gamma=2.0, **kwargs):
                super().__init__(*args, **kwargs)
                self.class_weights = class_weights
                self.use_focal = use_focal
                self.focal_gamma = focal_gamma
                if use_focal:
                    self.focal_loss = FocalLoss(alpha=class_weights, gamma=focal_gamma)

            def compute_loss(self, model, inputs, return_outputs=False, **kwargs):
                labels = inputs.pop("labels")
                outputs = model(**inputs)
                logits = outputs.logits

                if self.use_focal:
                    loss = self.focal_loss(logits, labels)
                elif self.class_weights is not None:
                    loss_fct = torch.nn.CrossEntropyLoss(weight=self.class_weights)
                    loss = loss_fct(logits, labels)
                else:
                    loss_fct = torch.nn.CrossEntropyLoss()
                    loss = loss_fct(logits, labels)

                return (loss, outputs) if return_outputs else loss

        # Create trainer
        trainer = WeightedTrainer(
            model=self.model,
            args=training_args,
            train_dataset=train_dataset,
            eval_dataset=eval_dataset,
            compute_metrics=compute_metrics,
            class_weights=self.class_weights,
            use_focal=self.use_focal_loss,
            focal_gamma=self.focal_gamma,
        )

        # Train
        trainer.train()

        return self

    def _predict_batched(self, X: np.ndarray, batch_size: int = 64) -> np.ndarray:
        """Run batched inference, returning raw logits."""
        if self.model is None:
            raise ValueError("Model not trained. Call train() first.")

        self._ensure_tokenizer()
        self.model.eval()

        all_logits = []
        texts = X.tolist()
        for i in range(0, len(texts), batch_size):
            batch = texts[i : i + batch_size]
            inputs = self.tokenizer(
                batch,
                padding="max_length",
                truncation=True,
                max_length=128,
                return_tensors="pt",
            )
            inputs = {k: v.to(self.device) for k, v in inputs.items()}
            with torch.no_grad():
                outputs = self.model(**inputs)
                all_logits.append(outputs.logits.cpu())
        return torch.cat(all_logits, dim=0).numpy()

    def predict(self, X: np.ndarray) -> np.ndarray:
        """Predict labels for commit messages (batched to avoid OOM)."""
        logits = self._predict_batched(X)
        preds = logits.argmax(axis=-1)
        return self.label_encoder.inverse_transform(preds)

    def predict_proba(self, X: np.ndarray) -> np.ndarray:
        """Predict class probabilities (batched to avoid OOM)."""
        logits = self._predict_batched(X)
        # softmax on numpy
        exp = np.exp(logits - logits.max(axis=-1, keepdims=True))
        return exp / exp.sum(axis=-1, keepdims=True)

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

    def export_onnx(self, output_dir: Path):
        """Export model to ONNX format for Rust inference.

        Creates a directory with:
        - model.onnx: ONNX model file
        - label_mapping.json: Label index to class name mapping
        - tokenizer.json: Tokenizer config for Rust

        Args:
            output_dir: Directory to save ONNX artifacts
        """
        if self.model is None:
            raise ValueError("Model not trained. Call train() first or load() an existing model.")

        self._ensure_tokenizer()
        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

        # Set model to eval mode
        self.model.eval()

        # Create dummy input (batch_size=1, seq_len=128)
        dummy_input_ids = torch.randint(0, self.tokenizer.vocab_size, (1, 128))
        dummy_attention_mask = torch.ones((1, 128), dtype=torch.long)

        # Move to same device as model
        dummy_input_ids = dummy_input_ids.to(self.device)
        dummy_attention_mask = dummy_attention_mask.to(self.device)

        # Export to ONNX
        onnx_path = output_dir / "model.onnx"
        print(f"Exporting ONNX model to {onnx_path}...")

        torch.onnx.export(
            self.model,
            (dummy_input_ids, dummy_attention_mask),
            onnx_path,
            input_names=["input_ids", "attention_mask"],
            output_names=["logits"],
            dynamic_axes={
                "input_ids": {0: "batch_size"},
                "attention_mask": {0: "batch_size"},
                "logits": {0: "batch_size"},
            },
            opset_version=14,
            do_constant_folding=True,
        )

        print(f"ONNX model exported to {onnx_path}")

        # Copy label mapping
        label_mapping = {
            i: label for i, label in enumerate(self.label_encoder.classes_)
        }
        label_path = output_dir / "label_mapping.json"
        with open(label_path, "w") as f:
            json.dump(label_mapping, f, indent=2)
        print(f"Label mapping saved to {label_path}")

        # Copy tokenizer.json if it exists in the saved model
        # The tokenizer.json is created by HuggingFace's save_pretrained
        # We need to save the tokenizer first if not already saved
        temp_tokenizer_dir = output_dir / "_temp_tokenizer"
        self.tokenizer.save_pretrained(temp_tokenizer_dir)
        tokenizer_src = temp_tokenizer_dir / "tokenizer.json"
        tokenizer_dst = output_dir / "tokenizer.json"

        if tokenizer_src.exists():
            shutil.copy(tokenizer_src, tokenizer_dst)
            print(f"Tokenizer saved to {tokenizer_dst}")
        else:
            print(f"Warning: tokenizer.json not found at {tokenizer_src}")

        # Clean up temp directory
        shutil.rmtree(temp_tokenizer_dir)

        # Validate ONNX model
        print("\nValidating ONNX model...")
        self._validate_onnx_export(onnx_path)

    def _validate_onnx_export(self, onnx_path: Path):
        """Validate ONNX model by comparing outputs with PyTorch.

        Args:
            onnx_path: Path to exported ONNX model
        """
        # Load ONNX model
        onnx_model = onnx.load(str(onnx_path))
        onnx.checker.check_model(onnx_model)
        print("  ✓ ONNX model structure is valid")

        # Create ONNX Runtime session
        ort_session = ort.InferenceSession(str(onnx_path))

        # Test with a sample input
        test_text = "Fix null pointer in auth module"
        inputs = self.tokenizer(
            test_text,
            padding="max_length",
            truncation=True,
            max_length=128,
            return_tensors="pt",
        )

        # PyTorch inference
        self.model.eval()
        with torch.no_grad():
            inputs_device = {k: v.to(self.device) for k, v in inputs.items()}
            pt_outputs = self.model(**inputs_device)
            pt_logits = pt_outputs.logits.cpu().numpy()

        # ONNX inference
        ort_inputs = {
            "input_ids": inputs["input_ids"].numpy(),
            "attention_mask": inputs["attention_mask"].numpy(),
        }
        onnx_logits = ort_session.run(None, ort_inputs)[0]

        # Compare outputs
        max_diff = np.abs(pt_logits - onnx_logits).max()
        print(f"  ✓ Max difference between PyTorch and ONNX: {max_diff:.6f}")

        if max_diff < 1e-4:
            print("  ✓ ONNX export validation PASSED (outputs match PyTorch)")
        else:
            print(f"  ⚠ Warning: Output difference {max_diff:.6f} is larger than expected")

        # Check predicted class matches
        pt_pred = pt_logits.argmax()
        onnx_pred = onnx_logits.argmax()
        if pt_pred == onnx_pred:
            pt_label = self.label_encoder.inverse_transform([pt_pred])[0]
            print(f"  ✓ Predicted class matches: {pt_label}")
        else:
            print(f"  ⚠ Warning: Predicted classes differ (PyTorch: {pt_pred}, ONNX: {onnx_pred})")


# Register with model registry
MODELS["transformer"] = TransformerClassifier
