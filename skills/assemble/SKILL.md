---
name: assemble
description: "Create a persistent agent team for a project with roles, memory namespaces, and a shared backlog. Use when starting a new project, forming a team for long-horizon work, or setting up agents that learn and improve across sessions. Keywords: team, project, setup, roles, persistent, agents, staff."
argument-hint: "<project description>"
disable-model-invocation: true
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Bash(mkdir:*), Write, Edit, Task
---

# Assemble: Persistent Team Creation

You are running **Assemble** -- a workflow to create a persistent agent team for a project. Unlike ad-hoc teams spawned by other skills (meeting, blossom), an assembled team persists across sessions. Each member has a role, a memory namespace, and accumulated learnings.

**Project:** $ARGUMENTS

## Phase 1: Understand the Project

### 1a. Explore

Read the project's key files to understand its domain:
- README, CLAUDE.md, package.json/Cargo.toml/pyproject.toml (or equivalent)
- Directory structure (`ls` top level)
- Existing `.claude/` config if present

### 1b. Identify Roles

Based on the project, select 3-6 roles. Each role should have a clear responsibility boundary.

**Role templates** (pick, adapt, or create new):

| Role | Responsibility | When to include |
|------|---------------|-----------------|
| Architect | System design, API contracts, patterns | Always -- every project needs design oversight |
| Backend | Server logic, data models, business rules | Projects with server-side code |
| Frontend | UI components, state management, UX | Projects with a user interface |
| Tester | Test strategy, coverage, edge cases | Projects with any test suite |
| DevOps | CI/CD, deployment, infrastructure | Projects with deployment pipelines |
| Security | Auth, input validation, threat modeling | Projects handling user data |
| Domain Expert | Domain-specific knowledge and rules | Domain-heavy projects (finance, health, etc.) |
| Tech Writer | Docs, API references, guides | Projects with external users |

### 1c. Confirm with User

Present the proposed team to the user:

```markdown
## Proposed Team for: [project name]

| Role | Responsibility | Why |
|------|---------------|-----|
| [role] | [1-sentence responsibility] | [why this project needs it] |
| ... | ... | ... |
```

Ask the user to approve, modify roles, or suggest additions.

---

## Phase 2: Create the Team

### 2a. Team Directory

Create the team configuration:

```bash
mkdir -p .claude/teams
```

Write `.claude/teams/<project-slug>.md`:

```markdown
# Team: [project name]

## Members

### [Role Name]
- **Responsibility**: [what they own]
- **Memory node**: agent:[project-slug]-[role]
- **Perspective**: [how they think, what they prioritize]
- **Tools**: [tools they typically need]

### [Next Role]
...

## Working Agreements
- All members follow the agent memory protocol (read-on-spawn, write-on-complete)
- Disagreements are surfaced in /meeting, not suppressed
- Each member owns their area but reviews are cross-functional

## Project Context
- **Repo**: [path]
- **Stack**: [languages, frameworks]
- **Key files**: [most important files for context]
```

### 2b. Initialize Memory Nodes

For each team member, create their memory entity via the Memory MCP:

```
mcp__memory__create_entities(entities: [{
  name: "agent:[project-slug]-[role]",
  entityType: "agent-memory",
  observations: [
    "Role: [role name] for [project name]",
    "Responsibility: [what they own]",
    "Project stack: [languages, frameworks]",
    "Key files: [files relevant to this role]"
  ]
}])
```

### 2c. Wire Relations

Create relations between team members who need to collaborate:

```
mcp__memory__create_relations(relations: [
  { from: "agent:[slug]-backend", relationType: "collaborates-with", to: "agent:[slug]-frontend" },
  { from: "agent:[slug]-tester", relationType: "reviews-work-of", to: "agent:[slug]-backend" }
])
```

### 2d. Initialize Backlog

If the project doesn't already have beads:

```bash
bd create --title="EPIC: [project name] team setup" --type=feature --priority=2 \
  --description="Tracking epic for initial team assembly and onboarding."
```

---

## Phase 3: Verify

### 3a. Check Memory

Verify all memory nodes were created:

```
mcp__memory__search_nodes(query: "[project-slug]")
```

### 3b. Report

```markdown
## Team Assembled: [project name]

### Members
| Role | Memory Node | Status |
|------|------------|--------|
| [role] | agent:[slug]-[role] | Ready |
| ... | ... | ... |

### How to Use This Team
- `/meeting [topic]` -- Discuss with team members (use role names as panelists)
- `/standup` -- Get status from each member (when implemented)
- `/fractal [goal]` -- Deep exploration dispatched to relevant members
- Dispatch work directly: `Task({ name: "[role]", prompt: "..." })`

### Memory Nodes Created
[count] entities, [count] relations

### Next Steps
[Suggested first actions for the team]
```

---

## Guidelines

1. **Roles, not people.** A role is a perspective and responsibility boundary. One human can fill multiple roles.
2. **Start small.** 3-4 roles for most projects. Add roles when the team feels a gap, not preemptively.
3. **Memory is the persistence layer.** Team config is a file; agent knowledge lives in the Memory MCP graph.
4. **Confirm before creating.** Always show the proposed team to the user before creating memory nodes.
5. **Teams evolve.** Roles can be added, removed, or redefined. Memory nodes accumulate -- they don't reset.
