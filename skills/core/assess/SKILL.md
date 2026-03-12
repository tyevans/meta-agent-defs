---
name: assess
description: "Use when items need categorical triage (critical/warning/ok), not numeric scoring. Groups items by verdict against a rubric. Unlike rank (scores) or filter (binary). Keywords: evaluate, judge, categorize, triage, verdict, rubric, severity, classify."
argument-hint: "<rubric: by severity | by goal-fit | by readiness>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob
---

# Assess: Categorical Evaluation Against Rubric

You are running the **assess** primitive — evaluating items against a rubric with categorical verdicts. Rubric: **$ARGUMENTS**

## When to Use

- After gather or distill to categorize findings by severity, readiness, or goal-fit
- When the user asks to "evaluate", "judge", "categorize", or "triage" items
- To classify items into discrete buckets rather than numeric scores (use rank for numeric scoring)

## Process

### 1. Find Items

Detect upstream pipe-format output in context. If none found, treat $ARGUMENTS as both rubric and item source — extract items directly.

### 2. Parse Rubric

Extract categories from $ARGUMENTS (e.g., "by severity: critical/warning/suggestion", "by goal-fit: answer/deepen/prune"). If no categories are specified, default to HIGH / MEDIUM / LOW priority.

**Gate**: Item count is non-zero and matches the upstream Items (N) count. If zero items were found, stop and report "no items to assess" rather than emitting an empty output.

### 3. Evaluate and Categorize

Assess each item against the rubric. Assign a categorical verdict with reasoning. Group items by category.

**Gate**: Every item has been assigned a category. No item left uncategorized. If any item lacked sufficient detail for categorization, assign the lowest-severity category and note the assumption in the summary.

### 4. Emit Categorized Output

Output in pipe format. Items grouped by category (highest severity first), each prefixed with verdict in bold (e.g., **CRITICAL** — detail). Include a `### Rubric` section (categories and definitions) between Items and Summary.

## Guidelines

- Group items by category — all CRITICAL items together, then WARNING, etc.
- Each item's detail line starts with the verdict in bold (e.g., **CRITICAL** — detail)
- If items lack detail for evaluation, note assumptions in the summary
