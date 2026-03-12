---
name: sketch
description: "Use when you've decided on an approach and need to see the structure before implementing. Produces a skeleton with TODO placeholders — no implementation. Keywords: scaffold, skeleton, prototype, outline, structure, boilerplate, stub."
argument-hint: "<what to sketch>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob
---

# Sketch: Structural Skeleton Generator

You are running the **sketch** primitive — producing a minimal structural skeleton with TODO placeholders. Request: **$ARGUMENTS**

## When to Use

- After gather/rank/distill to prototype the winning approach
- When you need structure without implementation (scaffold before building)
- To visualize file/module/section organization before committing to it
- When the user asks to "outline", "scaffold", "stub", or "prototype" anything

## Artifact Types

Sketch works on any structured artifact:

| Type | Skeleton Contains |
|------|-------------------|
| **Code** | Files, imports, function signatures, class outlines, TODO bodies |
| **Document** | Sections, headings, placeholder paragraphs, key questions |
| **Config** | Keys, structure, placeholder values with comments |
| **Schema** | Fields, types, relationships, TODO constraints |
| **Workflow** | Steps, phases, decision points, TODO details |

Detect the artifact type from $ARGUMENTS or prior primitive context. Default to code if ambiguous and the project has source files.

## Process

1. **Check context** for upstream pipe-format output. If found, sketch based on those findings.

**Gate**: Artifact type is determined (code / document / config / schema / workflow) and stated explicitly before generating the skeleton. If the type is ambiguous and no source files exist to default to, ask the user to clarify rather than guessing.

2. **Search codebase/project** (if needed) using Grep/Glob to understand existing structure/conventions.
3. **Emit skeleton** in pipe format with content blocks containing TODO placeholders.

**Gate**: Every section or component in the skeleton has at least one TODO placeholder. A section with no TODO is either already implemented (should not appear in a sketch) or was silently skipped — flag it and add a placeholder.

Output format: numbered list where each item is a file, section, or component with a content block showing structure and TODO comments marking implementation points.

## Guidelines

- Code blocks must have language annotations (```python, ```typescript, etc.)
- Document skeletons use markdown structure with `<!-- TODO: ... -->` placeholders
- Use TODO comments to mark where implementation goes — never add actual content
- For code: include imports/exports, function signatures, class outlines — omit bodies
- For documents: include section headings, key questions per section, cross-references
- Follow existing project conventions (read similar artifacts if uncertain)
- Keep it minimal — the user will fill in the blanks
- Summary should describe the skeleton's structure and design rationale (one paragraph)
