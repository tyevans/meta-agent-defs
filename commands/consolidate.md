# Consolidate: Backlog Review and Tightening

You are running the **Consolidate** workflow — a structured pass over the current beads backlog to eliminate sprawl, fill gaps, and sharpen priorities. Target area (optional): **$ARGUMENTS**

## When to Use

- After a blossom run produced a large backlog
- When the backlog has accumulated over multiple sessions and feels unwieldy
- Before starting a new sprint of work — get a clean picture first
- When you suspect duplicate or stale tasks exist

## Overview

Consolidate works in 6 phases:

```
Survey the backlog
  -> Dedup (merge overlapping tasks)
    -> Vertical slice audit (fill gaps)
      -> Stale detection (close dead tasks)
        -> Dependency cleanup (simplify the DAG)
          -> Report changes
```

---

## Phase 1: Survey

### 1a. Get the Full Picture

```bash
bd stats
bd list --status=open
bd list --status=in_progress
bd blocked
```

### 1b. Build a Mental Map

For each open task, note:
- **Scope**: What code/area does this touch?
- **Status**: Is anyone working on it? Is it blocked?
- **Staleness**: When was it last updated? Is it still relevant?
- **Connections**: What does it block or depend on?

If `$ARGUMENTS` specifies a target area (e.g., "sandbox", "frontend"), focus only on tasks related to that area. Otherwise, review the full backlog.

### 1c. Categorize

Group tasks into clusters by the area they touch (e.g., "domain/sandbox", "infrastructure", "CLI", "frontend"). This grouping drives the rest of the workflow.

---

## Phase 2: Dedup

### 2a. Find Overlaps

Within each cluster, look for:
- **Exact duplicates**: Same work described differently
- **Subset tasks**: One task is entirely contained within another
- **Convergent tasks**: Different approaches to the same problem

### 2b. Merge Strategy

For each overlap found:

1. **Exact duplicates**: Close the newer one with a note pointing to the older one
   ```bash
   bd close <newer-id> --reason="Duplicate of <older-id>"
   ```

2. **Subset tasks**: Close the subset, add its specifics as notes on the parent
   ```bash
   bd update <parent-id> --notes="Absorbed <subset-id>: [specific details]"
   bd close <subset-id> --reason="Absorbed into <parent-id>"
   ```

3. **Convergent tasks**: Keep the better-scoped one, close the other
   ```bash
   bd close <weaker-id> --reason="Superseded by <better-id>"
   ```

### 2c. Report Dedup Results

Track: how many tasks closed, how many merged, how many clusters remain.

---

## Phase 3: Vertical Slice Audit

### 3a. Discover the Project's Architecture

Before auditing slices, identify the project's actual architectural layers. Read the project structure, CLAUDE.md, and top-level directories to determine the pattern in use. Common patterns:

| Architecture | Typical Layers |
|-------------|---------------|
| DDD / Hexagonal | Domain, Infrastructure, Application, Interface |
| MVC | Model, View, Controller, Service |
| Component-based | Feature modules (auth, billing, etc.) with internal structure |
| Flat / scripts | Modules, utilities, entry points |
| Frontend | Components, State/Store, API layer, Routing |

Record the discovered layers before proceeding. If the architecture is unclear, list the top-level directories and use those as the layer categories.

### 3b. Check Each Task for Slice Completeness

For every task that touches a core/inner layer, verify companion tasks exist across the relevant architectural boundaries. The specific companions depend on the architecture discovered in 3a:

- **New core logic** → Does a data access / persistence task exist?
- **New data access** → Does a wiring / bootstrapping task exist?
- **New service / use case** → Does at least one exposure point (CLI, API, UI) exist?
- **New events / messages** → Does at least one consumer task exist?
- **New exposure point** → Does the underlying logic task exist?
- **Any new feature** → Does a test task exist?

### 3c. Fill Gaps

For each missing companion:

```bash
bd create --title="[layer]: [companion task description]" --type=task --priority=<same-as-parent>
bd dep add <epic-or-parent> <new-task-id>
```

### 3d. Wire Dependencies

Ensure the natural flow follows the project's dependency direction (inner layers before outer layers). General pattern:

- Core/model tasks block data-access/infrastructure tasks
- Data-access tasks block service/application tasks
- Service tasks block interface/exposure tasks
- Test tasks can run in parallel with their layer

```bash
# Example: outer layer depends on inner layer
bd dep add <outer-task> <inner-task>
```

---

## Phase 4: Stale Detection

### 4a. Identify Stale Tasks

A task is potentially stale if:
- It was created more than 2 weeks ago with no progress
- Its parent epic has been closed or abandoned
- The code area it targets has been significantly refactored since creation
- It describes work that has already been done (check git log)

### 4b. Verify Before Closing

For each potentially stale task:
1. **Check git log** — has this work already been done?
   ```bash
   git log --oneline --all --grep="<relevant keywords>"
   ```
2. **Check the code** — does the issue still exist?
3. **Check dependencies** — would closing this unblock anything?

### 4c. Close Stale Tasks

```bash
bd close <stale-id> --reason="Stale: [explanation]"
```

Or if the work was already done:
```bash
bd close <stale-id> --reason="Already completed in commit <hash>"
```

---

## Phase 5: Dependency Cleanup

### 5a. Remove Redundant Transitive Dependencies

If A depends on B, and B depends on C, then A does NOT need a direct dependency on C. The transitive dependency through B is sufficient.

```bash
# Check: does A directly depend on C when B already bridges them?
bd show <A>  # Look at depends-on list
# If A -> B -> C AND A -> C, remove A -> C
```

### 5b. Check for Cycles

Verify no circular dependencies exist:
- A depends on B, B depends on A (direct cycle)
- A depends on B, B depends on C, C depends on A (indirect cycle)

If cycles are found, break them by identifying which dependency is the weakest (most optional) and removing it.

### 5c. Validate Epic Structure

Every epic should:
- Depend on all its child tasks (epic completes when children complete)
- NOT be depended on by its children (children don't wait for the epic)

```bash
bd show <epic-id>  # Verify "depends on" lists children, "blocks" is empty or external
```

---

## Phase 6: Report

### 6a. Summarize Changes

Present a clear summary:

```markdown
## Consolidation Report

### Actions Taken
- **Closed (dedup)**: X tasks merged or deduplicated
- **Closed (stale)**: X tasks no longer relevant
- **Created (gap fill)**: X companion tasks added for vertical slices
- **Dependencies**: X redundant deps removed, X new deps added

### Backlog Health
- **Before**: X open tasks, Y blocked, Z in-progress
- **After**: X open tasks, Y blocked, Z in-progress
- **Clusters**: [list of task clusters with counts]

### Recommendations
- [Any observations about backlog health]
- [Suggested next priorities]
- [Areas that might need a fresh blossom spike]
```

### 6b. Sync

```bash
bd sync
```

---

## Guidelines

- **Be conservative with closures** — when in doubt, keep a task open and add a note
- **Check the code** before declaring something stale — read, don't guess
- **Preserve context** — when closing or merging, always explain why in the reason
- **Don't reprioritize aggressively** — small adjustments only, unless the user directs otherwise
- **Focus on the target area** if `$ARGUMENTS` was provided — don't let scope creep into unrelated clusters
