# Learnings: rust-dev

## Codebase Patterns
- Project is tools/git-intel/ — a Rust CLI that analyzes git history and outputs JSON
- Uses git2-rs crate for git repository access (chosen for maturity/stability over shelling out)
- Subcommands: metrics, churn, lifecycle, patterns — each outputs JSON to stdout
- Parent project is a content-only repo of Markdown definitions; this is the only compiled artifact

## Gotchas
- clap needs `global = true` on args so flags work after subcommand name (e.g., `git-intel metrics --repo .`) (added: 2026-02-14)
- git2's `diff.foreach` callback API requires careful lifetime management with multiple closures — line counting callback needs file paths from delta callback (added: 2026-02-14)

## Preferences
- git2 + clap + serde stack compiles in ~12s release, no async needed (added: 2026-02-14)
- `Repository::discover()` handles finding .git from subdirectories automatically — important for hooks running from nested paths (added: 2026-02-14)

## Algorithms
- Sort-then-scan for pairwise comparison: sort by key, scan adjacent entries, break when threshold exceeded. Produces identical output to nested loop but O(n log n) instead of O(n^2). Use `retain` to filter the working set before sorting. (added: 2026-02-14)
- Lifecycle "created" detection: check file existence in parent tree via get_path(), not parent_tree.is_none(). The latter only catches root commits. (added: 2026-02-14)

## Refactoring
- walk_commits needs explicit lifetime `<'repo>` on return type since Commit borrows from Repository (added: 2026-02-14)
- Return `&'static str` from classify functions instead of String when all returns are literals — zero-allocation, callers .to_string() if needed (added: 2026-02-14)

## Cross-Agent Notes
- (none yet)
