---
name: standup
description: "Run a team standup to sync status, surface blockers, and check learning health. Use when a team manifest exists but current member status is unknown, the beads backlog has changed since the last standup, or you've re-entered a session after a context switch. Keywords: status, sync, blockers, progress, team, check-in, reorient."
argument-hint: "[focus area]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(tk:*), Bash(git:*)
---

# Standup: Team Status Sync

You are running a **Standup** -- a quick, read-only status check with a persistent learning team. Each member's status is derived from their learnings file, git activity on owned files, and the beads backlog.

**Focus area (optional):** $ARGUMENTS

## When to Use

- Team manifest exists but you don't know who is active, blocked, or ready
- Beads backlog has changed since the last standup (new tasks, closed tasks, status shifts)
- Re-entering a session after a context switch and need to reorient before deciding what to dispatch
- Before planning a sprint, to confirm team state rather than assume it

## Don't Use When

- No `.claude/team.yaml` exists — run `/assemble` first
- You just ran standup in this session and nothing has changed (git log and beads unchanged)
- You need to assign or dispatch work — standup is read-only; use `/sprint` to dispatch
- You need a solo status check with no team — use `/status` instead

## Overview

```
Validate preconditions
  -> Load team manifest + backlog + git log
    -> Gather per-member activity (git + learnings)
      -> Verify data completeness
        -> Synthesize board
```

---

## Phase 1: Load Context

### 1a. Read Team Manifest

Read `.claude/team.yaml` to discover members, their roles, and ownership patterns.

If no `.claude/team.yaml` exists, stop here: "No team manifest found. Run `/assemble` to create one."

Note the member count. You must produce a status entry for every member — missing members make the standup incomplete.

### 1b. Check Backlog (conditional)

**If `.tacks/` or `.beads/` exists in the project root**, run:

```bash
tk stats
tk ready
tk list --status=in_progress
tk blocked
tk epic
```

**If neither `.beads/` nor `.tacks/` exists**, skip this step. The standup will focus on git activity and learning health only.

### 1c. Check Git

```bash
git log --oneline -10
git status
```

Note the most recent commit SHA and date. This anchors the freshness of activity signals in Phase 2.

---

## Phase 2: Per-Member Activity

For each team member, gather their status **without dispatching agents** — standup must be fast and read-only.

### 2a. Git Activity by Ownership

For each member, check recent commits touching their owned files:

```bash
git log --oneline -5 -- <owns patterns>
```

For example, if backend owns `src/domain/**` and `src/infra/**`:
```bash
git log --oneline -5 -- "src/domain/" "src/infra/"
```

Record: number of commits, date of most recent commit, and whether any commits appear in the last 7 days.

### 2b. Learning Health

Read each member's `memory/agents/<name>/learnings.md` and note:
- **Total entries**: Number of non-empty bullet points
- **Recent additions**: Entries with dates in the last 7 days
- **Staleness**: Time since last entry was added
- **Cross-agent notes**: Pending notes from other members
- **Size**: Whether approaching the 60-line cap (30 Core + 30 Task-Relevant)

If the file does not exist, record status as **cold** (role not yet dispatched).

### 2c. Relevant Beads (conditional)

**If beads are available**, for each member check whether any in-progress or ready beads match their ownership patterns by scanning bead titles against the member's role keywords.

**If beads are not available**, skip this step.

---

## Phase 3: Verification Gate

Before synthesizing, verify the data you gathered is complete and consistent.

**Coverage check:**
- [ ] Every member from team.yaml has a row of data (git + learnings). If any member is missing, go back to Phase 2 and complete their data.
- [ ] Git activity was checked against each member's actual `owns` patterns, not just the overall log. A member with recent commits to unowned files should still show "no owned-file activity."

**Consistency check:**
- [ ] If a member has recent commits but stale learnings (no entries in the last 14 days), flag this as an inconsistency: active work without captured learnings suggests a retro is overdue.
- [ ] If a member has zero owned-file commits AND zero learnings entries, mark them **cold** rather than **stale** — cold means the role hasn't been exercised yet, not that it has gone quiet.

**Backlog completeness check (if backlog tool available):**
- [ ] Confirm `tk ready`, `tk list --status=in_progress`, and `tk blocked` all returned results (even if the result is zero items). If a command failed or was skipped, note "backlog data incomplete" in the board output.

If any check fails, complete the missing data before proceeding. Do not synthesize on partial information.

---

## Phase 4: Synthesize

### 4a. Present the Board

```markdown
## Standup: [team name]

### Team Activity
| Member | Recent Commits | Learnings | Health |
|--------|---------------|-----------|--------|
| [name] | [count] in owned files ([date of most recent]) | [total] entries ([recent] new in 7 days) | [status] |
| ... | ... | ... | ... |

Health labels:
- healthy: recent owned-file commits AND learnings active in last 7 days
- active: recent owned-file commits but learnings not updated recently (flag for retro)
- learning: new learnings entries but no recent owned-file commits
- stale: no owned-file commits AND no new learnings in last 7 days
- bloated: learnings approaching or over 60-line cap (run /curate or /retro)
- cold: no learnings file and no owned-file commits (role not yet dispatched)

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
[Only if beads are available. List any blocked beads or issues. "No blockers" if clear.]

### Learning Highlights
[Notable recent learnings across the team. 2-3 most relevant items. "No recent learnings" if none in last 7 days.]

### Cross-Agent Notes Pending
[Any cross-agent notes that haven't been acknowledged. "None pending" if clear.]

### Flags
[Inconsistencies from Phase 3 verification: members with active commits but stale learnings, beads with no assigned member, etc. Omit section if no flags.]

### Suggested Actions
1. [Highest-priority action based on standup data]
2. [Second priority, if any]
3. [Third priority, if any]
```

### 4b. Standup Is Complete When

The standup is complete when all of the following are true:

- Every member in team.yaml has a row in the Team Activity table
- Every health label is supported by at least one data point (commit count or learnings entry count)
- Backlog snapshot is present if beads are available, or explicitly noted as unavailable
- Flags section is present if any inconsistency was detected in Phase 3, or omitted if none

If any condition is not met, complete the missing work before presenting the board.

### 4c. Offer Next Steps

After presenting the board, offer:
- Dispatch work on the highest-priority ready bead → `/sprint`
- Resolve a blocker → describe the blocked bead and options
- Discuss a tension or decision → `/meeting`
- Prune stale or bloated learnings → `/retro` or `/curate <member>`

---

## Guidelines

1. **Fast and read-only.** Standup completes in under 30 seconds. No agent dispatch, no file writes, no bead creation.
2. **Honest over polished.** "No activity" and "No learnings yet" are valid states. Do not interpret absence of data as presence of progress.
3. **Verify before synthesizing.** Phase 3 exists to catch incomplete or contradictory data before it becomes a misleading board.
4. **Learning health alongside work status.** A member with stale learnings and active commits is working without capturing context — that is a signal, not just a gap.
5. **60-line cap for learnings.** The cap is 60 lines (30 Core + 30 Task-Relevant). Flag members at or above 50 lines as approaching the cap. Do not use 150 lines as the threshold — that is not the convention for this project.

See also: /status (solo equivalent — use when no team is configured or you want a faster individual view), /sprint (dispatch work after standup reveals what's ready), /retro (address stale learnings flagged during standup), /curate (optimize a member's learnings before the next sprint).
