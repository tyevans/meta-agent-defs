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

1. **Detect Input Source**: Check conversation context for prior primitive output (the `## ... / **Source**: /...` pattern). If found, use that as input. Otherwise treat conversation context as raw input.

2. **Parse Target**: Extract reduction target from $ARGUMENTS ("to N bullets", "to 1 paragraph", or a topic to filter by). Default to "5 bullets" if unspecified.

3. **Distill**: Reduce to essential points while preserving source attribution and confidence levels (if present in input). Prioritize CONFIRMED over LIKELY over POSSIBLE.

4. **Emit Output**: Structured in pipe format with header, metadata, numbered items (even for single items), and one-paragraph summary.

## Guidelines

- Preserve source attribution and confidence from input when distilling primitive output
- When distilling raw conversation context, omit confidence levels (not applicable)
- "N bullets" means N numbered items; "1 paragraph" means Summary section only (no Items)
- Filter by topic when $ARGUMENTS specifies one (e.g., "auth" extracts only auth-related points)
- Keep items concise — detail belongs in the source material, not the distillation
