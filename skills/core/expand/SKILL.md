---
name: expand
description: "Elaborate sparse items into detailed descriptions. Inverse of distill — takes compressed findings and adds depth, context, and actionable detail. Keywords: elaborate, expand, detail, unpack, enrich, flesh out."
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

1. **Detect Input Source**: Check conversation context for prior primitive output (the `## ... / **Source**: /...` pattern). If found, use those items as input and read its `**Pipeline**` field to construct provenance. Otherwise treat $ARGUMENTS as both target and item source.

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

4. **Emit Output**: Structured in pipe format with header, metadata (including `**Pipeline**` — append this step to the upstream chain), numbered items with expanded detail, and one-paragraph summary.

## Guidelines

- Preserve original item numbering and titles — expand the detail, don't replace it
- Each item expands independently — do not merge or reorder items
- Ground elaborations in evidence (cite files, URLs) rather than generating unsourced prose
- Confidence levels from upstream are preserved; new claims added during expansion get their own confidence
- If an item is already detailed enough, pass it through unchanged rather than padding it
- Summary should note what depth was added and which items changed most
