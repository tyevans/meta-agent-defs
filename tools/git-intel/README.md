# git-intel

Git history analyzer that outputs JSON for consumption by hooks and skills.

## Build

```bash
cd tools/git-intel
cargo build --release
```

The binary is at `target/release/git-intel`.

## Usage

```bash
git-intel <subcommand> [--repo <path>] [--since <YYYY-MM-DD>] [--limit <n>]
```

### Global flags

| Flag | Default | Description |
|------|---------|-------------|
| `--repo <path>` | `.` | Path to git repository |
| `--since <date>` | all history | Limit to commits after YYYY-MM-DD |
| `--limit <n>` | unlimited | Cap output items (applies to churn, metrics activity, patterns) |

### Subcommands

#### metrics

Commit type distribution (conventional commits), daily activity bursts, and velocity stats.

```bash
git-intel metrics --repo /path/to/repo --limit 10
```

Output fields: `commit_types`, `activity` (per-day counts), `velocity` (avg/max/min lines per commit), `total_commits`.

#### churn

File volatility ranking by total lines added + removed, sorted descending.

```bash
git-intel churn --repo /path/to/repo --limit 20
```

Output fields: `files` (path, additions, deletions, total_churn, commit_count), `total_files`, `total_commits_analyzed`.

#### lifecycle

Track specific files across commits showing growth, shrinkage, and modification history.

```bash
git-intel lifecycle --repo /path/to/repo skills/retro/SKILL.md README.md
```

Output fields: per-file `exists`, `current_lines`, `history` (commit, date, message, lines, additions, deletions, net_change, status).

#### patterns

Detect fix-after-feat sequences, multi-edit chains (files touched 3+ times), and file size convergence.

```bash
git-intel patterns --repo /path/to/repo --limit 5
```

Output fields: `fix_after_feat` (feat/fix commit pairs with gap), `multi_edit_chains` (hot files), `convergence` (file pairs within 10% size), `total_commits_analyzed`.

## Dependencies

- `git2` — libgit2 bindings for repository access
- `clap` — CLI argument parsing (derive macros)
- `serde` + `serde_json` — JSON serialization
- `chrono` — date parsing and formatting
- `anyhow` — error handling
