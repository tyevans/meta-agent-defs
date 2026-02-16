---
name: team-meeting
description: "Run a goal-oriented planning session with the persistent team to decompose a goal into sprint-ready tasks with member assignments. Use when you have a goal and want the team to plan it collaboratively rather than exploring (blossom) or refining (meeting). Keywords: plan, team, goal, collaborate, assign, tasks, kickoff, planning session."
argument-hint: "<goal to plan>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Write, Edit, Grep, Glob, Bash(bd:*), Bash(git:*), Task, SendMessage, TeamCreate, TeamDelete, AskUserQuestion
---

# Team Meeting: Goal-Oriented Planning with Your Team

You are facilitating a **Team Meeting** -- a collaborative planning session where the persistent team (from `/assemble`) breaks down a goal into sprint-ready tasks with concrete member assignments.

**Goal:** $ARGUMENTS

## When to Use

- When you have a goal and want the team to plan it before execution
- When multiple ownership areas are involved and members need to coordinate
- When you want task assignments that reflect actual team expertise and ownership
- When the goal is roughly scoped but needs decomposition and sequencing
- After a decision is made (e.g., from `/meeting` or `/consensus`) and you need to plan the execution

## How It Differs

| Skill | Purpose | Output |
|-------|---------|--------|
| /meeting | Conservative refinement, explore ideas with abstract roles | Decisions + action items |
| /team-meeting | Goal-oriented planning with your actual team | Sprint-ready task assignments |
| /blossom | Discover unknown scope via exploration spikes | Epic + backlog |
| /sprint | Execute already-planned work | Completed tasks + learnings |

## Overview

```
Load team manifest + learnings
  -> Present goal to team members (parallel)
    -> Each member scopes their contribution
      -> Facilitator surfaces dependencies and conflicts
        -> Members discuss sequencing and risks
          -> Synthesize into sprint-ready plan
            -> User approves -> optionally create beads
```

---

## Phase 0: Prerequisite Gate

### 0a. Validate Team Exists

Read `.claude/team.yaml`. If it does not exist, tell the user:

> No team manifest found. Run `/assemble` first to create your team, then come back with `/team-meeting <goal>`.

Stop here if no team exists.

### 0b. Validate Goal

If `$ARGUMENTS` is empty or too vague to act on, ask the user one clarifying question. The goal should be specific enough that team members can identify what files and systems they would touch. Do not over-question -- the team discussion itself will clarify scope.

---

## Phase 1: Load Context

### 1a. Read Team Manifest

Read `.claude/team.yaml` and extract all member names, roles, and ownership patterns.

### 1b. Read All Learnings

For each member, read `memory/agents/<name>/learnings.md`. Note relevant learnings that might affect planning for this goal.

### 1c. Read Team Decisions

Read `memory/team/decisions.md` for prior decisions that constrain this goal.

### 1d. Load Backlog (conditional)

If `.beads/` exists, run:

```bash
bd ready
bd list --status=in_progress
```

Check for existing work that overlaps with the goal. Flag duplicates or related beads.

### 1e. Prerequisite Confirmation

Do not proceed until:
- [ ] team.yaml contents are in context
- [ ] Every member's learnings.md is read
- [ ] Team decisions are read
- [ ] Backlog state is known (or confirmed absent)

---

## Phase 2: Assemble the Planning Team

### 2a. Select Relevant Members

Not every team member needs to participate. Select members whose `owns` patterns overlap with the goal. A 2-4 member planning session is ideal -- more creates noise.

Present the selection:

```markdown
## Planning Session for: [goal]

**Participating members:**
| Member | Role | Why included |
|--------|------|-------------|
| [name] | [role] | [ownership overlap or expertise match] |
| ... | ... | ... |

**Excluded:** [names] -- [reason, e.g., "no ownership overlap with this goal"]
```

### 2b. Create the Team

```
TeamCreate({ team_name: "plan-<short-goal-slug>" })
```

### 2c. Spawn Members

For each participating member, spawn a teammate:

```
Task({
  team_name: "plan-<slug>",
  name: "<member-name>",
  subagent_type: "general-purpose",
  run_in_background: true,
  prompt: "<member planning prompt -- see below>"
})
```

**Member planning prompt template:**

> You are **[name]**, the [role] on this team.
>
> Your ownership areas: [owns patterns]
>
> ## Your Accumulated Learnings
> [contents of memory/agents/<name>/learnings.md]
>
> ## Relevant Team Decisions
> [applicable entries from decisions.md]
>
> ## The Goal
> [goal from $ARGUMENTS]
>
> **CRITICAL: You MUST use the SendMessage tool to communicate.** Your plain text output is NOT visible to anyone. Every response you give must be sent via `SendMessage({ type: "message", recipient: "team-lead", content: "...", summary: "..." })`. Always send to **team-lead** (the facilitator). If you do not call SendMessage, nobody will see what you said.
>
> ## Your Task
>
> Analyze this goal from your ownership perspective and send a planning report to the facilitator covering:
>
> 1. **Your scope**: What specific work falls in your ownership area? Be concrete -- name files, modules, or systems you would touch.
> 2. **Task breakdown**: Break your scope into 1-4 discrete tasks. Each task should be completable in one agent session. For each task:
>    - Title (action verb + specific thing)
>    - What changes (files, functions, patterns)
>    - Estimated complexity (small / medium / large)
>    - Acceptance criteria (how to verify it is done)
> 3. **Dependencies**: What do you need from other team members before you can start? What do other members need from you?
> 4. **Risks**: What could go wrong in your area? What assumptions are you making?
> 5. **Questions**: Anything you need clarified before committing to this plan?
>
> Be concrete and specific. Name files, functions, and patterns. Do not be abstract.

---

## Phase 3: Facilitation

### 3a. Collect Initial Reports

Wait for all member reports to arrive via SendMessage. As each arrives, extract:
- Their proposed tasks (with details)
- Dependencies they declared
- Risks they flagged
- Questions they raised

### 3b. Cross-Reference Dependencies

Build a dependency map from member reports:
- **Match dependencies**: If member A says "I need X from B" and B's tasks include producing X, wire the dependency.
- **Flag gaps**: If A needs something no member is producing, flag it.
- **Flag conflicts**: If two members plan to modify the same files, flag the overlap.

### 3c. Relay Conflicts and Questions (if any)

If dependencies, conflicts, or questions need resolution, send targeted messages to the relevant members. Use the same facilitation principles as /meeting:

- **Send direct messages**, not broadcasts
- **Anonymize sources** when relaying concerns between members -- describe the concern, not who raised it
- **Keep exchanges short** -- 1-2 rounds maximum per conflict
- **Decide for trivial conflicts** -- if the resolution is obvious, state it and move on

If no conflicts exist, skip directly to Phase 4.

### 3d. Resolve Open Questions

For questions that members cannot answer (require user input), collect them and ask the user:

```
Team members raised these questions about the plan:

1. [question from member, anonymized]
2. [question from member, anonymized]

Your answers will shape the final task assignments.
```

After user answers, relay relevant answers to members if they need to revise their scope.

---

## Phase 4: Synthesize the Plan

### 4a. Merge Task Lists

Combine all member task proposals into a single plan. For each task:
- Assigned member (from who proposed it)
- Dependencies (from cross-reference in 3b)
- Priority (P0-P4, based on dependency depth and risk)
- Estimated complexity

### 4b. Sequence by Dependency

Order tasks so dependencies come before dependents. Identify:
- **Critical path**: Longest dependency chain (minimum time to completion)
- **Parallel opportunities**: Independent tasks that can run simultaneously
- **Blocking tasks**: Tasks that many others depend on (priority boost)

### 4c. Present the Plan

```markdown
## Team Plan: [goal]

### Task Assignments

| # | Task | Member | Depends On | Priority | Complexity |
|---|------|--------|------------|----------|------------|
| 1 | [title] | [name] | none | P[0-4] | [small/med/large] |
| 2 | [title] | [name] | #1 | P[0-4] | [small/med/large] |
| ... | ... | ... | ... | ... | ... |

### Execution Sequence

**Round 1 (parallel):** #1, #3 -- [brief reason these can run together]
**Round 2 (parallel):** #2, #4 -- [after round 1 dependencies satisfied]
**Round 3:** #5 -- [after round 2]

### Critical Path
[task chain with IDs and titles]

### Risks
- [risk from member reports, with mitigation if discussed]

### Open Items
- [anything unresolved, or "None -- plan is complete"]

### Sharpening Gate

Every task must pass three tests:

1. **Name the specific code/file/workflow** where the change happens
2. **State what concretely should change** (a function to add, a check to insert, a pattern to adopt)
3. **Make it assignable** -- could an agent implement this in one session without design decisions?

Tasks that fail the gate are either sharpened in-place or converted to investigation spikes.
```

Ask the user:

> **Approve** this plan, **adjust** assignments or sequencing, or **add/remove** tasks?

---

## Phase 5: Finalize

### 5a. Shutdown Team

After user approval, send shutdown requests to all members:

```
SendMessage({
  type: "shutdown_request",
  recipient: "<member-name>",
  content: "Planning session complete, shutting down"
})
```

After all members confirm, delete the team:

```
TeamDelete()
```

### 5b. Create Beads (conditional)

If `.beads/` exists, offer to create beads from the approved plan:

```bash
bd create --title="[task title]" --type=task --priority=[0-4] \
  --description="Assigned: [member]. [acceptance criteria]. Depends on: [dep titles or none]. From team-meeting on [goal]."
```

Wire dependencies:

```bash
bd dep add <downstream-id> <upstream-id>
```

### 5c. Report

```markdown
## Planning Complete: [goal]

**Tasks created:** [count]
**Members involved:** [names]
**Critical path:** [length] tasks, [rounds] execution rounds
**Ready for:** `/sprint` to execute the plan

### Quick Reference
| Task | Member | Status |
|------|--------|--------|
| [title] | [name] | ready / blocked by [dep] |
| ... | ... | ... |
```

---

## Guidelines

1. **Use the real team.** Members come from `.claude/team.yaml`, not abstract roles. Their ownership patterns and learnings shape their planning contributions.
2. **Goal in, sprint plan out.** The output is a sprint-ready task list with assignments, not a discussion summary. Every task must pass the sharpening gate.
3. **Short planning sessions.** Target 1-2 facilitation rounds maximum. If conflicts cannot be resolved in 2 rounds, flag them as open items and let the user decide.
4. **Respect ownership.** Members plan work within their `owns` patterns. If a task crosses boundaries, split it or assign the primary owner with a dependency on the secondary.
5. **Dependencies are the hard part.** Most planning value comes from surfacing what members need from each other. The dependency cross-reference (Phase 3b) is the core of this skill.
6. **Not every member attends.** Only include members with ownership overlap. A 2-member planning session for a backend change is better than a 5-member session where 3 have nothing to contribute.
7. **Builds on /meeting, feeds /sprint.** Use `/meeting` first when the direction is unclear (explore and decide). Use `/team-meeting` when direction is set and you need a plan. Use `/sprint` to execute the plan.
8. **DMs over broadcasts.** Direct messages produce substantive responses. Broadcasts produce acknowledgments. Use DMs for all questions.

See also: /meeting (explore ideas with abstract roles), /sprint (execute planned work), /assemble (create the team).
