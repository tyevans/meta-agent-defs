# Learnings: rust-dev

## Codebase Patterns
- Project is tools/git-intel/ — a Rust CLI that analyzes git history and outputs JSON
- Uses git2-rs crate for git repository access (chosen for maturity/stability over shelling out)
- Subcommands: metrics, churn, lifecycle, patterns — each outputs JSON to stdout
- Parent project is a content-only repo of Markdown definitions; this is the only compiled artifact
- lib.rs + main.rs split: library crate re-exports all modules for testability, main.rs is thin CLI wrapper (added: 2026-02-14)

## Gotchas
- clap needs `global = true` on args so flags work after subcommand name (e.g., `git-intel metrics --repo .`) (added: 2026-02-14)
- git2's `diff.foreach` callback API requires careful lifetime management with multiple closures — line counting callback needs file paths from delta callback (added: 2026-02-14)
- classify_commit uses strict matching (type: type( type! type+space) not starts_with — prevents "fixing"→"fix" false positives (added: 2026-02-14)
- classify_commit_with_parents(message, parent_count): merge detection (>=2 parents) takes priority over message-based classification (added: 2026-02-15)
- Revert detection: guard clause before conventional commit loop, matches `Revert "..."`, `revert:`, `revert(` — priority: merge > revert > conventional (added: 2026-02-15)
- NL heuristics fallback: "fixed"→fix, "added"→feat, "Fixes/Closes #N"→fix, "bugfix/hotfix"→fix. Last check before "other" (added: 2026-02-15)
- Convergence pairs: DEFAULT_CONVERGENCE_LIMIT=50, MIN_CONVERGENCE_BYTES=500, --convergence-limit flag, sorted by bytes_ratio before truncation (added: 2026-02-15)
- churn.rs: single-pass diff.foreach with shared HashMap hits borrow issues — use two-pass (file-level then line-level) or Oid-based lookup (added: 2026-02-14)

## Preferences
- git2 + clap + serde stack compiles in ~12s release, no async needed (added: 2026-02-14)
- `Repository::discover()` handles finding .git from subdirectories automatically — important for hooks running from nested paths (added: 2026-02-14)

## Algorithms
- Sort-then-scan for pairwise comparison: sort by key, scan adjacent entries, break when threshold exceeded. Produces identical output to nested loop but O(n log n) instead of O(n^2). Use `retain` to filter the working set before sorting. (added: 2026-02-14)
- Lifecycle "created" detection: check file existence in parent tree via get_path(), not parent_tree.is_none(). The latter only catches root commits. (added: 2026-02-14)

## Refactoring
- walk_commits needs explicit lifetime `<'repo>` on return type since Commit borrows from Repository (added: 2026-02-14)
- Return `&'static str` from classify functions instead of String when all returns are literals — zero-allocation, callers .to_string() if needed (added: 2026-02-14)

## Testing
- git2 test fixtures: work with `Oid` values between commits, never hold `Commit<'repo>` across commit boundaries — avoids borrow conflicts (added: 2026-02-14)
- Fixture builder pattern: `stage_files` + `do_commit` closures create reproducible repos with controlled dates and content (added: 2026-02-14)
- 75 tests: 39 unit + 36 integration (added: 2026-02-15)
- Merge commit fixture: create branch, divergent commits on main+feature, then merge — produces commit with 2 parents for testing (added: 2026-02-15)

## Cache
- Cache write errors should always be silently ignored (`let _ =`) to avoid breaking the main path (added: 2026-02-15)
- serde_json::Value for partial cache validation (check head_commit without deserializing result) avoids needing Deserialize on output structs (added: 2026-02-15)
- Cache lives in .git/git-intel-cache/, keyed by {subcommand}-{since_epoch}.json, invalidated on HEAD change (added: 2026-02-15)

## Subcommand Composition
- Reusing existing subcommand output (e.g. churn::run()) for aggregation is cleaner than duplicating diff traversal — hotspots is ~60 lines of logic on top of churn (added: 2026-02-15)

## Cross-Agent Notes
- (none yet)
