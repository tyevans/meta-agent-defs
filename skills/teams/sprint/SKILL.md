---
name: sprint
description: "Plan and dispatch work to team members with the learning loop: assign tasks by ownership, spawn agents with injected learnings, parse reflections, and persist new learnings. Works with or without beads backlog tracking. Use when you have a team assembled and work to dispatch. Keywords: dispatch, execute, work, plan, assign, run, do, build."
argument-hint: "[task filter or focus area]"
disable-model-invocation: false
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

## Phase 0: Session Health Gate

Before loading context and dispatching agents, check that the session has enough capacity for the sprint.

**Assess the current session inline:**
- Roughly how many files have you read this session?
- How many agents have you dispatched so far?
- How many distinct codebase areas have you touched?

| Load Level | Files Read | Agents Dispatched | Signal |
|------------|-----------|-------------------|--------|
| Light | <20 | 0-2 | Healthy — proceed |
| Moderate | 20-50 | 3-5 | Proceed, watch quality |
| Heavy | 50-100 | 6-9 | Warn user before dispatching |
| Overloaded | 100+ | 10+ | Recommend handoff |

**If load is Heavy or Overloaded:** Warn the user:

> Session context is heavy (~X files read, ~Y agents dispatched). Dispatching additional agents in this session risks degraded quality — agents may receive compressed or incomplete context. Recommend running `/handoff` to capture current state and starting a fresh session for the sprint. Continue anyway? (y/n)

Wait for user confirmation before proceeding. If the user declines, stop here and run `/handoff`.

**If load is Light or Moderate, or the user confirms they want to continue:** Proceed to Phase 1.

---

## Phase 1: Load Context

### 1a. Read Team Manifest

Read `.claude/team.yaml`. If it doesn't exist, tell the user to run `/assemble` first and stop.

### 1b. Read All Learnings

For each member, read `memory/agents/<name>/learnings.md`. Note the current size (line count) and most recent entries.

### 1c. Load Epic State (conditional)

Check for epic state files written by `/blossom`:

```bash
ls memory/epics/*/epic.md 2>/dev/null
```

If any epic state files exist, read them. These contain spike findings, priority order, task IDs, critical path, and parallel opportunities from a prior blossom run. When matching tasks to members in Phase 2, use the spike findings (confidence levels, file scopes, agent hints) and priority order from the epic state to inform assignments. If `$ARGUMENTS` names a specific epic, load only that epic's state file.

### 1d. Load Backlog (conditional)

**If `.beads/` exists in the project root**, check the backlog:

```bash
bd ready
bd list --status=in_progress
bd blocked
```

If a focus area was provided via `$ARGUMENTS`, filter to relevant beads.

**If `.beads/` does not exist**, skip this step. The sprint will accept manual task descriptions from the user instead of pulling from the backlog.

### 1e. Prerequisite Gate (STOP if incomplete)

Do not proceed to Phase 2 until all context is loaded:
- [ ] team.yaml was read (not just referenced — Read tool returned its contents)
- [ ] Every member's learnings.md was read
- [ ] Epic state files checked (loaded if present)
- [ ] Backlog state is known (bd ready output is in context, or user provided tasks)

If any prerequisite is missing, go back and complete it now. Planning without loaded context produces wrong assignments.

---

## Phase 2: Plan Assignments

### 2a. Match Tasks to Members

**If beads are available**, for each ready bead, determine the best-fit member by:
1. **Ownership match**: Does the bead's likely file scope overlap with a member's `owns` patterns?
2. **Role match**: Does the bead's topic align with a member's role description?
3. **Learning advantage**: Has a member accumulated relevant learnings for this type of task?

**If beads are not available**, ask the user to describe the tasks they want dispatched, then match tasks to members based on role and ownership fit.

### 2b. Detect Shared-File Conflicts

Before presenting the plan, scan for tasks that will likely touch the same files.

**Detection heuristic**: For each task, estimate its file scope using:
1. **Ownership patterns**: Which member is assigned? Their `owns` globs define the likely file territory.
2. **Bead description keywords**: Does the description mention specific files, classes, or systems by name?
3. **Cross-cutting signals**: Keywords like "autoload", "game_config", "project.godot", "test_launcher", or "signal" suggest files that many tasks may touch.

**Cross-reference**: Build a map of `file/pattern -> [task-ids]`. Any entry with 2+ tasks is a conflict candidate.

**In worktree mode:** When dispatching with `isolation: "worktree"` (see Phase 3b), each agent gets its own repo copy, so shared-file conflicts cannot cause overwrites or merge failures. Conflict detection is still useful for awareness — it tells the orchestrator which worktree results will need careful merging — but no resolution action or user prompt is needed. Skip the resolution options block below and proceed to 2c.

**If conflicts are found**, surface them before presenting the plan:

```markdown
### Shared-File Conflicts Detected

The following files/areas are estimated to be modified by multiple tasks:

| File / Area | Tasks | Risk |
|-------------|-------|------|
| scripts/autoload/game_config.gd | [id1], [id2] | Overwrite risk |
| project.godot (autoload section) | [id1], [id3] | Overwrite risk |
| tests/test_launcher.tscn | [id2], [id3] | Merge conflict |

**Resolution options (choose one per conflict):**
(a) **Batch** — assign all changes to that file to a single agent (recommended)
(b) **Sequence + verify** — keep agents separate, but add an explicit merge/rebuild pass after the conflicting agents complete

Which resolution do you prefer for each conflict?
```

Wait for the user's choice before proceeding. Apply the chosen resolution to the sprint plan assignments and dispatch order.

**If no conflicts are found**, proceed silently to 2c.

### 2c. Present Sprint Plan

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

### 3a. Compose the Prompt

Build the task prompt following the spawn protocol from team-protocol.md:

1. Read the member's current `learnings.md`
2. **Compute the agent identity trailer**: Run `git log -1 --format=%h -- memory/agents/<name>/learnings.md` to get the short SHA of the last commit that modified this member's learnings. The trailer value is `<name>@<sha>`. If the learnings file has no git history (new member), use `git log -1 --format=%h` to get HEAD instead.
3. Compose a prompt that includes: member identity (name, role, owns), learnings, task description, and reflection instructions
4. Include any relevant context from previous dispatches in this sprint
5. **Add commit trailer instruction**: Tell the agent that when making git commits, they must include this trailer on a new line after the commit message body:
   ```
   Agent: <computed-trailer-value>
   ```
   Make it clear the agent should use the LITERAL value provided in the prompt, not compute it themselves. This trailer goes alongside the existing `Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>` line.

### 3b. Execute

Dispatch via the Task tool. **Capture the agent ID** from the Task tool's response — it is needed to resume the agent if the task returns partial or blocked.

**With worktree isolation** (recommended for parallel dispatch): Each agent gets its own repo copy, eliminating merge conflicts between concurrent agents. Use this when dispatching independent tasks in parallel:

```
Task({
  subagent_type: "general-purpose",
  model: "<member's model from team.yaml>",
  prompt: "<composed prompt with identity, learnings, task, and reflection instructions>",
  isolation: "worktree",
  run_in_background: true
})
```

**Without worktree** (serial, default): Agent works on the shared working tree. Use this for the default serialize-by-default strategy:

```
Task({
  subagent_type: "general-purpose",
  model: "<member's model from team.yaml>",
  prompt: "<composed prompt with identity, learnings, task, and reflection instructions>"
})
```

For independent tasks that touch different ownership areas, use worktree isolation with `run_in_background: true` to parallelize safely.

**Note**: If dispatching multi-step primitive chains (3+ steps like gather->distill->rank), set `max_turns: 40` to avoid turn limits.

### 3c. Mark Bead In-Progress (conditional)

**If beads are available**:

```bash
bd update <bead-id> --status=in_progress
```

**If beads are not available**, skip this step.

---

## Phase 4: Process Results

After each dispatch returns:

**For worktree-isolated agents:** The Task tool returns the worktree path and branch name if the agent made commits. The orchestrator should review these changes (`git log` and `git diff` against the base branch in the worktree) and decide how to integrate them: cherry-pick individual commits, merge the worktree branch, or flag for manual review. Multiple worktree branches can be merged sequentially after all parallel agents complete.

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
3. **Append**: Add to `memory/agents/<name>/learnings.md` with today's date and dispatch provenance
4. **Route cross-agent**: If `for_agent` is specified, also add to that agent's learnings under "Cross-Agent Notes" prefixed with `(from <source-agent>)`

When appending entries, include the `dispatch:` field to track task provenance:

```markdown
## Gotchas
- TrustService requires bootstrap before first call (added: 2026-02-13, dispatch: bead-abc123)
```

The `dispatch:` field is optional but recommended. Use the bead ID if beads are available (e.g., `dispatch: bead-abc123`), or a brief identifier if not (e.g., `dispatch: sprint-manual`). Existing entries without dispatch provenance are backward-compatible and do not need updating.

### 4c. Update Bead (conditional)

**If beads are available**, based on `task_result.status`:
- **completed**: `bd close <bead-id>`
- **partial**: Keep in-progress, note what remains. Record the agent ID for potential resume.
- **blocked**: `bd update <bead-id> --notes="Blocked: <blocked_by>"`. Record the agent ID for potential resume.
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

**In partial/blocked status:** Add a `**Resumable**: <agent-id>` field to the progress update. This agent ID can be passed to the Task tool's `resume` parameter in a follow-up sprint to continue the agent with its full prior context instead of cold-starting.

### 4e. Dispatch Next

If there are more tasks in the sprint, proceed to 3a for the next assignment. The next agent will benefit from any learnings just persisted.

---

## Phase 5: Sprint Report

After all tasks are dispatched (or the sprint is stopped), emit a pipe-format report so downstream skills (`/retro`, `/assess`, `/curate`) can consume sprint outcomes:

```markdown
## Sprint Report

**Source**: /sprint
**Input**: [focus area from $ARGUMENTS, or "full sprint"]
**Pipeline**: (none — working from direct input)

### Items (N)

1. **[bead-title or task description]** — [1-line outcome summary]
   - member: [agent name]
   - status: completed | partial | blocked | failed
   - confidence: [agent's self-assessed confidence]
   - learnings: [count] new entries persisted
   - bead: [bead-id if available]
   - resumable: [agent-id if partial or blocked, omit otherwise]

2. **[bead-title or task description]** — [1-line outcome summary]
   - member: [agent name]
   - status: completed | partial | blocked | failed
   - confidence: [agent's self-assessed confidence]
   - learnings: [count] new entries persisted
   - bead: [bead-id if available]
   - resumable: [agent-id if partial or blocked, omit otherwise]

[... one item per dispatched task]

### Resumable Agents (M)

[Only include this section if any tasks returned partial or blocked. List each resumable agent so a follow-up sprint can continue them with full prior context.]

1. **[bead-title or task description]** — [why it stopped / what remains]
   - agent-id: [agent-id]
   - bead: [bead-id if available]
   - context: [1-sentence description of what the agent had completed before stopping]

[To resume: Task({resume: "<agent-id>", prompt: "additional context or follow-up instructions"})]

### Learning Deltas

- **Total new learnings**: [count] across [N] members
- **Cross-agent notes delivered**: [count]
- **Notable learnings**: [2-3 most significant new learnings]

### Blockers Encountered

[List any blocked or failed tasks with their blockers. "None" if clean sprint.]

### Summary

[One paragraph: tasks completed vs planned, overall confidence level, key learnings, and recommended next actions (next sprint focus, /retro, /curate).]
```

---

## Guidelines

1. **Serialize by default.** Dispatch one task at a time so each agent benefits from the previous one's learnings. Only parallelize if tasks are truly independent AND touch different ownership areas. **Exception:** With `isolation: "worktree"`, parallelization is safe even for tasks touching overlapping files, since each agent works on its own repo copy. The serialize default still applies when NOT using worktrees.
2. **Learnings are the product.** A sprint that completes tasks but doesn't persist learnings has wasted half its value. Always update learnings files.
3. **Validate learnings.** Don't blindly append everything an agent suggests. Filter out ephemeral observations and duplicates.
4. **Date every entry.** Always add `(added: YYYY-MM-DD)` to new learnings for staleness tracking.
5. **Watch for bloat.** If a learnings file exceeds 120 lines during a sprint, flag it for pruning in the next `/retro`.
6. **Context carries forward.** When dispatching the second+ task in a sprint, include a brief summary of what previous agents found/changed.

See also: /review (structured code review after agent implementation), /retro (capture sprint learnings at session end), /curate (optimize agent learnings before sprint dispatch), /tend (full learnings lifecycle: curate + promote).
