# Team Protocol Reference

Full schemas, templates, and examples for persistent learning teams. Behavioral rules are in [rules/team-protocol.md](../rules/team-protocol.md).

## Team Manifest Schema

```yaml
team: project-slug
description: "One-line project description"

members:
  - name: architect
    role: "System design, API contracts, patterns"
    tools: [Read, Grep, Glob, "Bash(git:*)"]
    owns: ["src/core/**", "docs/**", "*.md"]

  - name: backend
    role: "Server logic, data models, business rules"
    tools: [Read, Write, Edit, Grep, Glob, "Bash(git:*)", "Bash(uv run pytest:*)"]
    owns: ["src/domain/**", "src/infra/**", "tests/**"]
    isolation: worktree
```

### Required Fields

| Field | Scope | Description |
|-------|-------|-------------|
| `team` | top | Slug used for directory naming |
| `description` | top | One-line project description |
| `members` | top | Array of team member definitions |
| `name` | member | Unique identifier, lowercase |
| `role` | member | One-line responsibility description |
| `tools` | member | Tool list for `--allowedTools` |
| `owns` | member | Glob patterns for files this member is responsible for |

### Optional Fields

| Field | Scope | Values | Default | Description |
|-------|-------|--------|---------|-------------|
| `isolation` | member | `worktree`, `none` | `none` | Dispatch isolation mode. `worktree` = dedicated git worktree; `none` = main working context. |

## Dispatch Prompt Template

```
You are acting as team member "<name>".
Role: <role>
Owns: <owns patterns>

## Your Accumulated Learnings
<contents of .claude/tackline/memory/agents/<name>/learnings.md>

## Task
<task description>

## Commit Discipline
You are working in an isolated worktree. You MUST commit your changes before finishing.
- Commit early and often — each logical change gets its own commit
- Keep commits focused: one concern per commit (e.g., separate "add feature" from "update tests")
- Do not batch all changes into a single large commit
- Use conventional commit messages (feat:, fix:, refactor:, docs:, chore:)
- If your task is partially complete, commit what you have — partial progress committed is better than full progress uncommitted

## Reflection Protocol
After completing your task, end your response with a structured reflection:
- task_result: status (completed/partial/blocked/failed), summary, files_changed
- reflection: what_worked, what_didnt, confidence (high/medium/low)
- suggested_learnings: category, content, for_agent (if cross-agent)
- follow_up: suggested_next, needs_human
```

## Learnings File Format

```markdown
# Learnings: <name>

## Codebase Patterns
- API v2 uses joi for validation, not zod (added: 2026-02-13)

## Gotchas
- TrustService requires bootstrap before first call (added: 2026-02-13)

## Preferences
- User prefers explicit error types over generic Error (added: 2026-02-13)

## Cross-Agent Notes
- (from architect) Use TrustLevel enum from src/core/types.ts (added: 2026-02-13)
```

### Categories

- **Codebase Patterns** — Confirmed conventions, naming patterns, architectural rules
- **Gotchas** — Bugs, quirks, workarounds that cost time to discover
- **Preferences** — User/project preferences for style, approach, tooling
- **Cross-Agent Notes** — Learnings forwarded from other team members

## Shared Team Memory

Cross-cutting decisions live in `.claude/tackline/memory/team/decisions.md`:

```markdown
# Team Decisions

## Architecture
- REST over GraphQL for all public APIs (decided: 2026-02-13, by: architect)

## Conventions
- All dates stored as ISO 8601 UTC (decided: 2026-02-13, by: backend)
```

Retrospective summaries append to `.claude/tackline/memory/team/retro-history.md`.

## Reflection Schema

```json
{
  "type": "object",
  "required": ["task_result", "reflection", "suggested_learnings", "follow_up"],
  "properties": {
    "task_result": {
      "type": "object",
      "required": ["status", "summary", "files_changed"],
      "properties": {
        "status": { "type": "string", "enum": ["completed", "partial", "blocked", "failed"] },
        "summary": { "type": "string", "description": "1-3 sentence summary of what was done" },
        "files_changed": { "type": "array", "items": { "type": "string" }, "description": "Paths of files created or modified" }
      }
    },
    "reflection": {
      "type": "object",
      "required": ["what_worked", "what_didnt", "confidence"],
      "properties": {
        "what_worked": { "type": "string", "description": "Techniques or approaches that went well" },
        "what_didnt": { "type": "string", "description": "Issues encountered or suboptimal choices made" },
        "confidence": { "type": "string", "enum": ["high", "medium", "low"], "description": "Confidence in the quality of work produced" }
      }
    },
    "suggested_learnings": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["category", "content"],
        "properties": {
          "category": { "type": "string", "enum": ["codebase-pattern", "gotcha", "preference", "cross-agent"] },
          "content": { "type": "string", "description": "The learning to persist" },
          "for_agent": { "type": "string", "description": "Target agent name if cross-agent, otherwise omit" }
        }
      }
    },
    "follow_up": {
      "type": "object",
      "required": ["needs_human"],
      "properties": {
        "blocked_by": { "type": "string", "description": "What is blocking further progress, if anything" },
        "suggested_next": { "type": "string", "description": "Recommended next task or action" },
        "needs_human": { "type": "boolean", "description": "Whether human input is needed to proceed" }
      }
    }
  }
}
```

## Agent Checkpoint Format

```markdown
# Agent Checkpoint: <skill-name> / <member>

agent_id: <id returned by Task tool>
task: <one-line task description>
phase_completed: <last completed phase, e.g. "research", "draft", "review">
recorded: <ISO date>
```

## Dispatch Decision Table

| Condition | Mechanism |
|-----------|-----------|
| Tasks are independent or sequentially ordered | Task tool (serialize or parallelize) |
| Agents must communicate mid-execution | Native TeamCreate |
| Tasks have shared dependencies but no runtime message exchange | Task tool |

## Resume Decision Table

| Condition | Decision |
|-----------|----------|
| Checkpoint is older than 7 days | Fresh dispatch |
| Task definition has changed since checkpoint | Fresh dispatch |
| Working branch or major files changed since checkpoint | Fresh dispatch |
| Agent status was `blocked` or `failed` | Fresh dispatch |
| No checkpoint file exists | Fresh dispatch |
| Checkpoint exists, agent paused mid-task, context still valid | Resume |
| Checkpoint exists, task is identical, files unchanged | Resume |

## Learning Lifecycle

### 1. Seed (during /assemble)
- Create `.claude/tackline/memory/agents/<name>/learnings.md` with initial role context
- Create `.claude/tackline/memory/team/decisions.md` with empty sections

### 2. Inject (during spawn)
- Read learnings file, embed in `--append-system-prompt`
- Agent behavior is shaped by accumulated learnings

### 3. Reflect (after task)
- Parse `suggested_learnings` from reflection JSON
- Categorize each learning and append to the appropriate agent's file
- Route `cross-agent` learnings to the target agent's file under "Cross-Agent Notes"

### 4. Prune (during /retro)
- Merge duplicate or similar entries
- Archive stale entries (>21 days, never referenced)
- Promote high-value entries to rules or CLAUDE.md
- Validate cross-agent notes (acknowledge, act, or discard within 14 days)
- Ensure no file exceeds 60 lines (30 core + 30 task-relevant)

### 5. Transfer (optional)
- When a new member is added, seed their learnings from related members
- When a member is removed, redistribute their cross-agent notes

## Memory MCP Integration

File-based learnings are primary. Memory MCP is secondary.

| Scenario | Use |
|----------|-----|
| Team member learnings (within a project) | File-based |
| Cross-project knowledge queries | MCP (`mcp__memory__search_nodes`) |
| Agent identity when file context unavailable | MCP |
| Relation-based discovery between agents | MCP |

### MCP Namespace Convention

Each agent gets a unique entity: `agent:<agent-name>` (e.g., `agent:code-reviewer`) with type `agent-memory`.

### Read-on-Spawn

```
mcp__memory__open_nodes(names: ["agent:<agent-name>"])
```

If the node exists, review observations before beginning work. If not, proceed normally.

### Write-on-Complete

```
mcp__memory__add_observations(observations: [{
  entityName: "agent:<agent-name>",
  contents: ["<observation>"]
}])
```

Create the entity first if it doesn't exist:

```
mcp__memory__create_entities(entities: [{
  name: "agent:<agent-name>",
  entityType: "agent-memory",
  observations: ["<initial observations>"]
}])
```

### Relation Wiring

When an agent discovers knowledge relevant to another agent:

```
mcp__memory__create_relations(relations: [{
  from: "agent:code-reviewer",
  relationType: "learned-relevant-to",
  to: "agent:test-generator"
}])
```

### For Skill Authors

Skills dispatching agents with memory should:

1. Check for team manifest first (`.claude/team.yaml`). If present, use file-based learnings.
2. Fall back to MCP if no team manifest exists. Include in the agent's prompt: "Before starting, check for prior learnings via `agent:<name>` in Memory MCP. Before finishing, write any new learnings worth preserving."
