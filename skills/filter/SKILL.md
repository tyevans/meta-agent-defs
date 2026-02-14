---
name: filter
description: "Apply a criterion to items and keep or drop them. The grep of knowledge work — binary keep/drop decision per item. Different from rank (which reorders but keeps all). Keywords: filter, sift, keep, drop, remove, prune, exclude, include, grep."
argument-hint: "<criterion: security-related | confirmed only | not stale>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob
---

# Filter: Apply Binary Criterion

You are running the **filter** primitive — applying a criterion to items and keeping or dropping them. Criterion: **$ARGUMENTS**

## When to Use

- After gather, distill, or rank to remove irrelevant items
- When the user asks to "filter", "keep only", "remove", or "exclude" items
- To apply binary inclusion/exclusion logic (different from rank which reorders but keeps all)
- During consolidate or fractal workflows to prune duplicates, stale entries, or low-confidence findings

## Process

1. **Find Items**: Search conversation context for prior primitive output (the `## ... / **Source**: /...` pattern). If found, use those items and read the `**Pipeline**` field to construct provenance. Otherwise extract items from conversation context or $ARGUMENTS.

2. **Parse Criterion**: Extract the filter rule from $ARGUMENTS. Support both positive ("security-related", "confirmed only") and negative ("not stale", "not duplicate") filters.

3. **Apply Filter**: Evaluate each item against the criterion. Binary decision: KEEP or DROP. Preserve all attributes (source, confidence, detail) for kept items.

4. **Emit Filtered Output**: Output in pipe format with header, metadata (including `**Pipeline**` — append this step to the upstream pipeline chain, or `(none — working from direct input)` if no upstream), kept items as numbered list, a Dropped section noting what was removed and why, and summary.

## Guidelines

- Kept items appear in the Items section with original numbering reset (1, 2, 3...)
- Dropped items are summarized in a Dropped section (between Items and Summary) with count and reason
- Support negation: "not X" means keep items that DO NOT match X
- Preserve source attribution and confidence levels from input
- If criterion is ambiguous, interpret conservatively (keep items unless clearly excluded)
- If no items remain after filtering, emit empty Items section and explain in summary
