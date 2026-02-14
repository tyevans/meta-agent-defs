---
name: sketch
description: "Produce a minimal code skeleton with structure and TODO placeholders. No implementation — just the bones. Use after gather/rank to prototype the winning approach. Keywords: scaffold, skeleton, prototype, outline, structure, boilerplate, stub."
argument-hint: "<what to sketch>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob
---

# Sketch: Minimal Code Skeleton Generator

You are running the **sketch** primitive — producing a minimal code skeleton with structure and TODO placeholders. Request: **$ARGUMENTS**

## When to Use

- After gather/rank/distill to prototype the winning approach
- When you need structure without implementation (scaffold before coding)
- To visualize file/module organization before committing to it
- When the user asks to "outline", "scaffold", "stub", or "prototype" code

## Process

1. **Check context** for prior primitive output (gather, rank, distill). If found, sketch based on those findings and read the `**Pipeline**` field to construct provenance.
2. **Search codebase** (if needed) using Grep/Glob to understand existing structure/conventions.
3. **Emit skeleton** in pipe format with code blocks containing TODO placeholders. Include `**Pipeline**` in metadata — append this step to the upstream chain, or `(none — working from direct input)` if no upstream.

Output format: numbered list where each item is a file/module with a code block showing structure and TODO comments marking implementation points.

## Guidelines

- Code blocks must have language annotations (```python, ```typescript, etc.)
- Use TODO comments to mark where implementation goes — never add actual logic
- Include imports/exports, function signatures, class outlines — omit bodies
- Follow existing codebase conventions (read similar files if uncertain)
- Keep it minimal — the user will fill in the blanks
- If composing with prior primitives, the items from that primitive guide what to sketch
- Summary should describe the skeleton's structure and design rationale (one paragraph)
