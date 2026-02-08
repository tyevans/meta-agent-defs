---
name: blossom
description: "Run spike-driven exploration to discover and plan work for an unknown or loosely-defined goal. Use when you need to explore a codebase area, create an epic with prioritized tasks, or convert a vague objective into a structured backlog. Keywords: explore, discover, spike, plan, epic, backlog, investigate."
argument-hint: "<goal or area to explore>"
disable-model-invocation: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Task
context: fork
---

# Blossom: Emergent Spike-Driven Epic Workflow

You are running the **Blossom** workflow -- a recursive spike-driven exploration pattern that converts an unknown or loosely-defined goal into a comprehensive, prioritized backlog. The user wants to explore: **$ARGUMENTS**

## Overview

Blossom works in 6 phases:

```
Seed epic
  -> spawn discovery spikes (expand)
    -> each spike produces firm tasks + deeper spikes
      -> repeat until all areas explored
        -> consolidate via /consolidate
          -> verify (DAG check, priorities, criteria)
            -> report final backlog
```

---

## Phase 1: Seed the Epic

### 1a. Clarify the Goal

If `$ARGUMENTS` is empty or too vague, ask the user one clarifying question. Otherwise proceed immediately. Do not over-question -- the whole point of blossom is to discover scope through exploration, not upfront specification.

### 1b. Create the Epic

```bash
bd create --title="EPIC: [goal description]" --type=feature --priority=2
```

Save the returned epic ID. All subsequent beads will be wired as dependencies of this epic.

### 1c. Identify Initial Spike Areas

Based on the goal, identify 3-6 areas that need investigation. Each spike should target a specific, bounded area of the codebase or architecture. Good spike scoping examples:

- "Audit domain/agents/ for dead code and unused events"
- "Map all trust system integration points"
- "Inventory sandbox execution pipeline gaps"
- "Survey frontend components for accessibility issues"

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

## Phase 2: Execute Spikes

### Dispatch Rules

- Launch spike agents **up to 4 concurrently** using the Task tool with `run_in_background=true`
- Use the Task tool with `subagent_type="Explore"` for each spike
- As each spike completes, process its results immediately (create firm tasks + new spikes)
- When new spikes are created during processing, dispatch them in the next batch

### Spike Agent Instructions

Each spike agent receives these instructions:

> You are executing a discovery spike for the Blossom workflow.
>
> **Your area:** [spike description]
>
> **Your job:** Thoroughly investigate this area and produce a structured report. Do NOT implement fixes -- only discover and document.
>
> **Investigation protocol (CRITICAL -- follow this exactly):**
>
> 1. Use Glob to find all relevant files in the area
> 2. **READ the actual implementation** -- do not just grep for patterns. Open files, read functions, trace call chains. Understand what the code actually does.
> 3. When you find something that looks like an issue, **VERIFY by reading surrounding code**:
>    - Check who calls this code (callers)
>    - Check if tests cover it
>    - Check if it's wired in bootstrap/DI
>    - Check if the interface layer exposes it
> 4. **Never flag something as uncertain if you can verify it by reading one more file.**
> 5. For each finding, state your **confidence level**:
>    - **CONFIRMED**: You read the code and verified the issue exists
>    - **LIKELY**: Strong evidence from multiple signals but couldn't fully trace the chain
>    - **POSSIBLE**: Suspicious pattern that needs a deeper spike to verify
>
> **Report format (you MUST follow this exactly):**
>
> ```
> ## Spike Report: [area]
>
> ### Firm Tasks Found
> For each confirmed issue, provide:
> - **Title:** [action verb] [specific thing] (e.g., "Remove dead SandboxV1 handler")
> - **Confidence:** CONFIRMED | LIKELY
> - **Priority:** P0-P4
> - **Evidence:** [file path + line number + what you found + how you verified]
> - **Scope:** [estimated files to change]
>
> ### Areas Needing Deeper Spikes
> For each area that needs more investigation:
> - **Area:** [specific sub-area]
> - **Why:** [what suggests there is work here but you could not confirm]
> - **What to look for:** [specific questions the deeper spike should answer]
>
> ### Clean Areas
> - [area]: No issues found. [brief evidence -- what you read and why you're confident]
>
> ### Summary
> - Firm tasks: N (CONFIRMED: X, LIKELY: Y)
> - Deeper spikes needed: M
> - Clean areas: K
> ```

### After Each Spike Completes

1. **Review the spike report** for quality:
   - Are findings CONFIRMED with evidence, or just hedged guesses?
   - Did the agent actually read code, or just grep for patterns?
   - If quality is poor, note it in the spike's closing notes for future tuning

2. **Create firm task beads** from the "Firm Tasks Found" section:

```bash
bd create --title="[title from spike report]" --type=task --priority=[P level as 0-4] \
  --description="[confidence level]. [evidence and scope from spike report]"
bd dep add <epic-id> <task-id>
```

3. **Create new spike beads** from the "Areas Needing Deeper Spikes" section:

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

If new spikes were created, loop back and execute them. Continue until no new spikes are generated.

**Safety limit:** If total spikes executed exceeds 20, stop and report to the user. The goal may be too broad.

---

## Phase 3: Consolidate

After all spikes are complete and all firm tasks created, run the consolidation workflow to clean up the backlog before wiring dependencies.

### 3a. Run /consolidate

Instruct the user to run:

```
/consolidate [epic title or area]
```

This handles dedup, vertical slice audit, stale detection, and dependency cleanup — see `commands/consolidate.md` for details. Do not duplicate that logic here.

### 3b. Agent Assignment Hints

After consolidation, tag each firm task with the recommended agent type:

```bash
bd update <task-id> --notes="Recommended agent: domain-architect | infrastructure-implementer | api-developer | cli-developer | test-generator | refactorer | general-purpose"
```

---

## Phase 4: Prioritize and Wire Dependencies

### Cross-Task Dependencies

Review the firm tasks and add dependencies between them where order matters:

```bash
# If task B requires task A to be done first
bd dep add <task-B-id> <task-A-id>
```

Look for:
- Core/model changes that must precede integration or infrastructure changes
- Interface definitions that must precede implementations
- Data access layers before the services that consume them
- Backend logic before the UI or API surfaces that expose it
- Tasks that modify the same files (sequence them)

### Priority Review

Scan all created tasks and adjust priorities if the full picture reveals:
- A seemingly-P3 task that actually blocks several others (upgrade to P1)
- Multiple P1 tasks that could reasonably be P2
- Quick wins (small scope + high value) that deserve priority bumps
- Critical path tasks that should be P0

---

## Phase 5: Verify

Run these checks on the backlog before presenting it:

### 5a. DAG Check

Verify no dependency cycles exist. If A depends on B and B depends on A, one dependency is wrong -- fix it.

### 5b. Priority Consistency

P0/P1 tasks should not depend on P3/P4 tasks. If they do, upgrade the blocker's priority.

### 5c. Acceptance Criteria Audit

Every firm task should have at least one testable criterion in its description. If a task description is just "fix X", flesh it out with what "fixed" looks like.

### 5d. Critical Path Identification

Trace the longest dependency chain. This is the minimum time to epic completion. Flag it in the report.

---

## Phase 6: Report

Present the final blossom report to the user:

```markdown
## Blossom Report: [epic title]

### Epic
- **ID:** [epic-id]
- **Title:** [epic title]

### Exploration Summary
- **Total spikes executed:** N
- **Spike depth:** M levels (how many rounds of recursion)
- **Spike quality:** [brief assessment -- did agents confirm or hedge?]
- **Areas explored:** [list of top-level spike areas]
- **Clean areas:** [areas confirmed as needing no work]

### Consolidation Results
*(Include results from `/consolidate` run — see its report for dedup, gap-fill, and stale task counts)*

### Backlog

| ID | Title | Priority | Depends On | Agent | Confidence |
|----|-------|----------|------------|-------|------------|
| tehmop-xxx | Remove dead handler | P1 | - | refactorer | CONFIRMED |
| tehmop-yyy | Add missing tests | P2 | tehmop-xxx | test-generator | CONFIRMED |
| ... | ... | ... | ... | ... | ... |

### Critical Path
[Longest dependency chain with task IDs and titles]

### Parallel Opportunities
[Tasks that can be worked on simultaneously because they have no dependencies on each other]

### Statistics
- Firm tasks created: N
- P0 (critical): N
- P1 (high): N
- P2 (medium): N
- P3 (low): N
- P4 (backlog): N
- CONFIRMED: N
- LIKELY: N

### Recommended Execution Order
1. [First task to tackle and why]
2. [Second task]
3. ...

### Open Questions
[Any unresolved questions that came up during exploration]
```

---

## Session Close Reminder

Before finishing, run the session close protocol:

```bash
bd sync
git status
```

If there are beads state changes to commit:

```bash
git add .beads/
git commit -m "chore: blossom backlog for [goal description]"
```

---

## Key Principles

1. **Explore, don't implement.** Spikes discover work; they never do it.
2. **Verify, don't speculate.** Read the actual code. CONFIRMED findings over hedged guesses.
3. **Consolidate before reporting.** Run `/consolidate` to dedup, fill slice gaps, and resolve orphans.
4. **Epic depends on children.** Always `bd dep add <epic> <child>`, never the reverse.
5. **Batched dispatching.** Up to 4 concurrent spike agents. Process results as they arrive.
6. **Structured reports.** Spike agents must follow the exact report format for consistent processing.
7. **Depth limit.** Stop at 20 spikes and reassess with the user if the goal is too broad.
8. **Confidence levels.** Every finding is CONFIRMED, LIKELY, or POSSIBLE. Possible → deeper spike.
