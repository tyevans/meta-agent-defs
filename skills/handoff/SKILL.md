---
name: handoff
description: "Capture the orchestrator's mental model for a structured session transition. Use at end of long sessions, when /session-health recommends a fresh session, or before switching workstreams."
argument-hint: "[focus area]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*)
---

!`bd list --status=in_progress 2>/dev/null`
!`git log --oneline -10`

# Handoff: Structured Session Transition

You are running a **session handoff** -- capturing the orchestrator's mental model so a future session can pick up where this one left off. Focus area (optional): **$ARGUMENTS**

## When to Use

- When /session-health recommends "fresh session" and you want a clean transition
- At the end of a long work session before signing off
- When switching to a different workstream and want to preserve current context
- Before a planned break where someone else (or future-you) will continue

## Overview

Handoff works in 4 phases:

```
Gather state (backlog + git + context)
  -> Document decisions and discoveries
    -> Identify loose ends and risks
      -> Write handoff note
```

---

## Phase 1: Gather State

### 1a. Backlog Snapshot

```bash
bd stats
bd list --status=in_progress
bd ready
bd blocked
```

Note which tasks are in-progress, what is blocked, and what is ready to pick up.

### 1b. Working Tree State

```bash
git status
git log --oneline -10
git stash list
```

Note any uncommitted changes, recent commits made this session, and stashed work.

### 1c. Session Scope

Review what you set out to do this session and what actually happened:
- **Original goal**: What did the user first ask for?
- **What got done**: Which tasks were completed?
- **What shifted**: Did scope change? Why?

---

## Phase 2: Document Decisions

### 2a. Key Decisions Made

List every significant decision made during this session:
- What was decided
- Why (the reasoning, not just the outcome)
- What alternatives were considered and rejected

These are the decisions most likely to be re-litigated in a fresh session without context.

### 2b. Patterns Discovered

Note any patterns, conventions, or codebase behaviors discovered during exploration:
- Surprising code structures or relationships
- Undocumented conventions that guided decisions
- Gotchas or edge cases encountered

### 2c. Mental Model

Capture the high-level understanding built up during this session:
- How do the pieces fit together?
- What is the current theory about the right approach?
- What assumptions are being made?

---

## Phase 3: Identify Loose Ends

### 3a. Unfinished Work

For each in-progress task:
- What has been done so far
- What remains
- Where exactly to pick up (specific file, function, or step)

### 3b. Known Risks

Flag anything that could cause problems:
- Fragile code that was touched but not fully tested
- Dependencies that might shift
- Timing-sensitive work (PRs awaiting review, deploys in flight)

### 3c. Open Questions

List unresolved questions that came up during the session:
- Questions that were deferred
- Questions that need input from someone else
- Questions that need more investigation

**Sharpening gate:** Transform vague questions into decision frameworks. For each question:
1. **Name the specific code/component** where the question arose
2. **State what decision is needed** with concrete options and constraints
3. **Make it decidable** — include the criteria needed to resolve it

Example transformation:
- **Before:** "Need to figure out the right caching strategy"
- **After:** "Cache provider choice for `src/cache/provider.ts`: Redis (latency <10ms, requires infra) vs in-memory (no infra, scale limit at ~1M records). Decision criteria: expected data volume and ops budget. Ask: product team for volume projections"

---

## Phase 4: Write Handoff Note

Produce a structured handoff document:

```markdown
## Session Handoff: [date or topic]

### What Got Done
- [completed task/outcome 1]
- [completed task/outcome 2]

### Key Decisions
- **[Decision]**: [reasoning] (rejected: [alternatives])

### Patterns & Discoveries
- [pattern/discovery with evidence]

### In-Progress Work
- **[task/bead ID]**: [status, where to pick up, what remains]

### Uncommitted Changes
- [file or area]: [what changed and why it is not yet committed]

### Blocked Work
- **[task/bead ID]**: blocked on [reason]

### Open Questions
- **[Component/file]**: [decision needed] — Options: [A (pros/cons), B (pros/cons)]. Criteria: [what determines choice]. Ask: [who or what investigation resolves this]

### Recommended Next Steps
1. [Highest priority action with specific file/function/command to run]
2. [Second priority with pickup point]
3. [Third priority with context]

### Risks & Warnings
- [anything the next session should watch out for]
```

### 4a. Persist the Handoff

If `$ARGUMENTS` specifies a focus area, use it as the filename context. Write the handoff note to the project's memory:

```bash
bd create --title="HANDOFF: [summary of session]" --type=task --priority=1 \
  --description="[full handoff note content]"
```

If the project has a `memory/` directory, optionally write the handoff note there as `memory/handoff-[date].md` for persistent reference.

---

## Guidelines

- **Capture reasoning, not just outcomes** -- the next session can see what was done from git log, but not why
- **Be specific about pickup points** -- "continue working on X" is useless; "open file Y at line Z, the remaining cases are A, B, C" is actionable
- **Don't over-document** -- focus on context that would be lost, not things that are obvious from the code
- **Commit before handing off** -- uncommitted work is the biggest risk in a session transition
- **When in doubt, write it down** -- it is cheaper to ignore a handoff note than to rediscover lost context
