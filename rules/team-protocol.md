---
paths:
  - "skills/**/*.md"
  - "agents/**/*.md"
strength: should
freshness: 2026-02-21
---

# Team Protocol

Spec for persistent learning teams. Defines the manifest format, spawn protocol, reflection schema, and learning lifecycle. All team-aware skills reference this document.

## Team Manifest

Teams are defined in `.claude/team.yaml`. This is the single source of truth for team composition.

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
| `isolation` | member | `worktree`, `none` | `none` | Dispatch isolation mode. `worktree` means the member can be dispatched in a dedicated git worktree; `none` means the member must run in the main working context. |

**When to set `isolation: worktree`:**
- The member's `owns` patterns don't overlap with any other member's patterns
- The member's tasks don't require reading or writing shared state files (e.g., `.claude/tackline/memory/team/decisions.md`)
- The member's work is self-contained within its owned paths for the duration of the sprint

**Consumed by /sprint**: The sprint skill reads `isolation` at dispatch time and passes worktree-isolated members to the worktree dispatch path. Members with `isolation: none` (or no `isolation` field) are dispatched in the main context.

## Learnings Files

Each member has a file at `.claude/tackline/memory/agents/<name>/learnings.md`. These are version-controlled, human-readable, and injected into every spawn.

### Format

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

### Size Cap and Tiered Structure

Learnings files are capped at **60 lines**. Structure them as:

- **Core (30 lines max)**: Always-injected entries with high reuse frequency (5+ references across sprints). These are foundational patterns and gotchas that apply to every task.
- **Task-Relevant (30 lines max)**: Selective entries that apply to specific contexts. These should be pruned more aggressively.

### Consolidation Triggers

When a file exceeds 60 lines or meets any of these conditions:
1. **Merge similar entries**: Consolidate learnings that say the same thing in different ways
2. **Archive stale entries**: Move entries older than 21 days (without recent references) to `.claude/tackline/memory/agents/<name>/archive.md`
3. **Promote high-value entries**: If a learning has been confirmed across 3+ sprints, promote it to `.claude/rules/` or CLAUDE.md
4. **Validate cross-agent notes**: Notes in "Cross-Agent Notes" that are older than 14 days must be either:
   - Acknowledged (merged into other sections)
   - Acted upon (integrated into the agent's workflow)
   - Discarded (moved to archive with rationale)

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

## Spawn Protocol

The orchestrator dispatches team members via the Task tool.

### Dispatch via Task Tool

Use the Task tool with `subagent_type: "general-purpose"` (or a custom agent type matching the member name if registered in `.claude/agents/`).

The task prompt must include:
1. The member's accumulated learnings (contents of `.claude/tackline/memory/agents/<name>/learnings.md`)
2. The task description with context
3. Reflection instructions requesting structured output

### Prompt Template

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

### Dispatch Strategy

**Default: parallelize with worktree isolation.** Dispatch tasks concurrently using `isolation: "worktree"` and `run_in_background: true`. Each agent gets its own repo copy, eliminating merge conflicts regardless of file overlap. Cherry-pick or merge worktree branches after agents complete.

**Fall back to serial** only when tasks have true sequential dependencies — i.e., each task needs the previous one's output to proceed. Serial dispatch is the exception, not the default.

## Dispatch Model

### Decision Boundary: Task vs. Native Teams

Default to Task tool dispatch. Prefer native TeamCreate only when agents must exchange messages during execution — not merely because tasks depend on each other.

| Condition | Mechanism |
|-----------|-----------|
| Tasks are independent or sequentially ordered | Task tool (serialize or parallelize) |
| Agents must communicate mid-execution (e.g., one agent's output shapes another's next step in real time) | Native TeamCreate |
| Tasks have shared dependencies but no runtime message exchange | Task tool |

Having dependencies between tasks is not sufficient justification for native teams. Use your task tracker for ordering, Task tool for dispatch.

### Injection Invariant

Regardless of dispatch mechanism, the orchestrator always injects learnings into the agent's initial prompt. Agents never self-load their own learnings files.

1. Read `.claude/tackline/memory/agents/<name>/learnings.md` before dispatching.
2. Embed the contents in the agent's initial prompt (see Prompt Template above).
3. This applies to both Task tool dispatch and native TeamCreate spawns.

### Known Gap: team.yaml Budget and Tools Enforcement

The `tools` and budget fields in `team.yaml` are not enforced by native TeamCreate. Enforcement is the orchestrator's responsibility:

- Pass `--allowedTools` explicitly when constructing the spawn.
- Validate that the tool list matches the member's `tools` field before dispatching.
- Budget limits must be applied in the orchestrator logic, not relied upon from TeamCreate.

## Agent Continuity

After expensive phases — multi-turn research, large code generation, or long iterative tasks — the orchestrator SHOULD record the agent ID returned by the Task tool so the agent can be resumed rather than re-dispatched from scratch.

### Resume Convention

When the Task tool returns an agent ID for a completed or partial run, write it to a checkpoint file:

```
.claude/tackline/memory/scratch/<skill-name>-agent-<member>.md
```

Format:

```markdown
# Agent Checkpoint: <skill-name> / <member>

agent_id: <id returned by Task tool>
task: <one-line task description>
phase_completed: <last completed phase, e.g. "research", "draft", "review">
recorded: <ISO date>
```

### When to Resume

On re-invocation, check for a checkpoint file before dispatching. Offer resume when:

1. The checkpoint file exists and is less than 7 days old
2. The files the agent was working on have not changed significantly since the checkpoint
3. The task definition has not changed
4. The agent did not fail or block — it paused or was partially completed

Resume by passing the agent ID to the Task tool's `agent_id` parameter, injecting the same learnings plus a brief resume note in the prompt.

### When to Dispatch Fresh

| Condition | Decision |
|-----------|----------|
| Checkpoint is older than 7 days | Fresh dispatch |
| Task definition has changed since checkpoint | Fresh dispatch |
| Working branch or major files changed since checkpoint | Fresh dispatch |
| Agent status was `blocked` or `failed` | Fresh dispatch |
| No checkpoint file exists | Fresh dispatch |
| Checkpoint exists, agent paused mid-task, context still valid | Resume |
| Checkpoint exists, task is identical, files unchanged | Resume |

### Checkpoint Cleanup

Delete the checkpoint file after a successful fresh dispatch or after the resumed agent reaches `completed` status. Stale checkpoints older than 14 days may be deleted during `/retro` without review.

## Reflection Schema

Every spawned team member returns this JSON structure:

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

## Compatibility with Memory MCP

File-based learnings are the **primary** persistence mechanism. Memory MCP is a **secondary** mechanism for cross-project queries, agent identity when file context is unavailable, and relation-based discovery.

| Scenario | Use |
|----------|-----|
| Team member learnings (within a project) | File-based |
| Cross-project knowledge queries | MCP (`mcp__memory__search_nodes`) |
| Agent identity when file context unavailable | MCP |
| Relation-based discovery between agents | MCP |

### MCP Namespace Convention

Each agent gets a unique entity: `agent:<agent-name>` (e.g., `agent:code-reviewer`) with type `agent-memory`.

### Read-on-Spawn (MCP path)

```
mcp__memory__open_nodes(names: ["agent:<agent-name>"])
```

If the node exists, review observations before beginning work. If not, proceed normally.

### Write-on-Complete (MCP path)

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

### What to Remember (MCP)

- Patterns confirmed across multiple files or sessions
- Codebase-specific conventions (naming, structure, architecture)
- Library gotchas and workarounds that were hard-won
- User preferences for approach, style, or tooling
- Recurring issues and their root causes

Do NOT remember: session-specific context, unverified hunches, information already in CLAUDE.md or rules, secrets.

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
