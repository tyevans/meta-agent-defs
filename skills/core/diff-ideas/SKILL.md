---
name: diff-ideas
description: "Use when deciding between two approaches and you need a structured tradeoff comparison. Produces a dimension-by-dimension table with a recommendation. Keywords: compare, versus, vs, tradeoff, decision, choose, pros cons."
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

Identify the two approaches from $ARGUMENTS or from the top 2 items in upstream pipe-format output. Research each approach using codebase (Grep, Glob, Read) first, then web (WebSearch, WebFetch) if needed.

**Gate**: Both approaches are identified and have at least one concrete characteristic each (a behavioral difference, a tradeoff, or a use-case). If only one approach is understood, ask for clarification rather than comparing against a vague alternative.

Score each approach across 4-6 dimensions.

**Gate**: Every dimension has a score or winner for both approaches. No dimension left with "unclear" on both sides. If evidence was insufficient for a dimension, mark the stronger approach as winner with a note rather than leaving it unresolved.

Emit in pipe format with a `### Comparison` section (table: dimensions as rows, approaches as columns, winner per dimension) between Items and Summary.

## Decomposition

Diff-ideas is a convenience macro that bundles three primitive operations:

```
/gather approach-A -> /gather approach-B -> /merge -> /rank by dimensions -> /distill to recommendation
```

The bundling is justified because pairwise comparison with a per-dimension winner table is a specific output structure that none of the individual primitives produce. Decomposing would require 5 primitive invocations and manual coordination of the comparison table format.

If you need finer control (e.g., comparing 3+ approaches, or using custom research beyond Grep/WebSearch), decompose manually. For 3+ approaches, consider `/gather` each approach separately, then `/merge` and `/rank`.

## Guidelines

- Choose 4-6 comparison dimensions relevant to the decision (performance, DX, ecosystem, complexity, etc.)
- Mark the winner per dimension in the table — use "✓" or "stronger" indicators
- Recommendation in summary must reference specific dimension tradeoffs
- If one approach dominates all dimensions, say so clearly
- Code sources are more reliable than web sources — prioritize codebase evidence
