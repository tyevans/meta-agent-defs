---
name: advise
description: "Proactive session recommendations by composing git state, session history, backlog, and signals. Degrades gracefully — works with just git, richer with each layer present. Keywords: advise, suggest, next, what now, recommend, orient."
argument-hint: "[focus area]"
disable-model-invocation: false
user-invocable: true
allowed-tools: [Read, Glob, Grep, "Bash(bd:*)", "Bash(git status:*)", "Bash(git log:*)", "Bash(git diff:*)", "Bash(bin/git-pulse.sh:*)", "Bash($HOME/.claude/bin/git-pulse.sh:*)"]
context: inline
---

# Advise: Proactive Session Recommendations

You are generating **actionable recommendations** for what to do next, based on whatever project state is available. This skill degrades gracefully across four layers — it always produces useful output, even in a bare git repo with no tooling installed.

**Focus area (optional):** $ARGUMENTS

## When to Use

- At the start of a session when you're not sure what to work on
- After completing a task and wondering what's next
- When onboarding to a new project ("what should I focus on?")
- When the user asks "what now?" or "what should I do next?"

## How It Works

```
Probe 4 layers (git → session → backlog → signals)
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

### Layer 2: Session Continuity (if `memory/sessions/last.md` exists)

Read `memory/sessions/last.md`.

Extract:
- **What was worked on last**: Commits, in-progress items
- **Unfinished work**: Items that were in-progress when the session ended
- **Working tree state at close**: Were there uncommitted changes left behind?

### Layer 3: Backlog (if `bd` is available)

```bash
bd ready 2>/dev/null
bd list --status=in_progress 2>/dev/null
bd blocked 2>/dev/null
```

Extract:
- **Ready work**: Tasks with no blockers, sorted by priority
- **In-progress items**: Work that was started but not finished
- **Blocked items**: What's stuck and why

### Layer 4: Signals (if `git-pulse.sh` is available)

```bash
"$HOME/.claude/bin/git-pulse.sh" 2>/dev/null || bin/git-pulse.sh 2>/dev/null
```

Extract:
- **Fix rate**: High fix rate suggests rushing or insufficient review
- **Churn concentration**: Scattered churn suggests exploratory phase; focused churn suggests targeted work
- **Signal count**: fix-after-feat patterns indicate incomplete rollouts

---

## Phase 2: Generate Recommendations

Based on available layers, generate 3-5 concrete recommendations. Each recommendation must be:
- **Actionable**: Starts with a verb ("Review...", "Complete...", "Fix...")
- **Grounded**: References specific data from Phase 1 (file names, bead IDs, commit hashes)
- **Prioritized**: Ordered by urgency (uncommitted work > in-progress > ready backlog > new work)

### Priority Rules

1. **Uncommitted changes from a previous session** → Always recommend reviewing/committing first
2. **In-progress beads** → Resume before starting new work
3. **High fix rate (>30%)** → Suggest slowing down, adding review steps
4. **Blocked items that could be unblocked** → Unblock before starting independent work
5. **Ready backlog by priority** → Work highest-priority items first
6. **No backlog** → Suggest /blossom or /meeting to discover work

### Focus Area Filter

If `$ARGUMENTS` specifies a focus area, filter recommendations to that area. Still mention urgent items outside the focus (uncommitted changes, high fix rate) but deprioritize unrelated backlog items.

---

## Phase 3: Output

### Format

```markdown
## Session Advice

**Layers**: [git] [session] [backlog] [signals]
(Filled layers shown as bold, missing layers shown as dim/struck)

### Recommendations

1. **[verb] [specific thing]** — [why, with evidence]
   _Source: [layer that informed this]_

2. **[verb] [specific thing]** — [why, with evidence]
   _Source: [layer that informed this]_

3. ...

### Missing Layers

[Only if layers 2-4 are missing. Frame as value proposition, not error.]

- **Session history**: Run a session with SessionEnd hook to get continuity advice next time
- **Backlog**: Install beads (`bd init`) to get priority-aware recommendations
- **Signals**: Install git-intel for pattern detection (fix-after-feat, churn hotspots)
```

### Layer Indicator

Show which layers were available using a compact indicator:
- All four: `**Layers**: **git** · **session** · **backlog** · **signals**`
- Partial: `**Layers**: **git** · ~~session~~ · **backlog** · ~~signals~~`

### Missing Layers Section

Only show this section if at least one layer is missing. Frame each missing layer as a value proposition ("what you'd get"), not as an error. This serves as progressive discovery — users learn about capabilities organically.

---

## Guidelines

- **Always produce output.** Even with only Layer 1 (git), there's useful advice to give based on uncommitted changes, recent commit patterns, and branch state.
- **Be specific, not generic.** "Review the 3 uncommitted files in src/auth/" is useful. "Check your uncommitted changes" is not.
- **Don't over-recommend.** 3-5 items maximum. If everything looks clean and the backlog is empty, say so — "No urgent work detected. Consider /blossom to explore new areas."
- **Respect the focus area.** If the user asked about "testing", don't lead with unrelated backlog items (but do mention urgent git state issues).
- **Missing layers are opportunities, not failures.** The nudge section should make users want to install tools, not feel bad about missing them.
