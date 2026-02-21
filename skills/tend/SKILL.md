---
name: tend
description: "Run the full learning lifecycle: curate learnings for upcoming work, then promote durable patterns to rules. Orchestrates /curate and /promote in sequence. Use after sprints, before major work phases, or on a regular cadence to keep learnings sharp and rules growing. Keywords: maintain, groom, lifecycle, learnings, rules, optimize, promote."
argument-hint: "[agent-name or 'all']"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Bash(ls:*), Write, Edit, Skill
---

# Tend: Learning Lifecycle Orchestrator

You are running **tend** — the full learning maintenance lifecycle. This orchestrates `/curate` and `/promote` in sequence so that agent learnings stay relevant to upcoming work and durable patterns graduate to rules over time.

**Target (optional):** $ARGUMENTS — an agent name (runs for that agent only) or "all" (default: all agents with learnings files).

## When to Use

- After completing a sprint or epic — learnings accumulated, time to optimize
- Before starting a major work phase — ensure agents have the right context loaded
- On a regular cadence (weekly or bi-weekly) — keep learnings from going stale
- When learnings files feel bloated or rules feel incomplete
- When you notice agents re-learning things that should be rules by now

## Overview

```
Load context (team manifest, learnings, upcoming work, rules inventory)
  -> Phase 1: /curate agents (per agent — optimize learnings for upcoming work)
    -> Phase 2: /curate rules (audit project rules for health)
      -> Phase 3: /promote (cross-agent — graduate patterns to rules)
        -> Phase 4: Demotion pass (downgrade stale rules)
          -> Phase 5: Summary report
```

---

## Phase 0: Load Context

Before orchestrating, gather the inputs that both /curate and /promote need.

### 0a. Team Manifest

Read `.claude/team.yaml`. If it doesn't exist, check for learnings files directly:

```bash
ls memory/agents/*/learnings.md 2>/dev/null
```

If neither exists, stop: "No team manifest or agent learnings found. Nothing to tend."

### 0b. Determine Target Agents

- If `$ARGUMENTS` names a specific agent, validate that agent has a learnings file
- If `$ARGUMENTS` is "all" or empty, collect all agents with learnings files
- Report: "Tending learnings for: [agent1, agent2, ...]"

### 0c. Upcoming Work Snapshot

Capture what work is coming so /curate has the signal it needs:

```bash
bd ready 2>/dev/null
bd list --status=in_progress 2>/dev/null
ls memory/epics/*/epic.md 2>/dev/null
```

If beads aren't available, note that curate will rely on git signals and conversation context only.

### 0d. Rules Inventory

List existing rules so /promote can detect overlap:

```bash
ls rules/*.md .claude/rules/*.md 2>/dev/null
```

Read CLAUDE.md to understand what's already in passive context.

---

## Phase 1: Curate Agent Learnings

For each target agent, invoke `/curate` to optimize their learnings for upcoming work.

### 1a. Invoke /curate

Use the Skill tool to invoke `/curate` for each agent:

- If single agent: `/curate <agent-name>`
- If multiple agents: invoke `/curate` once per agent, sequentially

### 1b. Collect Results

After each /curate invocation, note:
- How many learnings were kept, archived, or added
- Any gaps identified (upcoming work needs knowledge not in learnings)
- Any items flagged for cross-agent relevance

### 1c. Progress Check

After curating all agents, summarize before moving to rules:

```markdown
### Agent Curate Phase Complete

| Agent | Kept | Archived | Added | Gaps Found |
|-------|------|----------|-------|------------|
| [name] | [n] | [n] | [n] | [n] |
| ... | ... | ... | ... | ... |
```

---

## Phase 2: Curate Rules

After agent learnings are curated, audit the project rules for health and relevance.

### 2a. Invoke /curate rules

Use the Skill tool to invoke `/curate rules`. The curated learnings from Phase 1 are already in context, which helps /curate detect PASSIVE rules (internalized by agents) and gap-area promote candidates.

### 2b. Collect Results

Note from the rules curate output:
- Relevance distribution (high/medium/low/passive) and freshness distribution (fresh/aging/stale)
- Total passive context budget (lines across all rule files)
- Potential savings from low-relevance or stale rules
- Any gaps where upcoming work lacks guardrails

### 2c. Progress Check

```markdown
### Rules Curate Phase Complete

| Metric | Value |
|--------|-------|
| Rules scored | [n] |
| Relevance high + medium | [n] |
| Relevance low + passive (review) | [n] |
| Freshness stale | [n] |
| Total lines | [n] |
| Potential savings | [n] lines |
| Gaps found | [n] |
```

---

## Phase 3: Promote

After both agents and rules are curated, run `/promote` to graduate durable cross-agent patterns to rules.

### 3a. Invoke /promote

Use the Skill tool to invoke `/promote`. The curated learnings from Phase 1 and curated rules from Phase 2 are already in context, so /promote can read both.

### 3b. Collect Results

Note:
- How many promotion candidates were identified
- How many were actually promoted to rules
- Any candidates deferred (and why)

---

## Phase 4: Demotion Pass

After promotion, check existing rules for staleness and consider strength downgrades. This is the bidirectional lifecycle — rules can be promoted (via /promote) and demoted (via this pass).

### 4a. Renew Freshness for Active Rules

Before scanning for demotions, bump `freshness:` on rules whose governed files have recent activity. This prevents actively-relevant rules from being demoted just because no one edited the rule file itself.

For each rule with `paths:` patterns, check if governed files had commits in the last 90 days:

```bash
git log --oneline --since="90 days ago" -- <paths patterns> 2>/dev/null
```

If there is activity, update the rule's `freshness:` frontmatter to today's date. This resets the 90-day demotion clock.

Rules without `paths:` (unconditional rules) are not auto-renewed — they require manual review or edit to stay fresh.

Report renewals:

```markdown
### Freshness Renewed

| Rule | Governed Files Activity | Previous Freshness | New Freshness |
|------|------------------------|--------------------|---------------|
| [filename] | [n] commits in 90 days | [old date] | [today] |
```

If no rules qualify for renewal, note "No freshness renewals — no governed-file activity detected."

### 4b. Scan Rule Freshness

For each rule file with `strength:` and `freshness:` frontmatter:

```bash
git log --oneline --since="90 days ago" -- rules/*.md .claude/rules/*.md 2>/dev/null
```

Cross-reference each rule's `freshness:` date with git activity on files the rule governs (using `paths:` patterns if present). Rules renewed in 4a will naturally pass this check.

### 4c. Identify Demotion Candidates

| Condition | Action |
|-----------|--------|
| `freshness:` >90 days AND no git activity in governed files | Downgrade strength one level: `must` → `should` → `may` |
| `strength: may` AND `freshness:` >180 days | Flag for retirement review |

A rule is "governed" by the files matching its `paths:` patterns. Rules without `paths:` (unconditional) use their topic area — check if any files in that domain had recent commits.

### 4d. Apply Demotions (with confirmation)

Present demotion candidates:

```markdown
### Demotion Candidates

| Rule | Current Strength | Freshness | Governed Activity | Proposed Action |
|------|-----------------|-----------|-------------------|-----------------|
| [filename] | [strength] | [date] | [last activity] | downgrade to [new strength] |
| [filename] | may | [date] | [last activity] | flag for retirement review |
```

Ask: "Apply these demotions? (y/n/select)"

If approved, update the `strength:` field in each rule's frontmatter and set `freshness:` to today's date (the demotion is itself a freshness event).

### 4e. Report

Note how many rules had freshness renewed, how many were demoted, how many flagged for retirement, and how many were unchanged.

---

## Phase 5: Summary Report

Present a unified report of the full lifecycle run:

```markdown
## Tend Report

### Scope
- **Agents tended**: [list]
- **Upcoming work considered**: [count] ready tasks, [count] in-progress, [count] epics

### Agent Curate Results
| Agent | Before | After | Archived | Added | Gaps |
|-------|--------|-------|----------|-------|------|
| [name] | [n] | [n] | [n] | [n] | [n] |

### Rules Curate Results
| Metric | Value |
|--------|-------|
| Rules scored | [n] |
| Relevance high + medium | [n] |
| Relevance low + passive (review) | [n] |
| Freshness stale | [n] |
| Total passive context lines | [n] |
| Potential savings | [n] lines |
| Rule gaps | [n] |

### Promote Results
- **Candidates identified**: [n]
- **Promoted to rules**: [n] ([list rule names/files])
- **Deferred**: [n] ([brief reasons])

### Demotion Results
- **Freshness renewed**: [n] ([list: filename old-date→today])
- **Rules demoted**: [n] ([list: filename strength→strength])
- **Flagged for retirement**: [n] ([list filenames])
- **Unchanged**: [n]

### Net Effect
- **Learnings optimized**: [total items curated across all agents]
- **New rules created**: [count]
- **Rules demoted**: [count]
- **Knowledge gaps flagged**: [count] (create beads with /bug or bd create if actionable)

### Recommended Next Steps
- [Any gaps that need investigation tasks]
- [Any deferred promotions that need more evidence]
- [Suggested next /tend cadence based on learnings velocity]
```

---

## Guidelines

1. **Run curate before promote.** Curate ensures learnings are clean and relevant. Promote then works with high-quality input.
2. **Don't force promotions.** If /promote finds no candidates meeting graduation criteria, that's fine. Better no rule than a premature one.
3. **Flag gaps as actionable items.** If curate reveals knowledge gaps for upcoming work, surface them — they may warrant investigation beads.
4. **Respect the cadence.** Weekly or bi-weekly is a good rhythm. Running after every sprint is ideal. Running mid-sprint adds overhead without enough new signal.
5. **Single agent is fine.** If one agent just finished heavy work, tend just that agent — no need to curate everyone every time.

See also: /curate (standalone learnings optimization), /promote (standalone rule graduation), /retro (session-level reflection that feeds learnings), /sprint (dispatch loop that generates learnings).
