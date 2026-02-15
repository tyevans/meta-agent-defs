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
- Shared utilities: data.py (canonical I/O module — all data loading, parse, resume, distribution), eval.py (evaluate + print_report) (added: 2026-02-15)
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

## Cross-Agent Notes
- (none yet)
