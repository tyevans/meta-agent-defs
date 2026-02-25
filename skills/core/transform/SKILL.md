---
name: transform
description: "Apply a rewrite instruction to every item independently. The map primitive — transforms each item's content without changing the collection structure. Keywords: rewrite, rephrase, convert, map, translate, format, each."
argument-hint: "<rewrite instruction>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob
---

# Transform: Item-by-Item Rewrite

You are running the **transform** primitive — applying a rewrite instruction to every item independently. Instruction: **$ARGUMENTS**

## When to Use

- After gather/distill to rewrite findings into a different format (e.g., ticket titles, test cases, action items)
- When items need rephrasing for a different audience (technical to non-technical, or vice versa)
- When converting between representations (findings to requirements, bugs to test cases)
- When the user asks to "rewrite each", "convert each", "rephrase", or "format as"

## Process

1. **Detect Input Source**: Check conversation context for prior primitive output (the `## ... / **Source**: /...` pattern). If found, use those items as input and read its `**Pipeline**` field to construct provenance. Otherwise treat $ARGUMENTS as both instruction and item source — extract items directly.

2. **Parse Instruction**: Extract the rewrite rule from $ARGUMENTS (e.g., "as one-line ticket titles", "as acceptance criteria", "for a non-technical audience"). The instruction is applied identically to every item.

3. **Transform Each Item**: For each item independently:
   - Apply the rewrite instruction to produce the new content
   - Preserve source attribution and confidence levels from the original
   - Maintain the original item number

4. **Emit Output**: Structured in pipe format with header, metadata (including `**Pipeline**`), transformed items as numbered list, and one-paragraph summary.

## Guidelines

- Item count in equals item count out — transform never adds, removes, or merges items
- Each item is transformed independently — no cross-item reasoning
- Preserve source and confidence metadata from upstream (append, do not replace)
- If the instruction is ambiguous for a specific item, apply the best-fit interpretation and note it
- Summary should state the transform applied and note any items where the instruction was ambiguous
