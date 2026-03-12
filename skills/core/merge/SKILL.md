---
name: merge
description: "Use when multiple agents or skills produced separate findings that need combining into one list. Deduplicates and upgrades confidence when sources agree. Keywords: merge, combine, unify, join, consolidate, union."
argument-hint: "[optional: topic filter]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, TaskList
---

# Merge: Combine Multiple Pipe-Format Outputs

You are running the **merge** primitive — combining multiple pipe-format blocks from conversation context into one unified output. Filter: **$ARGUMENTS**

## When to Use

- After running /gather multiple times on different topics
- After running parallel primitives and wanting to combine results
- When you have scattered findings across conversation and want one unified view
- During consolidate or fractal workflows to merge parallel investigation results

## Process

### 0. Detect Input Sources (default: conversation context)

Scan conversation context for ALL pipe-format blocks (not just the most recent). Note the source skill for each block.

**In team context** (`.claude/team.yaml` exists or TaskList returns results): additionally collect pipe-format blocks from two supplementary sources before proceeding:

1. **TaskList completed outputs** — call TaskList to retrieve completed background agent results. For each completed task, extract any pipe-format blocks embedded in the task output. Label each block's source as `task:<task-id>` to distinguish it from conversation-context blocks.
2. **SendMessage history** — scan SendMessage history for agent messages containing pipe-format blocks. Agents that communicated results via messages (rather than TaskList output) may have additional findings. Label each block's source as `message:<agent-id>`.

After collecting from all supplementary sources, merge them with the conversation-context blocks into a **unified input set** before proceeding to Step 1. If no pipe-format blocks are found in any source, emit an error and stop.

### 1. Detect All Pipe-Format Blocks

Use the unified input set assembled in Step 0. If running in team context, this set already combines conversation, task, and message blocks — no additional scanning needed.

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

### 7. Emit Unified Output

Output in pipe format with `### Merge Details` section between Items and Summary. Build `**Pipeline**` by combining upstream chains with `+` notation (see pipe-format rule).

## Modes

This primitive operates in two modes depending on `$ARGUMENTS`:

| Mode | Trigger | Behavior |
|------|---------|----------|
| **Merge all** | `$ARGUMENTS` is empty | Deduplicates and unifies all pipe-format blocks in context. Pure merge — single operation. |
| **Merge + filter** | `$ARGUMENTS` specifies a topic (e.g., "auth") | Deduplicates, then filters to items matching the topic. Embeds /filter behavior after merging (Step 4). |

In topic mode, merge silently performs a filter pass. If you are composing a pipeline and plan to call `/filter` downstream, omit the topic argument from merge to avoid double-filtering.

## Guidelines

- If no pipe-format blocks are found in context, emit an error message and stop
- When combining sources, use format: `source: file:line, file:line, URL` (comma-separated)
- Confidence upgrade logic: CONFIRMED if 2+ sources confirm, LIKELY if sources mixed between LIKELY and POSSIBLE, POSSIBLE if all sources POSSIBLE
- Preserve original item detail when deduplicating — merge into the richer description
- If $ARGUMENTS is empty, merge ALL items from ALL blocks (no filtering)
- If merging results in zero items (all filtered out), emit empty Items section and explain in summary
