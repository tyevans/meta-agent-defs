# git-intel

Git history analyzer that surfaces risk signals, ownership gaps, and velocity patterns as structured JSON for programmatic consumption by hooks, skills, and scripts.

**Why git-intel?** Git commit history contains valuable signals about technical risk (fix-after-feat patterns), ownership fragility (bus factor), and code volatility (churn hotspots). These signals are hard to extract with basic `git log` commands and even harder to consume programmatically. git-intel extracts them as structured JSON with file-based caching, relative date support, and optional ML-based commit classification — making git history intelligence accessible to automation.

**Primary use case**: Integration into SessionStart/SessionEnd hooks, /status skill, and other workflow automation where you need quantified git metrics without hand-parsing log output or writing custom analysis scripts.

## Quick Start

Build the binary:

```bash
cd tools/git-intel
cargo build --release
```

Run your first command:

```bash
./target/release/git-intel metrics --since 30d
```

Output is pretty-printed JSON to stdout. Pipe to `jq` for field extraction or redirect to a file for storage.

## Global Flags

All subcommands accept these global flags:

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--repo` | path | `.` | Path to git repository (discovers .git from current or parent dirs) |
| `--since` | date/relative | all history | Limit history to commits after this date. Accepts ISO dates (YYYY-MM-DD) or relative formats (30d, 4w, 6m, 1y) |
| `--until` | date/relative | now | Limit history to commits before this date (inclusive). Same formats as --since |
| `--limit` | number | unlimited | Cap number of output items (applies to: metrics activity/commit_types, churn files, patterns sequences, hotspots/authors directories, trends windows) |
| `--no-cache` | flag | false | Bypass cache and always recompute (useful for cache invalidation after data changes) |
| `--ml` | flag | false | Enable ML-based commit classification (requires `--model-dir` and ML feature compiled in) |
| `--model-dir` | path | none | Path to ONNX model directory containing model.onnx, tokenizer.json, label_mapping.json (required when `--ml` is used) |

**Relative date examples**:
- `--since 30d` = last 30 days
- `--since 6m` = last 6 months
- `--since 1y` = last year
- `--since 4w` = last 4 weeks

## Subcommands

### metrics

**What it does**: Commit type distribution (via conventional commits classification), daily activity bursts, velocity stats (avg/max/min lines changed per commit), and ticket references extracted from commit messages.

**When to use it**: Session pulse cards, velocity tracking, commit type health checks (high fix rate = quality issue), ticket traceability.

**Syntax**:

```bash
# Last 90 days, top 10 commit types
git-intel metrics --since 90d --limit 10

# Specific date range with ML classification
git-intel metrics --since 2025-01-01 --until 2025-02-01 --ml --model-dir ./tools/data/onnx-model
```

**Output fields**:

| Field | Type | Description |
|-------|------|-------------|
| `commit_types` | array | List of commit types with count and percentage (sorted by count descending) |
| `commit_types[].type` | string | Commit type (feat, fix, refactor, docs, chore, merge, revert, release, other) |
| `commit_types[].count` | number | Number of commits of this type |
| `commit_types[].percentage` | number | Percentage of total commits (0-100) |
| `activity` | array | Daily commit counts sorted by date descending |
| `activity[].date` | string | Date in YYYY-MM-DD format |
| `activity[].commits` | number | Number of commits on that date |
| `velocity` | object | Line-change velocity statistics |
| `velocity.avg_lines_per_commit` | number | Mean lines changed (additions + deletions) per commit |
| `velocity.max_lines_in_commit` | number | Maximum lines changed in a single commit |
| `velocity.min_lines_in_commit` | number | Minimum lines changed in a single commit |
| `velocity.total_lines_changed` | number | Sum of all line changes across analyzed commits |
| `total_commits` | number | Total number of commits analyzed in the time window |
| `ticket_refs` | array | Ticket/issue references found in commit messages (e.g., #123, JIRA-456) sorted by frequency |
| `ticket_refs[].ticket` | string | Ticket identifier extracted from commit message |
| `ticket_refs[].count` | number | Number of commits mentioning this ticket |

**Example** (from this repo, last 30 days):

```json
{
  "commit_types": [
    {"type": "feat", "count": 66, "percentage": 50.77},
    {"type": "chore", "count": 36, "percentage": 27.69},
    {"type": "docs", "count": 12, "percentage": 9.23}
  ],
  "activity": [
    {"date": "2026-02-16", "commits": 1},
    {"date": "2026-02-15", "commits": 34},
    {"date": "2026-02-14", "commits": 60}
  ],
  "velocity": {
    "avg_lines_per_commit": 358.59,
    "max_lines_in_commit": 4307,
    "min_lines_in_commit": 0,
    "total_lines_changed": 46617
  },
  "total_commits": 130,
  "ticket_refs": [
    {"ticket": "#1", "count": 1}
  ]
}
```

### churn

**What it does**: File volatility ranking by total lines changed (additions + deletions), sorted by churn descending. Identifies the most actively modified files.

**When to use it**: Hotspot detection, identifying files under heavy development, finding code that may benefit from refactoring (high churn often correlates with design issues).

**Syntax**:

```bash
# Top 20 highest-churn files in last 30 days
git-intel churn --since 30d --limit 20

# All files changed in specific date range
git-intel churn --since 2025-01-01 --until 2025-01-31
```

**Output fields**:

| Field | Type | Description |
|-------|------|-------------|
| `files` | array | File churn entries sorted by total_churn descending |
| `files[].path` | string | File path relative to repository root |
| `files[].additions` | number | Total lines added across all commits touching this file |
| `files[].deletions` | number | Total lines deleted across all commits touching this file |
| `files[].total_churn` | number | Sum of additions + deletions |
| `files[].commit_count` | number | Number of commits that touched this file |
| `total_files` | number | Total number of unique files changed in the time window |
| `total_commits_analyzed` | number | Total commits walked during analysis |

**Example** (from this repo, last 30 days, top 10):

```json
{
  "files": [
    {
      "path": "tools/git-intel/tests/integration.rs",
      "additions": 3035,
      "deletions": 201,
      "total_churn": 3236,
      "commit_count": 9
    },
    {
      "path": "tools/commit-labeler/uv.lock",
      "additions": 2292,
      "deletions": 29,
      "total_churn": 2321,
      "commit_count": 5
    },
    {
      "path": "README.md",
      "additions": 613,
      "deletions": 429,
      "total_churn": 1042,
      "commit_count": 19
    }
  ],
  "total_files": 162,
  "total_commits_analyzed": 130
}
```

### lifecycle

**What it does**: Track specific files across commits, showing history of changes (created, modified, grown, shrunk, deleted) with line counts and diff stats at each commit.

**When to use it**: Understanding how a file evolved over time, debugging when/how a file was modified, tracking file size growth for capacity planning.

**Syntax**:

```bash
# Track two files over last 6 months
git-intel lifecycle --since 6m skills/retro/SKILL.md README.md

# Track a single file across all history
git-intel lifecycle src/main.rs
```

**Output fields**:

| Field | Type | Description |
|-------|------|-------------|
| `files` | array | Per-file lifecycle data (one entry per tracked file) |
| `files[].path` | string | File path as provided in arguments |
| `files[].exists` | boolean | Whether the file exists in current HEAD |
| `files[].current_lines` | number/null | Line count at HEAD (null if file doesn't exist) |
| `files[].history` | array | Chronological list of commits touching this file (newest first) |
| `files[].history[].commit` | string | Short commit hash (7 chars) |
| `files[].history[].date` | string | Commit date in YYYY-MM-DD format |
| `files[].history[].message` | string | First line of commit message |
| `files[].history[].lines` | number/null | File line count after this commit (null if deleted) |
| `files[].history[].additions` | number | Lines added in this commit |
| `files[].history[].deletions` | number | Lines deleted in this commit |
| `files[].history[].net_change` | number | Net change (additions - deletions) |
| `files[].history[].status` | string | Change type: created, modified, grown, shrunk, deleted, touched |

**Example** (tracking two files across all history):

```json
{
  "files": [
    {
      "path": "tools/git-intel/README.md",
      "exists": true,
      "current_lines": 76,
      "history": [
        {
          "commit": "f95289b",
          "date": "2026-02-14",
          "message": "feat: build git-intel Rust CLI with metrics, churn, lifecycle, and patterns subcommands",
          "lines": 76,
          "additions": 76,
          "deletions": 0,
          "net_change": 76,
          "status": "created"
        }
      ]
    },
    {
      "path": "skills/retro/SKILL.md",
      "exists": true,
      "current_lines": 390,
      "history": [
        {
          "commit": "0a593bf",
          "date": "2026-02-16",
          "message": "feat: add sharpening gates to 10 skills that produce recommendations",
          "lines": 390,
          "additions": 19,
          "deletions": 4,
          "net_change": 15,
          "status": "modified"
        },
        {
          "commit": "a428f2e",
          "date": "2026-02-14",
          "message": "feat: add /evolution and /drift skills, enrich /retro and /status with git-intel",
          "lines": 373,
          "additions": 107,
          "deletions": 4,
          "net_change": 103,
          "status": "modified"
        }
        // ... (10 more history entries)
      ]
    }
  ]
}
```

### patterns

**What it does**: Detect risk signals in commit patterns — fix-after-feat sequences (likely regressions), multi-edit chains (files under heavy churn), directory churn aggregates, and temporal clusters (burst activity). Includes a signal system with severity scores for automation. Filters out cosmetic fixes (style/typo) and gravity files (>15% commit frequency) to reduce false positives.

**When to use it**: Code quality audits, regression risk assessment, identifying unstable files/directories, detecting firefighting activity (temporal clusters of fixes).

**Syntax**:

```bash
# Last 90 days, top 5 sequences
git-intel patterns --since 90d --limit 5

# With ML classification for better signal precision
git-intel patterns --since 30d --ml --model-dir ./tools/data/onnx-model
```

**Output fields**:

| Field | Type | Description |
|-------|------|-------------|
| `fix_after_feat` | array | Fix commits within 5 commits of a feat commit, sharing files (excludes cosmetic fixes, filters gravity files) |
| `fix_after_feat[].feat_commit` | string | Short hash of the feat commit |
| `fix_after_feat[].feat_date` | string | Date of feat commit (YYYY-MM-DD) |
| `fix_after_feat[].feat_message` | string | First line of feat commit message |
| `fix_after_feat[].fix_commit` | string | Short hash of the fix commit |
| `fix_after_feat[].fix_date` | string | Date of fix commit |
| `fix_after_feat[].fix_message` | string | First line of fix commit message |
| `fix_after_feat[].gap_commits` | number | Number of commits between feat and fix (0 = adjacent) |
| `fix_after_feat[].shared_files` | array | Files touched by both commits (gravity files excluded) |
| `multi_edit_chains` | array | Files touched 3+ times with >100 total lines changed (top 10 by edit count) |
| `multi_edit_chains[].path` | string | File path |
| `multi_edit_chains[].edit_count` | number | Number of times this file was edited |
| `multi_edit_chains[].total_churn` | number | Sum of all line changes to this file |
| `multi_edit_chains[].type_distribution` | object | Commit type counts (e.g., {"feat": 3, "fix": 2}) |
| `multi_edit_chains[].commits` | array | List of commits touching this file |
| `multi_edit_chains[].commits[].commit` | string | Short hash |
| `multi_edit_chains[].commits[].date` | string | Date (YYYY-MM-DD) |
| `multi_edit_chains[].commits[].message` | string | First line of commit message |
| `multi_edit_chains[].commits[].commit_type` | string | Classified commit type |
| `directory_chains` | array | Directories touched 3+ times (aggregated from file-level edits, top 10 by churn) |
| `directory_chains[].path` | string | Directory prefix (depth=1 by default) |
| `directory_chains[].total_edit_count` | number | Number of distinct commits touching this directory |
| `directory_chains[].total_churn` | number | Sum of all file churn within this directory |
| `directory_chains[].files` | array | List of files in this directory (sorted) |
| `temporal_clusters` | array | 3+ commits of same type within 1-hour window |
| `temporal_clusters[].cluster_type` | string | Commit type of the cluster (all commits in cluster share this type) |
| `temporal_clusters[].start_time` | string | ISO timestamp of first commit in cluster |
| `temporal_clusters[].end_time` | string | ISO timestamp of last commit in cluster |
| `temporal_clusters[].commit_count` | number | Number of commits in cluster |
| `temporal_clusters[].commits` | array | List of commits in cluster (same structure as multi_edit_chains.commits) |
| `temporal_clusters[].affected_files` | array | All unique files touched by cluster commits (sorted) |
| `total_commits_analyzed` | number | Total commits walked |
| `signals` | array | Actionable risk signals extracted from patterns (for automation) |
| `signals[].kind` | string | Signal type: fix_after_feat, fix_after_refactor |
| `signals[].severity` | number | Severity score 0.0-1.0 (proximity and file overlap weighted) |
| `signals[].message` | string | Human-readable one-liner describing the signal |
| `signals[].commits` | array | Short hashes of commits involved (e.g., [feat_hash, fix_hash]) |
| `signals[].files` | array | File paths involved in the signal |
| `gravity_files` | array | Files touched by >15% of commits (excluded from signal detection, sorted) |

**Example** (from this repo, last 30 days, limited to 2 items per section):

```json
{
  "fix_after_feat": [
    {
      "feat_commit": "e7af4ee",
      "feat_date": "2026-02-15",
      "feat_message": "feat: cross-project test harness and skill telemetry hook",
      "fix_commit": "fb8f972",
      "fix_date": "2026-02-15",
      "fix_message": "fix: cold-install audit — cross-project portability for scripts and hooks",
      "gap_commits": 3,
      "shared_files": ["settings.json"]
    }
  ],
  "multi_edit_chains": [
    {
      "path": ".beads/issues.jsonl",
      "edit_count": 73,
      "total_churn": 674,
      "type_distribution": {"feat": 45, "chore": 13, "fix": 6},
      "commits": [
        {
          "commit": "888d9f2",
          "commit_type": "feat",
          "date": "2026-02-15",
          "message": "feat: 9-experiment ML training comparison + transformer-focal ONNX export"
        }
        // ... (72 more commits)
      ]
    }
  ],
  "temporal_clusters": [
    {
      "cluster_type": "feat",
      "start_time": "2026-02-08T04:27:05Z",
      "end_time": "2026-02-08T05:14:40Z",
      "commit_count": 14,
      "commits": [/* 14 commits within 47 minutes */],
      "affected_files": [/* 26 files touched by cluster */]
    }
  ],
  "signals": [
    {
      "kind": "fix_after_feat",
      "severity": 0.04,
      "message": "Fix fb8f972 likely caused by feat e7af4ee (1 shared files, 3 commits apart)",
      "commits": ["e7af4ee", "fb8f972"],
      "files": ["settings.json"]
    }
  ],
  "gravity_files": [".beads/issues.jsonl", "memory/team/retro-history.md"],
  "total_commits_analyzed": 130
}
```

**Signal precision** (measured on cross-project validation):
- `fix_after_refactor`: 89% true positive rate (near-viable for automation)
- `fix_after_feat`: 40% true positive rate (needs human review)

### hotspots

**What it does**: Directory-level churn aggregation. Groups file churn by path prefix at configurable depth and adds commit type distribution per directory. Identifies which parts of the codebase are under the most development pressure.

**When to use it**: Codebase health dashboards, architectural hotspot detection, prioritizing refactoring efforts, understanding where the team spends effort.

**Syntax**:

```bash
# Top-level directories, last 90 days, top 10
git-intel hotspots --depth 1 --since 90d --limit 10

# Two-level depth (e.g., src/utils, src/models)
git-intel hotspots --depth 2 --since 6m
```

**Output fields**:

| Field | Type | Description |
|-------|------|-------------|
| `directories` | array | Directory hotspots sorted by total_churn descending |
| `directories[].path` | string | Directory prefix at specified depth ("." for root-level files) |
| `directories[].additions` | number | Total lines added across all files in this directory |
| `directories[].deletions` | number | Total lines deleted across all files in this directory |
| `directories[].total_churn` | number | Sum of additions + deletions |
| `directories[].commit_count` | number | Sum of file-level commit counts (may overcount if multiple files change in same commit) |
| `directories[].file_count` | number | Number of unique files in this directory |
| `directories[].type_distribution` | object | Commit type counts for commits touching this directory |
| `total_directories` | number | Total number of unique directories at specified depth |
| `total_commits_analyzed` | number | Total commits walked |
| `depth` | number | Directory depth used (echoed from --depth flag) |

**Example** (from this repo, last 30 days, depth 1, top 8):

```json
{
  "directories": [
    {
      "path": "tools",
      "additions": 16887,
      "deletions": 1138,
      "total_churn": 18025,
      "commit_count": 123,
      "file_count": 44,
      "type_distribution": {"feat": 16, "chore": 2}
    },
    {
      "path": "skills",
      "additions": 7616,
      "deletions": 1622,
      "total_churn": 9238,
      "commit_count": 167,
      "file_count": 31,
      "type_distribution": {"feat": 26, "fix": 4, "chore": 3}
    },
    {
      "path": "docs",
      "additions": 4800,
      "deletions": 5,
      "total_churn": 4805,
      "commit_count": 17,
      "file_count": 8,
      "type_distribution": {"feat": 6, "docs": 3}
    }
    // ... (5 more directories)
  ],
  "total_directories": 15,
  "total_commits_analyzed": 130,
  "depth": 1
}
```

### authors

**What it does**: Per-directory ownership analysis — top contributor, bus factor (minimum authors representing >50% of commits), and per-author stats (commits, lines added/deleted). Respects .mailmap for author identity consolidation.

**When to use it**: Ownership gap detection, bus factor risk assessment, understanding team distribution across the codebase, finding single points of failure.

**Syntax**:

```bash
# Top-level directory ownership, last 6 months
git-intel authors --depth 1 --since 6m

# Two-level depth with ML classification
git-intel authors --depth 2 --since 90d --limit 15
```

**Output fields**:

| Field | Type | Description |
|-------|------|-------------|
| `directories` | array | Per-directory author statistics sorted by total_commits descending |
| `directories[].path` | string | Directory prefix at specified depth |
| `directories[].authors` | array | Author stats sorted by commits descending |
| `directories[].authors[].name` | string | Author name (resolved via .mailmap if present) |
| `directories[].authors[].email` | string | Author email (canonical email after .mailmap resolution) |
| `directories[].authors[].commits` | number | Number of commits by this author in this directory |
| `directories[].authors[].lines_added` | number | Total lines added by this author in this directory |
| `directories[].authors[].lines_deleted` | number | Total lines deleted by this author in this directory |
| `directories[].top_contributor` | string | Name of author with most commits in this directory |
| `directories[].bus_factor` | number | Minimum number of authors needed to exceed 50% of total commits (1 = single point of failure) |
| `directories[].total_commits` | number | Total commits touching this directory |
| `total_authors` | number | Total unique authors across entire repository (global count, not per-directory) |
| `total_commits_analyzed` | number | Total commits walked |
| `depth` | number | Directory depth used |

**Example** (from this repo, last 30 days, depth 1, top 5 — note this repo has only 2 contributors, so `bus_factor` is 1 everywhere):

```json
{
  "directories": [
    {
      "path": ".beads",
      "authors": [
        {
          "name": "Ty Evans",
          "email": "tyler@poorlythoughtout.com",
          "commits": 72,
          "lines_added": 670,
          "lines_deleted": 162
        },
        {
          "name": "Tyler Evans",
          "email": "tyevans@gmail.com",
          "commits": 2,
          "lines_added": 31,
          "lines_deleted": 2
        }
      ],
      "top_contributor": "Ty Evans",
      "bus_factor": 1,
      "total_commits": 74
    },
    {
      "path": "skills",
      "authors": [
        {
          "name": "Ty Evans",
          "email": "tyler@poorlythoughtout.com",
          "commits": 37,
          "lines_added": 6705,
          "lines_deleted": 1379
        }
        // ... (1 more author)
      ],
      "top_contributor": "Ty Evans",
      "bus_factor": 1,
      "total_commits": 39
    }
    // ... (3 more directories)
  ],
  "total_authors": 2,
  "total_commits_analyzed": 130,
  "depth": 1
}
```

### trends

**What it does**: Multi-window temporal comparison showing how metrics change over time. Divides history into N equal time windows (default: 4 windows of 90 days each) and computes metrics + churn for each. Includes delta analysis (commit trend, fix rate trend) and dormant file detection (files active in older windows but not in newest).

**When to use it**: Velocity trend analysis, detecting declining activity, identifying abandoned code (dormant files), fix rate regression monitoring.

**Syntax**:

```bash
# Default: 4 windows of 90 days each, top 5 churn files per window
git-intel trends

# Custom: 6 windows of 30 days, top 10 churn files
git-intel trends --windows 6 --window-size 30 --limit 10
```

**Output fields**:

| Field | Type | Description |
|-------|------|-------------|
| `windows` | array | Time window data (index 0 = newest, ascending index = older) |
| `windows[].index` | number | Window index (0 = most recent) |
| `windows[].label` | string | Human-readable label (e.g., "2025-01-01 to 2025-03-31") |
| `windows[].since` | string | Window start date (YYYY-MM-DD) |
| `windows[].until` | string | Window end date (YYYY-MM-DD) |
| `windows[].total_commits` | number | Commits in this window |
| `windows[].type_distribution` | object | Commit type counts in this window |
| `windows[].velocity` | number | Commits per day (total_commits / window_size_days) |
| `windows[].top_churn_files` | array | Top N files by churn in this window (N = --limit, default 5) |
| `window_count` | number | Number of windows (echoed from --windows) |
| `window_size_days` | number | Size of each window in days (echoed from --window-size) |
| `deltas` | object | Comparison of window[0] vs window[1] |
| `deltas.commit_trend` | string | Trend label: "increasing", "decreasing", "stable" (stable = within 10%) |
| `deltas.fix_rate_trend` | string | Trend label for fix commit rate (fixes / total_commits) |
| `dormant_files` | array | Files active in any older window but not in newest (sorted) |

**Example** (from this repo, 3 windows of 30 days, top 5 churn files per window — note that this repo had no activity in windows 1 and 2):

```json
{
  "windows": [
    {
      "index": 0,
      "label": "2026-01-17 to 2026-02-16",
      "since": "2026-01-17",
      "until": "2026-02-16",
      "total_commits": 130,
      "type_distribution": {"feat": 66, "chore": 36, "docs": 12, "fix": 8},
      "velocity": 4.33,
      "top_churn_files": [
        "tools/git-intel/tests/integration.rs",
        "tools/commit-labeler/uv.lock",
        "docs/team-system-guide.md"
        // ... (2 more files)
      ]
    },
    {
      "index": 1,
      "label": "2025-12-18 to 2026-01-17",
      "since": "2025-12-18",
      "until": "2026-01-17",
      "total_commits": 0,
      "type_distribution": {},
      "velocity": 0.0,
      "top_churn_files": []
    },
    {
      "index": 2,
      "label": "2025-11-18 to 2025-12-18",
      "since": "2025-11-18",
      "until": "2025-12-18",
      "total_commits": 0,
      "type_distribution": {},
      "velocity": 0.0,
      "top_churn_files": []
    }
  ],
  "window_count": 3,
  "window_size_days": 30,
  "deltas": {
    "commit_trend": "increasing",
    "fix_rate_trend": "increasing"
  },
  "dormant_files": []
}
```

## Classifier Chain

Commit classification follows a priority cascade to maximize accuracy and coverage:

1. **Merge** — Any commit with 2+ parents is classified as `merge` regardless of message
2. **Revert** — Detects both git default format (`Revert "..."`) and conventional style (`revert:` or `revert(`)
3. **Release** — Matches version tags (`v1.2.3`), "release" keyword, or "bump version" patterns
4. **Conventional commit** — Strict prefix matching: `type:`, `type(`, `type!`, or bare `type` followed by whitespace/end-of-string
   - Supported types: feat, fix, chore, docs, refactor, test, style, perf, ci, build
5. **Natural language heuristics** — Fallback patterns for non-conventional commits:
   - Past tense: "Fixed ..." → fix, "Added ..." → feat
   - Compound words: "bugfix", "bug fix", "hotfix" → fix
   - GitHub auto-close: "Fixes #N", "Closes #N" anywhere in message → fix
6. **ML model** (when `--ml` enabled) — ONNX transformer model trained on labeled commit corpus, used as final attempt before "other"
7. **Other** — Fallback for unrecognized commits

**Why the priority order matters**: A merge commit with message "fix: resolve auth bug" is still classified as `merge`, not `fix`. A commit starting with "Revert \"feat: add login\"" is classified as `revert`, not `feat`. The cascade ensures structural commit properties (merge/revert/release) take precedence over message-based classification.

**ML integration**: The ML classifier extends the chain by inserting before "other" as a final attempt. When rule-based classification returns "other" and ML is enabled, the model processes the commit message. If the top prediction exceeds the confidence threshold (default: 0.5), that label is used; otherwise "other" is kept. This design preserves rule-based determinism for common cases while leveraging ML for edge cases.

## Signal System

**What signals are**: Actionable risk indicators extracted from commit patterns. Unlike raw pattern data (fix-after-feat, multi-edit chains), signals combine pattern detection with severity scoring and precision filters to surface high-confidence issues.

**Signal types**:

- `fix_after_feat` — Fix commit within 5 commits of a feat commit, with file overlap
- `fix_after_refactor` — Fix commit within 5 commits of a refactor commit, with file overlap

**How signals are generated**:

1. **File intersection**: For each fix commit, scan the previous 5 commits for feat/refactor commits. If any share files with the fix, a signal is generated.
2. **Severity scoring** (0.0-1.0): Computed as `(1 / (gap + 1)) * (min(shared_files, 5) / 5)`. Factors:
   - **Gap**: Closer fixes score higher (gap=1 → 0.5 multiplier, gap=5 → 0.17 multiplier)
   - **File overlap**: More shared files score higher (capped at 5 files)
3. **Precision filters**: Two filters reduce false positives before signal emission:
   - **Cosmetic-fix exclusion**: Fix commits matching style/typo/format patterns (e.g., "fix typo", "fix formatting") are excluded
   - **Gravity-file detection**: Files appearing in >15% of commits (20-commit minimum) are excluded from file intersection. These are high-traffic files like package-lock.json, CHANGELOG.md, or central config files that appear in many unrelated commits.

**Precision numbers** (cross-project validated on 18 repos):

- `fix_after_refactor`: **89% true positive rate** — near-viable for automation
- `fix_after_feat`: **40% true positive rate** — needs more filtering

See [PRECISION-STUDY.md](PRECISION-STUDY.md) for full methodology, validation process, and per-repo breakdown.

## Cache

**Location**: `.git/git-intel-cache/` (inside the repository's `.git` directory)

**Key structure**: `{subcommand}-{since_epoch}-{until_epoch}.json`
- `since_epoch` and `until_epoch` are Unix timestamps (from `--since`/`--until` flags)
- When a bound is omitted, the key uses `"all"` instead of a timestamp
- Example: `metrics-1768435200-all.json` (--since 2026-01-15, no --until)
- `lifecycle` includes a file hash: `lifecycle-{hash}-{since}-{until}.json` (files are sorted before hashing to ensure order-independence)

**Invalidation**: Cache entries store the HEAD commit hash at write time. On read, if HEAD has changed, the cache is considered stale and ignored. This ensures cache hits only occur when the repository state is unchanged.

**`--no-cache` flag**: Bypasses cache read and write entirely. Use when you need fresh results or suspect cache corruption.

**Write error handling**: Cache writes are best-effort — errors are silently ignored. This design treats cache as an optimization, not correctness. If `.git/git-intel-cache/` cannot be created or written to, the command still succeeds and returns fresh results.

**When `--until` is used**: Both `since` and `until` appear in the cache key. This allows independent caching of different time windows (e.g., `--since 30d` vs `--since 30d --until 1w` are separate cache entries).

## Building & Installation

### Prerequisites

- **Rust toolchain** (cargo) — install via [rustup](https://rustup.rs)
- No other system dependencies required for basic build
- **ONNX Runtime** (optional) — required only for ML-enabled builds (see ML / ONNX Configuration below)

### Standard Build

From this repository root:

```bash
cd tools/git-intel
cargo build --release
```

Binary will be available at `tools/git-intel/target/release/git-intel`.

### ML-Enabled Build

To enable ML-based commit classification:

```bash
cargo build --release --features ml
```

This requires ONNX Runtime installed on your system. See ML / ONNX Configuration below for setup details.

### Via install.sh

The root-level `install.sh` script can optionally build git-intel during installation:

```bash
# Interactive prompt (local install)
./install.sh

# Skip git-intel build (e.g., in CI)
./install.sh --skip-rust
```

When cargo is detected and `--skip-rust` is not used, the installer prompts to build git-intel. If you skip the build, skills that use git-intel will fall back gracefully to raw git commands.

### Cross-Project Usage

The binary location is `tools/git-intel/target/release/git-intel` (relative to the meta-agent-defs repository root). Other projects that install this workbench access the binary via absolute path resolution.

**Path resolution order** (from `bin/git-pulse.sh`):

1. `tools/git-intel/target/release/git-intel` (relative to current repo root)
2. `$SCRIPT_DIR/../tools/git-intel/target/release/git-intel` (derived from script's own location via `BASH_SOURCE`)

This allows cross-project access when meta-agent-defs is installed globally via symlinks.

## ML / ONNX Configuration

### The `ml` Cargo Feature

When built with `--features ml`, git-intel enables ML-based commit classification as a fallback in the classifier chain. The ML model runs after all rule-based classifiers (conventional commits, natural language heuristics) and before the "other" fallback. See Classifier Chain above for the full priority order.

### ORT_DYLIB_PATH Environment Variable

The `ort` crate (ONNX Runtime bindings) requires the ONNX Runtime shared library at runtime. You must set `ORT_DYLIB_PATH` to point to the library location.

**Finding the library**: If you installed ONNX Runtime via Python (e.g., via uv or pip), the library is often in the package cache:

```bash
# Example: uv cache location
export ORT_DYLIB_PATH=/home/user/.cache/uv/archive-*/onnxruntime-*/onnxruntime/capi/libonnxruntime.so.*
```

**Note**: This path is machine-specific and depends on how you installed ONNX Runtime. If the library is not found at runtime, git-intel will print an error and fall back to non-ML classification.

### Model Directory

The ONNX model lives at `tools/data/onnx-model/` (relative to meta-agent-defs repository root) and contains:

- `model.onnx` — The ONNX-exported transformer model
- `tokenizer.json` — HuggingFace tokenizer config
- `label_mapping.json` — Mapping from model output indices to commit type labels

To use the ML model at runtime:

```bash
git-intel metrics --ml --model-dir tools/data/onnx-model
```

### git-pulse.sh Auto-Detection

The `bin/git-pulse.sh` script automatically detects and enables ML features when all of the following are true:

1. `git-intel` binary is found at one of the fallback paths
2. The ONNX model exists at `tools/data/onnx-model/model.onnx` (checked in current repo root first, then meta-agent-defs root)
3. `git-intel --help` output includes `--ml` flag (confirms ML feature compiled in)
4. A runtime validation test succeeds (runs `git-intel metrics --ml` on a small dataset to verify ONNX Runtime is accessible)

If all gates pass, `git-pulse.sh` automatically adds `--ml --model-dir <path>` to all git-intel invocations. If any gate fails, it silently falls back to non-ML classification.

## Integration & Workflows

### Architecture

```
┌─────────────┐
│   Skills    │ (/status, /retro, /sprint, /evolution, /drift, /advise)
└──────┬──────┘
       │
       v
┌─────────────────┐
│ bin/git-pulse.sh │ (shared entry point: metrics + churn + patterns)
└──────┬───────────┘
       │
       v
┌─────────────┐
│  git-intel  │ (JSON subcommands: metrics, churn, patterns, lifecycle, etc.)
└──────┬──────┘
       │
       v
┌─────────────┐
│ Skill output│ (churn heatmaps, signal warnings, session metrics)
└─────────────┘
```

### Skill Integration Map

| Skill | What it uses | How it uses the data |
|-------|--------------|----------------------|
| **/status** | `git-pulse.sh` (delegates to churn) | Displays churn heatmap showing top 10 files by volatility in the status card |
| **/retro** | `git-pulse.sh` (delegates to metrics+churn+patterns), `git-intel lifecycle` | Session metrics (velocity, fix rate), mistake pattern analysis (fix-after-feat), learning survival (tracking learnings.md history) |
| **/sprint** | `git-intel patterns` | Extracts fix_after_refactor signals, injects warnings into dispatch prompts to flag risky files before work starts |
| **/evolution** | `git-intel lifecycle` | Tracks file change history — creates timeline of modifications with commit types, line count deltas, and net changes |
| **/drift** | `git-intel patterns` (optional, falls back to raw git) | Uses pattern data when available to enrich divergence analysis between related definition files |
| **/advise** | `git-pulse.sh` (full delegation) | Signal layer (Layer 4 of 4-layer recommendation system) — detects fix-after-feat patterns and churn hotspots for proactive recommendations |

### git-pulse.sh Bridge

`bin/git-pulse.sh` is the shared entry point for common git-intel operations. It runs `metrics + churn + patterns` in a single call, outputs plain text (not JSON), handles date conversion from relative formats (e.g., "8 hours ago" → ISO date), and auto-detects the ONNX model at `tools/data/onnx-model/`. Skills call git-pulse.sh for pulse metrics. Only specialized skills (`/evolution`, `/drift`, `/sprint`) call git-intel directly for specific subcommands like `lifecycle` or `patterns`.

**Why the bridge exists**: Skills need simple text output for status cards and reports, not raw JSON parsing. git-pulse.sh abstracts the "give me session metrics" use case so skills don't repeat JSON-to-text formatting logic.

### Workflow Recipes

#### Morning Orientation

```bash
/status
```

**What happens**: Status card runs `git-pulse.sh --since "7 days ago"`, which delegates to git-intel for churn data. The churn heatmap shows top 10 files by volatility — you see where the team has been working before reading any code.

**Output example**:
```
Hot:  skills/retro/SKILL.md (1580), tools/git-intel/src/main.rs (1245), README.md (1042)
```

#### Sprint Safety Check

```bash
/sprint
```

**What happens**: Before dispatching tasks, sprint runs `git-intel patterns --since 30d` and extracts signals where `kind == "fix_after_refactor"`. For each signal with severity >0.1, it injects a warning into the dispatch prompt:

```
⚠️ Signal detected: Fix abc1234 likely caused by refactor def5678 (2 shared files, 1 commit apart)
```

Agents see these warnings before modifying risky files, reducing the chance of introducing new bugs in recently-refactored code.

#### Session Wrap-Up

```bash
/retro
```

**What happens**: Retro uses `git-pulse.sh --since "8 hours ago"` for session metrics (commits, fix rate, velocity) and `git-intel patterns` for mistake analysis. If the fix rate is high (>30%), it parses fix-after-feat signals to identify which features caused regressions. The retrospective includes:

- **Session facts**: "4 commits (3 feat, 1 fix), 1245 lines changed"
- **Mistake patterns**: "Fix abc1234 followed feat def5678 on auth.rs — likely regression"
- **Learning survival**: Uses `git-intel lifecycle` to check if learnings.md files were modified, showing which team member learnings are actively maintained vs stale

## Graceful Degradation

git-intel and its ML extensions are **optional**. Skills that use git history degrade gracefully across three tiers based on what's available. This ensures you always get useful output, even in a fresh project with minimal tooling.

### Tier Table

| Tier | Requirements | What You Get |
|------|-------------|-------------|
| **Full** | git-intel binary + ONNX model + ML feature compiled in | ML-enhanced commit classification, all 7 subcommands, signal detection with 89% precision (fix_after_refactor), churn heatmaps, lifecycle tracking, ownership analysis |
| **Standard** | git-intel binary (no ML) | All 7 subcommands, rule-based commit classification, signal detection with lower precision, churn heatmaps, lifecycle tracking, ownership analysis |
| **Minimal** | Raw git only | Basic commit counts, file lists via git log, no commit type classification, no signals, no structured JSON output |

### Per-Skill Degradation

How the 6 integrated skills adapt based on available tier:

| Skill | Full Tier | Standard Tier | Minimal Tier |
|-------|-----------|---------------|--------------|
| **/status** | Churn heatmap (top 10 files) with ML-classified commit types | Churn heatmap (top 10 files) with rule-classified commit types | Top 5 files by raw git commit count (no churn calculation) |
| **/retro** | Full session metrics (velocity, fix rate, ML types) + fix-after-feat pattern analysis + learning survival tracking | Same metrics with rule-based types + pattern analysis + learning survival | Basic commit count only ("4 commits this session") |
| **/sprint** | Injects fix_after_refactor signal warnings (89% precision) into dispatch prompts | Injects fix_after_refactor warnings with lower precision (rule-based classification) | No signal warnings injected |
| **/evolution** | File lifecycle tracking with ML commit types, net change analysis, status labels (created/modified/grown/shrunk) | Same lifecycle tracking with rule-based commit types | Raw git log output for specified files (chronological list only) |
| **/drift** | Pattern data with ML commit types enriches divergence analysis | Same pattern data with rule-based types | Raw git log showing when files were last modified together |
| **/advise** | 4-layer recommendations including signal layer (fix rate, churn concentration, fix-after-feat patterns with ML) | 4-layer recommendations with signal layer using rule-based classification | 3-layer recommendations (git state + session + backlog, no signals layer) |

### Checking Your Tier

Run `bin/git-pulse.sh` from your project root (or `~/.claude/bin/git-pulse.sh` if meta-agent-defs is installed globally):

- **Full tier**: Output includes lines starting with `git-intel:` AND mentions ML model path
- **Standard tier**: Output includes `git-intel:` lines but no ML model path
- **Minimal tier**: Output shows raw git command output with no `git-intel:` prefix

The auto-detection happens at runtime via a four-gate check (see ML / ONNX Configuration above). No manual configuration needed — install the binary and model, and skills automatically upgrade to the best available tier.

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| **"git-intel: command not found"** | Binary not built, or not on PATH | Build with `cargo build --release` from `tools/git-intel/`. Use full path `./target/release/git-intel` or add to PATH |
| **"git-intel not used by git-pulse.sh"** | One of the four gates failing: binary missing, jq not installed, date format issues, or runtime ML validation failed | Check each gate: (1) Binary exists at `tools/git-intel/target/release/git-intel`, (2) `jq` is installed, (3) Date arguments are ISO format or relative (30d/6m/1y), (4) ML validation passes (run `git-intel metrics --ml` manually to test) |
| **"ML classification not activating"** | Either `ORT_DYLIB_PATH` not set, or binary not compiled with `--features ml` | Set `ORT_DYLIB_PATH` to your ONNX Runtime library (e.g., `export ORT_DYLIB_PATH=/path/to/libonnxruntime.so.*`). Rebuild with `cargo build --release --features ml` |
| **Empty or sparse output** | Time window too narrow (--since excludes most commits), or repository has few commits in the selected range | Widen the time window (e.g., `--since 90d` instead of `--since 7d`), or remove `--since` entirely to analyze full history |
| **"Cache returning stale results"** | Rare edge case if HEAD unchanged but history was rewritten (e.g., amended commits, rebased branches) | Use `--no-cache` to bypass cache and force fresh computation |
| **"jq: command not found" in git-pulse.sh** | `jq` JSON processor not installed on system | Install jq: `apt install jq` (Debian/Ubuntu), `brew install jq` (macOS), `pacman -S jq` (Arch) |
| **"Permission denied" when running binary** | Binary lacks execute permissions after build | Run `chmod +x tools/git-intel/target/release/git-intel` |
| **"No such file or directory: .git"** | Command run from non-repository directory, or repository not initialized | Run from inside a git repository, or use `--repo /path/to/repo` to specify location |
| **"ONNX Runtime library not found"** | `ORT_DYLIB_PATH` points to wrong location or ONNX Runtime not installed | Verify library exists at the path in `ORT_DYLIB_PATH`. If missing, install ONNX Runtime (e.g., `pip install onnxruntime` or system package) |
| **"Model directory missing model.onnx"** | `--model-dir` path incorrect or model files not present | Verify `model.onnx`, `tokenizer.json`, and `label_mapping.json` exist at the specified path. Default location: `tools/data/onnx-model/` |

## Dependencies

- `git2` — libgit2 bindings for repository access
- `clap` — CLI argument parsing (derive macros)
- `serde` + `serde_json` — JSON serialization
- `chrono` — date parsing and formatting
- `anyhow` — error handling
- `ort` (ML feature only) — ONNX Runtime bindings for ML inference
- `tokenizers` (ML feature only) — HuggingFace tokenizer library
