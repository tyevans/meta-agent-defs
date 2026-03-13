---
name: advise
description: "Use when you're unsure what to work on next. Reads git state, session history, and task tracker to recommend next actions. Works with just git, richer with more context. Keywords: advise, suggest, next, what now, recommend, orient."
argument-hint: "[focus area]"
disable-model-invocation: false
user-invocable: true
allowed-tools: [Read, Glob, Grep, TaskList, "Bash(git status:*)", "Bash(git log:*)", "Bash(git diff:*)"]
context: inline
---

# Advise: Proactive Session Recommendations

You are generating **actionable recommendations** for what to do next, based on whatever project state is available. This skill degrades gracefully across five layers — it always produces useful output, even in a bare git repo with no tooling installed.

**Focus area (optional):** $ARGUMENTS

## When to Use

- At the start of a session when you're not sure what to work on
- After completing a task and wondering what's next
- When onboarding to a new project ("what should I focus on?")
- When the user asks "what now?" or "what should I do next?"

## Don't Use When

- You already know what to work on — go work on it; /advise adds no value when the path is clear
- Mid-task, not between tasks — this is an orientation tool, not a mid-flow interrupt
- No git history and no task tracker exist — Layer 1 requires at least one commit for meaningful advice (a brand-new empty repo produces noise, not signal)

## How It Works

```
Probe 5 layers (git → session → task tracker → team → live agents)
  → Score what's available
    → Generate recommendations ranked by urgency
      → Nudge about missing layers that would improve advice
```

No agents dispatched. No files written. Pure read + recommend.

---

## Phase 1: Probe Layers

Probe each layer in order. Each layer is independent — a missing layer does not block subsequent layers.

### Layer 1: Git State (always available)

```bash
git status --short
git log --oneline -5
git branch --show-current
```

Extract:
- **Uncommitted changes**: Count and categorize (new files, modifications, deletions)
- **Recent commit pattern**: What types of work happened recently (feat/fix/chore)
- **Branch context**: Are we on a feature branch? How far from main?

### Layer 2: Session Continuity (if `.claude/tackline/memory/sessions/last.md` exists)

Read `.claude/tackline/memory/sessions/last.md`.

Extract:
- **What was worked on last**: Commits, in-progress items
- **Unfinished work**: Items that were in-progress when the session ended
- **Working tree state at close**: Were there uncommitted changes left behind?

### Layer 3: Task Tracker (if available)

Check your task tracker for current state.

Extract:
- **Ready work**: Tasks with no blockers, sorted by priority
- **In-progress items**: Work that was started but not finished
- **Blocked items**: What's stuck and why

### Layer 4: Team Health (if `.claude/team.yaml` exists)

Read `.claude/team.yaml` and check each member's learnings:

```bash
ls .claude/tackline/memory/agents/*/learnings.md 2>/dev/null
```

For each member, check:
- **Line count**: >120 lines = bloated (recommend `/curate`)
- **Staleness**: >14 days since last learnings update = cold agent
- **Missing learnings**: Agent in team.yaml but no learnings file = unconfigured

Extract:
- **Stale agents**: Members whose learnings haven't been updated recently
- **Bloated agents**: Members whose learnings exceed 120 lines
- **Missing agents**: Members in manifest but without learnings files

### Layer 5: Active Agents (if TaskList returns results)

Call TaskList to retrieve live task state.

In active-agents mode: extract:
- **Running tasks**: tasks with status `running` — capture agent name, task description, and (if available) how long they have been running
- **Completed tasks awaiting review**: tasks with status `completed` that have no corresponding task close or commit since completion
- **Idle agents**: agents in `.claude/team.yaml` (if present) that have no running or recently completed task

**If TaskList returns no results**, skip this layer and do not include it in the layers indicator.

**Gate**: At least 2 layers contributed usable signals (non-empty, non-error results). If only Layer 1 (git) returned data, note this in the Missing Layers section — git-only advice is minimal and will be shallow without additional layers. Do not pad recommendations to appear more comprehensive than the data supports.

---

## Phase 2: Generate Recommendations

Based on available layers, generate 3-5 concrete recommendations. Each recommendation must be:
- **Actionable**: Starts with a verb ("Review...", "Complete...", "Fix...")
- **Grounded**: References specific data from Phase 1 (file names, task IDs, commit hashes)
- **Prioritized**: Ordered by urgency (uncommitted work > in-progress > ready tasks > new work)

### Priority Rules

1. **Uncommitted changes from a previous session** → Always recommend reviewing/committing first
2. **In-progress tasks** → Resume before starting new work
3. **Completed agent tasks awaiting review** → Review before dispatching new work ("Agent X completed — review results before dispatching next task")
4. **Blocked items that could be unblocked** → Unblock before starting independent work
5. **Ready tasks by priority** → Work highest-priority items first
6. **Idle agents with ready tasks** → "Agent Y is idle — dispatch to: [top ready task title]"
7. **Stale team agents** → Recommend `/curate` or `/tend` for agents with cold learnings
8. **No tracked tasks** → Suggest /blossom or /meeting to discover work

### Focus Area Filter

If `$ARGUMENTS` specifies a focus area, filter recommendations to that area. Still mention urgent items outside the focus (uncommitted changes) but deprioritize unrelated task items.

### Sharpening Gate

Before finalizing recommendations, apply these tests to each one:

1. **Name the specific code/file/workflow** — Does it reference a commit hash, task ID, file path, or metric value?
2. **State what concretely should change** — Is there a command to run, a file to review, or a task to close?
3. **Make it actionable** — Can this be executed as a single command, task ID, or specific next step?

Reject recommendations starting with "Consider", "You might want to", or "Think about". Every recommendation must reference specific data from Phase 1.

**Example:**
- ❌ Before: "Consider addressing the high fix rate"
- ✅ After: "Run /review on commits c752c8d..888d9f2 — fix rate is 45% (9/20 commits in last 30d), concentrated in src/integration.rs"

**Gate**: Every recommendation passes the sharpening tests above — it names a specific artifact (commit hash, task ID, file path, or agent name) and states a concrete next action. Drop any recommendation that fails both tests rather than including a vague item.

---

## Phase 3: Output

### Format

```markdown
## Session Advice

**Layers**: [git] [session] [task-tracker] [team] [live-agents]
(Filled layers shown as bold, missing layers shown as dim/struck)

### Recommendations

1. **[verb] [specific thing]** — [why, with evidence]
   _Source: [layer that informed this]_

2. **[verb] [specific thing]** — [why, with evidence]
   _Source: [layer that informed this]_

3. ...

### Missing Layers

[Only if layers 2-5 are missing. Frame as value proposition, not error.]

- **Session history**: Run a session with SessionEnd hook to get continuity advice next time
- **Task tracker**: Set up a task tracker to get priority-aware recommendations
- **Team**: Run `/assemble` to get team-aware recommendations (stale learnings, agent health, team-specific actions)
- **Live agents**: No active agent tasks detected — live-agents layer enriches advice when a sprint or parallel dispatch is running
```

### Layer Indicator

Show which layers were available using a compact indicator:
- All five: `**Layers**: **git** · **session** · **task-tracker** · **team** · **live-agents**`
- Partial: `**Layers**: **git** · ~~session~~ · **task-tracker** · ~~team~~ · ~~live-agents~~`

### Missing Layers Section

Only show this section if at least one layer is missing. Frame each missing layer as a value proposition ("what you'd get"), not as an error. This serves as progressive discovery — users learn about capabilities organically.

---

## Guidelines

- **Always produce output.** Even with only Layer 1 (git), there's useful advice to give based on uncommitted changes, recent commit patterns, and branch state.
- **Be specific, not generic.** "Review the 3 uncommitted files in src/auth/" is useful. "Check your uncommitted changes" is not.
- **Don't over-recommend.** 3-5 items maximum. If everything looks clean and no tracked tasks exist, say so — "No urgent work detected. Consider /blossom to explore new areas."
- **Respect the focus area.** If the user asked about "testing", don't lead with unrelated task items (but do mention urgent git state issues).
- **Missing layers are opportunities, not failures.** The nudge section should make users want to install tools, not feel bad about missing them.
