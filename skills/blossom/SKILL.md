---
name: blossom
description: "Run spike-driven exploration to discover and plan work for an unknown or loosely-defined goal. Use when you need to explore a codebase area, create an epic with prioritized tasks, or convert a vague objective into a structured backlog. Keywords: explore, discover, spike, plan, epic, backlog, investigate."
argument-hint: "<goal or area to explore>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Task, SendMessage
context: fork
---

# Blossom: Emergent Spike-Driven Epic Workflow

You are running the **Blossom** workflow -- a recursive spike-driven exploration pattern that converts an unknown or loosely-defined goal into a comprehensive, prioritized backlog. The user wants to explore: **$ARGUMENTS**

## Overview

Blossom works in 6 phases. Spike dispatch uses agent teams for large explorations (6+ spikes) or background Task agents for small ones (5 or fewer).

```
Seed epic (identify spike areas, count determines dispatch mode)
  -> [SMALL: background agents] or [LARGE: spawn team blossom-<id>]
    -> spike teammates investigate areas, report via SendMessage
      -> orchestrator reviews reports, creates beads, reuses idle teammates
        -> consolidator teammate runs /consolidate logic
          -> shutdown team, verify DAG, report final backlog
```

---

## Phase 1: Seed the Epic

### 1a. Clarify the Goal

If `$ARGUMENTS` is empty or too vague, ask the user one clarifying question. Otherwise proceed immediately. Do not over-question -- the whole point of blossom is to discover scope through exploration, not upfront specification.

### 1b. Create the Epic

```bash
bd create --title="EPIC: [goal description]" --type=epic --priority=2
```

Save the returned epic ID. All subsequent beads will be wired as dependencies of this epic.

### 1c. Identify Initial Spike Areas

Decompose the goal into 3-6 bounded spike areas (a **/decompose** — MECE sub-parts). Each spike targets a specific codebase area or architectural concern (e.g., "Audit domain/agents/ for dead code", "Map trust system integration points").

**Count determines dispatch mode:** 5 or fewer spikes → background Task agents; 6+ spikes → agent teams.

### 1d. Create Spike Beads

For each spike area:

```bash
bd create --title="SPIKE: [specific area to investigate]" --type=task --priority=2 \
  --description="Discovery spike. Investigate [area] and report: (1) firm tasks found, (2) areas needing deeper spikes, (3) clean areas requiring no work."
```

### 1e. Wire Dependencies

Epic depends on children (epic waits for all children to complete):

```bash
bd dep add <epic-id> <spike-id>
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

> You are executing a discovery spike for the Blossom workflow. This is a **/gather**-style investigation — collect findings with sources and confidence levels.
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

> When your investigation is complete, send your full report via SendMessage to the orchestrator (team lead). Do not create beads or tasks directly -- the orchestrator handles that.

### After Each Spike Completes

1. **Review the spike report** for quality. A passing report has: pipe format structure, an Items section with at least one CONFIRMED finding, and file:line citations from actual code reading.

   **Quality gate:** If the report fails these checks:
   - **Team mode:** Send ONE pushback message demanding concrete output (Items section, CONFIRMED findings with file:line citations, evidence from code reading). One retry only -- if still inadequate, log failure and move on.
   - **Background mode:** Cannot re-prompt. Flag via `bd update <spike-id> --notes="QUALITY ISSUE: [problem]"`. Do not create firm tasks from low-quality reports.

2. **Create firm task beads** from the Items section:

```bash
bd create --title="[title from spike report]" --type=task --priority=[P level as 0-4] \
  --description="[confidence level]. [evidence and scope from spike report]"
bd dep add <epic-id> <task-id>
```

3. **Create new spike beads** from the "Deeper Spikes Needed" section:

```bash
bd create --title="SPIKE: [deeper area]" --type=task --priority=2 \
  --description="Deeper discovery spike spawned from SPIKE: [parent spike]. Reason: [why from report]. Look for: [specific questions]"
bd dep add <epic-id> <new-spike-id>
```

4. **Close the completed spike** with findings summary:

```bash
bd close <spike-id>
bd update <spike-id> --notes="Completed. Found N firm tasks (X confirmed, Y likely), M deeper spikes needed. Key findings: [1-2 sentence summary]"
```

### Recursion

If new spikes were created, dispatch them (via idle teammate reuse or new background agent). Continue until no new spikes are generated.

**Safety limit:** If total spikes executed exceeds 20, stop and report to the user. The goal may be too broad.

---

## Phase 3: Consolidate

After all spikes are complete and all firm tasks created, run consolidation to clean up the backlog before wiring dependencies. Consolidation applies **/filter** logic (dedup, stale detection — binary keep/drop per item) and **/assess** logic (completeness audit — categorical verdict per architectural slice).

### Dispatch Mode A: Background Agents (no team active)

Instruct the user to run:

```
/consolidate [epic title or area]
```

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

> You are the consolidation agent for the Blossom workflow. Your job is to review the backlog under epic [epic-id] and tighten it before final dependency wiring.
>
> Run these steps in order:
>
> **1. Survey:**
> ```bash
> bd stats
> bd list --status=open
> bd blocked
> ```
>
> **2. Dedup:** Within each cluster of tasks, find exact duplicates, subset tasks, and convergent tasks. Close duplicates with `bd close <id> --reason="Duplicate of <other-id>"`. Merge subsets into their parent with `bd update <parent> --notes="Absorbed <subset>: [details]"`.
>
> **3. Vertical slice audit:** Read the project structure to discover its architectural layers. For each task touching an inner layer, verify companion tasks exist across layer boundaries (persistence, wiring, exposure, tests). Create missing companions with `bd create`.
>
> **4. Stale detection:** Check for tasks created more than 2 weeks ago with no progress, tasks whose target code has been refactored, or tasks describing work already done (check git log). Close stale tasks with `bd close <id> --reason="Stale: [explanation]"`.
>
> **5. Dependency cleanup:** Remove redundant transitive dependencies. Check for cycles. Validate epic structure (epic depends on children, not the reverse).
>
> **6. Report:** Send your consolidation report via SendMessage to the orchestrator with these counts: tasks closed (dedup), tasks closed (stale), tasks created (gap fill), dependencies modified.
>
> Be conservative with closures -- when in doubt, keep a task open and add a note. Always check the code before declaring something stale.

When the consolidator's report arrives, review it and proceed to Phase 3b.

### 3b. Agent Assignment Hints

After consolidation, tag each firm task with the recommended agent type:

```bash
bd update <task-id> --notes="Recommended agent: domain-architect | infrastructure-implementer | api-developer | cli-developer | test-generator | refactorer | general-purpose"
```

### 3c. Team Shutdown (if team active)

After consolidation completes and agent hints are assigned, shut down all teammates (see Team Lifecycle > Shutdown and Cleanup). The orchestrator proceeds solo for Phases 4-6.

---

## Phase 4: Prioritize and Wire Dependencies

### Cross-Task Dependencies

Wire dependencies (`bd dep add <downstream> <upstream>`) where order matters: inner layers before outer layers, interfaces before implementations, shared files sequenced. Think bottom-up through the dependency graph.

### Priority Review

Adjust priorities now that the full picture is visible. Upgrade tasks that block many others, downgrade inflated P1s, and bump quick wins (small scope + high value).

---

## Phase 5: Verify

Validate the backlog before presenting it: check for dependency cycles (fix any A↔B loops), verify priority consistency (P0/P1 tasks must not depend on P3/P4 -- upgrade blockers), ensure every task has testable acceptance criteria, and trace the critical path (longest dependency chain = minimum time to completion).

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
6. **Epic depends on children.** Always `bd dep add <epic> <child>`, never the reverse.
7. **Quality gate.** The orchestrator reviews every spike report before creating beads. Spike agents never create beads directly.
8. **Structured reports.** Spike agents must follow the exact report format for consistent processing.
9. **Depth limit.** Stop at 20 total spikes (including reused teammates) and reassess with the user if the goal is too broad.
10. **Confidence levels.** Every finding is CONFIRMED, LIKELY, or POSSIBLE. Possible triggers a deeper spike.
11. **Clean shutdown.** After consolidation, shut down all teammates before proceeding to final phases. The orchestrator works solo for prioritization, verification, and reporting.

See also: /meeting (discuss blossom findings with multiple perspectives before committing to a direction).
