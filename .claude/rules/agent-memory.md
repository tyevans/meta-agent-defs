---
paths:
  - "agents/**/*.md"
  - "skills/**/*.md"
---

# Agent Memory Protocol

Convention for how agents use the Memory MCP to accumulate and recall knowledge across sessions.

## Memory Namespace

Each agent identity gets a unique entity in the knowledge graph:

- **Entity name**: `agent:<agent-name>` (e.g., `agent:code-reviewer`, `agent:spike-handler`)
- **Entity type**: `agent-memory`
- **Observations**: Accumulated learnings, patterns, preferences, gotchas

## Read-on-Spawn

When an agent starts work, it checks for prior memory:

```
mcp__memory__open_nodes(names: ["agent:<agent-name>"])
```

If the node exists, review the observations before beginning work. Prior learnings should inform current decisions (e.g., "this codebase uses X pattern", "library Y has gotcha Z").

If no node exists, proceed normally -- the agent is running for the first time.

## Write-on-Complete

Before finishing work, the agent writes observations worth remembering:

```
mcp__memory__add_observations(observations: [{
  entityName: "agent:<agent-name>",
  contents: ["<observation>"]
}])
```

If the entity doesn't exist yet, create it first:

```
mcp__memory__create_entities(entities: [{
  name: "agent:<agent-name>",
  entityType: "agent-memory",
  observations: ["<initial observations>"]
}])
```

## What to Remember

- Patterns confirmed across multiple files or sessions
- Codebase-specific conventions (naming, structure, architecture)
- Library gotchas and workarounds that were hard-won
- User preferences for approach, style, or tooling
- Recurring issues and their root causes

## What NOT to Remember

- Session-specific context (current task details, in-progress state)
- Unverified hunches from a single observation
- Information already captured in CLAUDE.md or rules files
- Secrets, credentials, or sensitive data (see security.md)

## Relation Wiring

When an agent discovers knowledge relevant to another agent, create a relation:

```
mcp__memory__create_relations(relations: [{
  from: "agent:code-reviewer",
  relationType: "learned-relevant-to",
  to: "agent:test-generator"
}])
```

This builds a knowledge graph where agents can discover each other's relevant learnings.

## For Skill Authors

Skills that dispatch agents with memory should include this in the agent's prompt:

> Before starting, check for prior learnings: read your memory node `agent:<name>` via the Memory MCP. Before finishing, write any new learnings worth preserving.

Skills should include `mcp__memory__open_nodes`, `mcp__memory__add_observations`, and `mcp__memory__create_entities` in the agent's available tools (these are available to all agents by default via the MCP server).
