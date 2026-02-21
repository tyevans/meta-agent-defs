---
name: evolution
description: "Track how any definition changed over time — edit history, churn, stability, fix-after-feat patterns. Use when reviewing a skill/agent/rule for maintenance or understanding why it looks the way it does. Keywords: history, evolution, changes, timeline, churn, lifecycle."
argument-hint: "<file-path> [commit1..commit2]"
disable-model-invocation: false
user-invocable: true
allowed-tools: [Read, Grep, Glob, "Bash(git:*)"]
---

# Evolution: File Change History

You are running **evolution** — tracking how a definition file changed over time. Target: **$ARGUMENTS**

## When to Use

- Understanding why a skill/agent/rule looks the way it does
- Assessing stability before making changes
- Investigating quality issues (rushed feat→fix cycles)
- Planning refactoring (high churn = design instability)

## How It Works

```
Parse args → Gather history → Analyze patterns → Report timeline + stability
```

---

## Phase 0: Parse Arguments

Extract file path and optional commit range from `$ARGUMENTS`. Formats: `<file-path>`, `<file-path> commit1..commit2`, or `<file-path> commit1 commit2`.

Verify file exists in working tree or git history. If not found, error: "File not found in current tree or git history."

---

## Phase 1: Gather History

```bash
# Timeline
git log --follow --format="%H|%ai|%s" -- <file-path>
# Line deltas
git log --follow --stat --format="%H" -- <file-path>
```

Extract for each commit: hash, date, type (from conventional prefix), message, line delta (+/-).

Compute:
- Creation: first commit (date, hash, message)
- Last modified: most recent commit (date, days since, hash)
- Total churn: sum of all line additions + removals

---

## Phase 2: Analyze Patterns

Detect fix-after-feat: consecutive `feat` → `fix` commits within 7 days (rushed changes).

Classify stability by days since last edit:
- **stable**: >90 days
- **active**: 15-90 days
- **volatile**: <15 days

Compute edit frequency: (last date - creation date) / (edit count - 1). Report "single edit" if only one.

Trend (last 5 edits): net line change. **growing** (>+50), **shrinking** (<-50), or **stable** (-50 to +50).

---

## Phase 3: Report

Format output:

```markdown
## Evolution: <file-path>

**Created**: <date> by <short-hash> (<message>)
**Last modified**: <date> (<N days ago>)
**Stability**: <stable|active|volatile> (<N days since last edit>)
**Total churn**: <added> added, <removed> removed across <N> edits

### Timeline
| Date | Commit | Type | Message | Delta |
|------|--------|------|---------|-------|
[Most recent first, all commits]

### Patterns
- **Fix-after-feat**: [N instances | none detected] [list pairs if any]
- **Edit frequency**: [avg days | single edit]
- **Trend (last 5)**: [growing|shrinking|stable] (net +/-N)

### Summary
[1-2 sentences. Examples: "Stable — 120 days, minimal churn." or "Active — 8 edits in 45 days, ongoing refinement."]
```

If commit range provided, append diff (or `--stat` if >500 lines).

---

## Guidelines

1. **Fast.** Git history queries should complete in <5 seconds for typical files. No heavy processing.
2. **Honest.** Report actual data. If a metric cannot be computed (e.g., trend with <2 edits), say so.
3. **Mechanical.** Pattern detection uses simple rules (fix-after-feat = consecutive commits with type match). No LLM speculation.
4. **Read-only.** No file writes, no git operations beyond read-only log/diff.
5. **Graceful degradation.** If commit messages lack conventional prefixes, show "unknown" for type.
6. **Cite sources.** Every claim traces to a specific commit hash. The timeline table is the evidence.
