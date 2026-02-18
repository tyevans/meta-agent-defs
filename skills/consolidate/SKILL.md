---
name: consolidate
description: "Review and tighten the beads backlog by deduplicating tasks, filling vertical slice gaps, detecting stale items, and cleaning up dependencies. Use after a blossom run, when the backlog feels unwieldy, before starting a new sprint, or when you suspect duplicate or stale tasks. Keywords: backlog, cleanup, dedup, triage, hygiene, review, organize."
argument-hint: "[area or scope]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*)
context: fork
---

!`bd list --status=open 2>/dev/null`

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
Survey [gather]
  -> Dedup [filter: not duplicate]
    -> Vertical slice audit [assess: complete/missing-companion]
      -> Stale detection [filter: not stale]
        -> Dependency cleanup [verify]
          -> Report [pipe format]
```

---

## Phase 1: Survey *(gather pattern — collect backlog info with sources)*

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

## Phase 2: Dedup *(filter pattern — criterion: "not duplicate")*

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

## Phase 3: Vertical Slice Audit *(assess pattern — rubric: complete / missing-companion)*

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

For each missing companion, create as a child of the epic (if one exists) or standalone:

```bash
bd create --title="[layer]: [companion task description]" --type=task --priority=<same-as-parent> \
  --parent=<epic-id>
```

If no epic exists, omit `--parent` and wire ordering dependencies with `bd dep add` as needed.

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

## Phase 4: Stale Detection *(filter pattern — criterion: "not stale")*

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

## Phase 5: Dependency Cleanup *(verify pattern — check dependency validity)*

### 5a. Remove Redundant Transitive Dependencies

If A depends on B, and B depends on C, then A does NOT need a direct dependency on C. The transitive dependency through B is sufficient.

```bash
# Check: does A directly depend on C when B already bridges them?
bd show <A>  # Look at depends-on list
# If A -> B -> C AND A -> C, remove A -> C
```

### 5b. Check for Cycles and Validate Structure

Use `bd` built-in validation instead of manual checks:

```bash
bd dep cycles                  # Detect dependency cycles
bd swarm validate <epic-id>    # Full structural validation (cycles, orphans, disconnected subgraphs, ready fronts)
```

If cycles are found, break them by identifying which dependency is the weakest (most optional) and removing it. If `bd swarm validate` reports orphaned or disconnected tasks, re-parent or wire them.

### 5c. Validate Epic Structure

Every epic should have its tasks as children (via `--parent`), not just blocking deps:

```bash
bd children <epic-id>  # Should list all child tasks
bd epic status         # Should show completion progress
```

If tasks were created without `--parent`, they will be missing from `bd children`. Flag these for re-parenting.

---

## Phase 6: Report *(pipe format output)*

### 6a. Summarize Changes

Emit the consolidation report in pipe format:

```markdown
## Consolidated Backlog for [area or "full backlog"]

**Source**: /consolidate
**Input**: [target area or "full backlog review"]

### Items

1. **Closed (dedup)** — X tasks merged or deduplicated
   - source: Phase 2
2. **Closed (stale)** — X tasks no longer relevant
   - source: Phase 4
3. **Created (gap fill)** — X companion tasks added for vertical slices
   - source: Phase 3
4. **Dependencies cleaned** — X redundant deps removed, X new deps added
   - source: Phase 5

### Backlog Health

| Metric | Before | After |
|--------|--------|-------|
| Open tasks | X | X |
| Blocked | Y | Y |
| In-progress | Z | Z |
| Clusters | [list] | [list] |

### Recommendations

**Sharpening gate:** Each recommendation must pass three tests:
1. Name the specific bead/area/cluster
2. State what concretely should change
3. Make it actionable (a single bd command or skill invocation)

**Before:** "Some clusters might benefit from prioritization review"
**After:** "Reprioritize beads abc1, def2, ghi3 in the auth cluster from P3→P2 — they block 4 downstream tasks"

- [Recommendation 1: specific action]
- [Recommendation 2: specific action]
- [Recommendation 3: specific action]

### Summary

[One paragraph synthesis of the consolidation: what changed, current backlog health, and recommended next steps.]
```

---

## Guidelines

- **Be conservative with closures** — when in doubt, keep a task open and add a note
- **Check the code** before declaring something stale — read, don't guess
- **Preserve context** — when closing or merging, always explain why in the reason
- **Don't reprioritize aggressively** — small adjustments only, unless the user directs otherwise
- **Focus on the target area** if `$ARGUMENTS` was provided — don't let scope creep into unrelated clusters
