---
name: blossom
description: "Use when a goal is vague and you need to explore before planning. Runs spikes to discover what work exists, then produces a prioritized backlog. Keywords: explore, discover, spike, plan, epic, backlog, investigate."
argument-hint: "<goal or area to explore>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git:*), Task, SendMessage
context: inline
---

# Blossom: Emergent Spike-Driven Epic Workflow

You are running the **Blossom** workflow -- a recursive spike-driven exploration pattern that converts an unknown or loosely-defined goal into a comprehensive, prioritized backlog. The user wants to explore: **$ARGUMENTS**

## Don't Use When

- Goal is already well-defined with specific, actionable tasks — use /decompose instead to break down known work
- You need depth on a single well-scoped topic — use /fractal for focused recursive exploration rather than broad discovery
- Scope is a single file or function — blossom's spike-dispatch overhead is wasted on micro-scope work

## Overview

Blossom works in 6 phases. Spike dispatch uses agent teams for large explorations (6+ spikes) or background Task agents for small ones (5 or fewer).

```
Seed epic (identify spike areas, count determines dispatch mode)
  -> [SMALL: background agents] or [LARGE: spawn team blossom-<id>]
    -> spike teammates investigate areas, report via SendMessage
      -> orchestrator reviews reports, creates tasks, reuses idle teammates
        -> consolidator teammate runs /consolidate logic
          -> shutdown team, verify DAG, report final task list
```

---

## Phase 0: Session Health Gate

Before dispatching spikes, check that the session has enough capacity for the work ahead. Do this inline — do not invoke `/session-health` as a subagent.

**Assess the current session:**
- Roughly how many files have you read this session?
- How many agents have you dispatched?
- How many distinct codebase areas have you touched?

| Load Level | Files Read | Agents Dispatched | Signal |
|------------|-----------|-------------------|--------|
| Light | <10 | 0-1 | Healthy — proceed |
| Moderate | 10-25 | 2-4 | Proceed, watch quality |
| Heavy | 25-50 | 5+ | Warn user before dispatching |
| Overloaded | 50+ | 8+ | Recommend handoff |

**If load is Heavy or Overloaded:** Warn the user:

> Session context is heavy (~X files read, ~Y agents dispatched). Dispatching additional spike agents risks dropping results mid-workflow due to context compaction. Recommend running `/handoff` to capture current state and re-invoking `/blossom` in a fresh session. Continue anyway? (y/n)

Wait for user confirmation before proceeding. If the user declines, stop here and run `/handoff`.

**If load is Light or Moderate, or the user confirms they want to continue:** Proceed immediately to Phase 1.

---

## Phase 1: Seed the Epic

### 1a. Clarify the Goal

If `$ARGUMENTS` is empty or too vague, ask the user one clarifying question. Otherwise proceed immediately. Do not over-question -- the whole point of blossom is to discover scope through exploration, not upfront specification.

### 1b. Create the Epic

Create an epic to track the exploration goal using your preferred task tracking approach. Save the epic identifier. All subsequent tasks will be organized under this epic.

If no task tracker is configured, track the epic and all spike/task items in `TODO.md` instead, using a flat markdown list. Create or append to `TODO.md` at the project root with the epic title as a heading and each task as a checkbox item.

### 1c. Identify Initial Spike Areas

Decompose the goal into 3-6 bounded spike areas (a **/decompose** — MECE sub-parts). Each spike targets a specific codebase area or architectural concern (e.g., "Audit domain/agents/ for dead code", "Map trust system integration points").

**Count determines dispatch mode:** 5 or fewer spikes → background Task agents; 6+ spikes → agent teams.

### 1d. Create Spike Tasks

For each spike area, create a task as a child of the epic using your task tracker:

- **Title**: "SPIKE: [specific area to investigate]"
- **Description**: "Discovery spike. Investigate [area] and report: (1) firm tasks found, (2) areas needing deeper spikes, (3) clean areas requiring no work."

If no task tracker is configured, add each spike as a checkbox under the epic heading in `TODO.md`:
```
- [ ] SPIKE: [specific area to investigate]
```

---

## Team Lifecycle

This section applies only when the team dispatch mode is selected (6+ spikes). For background agent dispatch (5 or fewer spikes), skip to Phase 2.

### Setup

After Phase 1 completes, create the team. Use the short epic ID for uniqueness:

```
Team name: blossom-<short-epic-id>
```

Spawn spike teammates using the Task tool:

```
Task({
  team_name: "blossom-<short-epic-id>",
  name: "spike-1",
  subagent_type: "Explore",
  run_in_background: true,
  prompt: "<spike instructions -- see Phase 2>"
})
```

Spawn up to 4 spike teammates initially. Number them sequentially: `spike-1`, `spike-2`, `spike-3`, `spike-4`. Do not use area names as teammate names (they may contain special characters).

### Teammate Reuse

When a spike teammate finishes and goes idle, **reuse it** by sending new spike instructions via SendMessage rather than spawning fresh. Only spawn new teammates if all existing ones are busy AND total count is under 6.

### Fallback

If team creation fails (teams not enabled, API error, or other failure), fall back to the background Task agent dispatch mode described in Phase 2. Log the failure reason but do not block the workflow.

### Shutdown and Cleanup

After consolidation (Phase 3), send `shutdown_request` via SendMessage to each active teammate (spike-N and consolidator). Retry once if no response. Proceed with Phases 4-6 without the team.

---

## Phase 2: Execute Spikes

### Dispatch Mode A: Background Task Agents (5 or fewer spikes)

Launch spike agents using the Task tool with `run_in_background=true` and `subagent_type="Explore"`. Launch up to 4 concurrently. As each completes, process its results immediately (create firm tasks + new spikes). If new spikes are created, dispatch them in the next batch.

### Dispatch Mode B: Agent Teams (6+ spikes)

After the team is created (see Team Lifecycle), spike teammates are already running. Communication flows through SendMessage:

1. **Initial dispatch**: The first batch of spike teammates (up to 4) receive their instructions at spawn time via the Task tool prompt.

2. **Receiving reports**: Spike teammates send their reports back via SendMessage. Monitor for incoming messages from spike-N teammates.

3. **Processing reports**: When a report arrives, process it immediately (see "After Each Spike Completes" below).

4. **Dispatching more spikes**: After processing a report, check for pending spikes. If spikes remain:
   - **Prefer reuse**: Send the next spike's instructions to the now-idle teammate via SendMessage.
   - **Spawn new**: Only if all teammates are busy and total teammate count is under 6.

5. **Tracking**: Maintain a count of total spikes dispatched (including reused teammates). This count tracks against the 20-spike safety limit.

### Spike Instructions

Each spike agent (whether background Task or team teammate) receives these instructions. The spike report uses **pipe format** (see `/rules/pipe-format.md`) so downstream primitives can consume spike output directly.

> You are executing a discovery spike for the Blossom workflow. This is an investigation using the **/gather** pattern — collect findings with sources and confidence levels.
>
> **Your area:** [spike description]
>
> **Your job:** Thoroughly investigate this area and produce a structured report. Do NOT implement fixes -- only discover and document.
>
> Follow the investigation protocol and report requirements from the Agent Preamble (fan-out-protocol rule).
>
> **Spike-specific requirements:**
>
> 1. Use Glob to find all relevant files in the area
> 2. When you find something that looks like an issue, verify by reading surrounding code:
>    - Check who calls this code (callers)
>    - Check if tests cover it
>    - Check if it's wired in bootstrap/DI
>    - Check if the interface layer exposes it
> 3. For each finding, state your confidence level:
>    - **CONFIRMED**: You read the code and verified the issue exists
>    - **LIKELY**: Strong evidence from multiple signals but couldn't fully trace the chain
>    - **POSSIBLE**: Suspicious pattern that needs a deeper spike to verify
>
> **Report format (pipe format — you MUST follow this exactly):**
>
> ```
> ## Spike findings for [area]
>
> **Source**: /blossom (spike)
> **Input**: [spike description]
>
> ### Items
>
> 1. **[action verb] [specific thing]** — [what you found and how you verified]
>    - source: [file path:line number]
>    - confidence: CONFIRMED | LIKELY | POSSIBLE
>    - priority: P0-P4
>    - scope: [estimated files to change]
>
> 2. ...
>
> ### Deeper Spikes Needed
>
> For each area needing more investigation:
> - **[specific sub-area]** — [what suggests work here but could not confirm]
>   - look-for: [specific questions the deeper spike should answer]
>
> ### Clean Areas
> - [area]: No issues found. [brief evidence — what you read and why you're confident]
>
> ### Summary
>
> [One paragraph: N firm tasks (X CONFIRMED, Y LIKELY), M deeper spikes needed, K clean areas.]
> ```

**For team teammates, add this prefix to the instructions:**

> When your investigation is complete, send your full report via SendMessage to the orchestrator (team lead). Do not create tasks directly -- the orchestrator handles that.

### After Each Spike Completes

1. **Review the spike report** for quality. Run through these explicit checks before accepting the report:

   - [ ] Response contains pipe-format structure: a `##` heading, a `**Source**:` line, and a `### Items` section
   - [ ] At least one `CONFIRMED`, `LIKELY`, or `POSSIBLE` confidence tag is present in the Items section
   - [ ] At least one `file:line` citation is present (evidence from actual code reading, not speculation)

   **If any check fails:**
   - **Team mode:** Send ONE pushback message naming which checks failed and demanding a corrected report (Items section with confidence tags and file:line citations from actual code reading). One retry only -- if still inadequate, log failure and move on.
   - **Background mode:** Resume the agent with `Task({resume: "<agent-id>", prompt: "Your report failed quality checks: [which checks failed]. Please re-investigate and resubmit with pipe-format structure, confidence tags (CONFIRMED/LIKELY/POSSIBLE), and file:line citations from actual code reading."})`. **One retry only — if the second attempt still fails quality, accept the result with a quality note.** Flag the quality issue on the spike task in your task tracker. If no tracker is configured, note the quality failure in `TODO.md` next to the spike item. Do not create firm tasks from low-quality reports.

2. **Create firm tasks** as children of the epic using your task tracker. Each task should include the title, priority (P level as 0-4), confidence level, and evidence/scope from the spike report. If no tracker is configured, add to `TODO.md` under the epic heading as `- [ ] [title from spike report]`.

3. **Create new spike tasks** as children of the epic for areas needing deeper investigation. Include the parent spike reference, reason from the report, and specific questions. If no tracker is configured, add to `TODO.md` as `- [ ] SPIKE: [deeper area]`.

4. **Close the completed spike** with a findings summary noting N firm tasks (X confirmed, Y likely), M deeper spikes needed. If using `TODO.md`, mark the spike done by checking its checkbox (`- [x] SPIKE: ...`) and appending a findings note inline.

### Recursion

If new spikes were created, dispatch them (via idle teammate reuse or new background agent). Continue until no new spikes are generated.

**Safety limit:** If total spikes executed exceeds 20, stop and report to the user. The goal may be too broad.

---

## Phase 3: Consolidate

After all spikes are complete and all firm tasks created, run consolidation to clean up the task list before wiring dependencies. Consolidation applies **/filter** logic (dedup, stale detection — binary keep/drop per item) and **/assess** logic (completeness audit — categorical verdict per architectural slice).

### Dispatch Mode A: Background Agents (no team active)

Instruct the user to run:

```
/consolidate [epic title or area]
```

If using a simple `TODO.md` list, skip consolidation and proceed directly to Phase 3b — dedup and stale detection must be done manually by reviewing the file.

Then proceed to Phase 3b.

### Dispatch Mode B: Agent Teams (team active)

Spawn a consolidator teammate in the existing team:

```
Task({
  team_name: "blossom-<short-epic-id>",
  name: "consolidator",
  subagent_type: "general-purpose",
  run_in_background: true,
  prompt: "<consolidation instructions below>"
})
```

**Consolidation instructions for the teammate:**

> You are the consolidation agent for the Blossom workflow. Your job is to review the task list under epic [epic-id] and tighten it before final dependency wiring.
>
> Run these steps in order:
>
> **1. Survey:** Review all open tasks, in-progress work, and blocked items for the epic.
>
> **2. Dedup:** Within each cluster of tasks, find exact duplicates, subset tasks, and convergent tasks. Close duplicates noting what they duplicate. Merge subsets into their parent.
>
> **3. Vertical slice audit:** Read the project structure to discover its architectural layers. For each task touching an inner layer, verify companion tasks exist across layer boundaries (persistence, wiring, exposure, tests). Create missing companions.
>
> **4. Stale detection:** Check for tasks created more than 2 weeks ago with no progress, tasks whose target code has been refactored, or tasks describing work already done (check git log). Close stale tasks with an explanation.
>
> **5. Dependency cleanup:** Remove redundant transitive dependencies. Check for cycles. Validate the full epic structure (cycles, orphans, disconnected subgraphs). Verify all tasks are proper children of the epic.
>
> **6. Report:** Send your consolidation report via SendMessage to the orchestrator with these counts: tasks closed (dedup), tasks closed (stale), tasks created (gap fill), dependencies modified.
>
> Be conservative with closures -- when in doubt, keep a task open and add a note. Always check the code before declaring something stale.

When the consolidator's report arrives, review it and proceed to Phase 3b.

### 3b. Agent Assignment Hints

After consolidation, tag each firm task with grounded agent assignment notes. The sharpening gate: name specific files the agent will touch (from spike findings), state concrete skills/knowledge needed (not just a role), and make it dispatchable.

Add agent hints to each task in your tracker (or as inline notes in `TODO.md` next to each task item).

**Before:** "Recommended agent: refactorer"
**After:** "Recommended agent: refactorer — touches src/domain/auth/User.ts + src/domain/auth/Session.ts. Requires: DDD pattern knowledge, extract-interface refactoring"

This gives /sprint enough context to make good dispatch decisions without reading every spike report.

### 3c. Team Shutdown (if team active)

After consolidation completes and agent hints are assigned, shut down all teammates (see Team Lifecycle > Shutdown and Cleanup). The orchestrator proceeds solo for Phases 4-6.

---

## Phase 4: Prioritize and Wire Dependencies

### Cross-Task Dependencies

Wire dependencies where order matters using your task tracker. Inner layers before outer layers, interfaces before implementations, shared files sequenced. Think bottom-up through the dependency graph.

If using `TODO.md`, note ordering constraints as inline comments next to each task item (e.g., `<!-- depends on: [task name] -->`).

### Priority Review

Adjust priorities now that the full picture is visible. Upgrade tasks that block many others, downgrade inflated P1s, and bump quick wins (small scope + high value). If using `TODO.md`, reorder items to reflect execution order.

---

## Phase 5: Verify

Validate the epic structure using your task tracker's validation capabilities. Check for:
- Dependency cycles
- Orphaned tasks (children with no dependencies wired)
- Disconnected subgraphs
- Ready fronts (waves of parallelizable work)

If validation reports issues, fix them before proceeding.

If using `TODO.md`, manually review to verify: no duplicate items, all noted ordering constraints are consistent (no circular dependencies), and execution order is clear.

Then manually verify: priority consistency (P0/P1 tasks must not depend on P3/P4 -- upgrade blockers), every task has testable acceptance criteria, and trace the critical path (longest dependency chain = minimum time to completion).

---

## Phase 6: Report

Present the final blossom report in **pipe format** so downstream primitives (/rank, /filter, /assess) can consume the backlog directly:

```markdown
## Explored backlog for [epic title]

**Source**: /blossom
**Input**: [original $ARGUMENTS]

### Items

1. **[task title]** — [evidence summary]
   - source: [primary file:line or area]
   - confidence: CONFIRMED | LIKELY
   - priority: P0-P4
   - depends-on: [task IDs or "none"]
   - agent: [recommended agent type]

2. ...

### Backlog Status

> **All items above already exist as tracked tasks** under epic [epic-id]. Do NOT create duplicate tasks — they are tracked and ready for `/sprint` dispatch.

### Exploration

- **Epic ID:** [epic-id]
- **Dispatch mode:** [background agents | team blossom-<id>]
- **Total spikes:** N executed across M depth levels
- **Spike quality:** [brief assessment — did agents confirm or hedge?]
- **Areas explored:** [list of top-level spike areas]
- **Clean areas:** [areas confirmed as needing no work]
- **Consolidation:** [dedup N, stale N, gap-fill N, deps modified N]

### Critical Path

[Longest dependency chain with task IDs and titles]

### Parallel Opportunities

[Tasks with no dependencies on each other that can be worked simultaneously]

### Summary

[One paragraph synthesizing the exploration: total tasks, priority distribution (P0: N, P1: N, P2: N, P3: N, P4: N), confidence distribution (CONFIRMED: N, LIKELY: N), recommended execution order, and any open questions from exploration.]
```

---

## Key Principles

1. **Explore, don't implement.** Spikes discover work; they never do it.
2. **Verify, don't speculate.** Read the actual code. CONFIRMED findings over hedged guesses.
3. **Hybrid dispatch.** Use teams for large explorations (6+ spikes), background agents for small ones (5 or fewer). Fall back to background agents if team creation fails.
4. **Reuse over respawn.** When a spike teammate finishes, send it a new spike via SendMessage instead of spawning a fresh teammate. This avoids spawn overhead and reuses warm context.
5. **Consolidate before reporting.** Run consolidation (via teammate or /consolidate) to dedup, fill slice gaps, and resolve orphans before wiring final dependencies.
6. **Use epic hierarchy.** Always create child tasks under the epic. This establishes the hierarchy needed for status tracking and completion checks. Use cross-task dependencies only for ordering (task A blocks task B).
7. **Quality gate.** The orchestrator reviews every spike report before creating tasks. Spike agents never create tasks directly.
8. **Structured reports.** Spike agents must follow the exact report format for consistent processing.
9. **Depth limit.** Stop at 20 total spikes (including reused teammates) and reassess with the user if the goal is too broad.
10. **Confidence levels.** Every finding is CONFIRMED, LIKELY, or POSSIBLE. Possible triggers a deeper spike.
11. **Clean shutdown.** After consolidation, shut down all teammates before proceeding to final phases. The orchestrator works solo for prioritization, verification, and reporting.

12. **Persist epic state.** After reporting, write `memory/epics/<epic-id>/epic.md` so downstream skills (/sprint) can load context across sessions.
13. **Report that tasks exist.** The Phase 6 report MUST include the Backlog Status callout stating that all items already exist as tracked tasks. The caller (primary session) should NOT create duplicate tasks — blossom has already done this in Phases 2-4.
14. **Compaction resilience**: This skill has 7 phases. Write intermediate state to `memory/scratch/blossom-checkpoint.md` at phase boundaries per `rules/compaction-resilience.md`.

## Phase 7: Persist Epic State

After the Phase 6 report, write epic state to `memory/epics/<epic-id>/epic.md` so that `/sprint` and other downstream skills can load blossom findings in a later session without re-exploring.

**Create the directory and file:**

```bash
mkdir -p memory/epics/<epic-id>
```

Write `memory/epics/<epic-id>/epic.md` with this structure:

```markdown
# Epic: [epic title]

**Epic ID**: [epic-id]
**Created**: [date]
**Source**: /blossom
**Goal**: [original $ARGUMENTS]

## Spike Findings

[Copy the Phase 6 pipe-format Items section here verbatim — numbered list with source, confidence, priority, depends-on, and agent fields per item.]

## Priority Order

[Ordered list of task IDs/titles from highest to lowest priority, reflecting final Phase 4 ordering.]

## Task IDs

| Task ID | Title | Priority | Status | Assigned Agent |
|---------|-------|----------|--------|----------------|
| [id]    | [title] | P[N]  | open   | [agent hint]   |
| ...     | ...   | ...      | ...    | ...            |

## Critical Path

[Copy the Phase 6 Critical Path section.]

## Parallel Opportunities

[Copy the Phase 6 Parallel Opportunities section.]
```

If no task tracker is configured, write the same file but use TODO.md task names instead of tracker IDs in the Task IDs table.

See also: /meeting (discuss blossom findings with multiple perspectives before committing to a direction); /review (evaluate the implementations that result from executing blossom tasks).
