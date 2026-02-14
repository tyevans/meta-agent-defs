---
name: status
description: "Show unified system status: backlog, recent activity, team health, and last session summary. Use when you want a full picture of where things stand in one view. Keywords: status, overview, dashboard, state, orientation."
argument-hint: "[focus area]"
disable-model-invocation: false
user-invocable: true
allowed-tools: [Read, Glob, Grep, "Bash(bd:*)", "Bash(git status:*)", "Bash(git log:*)", "Bash(git branch:*)"]
context: inline
---

# Status: Unified System View

You are generating a **system status snapshot** — a single-command overview of backlog, recent activity, team health, and last session state. This is a read-only, low-cost diagnostic.

**Focus area (optional):** $ARGUMENTS

## When to Use

- At the start of a session to orient quickly
- After returning from a break to see what changed
- When you want a full picture without dispatching agents or running multiple commands
- Before deciding what to work on next

## How It Works

```
Read known paths → Run bd + git commands → Format sections → Suggest actions
```

No agents dispatched. No files written. Pure read + format.

---

## Phase 1: Collect Data

Gather all data before producing any output. Read from the known paths defined in `rules/memory-layout.md`.

### 1a. Last Session

Read `memory/sessions/last.md`. If it does not exist, note "No previous session data."

### 1b. Backlog

**If `.beads/` exists**, run:

```bash
bd stats
bd ready
```

**If `.beads/` does not exist**, note "Beads not configured."

### 1c. Recent Activity

Run:

```bash
git branch --show-current
git log --oneline -5
git status --short
```

### 1d. Team

**If `.claude/team.yaml` exists**, read it to get the team name and member list. Then for each member, check `memory/agents/<name>/learnings.md`:

- Line count (via Grep or Read)
- Whether the file exists at all

**If `.claude/team.yaml` does not exist**, note "No team configured."

---

## Phase 2: Format Output

Present all collected data in this structure. Do not add commentary or analysis beyond what is specified — keep it mechanical and cheap.

```markdown
## System Status

**Branch**: [current branch]
**Date**: [today]

### Last Session
[Contents of memory/sessions/last.md — include verbatim, trimmed to first 20 lines if longer]
[Or: "No previous session data."]

### Backlog
[bd stats output, formatted]
[bd ready output — show task titles, max 5]
[Or: "Beads not configured."]

### Recent Activity
[git log --oneline -5 output]

**Working tree**: [clean | N files modified/untracked]
[If dirty, show git status --short output]

### Team
[Team name and member table:]
| Member | Role | Learnings | Health |
|--------|------|-----------|--------|
| [name] | [role from team.yaml] | [N lines] | [see below] |

Health values (mechanical):
- **active**: learnings file exists, >0 lines
- **cold**: learnings file missing or empty
- **bloated**: learnings file >120 lines

[Or: "No team configured."]

### Suggested Actions
[Numbered list, derived mechanically from the data above]
```

---

## Phase 3: Derive Suggested Actions

Generate 1-5 suggestions based purely on observed state. Use these rules:

| Condition | Suggestion |
|-----------|------------|
| Working tree is dirty | "Commit or stash uncommitted changes" |
| Ready tasks exist (beads) | "Work on: [top 3 ready task titles]" |
| Blocked tasks exist | "Unblock: [blocked task titles]" |
| Any team member is bloated (>120 lines) | "Run `/retro` to consolidate [member] learnings" |
| Any team member is cold | "Dispatch work to [member] to build learnings" |
| No last session file | "First session — review backlog and pick a starting task" |
| In-progress tasks exist | "Resume in-progress work: [task titles]" |
| Everything is clean and no ready tasks | "Backlog is clear — create new tasks or run `/blossom` to explore" |

Only include suggestions that match the current state. Do not invent conditions.

---

## Guidelines

1. **Fast.** This should complete in under 10 seconds. No agent dispatch, no synthesis essays.
2. **Honest.** Show actual data. "No team configured" and "Beads not configured" are valid outputs.
3. **Mechanical.** Suggested actions are derived from rules, not LLM creativity. If no rule matches, say "No immediate actions suggested."
4. **Read-only.** No file writes, no bead creation, no git commits. Pure observation.
5. **Graceful degradation.** Every section works independently. Missing data sources produce a one-line fallback, not an error.
