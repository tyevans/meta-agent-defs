# Team System Guide

A comprehensive guide to the persistent learning team system for Claude Code — a framework for creating project teams where agents accumulate knowledge and improve across sessions.

---

## Table of Contents

1. [What is a Persistent Learning Team?](#what-is-a-persistent-learning-team)
2. [Core Concepts](#core-concepts)
3. [Setup: Creating Your Team](#setup-creating-your-team)
4. [The Learning Loop](#the-learning-loop)
5. [Memory Structure](#memory-structure)
6. [Pruning & Maintenance](#pruning--maintenance)
7. [Cross-Agent Knowledge Sharing](#cross-agent-knowledge-sharing)
8. [Workflow Integration](#workflow-integration)
9. [Working Without Beads](#working-without-beads)
10. [Troubleshooting & Tips](#troubleshooting--tips)

---

## What is a Persistent Learning Team?

A **persistent learning team** is a set of specialized agents that:
- Have well-defined roles and ownership boundaries
- Accumulate knowledge in version-controlled files
- Get better at their jobs over time as they learn from each task
- Share knowledge across team members and sessions

### How It Differs from Ad-Hoc Agent Dispatch

| Ad-Hoc Dispatch | Persistent Learning Team |
|----------------|-------------------------|
| Agent starts fresh every time | Agent loads accumulated learnings at startup |
| No memory between sessions | Learnings persist in git-tracked files |
| Generic prompt every time | Role-specific context + learned patterns |
| No knowledge sharing | Cross-agent notes route insights between members |
| You repeat context manually | System injects role + learnings automatically |

### When to Use

**Use a team when:**
- Your project will span multiple work sessions
- You have clear ownership boundaries (backend, frontend, infra, etc.)
- Agents would benefit from remembering patterns they discover
- You want agents to share knowledge with each other
- You're working on long-horizon work where context compounds

**Stick with ad-hoc dispatch when:**
- One-off tasks or scripts
- Exploration with no clear ownership structure yet
- You're still figuring out the project architecture

---

## Core Concepts

### Team Manifest

A YAML file at `.claude/team.yaml` defining:
- Team name and description
- Default settings (model, budget, permissions)
- Member definitions (role, ownership, tools, overrides)

### Member Roles

Each team member has:
- **Name**: Unique identifier (e.g., `backend`, `frontend`, `tester`)
- **Role**: One-line responsibility description
- **Tools**: Allowed tools for `--allowedTools` parameter
- **Owns**: Glob patterns for files this member is responsible for
- **Model/Budget**: Optional overrides of team defaults

### Learnings Files

Each member has a markdown file at `memory/agents/<name>/learnings.md` containing:
- **Codebase Patterns**: Confirmed conventions, architectural rules
- **Gotchas**: Bugs, quirks, workarounds discovered the hard way
- **Preferences**: User/project style preferences
- **Cross-Agent Notes**: Knowledge forwarded from other team members

These files are:
- Version-controlled (git tracks evolution)
- Injected into the agent's prompt at spawn time
- Human-readable and editable
- Capped at 60 lines (30 core + 30 task-relevant)

### The Learning Loop

```
Spawn agent with learnings
  → Agent works on task
    → Agent reflects on what it learned
      → Orchestrator extracts learnings
        → Learnings appended to member's file
          → Next spawn: agent sees new learnings
```

---

## Setup: Creating Your Team

### Quick Start: Use `/assemble`

The `/assemble` skill automates team creation:

```
/assemble <project description>
```

This will:
1. Explore your project structure
2. Propose roles and ownership patterns
3. Ask for your approval
4. Create all necessary files

### Manual Setup

If you prefer manual setup or want to understand the internals:

#### Step 1: Create Directory Structure

```bash
mkdir -p .claude
mkdir -p memory/agents
mkdir -p memory/team
```

#### Step 2: Write Team Manifest

Create `.claude/team.yaml`:

```yaml
team: my-project
description: "Web app with REST API and React frontend"

defaults:
  model: sonnet
  budget: 0.50
  permission-mode: dontAsk

members:
  - name: backend
    role: "Server logic, data models, business rules"
    tools: [Read, Write, Edit, Grep, Glob, "Bash(git:*)", "Bash(pytest:*)"]
    owns: ["src/api/**", "src/models/**", "tests/api/**"]

  - name: frontend
    role: "UI components, state management, UX"
    tools: [Read, Write, Edit, Grep, Glob, "Bash(git:*)", "Bash(npm:*)"]
    owns: ["src/components/**", "src/pages/**", "src/hooks/**"]

  - name: tester
    role: "Test strategy, coverage, edge cases"
    model: opus  # Override default for complex analysis
    tools: [Read, Write, Edit, Grep, Glob, "Bash(git:*)", "Bash(pytest:*)"]
    owns: ["tests/**", "**/*.test.*"]
```

#### Step 3: Create Learnings Files

For each member, create `memory/agents/<name>/learnings.md`:

```bash
# For backend
mkdir -p memory/agents/backend
cat > memory/agents/backend/learnings.md << 'EOF'
# Learnings: backend

## Codebase Patterns
- (none yet)

## Gotchas
- (none yet)

## Preferences
- (none yet)

## Cross-Agent Notes
- (none yet)
EOF
```

Repeat for each member. Optionally seed with 2-3 observations from your initial project exploration.

#### Step 4: Create Shared Memory

Create `memory/team/decisions.md`:

```markdown
# Team Decisions

## Architecture
- (none yet)

## Conventions
- (none yet)
```

Create `memory/team/retro-history.md`:

```markdown
# Retrospective History

(No retrospectives yet. Run /retro to add entries.)
```

### Role Templates

Common role patterns to adapt:

| Role | Responsibility | Typical Ownership | When to Include |
|------|---------------|-------------------|-----------------|
| **architect** | System design, API contracts, patterns | `src/core/**`, `*.md`, `docs/**` | Always (foundation role) |
| **backend** | Server logic, data models, business rules | `src/domain/**`, `src/infra/**`, `tests/api/**` | Projects with server-side code |
| **frontend** | UI components, state management, UX | `src/ui/**`, `src/components/**`, `src/pages/**` | Projects with a user interface |
| **tester** | Test strategy, coverage, edge cases | `tests/**`, `*.test.*` | Projects with test suites |
| **devops** | CI/CD, deployment, infrastructure | `.github/**`, `Dockerfile`, `*.yml` | Projects with deployment pipelines |
| **security** | Auth, input validation, threat modeling | `src/auth/**`, `src/middleware/**` | Projects handling user data |

**Guideline**: Start with 3-4 roles. Add roles when the team feels a gap, not preemptively.

---

## The Learning Loop

The learning loop is the heartbeat of the team system. Here's how it works step by step.

### Phase 1: Spawn with Context

When you dispatch a team member (via `/sprint` or manually via the Task tool), the orchestrator:

1. Reads the member's `learnings.md` file
2. Composes a prompt that includes:
   - Member identity (name, role, ownership)
   - Full contents of their learnings file
   - The task description
   - Instructions for structured reflection

Example prompt structure:

```
You are acting as team member "backend".
Role: Server logic, data models, business rules
Owns: src/api/**, src/models/**, tests/api/**

## Your Accumulated Learnings
<contents of memory/agents/backend/learnings.md>

## Task
[Task description with bead reference or manual context]

## Reflection Protocol
After completing your task, end your response with a structured reflection:
- task_result: status (completed/partial/blocked/failed), summary, files_changed
- reflection: what_worked, what_didnt, confidence (high/medium/low)
- suggested_learnings: category, content, for_agent (if cross-agent)
- follow_up: suggested_next, needs_human
```

### Phase 2: Agent Works

The agent:
- Sees its accumulated knowledge from past tasks
- Uses that context to make better decisions
- Avoids repeating past mistakes (logged in Gotchas)
- Follows established patterns (logged in Codebase Patterns)
- Respects user preferences (logged in Preferences)

### Phase 3: Agent Reflects

At the end of the task, the agent provides structured reflection:

```json
{
  "task_result": {
    "status": "completed",
    "summary": "Added pagination to user listing endpoint",
    "files_changed": ["src/api/users.py", "tests/api/test_users.py"]
  },
  "reflection": {
    "what_worked": "Using the UserRepository pattern kept business logic clean",
    "what_didnt": "Initial approach forgot to handle empty result sets",
    "confidence": "high"
  },
  "suggested_learnings": [
    {
      "category": "codebase-pattern",
      "content": "Pagination endpoints return {items, total, page, per_page} structure"
    },
    {
      "category": "gotcha",
      "content": "Empty user queries need explicit [] not null for frontend compatibility"
    },
    {
      "category": "cross-agent",
      "content": "User list endpoint now supports ?page=N&per_page=M params",
      "for_agent": "frontend"
    }
  ],
  "follow_up": {
    "suggested_next": "Add pagination to other listing endpoints for consistency",
    "needs_human": false
  }
}
```

### Phase 4: Orchestrator Persists Learnings

The orchestrator (you or `/sprint`) processes the reflection:

1. **Validates each learning**: Is this durable (useful across sessions) or ephemeral (only relevant now)?
2. **Categorizes**: Maps to the correct section (Codebase Patterns, Gotchas, Preferences, Cross-Agent Notes)
3. **Appends to learnings file**: Adds entries with today's date
4. **Routes cross-agent notes**: If `for_agent` is specified, also adds to that member's file under "Cross-Agent Notes"

Example update to `memory/agents/backend/learnings.md`:

```markdown
## Codebase Patterns
- Pagination endpoints return {items, total, page, per_page} structure (added: 2026-02-13)

## Gotchas
- Empty user queries need explicit [] not null for frontend compatibility (added: 2026-02-13)
```

And to `memory/agents/frontend/learnings.md`:

```markdown
## Cross-Agent Notes
- (from backend) User list endpoint now supports ?page=N&per_page=M params (added: 2026-02-13)
```

### Phase 5: Next Spawn Sees New Context

The next time you dispatch the backend member, they see these new learnings in their prompt. They'll naturally:
- Use the pagination pattern on other endpoints
- Remember to return `[]` not `null` for empty queries
- Build on what they learned

**This is how agents improve over time.**

---

## Memory Structure

### Learnings File Format

Each `memory/agents/<name>/learnings.md` follows this structure:

```markdown
# Learnings: <name>

## Codebase Patterns
- API v2 uses joi for validation, not zod (added: 2026-02-13)
- All database models inherit from BaseModel in src/models/base.py (added: 2026-02-10)

## Gotchas
- TrustService requires bootstrap() before first call (added: 2026-02-13)
- Auth middleware must come before CORS in middleware chain (added: 2026-02-11)

## Preferences
- User prefers explicit error types over generic Error (added: 2026-02-13)
- Always add docstrings for public API methods (added: 2026-02-10)

## Cross-Agent Notes
- (from architect) Use TrustLevel enum from src/core/types.ts (added: 2026-02-13)
- (from tester) Backend endpoints need request/response examples in tests (added: 2026-02-12)
```

### Category Definitions

| Category | Purpose | Examples |
|----------|---------|----------|
| **Codebase Patterns** | Confirmed conventions, architectural rules, established patterns | "All async functions use asyncio, not threads", "Config loaded from .env via pydantic Settings" |
| **Gotchas** | Bugs, quirks, workarounds that cost time to discover | "DB migrations fail if run twice", "ImageProcessor needs 2GB RAM minimum" |
| **Preferences** | User/project style preferences for approach, tooling, style | "Prefer composition over inheritance", "Use pytest fixtures not unittest mocks" |
| **Cross-Agent Notes** | Knowledge forwarded from other team members | "(from frontend) API expects camelCase not snake_case", "(from devops) Deploy script now requires Docker 24+" |

### Dating Entries

**Every entry must include a date**: `(added: YYYY-MM-DD)`

This enables:
- Staleness tracking (entries >21 days with no references)
- Learning velocity analysis (how fast is the agent learning?)
- Pruning decisions (recent entries are more likely to be relevant)

### Tiered Structure

Learnings files use a two-tier structure:

- **Core (30 lines max)**: High-reuse fundamentals that apply to every task. These are patterns referenced 5+ times across sprints.
- **Task-Relevant (30 lines max)**: Context-specific entries that apply to specific scenarios. Prune these more aggressively.

**Total cap: 60 lines.**

Why the cap?
- Keeps learnings focused and high-signal
- Prevents prompt bloat (learnings are injected at spawn time)
- Forces consolidation and archival of stale knowledge

---

## Pruning & Maintenance

### When to Prune

Run `/retro` (retrospective skill) to prune learnings:
- At the end of a work session
- When a learnings file exceeds 50 lines (warning threshold)
- When you notice stale entries (>21 days old with no recent references)
- When cross-agent notes have piled up without being acknowledged

### Consolidation Triggers

When a learnings file exceeds 60 lines or meets any of these conditions, consolidate:

#### 1. Merge Similar Entries

Combine entries that say the same thing in different ways.

**Before:**
```markdown
- API uses joi validation (added: 2026-02-10)
- All validation done with joi library (added: 2026-02-13)
```

**After:**
```markdown
- API validation uses joi library across all endpoints (added: 2026-02-10, confirmed: 2026-02-13)
```

#### 2. Archive Stale Entries

Move entries older than 21 days (with no recent references) to `memory/agents/<name>/archive.md`.

Create the archive file if it doesn't exist:

```markdown
# Archive: <name>

Learnings that were once relevant but are no longer active. Preserved for historical context.

---

## Archived: 2026-02-13
- Old validation library was jsonschema (added: 2026-01-15, archived: 2026-02-13, reason: migrated to joi)
```

#### 3. Promote High-Value Entries

If a learning has been confirmed across 3+ sprints, promote it to:
- `.claude/rules/` (if it's a rule all agents should follow)
- `CLAUDE.md` (if it's fundamental project context)
- `memory/team/decisions.md` (if it's a team-wide architectural decision)

**Example promotion:**

Learnings file had:
```markdown
- All dates stored as ISO 8601 UTC (added: 2026-02-01, confirmed in 8 sprints)
```

Promoted to `memory/team/decisions.md`:
```markdown
## Conventions
- All dates stored as ISO 8601 UTC (decided: 2026-02-01, by: backend)
```

And removed from individual learnings files (now global knowledge).

#### 4. Validate Cross-Agent Notes

Cross-agent notes older than 14 days must be:
- **Acknowledged**: Merged into another section ("Yes, I'm aware of this pattern")
- **Acted upon**: Integrated into the agent's workflow
- **Discarded**: Moved to archive with rationale if not relevant

**Example acknowledgment:**

Before:
```markdown
## Cross-Agent Notes
- (from backend) User list endpoint now supports pagination (added: 2026-02-01)
```

After (merged into Codebase Patterns):
```markdown
## Codebase Patterns
- User list endpoint supports ?page=N&per_page=M for pagination (added: 2026-02-13, from: backend)

## Cross-Agent Notes
- (none pending)
```

### Manual Pruning Steps

If you're pruning manually (not using `/retro`):

1. **Read the learnings file** and identify candidates for each consolidation type
2. **Create archive file** if needed: `memory/agents/<name>/archive.md`
3. **Edit the learnings file**:
   - Merge similar entries
   - Move stale entries to archive
   - Promote high-value entries to global files
   - Acknowledge or discard old cross-agent notes
4. **Verify line count**: Ensure file is under 60 lines
5. **Commit**: The pruning itself is a learning event worth recording in git

---

## Cross-Agent Knowledge Sharing

### How Cross-Agent Notes Work

When an agent discovers something relevant to another team member, they suggest a cross-agent learning in their reflection:

```json
{
  "category": "cross-agent",
  "content": "User API now requires authentication header",
  "for_agent": "frontend"
}
```

The orchestrator:
1. Adds this to the **source agent's** Codebase Patterns (as a confirmed learning)
2. Adds this to the **target agent's** Cross-Agent Notes (as a pending notification)

### Target Agent Sees the Note

Next time the frontend agent spawns, they see:

```markdown
## Cross-Agent Notes
- (from backend) User API now requires authentication header (added: 2026-02-13)
```

The frontend agent should:
- Read and understand the note
- Update their code accordingly
- Acknowledge the note (merge it into their own Codebase Patterns or Gotchas)

### Acknowledgment Flow

During the next sprint involving frontend:

**Frontend's reflection:**
```json
{
  "category": "codebase-pattern",
  "content": "User API requires Authorization: Bearer <token> header"
}
```

**Orchestrator action:**
- Adds to frontend's Codebase Patterns
- **Removes the note from Cross-Agent Notes** (now acknowledged)

### Why 14-Day Validation Window?

Cross-agent notes are meant to be **action triggers**, not permanent storage. If a note sits unacknowledged for 14+ days:
- Either the target agent hasn't been dispatched (inactive role)
- Or the note isn't relevant (should be discarded)
- Or the note was missed (manual review needed)

This keeps Cross-Agent Notes sections clean and actionable.

### Team-Wide Decisions

Some knowledge benefits the entire team, not just one member. Use `memory/team/decisions.md` for:
- Architectural choices that affect everyone
- Conventions adopted team-wide
- Major technology decisions

Example:

```markdown
# Team Decisions

## Architecture
- REST over GraphQL for all public APIs (decided: 2026-02-13, by: architect)
- Microservices communicate via RabbitMQ message bus (decided: 2026-02-10, by: architect)

## Conventions
- All dates stored as ISO 8601 UTC (decided: 2026-02-01, by: backend)
- Error responses use RFC 7807 Problem Details format (decided: 2026-02-05, by: backend)
```

---

## Workflow Integration

The team system integrates with four primary skills: `/assemble`, `/standup`, `/sprint`, and `/retro`.

### `/assemble` — Team Creation

**When to use:** At the start of a new project or when upgrading from ad-hoc dispatch to structured teams.

**What it does:**
1. Explores your project structure
2. Proposes roles and ownership patterns
3. Creates `.claude/team.yaml`
4. Creates `memory/agents/<name>/learnings.md` for each member
5. Creates `memory/team/decisions.md` and `memory/team/retro-history.md`
6. Optionally initializes a beads backlog

**Output:** A fully configured team ready for dispatch.

**Example:**
```
/assemble Web app with FastAPI backend and React frontend
```

### `/standup` — Status Sync

**When to use:**
- At the start of a work session to get oriented
- After a break or context switch
- When you need a quick view of blockers and learning health

**What it does:**
1. Reads team manifest and learnings files
2. Checks git activity by ownership patterns
3. Checks backlog (if beads available)
4. Reports per-member status and learning health

**Output:** A status board showing team activity, learning health, blockers, and suggested actions.

**Example:**
```
/standup
```

Sample output:

```markdown
## Standup: my-project

### Team Activity
| Member | Recent Commits | Learnings | Health |
|--------|---------------|-----------|--------|
| backend | 3 in owned files | 12 entries (2 new) | 🟢 healthy |
| frontend | 1 in owned files | 8 entries (1 new) | 🟢 healthy |
| tester | 0 in owned files | 5 entries (0 new) | 🟡 stale |

### Backlog Snapshot
- **Ready**: 3 tasks available
- **In Progress**: 1 task active
- **Blocked**: 0 tasks blocked

### Blockers
No blockers

### Learning Highlights
- backend: Pagination pattern now standardized across all list endpoints
- frontend: API client auto-retries on 429 rate limits

### Cross-Agent Notes Pending
- frontend has 1 note from backend (added: 2026-02-13)

### Suggested Actions
1. Dispatch next ready task to tester (stale member needs activation)
2. Review frontend's cross-agent note during next sprint
```

### `/sprint` — Plan and Dispatch

**When to use:**
- When you have work to dispatch to team members
- When you want the full learning loop (spawn → work → reflect → persist)

**What it does:**
1. Loads team manifest, learnings, and backlog
2. Auto-assigns tasks to members by ownership and role
3. Presents sprint plan for approval
4. Dispatches members serially (default) or in parallel
5. Parses reflections and updates learnings files
6. Reports progress and learning summary

**Output:** Completed tasks + updated learnings files.

**Example with beads:**
```
/sprint
```

**Example without beads:**
```
/sprint Add user authentication
```

Sample sprint plan:

```markdown
## Sprint Plan

| Bead | Title | Assigned To | Reason |
|------|-------|-------------|--------|
| 42 | Add user authentication endpoint | backend | Ownership: src/api/** |
| 43 | Create login form component | frontend | Ownership: src/components/** |
| 44 | Add auth integration tests | tester | Ownership: tests/** |

### Dispatch Order
1. Bead 42: backend — Foundation layer (auth endpoint must exist before UI/tests)
2. Bead 43: frontend — UI layer (depends on endpoint contract)
3. Bead 44: tester — Validation layer (depends on both endpoint and UI)

**Strategy**: Serial dispatch (each task benefits from the previous one's learnings)
```

After user approval, `/sprint` dispatches each task, parses reflections, updates learnings, and reports:

```markdown
### backend: Add user authentication endpoint
**Status**: completed | **Confidence**: high
**Summary**: Implemented JWT-based auth endpoint with refresh tokens
**Learnings persisted**: 3 new entries (2 patterns, 1 cross-agent note to frontend)
**Next**: Dispatching frontend task

### frontend: Create login form component
**Status**: completed | **Confidence**: high
**Summary**: Built LoginForm with email/password fields, connected to auth endpoint
**Learnings persisted**: 2 new entries (1 pattern, 1 cross-agent note to tester)
**Next**: Dispatching tester task

...
```

### `/retro` — Session Retrospective

**When to use:**
- At the end of a work session
- After completing an epic
- When learnings files are approaching the 60-line cap
- When you want to reflect on what worked and what didn't

**What it does:**
1. Gathers session data (git, backlog, conversation context)
2. Analyzes across 5 dimensions (velocity, quality, process, blockers, discoveries)
3. Extracts keep/stop/try learnings
4. Prunes bloated learnings files (if team exists)
5. Updates project memory (MEMORY.md)
6. Appends to `memory/team/retro-history.md`

**Output:** Structured retrospective report + pruned learnings files + updated memory.

**Example:**
```
/retro
```

Sample output:

```markdown
## Session Retrospective

### Summary
Completed 8 tasks across 3 team members with 12 new learnings persisted. Serial dispatch strategy worked well — no rework needed.

### What Went Well
- All 8 tasks completed on first attempt with high confidence ratings
- Backend → frontend → tester dispatch order prevented dependency blockers
- Cross-agent notes effectively communicated API changes before downstream work started

### What Could Improve
- Tester member went stale (5 days no new learnings) — needs richer tasks or role might be underutilized
- Frontend learnings file at 58 lines, approaching cap — should prune in next retro

### Action Items
- [ ] Prune frontend learnings file next session (consolidate 3 similar validation entries)
- [ ] Assign tester a spike task to explore edge cases and generate fresh learnings

### Memory Updates
- Added: "Serial dispatch with clear dependency order prevents rework"
- Updated: "Cross-agent notes are action triggers — 14-day validation window keeps them relevant"

### Team Learning Health
| Member | Entries | Recent | Status | Action Needed |
|--------|---------|--------|--------|--------------|
| backend | 12 | 3 | growing | none |
| frontend | 58 | 2 | steady | prune next session |
| tester | 5 | 0 | stale | assign richer tasks |

**Cross-agent notes delivered this session**: 4
**Entries pruned/archived**: 0
**Entries promoted to rules**: 0
```

### Workflow Cycle

The complete team workflow cycle:

```
/assemble
  → Create team manifest + learnings files
    → /standup (optional: get oriented)
      → /sprint
        → Dispatch work, agents learn, learnings persist
          → /standup (optional: check progress)
            → /sprint (more work)
              → /retro
                → Prune learnings, update memory, reflect on session
                  → Session ends
                    → Next session: /standup → /sprint → ... (learnings carry forward)
```

---

## Working Without Beads

The team system **does not require beads** (the `bd` CLI backlog tracker). All team skills support a fallback mode for projects without beads.

### How Skills Adapt

| Skill | With Beads | Without Beads |
|-------|-----------|---------------|
| `/assemble` | Optionally creates initial epic | Skips backlog initialization |
| `/standup` | Shows backlog snapshot (ready, in-progress, blocked) | Shows only git activity + learning health |
| `/sprint` | Pulls tasks from `bd ready` | Accepts manual task descriptions from user |
| `/retro` | Includes backlog stats in analysis | Relies on git + conversation context only |

### Manual Task Dispatch

If you don't have beads, you provide task descriptions directly to `/sprint`:

```
/sprint Add pagination to user listing endpoint
```

Sprint will:
1. Match the task to a team member by role/ownership
2. Ask for your approval
3. Dispatch as normal
4. Persist learnings as normal

The learning loop works identically — beads just provide backlog structure and task tracking.

### When to Use Beads

**Use beads if:**
- You want structured backlog management
- You need dependency tracking between tasks
- You want to track task status (ready, in-progress, blocked, closed)
- You're managing 10+ tasks across sessions

**Skip beads if:**
- You have <5 tasks total
- Tasks are simple and don't have dependencies
- You prefer freeform task descriptions
- You're prototyping and the backlog is fluid

---

## Troubleshooting & Tips

### Common Issues

#### "Learnings file is empty or has no useful content"

**Cause:** Agent didn't suggest learnings, or orchestrator didn't persist them.

**Fix:**
- Check that the agent's reflection included `suggested_learnings`
- Manually add 2-3 seed learnings based on your knowledge of the codebase
- Next sprint should accumulate more

#### "Agent keeps making the same mistake"

**Cause:** The gotcha isn't in learnings, or learnings file wasn't injected.

**Fix:**
- Verify the gotcha is documented in `memory/agents/<name>/learnings.md` under Gotchas
- Check that the dispatch prompt includes the learnings (read the Task prompt)
- Add the gotcha manually if the agent didn't suggest it

#### "Learnings file is bloated (>60 lines)"

**Cause:** Too many learnings accumulated without pruning.

**Fix:**
- Run `/retro` to trigger automatic pruning
- Manually consolidate similar entries
- Archive stale entries (>21 days) to `memory/agents/<name>/archive.md`
- Promote high-value entries to `.claude/rules/` or `CLAUDE.md`

#### "Cross-agent note is stale (>14 days)"

**Cause:** Target agent hasn't been dispatched, or note wasn't acknowledged.

**Fix:**
- If the target agent is active, manually merge the note into their Codebase Patterns
- If the target agent is inactive, discard the note (move to archive with reason)
- If the note is outdated, remove it

#### "Team member has wrong tools/ownership"

**Cause:** Team manifest needs updating.

**Fix:**
- Edit `.claude/team.yaml` to fix the member definition
- No need to recreate learnings files — they persist independently

### Best Practices

#### Start Small

Begin with 3-4 roles. Add roles when you feel a gap, not preemptively.

#### Seed Learnings

Don't leave learnings files empty after `/assemble`. Add 2-3 initial observations based on your project exploration so agents start with useful context.

#### Date Everything

Always include `(added: YYYY-MM-DD)` on new learnings. This enables staleness tracking and pruning.

#### Serialize by Default

Dispatch one task at a time unless tasks are truly independent and touch different ownership areas. Serial dispatch lets each agent benefit from the previous one's learnings.

#### Review Learnings in PRs

Learnings files are version-controlled. Review them in PRs to ensure quality and catch duplicates or low-value entries.

#### Run `/retro` Regularly

At the end of each session or epic, run `/retro` to prune bloated files and extract durable insights.

#### Use Cross-Agent Notes Sparingly

Cross-agent notes are for **action triggers**, not documentation. If a pattern is relevant to multiple agents, promote it to `memory/team/decisions.md` instead.

#### Promote High-Value Learnings

If a learning has been confirmed across 3+ sprints, promote it to `.claude/rules/` or `CLAUDE.md` so all agents (not just one role) benefit.

### Advanced Patterns

#### Role Specialization

Some projects benefit from specialized roles beyond backend/frontend/tester:

- **security**: Auth, input validation, threat modeling
- **performance**: Profiling, optimization, caching
- **docs**: Documentation, API specs, user guides
- **infra**: CI/CD, deployment, monitoring

#### Dynamic Role Addition

You can add new roles mid-project:

1. Add the member to `.claude/team.yaml`
2. Create `memory/agents/<name>/learnings.md`
3. Seed with relevant context from existing members
4. Dispatch via `/sprint`

#### Learning Transfer

When a team member leaves or a role is deprecated:

1. Read their learnings file
2. Distribute cross-agent notes to relevant remaining members
3. Promote high-value learnings to team decisions or rules
4. Archive the learnings file (keep for historical context)

#### Team Templates

For common project types (e.g., "web app", "CLI tool", "data pipeline"), create team templates in `templates/teams/`:

```yaml
# templates/teams/web-app.yaml
team: web-app-template
description: "Full-stack web application with API + frontend"

defaults:
  model: sonnet
  budget: 0.50
  permission-mode: dontAsk

members:
  - name: backend
    role: "Server logic, data models, business rules"
    tools: [Read, Write, Edit, Grep, Glob, "Bash(git:*)"]
    owns: ["src/api/**", "src/models/**", "tests/api/**"]

  - name: frontend
    role: "UI components, state management, UX"
    tools: [Read, Write, Edit, Grep, Glob, "Bash(git:*)"]
    owns: ["src/components/**", "src/pages/**", "tests/ui/**"]

  - name: tester
    role: "Test strategy, coverage, edge cases"
    tools: [Read, Write, Edit, Grep, Glob, "Bash(git:*)"]
    owns: ["tests/**"]
```

Copy and customize for each new project.

---

## Summary

The persistent learning team system transforms agent dispatch from stateless execution to **cumulative knowledge building**. Agents remember what they learn, share insights across roles, and improve over time.

**Key takeaways:**

1. **Teams persist across sessions** via version-controlled files
2. **Learnings accumulate** in `memory/agents/<name>/learnings.md`
3. **The learning loop** ensures agents get smarter with every task
4. **Cross-agent notes** route knowledge between team members
5. **Pruning** keeps learnings focused and high-signal
6. **Four skills** integrate the system: `/assemble`, `/standup`, `/sprint`, `/retro`
7. **Beads are optional** — teams work with or without backlog tracking

**Next steps:**

- Run `/assemble` to create your first team
- Dispatch a few tasks via `/sprint` and watch learnings accumulate
- Run `/standup` to check learning health
- Run `/retro` at the end of your session to prune and reflect

**Welcome to team-based agent coordination with persistent learning.**
