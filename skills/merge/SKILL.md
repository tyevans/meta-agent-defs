---
name: merge
description: "Combine multiple pipe-format blocks from context into one unified output. Handles dedup, source attribution, and confidence upgrade. Keywords: merge, combine, unify, join, consolidate, union."
argument-hint: "[optional: topic filter]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob
---

# Merge: Combine Multiple Pipe-Format Outputs

You are running the **merge** primitive — combining multiple pipe-format blocks from conversation context into one unified output. Filter: **$ARGUMENTS**

## When to Use

- After running /gather multiple times on different topics
- After running parallel primitives and wanting to combine results
- When you have scattered findings across conversation and want one unified view
- During consolidate or fractal workflows to merge parallel investigation results

## Process

### 1. Detect All Pipe-Format Blocks

Scan conversation context for ALL blocks matching the pipe-format pattern (the `**Source**: /...` marker). Collect every one, not just the most recent. Note the source skill and `**Pipeline**` field for each block.

### 2. Parse Items

Extract items from each block into a unified working list. Preserve all attributes: title, detail, source, confidence (if present).

### 3. Deduplicate

- Items with the **same title** are duplicates
- Items with **substantially the same content** (>80% overlap in detail) are duplicates
- When merging duplicates:
  - Keep the item with richer detail
  - Combine source attributions (preserve all sources)
  - Upgrade confidence: if 2+ sources confirm the same finding → CONFIRMED, if mixed → highest confidence level present

### 4. Filter (Optional)

If $ARGUMENTS specifies a topic filter, keep only items matching that topic. If no filter provided, keep all deduplicated items.

### 5. Renumber

Reset item numbering to sequential (1, 2, 3...) for the final output.

### 6. Add Merge Summary Section

Between Items and Summary, add a **Merge Details** section showing:
- Number of input blocks merged (list source skills)
- Item count before dedup
- Item count after dedup
- Number of confidence upgrades applied
- Topic filter used (if any)

### 7. Construct Pipeline Provenance

Build the `**Pipeline**` field by combining the pipeline chains from all input blocks. Use `+` to show merged branches (e.g., `/gather (8 items) + /gather (6 items) -> /merge (10 items)`).

### 8. Emit Unified Output

Output in pipe format with header, metadata (including `**Pipeline**`), deduplicated items as numbered list, Merge Details section, and final summary.

## Guidelines

- If no pipe-format blocks are found in context, emit an error message and stop
- When combining sources, use format: `source: file:line, file:line, URL` (comma-separated)
- Confidence upgrade logic: CONFIRMED if 2+ sources confirm, LIKELY if sources mixed between LIKELY and POSSIBLE, POSSIBLE if all sources POSSIBLE
- Preserve original item detail when deduplicating — merge into the richer description
- If $ARGUMENTS is empty, merge ALL items from ALL blocks (no filtering)
- If merging results in zero items (all filtered out), emit empty Items section and explain in summary
