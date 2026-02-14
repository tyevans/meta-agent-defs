---
paths:
  - "skills/**/*.md"
  - "agents/**/*.md"
---

# Team Protocol

Spec for persistent learning teams. Defines the manifest format, spawn protocol, reflection schema, and learning lifecycle. All team-aware skills reference this document.

## Team Manifest

Teams are defined in `.claude/team.yaml`. This is the single source of truth for team composition.

```yaml
team: project-slug
description: "One-line project description"

defaults:
  model: sonnet
  budget: 0.50
  permission-mode: dontAsk

members:
  - name: architect
    role: "System design, API contracts, patterns"
    model: opus            # override default
    budget: 1.00           # override default
    tools: [Read, Grep, Glob, "Bash(git:*)"]
    owns: ["src/core/**", "docs/**", "*.md"]

  - name: backend
    role: "Server logic, data models, business rules"
    tools: [Read, Write, Edit, Grep, Glob, "Bash(git:*)", "Bash(uv run pytest:*)"]
    owns: ["src/domain/**", "src/infra/**", "tests/**"]
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

| Field | Scope | Default | Description |
|-------|-------|---------|-------------|
| `defaults.model` | top | `sonnet` | Default model for all members |
| `defaults.budget` | top | `0.50` | Default max budget in USD |
| `defaults.permission-mode` | top | `dontAsk` | Default permission mode |
| `model` | member | from defaults | Model override |
| `budget` | member | from defaults | Budget override |
| `permission-mode` | member | from defaults | Permission mode override |

## Learnings Files

Each member has a file at `memory/agents/<name>/learnings.md`. These are version-controlled, human-readable, and injected into every spawn.

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
2. **Archive stale entries**: Move entries older than 21 days (without recent references) to `memory/agents/<name>/archive.md`
3. **Promote high-value entries**: If a learning has been confirmed across 3+ sprints, promote it to `.claude/rules/` or CLAUDE.md
4. **Validate cross-agent notes**: Notes in "Cross-Agent Notes" that are older than 14 days must be either:
   - Acknowledged (merged into other sections)
   - Acted upon (integrated into the agent's workflow)
   - Discarded (moved to archive with rationale)

## Shared Team Memory

Cross-cutting decisions live in `memory/team/decisions.md`:

```markdown
# Team Decisions

## Architecture
- REST over GraphQL for all public APIs (decided: 2026-02-13, by: architect)

## Conventions
- All dates stored as ISO 8601 UTC (decided: 2026-02-13, by: backend)
```

Retrospective summaries append to `memory/team/retro-history.md`.

## Spawn Protocol

The orchestrator constructs `claude -p` invocations from the manifest and learnings.

### Command Template

```bash
claude -p \
  --append-system-prompt "$(cat <<'PROMPT'
# Team Member: <name>
Role: <role>
Owns: <owns patterns>

## Your Accumulated Learnings
<contents of memory/agents/<name>/learnings.md>

## Reflection Protocol
After completing your task, return structured JSON matching the reflection schema.
Include honest self-assessment in the reflection section.
Suggest learnings that should be persisted for future spawns.
PROMPT
)" \
  --model <model> \
  --allowedTools "<tools joined by comma>" \
  --permission-mode <permission-mode> \
  --max-budget-usd <budget> \
  --output-format json \
  --json-schema '<reflection schema JSON>' \
  --no-session-persistence \
  "<task description with bead reference and context>"
```

### Prompt Composition Order

1. Base system prompt (Claude Code default)
2. `--append-system-prompt` with: role, owns, learnings, reflection protocol
3. Task prompt (the `-p` argument)

This means the agent sees its identity and accumulated knowledge before the task.

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
- Create `memory/agents/<name>/learnings.md` with initial role context
- Create `memory/team/decisions.md` with empty sections

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

File-based learnings are the **primary** persistence mechanism:
- Version-controlled (git history shows learning evolution)
- Injectable into CLI prompts (works with `claude -p`)
- Human-readable and editable

Memory MCP remains available as a **secondary** mechanism for:
- Cross-project knowledge queries (`mcp__memory__search_nodes`)
- Agent identity persistence when file context is unavailable
- Relation-based discovery between agents

Skills should default to file-based learnings and only use MCP when specifically needed for cross-project queries.
