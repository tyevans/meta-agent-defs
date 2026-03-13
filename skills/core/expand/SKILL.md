---
name: expand
description: "Use when prior findings are too sparse for action. Adds depth, context, and evidence to compressed items. Inverse of distill. Keywords: elaborate, expand, detail, unpack, enrich, flesh out."
argument-hint: "[depth target: to paragraphs | with examples | with evidence]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, WebSearch, WebFetch
---

# Expand: Elaborate Sparse Items

You are running the **expand** primitive — elaborating sparse items into detailed descriptions. Target: **$ARGUMENTS**

## When to Use

- After distill when a compressed finding needs unpacking before implementation
- When pipe-format items are too terse for downstream consumption (e.g., before sketch or spec)
- When the user asks to "elaborate", "expand", "flesh out", or "add detail" to findings
- To enrich items with evidence, examples, or implementation guidance

## Process

1. **Detect Input Source**: Detect upstream pipe-format output in context. If none found, treat $ARGUMENTS as both target and item source.

2. **Parse Depth Target**: Extract elaboration target from $ARGUMENTS. Options:
   - "to paragraphs" — expand each item to a full paragraph (default)
   - "with examples" — add concrete examples to each item
   - "with evidence" — add source references and supporting data to each item
   - A topic — expand only items related to that topic, pass others through unchanged

3. **Elaborate Each Item**: For each item independently:
   - Preserve the original title and source attribution
   - Add depth: context, implications, concrete details, or examples depending on target
   - Use Grep/Read to ground elaborations in actual code/docs when possible
   - Use WebSearch/WebFetch for external context when codebase evidence is insufficient

**Gate**: Every input item appears in the output — none skipped or merged. Item count in equals item count out. If an item was passed through unchanged (already detailed), note it explicitly rather than silently omitting it.

**Gate**: Claims added during elaboration are grounded in at least one cited source (file path or URL). If no evidence was found for a claim, mark it POSSIBLE rather than presenting it as fact.

4. **Emit Output**: Output in pipe format.

## Guidelines

- Preserve original item numbering and titles — expand the detail, don't replace it
- Each item expands independently — do not merge or reorder items
- Ground elaborations in evidence (cite files, URLs) rather than generating unsourced prose
- New claims added during expansion get their own confidence level
- If an item is already detailed enough, pass it through unchanged rather than padding it
- Summary should note what depth was added and which items changed most
