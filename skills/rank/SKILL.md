---
name: rank
description: "Score and order items by user-specified criteria. Reads items from prior primitive output in context. Keywords: sort, score, prioritize, order, compare, evaluate, weight."
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

Search conversation context for structured output from a prior primitive (the `## ... / **Source**: /...` pattern). If found, use those items as input. If no prior primitive output exists, treat $ARGUMENTS as both criteria and item source — extract items directly.

### 2. Parse Criteria

Extract scoring dimensions from $ARGUMENTS (e.g., "by complexity" → one dimension; "by risk, DX" → two dimensions). Use equal weighting unless user specifies otherwise.

### 3. Score and Order

Assign numeric scores (1-5 scale) for each criterion per item. Calculate overall rank (average or weighted sum). Re-order items from highest to lowest score.

### 4. Emit Ranked Output

Output in pipe format:

- **Header**: `## [Ranked ...]`
- **Metadata**: `**Source**: /rank`, `**Input**: [one-line criteria]`
- **Items**: Numbered list (re-ordered by score), each with title, detail, and source from original
- **Criteria**: Table showing item scores per criterion (inserted between Items and Summary per pipe format rules)
- **Summary**: One paragraph explaining the ranking rationale and top/bottom items

## Guidelines

- Highest-ranked items appear first in the numbered list
- Criteria table format: rows=items, columns=criteria+rank, values=numeric scores
- If composing with a prior primitive, preserve original source attribution and confidence levels
- If items lack detail for scoring, note assumptions in the summary
