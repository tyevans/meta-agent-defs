---
name: retro
description: "Run an automated session retrospective to evaluate velocity, quality, process, and blockers, then persist durable learnings to MEMORY.md. Use at the end of a work session, after completing an epic, or when you want to reflect on what worked and what did not."
argument-hint: "[focus area or session topic]"
disable-model-invocation: true
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Write, Edit
---

# Retro: Automated Session Retrospective

You are running a **session retrospective** -- a structured reflection on the current session to extract durable learnings and update project memory. Focus area (optional): **$ARGUMENTS**

## When to Use

- At the end of a work session before signing off
- After completing an epic or major milestone
- When a session felt unusually productive or unusually rough and you want to capture why
- When the user asks "what did we learn?" or "how did that go?"

## Overview

Retro works in 5 phases:

```
Gather session data (git + backlog + conversation context)
  -> Analyze across 5 dimensions
    -> Extract keep/stop/try learnings
      -> Update MEMORY.md with durable insights
        -> Present structured report
```

---

## Phase 1: Gather Session Data

### 1a. Git Activity

```bash
git log --oneline -20
```

Note which commits were made this session, how many files were touched, and the types of changes (feat, fix, refactor, docs, chore).

### 1b. Backlog Activity

```bash
bd stats
bd list --status=closed
```

Note how many tasks were completed, what types they were, and whether any were closed as duplicates or stale (vs. genuinely completed).

### 1c. Conversation Context

Review the current conversation for:
- **Pivots**: Points where the plan changed direction
- **Blockers**: Things that stalled progress (API errors, unclear requirements, missing tools)
- **Rework**: Tasks that had to be redone or corrected
- **Surprises**: Unexpected findings, both positive and negative

---

## Phase 2: Analyze

Evaluate the session across these five dimensions:

### Velocity

- How many tasks were completed?
- What was the ratio of planned work to discovered work?
- Were any tasks significantly larger or smaller than expected?
- How much time was spent on overhead (setup, debugging tools, context gathering) vs. actual work?

### Quality

- Did agents produce CONFIRMED findings or hedge with LIKELY/POSSIBLE?
- Were there rework cycles where output had to be corrected?
- Did commits pass on the first try or were there hook failures?
- Was code read and verified before changes, or were assumptions made?

### Process

- Did the dispatch strategy work? Was serialization effective, or did it bottleneck?
- If teams were used, did parallelization pay off or cause coordination overhead?
- Did the orchestrator stay in orchestration mode or get pulled into implementation?
- Were the right agents/skills used for the right tasks?

### Blockers

- What slowed things down?
- Categories to check: API throttling, unclear requirements, missing tools or skills, context window pressure, dependency chains, bugs in tooling
- Which blockers were resolved vs. worked around vs. still open?

### Discoveries

- What unexpected things were found?
- New patterns or anti-patterns in the codebase or workflow?
- New capabilities or limitations of tools/agents discovered?
- Insights about project architecture or domain that were not documented before?

---

## Phase 3: Extract Learnings

For each dimension from Phase 2, identify:

- **Keep doing**: What worked well and should continue
- **Stop doing**: What was wasteful, harmful, or counterproductive
- **Try next time**: Experiments or changes to test in the next session

Be specific and actionable. "Communication was good" is not useful. "Sending spike reports via SendMessage with structured format made orchestrator processing 3x faster" is useful.

---

## Phase 4: Update MEMORY.md

### 4a. Read Current State

Read the current MEMORY.md:

```
~/.claude/projects/<project-path>/memory/MEMORY.md
```

### 4b. Identify Updates

Determine which learnings are **durable** (useful across sessions) vs. **ephemeral** (only relevant to this session). Only persist durable learnings.

Durable examples:
- "Agent teams work well for parallel audits but add overhead for < 5 tasks"
- "Always read existing skill files before writing new ones to match format"
- "bd dep add direction: epic depends on child, never reverse"

Ephemeral examples (do NOT persist):
- "Completed 8 tasks today"
- "Session started at 2pm"
- "User seemed happy with the output"

### 4c. Apply Updates

Using Write or Edit, update MEMORY.md:
- Add new durable learnings under appropriate sections
- Update existing entries if new evidence confirms, refutes, or refines them
- Remove entries that are no longer accurate
- Keep the file under 200 lines (it is loaded into the system prompt)

If detailed notes are needed for a topic, create a separate file in the memory directory (e.g., `team-patterns.md`, `skill-authoring.md`) and link to it from MEMORY.md.

### 4d. Verify

Read MEMORY.md after edits to confirm it is well-structured and under the line limit.

---

## Phase 5: Report

Present a structured retrospective report:

```markdown
## Session Retrospective

### Summary
[1-2 sentence summary of what this session accomplished and its overall character]

### What Went Well
- [specific item with evidence]
- [specific item with evidence]

### What Could Improve
- [specific item with evidence and suggested change]
- [specific item with evidence and suggested change]

### Action Items
- [ ] [specific, actionable item for the next session]
- [ ] [specific, actionable item for the next session]

### Memory Updates
- [list each change made to MEMORY.md: added, updated, or removed]
```

---

## Guidelines

- **Be honest and specific, not generic.** "Quality was good" tells you nothing. "All 12 findings were CONFIRMED with file:line evidence" tells you something.
- **Focus on actionable learnings.** Every "what could improve" should have a concrete suggestion attached.
- **Update MEMORY.md with durable insights only.** Session-specific details belong in the report, not in persistent memory.
- **Keep it concise.** A retro should take less than 2 minutes. Do not over-analyze.
- **Distinguish root causes from symptoms.** "Agent output was low quality" is a symptom. "Agent was given a vague spike description without specific files to investigate" is a root cause.
- **Credit what worked.** Retros that only focus on problems train you to ignore successes. Capture positive patterns too.
