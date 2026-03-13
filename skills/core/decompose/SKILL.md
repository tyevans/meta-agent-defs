---
name: decompose
description: "Use when a goal is too large to tackle as one unit. Splits into 3-6 bounded sub-parts with clear scope boundaries. Keywords: break down, split, subdivide, decompose, plan, areas, parts."
argument-hint: "<goal or topic to break down>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob
---

# Decompose: Break Into Bounded Sub-Parts

You are running the **decompose** primitive — breaking a goal or topic into bounded sub-parts with clear scope. Target: **$ARGUMENTS**

## When to Use

- When a goal is too large or vague to tackle as a single unit
- Before fan-out workflows (fractal, blossom) to identify investigation areas
- After gathering to split findings into workable chunks
- When the user asks to "break down", "split", or "plan" an approach

## Process

1. **Identify the whole**: Parse $ARGUMENTS or detect upstream pipe-format output in context.
2. **Search for structure**: Use Grep/Glob/Read to understand the codebase's actual structure relevant to the topic. Ground decomposition in reality, not speculation.
3. **Split into sub-parts**: Identify 3-6 bounded sub-parts that are MECE (mutually exclusive, collectively exhaustive).
4. **Assess isolation**: For each sub-part, compare its `scope` (file patterns) against all other sub-parts. If their file sets are disjoint, mark `independent`; if any files overlap, mark `shared-state`. When uncertain, default to `shared-state` (conservative). Downstream skills (/plan, /sprint) use this field to decide whether to parallelize via worktree isolation or serialize.

## Output Format

Output in pipe format. Each item includes:
- **Scope**: what's included
- **Boundary**: what's explicitly excluded
- **Isolation**: `independent` (disjoint file sets, safe for worktree isolation) or `shared-state` (overlapping files, must serialize or merge)

## Guidelines

- Target 3-6 sub-parts. Fewer means the goal wasn't complex enough; more means sub-parts are too granular
- Each sub-part must be independently investigable — no sub-part should require another to make sense
- Boundaries matter more than scope — ambiguity lives at the edges
- Ground in codebase structure when decomposing code-related goals
- For `isolation`: non-overlapping scope file patterns → `independent`; any shared files → `shared-state`; doubt → `shared-state`
