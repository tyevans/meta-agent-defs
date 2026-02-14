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

## File-Based Learnings (Primary)

When a project has a team manifest (`.claude/team.yaml`), file-based learnings are the primary persistence mechanism. See `team-protocol.md` for the full spec.

- **Location**: `memory/agents/<name>/learnings.md`
- **Format**: Categorized markdown (Codebase Patterns, Gotchas, Preferences, Cross-Agent Notes)
- **Injection**: Contents are embedded in `--append-system-prompt` at spawn time
- **Lifecycle**: Seed → Inject → Reflect → Prune (see team-protocol.md)

### Advantages over MCP

- Version-controlled (git history shows learning evolution)
- Injectable into CLI prompts (works with `claude -p`)
- Human-readable and editable
- Reviewable in PRs

### When to Use File-Based vs MCP

| Scenario | Use |
|----------|-----|
| Team member learnings (within a project) | File-based |
| Cross-project knowledge queries | MCP |
| Agent identity when file context unavailable | MCP |
| Relation-based discovery between agents | MCP |

## For Skill Authors

Skills that dispatch agents with memory should:

1. **Check for team manifest first** (`.claude/team.yaml`). If present, use file-based learnings from `memory/agents/<name>/learnings.md`.
2. **Fall back to MCP** if no team manifest exists. Include this in the agent's prompt:

> Before starting, check for prior learnings: read your memory node `agent:<name>` via the Memory MCP. Before finishing, write any new learnings worth preserving.

Skills using MCP should include `mcp__memory__open_nodes`, `mcp__memory__add_observations`, and `mcp__memory__create_entities` in the agent's available tools (these are available to all agents by default via the MCP server).
