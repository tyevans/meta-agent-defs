---
name: diagnose-agent
description: "Profile a team agent's strengths and weaknesses from learnings evolution, commit history, and git-intel signals. Outputs a struggle profile in pipe format for downstream consumption by /challenge-gen or /active-learn. Use when you want to understand where an agent excels or struggles before generating training challenges. Keywords: diagnose, profile, agent, weakness, strength, capability, assessment."
argument-hint: "<agent-name>"
disable-model-invocation: false
user-invocable: true
allowed-tools: [Read, Grep, Glob, "Bash(git:*)", "Bash(git-intel:*)", "Bash(wc:*)"]
context: inline
---

# Diagnose Agent: Struggle Profile from Historical Evidence

You are running **diagnose-agent** — profiling a team agent's strengths and weaknesses from learnings evolution, commit history, and git-intel signals. Target agent: **$ARGUMENTS**

## When to Use

- Before generating training challenges — understand where an agent struggles first
- When an agent's performance seems uneven and you want evidence-based assessment
- When onboarding a new task type to an agent and want to calibrate difficulty
- As input to /challenge-gen, /rank, or /assess via pipe format

## How It Works

```
Validate agent exists
  -> Gather learnings evolution (git log diffs)
    -> Gather task performance signals (git-intel or raw git)
      -> Analyze dispatch provenance (if available)
        -> Synthesize struggle profile (pipe format)
```

---

## Phase 0: Validate Agent Exists

If `$ARGUMENTS` is empty:
1. Read `.claude/team.yaml` and list available agent names
2. Ask the user: "Which agent should I diagnose? Available: [names]"
3. Stop and wait for response

If `$ARGUMENTS` is provided:
1. Read `.claude/team.yaml` and confirm the agent name exists in the `members` list
2. Check that `memory/agents/<name>/learnings.md` exists
3. If agent not found in team.yaml, error: "Agent '<name>' not found in .claude/team.yaml. Available agents: [list]."
4. If learnings file missing, note it — the skill can still produce partial results from git signals alone, but learnings evolution (Phase 1) will be skipped

---

## Phase 1: Gather Learnings Evolution

Read the agent's learnings history from git:

```bash
# Edit timeline
git log --oneline --follow -- memory/agents/<name>/learnings.md

# Actual diffs to see what changed
git log -p --follow -- memory/agents/<name>/learnings.md
```

Parse the diffs to extract five signals:

### 1a. Entry Survival (-> STRENGTHS)

Entries that persisted across 3+ retro edits without removal. These represent durable knowledge the agent reliably applies. Each surviving entry is a confirmed strength area.

### 1b. Entry Churn (-> FALSE POSITIVES)

Entries added then removed within 1-2 edits. These are things the agent thought were learnings but proved wrong, irrelevant, or superseded. High churn in a category suggests the agent is struggling to form stable mental models there.

### 1c. Entry Velocity

Rate of new entries per session (count new entries / count edit commits). Interpret:
- **High velocity** (>2 entries/session): actively learning, encountering novel problems
- **Medium velocity** (1-2 entries/session): steady growth
- **Low velocity** (<1 entry/session): plateau (tasks are easy) or stagnation (tasks are too hard to learn from)

### 1d. Category Distribution

Which sections of the learnings file are growing? Map to interpretation:
- **Gotchas growing**: encountering problems, tripping on edge cases
- **Codebase Patterns growing**: building structural understanding
- **Preferences growing**: developing style and workflow habits
- **Task-Relevant growing**: accumulating domain-specific knowledge

### 1e. Sparse Categories

Categories with few or no entries represent potential knowledge gaps. An agent with many Gotchas but no Codebase Patterns may be reacting to problems without building systemic understanding.

---

## Phase 2: Gather Task Performance Signals

Read `.claude/team.yaml` to extract the agent's `owns` patterns (file globs defining ownership).

### If `command -v git-intel` succeeds:

```bash
# Fix-after-feat signals on owned files
git-intel patterns --repo . --since 30d

# Churn on owned files
git-intel churn --repo . --since 30d

# Hotspots for volatility
git-intel hotspots --repo . --since 30d
```

Filter results to files matching the agent's `owns` patterns. Look for:
- **fix_after_feat on owned files**: agent ships features that need immediate fixes (quality signal)
- **High churn on owned files**: instability in agent's domain (design signal)
- **Hotspots in owned area**: files the agent touches frequently (focus signal — or thrashing signal if combined with fix_after_feat)

### If git-intel does not exist, fall back to raw git:

```bash
# Recent activity on owned files
git log --oneline --since="30 days ago" -- <owns-patterns>

# Check for fix-after-feat manually
git log --oneline --since="30 days ago" -- <owns-patterns> | head -40
```

Scan commit messages for `fix:` commits that follow `feat:` commits on the same files within 7 days. This is a rough approximation of git-intel's pattern detection.

---

## Phase 3: Analyze Dispatch Provenance (Conditional)

**Skip this phase if learnings entries lack `dispatch:` fields.**

If entries contain `(dispatch: bead-xyz)` annotations:

1. Group learnings by their dispatch source (bead ID)
2. Identify which task types produced the most learnings — these are areas of **active growth** (the agent is learning from challenging work)
3. Identify which task types produced entries that were later pruned — these are areas of **false confidence** (the agent overclaimed what it learned)
4. Cross-reference with Phase 2: do dispatch-heavy task types correlate with fix_after_feat signals?

---

## Phase 4: Synthesize Struggle Profile

Combine signals from Phases 1-3 into a ranked list. Ranking criteria:
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
```

Order items: WEAKNESS (HIGH first) -> GAP (HIGH first) -> STRENGTH (HIGH first). This puts actionable items at the top for downstream consumption.

---

## Guidelines

1. **Read-only.** No file writes, no git operations beyond read-only log/diff/show. No modifications to learnings, team.yaml, or any other file.
2. **Evidence-based.** Every item in the output must cite a specific source — a git diff, a git-intel signal, a learnings entry, or a file path. No speculation without evidence.
3. **Graceful degradation.** Work with whatever is available. No git-intel? Use raw git. No dispatch provenance? Skip Phase 3. No learnings file? Produce a partial profile from git signals alone, and note the limitation in the Summary.
4. **Diagnose, do not prescribe.** The output describes what IS, not what to DO about it. Challenge generation is /challenge-gen's job. Training plans are /active-learn's job.
5. **Honest calibration.** Do not inflate coverage estimates. If the agent has 5 learnings entries for a domain with 30+ files, coverage is low regardless of how good those 5 entries are.
6. **Cite the diff, not the conclusion.** When reporting entry churn, quote the actual entry that was added and removed — not just "entries were churned in the Gotchas section."
