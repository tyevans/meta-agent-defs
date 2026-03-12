---
name: rank
description: "Use when you have items and need to know which matter most. Scores and reorders by criteria you specify. Keeps all items (use filter to drop). Keywords: sort, score, prioritize, order, compare, evaluate, weight."
argument-hint: "<criteria: by complexity, by risk, by effort>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob
---

# Rank: Score and Order Items

You are running the **rank** primitive — scoring and ordering items by user-specified criteria. Criteria: **$ARGUMENTS**

## When to Use

- After gather or distill to prioritize findings by importance, risk, complexity, or effort
- When the user asks to "sort", "score", "prioritize", or "order" items
- To weight items by multiple criteria (e.g., "by security risk and DX impact")

## Process

### 1. Find Items

Detect upstream pipe-format output in context. If none found, treat $ARGUMENTS as both criteria and item source — extract items directly.

### 2. Parse Criteria

Extract scoring dimensions from $ARGUMENTS (e.g., "by complexity" → one dimension; "by risk, DX" → two dimensions). Use equal weighting unless user specifies otherwise.

### 3. Score and Order

Assign numeric scores (1-5 scale) for each criterion per item. Calculate overall rank (average or weighted sum). Re-order items from highest to lowest score.

**Gate**: Every item has a numeric score for every criterion — no blanks in the criteria table. If an item lacked sufficient detail to score a criterion, assign the median score (3) and note the assumption in the summary rather than leaving the cell empty.

**Gate**: No two items share the same overall rank. If a tie exists, break it using the most decision-relevant criterion from $ARGUMENTS; note the tiebreak in the summary.

### 4. Emit Ranked Output

Output in pipe format with a `### Criteria` section (table: rows=items, columns=criteria+rank) between Items and Summary.

## Guidelines

- Highest-ranked items appear first in the numbered list
- If items lack detail for scoring, note assumptions in the summary
