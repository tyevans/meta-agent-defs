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
- Shared utilities: data.py (data loading, parse, resume), eval.py (evaluate + print_report) (added: 2026-02-14)
- label.py now outputs multi-label format: "labels" list instead of "label" string (added: 2026-02-14)
- print_distribution() handles both old single-label and new multi-label JSONL formats (added: 2026-02-14)

## Gotchas
- relabel.py deleted — all diff-aware labeling is now in label.py --enriched (added: 2026-02-14)
- data.py has load_done_hashes/parse_commit_line/print_distribution but label.py still has its own copies — wire-up pending (added: 2026-02-14)
- enrich_diffs.py auto-detects pipe-delimited vs JSONL input format (added: 2026-02-14)
- pipeline.sh Phase 5 auto-detects enriched vs raw input based on whether Phase 4 ran (added: 2026-02-14)

## Preferences
- (none yet)

## Cross-Agent Notes
- (none yet)
