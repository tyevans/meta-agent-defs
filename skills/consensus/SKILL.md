---
name: consensus
description: "Surface design trade-offs by having three independent agents propose solutions optimized for different qualities, then synthesize agreements and tensions. Use for architectural decisions, when torn between approaches, or when you suspect hidden trade-offs. Keywords: consensus, architecture, trade-off, design, decision, compare."
argument-hint: "<design problem or architectural decision>"
disable-model-invocation: true
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Task, AskUserQuestion
context: fork
---

# Consensus: Multi-Perspective Design Synthesis

You are running the **Consensus** workflow -- a multi-perspective synthesis pattern that surfaces design trade-offs by having three independent agents solve the same problem optimized for different quality attributes. Design problem: **$ARGUMENTS**

## When to Use

- Making architectural decisions where multiple valid approaches exist
- When torn between competing designs and unsure what you're trading off
- When you suspect hidden complexity or unrecognized trade-offs
- Before committing to a significant refactoring or new feature approach
- When a design discussion has stalled because all options seem equally good (or bad)

## Overview

Consensus works in 5 phases:

```
Frame the problem (same starting point for all agents)
  -> Solicit 3 perspectives (simplicity, performance, maintainability)
    -> Synthesize (agreements = high confidence, tensions = actual trade-offs)
      -> Present report with recommendation
        -> Record decision for future context
```

---

## Phase 1: Frame the Problem

### 1a. Parse Arguments

If `$ARGUMENTS` is empty, ask the user one clarifying question to understand the decision being made. If `$ARGUMENTS` is vague (e.g., "improve performance"), ask for specifics: which component, what metric, what constraints?

Otherwise, proceed immediately.

### 1b. Explore Context

Use Glob and Read to explore the relevant codebase areas:

- What files/modules are relevant to this decision?
- What is the current implementation (if any)?
- What constraints exist (tech stack, existing patterns, performance requirements, compatibility)?
- What similar decisions have been made elsewhere in this codebase?

Do not spend more than 5 minutes on exploration. The goal is enough context to write a clear problem statement, not exhaustive analysis.

### 1c. Write the Problem Statement

Draft a structured problem statement that will be given to all three agents identically:

```markdown
## Problem Statement

**What needs to be decided**: [the core question being answered]

**Current state**: [brief description of what exists today, if anything]

**Constraints**:
- [technical constraint, e.g., "must integrate with existing EventBus"]
- [performance constraint, e.g., "sub-100ms latency required"]
- [compatibility constraint, e.g., "cannot break public API"]
- [resource constraint, e.g., "team has limited frontend expertise"]

**Success criteria**: [what "solved" looks like -- concrete and testable]

**Relevant files/areas**:
- [file or directory path with brief description]
- [file or directory path with brief description]
```

**Critical**: All three agents receive this exact same problem statement. The only difference between agents is their optimization lens (see Phase 2).

---

## Phase 2: Solicit Perspectives

### 2a. Dispatch Three Agents in Parallel

Use the Task tool to spawn three background agents with `subagent_type: "general-purpose"` and `run_in_background: true`. Give each agent the same problem statement, but different optimization instructions.

**Agent 1: Simplicity Advocate**

> You are the **Simplicity Advocate**. Your job is to propose the simplest possible solution to the problem below.
>
> **Optimization lens**: Minimize complexity. Fewer moving parts, fewer abstractions, less indirection. Prefer boring technology. Ask: "What's the least complex way to solve this?"
>
> [Problem Statement from Phase 1]
>
> **Your deliverables**:
> 1. **Proposed approach** (concrete, not abstract -- describe exactly what you would do)
> 2. **Key design decisions and why**
> 3. **Trade-offs you're accepting** (what you're giving up by optimizing for simplicity)
> 4. **Files you would touch** (specific paths and what changes)
> 5. **Estimated complexity** (simple / moderate / complex)
>
> Before proposing, READ the relevant files to understand the current state. Do not guess.

**Agent 2: Performance Advocate**

> You are the **Performance Advocate**. Your job is to propose the most performant solution to the problem below.
>
> **Optimization lens**: Maximize runtime performance, resource efficiency, and scalability. Minimize latency, memory usage, and computational overhead. Ask: "What's the fastest/most efficient way to solve this?"
>
> [Problem Statement from Phase 1]
>
> **Your deliverables**:
> 1. **Proposed approach** (concrete, not abstract -- describe exactly what you would do)
> 2. **Key design decisions and why**
> 3. **Trade-offs you're accepting** (what you're giving up by optimizing for performance)
> 4. **Files you would touch** (specific paths and what changes)
> 5. **Estimated complexity** (simple / moderate / complex)
>
> Before proposing, READ the relevant files to understand the current state. Do not guess.

**Agent 3: Maintainability Advocate**

> You are the **Maintainability Advocate**. Your job is to propose the most maintainable solution to the problem below.
>
> **Optimization lens**: Maximize long-term maintainability, testability, and extensibility. Optimize for code that will be easy to debug, evolve, and onboard new developers to a year from now. Ask: "What will be easiest to understand and change over time?"
>
> [Problem Statement from Phase 1]
>
> **Your deliverables**:
> 1. **Proposed approach** (concrete, not abstract -- describe exactly what you would do)
> 2. **Key design decisions and why**
> 3. **Trade-offs you're accepting** (what you're giving up by optimizing for maintainability)
> 4. **Files you would touch** (specific paths and what changes)
> 5. **Estimated complexity** (simple / moderate / complex)
>
> Before proposing, READ the relevant files to understand the current state. Do not guess.

### 2b. Wait for All Three Reports

Allow all three agents to complete. Do not proceed until all three have returned their proposals.

---

## Phase 3: Synthesize

### 3a. Review Proposals

Read all three agent reports thoroughly. For each agent, extract:
- Core approach (what they're proposing)
- Key decisions (why they chose this approach)
- Trade-offs (what they're sacrificing)
- Files touched
- Complexity estimate

### 3b. Identify Agreements

Compare the three proposals to find areas where **all three agents agree**. These are high-confidence recommendations.

Examples of agreements:
- All three suggest using the same data structure
- All three recommend touching the same core files
- All three agree a certain abstraction is unnecessary
- All three converge on the same integration point

Create an **Agreements** list with each point of consensus and what it tells you:

```markdown
### Agreements (HIGH CONFIDENCE)

- **All three agents propose [X]**: This is very likely the right choice.
- **All three avoid [Y]**: Strong signal that [Y] is the wrong approach.
- **All three touch [file path]**: This file is central to the solution.
```

### 3c. Identify Tensions

Find areas where agents **disagree** or propose **conflicting approaches**. These are the actual design tensions.

For each tension, articulate what the trade-off is:

```markdown
### Tension: [Brief title]

**Simplicity says**: [approach] because [reason]
**Performance says**: [approach] because [reason]
**Maintainability says**: [approach] because [reason]

**The actual trade-off**: [what you gain and lose with each choice]
```

Example:

```markdown
### Tension: In-memory cache vs. database query

**Simplicity says**: Query the database every time because it requires no cache management code or invalidation logic
**Performance says**: Pre-load all data into an in-memory map on startup because database round-trips add 50-100ms per request
**Maintainability says**: Use a TTL-based cache with automatic invalidation because it handles updates gracefully and is testable

**The actual trade-off**: Simplicity minimizes code but sacrifices performance. Performance maximizes speed but adds staleness risk. Maintainability balances both but adds a dependency and configuration complexity.
```

### 3d. Identify Majority Positions

Find cases where **2 of 3 agents agree**. These are likely good choices, but note the dissenting opinion.

```markdown
### Majority: [Brief title]

**2 agents (Simplicity + Maintainability) propose**: [approach]
**1 agent (Performance) dissents**: [concern or alternative]

**Interpretation**: [what this tells you]
```

---

## Phase 4: Present Report

### 4a. Synthesize Recommendation

Based on agreements, tensions, and the problem constraints, propose which approach best fits the situation.

**Decision rubric**:
- If constraints include performance requirements, weight the Performance advocate more heavily
- If constraints include complexity limits or team expertise gaps, weight the Simplicity advocate more heavily
- If constraints include long-term evolution or extensibility, weight the Maintainability advocate more heavily
- Agreements override individual preferences -- if all three converge, that's the answer

### 4b. Structure the Report

Present the synthesis to the user:

```markdown
## Consensus Report: [Problem Title]

### Problem Statement
[Restate the problem from Phase 1]

### Three Proposals

**Simplicity Advocate**:
- Approach: [1-2 sentence summary]
- Complexity: [simple/moderate/complex]
- Key trade-offs: [what's sacrificed]

**Performance Advocate**:
- Approach: [1-2 sentence summary]
- Complexity: [simple/moderate/complex]
- Key trade-offs: [what's sacrificed]

**Maintainability Advocate**:
- Approach: [1-2 sentence summary]
- Complexity: [simple/moderate/complex]
- Key trade-offs: [what's sacrificed]

---

### Agreements (HIGH CONFIDENCE)

| What | Why it Matters |
|------|----------------|
| [point of consensus] | [implication] |
| [point of consensus] | [implication] |

---

### Tensions (REQUIRES HUMAN DECISION)

#### Tension 1: [title]
**Simplicity**: [position and reason]
**Performance**: [position and reason]
**Maintainability**: [position and reason]

**Trade-off**: [what you gain and lose with each]

#### Tension 2: [title]
[same structure]

---

### Recommendation

**Approach**: [which agent's proposal, or hybrid of multiple]

**Rationale**: [why this approach best fits the constraints and success criteria]

**What you're choosing**:
- [benefit 1]
- [benefit 2]

**What you're accepting**:
- [trade-off 1]
- [trade-off 2]

**Critical dependencies**: [anything that must be true for this to work]

---

### Next Steps

1. [immediate next action -- usually "decide on unresolved tensions" or "proceed to /spec"]
2. [second action]
3. [third action]
```

### 4c. Ask for Decision on Unresolved Tensions

If there are tensions where no clear winner exists based on constraints, use AskUserQuestion to get human input:

```
We have a design tension that requires your decision:

[Present the tension with trade-offs]

Which approach do you prefer, or should we explore a hybrid?
```

Do not proceed past this point until the user resolves critical tensions.

---

## Phase 5: Record Decision

### 5a. Create a Beads Task

Capture the decision in the backlog so future sessions have context:

```bash
bd create --title="DECISION: [brief description of what was decided]" --type=task --priority=2 \
  --description="Consensus workflow on [problem]. Decision: [chosen approach]. Rationale: [1-2 sentences]. Key trade-offs accepted: [list]. Next: [/spec or implementation or further exploration]."
```

### 5b. Note Follow-Up Work

If the decision leads directly to implementation, note which skill should run next:

- **If implementation is straightforward**: Create task(s) for the work and assign to appropriate agent
- **If implementation needs detailed planning**: Recommend `/spec [chosen approach]`
- **If implementation needs incremental exploration**: Recommend `/tracer [starting point]`

### 5c. Session Close Reminder

```bash
bd sync
git status
```

If there are beads state changes to commit:

```bash
git add .beads/
git commit -m "chore: consensus decision on [problem title]"
```

---

## Guidelines

- **Same starting point, different lenses**: All three agents get identical problem statements. The ONLY difference is their optimization goal.
- **Fresh agents prevent contamination**: Using separate background agents ensures one perspective doesn't influence another.
- **Agreements are gold**: Where all three converge, that's a high-confidence decision. Do it.
- **Tensions are the point**: Disagreements surface actual trade-offs the user might not have considered. Make them explicit.
- **Recommend honestly**: Your recommendation should acknowledge what's being sacrificed, not just what's being gained.
- **This is expensive**: Three parallel agents reading the codebase independently. Use this for meaningful decisions, not trivial choices.
- **This produces a DECISION, not an IMPLEMENTATION**: Consensus answers "which approach" but not "exactly how". Pair with `/spec` or `/tracer` for execution.
- **Concrete over abstract**: Reject agent proposals that are too high-level. Demand specific files, specific changes, specific trade-offs.
- **Verify agents read code**: If an agent's proposal doesn't reference actual file contents, challenge it. Speculation is not useful here.

---

## Key Principles

1. **Same input, different optimization**: Identical problem statement ensures apples-to-apples comparison
2. **Isolation prevents anchoring**: Fresh background agents can't see each other's reasoning
3. **Agreement = high confidence**: Convergence from independent optimizations is a strong signal
4. **Tension = real trade-off**: Disagreement reveals what you're actually choosing between
5. **Human decides ties**: When no approach dominates, the user must weigh constraints and priorities
6. **Decision before implementation**: This workflow produces a choice, not a solution. Follow with /spec or direct implementation.
7. **Quality over speed**: Three agents reading the codebase is slow. That's the price of multi-perspective synthesis.
8. **Record the decision**: Future sessions need to know not just what was chosen, but why and what was rejected.
