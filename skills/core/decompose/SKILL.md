---
name: decompose
description: "Break a goal or topic into bounded sub-parts with clear scope boundaries. The split primitive — takes one big thing, emits several smaller things. Keywords: break down, split, subdivide, decompose, plan, areas, parts."
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

1. **Identify the whole**: Parse $ARGUMENTS or read prior primitive output from context (detected via `## ... / **Source**: /...` pattern). If upstream found, read its `**Pipeline**` field to construct provenance.
2. **Search for structure**: Use Grep/Glob/Read to understand the codebase's actual structure relevant to the topic. Ground decomposition in reality, not speculation.
3. **Split into sub-parts**: Identify 3-6 bounded sub-parts that are MECE (mutually exclusive, collectively exhaustive).

## Output Format

Output in pipe format:

- **Header**: `## Decomposition of [topic]`
- **Metadata**: `**Source**: /decompose`, `**Input**: [one-line topic]`, `**Pipeline**: [upstream chain -> /decompose (N items)]` or `(none — working from direct input)`
- **Items**: Numbered list where each item includes:
  - **Title** — what this sub-part covers
  - **Scope**: what's included
  - **Boundary**: what's explicitly excluded
- **Summary**: One paragraph explaining the decomposition rationale

## Guidelines

- Target 3-6 sub-parts. Fewer means the goal wasn't complex enough; more means sub-parts are too granular
- Each sub-part must be independently investigable — no sub-part should require another to make sense
- Boundaries matter more than scope — ambiguity lives at the edges
- Ground in codebase structure when decomposing code-related goals
- If composing with prior primitive output, decompose the items or topic from that output
