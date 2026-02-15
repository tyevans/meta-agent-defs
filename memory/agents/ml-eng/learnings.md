# Learnings: ml-eng

## Codebase Patterns
- Project lives at tools/commit-labeler/ (relocated from research/commit-labeler/) (added: 2026-02-14)
- Python project managed with uv (pyproject.toml + uv.lock), Python 3.x (added: 2026-02-14)
- Models live in models/ package with ModelProtocol registry: tfidf_logreg.py and embed_mlp.py, auto-register via MODELS dict (added: 2026-02-14)
- Labeling via Ollama API: label.py handles both raw and enriched modes (--enriched flag) (added: 2026-02-14)
- Data pipeline: pipeline.sh orchestrates 7 phases: clone → extract → stride → enrich → label → train → summary (added: 2026-02-14)
- Unified train.py CLI with --model flag dispatches to tfidf-logreg or embed-mlp via MODELS registry (added: 2026-02-14)
- train_embed.py deleted — code moved to models/embed_mlp.py + unified train.py (added: 2026-02-14)
- labels.json is the canonical vocabulary contract — 13 labels, version 1. data.py:load_labels() is the Python entry point (added: 2026-02-14)
- enrich_diffs.py adds git diff context to commit messages for richer classification (added: 2026-02-14)
- Shared utilities: data.py (canonical I/O module — all data loading, parse, resume, distribution), eval.py (run_eval + evaluate + print_report) (added: 2026-02-15)
- eval.py:run_eval() is the canonical eval harness — all models should use it. Wraps evaluate()+print_report(), adds probability metrics and JSON export (added: 2026-02-15)
- JSON eval export format: {"model": str, "metrics": {...}, "report": {...}, "proba_metrics": {...}, "top_confusions": [...]} (added: 2026-02-15)
- train.py --eval-output flag saves eval results as JSON for cross-model comparison (added: 2026-02-15)
- Phase 1 cleanup removed 4 files: relabel.py, train_embed.py, train_hybrid.py, analyze_errors.py. Only data.py, label.py, train.py, enrich_diffs.py, eval.py remain (added: 2026-02-15)
- label.py now outputs multi-label format: "labels" list instead of "label" string (added: 2026-02-14)
- print_distribution() handles both old single-label and new multi-label JSONL formats (added: 2026-02-14)

## Gotchas
- relabel.py deleted — all diff-aware labeling is now in label.py --enriched (added: 2026-02-14)
- All I/O duplicates eliminated: parse_commit_line, load_done_hashes, print_distribution now only in data.py — label.py and enrich_diffs.py import from it (added: 2026-02-15)
- enrich_diffs.py auto-detects pipe-delimited vs JSONL input format (added: 2026-02-14)
- pipeline.sh Phase 5 auto-detects enriched vs raw input based on whether Phase 4 ran (added: 2026-02-14)

## Preferences
- schemas.py for Pydantic models (CommitLabel, LabelEntry, CommitClassification, BatchClassification); models/ package for ML implementations (ModelProtocol registry) — avoids namespace collision (added: 2026-02-15)

## Transformer Integration
- ModernBERT-base is at answerdotai/ModernBERT-base on HuggingFace hub (149M params) (added: 2026-02-15)
- Transformer models need custom save/load (directory-based) vs sklearn models (pickle) — train.py has special save logic for this (added: 2026-02-15)
- HuggingFace Trainer requires datasets.Dataset, not raw numpy arrays (added: 2026-02-15)
- Commit messages are short — max_length=128 tokens is sufficient (added: 2026-02-15)
- models/__init__.py uses _register_models() to auto-import all model modules (added: 2026-02-15)

## Benchmarking
- Benchmark scripts should be standalone (not model implementations) — benchmark.py is separate from models/ (added: 2026-02-15)
- Reusing existing architecture patterns (MLP from embed_mlp) ensures fair comparison across embedding models (added: 2026-02-15)

## Class Imbalance
- FocalLoss in losses.py: standalone PyTorch module, formula FL(p_t) = -(1-p_t)^gamma * log(p_t), supports alpha (class weights). Default gamma=2.0 (added: 2026-02-15)
- train.py CLI flags: --class-weight (none|balanced|auto), --focal-loss, --focal-gamma (added: 2026-02-15)
- embed_mlp.py rewritten from sklearn MLPClassifier to custom PyTorch MLP — needed for custom loss functions. Has early stopping (patience=10), device management (added: 2026-02-15)
- transformer.py uses WeightedTrainer subclass of HuggingFace Trainer — overrides compute_loss for class weights and focal loss (added: 2026-02-15)
- Class weight computation: sklearn compute_class_weight('balanced') or manual inverse frequency n_samples/(n_classes*class_count) (added: 2026-02-15)

## ONNX Export
- torch.onnx.export needs `onnxscript` for newer PyTorch — not just `onnx` + `onnxruntime` (added: 2026-02-15)
- ONNX export: model must be in eval mode, dummy inputs must match device. Use opset_version=14+ for ModernBERT (added: 2026-02-15)
- Exported model outputs f32 logits (not probabilities) — Rust side must apply softmax. Expects i64 input tensors. Large models produce model.onnx + model.onnx.data (external weights) (added: 2026-02-15)
- train.py --export-onnx supports standalone mode (load existing model + export) and train+export mode (added: 2026-02-15)

## Cross-Agent Notes
- (none yet)
