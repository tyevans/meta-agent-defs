# Learnings: rust-dev

## Codebase Patterns
- Project is tools/git-intel/ — a Rust CLI that analyzes git history and outputs JSON
- Uses git2-rs crate for git repository access (chosen for maturity/stability over shelling out)
- Subcommands: metrics, churn, lifecycle, patterns, hotspots, authors, trends — each outputs JSON to stdout
- Parent project is a content-only repo of Markdown definitions; this is the only compiled artifact
- lib.rs + main.rs split: library crate re-exports all modules for testability, main.rs is thin CLI wrapper (added: 2026-02-14)

## Gotchas
- clap needs `global = true` on args so flags work after subcommand name (added: 2026-02-14)
- git2's `diff.foreach` callback API requires careful lifetime management — line counting callback needs file paths from delta callback. Single-pass with shared HashMap hits borrow issues; use two-pass or Oid-based lookup (added: 2026-02-14)
- Classifier chain priority: merge (>=2 parents) > revert (`Revert "..."`, `revert:`) > release (`v`+digit, "release", "bump version") > conventional (strict match, not starts_with) > NL heuristics ("fixed"→fix, "added"→feat, "bugfix/hotfix"→fix) > "other" (consolidated: 2026-02-15)
- extract_ticket_ref(): manual parsing, priority: bracketed JIRA > unbracketed JIRA > Fixes/Closes #N > bare #N (added: 2026-02-15)
- --until flag: end-of-day semantics (23:59:59 UTC), inverted ranges error immediately. Cache key includes both since+until (added: 2026-02-15)
- dir_prefix() in common.rs (shared by hotspots + authors). depth=0 returns "." (updated: 2026-02-15)
- Pattern detection: fix-after-feat uses HashSet file intersection (filters noise); temporal clusters scan for 3+ same-type within 1h window (non-overlapping); convergence pairs removed as noise (consolidated: 2026-02-15)
- Signal system: signals.rs defines Signal struct + SignalKind enum; patterns.rs generates signals alongside existing fix_after_feat Vec (backward compatible). Signals cover both feat and refactor predecessors (added: 2026-02-15)

## Preferences
- git2 + clap + serde stack compiles in ~12s release, no async needed (added: 2026-02-14)
- `Repository::discover()` handles finding .git from subdirectories automatically — important for hooks running from nested paths (added: 2026-02-14)

## Algorithms
- Sort-then-scan for pairwise comparison: sort by key, scan adjacent entries, break when threshold exceeded. Produces identical output to nested loop but O(n log n) instead of O(n^2). Use `retain` to filter the working set before sorting. (added: 2026-02-14)
- Lifecycle "created" detection: check file existence in parent tree via get_path(), not parent_tree.is_none(). The latter only catches root commits. (added: 2026-02-14)
- push_head() in git2-rs correctly resolves symbolic refs (branches) — no special handling needed for feature branches. Cache invalidation via HEAD commit comparison prevents stale results across branch switches. (added: 2026-02-15)

## Refactoring
- walk_commits returns CommitIter (concrete struct over Box<dyn Iterator>) — avoids heap alloc, enables inlining, carries `'repo` lifetime naturally. 5/6 subcommands stream; lifecycle collects explicitly (updated: 2026-02-15)
- Return `&'static str` from classify functions instead of String when all returns are literals — zero-allocation, callers .to_string() if needed (added: 2026-02-14)
- Relative date parsing: try_parse_relative() handles Nd/Nw/Nm/Ny, resolves to absolute epoch at parse time so cache keys are correct without changes. Leap year edge case for Ny uses fallback day-1 (added: 2026-02-15)

## Testing
- git2 test fixtures: work with `Oid` values between commits, never hold `Commit<'repo>` across commit boundaries — avoids borrow conflicts (added: 2026-02-14)
- Fixture builder pattern: `stage_files` + `do_commit` closures create reproducible repos with controlled dates and content (added: 2026-02-14)
- 175 tests: 95 unit + 80 integration (updated: 2026-02-15, iterator refactor + relative date tests)
- Merge commit fixture: create branch, divergent commits on main+feature, then merge — produces commit with 2 parents for testing (added: 2026-02-15)

## Cache
- Cache in .git/git-intel-cache/, keyed by {subcommand}-{since_epoch}.json, invalidated on HEAD change. Write errors silently ignored (`let _ =`). serde_json::Value for partial validation (consolidated: 2026-02-15)

## Subcommand Composition
- Reusing existing subcommand output (e.g. churn::run(), metrics::run()) for aggregation is cleaner than duplicating diff traversal — hotspots, trends both compose this way (updated: 2026-02-15)
- Bus factor algorithm: sort authors by commits desc, accumulate until >50% of total. Count = bus factor (added: 2026-02-15)
- .mailmap must be committed (not just in working dir) for repo.mailmap() to pick it up. Empty mailmap on repos without .mailmap is fine — resolve returns original identity (added: 2026-02-15)
- Two-pass hotspots: churn for add/del stats + separate commit walk for type classification. Merging into one pass would require refactoring churn API — not worth it (added: 2026-02-15)

## ML/ONNX Integration
- ort v2 API: TensorRef::from_array_view for tensors, session.run(ort::inputs![...]) returns directly, try_extract_tensor for output. session.run needs `&mut Session` — propagates mutability through all callers; reborrow with `ml.as_mut().map(|m| &mut **m)` (consolidated: 2026-02-15)
- ML feature flag: `#[cfg(feature = "ml")]` guards module + CLI flags + function variants. ort + tokenizers as optional deps, `load-dynamic` for system ONNX Runtime (consolidated: 2026-02-15)

## Precision Study Findings
- Signal detector highly conservative: 0 signals on ripgrep/tokio/serde/rayon, 12 on clap (2y). 67% TP rate — "incomplete feature rollout" pattern (same author, small gap). FPs from unrelated code paths in same file, opportunistic cleanup (consolidated: 2026-02-15)

## Cross-Agent Notes
- (none yet)
