---
name: status
description: "Show unified system status: backlog, recent activity, team health, and last session summary. Use when you want a full picture of where things stand in one view. Keywords: status, overview, dashboard, state, orientation."
argument-hint: "[focus area]"
disable-model-invocation: false
user-invocable: true
allowed-tools: [Read, Glob, Grep, TaskList, "Bash(bd:*)", "Bash(git status:*)", "Bash(git log:*)", "Bash(git branch:*)"]
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

## Don't Use When

- You need deep diagnosis of why a session feels wrong — use /session-health instead
- You already know what to work on and just need to start — /status adds no value if the path is already clear
- You only need backlog info — run `bd ready` or `bd stats` directly rather than the full status view

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

**If `.beads/` or `.tacks/` exists**, run:

```bash
bd stats
bd ready
bd epic status
```

**If neither `.beads/` nor `.tacks/` exists**, note "Beads not configured."

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

### 1e. Recent Activity Summary

```bash
git log --oneline --since="7 days ago" | head -20
```

**If no recent activity**, skip this section.

### 1f. Live Agents (conditional)

Call TaskList to retrieve active task state. If TaskList returns results, collect:

- **Running tasks**: tasks with status `running` — capture agent name, task description
- **Completed tasks**: tasks with status `completed` that have not yet been reviewed (no corresponding commit or bead close since completion)
- **Idle agents**: agents listed in `.claude/team.yaml` (if present) that have no running or recently completed task

**If TaskList returns no results**, skip this section entirely.

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

### Epic Progress
[bd epic status output — show each epic with completion %]
[Omit section if no epics exist or beads not configured]

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

### Live Agents
[Only present when TaskList returned results.]

| Agent | Task | Status |
|-------|------|--------|
| [agent name] | [task description] | running / completed (awaiting review) |

Idle agents (in team.yaml but no active or recent task): [list or "none"]

[Omit this section entirely when TaskList returns no results.]

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
| Epics with all children complete | "Run `bd epic close-eligible` to auto-close completed epics" |
| Everything is clean and no ready tasks | "Backlog is clear — create new tasks or run `/blossom` to explore" |
| Completed tasks awaiting review (from TaskList) | "Review results from [agent name]: [task description]" |
| Idle agents with ready backlog tasks | "Dispatch [agent name] to: [top ready task title]" |

Only include suggestions that match the current state. Do not invent conditions.

---

After the Suggested Actions section, emit a pipe-format summary so /advise can consume status signals:

```
## System status signals

**Source**: /status
**Input**: [focus area from $ARGUMENTS, or "full system status"]
**Pipeline**: (none — working from direct input)

### Items (4)

1. **Backlog state** — [N open tasks, N ready, N blocked, N in-progress]
   - ready: [top 3 task titles, or "none"]
   - blocked: [N tasks blocked, or "none"]
   - epics: [N epics, M% complete on average, or "none"]

2. **Recent activity** — [active | quiet | stalled]
   - branch: [current branch]
   - commits-last-7d: [N]
   - working-tree: [clean | N files modified/untracked]

3. **Team health** — [healthy | cold members | bloated learnings | no team]
   - members: [N total, N active, N cold, N bloated]
   - action-needed: [member names needing attention, or "none"]

4. **Suggested actions** — [N actions derived from state]
   - actions: [top 3 suggestions as short phrases]
   - confidence: CONFIRMED

### Summary

[One paragraph: current backlog shape, recent activity level, team health, and the single most important action to take next.]
```

## Guidelines

1. **Fast.** This should complete in under 10 seconds. No agent dispatch, no synthesis essays.
2. **Honest.** Show actual data. "No team configured" and "Beads not configured" are valid outputs.
3. **Mechanical.** Suggested actions are derived from rules, not LLM creativity. If no rule matches, say "No immediate actions suggested."
4. **Read-only.** No file writes, no bead creation, no git commits. Pure observation.
5. **Graceful degradation.** Every section works independently. Missing data sources produce a one-line fallback, not an error.

See also: /evolution (definition change tracking). /standup (team-aware equivalent — use standup when a team is configured). /session-health (focused diagnostic when something feels off in the current session).
