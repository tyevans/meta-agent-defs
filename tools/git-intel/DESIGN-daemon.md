# git-intel Daemon Mode Design

**Status**: Proposed design, not implemented
**Author**: rust-dev
**Date**: 2026-02-14
**Target**: Repos with 1000+ commits where SessionStart hook latency becomes noticeable

## Problem Statement

As repos grow past 1000 commits, computing metrics on every SessionStart hook adds latency. Current implementation walks the entire commit history for every invocation:

- `walk_commits()` traverses HEAD to oldest commit
- `metrics` computes diffs for every commit to get line counts
- `churn` walks commits and diffs to rank file volatility
- Cold-start latency grows linearly with commit count

For this repo (89 commits): negligible. For large monorepos (10k+ commits): 500ms-2s per hook invocation.

**Solution**: A daemon watches `.git/` for changes, maintains a metrics cache, and serves results in <1ms.

---

## Architecture

### Components

1. **Daemon Process** (`git-intel daemon start`)
   - Long-running background process (one per repo)
   - Watches `.git/refs/` and `.git/HEAD` for changes
   - Recomputes metrics incrementally when new commits arrive
   - Writes cache to `.git/git-intel-cache/`
   - Exposes results via Unix domain socket

2. **CLI Client** (existing `git-intel metrics`, etc.)
   - Attempts to connect to daemon socket first
   - If daemon running: reads from cache (sub-1ms)
   - If daemon not running: falls back to current on-the-fly computation (no regression)

3. **Cache Layer**
   - Lives in `.git/git-intel-cache/` (gitignored by default)
   - Keyed by `HEAD commit hash + time window`
   - JSON files, one per metric type

---

## Watch Mechanism

**Recommendation: Polling with smart invalidation**

### Why Not inotify/fswatch

- **Platform dependency**: inotify is Linux-only, FSEvents is macOS-only, requires conditional compilation
- **Event noise**: `.git/` is noisy (lock files, index updates, gc, etc.). Filtering signal from noise is complex
- **Async dependency**: File watching crates (notify, etc.) require async runtime, adding tokio/async-std dependency
- **Complexity**: Edge cases (simultaneous git operations, rebase/merge conflicts) make event-based watching brittle

### Polling Design

Poll `.git/refs/heads/<current_branch>` modification time every **2 seconds**:

```rust
struct RepoWatcher {
    repo_path: PathBuf,
    last_head_commit: Oid,
    poll_interval: Duration,
}

impl RepoWatcher {
    fn check_for_changes(&mut self, repo: &Repository) -> bool {
        let current_head = repo.head().and_then(|h| h.target()).ok();
        match current_head {
            Some(oid) if oid != self.last_head_commit => {
                self.last_head_commit = oid;
                true  // Cache is stale, recompute
            }
            _ => false,
        }
    }
}
```

**Advantages**:
- Zero dependencies beyond git2
- Works identically on all platforms
- No async runtime needed
- Robust to git operation edge cases (rebase, merge, gc)
- 2-second worst-case staleness is acceptable for this use case

**Tradeoff**: 2-second lag between commit and cache refresh. Acceptable because SessionStart hooks fire on session boundaries (minutes/hours apart), not sub-second intervals.

---

## Cache Format

### Storage Location

`.git/git-intel-cache/` — automatically gitignored, co-located with repo state.

### Cache Files

One JSON file per metric type + time window combination:

```
.git/git-intel-cache/
├── HEAD                    # Stores current cached HEAD commit hash
├── metrics-all.json        # Full history metrics
├── metrics-7d.json         # Last 7 days metrics
├── metrics-30d.json        # Last 30 days metrics
├── churn-all.json
├── churn-7d.json
├── lifecycle-{hash}.json   # Per-query cache (hash of file list)
└── patterns-all.json
```

### Cache Entry Format

Each cache file contains:

```json
{
  "head_commit": "a1b2c3d",
  "computed_at": "2026-02-14T10:30:00Z",
  "result": { /* actual MetricsOutput, ChurnOutput, etc. */ }
}
```

### Invalidation Strategy

**Invalidate on HEAD change**:
- Compare `HEAD` commit hash in cache vs current repo HEAD
- On mismatch: recompute all caches

**TTL: None** — cache lives until HEAD changes or daemon restarts. No time-based expiry because:
- Metrics for a given HEAD are immutable (commits don't change)
- Time windows (`--since`) are recomputed on HEAD change anyway
- Avoids clock-skew issues

### Cache Key Design

For metrics/churn/patterns:
- Key = `{subcommand}-{time_window}`
- Example: `metrics-7d.json` = metrics for last 7 days

For lifecycle (file-specific):
- Key = `lifecycle-{hash(file_list)}.json`
- Example: `lifecycle-3f8a9b.json` = lifecycle for specific file set
- Max 10 lifecycle caches (LRU eviction)

---

## IPC Mechanism

**Recommendation: Unix Domain Socket**

### Why Unix Domain Socket

- **Fast**: Near-zero overhead, no serialization beyond JSON
- **Standard**: Widely supported, no platform quirks
- **Secure**: Filesystem permissions control access
- **No async needed**: Synchronous socket I/O is sufficient

### Socket Location

`/tmp/git-intel-{repo_hash}.sock`

Where `repo_hash = sha256(repo.path()).hex()[..8]`

Example: `/tmp/git-intel-3f8a9bcd.sock`

### Protocol

**Request** (CLI → daemon):
```json
{
  "subcommand": "metrics",
  "since": "2026-01-15",
  "limit": 10
}
```

**Response** (daemon → CLI):
```json
{
  "status": "hit",  // or "miss" if cache is stale
  "result": { /* MetricsOutput */ }
}
```

On cache miss: daemon returns `"status": "computing"` and client falls back to on-the-fly computation (no blocking).

### Fallback Behavior

If socket doesn't exist or connection fails:
- CLI silently falls back to current on-the-fly computation
- No error message (daemon absence is not an error)
- User experience identical to pre-daemon behavior

---

## CLI Changes

### New Subcommand: `git-intel daemon`

```
git-intel daemon start      # Start daemon in background
git-intel daemon stop       # Graceful shutdown
git-intel daemon status     # Show daemon state
git-intel daemon restart    # Stop + start
```

### Daemon Start

```bash
git-intel daemon start
```

Output:
```
Daemon started for repo /home/ty/workspace/meta-agent-defs
Socket: /tmp/git-intel-3f8a9bcd.sock
PID: 12345
```

Writes PID to `.git/git-intel-cache/daemon.pid` for stop/status.

### Daemon Stop

```bash
git-intel daemon stop
```

Reads PID from `.git/git-intel-cache/daemon.pid`, sends SIGTERM, waits up to 5s for graceful shutdown.

### Daemon Status

```bash
git-intel daemon status
```

Output:
```json
{
  "running": true,
  "pid": 12345,
  "uptime": "3h 15m",
  "cache_entries": 8,
  "last_update": "2026-02-14T10:30:00Z",
  "head_commit": "a1b2c3d"
}
```

### Existing Subcommands (Transparent Upgrade)

No flag changes. Behavior:

1. Check if socket exists at `/tmp/git-intel-{repo_hash}.sock`
2. If yes: send request, read response, print result
3. If no or connection fails: fall back to current on-the-fly logic

**User-visible change**: None. Metrics just get faster when daemon is running.

---

## Incremental Computation

### Strategy: Hybrid (Incremental + Boundary Recompute)

**For metrics**:
- **Incremental**: Maintain running totals for commit type counts, daily activity
- **Recompute**: Velocity stats (need full history to compute min/max accurately)
- Cache stores both incremental state and final output

**For churn**:
- **Recompute only**: File churn is a ranking problem — need full diff history to rank accurately
- No meaningful incremental optimization (would need to maintain per-file diff state)

**For lifecycle**:
- **Incremental**: Append new commit data to file history
- Track file existence per commit (binary: exists/doesn't exist + line count)

**For patterns**:
- **Recompute only**: Pattern detection (fix-after-feat, convergence) requires analyzing commit sequences
- Incremental pattern detection is complex (need sliding window state machine)

### What Gets Cached

| Metric | Incremental? | Cache Contains |
|--------|--------------|----------------|
| `metrics` | Partial | Commit type counts (incremental), velocity (recompute) |
| `churn` | No | Full ranked file list (recompute) |
| `lifecycle` | Yes | Per-commit file state (append-only) |
| `patterns` | No | Full pattern analysis (recompute) |

### Incremental State Format

For `metrics`, cache stores:

```json
{
  "head_commit": "a1b2c3d",
  "state": {
    "type_counts": {"feat": 12, "fix": 8, ...},
    "daily_counts": {"2026-02-14": 5, ...},
    "commit_line_counts": [23, 45, 12, ...]  // for velocity recompute
  },
  "result": { /* final MetricsOutput */ }
}
```

On new commits:
1. Walk commits from last cached HEAD to current HEAD
2. Increment type_counts, daily_counts
3. Append to commit_line_counts
4. Recompute velocity from full commit_line_counts
5. Write updated cache

**Complexity**: O(new commits) instead of O(all commits).

---

## Dependencies

### New Crates

**None required for recommended design.**

Polling-based watch + Unix domain socket are both in Rust std::

- `std::os::unix::net::UnixListener` for socket server
- `std::os::unix::net::UnixStream` for socket client
- `std::thread::sleep` for polling interval

### Optional (Future Enhancement)

- **notify** (v6): If we switch to event-based watching
  - Pros: Lower polling overhead
  - Cons: Adds async runtime dependency (tokio or async-std)
- **rusqlite**: If cache grows large enough to warrant SQLite
  - Current design: ~10-20 JSON files < 100KB each
  - Threshold: 50+ cache files or >10MB total cache size

---

## Risks and Alternatives

### Risk: Over-Engineering for Current Scale

**Current repo size**: 89 commits
**Estimated on-the-fly latency**: <50ms
**Daemon complexity**: 500+ lines of new code + new failure modes

**Mitigation**: Don't build daemon for this repo. Build it when latency becomes actual pain (1000+ commits, >200ms SessionStart hook).

**Threshold recommendation**:
- 0-500 commits: No daemon, on-the-fly only
- 500-1000 commits: Simple file-based cache (see Alternative 1)
- 1000+ commits: Full daemon worth the complexity

### Alternative 1: Simple File-Based Cache (No Daemon)

**Design**:
- `git-intel metrics` checks if `.git/git-intel-cache/metrics-all.json` exists
- If exists AND `head_commit` matches current HEAD: read cache, print, exit
- If stale or missing: compute, write cache, print

**Pros**:
- Zero background processes
- ~50 lines of code (cache read/write wrapper)
- Works for 90% of use cases

**Cons**:
- First invocation after commit is still slow (cold start)
- No incremental computation (always full recompute on cache miss)

**When to use**: Repos with 100-1000 commits where occasional cold-start latency is acceptable.

### Alternative 2: Git Hook-Based Cache

**Design**:
- Install a `post-commit` git hook that runs `git-intel metrics --update-cache`
- Hook runs synchronously after every commit, keeps cache hot

**Pros**:
- No daemon, no polling
- Cache always warm (updated immediately after commit)

**Cons**:
- Slows down `git commit` (blocks on metrics computation)
- Requires hook installation (intrusive, may conflict with existing hooks)
- Doesn't handle branch switches (would need `post-checkout` too)

**When to use**: Never. Post-commit hook latency is worse than SessionStart latency.

---

## Recommendation by Repo Size

| Commits | Latency | Recommendation |
|---------|---------|----------------|
| 0-100 | <20ms | No optimization needed |
| 100-500 | 20-100ms | File-based cache (Alt 1) |
| 500-1000 | 100-200ms | File-based cache (Alt 1) |
| 1000-5000 | 200-800ms | Full daemon (this design) |
| 5000+ | 800ms-2s+ | Full daemon + incremental computation |

**For this repo (89 commits)**: Do nothing. Build daemon when we hit 1000 commits or when SessionStart hook latency exceeds 200ms in practice.

---

## Implementation Phases

If/when we build this, ship in 3 phases:

### Phase 1: File-Based Cache (No Daemon)
- Add cache read/write to existing subcommands
- Test with large repos (git clone linux.git, measure speedup)
- **Deliverable**: 2-10x speedup for repeat queries
- **Effort**: 1-2 hours

### Phase 2: Daemon Scaffold
- Add `daemon start/stop/status` subcommands
- Implement polling watch loop
- Socket IPC (request/response)
- **Deliverable**: Daemon runs, serves cached results
- **Effort**: 4-6 hours

### Phase 3: Incremental Computation
- Implement incremental state for metrics
- Add lifecycle append-only updates
- Benchmark improvements
- **Deliverable**: 10-100x speedup on large repos
- **Effort**: 6-8 hours

**Total effort**: 12-16 hours for full daemon mode.

---

## Open Questions

1. **Multi-branch support**: Should daemon cache metrics per branch or just HEAD?
   - Recommendation: HEAD only (YAGNI — SessionStart runs on current branch)

2. **Cache eviction policy**: What happens when cache grows to 100MB?
   - Recommendation: LRU eviction, max 50 cache files OR 100MB total

3. **Daemon auto-start**: Should CLI auto-start daemon if not running?
   - Recommendation: No. Explicit `daemon start` required (avoid surprise background processes)

4. **Windows support**: Unix domain sockets don't exist on Windows.
   - Recommendation: Use named pipes on Windows (requires conditional compilation)

5. **Daemon crash recovery**: What if daemon dies mid-computation?
   - Recommendation: CLI fallback handles this gracefully (no stale cache reads)

---

## Conclusion

**Don't build daemon mode yet.** Start with Alternative 1 (file-based cache) when we hit 500 commits.

Build full daemon when:
- Commit count > 1000
- SessionStart hook latency > 200ms measured in practice
- Or when working on repos that already meet these thresholds

The design is ready. The implementation should wait for actual need.
