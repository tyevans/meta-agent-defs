---
name: standup
description: "Run a team standup to sync status, surface blockers, and plan next actions. Each team member reports from their memory and assigned beads. Use at the start of a session, after a break, or when you need to reorient. Keywords: status, sync, blockers, progress, team, check-in."
argument-hint: "[team-config-path]"
disable-model-invocation: true
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Task
---

# Standup: Team Status Sync

You are running a **Standup** -- a quick status sync with a persistent agent team. Each member checks their memory, reviews their assigned beads, and reports progress, blockers, and planned next actions.

**Team config:** $ARGUMENTS (path to team config, e.g., `.claude/teams/myproject.md`). If not provided, look for the first `.claude/teams/*.md` file.

## Phase 1: Load Context

### 1a. Read Team Config

Read the team config file to discover members, their roles, and memory nodes.

### 1b. Check Backlog

```bash
bd stats
bd ready
bd list --status=in_progress
bd blocked
```

### 1c. Check Git

```bash
git log --oneline -5
git status
```

---

## Phase 2: Collect Reports

For each team member, dispatch a lightweight agent to generate their standup report:

```
Task({
  subagent_type: "general-purpose",
  run_in_background: true,
  model: "haiku",
  prompt: "<standup agent prompt -- see below>"
})
```

Launch all standup agents concurrently (they're fast and independent).

**Standup agent prompt:**

> You are **[Role]** for [project name], generating your standup report.
>
> **Step 1**: Read your memory node:
> ```
> mcp__memory__open_nodes(names: ["agent:[memory-node-name]"])
> ```
>
> **Step 2**: Check beads assigned to you (if any):
> ```bash
> bd list --status=in_progress
> bd list --status=open
> ```
>
> **Step 3**: Report in this format:
>
> ## [Role] Standup
> **Done**: [What you completed or know from memory. "Nothing yet" if first standup.]
> **Doing**: [Current in-progress beads or active focus area]
> **Blocked**: [Anything preventing progress. "None" if clear.]
> **Next**: [What you'd work on next given the backlog]
> **Learning**: [One thing from your memory that's relevant right now. "None" if nothing applies.]
>
> Keep it to 5-10 lines total. This is a standup, not a report.

---

## Phase 3: Synthesize

After all standup agents complete:

### 3a. Present the Board

```markdown
## Standup: [project name]

| Role | Done | Doing | Blocked | Next |
|------|------|-------|---------|------|
| [role] | [summary] | [summary] | [summary] | [summary] |
| ... | ... | ... | ... | ... |

### Blockers
[List any blockers that need resolution. If none, say "No blockers."]

### Learnings
[Notable learnings from team memory that are relevant to current work]

### Suggested Actions
1. [Highest-priority action based on standup]
2. [Second priority]
3. [Third priority]
```

### 3b. Ask User for Direction

Present the standup board and ask what to focus on next. Offer options:
- Resolve a blocker
- Dispatch work on the highest-priority ready bead
- Run `/meeting` to discuss a tension
- Run `/fractal` to investigate an unknown

---

## Guidelines

1. **Fast.** Standup should complete in under a minute. Use haiku model for standup agents.
2. **Honest.** "Nothing yet" and "None" are valid answers. Don't fabricate progress.
3. **Actionable.** The output should make the next action obvious.
4. **Memory-aware.** Each member's report draws from their accumulated knowledge, not just beads.
5. **Lightweight.** No beads creation during standup. It's a read-only status check.
