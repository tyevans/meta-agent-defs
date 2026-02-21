# Domain Cross-Reference Index

Maps domain areas to their relevant rules, agent learnings, and skills. Enables domain-based knowledge discovery without restructuring storage.

## How to Use

When working in a domain area, consult this index to find all relevant knowledge sources. Skills like `/curate` and `/promote` can use this index to improve gap detection and cross-reference scoring.

## Domains

### Skill Authoring

Writing and editing skill definition files (`skills/<name>/SKILL.md`).

| Type | Source |
|------|--------|
| Rule | `.claude/rules/skill-authoring.md` |
| Rule | `rules/pipe-format.md` |
| Rule | `rules/information-architecture.md` |
| Learnings | `memory/agents/skill-author/learnings.md` |
| Skill | `/evolution`, `/drift` |

### Agent Authoring

Writing and editing agent definition files (`agents/*.md`, `.claude/agents/*.md`).

| Type | Source |
|------|--------|
| Rule | `.claude/rules/agent-authoring.md` |
| Rule | `.claude/rules/agent-memory.md` |
| Rule | `rules/information-architecture.md` |
| Skill | `/evolution`, `/drift` |

### Hook Authoring

Writing and editing hooks in `settings.json`.

| Type | Source |
|------|--------|
| Rule | `.claude/rules/hook-authoring.md` |
| Rule | `.claude/rules/security.md` |
| Skill | `/evolution` |

### Team Coordination

Assembling teams, running sprints, managing agent dispatch.

| Type | Source |
|------|--------|
| Rule | `rules/team-protocol.md` |
| Rule | `.claude/rules/fan-out-protocol.md` |
| Rule | `.claude/rules/agent-memory.md` |
| Skill | `/assemble`, `/sprint`, `/standup`, `/meeting` |

### Testing

Writing and organizing tests.

| Type | Source |
|------|--------|
| Rule | `rules/test-conventions.md` |
| Skill | `/test-strategy` |

### Knowledge Lifecycle

Managing learnings, rules, memory, and their lifecycle.

| Type | Source |
|------|--------|
| Rule | `rules/memory-layout.md` |
| Rule | `rules/information-architecture.md` |
| Learnings | `memory/agents/skill-author/learnings.md` |
| Skill | `/curate`, `/promote`, `/tend`, `/retro` |

### Git & Commits

Committing, branching, and git workflow.

| Type | Source |
|------|--------|
| Rule | `.claude/rules/commits.md` |
| Rule | `.claude/rules/security.md` |
| Skill | `/review`, `/evolution` |

### Composable Primitives

Using pipe-format skills for research and analysis.

| Type | Source |
|------|--------|
| Rule | `rules/pipe-format.md` |
| Rule | `rules/batch-safety.md` |
| Rule | `rules/context-trust.md` |
| Skill | `/gather`, `/distill`, `/rank`, `/critique`, `/verify`, `/filter`, `/merge` |

### Security

Preventing secrets leakage, unsafe operations, and unauthorized access.

| Type | Source |
|------|--------|
| Rule | `.claude/rules/security.md` |
| Skill | `/premortem`, `/review` |

### Quality & Definition of Done

Ensuring completeness and correctness of deliverables.

| Type | Source |
|------|--------|
| Rule | `.claude/rules/definition-of-done.md` |
| Skill | `/review`, `/assess` |

## Maintenance

This index is manually maintained. Update it when:
- A new rule file is created or removed
- A new agent starts accumulating learnings
- A domain area is discovered that cuts across multiple rules

Skills that detect cross-references (`/curate` with `related:` annotations, `/promote` with cross-agent detection) can suggest updates to this index.
