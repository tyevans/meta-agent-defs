---
name: plan
description: "Convert items into a dependency-aware execution sequence showing what must come before what. Outputs ordering by dependency, not score. Keywords: plan, sequence, order, dependencies, execution, schedule, critical path, before, after, blocks."
argument-hint: "<optional constraint: minimize risk, fastest path, 2-person team>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob
context: inline
---

# Plan: Sequence Items by Dependency

You are running the **plan** primitive — sequencing items into a dependency-aware execution order. Constraint: **$ARGUMENTS**

## When to Use

- After decompose or gather to sequence sub-parts into an execution plan
- When the user asks to "plan", "order", "sequence", or "schedule" work
- To identify what must come before what (dependency ordering vs. score-based ranking)
- When you need to visualize critical path and parallel opportunities

## Process

1. **Find Items**: Search conversation context for structured output from a prior primitive (the `## ... / **Source**: /...` pattern). If found, use those items as input and read the `**Pipeline**` field to construct provenance. If no prior primitive output exists, extract items directly from $ARGUMENTS.
2. **Identify Dependencies**: For each item, determine what it depends on. Use Read/Grep/Glob to understand technical dependencies in the codebase when applicable.
3. **Sequence by Dependency**: Order items so dependencies come before dependents. Apply constraint from $ARGUMENTS if present (e.g., "minimize risk" → de-risk blockers first; "fastest path" → shortest critical path).
4. **Emit Sequenced Output**: Output in pipe format with execution-order numbering, dependency metadata, ASCII dependency graph, and parallel groups with isolation hints.

## Output Format

Output in pipe format:

- **Header**: `## Execution Sequence for [topic]`
- **Metadata**: `**Source**: /plan`, `**Input**: [one-line topic + constraint]`, `**Pipeline**: [upstream chain -> /plan (N items)]` or `(none — working from direct input)`
- **Items**: Numbered list in EXECUTION ORDER, each with:
  - **Title** — what this step does
  - **depends-on**: [item numbers or "none"]
  - **rationale**: [why this position in the sequence]
- **Dependencies**: ASCII diagram showing dependency graph (e.g., `1 → 2 → 4`, `1 → 3 → 4`)
- **Parallel Groups**: Execution waves derived from the dependency graph, each listing concurrent items with isolation hints. Format:

  ```
  ### Parallel Groups

  **Wave 1** (no dependencies)
  - Item 1 — `shared` (read-only config changes)
  - Item 3 — `worktree` (modifies shared model layer)

  **Wave 2** (depends on Wave 1)
  - Item 2 — `shared` (isolated module, no shared file writes)
  - Item 4 — `worktree` (touches shared migration files)

  **Wave 3** (depends on Wave 2)
  - Item 5 — `worktree` (integration layer, touches multiple modules)
  ```

  Isolation hint rules:
  - `worktree` — item writes to files that other wave-siblings also read or write, or modifies shared infrastructure (migrations, config, interfaces)
  - `shared` — item is safe to run concurrently in a shared workspace (isolated module, read-only, or no file overlap with siblings)

- **Summary**: One paragraph synthesis including critical path length, parallel opportunities, and constraint application

## Guidelines

- Items are numbered in execution order (earliest-possible first), not score order
- If multiple items have no dependencies, constraint determines their relative order
- Critical path is the longest dependency chain from start to finish
- Parallel opportunities are items with the same depends-on set (can run concurrently); these form a wave in the Parallel Groups section
- Preserve original item source attribution and confidence when composing with prior primitives
- If dependencies are unclear from input, note assumptions in rationale
- Parallel Groups is the bridge to /sprint: wave items with `worktree` hint → dispatch with `isolation: "worktree"`; wave items with `shared` hint → dispatch normally; waves are sequential, items within a wave are parallel
- Isolation hint is `worktree` when an item writes to files that sibling items in the same wave also touch, or when it modifies shared infrastructure (migrations, config, public interfaces); otherwise `shared`
