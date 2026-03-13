---
name: diagnose-agent
description: "Use when an agent's performance is uneven and you want to understand where it excels or struggles. Produces a struggle profile for /challenge-gen or /active-learn. Keywords: diagnose, profile, agent, weakness, strength, capability, assessment."
argument-hint: "<agent-name>"
disable-model-invocation: false
user-invocable: true
allowed-tools: [Read, Grep, Glob, Task, "Bash(git:*)", "Bash(wc:*)"]
context: inline
---

# Diagnose Agent: Struggle Profile from Historical Evidence

You are running **diagnose-agent** — profiling a team agent's strengths and weaknesses from learnings evolution and commit history. Target agent: **$ARGUMENTS**

## Phase 0: Resume Check

<!-- Condition: only if $ARGUMENTS contains "resume:<id>" -->
If `$ARGUMENTS` contains the prefix `resume:<id>` (e.g., `resume:abc123 dig deeper into WEAKNESS: error handling`):

1. Extract the agent ID (`abc123`) and the follow-up directive (the text after the ID)
2. Resume the prior analysis agent: `Task({ resume: "<id>", prompt: "<follow-up directive>" })`
3. Do not run Phases 1-5. The resumed agent already has full context from the prior analysis.

If `$ARGUMENTS` does not contain `resume:`, proceed to the standard Phase 0 below.

## When to Use

- Before generating training challenges — understand where an agent struggles first
- When an agent's performance seems uneven and you want evidence-based assessment
- When onboarding a new task type to an agent and want to calibrate difficulty
- As input to /challenge-gen, /rank, or /assess via pipe format

## Don't Use When

- Agent has fewer than 5 learnings entries — insufficient history for meaningful signal; run the agent on real tasks first to build evidence
- Agent was just created and has no git activity on owned files — both Phase 2 (learnings evolution) and Phase 3 (commit signals) need history to analyze
- You want to improve the agent, not just profile it — use /active-learn, which includes diagnosis as part of a full training cycle

## Prerequisites

This skill requires the following infrastructure:

- **team.yaml** — Agent must be listed in `.claude/team.yaml` with `ownership` patterns (file globs); these drive Phase 3 commit signal analysis ([how to create](/assemble))
- **learnings file** — `.claude/tackline/memory/agents/<agent>/learnings.md` must exist with at least 5 entries; below this threshold there is insufficient signal for meaningful weakness detection ([populated by /retro and /active-learn](/retro))
- **git history for the learnings file** — Phase 2 runs `git log -p` on the learnings file to detect entry churn, survival, and velocity; a file with no commits (never version-controlled) will produce empty results for Phase 2

If the learnings file is missing, the skill produces a partial profile from git commit signals alone (Phase 3 only) and notes the limitation in the Summary. If the agent is not in team.yaml, the skill stops and lists available agents. If the learnings file has no git history, Phase 2 is skipped and the profile relies entirely on Phase 3 signals.

## How It Works

```
Validate agent exists
  -> Gather learnings evolution (git log diffs)
    -> Gather task performance signals (raw git)
      -> Analyze dispatch provenance (if available)
        -> Synthesize struggle profile (pipe format)
```

---

## Phase 1: Validate Agent Exists

If `$ARGUMENTS` is empty:
1. Read `.claude/team.yaml` and list available agent names
2. Ask the user: "Which agent should I diagnose? Available: [names]"
3. Stop and wait for response

If `$ARGUMENTS` is provided:
1. Read `.claude/team.yaml` and confirm the agent name exists in the `members` list
2. Check that `.claude/tackline/memory/agents/<name>/learnings.md` exists
3. If agent not found in team.yaml, error: "Agent '<name>' not found in .claude/team.yaml. Available agents: [list]."
4. If learnings file missing, note it — the skill can still produce partial results from git signals alone, but learnings evolution (Phase 2) will be skipped

---

## Phase 2: Gather Learnings Evolution

Read the agent's learnings history from git:

```bash
# Edit timeline
git log --oneline --follow -- .claude/tackline/memory/agents/<name>/learnings.md

# Actual diffs to see what changed
git log -p --follow -- .claude/tackline/memory/agents/<name>/learnings.md
```

Parse the diffs to extract five signals:

### 2a. Entry Survival (-> STRENGTHS)

Entries that persisted across 3+ retro edits without removal. These represent durable knowledge the agent reliably applies. Each surviving entry is a confirmed strength area.

### 2b. Entry Churn (-> FALSE POSITIVES)

Entries added then removed within 1-2 edits. These are things the agent thought were learnings but proved wrong, irrelevant, or superseded. High churn in a category suggests the agent is struggling to form stable mental models there.

### 2c. Entry Velocity

Rate of new entries per session (count new entries / count edit commits). Interpret:
- **High velocity** (>2 entries/session): actively learning, encountering novel problems
- **Medium velocity** (1-2 entries/session): steady growth
- **Low velocity** (<1 entry/session): plateau (tasks are easy) or stagnation (tasks are too hard to learn from)

### 2d. Category Distribution

Which sections of the learnings file are growing? Map to interpretation:
- **Gotchas growing**: encountering problems, tripping on edge cases
- **Codebase Patterns growing**: building structural understanding
- **Preferences growing**: developing style and workflow habits
- **Task-Relevant growing**: accumulating domain-specific knowledge

### 2e. Sparse Categories

Categories with few or no entries represent potential knowledge gaps. An agent with many Gotchas but no Codebase Patterns may be reacting to problems without building systemic understanding.

---

## Phase 3: Gather Task Performance Signals

Read `.claude/team.yaml` to extract the agent's `owns` patterns (file globs defining ownership).

```bash
# Recent activity on owned files
git log --oneline --since="30 days ago" -- <owns-patterns>

# Check for fix-after-feat patterns
git log --oneline --since="30 days ago" -- <owns-patterns> | head -40
```

Scan commit messages for `fix:` commits that follow `feat:` commits on the same files within 7 days.

Look for:
- **fix_after_feat on owned files**: agent ships features that need immediate fixes (quality signal)
- **High churn on owned files**: instability in agent's domain (design signal)
- **Concentrated activity in owned area**: files the agent touches frequently (focus signal — or thrashing signal if combined with fix_after_feat)

---

## Phase 4: Analyze Dispatch Provenance (Conditional)

**Skip this phase if learnings entries lack `dispatch:` fields.**

If entries contain `(dispatch: bead-xyz)` annotations:

1. Group learnings by their dispatch source (bead ID)
2. Identify which task types produced the most learnings — these are areas of **active growth** (the agent is learning from challenging work)
3. Identify which task types produced entries that were later pruned — these are areas of **false confidence** (the agent overclaimed what it learned)
4. Cross-reference with Phase 3: do dispatch-heavy task types correlate with fix_after_feat signals?

---

## Phase 5: Synthesize Struggle Profile

Combine signals from Phases 2-4 into a ranked list. Ranking criteria:
- **WEAKNESS** items rank by severity: HIGH (multiple corroborating signals), MEDIUM (single strong signal), LOW (suggestive pattern)
- **STRENGTH** items rank by durability: long-surviving learnings with low churn in that area rank highest
- **GAP** items rank by coverage importance: gaps in areas the agent owns rank higher than gaps in peripheral areas

### Difficulty Calibration

Estimate the agent's current coverage of their domain:
- Count durable learnings entries vs estimated total domain complexity (based on file count and diversity in owned patterns)
- Map to recommended challenge level:
  - **<25% coverage**: novice challenges (fill foundational gaps)
  - **25-50% coverage**: intermediate challenges (expand from known areas)
  - **50-75% coverage**: expert challenges (edge cases and cross-cutting concerns)
  - **>75% coverage**: adversarial challenges (failure modes, unusual inputs, multi-system interactions)

---

## Output Format

Emit in pipe format:

```markdown
## Struggle Profile: <agent-name>

**Source**: /diagnose-agent
**Input**: <agent-name>
**Pipeline**: (none — working from direct input)

### Items (N)

1. **WEAKNESS: <area>** — <description of what the agent struggles with>
   - evidence: <what signal revealed this>
   - type: WEAKNESS
   - severity: HIGH | MEDIUM | LOW
   - source: <file path or git command that produced the signal>

2. **STRENGTH: <area>** — <description of what the agent reliably handles>
   - evidence: <what signal revealed this>
   - type: STRENGTH
   - severity: HIGH | MEDIUM | LOW
   - source: <file path or git command that produced the signal>

3. **GAP: <area>** — <description of missing coverage>
   - evidence: <what signal revealed this>
   - type: GAP
   - severity: HIGH | MEDIUM | LOW
   - source: <file path or git command that produced the signal>

...

### Difficulty Calibration

Current coverage: <percentage estimate of domain covered by learnings>
Recommended challenge level: <novice | intermediate | expert | adversarial>
Basis: <brief explanation of how coverage maps to difficulty>

### Summary

<One paragraph synthesizing the agent's capability state — where they are strong, where they struggle, and what kind of challenges would produce the most growth.>

### Resume

Agent ID: <task-agent-id of this diagnose-agent run>

To dig deeper into a specific finding:

    /diagnose-agent resume:<agent-id> dig deeper into WEAKNESS: <area>
```

Order items: WEAKNESS (HIGH first) -> GAP (HIGH first) -> STRENGTH (HIGH first). This puts actionable items at the top for downstream consumption.

---

## Guidelines

1. **Compaction resilience**: Per `rules/memory-layout.md`, checkpoint at phase boundaries to `.claude/tackline/memory/scratch/diagnose-agent-checkpoint.md`.
2. **Read-only.** No file writes, no git operations beyond read-only log/diff/show. No modifications to learnings, team.yaml, or any other file.
3. **Evidence-based.** Every item in the output must cite a specific source — a git diff, a learnings entry, or a file path. No speculation without evidence.
4. **Graceful degradation.** Work with whatever is available. No dispatch provenance? Skip Phase 4. No learnings file? Produce a partial profile from git signals alone, and note the limitation in the Summary.
5. **Diagnose, do not prescribe.** The output describes what IS, not what to DO about it. Challenge generation is /challenge-gen's job. Training plans are /active-learn's job.
6. **Honest calibration.** Do not inflate coverage estimates. If the agent has 5 learnings entries for a domain with 30+ files, coverage is low regardless of how good those 5 entries are.
7. **Cite the diff, not the conclusion.** When reporting entry churn, quote the actual entry that was added and removed — not just "entries were churned in the Gotchas section."
