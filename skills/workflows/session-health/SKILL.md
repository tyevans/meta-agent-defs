---
name: session-health
description: "Run a session health diagnostic to assess context load, scope drift, and quality degradation. Use when responses feel degraded, before starting major new work in a long session, or when the user asks if you're still tracking well."
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git:*)
---

!`git log --oneline -5`

# Session Health Check

You are running a **session health check** — a quick diagnostic of the current conversation to determine if you should continue, summarize, checkpoint, or start fresh. Context: **$ARGUMENTS**

## When to Use

- When responses feel shorter, vaguer, or less accurate than earlier
- When you've been working for a long time and want to gut-check quality
- Before starting a major new piece of work in an existing session
- When the user asks "are you still tracking?" or similar

## Diagnostic Process

### 1. Assess Context Load

Review what you've done this session:

- **Files read**: Roughly how many distinct files have you read?
- **Files modified**: How many files have you written or edited?
- **Agents dispatched**: How many subagent tasks have you launched?
- **Topic breadth**: How many distinct areas of the codebase have you touched?

| Load Level | Files Read | Agents | Breadth | Signal |
|------------|-----------|--------|---------|--------|
| Light | <20 | 0-2 | 1-2 areas | Healthy, continue |
| Moderate | 20-60 | 3-5 | 3-4 areas | Watch quality |
| Heavy | 60-120 | 6-9 | 5+ areas | Check quality signals; consider fresh session |
| Overloaded | 120+ | 10+ | Many areas | Quality degraded — fresh session recommended |

> **Note**: Raw file counts are a lagging indicator. Quality signals (step 3) are the primary degradation signal — a session with 30 files but heavy hedging is more degraded than one with 80 files and sharp answers.

### 2. Check Scope Coherence

Ask yourself:
- **Original goal**: What did the user first ask for this session?
- **Current work**: What are you doing right now?
- **Drift**: Has scope crept from the original goal? How far?
- **Batch progress**: If processing a numbered list or collection, how many items are done vs remaining? If >50% done, recommend checkpointing results to a file before continuing.

Scope drift isn't always bad — discovered work is natural. But if you've drifted far from the original intent, flag it.

### 3. Check Quality Indicators

**Quality signals are the primary degradation signal.** A session can be Heavy by file count but still sharp; a session can be Moderate by file count but already degrading. Check these first and let them override the load level assessment:

| Signal | Healthy | Degrading |
|--------|---------|-----------|
| **Hedging language** | Direct, confident answers | "I think", "it might", "probably", "as I recall" — increasing over time |
| **Repetition** | Novel, additive responses | Re-explaining things already stated earlier in the session |
| **Tool call quality** | Targeted searches, specific file reads | Broad grep patterns, re-reading files already read, "let me check again" |
| **Tool call failure rate** | Rare failures or none | Repeated tool failures, fallbacks to guessing |
| **Answer specificity** | Cites file paths, line numbers, exact values | Vague references, omitting specifics that were available |

Rate each signal: Good / Watch / Degrading. If two or more signals are Degrading, elevate the overall session health to **Fresh** regardless of load level.

### 4. Check Session State

```bash
git status                     # Any uncommitted work?
```

Check your task tracker for in-progress and available work.

### 5. Check Team Health (conditional)

If `.claude/team.yaml` exists, read it and check each member's learnings:

```bash
ls .claude/tackline/memory/agents/*/learnings.md 2>/dev/null
```

For each member in the team manifest, check:
- **Learnings file exists?** If missing, flag as "unconfigured"
- **Line count**: >120 lines = bloated (flag for `/curate`)
- **Last modified date**: >14 days since last update = stale

Report team state in the output:

```markdown
**Team**: [team name] — [N] members
- [member]: [line count] learnings, last updated [date] — [healthy / stale / bloated]
```

If no team is configured, skip this section silently (not an error — many projects don't use teams).

### 6. Recommendation

Based on the above, recommend ONE of:

| Action | When | How |
|--------|------|-----|
| **Continue** | Light-moderate load, on-topic, quality fine | Keep working |
| **Partial summarize** | Heavy load but good progress, quality still sharp | Selectively compress completed work threads while preserving in-progress work at full context. For each completed thread (a task or subtask where the output is final): write a 3-5 bullet summary capturing goal, approach, outcome, and key decisions, then release the detailed exchange from working context. Leave all in-progress threads uncompressed. This is a model-side operation — do not ask the user to summarize. |
| **Checkpoint & Continue** | Moderate load, mid-batch work, or want a save point | Commit current work, write intermediate results to `.claude/tackline/memory/scratch/<slug>.md` if mid-batch, sync task state, then continue |
| **Fresh session** | Overloaded, scope has drifted, or 2+ quality signals degrading | Run /handoff to capture context, commit all work, sync task state, push, then start a new session |
| **Break into subagents** | Remaining work is parallelizable | Dispatch independent tasks to subagents to avoid further context fill |

## Output Format

```markdown
## Session Health: [Healthy / Watch / Summarize / Fresh]

**Load**: [Light/Moderate/Heavy/Overloaded] — ~X files read, ~Y agents, Z areas
**Scope**: [On track / Minor drift / Significant drift] — started with [X], now doing [Y]
**Quality**: [Good / Showing wear / Degrading]
**Uncommitted work**: [Yes/No] — [details if yes]
**Team**: [team name — N members, M stale | No team configured]

### Recommendation
[Action] — [1-2 sentence rationale]

### If continuing, suggested next steps:

**Sharpening requirement**: Each step must be a concrete command or specific file reference, not a suggestion.

- **Before**: "Consider refactoring the authentication module"
- **After**: "Run /review on src/auth/ — 3 files changed in last 5 commits, fix rate 40%"

1. [runnable command or specific file path]
2. [runnable command or specific file path]
```

After producing the health report, emit a pipe-format summary so /advise or /retro can consume health signals:

```
## Session health signals

**Source**: /session-health
**Input**: [current session, $ARGUMENTS if provided]
**Pipeline**: (none — working from direct input)

### Items (4)

1. **Context load** — [Light | Moderate | Heavy | Overloaded]
   - files-read: [~N]
   - agents-dispatched: [~N]
   - topic-breadth: [N areas]
   - confidence: CONFIRMED

2. **Quality signals** — [Good | Showing wear | Degrading]
   - hedging: [Good | Watch | Degrading]
   - repetition: [Good | Watch | Degrading]
   - tool-call-quality: [Good | Watch | Degrading]
   - answer-specificity: [Good | Watch | Degrading]
   - confidence: CONFIRMED

3. **Scope coherence** — [On track | Minor drift | Significant drift]
   - original-goal: [one-line summary]
   - current-work: [one-line summary]
   - batch-progress: [N/M items complete | not applicable]

4. **Recommendation** — [Continue | Partial summarize | Checkpoint & Continue | Fresh session | Break into subagents]
   - rationale: [one sentence]
   - next-action: [specific command or file reference]
   - confidence: CONFIRMED

### Summary

[One paragraph: load level, quality assessment, and recommended action with a concrete reason.]
```

## Important Notes

- This is a **self-diagnostic** — be honest, not optimistic
- **Quality signals override file counts.** Two or more degrading quality signals means Fresh, regardless of load level
- Committing and syncing before any session change is mandatory
- When in doubt, checkpoint (commit + sync) before deciding — it's cheap insurance
- A fresh session with good task context is often more productive than a degraded long session
- Before starting a fresh session, run /handoff to capture in-progress tasks, decisions, and open questions in a structured handoff document

## See Also

- **/handoff** — structured session transition; captures context before clearing or starting fresh
- **/status** — lightweight status card; use before /session-health for a quick pulse
- **/retro** — end-of-session retrospective; run after recommending "Fresh session" to log learnings before closing
