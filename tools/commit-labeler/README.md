# Commit Labeler

Python ML pipeline for commit classification using embeddings and transformers.

## Scripts

### benchmark.py

Compares multiple sentence-transformer embedding models with identical MLP heads on a frozen test set. This helps select the best embedding model before fine-tuning or expanding the architecture.

**Models tested:**
- `all-MiniLM-L6-v2` (384d, 22M params) — current baseline
- `all-MiniLM-L12-v2` (384d, 33M params) — larger MiniLM
- `bge-small-en-v1.5` (384d, 33M params) — BGE small
- `gte-small` (384d, 33M params) — GTE small
- `e5-small-v2` (384d, 33M params) — E5 small
- `nomic-embed-text-v1.5` (768d, 137M params) — Nomic large

**Usage:**
```bash
uv run python benchmark.py ../data/labeled-all.jsonl
uv run python benchmark.py ../data/labeled-all.jsonl --output results/benchmark.json
uv run python benchmark.py ../data/labeled-all.jsonl --quick  # Skip models >100M params
```

**Output:** Comparison table showing F1 macro, F1 weighted, accuracy, and timing for each model. Results saved to JSON for downstream analysis.

### train.py

Train commit classifiers using the model registry.

**Usage:**
```bash
uv run python train.py ../data/labeled-all.jsonl --model tfidf-logreg
uv run python train.py ../data/labeled-all.jsonl --model embed-mlp
uv run python train.py ../data/labeled-all.jsonl --model transformer
```

## Architecture

- **ModelProtocol registry**: Models self-register at import via `models/__init__.py`
- **7-phase pipeline**: clone → extract → stride → enrich → label → train → summary
- **Multi-label support**: `labels` list replaces single `label` string
- **Shared utilities**: `data.py` (I/O), `eval.py` (metrics), `schemas.py` (Pydantic models)

## Dependencies

Managed with `uv` (pyproject.toml + uv.lock). Key dependencies:
- sentence-transformers
- transformers
- torch
- scikit-learn
- datasets
- accelerate
