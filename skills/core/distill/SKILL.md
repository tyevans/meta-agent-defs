---
name: distill
description: "Reduce verbose input to essential points. Configurable target: bullets, paragraph, or count. Reads prior primitive output from context. Keywords: summarize, condense, reduce, extract, essentials, TLDR."
argument-hint: "[to N bullets | to 1 paragraph | topic]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob
---

# Distill: Reduce to Essentials

You are running the **distill** primitive — reducing verbose input to essential points. Target: **$ARGUMENTS**

## When to Use

- After /gather or another verbose primitive to condense findings
- When composing primitives to reduce information before ranking or verification
- When the user asks to "summarize", "condense", or get a "TLDR"
- To extract key points from conversation context when no prior primitive output exists

## Process

1. **Detect Input Source**: Check conversation context for prior primitive output (the `## ... / **Source**: /...` pattern). If found, use that as input and read its `**Pipeline**` field to construct provenance. Otherwise treat conversation context as raw input.

2. **Parse Target**: Extract reduction target from $ARGUMENTS ("to N bullets", "to 1 paragraph", or a topic to filter by). Default to "5 bullets" if unspecified.

3. **Distill**: Reduce to essential points while preserving source attribution and confidence levels (if present in input). Prioritize CONFIRMED over LIKELY over POSSIBLE.

4. **Emit Output**: Structured in pipe format with header, metadata (including `**Pipeline**` — append this step to the upstream pipeline chain, or `(none — working from direct input)` if no upstream), numbered items (even for single items), and one-paragraph summary.

## Modes

This primitive operates in two modes depending on `$ARGUMENTS`:

| Mode | Trigger | Behavior |
|------|---------|----------|
| **Compress** | `$ARGUMENTS` is a count or format (e.g., "to 5 bullets", "to 1 paragraph") | Reduces all items, preserving scope. Pure distill — single operation. |
| **Topic-filter + compress** | `$ARGUMENTS` is a topic (e.g., "auth", "security") | First filters items by topic relevance, then compresses the survivors. Embeds /filter behavior before compressing. |

In topic mode, distill silently performs a filter pass. If you are composing a pipeline and have already called `/filter`, calling `/distill <topic>` will double-filter. Use `/distill to N bullets` instead to avoid redundant filtering.

## Guidelines

- Preserve source attribution and confidence from input when distilling primitive output
- When distilling raw conversation context, omit confidence levels (not applicable)
- "N bullets" means N numbered items; "1 paragraph" means Summary section only (no Items)
- Filter by topic when $ARGUMENTS specifies one (e.g., "auth" extracts only auth-related points)
- Keep items concise — detail belongs in the source material, not the distillation
