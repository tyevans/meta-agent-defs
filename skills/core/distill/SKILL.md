---
name: distill
description: "Use when prior output is too verbose and needs condensing. Reduces to N bullets, 1 paragraph, or by topic. Keywords: summarize, condense, reduce, extract, essentials, TLDR."
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

1. **Detect Input Source**: Check for upstream pipe-format output in context. If none found, treat conversation context as raw input.

2. **Parse Target**: Extract reduction target from $ARGUMENTS ("to N bullets", "to 1 paragraph", or a topic to filter by). Default to "5 bullets" if unspecified.

3. **Distill**: Reduce to essential points. Prioritize CONFIRMED over LIKELY over POSSIBLE.

**Gate** (topic-filter mode): If $ARGUMENTS specifies a topic, state how many items matched the topic filter before compressing. If zero items matched, stop and report "no items matched topic '[topic]'" rather than emitting an empty output.

**Gate** (compress mode): Output item count is at most the requested N. If the upstream input was already at or below N items, pass all through rather than artificially compressing.

4. **Emit Output**: Output in pipe format.

## Modes

This primitive operates in two modes depending on `$ARGUMENTS`:

| Mode | Trigger | Behavior |
|------|---------|----------|
| **Compress** | `$ARGUMENTS` is a count or format (e.g., "to 5 bullets", "to 1 paragraph") | Reduces all items, preserving scope. Pure distill — single operation. |
| **Topic-filter + compress** | `$ARGUMENTS` is a topic (e.g., "auth", "security") | First filters items by topic relevance, then compresses the survivors. Embeds /filter behavior before compressing. |

In topic mode, distill silently performs a filter pass. If you are composing a pipeline and have already called `/filter`, calling `/distill <topic>` will double-filter. Use `/distill to N bullets` instead to avoid redundant filtering.

## Guidelines

- When distilling raw conversation context, omit confidence levels (not applicable)
- "N bullets" means N numbered items; "1 paragraph" means Summary section only (no Items)
- Filter by topic when $ARGUMENTS specifies one (e.g., "auth" extracts only auth-related points)
- Keep items concise — detail belongs in the source material, not the distillation
