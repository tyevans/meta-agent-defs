---
name: consensus
description: "Surface design trade-offs by having three independent agents propose solutions optimized for different qualities, then synthesize agreements and tensions. Use for architectural decisions, when torn between approaches, or when you suspect hidden trade-offs. Keywords: consensus, architecture, trade-off, design, decision, compare."
argument-hint: "<design problem or architectural decision>"
disable-model-invocation: false
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

Consensus works in 5 phases (with an optional debate round):

```
Frame the problem (same starting point for all agents)
  -> Select 3 optimization lenses (adaptive based on problem type)
    -> Solicit 3 perspectives (one per lens)
      -> [OPTIONAL] Debate round (cross-proposal challenge)
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

### 1d. Select Optimization Lenses

Choose the 3 lenses that will produce the most productive tension for this specific problem. Use the lens palette below:

**Lens Palette**:
- **Simplicity**: Minimize complexity, fewer moving parts, less indirection, boring technology
- **Performance**: Maximize speed/throughput, minimize latency, optimize resource efficiency
- **Maintainability**: Optimize for long-term code health, testability, debuggability, future extensibility
- **Security**: Minimize attack surface, reduce vulnerability exposure, defense in depth
- **Velocity**: Ship faster, reduce ceremony, minimize coordination overhead
- **Consistency**: Follow existing patterns, maintain architectural coherence, reduce surprises
- **Resilience**: Handle failures gracefully, degrade predictably, recover automatically
- **Usability**: Optimize developer/user experience, reduce cognitive load, minimize friction

**Selection criteria:**

1. Analyze the problem statement and constraints to identify which lenses are most relevant
2. Pick 3 lenses that will **disagree productively** -- never select two lenses that optimize for the same underlying value
3. Default to **Simplicity/Performance/Maintainability** if the problem doesn't strongly suggest specific alternatives
4. Consider constraint-driven selection:
   - Performance requirements in constraints → include Performance
   - Security/compliance requirements → include Security
   - Team velocity pressure → include Velocity
   - Large existing codebase → include Consistency
   - User-facing features → include Usability
   - Distributed systems / failure scenarios → include Resilience

**Present the selection** to the user in one line per lens:

```
Selected lenses:
- [Lens 1]: [1-sentence description from palette]
- [Lens 2]: [1-sentence description from palette]
- [Lens 3]: [1-sentence description from palette]
```

Do not proceed to Phase 2 until lens selection is complete.

---

## Phase 2: Solicit Perspectives

### 2a. Dispatch Three Agents in Parallel

Use the Task tool to spawn three background agents with `subagent_type: "general-purpose"` and `run_in_background: true`. Give each agent the same problem statement, but different optimization instructions based on the lenses selected in Phase 1d.

**Agent Prompt Template** (instantiate once per selected lens):

> You are the **[Lens Name] Advocate**. Your job is to propose a solution to the problem below that maximizes [lens name].
>
> **Optimization lens**: [Lens description from palette]. [Lens-specific guidance -- see table below]
>
> [Problem Statement from Phase 1]
>
> Follow the Agent Preamble from fan-out-protocol for investigation protocol.
>
> **Your deliverables**:
> 1. **Proposed approach** (concrete, not abstract -- describe exactly what you would do)
> 2. **Key design decisions and why**
> 3. **Trade-offs you're accepting** (what you're giving up by optimizing for [lens name])
> 4. **Files you would touch** (specific paths and what changes)
> 5. **Estimated complexity** (simple / moderate / complex)

**Lens-Specific Guidance** (add to agent prompt based on selected lens):

| Lens | What to Look For |
|------|------------------|
| Simplicity | Fewer moving parts, fewer abstractions, less indirection. Prefer boring technology. Ask: "What's the least complex way to solve this?" |
| Performance | Maximize runtime performance, resource efficiency, and scalability. Minimize latency, memory usage, and computational overhead. Ask: "What's the fastest/most efficient way to solve this?" |
| Maintainability | Optimize for long-term code health, testability, and extensibility. Optimize for code that will be easy to debug, evolve, and onboard new developers to a year from now. Ask: "What will be easiest to understand and change over time?" |
| Security | Minimize attack surface, validate all inputs, assume breach, defense in depth. Ask: "What could go wrong and how do we prevent it?" |
| Velocity | Minimize ceremony, reduce coordination overhead, use off-the-shelf solutions. Ask: "What's the fastest path to shipping a working solution?" |
| Consistency | Follow existing patterns, maintain architectural coherence, reduce surprises for future developers. Ask: "What would a developer familiar with this codebase expect?" |
| Resilience | Handle failures gracefully, degrade predictably, design for recovery. Ask: "What happens when this fails and how do we recover?" |
| Usability | Reduce cognitive load, minimize friction, optimize for intuitive workflows. Ask: "How do we make this obvious and easy to use?" |

### 2b. Wait for All Three Reports

Allow all three agents to complete. Do not proceed until all three have returned their proposals.

---

## Phase 2.5: Debate Round (OPTIONAL)

After collecting all three initial proposals, ask the user if they want to run a debate round:

```
Run debate round? (adds cost but surfaces deeper trade-offs)

Debate round: Each advocate will see anonymized summaries of the other proposals and can revise their approach or challenge competing ideas. This adds 3 more agent dispatches but often reveals trade-offs that isolated analysis misses.

Y/N?
```

If the user declines, skip directly to Phase 3.

If the user approves:

### 2.5a. Anonymize Proposals

Create a summary of each proposal that removes lens-identifying language:

```markdown
**Proposal A**:
- Approach: [1-2 sentence summary, neutral language]
- Complexity: [simple/moderate/complex]
- Key decisions: [bullet list]

**Proposal B**:
- Approach: [1-2 sentence summary, neutral language]
- Complexity: [simple/moderate/complex]
- Key decisions: [bullet list]

**Proposal C**:
- Approach: [1-2 sentence summary, neutral language]
- Complexity: [simple/moderate/complex]
- Key decisions: [bullet list]
```

**Critical**: Do not reveal which lens generated which proposal. Use neutral labels (A/B/C) and remove any lens-specific language ("to maximize simplicity", "for better performance", etc.). The goal is to let advocates respond to the IDEAS, not to known biases.

### 2.5b. Dispatch Debate Agents

Dispatch three NEW background agents (one per original lens). Each agent receives:

1. The original problem statement from Phase 1
2. Their own original proposal (fully detailed)
3. The anonymized summaries of the other two proposals (Proposal A/B/C)

**Debate Agent Prompt Template**:

> You are the **[Lens Name] Advocate** revisiting your proposal after seeing competing approaches.
>
> **Your original proposal**:
> [Full original proposal text]
>
> **Competing proposals** (anonymized):
> [Proposal A/B/C summaries -- excluding the agent's own proposal]
>
> **Original problem statement**:
> [Problem Statement from Phase 1]
>
> **Your task**:
> 1. **What you'd revise**: After seeing the alternatives, what would you change about your original proposal? Be specific. If nothing, say why your approach still holds.
> 2. **Weaknesses you see**: What concerns or risks do you identify in the competing proposals? What are they missing or getting wrong?
> 3. **Potential hybrid**: Is there a way to combine ideas from multiple proposals that would be superior to any single approach?
>
> Be concrete and constructive. The goal is productive disagreement, not point-scoring.

### 2.5c. Wait for Debate Results

Allow all three debate agents to complete. Do not proceed until all three have returned their responses.

---

## Phase 3: Synthesize

### 3a. Review Proposals

Read all three initial agent reports thoroughly. For each agent, extract:
- Core approach (what they're proposing)
- Key decisions (why they chose this approach)
- Trade-offs (what they're sacrificing)
- Files touched
- Complexity estimate

**If debate round ran**: Also review the debate responses to identify:
- Revisions each advocate made to their approach
- Cross-proposal critiques and concerns raised
- Potential hybrid approaches suggested
- Points of convergence revealed through dialogue

### 3b. Identify Agreements

Compare the three proposals (including debate revisions if available) to find areas where **all three agents agree**. These are high-confidence recommendations.

Examples of agreements:
- All three suggest using the same data structure
- All three recommend touching the same core files
- All three agree a certain abstraction is unnecessary
- All three converge on the same integration point
- **If debate ran**: Advocates revised toward a common approach
- **If debate ran**: All three identified the same weakness in a competing idea

Create an **Agreements** list with each point of consensus and what it tells you:

```markdown
### Agreements (HIGH CONFIDENCE)

- **All three agents propose [X]**: This is very likely the right choice.
- **All three avoid [Y]**: Strong signal that [Y] is the wrong approach.
- **All three touch [file path]**: This file is central to the solution.
```

**If debate ran**: Note any convergence that emerged during the debate round that wasn't present in initial proposals. This is particularly strong signal.

### 3c. Identify Tensions

Find areas where agents **disagree** or propose **conflicting approaches**. These are the actual design tensions.

For each tension, articulate what the trade-off is. **If debate ran**, incorporate critiques and revisions into the tension analysis:

```markdown
### Tension: [Brief title]

**[Lens 1] says**: [approach] because [reason]
  - **After debate**: [revision made, if any, or "stood firm because..."]
  - **Critiques raised**: [concerns about competing proposals]

**[Lens 2] says**: [approach] because [reason]
  - **After debate**: [revision made, if any, or "stood firm because..."]
  - **Critiques raised**: [concerns about competing proposals]

**[Lens 3] says**: [approach] because [reason]
  - **After debate**: [revision made, if any, or "stood firm because..."]
  - **Critiques raised**: [concerns about competing proposals]

**The actual trade-off**: [what you gain and lose with each choice, incorporating debate insights]
```

Example (using Simplicity/Performance/Maintainability lenses, with debate):

```markdown
### Tension: In-memory cache vs. database query

**Simplicity says**: Query the database every time because it requires no cache management code or invalidation logic
  - **After debate**: Acknowledged staleness isn't a concern for this use case, but still prefers no cache
  - **Critiques raised**: Performance approach adds complexity for marginal gains given query frequency

**Performance says**: Pre-load all data into an in-memory map on startup because database round-trips add 50-100ms per request
  - **After debate**: Revised to lazy-load with TTL after seeing Maintainability's invalidation concerns
  - **Critiques raised**: Simplicity's no-cache approach will bottleneck at scale

**Maintainability says**: Use a TTL-based cache with automatic invalidation because it handles updates gracefully and is testable
  - **After debate**: Stood firm; both alternatives have documented downsides
  - **Critiques raised**: Simplicity's approach ignores performance constraints, Performance's approach ignores update patterns

**The actual trade-off**: Simplicity minimizes code but sacrifices performance. Performance maximizes speed but adds staleness risk (now mitigated by TTL). Maintainability balances both but adds a dependency and configuration complexity. **Debate revealed**: All three agree updates are infrequent enough that staleness is manageable, narrowing the actual choice to code complexity vs. performance headroom.
```

**If debate ran**: Pay attention to revisions that narrow the design space -- when advocates adjust their positions after seeing alternatives, the remaining disagreement is more precisely defined.

### 3d. Identify Majority Positions

Find cases where **2 of 3 agents agree**. These are likely good choices, but note the dissenting opinion.

**If debate ran**: Check whether majority positions strengthened (dissenter revised toward majority) or weakened (majority advocate(s) acknowledged dissenter's concerns).

```markdown
### Majority: [Brief title]

**2 agents ([Lens A] + [Lens B]) propose**: [approach]
**1 agent ([Lens C]) dissents**: [concern or alternative]

**Interpretation**: [what this tells you]

**If debate ran**: [convergence/divergence observed]
```

---

## Phase 4: Present Report

### 4a. Synthesize Recommendation

Based on agreements, tensions, and the problem constraints, propose which approach best fits the situation.

**Decision rubric**:
- Weight advocates according to how strongly the constraints align with their lens (e.g., if constraints include performance requirements, weight the Performance advocate more heavily)
- Agreements override individual preferences -- if all three converge, that's the answer
- When lenses conflict, the constraint hierarchy determines priority (explicit requirements trump aspirational qualities)

### 4b. Structure the Report

Present the synthesis to the user:

```markdown
## Consensus Report: [Problem Title]

### Problem Statement
[Restate the problem from Phase 1]

### Selected Lenses
- [Lens 1]: [description]
- [Lens 2]: [description]
- [Lens 3]: [description]

### Three Proposals

**[Lens 1] Advocate**:
- Initial approach: [1-2 sentence summary]
- Complexity: [simple/moderate/complex]
- Key trade-offs: [what's sacrificed]
- **If debate ran**: [revisions made or "held firm"]

**[Lens 2] Advocate**:
- Initial approach: [1-2 sentence summary]
- Complexity: [simple/moderate/complex]
- Key trade-offs: [what's sacrificed]
- **If debate ran**: [revisions made or "held firm"]

**[Lens 3] Advocate**:
- Initial approach: [1-2 sentence summary]
- Complexity: [simple/moderate/complex]
- Key trade-offs: [what's sacrificed]
- **If debate ran**: [revisions made or "held firm"]

---

### Agreements (HIGH CONFIDENCE)

| What | Why it Matters |
|------|----------------|
| [point of consensus] | [implication] |
| [point of consensus] | [implication] |

---

### Tensions (REQUIRES HUMAN DECISION)

#### Tension 1: [title]
**[Lens 1]**: [position and reason]
**[Lens 2]**: [position and reason]
**[Lens 3]**: [position and reason]

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

---

## Guidelines

- **Same starting point, different lenses**: All three agents get identical problem statements. The ONLY difference is their optimization goal.
- **Adaptive lenses create relevant tension**: Select lenses based on the problem type and constraints. Security decisions need security-vs-velocity lenses, not simplicity-vs-performance.
- **Never pick lenses that agree**: Follow meeting's principle -- never select two lenses that optimize for the same underlying value. Tension is productive.
- **Default to classic trio when unclear**: If the problem doesn't strongly suggest specific lenses, use Simplicity/Performance/Maintainability.
- **Fresh agents prevent contamination**: Using separate background agents ensures one perspective doesn't influence another.
- **Agreements are gold**: Where all three converge, that's a high-confidence decision. Do it.
- **Tensions are the point**: Disagreements surface actual trade-offs the user might not have considered. Make them explicit.
- **Recommend honestly**: Your recommendation should acknowledge what's being sacrificed, not just what's being gained.
- **This is expensive**: Three parallel agents reading the codebase independently. Use this for meaningful decisions, not trivial choices. Debate round doubles the agent count to six.
- **Debate round surfaces deeper trade-offs**: The optional debate round adds cost but reveals tensions that isolated analysis often misses. Use it for high-stakes decisions where understanding second-order effects matters.
- **This produces a DECISION, not an IMPLEMENTATION**: Consensus answers "which approach" but not "exactly how". Pair with `/spec` or `/tracer` for execution.
- **Concrete over abstract**: Reject agent proposals that are too high-level. Demand specific files, specific changes, specific trade-offs.
- **Verify agents read code**: If an agent's proposal doesn't reference actual file contents, challenge it. Speculation is not useful here.

---

## Key Principles

1. **Same input, different optimization**: Identical problem statement ensures apples-to-apples comparison
2. **Adaptive lenses surface relevant tensions**: Problem-specific lens selection ensures disagreements reveal actual trade-offs, not artificial ones
3. **Isolation prevents anchoring**: Fresh background agents can't see each other's reasoning
4. **Agreement = high confidence**: Convergence from independent optimizations is a strong signal
5. **Tension = real trade-off**: Disagreement reveals what you're actually choosing between
6. **Human decides ties**: When no approach dominates, the user must weigh constraints and priorities
7. **Decision before implementation**: This workflow produces a choice, not a solution. Follow with /spec or direct implementation.
8. **Quality over speed**: Three agents reading the codebase is slow. That's the price of multi-perspective synthesis.
9. **Record the decision**: Future sessions need to know not just what was chosen, but why and what was rejected.
