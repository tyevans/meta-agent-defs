---
name: diff-ideas
description: "Compare two approaches side-by-side with tradeoff analysis across dimensions. Produces a comparison table with per-dimension winner and overall recommendation. Keywords: compare, versus, vs, tradeoff, decision, choose, pros cons."
argument-hint: "<approach A> vs <approach B>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, WebSearch, WebFetch
---

# Diff Ideas: Side-by-Side Comparison

You are running the **diff-ideas** primitive — comparing two approaches side-by-side with tradeoff analysis. Comparison: **$ARGUMENTS**

## When to Use

- When deciding between two alternatives (e.g., "zustand vs jotai", "REST vs GraphQL")
- After ranking — compare the top 2 items from prior /rank output
- When the user asks to "compare", "diff", or choose between options
- Before committing to a technical decision that has clear alternatives

## Process

Identify the two approaches from $ARGUMENTS or from the top 2 items in prior pipe-format output in context. Research each approach using codebase (Grep, Glob, Read) first, then web (WebSearch, WebFetch) if needed. Emit structured comparison in pipe format with a comparison table between Items and Summary.

## Output Format

- **Header**: `## Comparison: [A] vs [B]`
- **Metadata**: `**Source**: /diff-ideas`, `**Input**: [one-line comparison request]`
- **Items**: Numbered list (1 per approach) with key characteristics
- **Comparison**: Table with dimensions as rows, approaches as columns, winner per dimension
- **Summary**: One paragraph recommendation with reasoning

## Guidelines

- Choose 4-6 comparison dimensions relevant to the decision (performance, DX, ecosystem, complexity, etc.)
- Mark the winner per dimension in the table — use "✓" or "stronger" indicators
- Recommendation in summary must reference specific dimension tradeoffs
- If one approach dominates all dimensions, say so clearly
- Code sources are more reliable than web sources — prioritize codebase evidence
