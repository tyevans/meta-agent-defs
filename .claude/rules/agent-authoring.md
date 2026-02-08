---
paths:
  - "agents/**/*.md"
---

# Agent Authoring Rules

Rules for writing and editing agent definition files.

## Required Frontmatter

Every agent file must have YAML frontmatter with these fields:

```yaml
---
name: lowercase-with-hyphens
description: When to invoke this agent (not just what it does)
tools: Comma, Separated, Tool, Names
model: opus|sonnet|haiku
---
```

Optional: `permissionMode: default|acceptEdits|dontAsk|bypassPermissions|plan`

## Required Sections

Every agent MUST include these three sections, in addition to its core instructions:

1. **Investigation Protocol** -- How the agent verifies findings rather than guessing
2. **Context Management** -- How the agent avoids filling its context window
3. **Knowledge Transfer** -- Read bead notes before starting, report findings after completing

## Do This

- Start the description with a verb phrase describing when to invoke ("Explores a project...", "Reviews code changes...")
- List only the tools the agent actually needs
- Default to `sonnet` model unless the task requires deep reasoning (opus) or is trivial (haiku)
- Include project-specific file paths, commands, and conventions

## Don't Do This

- Do not give agents Write/Edit tools if they are read-only (reviewers, analyzers)
- Do not use `permissionMode: bypassPermissions` without explicit justification
- Do not write generic instructions -- every agent should reference THIS project's patterns
