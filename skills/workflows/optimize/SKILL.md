---
name: optimize
description: "Identify performance bottlenecks, measure baselines, implement targeted fixes, and iterate until a measurable target is reached. Use when a feature or system is functionally correct but too slow, memory-heavy, or resource-intensive. Keywords: optimize, performance, profiling, bottleneck, latency, throughput, memory, benchmark."
argument-hint: "<component or area to optimize>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(tk:*), Bash(git:*), Bash(npm:*), Bash(npx:*), Write, Edit
context: fork
---

# Optimize: Measurement-Driven Performance Improvement

You are running the **Optimize** workflow. The goal is to make **$ARGUMENTS** faster, lighter, or more efficient — without breaking correctness. Every decision is grounded in measured data, not intuition.

## When to Use

- A feature or system is functionally correct but fails performance targets (latency, throughput, memory, CPU)
- Profiling results exist and need to be acted on
- A regression was introduced and the cause is unknown
- You want to establish a baseline before a risky refactor
- You want to iterate toward a specific measurable target (e.g., "p99 latency under 200ms")

## When NOT to Use

- The feature is not yet correct — fix correctness first (use `/tracer`)
- There is no measurable target — optimization without a goal is speculative and risky
- The "performance problem" is unconfirmed by measurement — profile first before changing code

## Overview

Optimize works in 5 phases. Each phase produces a committed checkpoint before the next begins.

```
Baseline (measure current state)
  -> Profile (locate hot paths)
    -> Identify bottleneck (single highest-leverage target)
      -> Implement fix (one change at a time)
        -> Measure delta (confirm improvement, no regression)
          -> Iterate (repeat until target reached or returns diminish)
```

---

## Phase 1: Establish Baseline

### 1a. Define the Target

If `$ARGUMENTS` is vague or no performance target was stated, ask one clarifying question:
- What is the measurable target? (e.g., "p99 latency < 200ms", "memory under 50MB", "throughput > 1000 req/s")
- What is the current observed behavior that prompted this?

Do not proceed without a concrete, measurable target. Optimization without a target is refactoring in disguise.

### 1b. Run the Baseline Measurement

Choose the measurement method appropriate for the project:

- **Benchmarks**: Run the existing benchmark suite (`npm run bench`, `cargo bench`, `pytest --benchmark`, etc.)
- **Load test**: If no bench suite exists, write or invoke a representative load script
- **Profiling snapshot**: If the target is memory or CPU, take a profiling snapshot of the current state
- **Manual timing**: For scripted or CLI tools, measure wall-clock time with `time` or equivalent

Record the baseline numbers precisely:

```markdown
## Baseline: [component/area]

**Date**: [date]
**Target**: [measurable target]
**Current state**:
- p50 latency: Xms
- p99 latency: Xms
- Throughput: X req/s
- Memory: XMB RSS
- [other metrics relevant to target]

**Measurement method**: [benchmark/load-test/profiler/time]
**Commit**: [git hash of measured commit]
```

### 1c. Commit the Baseline

```bash
git add [benchmark or measurement files]
git commit -m "$(cat <<'EOF'
optimize: establish baseline for [component]

Measured current performance before any changes.
Target: [measurable target]
Baseline: [key metric and value]

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Phase 2: Profile

### 2a. Choose a Profiler

Select the appropriate profiler for the language and environment:

| Language | CPU profiling | Memory profiling |
|----------|--------------|-----------------|
| Node.js | `--cpu-prof`, `clinic flame` | `--heap-prof`, `clinic heap` |
| Python | `cProfile`, `py-spy`, `memray` | `tracemalloc`, `memray` |
| Rust | `perf`, `cargo flamegraph` | `heaptrack`, `valgrind --massif` |
| Go | `pprof` (cpu + mem) | `pprof` |
| JVM | async-profiler, JFR | JVM heap dump + MAT |
| C/C++ | `perf`, `gprof`, `Instruments` | `valgrind --massif`, `Instruments` |

If none of the above apply or no profiler is available, proceed to manual code analysis (Phase 3) using Grep and Read.

### 2b. Run Under Load

Run the profiler against a representative workload — the same workload used for the baseline if possible. Avoid synthetic microbenchmarks unless the target is a specific function in isolation.

### 2c. Produce a Flame Graph or Hot Path Report

Extract the top consumers from the profiler output:

```markdown
## Profile Results: [component]

**Profiler**: [name and version]
**Workload**: [what was profiled]

**Top CPU consumers:**
1. `[function/module]` — X% of samples — [file:line]
2. `[function/module]` — X% of samples — [file:line]
3. `[function/module]` — X% of samples — [file:line]

**Top memory allocators (if applicable):**
1. `[allocation site]` — XMB — [file:line]
2. `[allocation site]` — XMB — [file:line]

**Observations:**
- [Any non-obvious finding: e.g., "98% of CPU is in JSON parsing, not the DB query"]
```

---

## Phase 3: Identify Bottleneck

### 3a. Select One Target

From the profile results, select **one** bottleneck to address in this iteration. Criteria:

1. **Highest leverage**: Addresses the largest share of measured cost
2. **Feasibility**: Can be changed without breaking the interface contract
3. **Reversibility**: The change can be reverted cleanly if it causes regressions

State the bottleneck explicitly:

```markdown
## Bottleneck: [iteration N]

**Target**: [function/module/pattern]
**Rationale**: [why this is the highest-leverage target — cite profiler numbers]
**Hypothesis**: [what change is expected to help and why]
**Risk**: [what could break or regress]
**Not targeting**: [other hot paths deferred and why]
```

### 3b. Verify the Root Cause

Before implementing, read the hot path code to confirm the hypothesis:
- Use Read and Grep to find the implementation
- Check if there are existing tests covering this path
- Identify callers that will be affected by a change

Do not proceed with a fix until the root cause is confirmed in code — not just in profiler output.

---

## Phase 4: Implement Fix

### 4a. One Change at a Time

Implement **one targeted change** per iteration. Do not bundle multiple optimizations — each change must be independently measurable. Common patterns:

- **Caching**: Memoize or cache expensive repeated computations
- **Batching**: Replace N individual operations with one batched operation
- **Lazy evaluation**: Defer expensive work until it is actually needed
- **Algorithm substitution**: Replace O(n²) with O(n log n) or better
- **Allocation reduction**: Reuse buffers, avoid intermediate copies
- **Parallelism**: Move independent work off the critical path
- **Data structure substitution**: Swap a list for a map, a map for a set, etc.
- **I/O reduction**: Reduce round trips, increase payload size, use streaming

### 4b. Preserve Correctness

After implementing, run the existing test suite. If tests fail, fix the implementation — not the tests. The optimization is invalid if it changes observable behavior.

```bash
[project test command]
```

### 4c. Commit the Fix

```bash
git add [modified files]
git commit -m "$(cat <<'EOF'
optimize: [brief description of change] in [component]

Bottleneck: [what was identified in Phase 3]
Fix: [what was changed and why it should help]
Hypothesis: [expected improvement]

Tests: passing
No behavior change.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Phase 5: Measure Delta

### 5a. Re-Run the Benchmark

Run the same measurement method used in Phase 1 against the same workload. Record results in the same format:

```markdown
## Delta: Iteration N

**Commit**: [git hash of fix]
**Fix applied**: [description]

**Before**: [key metric and value from baseline or prior iteration]
**After**: [key metric and value]
**Delta**: [improvement as absolute value and percentage]

**Regression check**: [any other metric that got worse]
**Tests**: [passing / failing — if failing, revert the fix]
```

### 5b. Evaluate

- **Improvement confirmed and target reached** → proceed to Phase 6 (Report)
- **Improvement confirmed but target not yet reached** → return to Phase 3 (next bottleneck)
- **No measurable improvement** → revert the fix, return to Phase 3 with a different hypothesis
- **Regression** → revert the fix immediately, record what broke, return to Phase 3

```bash
# If reverting:
git revert HEAD --no-edit
```

---

## Phase 6: Report

### 6a. Optimization Summary

Produce a final report covering all iterations:

```markdown
## Optimization Report: [component/area]

**Target**: [measurable target stated in Phase 1]
**Outcome**: [target reached / partially reached / not reached — explain]

### Iterations

| # | Bottleneck | Fix | Before | After | Delta |
|---|-----------|-----|--------|-------|-------|
| 1 | [target] | [change] | [metric] | [metric] | [%] |
| 2 | [target] | [change] | [metric] | [metric] | [%] |
| N | ... | ... | ... | ... | ... |

### Final State vs Baseline

| Metric | Baseline | Final | Change |
|--------|---------|-------|--------|
| [metric 1] | X | Y | +/-Z% |
| [metric 2] | X | Y | +/-Z% |

### What Was Not Addressed

- [Deferred bottleneck]: [why it was not worth addressing]
- [Deferred concern]: [why it was out of scope]

### Known Risks

- [Risk introduced by the optimization]
- [Workload assumptions that may not hold in production]
```

### 6b. Create Follow-Up Tasks

If follow-up work was identified:

**If `.tacks/` or `.beads/` exists in the project root**, create tasks:

```bash
# tacks
tk create "[follow-up task]"
# bd equivalent: bd create --title="[follow-up task]" --type=task --priority=[0-4] \
#   --description="Follow-up from optimize: [component]. [Details and context.]"
```

**If neither `.tacks/` nor `.beads/` exists**, write follow-up tasks to `TODO.md` in the project root.

---

## Guidelines

1. **Measure before changing.** Every optimization decision must be grounded in profiler data or benchmark numbers, not intuition.
2. **One change per iteration.** Bundling changes makes it impossible to know what helped or hurt.
3. **Preserve correctness.** Run tests after every fix. Revert immediately on failure.
4. **Revert freely.** If a fix shows no improvement, revert it. Dead optimizations add complexity with no benefit.
5. **Know when to stop.** If the target is reached, stop. If iterations show diminishing returns (< 5% improvement per iteration), report and stop — further work is unlikely to yield results.
6. **Don't optimize what you haven't profiled.** Hot path intuition is frequently wrong. The profiler is the authority.
7. **Document the baseline.** Future sessions need the baseline commit hash and numbers to know whether a regression was introduced.
8. **Commit each phase.** Phase 1 (baseline), Phase 4 (each fix), and Phase 5 reverts should all be committed. This gives git bisect a clean signal.

## See Also

- `/tracer` — iterative correctness-first implementation (use before /optimize if the feature is incomplete)
- `/test-strategy` — ensure tests exist before optimizing (correctness is a precondition)
- `/review` — code review of optimization changes (useful after a complex fix)
- `/spec` — formalize performance requirements before starting if the target is unclear
