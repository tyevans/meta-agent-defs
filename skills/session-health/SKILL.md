---
name: session-health
description: "Run a session health diagnostic to assess context load, scope drift, and quality degradation. Use when responses feel degraded, before starting major new work in a long session, or when the user asks if you're still tracking well."
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*)
---

# Session Health Check

You are running a **session health check** — a quick diagnostic of the current conversation to determine if you should continue, compact, or start fresh. Context: **$ARGUMENTS**

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
| Light | <10 | 0-1 | 1-2 areas | Healthy, continue |
| Moderate | 10-25 | 2-4 | 3-4 areas | Watch quality |
| Heavy | 25-50 | 5+ | 5+ areas | Consider compacting |
| Overloaded | 50+ | 8+ | Many areas | Compact or clear |

### 2. Check Scope Coherence

Ask yourself:
- **Original goal**: What did the user first ask for this session?
- **Current work**: What are you doing right now?
- **Drift**: Has scope crept from the original goal? How far?

Scope drift isn't always bad — discovered work is natural. But if you've drifted far from the original intent, flag it.

### 3. Check Quality Indicators

Honest self-assessment:
- Are you repeating information you've already shared?
- Are you making assumptions instead of reading code?
- Are you hedging more than earlier in the session?
- Do your recent tool calls feel less targeted (broader searches, reading larger files)?
- Have you lost track of the beads backlog state?

### 4. Check Session State

```bash
bd list --status=in_progress   # What's claimed but not done?
bd ready                       # What's available?
git status                     # Any uncommitted work?
```

### 5. Recommendation

Based on the above, recommend ONE of:

| Action | When | How |
|--------|------|-----|
| **Continue** | Light-moderate load, on-topic, quality fine | Keep working |
| **Compact** | Heavy load but good progress, want to keep context | `/compact` — preserves key context, frees space |
| **Checkpoint & Continue** | Moderate load, want a save point | Commit current work, `bd sync`, then continue |
| **Fresh session** | Overloaded, scope has drifted, or quality degrading | Commit all work, `bd sync`, push, then `/clear` or new session |
| **Break into subagents** | Remaining work is parallelizable | Dispatch independent tasks to subagents to avoid further context fill |

## Output Format

```markdown
## Session Health: [Healthy / Watch / Compact / Fresh]

**Load**: [Light/Moderate/Heavy/Overloaded] — ~X files read, ~Y agents, Z areas
**Scope**: [On track / Minor drift / Significant drift] — started with [X], now doing [Y]
**Quality**: [Good / Showing wear / Degrading]
**Uncommitted work**: [Yes/No] — [details if yes]

### Recommendation
[Action] — [1-2 sentence rationale]

### If continuing, suggested next steps:
1. [next thing to do]
2. [thing after that]
```

## Important Notes

- This is a **self-diagnostic** — be honest, not optimistic
- Context compaction via `/compact` is lightweight and preserves important state
- Committing and syncing before any session change is mandatory
- When in doubt, checkpoint (commit + sync) before deciding — it's cheap insurance
- A fresh session with good beads context (via `bd prime`) is often more productive than a degraded long session
