# Learnings: ml-eng

## Codebase Patterns
- Project: tools/commit-labeler/ — Python (uv), 7-phase pipeline: clone → extract → stride → enrich → label → train → summary
- Models in models/ package with ModelProtocol registry: tfidf_logreg, embed_mlp, transformer, setfit, ensemble — auto-register via MODELS dict in __init__.py
- Shared utilities: data.py (canonical I/O — loading, parse, resume, distribution), eval.py (run_eval + evaluate + print_report with JSON export), schemas.py (Pydantic models)
- labels.json is vocabulary contract (13 labels, v1); data.py:load_labels() is Python entry point
- label.py handles Ollama labeling (raw + --enriched modes), outputs multi-label format ("labels" list)
- train.py unified CLI: --model flag dispatches to registry, --eval-output JSON, --registry auto-registers results

## Gotchas
- enrich_diffs.py auto-detects pipe-delimited vs JSONL input; pipeline.sh Phase 5 auto-detects enriched vs raw
- All I/O in data.py only — label.py and enrich_diffs.py import from it (no duplication)
- schemas.py for Pydantic models, models/ for ML implementations — avoids namespace collision

## Model-Specific
- ModernBERT-base (149M params): max_length=128, needs datasets.Dataset (not numpy), custom save/load (directory-based vs pickle)
- SetFit: contrastive fine-tuning + small head, lazy import, --samples-per-class for few-shot, directory-based save/load
- EnsembleClassifier: 3 strategies (majority/soft/weighted_soft_vote), loads pre-trained sub-models, _align_probas() handles different class orderings
- Smart model type detection: transformer (tokenizer_config.json), setfit (label_mapping.json), ensemble (config.json), else pickle

## Class Imbalance
- FocalLoss in losses.py: FL(p_t) = -(1-p_t)^gamma * log(p_t), default gamma=2.0. Train flags: --class-weight, --focal-loss, --focal-gamma
- embed_mlp rewritten from sklearn to PyTorch MLP for custom loss support (early stopping, patience=10)
- transformer uses WeightedTrainer subclass overriding compute_loss

## ONNX Export
- torch.onnx.export needs `onnxscript` for newer PyTorch, opset_version=14+ for ModernBERT
- Output: f32 logits (not probabilities) — Rust applies softmax. Expects i64 input. Large models produce model.onnx + model.onnx.data
- train.py --export-onnx supports standalone (load + export) and train+export modes

## Experiment Tracking
- registry.py: file-based JSON in results/ dir. register_result(), list_results(), best_result(), compare_results()
- Benchmark scripts standalone (benchmark.py separate from models/). Code-aware models: CodeBERT (CLS pooling), UniXcoder (mean pooling), CodeT5+ (encoder method)

## Remote Training
- NEVER train locally — always use GPU server debian@192.168.1.14 (RTX 3090) (added: 2026-02-15)
- Sync code+data up: `rsync -avz --exclude='.venv' --exclude='__pycache__' --exclude='results' tools/commit-labeler/ debian@192.168.1.14:~/commit-labeler/ && rsync -avz tools/data/ debian@192.168.1.14:~/data/` (added: 2026-02-15)
- Run training via ssh: `ssh debian@192.168.1.14 'cd ~/commit-labeler && ...'` (added: 2026-02-15)
- Sync results back: `rsync -avz --exclude='checkpoint-*' debian@192.168.1.14:~/commit-labeler/results/ tools/commit-labeler/results/` — exclude checkpoints to avoid multi-GB transfers (added: 2026-02-15)

## Training Comparison Results
- Best macro F1: tfidf-logreg + balanced — 0.641. Best fix F1: transformer + focal — 0.831 (P=0.819, R=0.843) (added: 2026-02-15)
- Transformer + focal is production model (exported to ONNX). Best fix detection, best refactor (0.549), high accuracy (0.765) (added: 2026-02-15)
- Class weighting hurts transformers (0.636→0.602 macro F1, fix recall drops 0.847→0.722). Opposite of tfidf where weighting helps +6.9% (added: 2026-02-15)
- Focal loss helps transformers (+1.0% macro F1) but hurts simpler models. Focuses learning on hard examples where transformer capacity matters (added: 2026-02-15)
- "other" is a catchall — 0→detectable is noise, not a real improvement (added: 2026-02-15)
- Non-interactive SSH needs `export PATH="$HOME/.local/bin:$PATH"` prefix for uv (added: 2026-02-15)
- Transformer batch_size=64 is optimal for RTX 3090 (128 OOMs, 16 underutilizes at 3.5/24GB) (added: 2026-02-15)

## Cross-Agent Notes
- (none yet)
