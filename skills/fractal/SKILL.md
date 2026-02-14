---
name: fractal
description: "Run goal-directed recursive exploration that prunes paths not serving the goal. Use when you need to deeply understand something, explore a codebase area with focus, or decompose a vague objective into concrete findings. Keywords: explore, research, investigate, understand, deep-dive, discover."
argument-hint: "<goal>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Bash(mkdir:*), Task, Write
---

# Fractal: Goal-Directed Recursive Exploration

You are running **Fractal** -- goal-directed recursive exploration. Unlike exhaustive discovery, fractal evaluates every finding against the original goal and prunes paths that don't serve it. Think depth-first search with a fitness function: go deep where the goal lives, stop where it doesn't.

**Goal:** $ARGUMENTS

## When to Use

- When you need to deeply understand a specific codebase area or concept
- When a vague objective needs to be decomposed into concrete findings
- When exploration should stay focused on a goal (not exhaustive discovery)
- When you want recursive investigation with aggressive pruning of irrelevant paths
- When you need to answer specific questions rather than survey everything

## Phase 1: Seed

### 1a. Clarify

If `$ARGUMENTS` is empty or too vague for investigation, ask one clarifying question. Otherwise proceed. Fractal discovers scope through exploration, not upfront specification.

### 1b. Stash the Goal

Create a run directory to anchor the goal:

```bash
mkdir -p .fractal
```

Write `.fractal/goal.md`:

```markdown
# Goal
[the goal, restated clearly in 1-3 sentences]

## Dimensions
[2-4 specific questions this goal needs answered]

## Boundaries
[1-2 things explicitly out of scope, if obvious]
```

This file is the source of truth. Every handler receives the goal verbatim. Every evaluation checks findings against the dimensions.

### 1c. Decompose into Areas

Follow the **decompose** primitive's approach: break the goal into 2-4 MECE investigation areas with clear scope boundaries. Prefer depth over breadth -- fewer focused areas over many shallow ones. Each area must be independently investigable.

### 1d. Dispatch Handlers

Launch handlers via `Task(run_in_background=true, subagent_type="Explore")` using the handler template below. Launch up to 3 concurrently.

---

## Phase 2: Investigate and Evaluate

This phase repeats until a termination condition is met.

### Handler Template

Each handler receives this prompt (fill in bracketed values):

> You are a fractal handler investigating a specific area for a goal-directed exploration.
>
> **GOAL**: [verbatim from .fractal/goal.md -- unchanged at every depth]
>
> **YOUR AREA**: [specific investigation area]
>
> **DEPTH**: [current level, e.g., "Level 1" or "Level 2 (from: [parent area])"]
>
> Investigate your area by reading actual code. For each finding, assess whether it serves the GOAL.
>
> Report using pipe format:
>
> ## Investigated [your area]
>
> **Source**: /fractal (handler)
> **Input**: [your area]
>
> ### Items
>
> 1. **[finding title]** — [concrete finding with evidence]
>    - source: [file:line]
>    - confidence: CONFIRMED | LIKELY | POSSIBLE
>    - goal-fit: [1 sentence connecting finding to goal]
>
> ### Go Deeper
>
> Sub-areas that might serve the goal but need their own investigation:
> - **[sub-area]** — [what you saw that suggests value]
>   - question: [what the next handler should answer]
>
> ### Dead Ends
>
> - **[area]** — [why irrelevant]
>
> ### Summary
>
> [2-3 sentences. How well does this area serve the goal? Is the richest vein in Items or Go Deeper? Should the orchestrator prioritize depth here or move on?]
>
> Rules:
> - Read code. Do not speculate. Cite file paths.
> - Resolve what you can now. Do not defer to "Go Deeper" what you can answer.
> - If an area is irrelevant, say so in Dead Ends and move on.
> - Keep your report under 500 words.

### Goal-Fit Evaluation

After each handler report, evaluate findings using the **assess** primitive's categorical rubric (ANSWER / DEEPEN / PRUNE):

```markdown
## Assessed handler findings for goal-fit

**Source**: /fractal (assess)
**Input**: Round [N] evaluation against goal

### Items

1. **[finding or sub-area]** — **ANSWER** — [serves goal because...]
   - source: [from handler report]
2. **[sub-area]** — **DEEPEN** — [promising but unresolved...]
3. **[area]** — **PRUNE** — [tangential to goal...]

### Rubric

| Category | Definition |
|----------|------------|
| ANSWER   | Finding directly serves the goal. Add to synthesis. |
| DEEPEN   | Promising signal, unresolved. Dispatch a new handler. |
| PRUNE    | Not goal-relevant. Skip. |

### Summary

Accumulated answers: [count]. Handlers dispatched: [used]/12 max. Status: [CONTINUE | STOP: goal satisfied | STOP: diminishing returns].
```

### User Checkpoint (at depth transitions)

When the current depth level is exhausted (all handlers at Level N have reported) and DEEPEN items exist, pause before going deeper. Synthesize progress and confirm direction with the user.

**Checkpoint message:**

> **Progress so far:**
> - Accumulated answers: [count]
> - Remaining DEEPEN items: [count]
> - Current depth: Level [N]
> - Handlers dispatched: [count]/12
>
> **Next step:** Go to Level [N+1] to investigate [list DEEPEN areas, one line each].
>
> **Continue** deeper, **pivot** to different areas, or **conclude** now?

**User choices:**
- **Continue** → Proceed with recursion (see below). No change to goal or DEEPEN items.
- **Pivot** → Ask user which DEEPEN items to pursue (let them prune or redirect). Update the recursion plan accordingly.
- **Conclude** → Skip to Phase 3 synthesis with current ANSWER findings.

This checkpoint is a safety net, not a gate. If the user doesn't respond within a reasonable time or says "continue", proceed normally.

### Termination Conditions

Check after every evaluation. Stop when ANY is true:
1. **Goal satisfied** -- accumulated answers comprehensively address the goal dimensions
2. **Handler limit** -- 12 handlers dispatched total
3. **Diminishing returns** -- last 2+ reports produced mostly Dead Ends and PRUNE decisions
4. **Goal drift** -- remaining DEEPEN items don't connect back to any goal dimension

When stopping, proceed to Phase 3 even if DEEPEN items remain. Fractal's value is knowing when to stop.

### Recursion

If DEEPEN items exist and no termination condition is met, and the user checkpoint confirms continuation (if applicable), dispatch new handlers for them. Increment the depth level in their prompt. Continue until termination.

---

## Phase 3: Synthesize

Follow the **distill** primitive's approach: reduce all accumulated ANSWER findings into a final pipe-format report.

### 3a. Final Report

```markdown
## Fractal findings on [goal, short form]

**Source**: /fractal
**Input**: [goal, one line]

### Items

1. **[finding title]** — [detail, organized by goal dimension]
   - source: [file:line]
   - confidence: CONFIRMED | LIKELY | POSSIBLE

### Exploration Shape

- **Areas explored**: [list]
- **Max depth reached**: [N]
- **Handlers dispatched**: [N]
- **Dead ends**: [list of pruned areas]

### Open Questions

[Anything unresolved that the user should know about]

### Summary

[One paragraph synthesis of all findings relative to the goal.]
```

### 3b. Optional: Create Beads

If the findings suggest concrete follow-up work, ask the user if they want beads created. If yes:

```bash
bd create --title="[action from findings]" --type=task --priority=[0-4] \
  --description="From fractal exploration of [goal]. Evidence: [summary]"
```

### 3c. Cleanup

```bash
git add .fractal/
git status
```

---

## Guidelines

1. **Goal is king.** Every decision filters through "does this serve the goal?" The goal never changes during a run.
2. **Prune aggressively.** Interesting-but-tangential findings get PRUNE, not DEEPEN. Fractal rewards focus.
3. **Depth over breadth.** 3 deep investigations beat 6 shallow ones. Start with 2-4 areas, not 6+.
4. **Handlers are disposable.** They investigate and report. The orchestrator makes all strategic decisions.
5. **Stop when valuable.** "Done" is when the goal dimensions are addressed, not when exploration is exhausted.
6. **Show your reasoning.** The assess-style evaluation table makes pruning decisions visible and auditable.
7. **No beads during exploration.** Fractal produces understanding. Beads are an optional output, not the workflow.
8. **Density over length.** Handler reports under 500 words. Pipe format, not prose.
