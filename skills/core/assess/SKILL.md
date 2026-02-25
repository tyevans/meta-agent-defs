---
name: assess
description: "Evaluate items against a rubric with categorical verdicts. Like rank but outputs categories (critical/warning/suggestion) instead of scores. Keywords: evaluate, judge, categorize, triage, verdict, rubric, severity, classify."
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

Search conversation context for structured output from a prior primitive (the `## ... / **Source**: /...` pattern). If found, use those items as input and read the `**Pipeline**` field to construct provenance. If no prior primitive output exists, treat $ARGUMENTS as both rubric and item source — extract items directly.

### 2. Parse Rubric

Extract categories from $ARGUMENTS (e.g., "by severity: critical/warning/suggestion", "by goal-fit: answer/deepen/prune"). If no categories are specified, default to HIGH / MEDIUM / LOW priority.

### 3. Evaluate and Categorize

Assess each item against the rubric. Assign a categorical verdict with reasoning. Group items by category.

### 4. Emit Categorized Output

Output in pipe format:

- **Header**: `## [Assessed ...]`
- **Metadata**: `**Source**: /assess`, `**Input**: [one-line rubric]`, `**Pipeline**: [upstream chain -> /assess (N items)]` or `(none — working from direct input)`
- **Items**: Numbered list grouped by category (highest severity first), each with verdict in bold (e.g., **CRITICAL** — detail)
- **Rubric**: Table showing categories and their definitions (inserted between Items and Summary per pipe format rules)
- **Summary**: One paragraph explaining the evaluation rationale and distribution across categories

## Guidelines

- Group items by category — all CRITICAL items together, then WARNING, etc.
- Each item's detail line starts with the verdict in bold (e.g., **CRITICAL** — detail)
- Preserve original source attribution and confidence levels if composing with a prior primitive
- If items lack detail for evaluation, note assumptions in the summary
