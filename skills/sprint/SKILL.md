---
name: sprint
description: "Plan and dispatch work to team members with the learning loop: assign tasks by ownership, spawn agents with injected learnings, parse reflections, and persist new learnings. Works with or without beads backlog tracking. Use when you have a team assembled and work to dispatch. Keywords: dispatch, execute, work, plan, assign, run, do, build."
argument-hint: "[task filter or focus area]"
disable-model-invocation: true
user-invocable: true
allowed-tools: Read, Write, Edit, Grep, Glob, Bash(bd:*), Bash(git:*), Bash(claude:*), Bash(mkdir:*), Task
---

# Sprint: Plan, Dispatch, Learn

You are running a **Sprint** -- the core execution loop for a persistent learning team. Sprint assigns tasks to team members, dispatches them with accumulated learnings, parses their reflections, and persists new learnings. Over time, agents effectively improve because their learnings grow.

**Focus area (optional):** $ARGUMENTS

## When to Use

- When you have a team assembled (via `/assemble`) and work to dispatch
- When you want to dispatch work to team members with the learning loop
- When agents should accumulate knowledge from each task to improve on the next
- After a planning session (like `/blossom`) produces tasks to execute
- When you want structured reflection and learning persistence after each task
- Works with beads backlog (pulls from `bd ready`) or without (accepts manual task descriptions)

## The Learning Loop

```
Load manifest + learnings + backlog
  → Auto-assign tasks to members by ownership
    → User approves assignments
      → Dispatch member via spawn protocol (serialize)
        → Parse reflection JSON
          → Update member's learnings.md
            → Dispatch next → Sprint report
```

---

## Phase 1: Load Context

### 1a. Read Team Manifest

Read `.claude/team.yaml`. If it doesn't exist, tell the user to run `/assemble` first and stop.

### 1b. Read All Learnings

For each member, read `memory/agents/<name>/learnings.md`. Note the current size (line count) and most recent entries.

### 1c. Load Backlog (conditional)

**If `.beads/` exists in the project root**, check the backlog:

```bash
bd ready
bd list --status=in_progress
bd blocked
```

If a focus area was provided via `$ARGUMENTS`, filter to relevant beads.

**If `.beads/` does not exist**, skip this step. The sprint will accept manual task descriptions from the user instead of pulling from the backlog.

---

## Phase 2: Plan Assignments

### 2a. Match Tasks to Members

**If beads are available**, for each ready bead, determine the best-fit member by:
1. **Ownership match**: Does the bead's likely file scope overlap with a member's `owns` patterns?
2. **Role match**: Does the bead's topic align with a member's role description?
3. **Learning advantage**: Has a member accumulated relevant learnings for this type of task?

**If beads are not available**, ask the user to describe the tasks they want dispatched, then match tasks to members based on role and ownership fit.

### 2b. Present Sprint Plan

```markdown
## Sprint Plan

| Bead | Title | Assigned To | Reason |
|------|-------|-------------|--------|
| [id] | [title] | [member] | [ownership match / role match / learning advantage] |
| ... | ... | ... | ... |

### Dispatch Order
1. [bead-id]: [member] — [brief reason for ordering]
2. [bead-id]: [member] — [brief reason]
...

**Strategy**: Serial dispatch (each task benefits from the previous one's learnings)
```

Ask the user to approve, reorder, reassign, or remove tasks.

---

## Phase 3: Dispatch

For each approved assignment, in the approved order:

### 3a. Compose the Spawn

Build the `claude -p` command following the spawn protocol from team-protocol.md:

1. Read the member's current `learnings.md`
2. Construct the `--append-system-prompt` with:
   - Team member identity (name, role, owns)
   - Full contents of their learnings.md
   - Reflection protocol instructions
3. Construct the task prompt with:
   - Bead ID and title
   - Bead description (from `bd show <id>`)
   - Any relevant context from previous dispatches in this sprint

### 3b. Execute

**Default: use Task tool dispatch** (in-process, simpler):

```
Task({
  subagent_type: "general-purpose",
  model: "<member's model>",
  prompt: "<composed prompt including learnings and reflection instructions>"
})
```

The prompt must include:
- The member's role and ownership context
- Their full learnings file contents
- The task description with bead reference
- Reflection instructions: "After completing your task, provide a structured summary with: task_result (status, summary, files_changed), reflection (what_worked, what_didnt, confidence), suggested_learnings (category, content, for_agent), and follow_up (blocked_by, suggested_next, needs_human)."

**Alternative: `claude -p` dispatch** (out-of-process, for budget enforcement):

Use the full spawn command template from team-protocol.md when:
- Budget enforcement is needed (`--max-budget-usd`)
- JSON schema enforcement is needed (`--json-schema`)
- The task is complex enough to warrant a full CLI session

### 3c. Mark Bead In-Progress (conditional)

**If beads are available**:

```bash
bd update <bead-id> --status=in_progress
```

**If beads are not available**, skip this step.

---

## Phase 4: Process Results

After each dispatch returns:

### 4a. Parse the Response

Extract from the agent's output:
- **task_result**: status, summary, files changed
- **reflection**: what worked, what didn't, confidence level
- **suggested_learnings**: new learnings to persist
- **follow_up**: blockers, suggested next steps, human input needed

If the agent returned unstructured text (Task tool dispatch), extract these fields by interpreting the output. Look for patterns like "I completed...", "I struggled with...", "I learned that...".

### 4b. Update Learnings

For each suggested learning:

1. **Validate**: Is this learning durable (useful across sessions) or ephemeral (only relevant now)?
2. **Categorize**: Map to the correct section (Codebase Patterns, Gotchas, Preferences, Cross-Agent Notes)
3. **Append**: Add to `memory/agents/<name>/learnings.md` with today's date
4. **Route cross-agent**: If `for_agent` is specified, also add to that agent's learnings under "Cross-Agent Notes" prefixed with `(from <source-agent>)`

```markdown
## Gotchas
- TrustService requires bootstrap before first call (added: 2026-02-13)
```

### 4c. Update Bead (conditional)

**If beads are available**, based on `task_result.status`:
- **completed**: `bd close <bead-id>`
- **partial**: Keep in-progress, note what remains
- **blocked**: `bd update <bead-id> --notes="Blocked: <blocked_by>"`
- **failed**: `bd update <bead-id> --notes="Failed: <summary>"`

**If beads are not available**, skip bead updates and simply note the task status in the sprint report.

### 4d. Report Progress

After each task, show a brief progress update:

```markdown
### [member]: [bead-title]
**Status**: [status] | **Confidence**: [confidence]
**Summary**: [1-2 sentences]
**Learnings persisted**: [count] new entries
**Next**: [dispatching next task / sprint complete / blocked]
```

### 4e. Dispatch Next

If there are more tasks in the sprint, proceed to 3a for the next assignment. The next agent will benefit from any learnings just persisted.

---

## Phase 5: Sprint Report

After all tasks are dispatched (or the sprint is stopped):

```markdown
## Sprint Report

### Completed
| Bead | Member | Status | Confidence | Learnings |
|------|--------|--------|------------|-----------|
| [id] | [name] | [status] | [confidence] | [count] new |
| ... | ... | ... | ... | ... |

### Learning Summary
- **Total new learnings**: [count] across [members] members
- **Cross-agent notes delivered**: [count]
- **Notable learnings**: [2-3 most significant new learnings]

### Blockers Encountered
[List any blocked or failed tasks with their blockers. "None" if clean sprint.]

### Suggested Next Sprint
[Based on follow_up.suggested_next from all tasks, recommend what to tackle next.]
```

---

## Guidelines

1. **Serialize by default.** Dispatch one task at a time so each agent benefits from the previous one's learnings. Only parallelize if tasks are truly independent AND touch different ownership areas.
2. **Learnings are the product.** A sprint that completes tasks but doesn't persist learnings has wasted half its value. Always update learnings files.
3. **Validate learnings.** Don't blindly append everything an agent suggests. Filter out ephemeral observations and duplicates.
4. **Date every entry.** Always add `(added: YYYY-MM-DD)` to new learnings for staleness tracking.
5. **Watch for bloat.** If a learnings file exceeds 120 lines during a sprint, flag it for pruning in the next `/retro`.
6. **Context carries forward.** When dispatching the second+ task in a sprint, include a brief summary of what previous agents found/changed.
