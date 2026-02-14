---
name: assemble
description: "Create a persistent learning team for a project with roles, ownership, and file-based learnings that improve agent behavior across sessions. Use when starting a new project, forming a team for long-horizon work, or upgrading from ad-hoc agent dispatch to structured team coordination. Keywords: team, project, setup, roles, persistent, agents, staff, learning."
argument-hint: "<project description>"
disable-model-invocation: true
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Bash(mkdir:*), Write, Edit, Task
---

# Assemble: Persistent Learning Team Creation

You are running **Assemble** -- a workflow to create a persistent agent team that learns and improves across sessions. Each member has a role, file ownership, and a learnings file that gets injected into every spawn.

**Project:** $ARGUMENTS

## When to Use

- When starting a new project that will have multiple work sessions
- When upgrading from ad-hoc agent dispatch to structured team coordination
- When you want agents to accumulate knowledge and improve over time
- When a project has clear ownership boundaries (backend, frontend, infrastructure, etc.)
- When working on long-horizon work that benefits from persistent context

## How It Works

```
Explore project → Propose roles + ownership → User confirms
  → Write team.yaml manifest
    → Create learnings.md per member (seeded with role context)
      → Create shared team memory (decisions.md)
        → Initialize backlog → Report
```

## Phase 1: Understand the Project

### 1a. Explore

Read the project's key files to understand its domain:
- README, CLAUDE.md, package.json/Cargo.toml/pyproject.toml (or equivalent)
- Directory structure (`ls` top level, key subdirectories)
- Existing `.claude/` config if present
- Existing `.claude/team.yaml` if present (this may be a team update, not creation)

### 1b. Identify Roles and Ownership

Based on the project, select 3-6 roles. Each role needs a clear responsibility boundary AND file ownership (glob patterns for the files they are responsible for).

**Role templates** (pick, adapt, or create new):

| Role | Responsibility | Typical Ownership | When to include |
|------|---------------|-------------------|-----------------|
| architect | System design, API contracts, patterns | `src/core/**`, `*.md`, `docs/**` | Always |
| backend | Server logic, data models, business rules | `src/domain/**`, `src/infra/**` | Projects with server-side code |
| frontend | UI components, state management, UX | `src/ui/**`, `src/components/**` | Projects with a user interface |
| tester | Test strategy, coverage, edge cases | `tests/**`, `*.test.*` | Projects with a test suite |
| devops | CI/CD, deployment, infrastructure | `.github/**`, `Dockerfile`, `*.yml` | Projects with deployment pipelines |
| security | Auth, input validation, threat modeling | `src/auth/**`, `src/middleware/**` | Projects handling user data |

### 1c. Confirm with User

Present the proposed team:

```markdown
## Proposed Team for: [project name]

| Role | Responsibility | Owns | Model | Budget |
|------|---------------|------|-------|--------|
| [role] | [1-sentence] | [glob patterns] | [model] | $[budget] |
| ... | ... | ... | ... | ... |

**Defaults**: model=sonnet, budget=$0.50, permission-mode=dontAsk
```

Ask the user to approve, modify roles, adjust ownership, or suggest additions.

---

## Phase 2: Create the Team

### 2a. Directory Structure

```bash
mkdir -p .claude
mkdir -p memory/agents
mkdir -p memory/team
```

### 2b. Write Team Manifest

Write `.claude/team.yaml` following the format defined in the team-protocol rule:

```yaml
team: <project-slug>
description: "<project description>"

defaults:
  model: sonnet
  budget: 0.50
  permission-mode: dontAsk

members:
  - name: <role>
    role: "<responsibility>"
    model: <model if non-default>
    budget: <budget if non-default>
    tools: [<tool list>]
    owns: [<glob patterns>]
```

### 2c. Create Learnings Files

For each member, create `memory/agents/<name>/learnings.md`:

```markdown
# Learnings: <name>

## Codebase Patterns
- (none yet)

## Gotchas
- (none yet)

## Preferences
- (none yet)

## Cross-Agent Notes
- (none yet)
```

Seed each file with 2-3 initial observations based on Phase 1 exploration. For example, an architect might get "Project uses Express.js with TypeScript strict mode" under Codebase Patterns.

### 2d. Create Shared Memory

Write `memory/team/decisions.md`:

```markdown
# Team Decisions

## Architecture
- (none yet)

## Conventions
- (none yet)
```

Write `memory/team/retro-history.md`:

```markdown
# Retrospective History

(No retrospectives yet. Run /retro to add entries.)
```

### 2e. Initialize Backlog (optional)

If the project doesn't already have beads and the user wants backlog tracking:

```bash
bd create --title="EPIC: [project name] team setup" --type=feature --priority=2
```

---

## Phase 3: Verify and Report

### 3a. Verify Files

Check all expected files were created:
- `.claude/team.yaml`
- `memory/agents/<name>/learnings.md` for each member
- `memory/team/decisions.md`
- `memory/team/retro-history.md`

### 3b. Report

```markdown
## Team Assembled: [project name]

### Members
| Role | Owns | Model | Learnings |
|------|------|-------|-----------|
| [role] | [patterns] | [model] | memory/agents/[name]/learnings.md |
| ... | ... | ... | ... |

### Files Created
- `.claude/team.yaml` — Team manifest
- `memory/agents/*/learnings.md` — Per-member learnings ([count] files)
- `memory/team/decisions.md` — Shared decisions log
- `memory/team/retro-history.md` — Retro history

### How to Use This Team
- `/sprint` — Plan and dispatch work to team members (the learning loop)
- `/standup` — Status sync from each member's perspective
- `/meeting [topic]` — Multi-agent discussion using team roles
- `/retro` — Reflect on session and prune/promote learnings

### The Learning Loop
Each time you dispatch work via /sprint:
1. Agent receives its accumulated learnings in the system prompt
2. Agent works and returns structured reflection
3. Orchestrator extracts learnings and appends to the agent's file
4. Next spawn: agent behaves differently because its learnings changed
```

---

## Guidelines

1. **Ownership is key.** Every source file should be owned by exactly one member. Use `owns` patterns to make this clear. Overlap is acceptable for shared files (e.g., README).
2. **Start small.** 3-4 roles for most projects. Add roles when the team feels a gap, not preemptively.
3. **Files are the persistence layer.** Team config is YAML; learnings are Markdown. Both are version-controlled and human-editable.
4. **Confirm before creating.** Always show the proposed team to the user before writing files.
5. **Seed learnings.** Don't leave learnings files empty -- add 2-3 observations from your Phase 1 exploration so agents start with useful context.
6. **Teams evolve.** Roles can be added, removed, or redefined. Learnings accumulate. Run `/retro` to prune.
