---
name: standup
description: "Run a team standup to sync status, surface blockers, and plan next actions. Reads the team manifest and each member's learnings for a file-based status report. Works with or without beads backlog tracking. Use at the start of a session, after a break, or when you need to reorient. Keywords: status, sync, blockers, progress, team, check-in."
argument-hint: "[focus area]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Task
---

# Standup: Team Status Sync

You are running a **Standup** -- a quick status sync with a persistent learning team. Each member's status is derived from their learnings file, git activity on owned files, and the beads backlog.

**Focus area (optional):** $ARGUMENTS

## When to Use

- At the start of a work session to get oriented on team status
- After a break or context switch to reorient on what's in progress
- When you need a quick view of blockers, learning health, and available work
- Before planning a sprint to understand current team state
- When you want a read-only status check without dispatching agents

## How It Works

```
Read team.yaml → Check backlog + git → Per-member activity report
  → Learning health check → Synthesize board → Suggest actions
```

---

## Phase 1: Load Context

### 1a. Read Team Manifest

Read `.claude/team.yaml` to discover members, their roles, and ownership patterns.

If no `.claude/team.yaml` exists, tell the user to run `/assemble` first and stop.

### 1b. Check Backlog (conditional)

**If `.beads/` exists in the project root**, check the backlog:

```bash
bd stats
bd ready
bd list --status=in_progress
bd blocked
bd epic status
```

**If `.beads/` does not exist**, skip this step entirely. The standup will focus on git activity and learning health only.

### 1c. Check Git

```bash
git log --oneline -10
git status
```

---

## Phase 2: Per-Member Activity

For each team member, gather their status **without dispatching agents** (standup should be fast and read-only):

### 2a. Git Activity by Ownership

For each member, check recent commits touching their owned files:

```bash
git log --oneline -5 -- <owns patterns>
```

For example, if backend owns `src/domain/**` and `src/infra/**`:
```bash
git log --oneline -5 -- "src/domain/" "src/infra/"
```

### 2b. Learning Health

Read each member's `memory/agents/<name>/learnings.md` and note:
- **Total entries**: Number of non-empty bullet points
- **Recent additions**: Entries with dates in the last 7 days
- **Staleness**: Time since last entry was added
- **Cross-agent notes**: Pending notes from other members
- **Size**: Whether approaching the 150-line cap

### 2c. Relevant Beads (conditional)

**If beads are available**, check if any in-progress or ready beads match this member's ownership patterns (by scanning bead titles/descriptions against the member's role keywords).

**If beads are not available**, skip this step.

---

## Phase 3: Synthesize

### 3a. Present the Board

```markdown
## Standup: [team name]

### Team Activity
| Member | Recent Commits | Learnings | Health |
|--------|---------------|-----------|--------|
| [name] | [count] in owned files | [total] entries ([recent] new) | [status emoji] |
| ... | ... | ... | ... |

Health: healthy (active commits + growing learnings), stale (no recent activity), bloated (learnings >120 lines), cold (no learnings yet)

### Backlog Snapshot
[Only include this section if beads are available]
- **Ready**: [count] tasks available
- **In Progress**: [count] tasks active
- **Blocked**: [count] tasks blocked

### Epic Progress
[Only include if beads are available and epics exist]
| Epic | Children | Complete | Progress |
|------|----------|----------|----------|
| [title] | [total] | [done] | [%] |

### Blockers
[Only include if beads are available. List any blocked beads or issues. "No blockers" if clear.]

### Learning Highlights
[Notable recent learnings across the team. 2-3 most relevant items.]

### Cross-Agent Notes Pending
[Any cross-agent notes that haven't been acknowledged. "None pending" if clear.]

### Suggested Actions
1. [Highest-priority action based on standup]
2. [Second priority]
3. [Third priority]
```

### 3b. Ask User for Direction

Present the board and offer options:
- Dispatch work on the highest-priority ready bead (`/sprint`)
- Resolve a blocker
- Run `/meeting` to discuss a tension
- Run `/retro` if learnings are stale or bloated

---

## Guidelines

1. **Fast.** Standup should complete in under 30 seconds. No agent dispatch -- everything is read-only file operations.
2. **Honest.** "No activity" and "No learnings yet" are valid. Don't fabricate progress.
3. **Actionable.** The output should make the next action obvious.
4. **Learning-aware.** Surface learning health alongside work status -- a member with stale learnings may need a retro.
5. **Lightweight.** No beads creation during standup. No file writes. Pure read-only status check.

See also: /status (solo equivalent — use when no team is configured or you want a faster individual view).
