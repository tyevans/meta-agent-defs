---
name: transform
description: "Use when items need reformatting without changing the collection (e.g., as ticket titles, test cases, or for a different audience). Rewrites each item independently. Keywords: rewrite, rephrase, convert, map, translate, format, each."
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

1. **Detect Input Source**: Detect upstream pipe-format output in context. If none found, treat $ARGUMENTS as both instruction and item source — extract items directly.

2. **Parse Instruction**: Extract the rewrite rule from $ARGUMENTS (e.g., "as one-line ticket titles", "as acceptance criteria", "for a non-technical audience"). The instruction is applied identically to every item.

**Gate**: The rewrite instruction is concrete enough to apply consistently — it specifies a target format, audience, or structure. If $ARGUMENTS is too vague to apply uniformly (e.g., "make it better"), ask one clarifying question before proceeding.

3. **Transform Each Item**: For each item independently:
   - Apply the rewrite instruction to produce the new content
   - Maintain the original item number

**Gate**: Output item count equals input item count. If any item was skipped or merged, that is a transform error — include it in the output with a note rather than silently dropping it.

4. **Emit Output**: Output in pipe format.

## Guidelines

- Item count in equals item count out — transform never adds, removes, or merges items
- Each item is transformed independently — no cross-item reasoning
- If the instruction is ambiguous for a specific item, apply the best-fit interpretation and note it
- Summary should state the transform applied and note any items where the instruction was ambiguous
