"""TF-IDF + Logistic Regression commit classifier."""

from __future__ import annotations

import numpy as np
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.linear_model import LogisticRegression
from sklearn.pipeline import Pipeline

from . import MODELS, ModelProtocol


class TfidfLogReg:
    """TF-IDF + Logistic Regression baseline classifier.

    Implements ModelProtocol. Note: X for train/predict is an array
    of strings (commit messages), not numeric features — the TF-IDF
    vectorizer handles the conversion internally.

    Args:
        class_weight: 'balanced', 'balanced_subsample', None, or dict mapping class -> weight
    """

    def __init__(self, class_weight: str | dict | None = "balanced"):
        self.pipeline = Pipeline([
            ("tfidf", TfidfVectorizer(
                max_features=10000,
                ngram_range=(1, 2),
                sublinear_tf=True,
                strip_accents="unicode",
                token_pattern=r"(?u)\b\w[\w\-\.]+\b",
            )),
            ("clf", LogisticRegression(
                max_iter=1000,
                C=1.0,
                class_weight=class_weight,
                solver="lbfgs",
            )),
        ])

    def train(self, X: np.ndarray, y: np.ndarray) -> TfidfLogReg:
        """Train the TF-IDF+LogReg pipeline. X is array of message strings."""
        self.pipeline.fit(X, y)
        return self

    def predict(self, X: np.ndarray) -> np.ndarray:
        """Predict labels. X is array of message strings."""
        return self.pipeline.predict(X)

    def predict_proba(self, X: np.ndarray) -> np.ndarray:
        """Predict probabilities (useful for confidence scores)."""
        return self.pipeline.predict_proba(X)

    @property
    def classes_(self) -> np.ndarray:
        """Expose fitted class labels."""
        return self.pipeline.named_steps["clf"].classes_

    @property
    def feature_names(self) -> np.ndarray:
        """Expose TF-IDF feature names for interpretability."""
        return np.array(self.pipeline.named_steps["tfidf"].get_feature_names_out())

    @property
    def coef_(self) -> np.ndarray:
        """Expose LogReg coefficients for feature analysis."""
        return self.pipeline.named_steps["clf"].coef_


# Register with model registry
MODELS["tfidf-logreg"] = TfidfLogReg
