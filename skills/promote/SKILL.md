---
name: promote
description: "Analyze learnings across all agents to identify durable patterns ready to graduate to project rules. Applies graduation criteria: survival across 3+ retro cycles, 21+ day stability, cross-agent applicability, universality, actionability, and no overlap with existing rules. Outputs promotion candidates with draft rule content. Optionally writes new rule files. Use after /curate, /retro, or directly via /tend. Keywords: promote, rules, graduate, learnings, patterns, cross-agent, durable, rules, governance."
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Bash(ls:*), Write, Edit
---

# Promote: Graduate Durable Patterns to Rules

You are running **promote** — scanning all agent learnings for patterns that have earned their place as project rules. A pattern graduates when it has survived repeated retro cycles, remained stable, applies across agents, and can be phrased as a testable constraint.

**Optional filter:** $ARGUMENTS — if provided, restrict candidates to those matching this keyword or agent name.

## When to Use

- After `/curate` produces a curated learnings set in context — promote reads it directly
- After `/retro` surfaces promotion candidates in Phase 4f lifecycle analysis
- Via `/tend`, which orchestrates curate then promote in sequence
- Standalone, when you suspect accumulated learnings contain rule-grade patterns
- After a team has been running for several sprints and rules feel incomplete

## How It Works

```
Check context for /curate output
  -> Load all agent learnings + archives
    -> Check git history for survival / stability signals
      -> Detect overlap with existing rules
        -> Apply graduation criteria (all 6 must pass)
          -> Emit pipe-format report with Draft Rules section
            -> (Optional) Write approved rules to rules/ or .claude/rules/
```

---

## Phase 0: Context Check

Before loading from disk, scan conversation context for a prior `/curate` output block — the `## Curated Learnings: ... / **Source**: /curate` pattern.

**If /curate output is present in context:**

State: "Reading curated learnings from /curate output above."

Note from the curate output:
- Which entries are tagged `cross-agent: true` — these are pre-flagged candidates
- Which entries have `KEEP (HIGH)` scores — stable, valued entries
- The agent name from the header (for scoping survival checks)
- Any `### Gaps` section (not relevant for promotion — skip it)

This pre-screening narrows the candidate pool before loading full learnings from disk.

**If no /curate output is in context:**

Proceed to Phase 1 without pre-screening. All learnings will be evaluated.

---

## Phase 1: Load Learnings

### 1a. Discover All Agent Learnings

```bash
ls memory/agents/*/learnings.md 2>/dev/null
ls memory/agents/*/archive.md 2>/dev/null
```

Read each `learnings.md` file found. For each file, note:
- Agent name (directory basename)
- Total line count
- Each distinct entry with its provenance metadata: `(added: YYYY-MM-DD, dispatch: <source>)`

### 1b. Load Archive Files

Read each `memory/agents/*/archive.md` that exists. Archive entries can reveal historical patterns — an entry archived and re-added across multiple cycles is a strong promotion signal.

### 1c. Load Existing Rules

```bash
ls rules/*.md .claude/rules/*.md 2>/dev/null
```

Read each rule file. Read `CLAUDE.md`. Build a topic map of what is already covered passively — these are used in Phase 3 for overlap detection.

---

## Phase 2: Collect Raw Candidates

### 2a. Build the Candidate Pool

From the learnings loaded in Phase 1, collect entries that are plausible rule candidates based on surface characteristics:

- Entries that appear in 2+ agents' learnings files (textually similar or semantically equivalent)
- Entries pre-flagged `cross-agent: true` from a /curate output in context
- Entries that are general rather than task-specific (no file paths, no sprint-specific context)
- Entries phrased as prescriptions ("always X", "prefer X over Y", "when X, do Y")

**If $ARGUMENTS is a keyword or agent name**, filter to candidates that match the argument before continuing.

### 2b. Deduplicate and Cluster

Group semantically similar entries across agents into clusters. A cluster is a set of entries that express the same underlying pattern. Each cluster becomes one candidate item.

For each cluster, record:
- The agent names that contributed an entry
- The earliest `added:` date across entries (used for stability check)
- Representative text (the clearest expression of the pattern)

---

## Phase 3: Apply Graduation Criteria

For each candidate cluster, evaluate all six criteria. **All six must pass** for promotion. Mark each as PASS or FAIL with a brief reason.

### Criterion 1: Survival (3+ retro cycles)

Check the git history of the contributing learnings files:

```bash
git log --follow --oneline -- memory/agents/<name>/learnings.md 2>/dev/null
```

A survival cycle is a retro commit where the entry was present and not removed. Look for commits with "retro" in the message that span the entry's lifecycle.

**PASS**: Entry appears across 3+ retro-tagged commits without being removed.
**FAIL**: Entry is newer than 3 retro cycles, or was added and removed within the same cycle.

**Graceful degradation**: If git history is unavailable or the file has fewer than 3 retro commits, check entry age instead. An entry older than 60 days with a stable `added:` date is a weak PASS signal — note the limitation.

### Criterion 2: Stability (not modified in 21+ days)

```bash
git log --follow --oneline --since="21 days ago" -- memory/agents/<name>/learnings.md 2>/dev/null
```

**PASS**: No modifications to the entry content in the last 21 days. The pattern is settled, not actively evolving.
**FAIL**: Entry text was revised recently — the learning is still being refined.

### Criterion 3: Cross-Agent Applicability (2+ agents)

**PASS**: 2 or more distinct agents have an entry in this cluster, or a /curate output in context flagged the entry `cross-agent: true`.
**FAIL**: Only one agent has this pattern. A single-agent pattern belongs in that agent's learnings, not in a global rule.

### Criterion 4: Universality (not domain-specific)

**PASS**: The pattern applies regardless of which agent is running or which type of task is being executed. It is a workflow constraint, not a domain heuristic.
**FAIL**: The pattern applies only to skill authoring, only to sprint planning, or only to a specific file type. Domain-specific knowledge stays in agent learnings.

### Criterion 5: Actionability (testable rule)

**PASS**: Can be phrased as "do X", "prefer X over Y", or "when X, do Y". A reviewer can determine whether the rule was followed or violated.
**FAIL**: Vague or aspirational ("think carefully about X", "be thorough"). Cannot be checked.

### Criterion 6: No Overlap (not already a rule)

Compare the pattern against the topic map built in Phase 1c.

**PASS**: No existing rule in `rules/` or `.claude/rules/` covers this pattern.
**FAIL**: An existing rule already addresses this behavior — the learning is PASSIVE (redundant). Cite the specific rule filename.

---

## Phase 4: Compose Output

Emit in pipe format per `rules/pipe-format.md`:

```markdown
## Promotion Candidates: Agent Learnings -> Rules

**Source**: /promote
**Input**: all agents (or: $ARGUMENTS if provided)
**Pipeline**: /curate (N items) -> /promote (M candidates) [if /curate output was in context]
OR
**Pipeline**: (none — working from direct input)

### Items (N)

1. **PROMOTE: <pattern title>** — <representative text of the pattern>
   - agents: <comma-separated list of agents with this pattern>
   - survival: PASS — present in [n] retro commits (oldest: YYYY-MM-DD)
   - stability: PASS — last modified YYYY-MM-DD ([n] days ago)
   - cross-agent: PASS — found in [agent1], [agent2]
   - universality: PASS — applies to all agents regardless of domain
   - actionability: PASS — "prefer X over Y" (testable)
   - no-overlap: PASS — not covered by any existing rule
   - target: rules/<suggested-filename>.md | .claude/rules/<suggested-filename>.md
   - placement-reason: <why this tier — every session vs. project-local>

2. **DEFER: <pattern title>** — <representative text>
   - agents: <comma-separated list>
   - survival: FAIL — only [n] retro cycles detected; needs [3-n] more
   - stability: PASS
   - cross-agent: PASS
   - universality: PASS
   - actionability: PASS
   - no-overlap: PASS
   - defer-reason: Insufficient retro survival. Re-evaluate after next 2 retros.

3. **SKIP: <pattern title>** — <representative text>
   - agents: <agent-name only>
   - cross-agent: FAIL — single agent only; keep in agent learnings
   - skip-reason: Single-agent pattern; not a rule candidate.

### Draft Rules (N)

For each PROMOTE candidate, provide the draft markdown for the rule file:

---
**Draft: rules/<filename>.md**

# <Rule Title>

<One paragraph explaining the constraint and its rationale.>

## Rule

<The testable constraint in "do X", "prefer X over Y", or "when X do Y" form.>

## Rationale

<Why this constraint exists — what it prevents or enables.>

---

### Summary

<One paragraph: how many candidates were evaluated, how many passed all criteria, how many were deferred (and the most common reason), how many were skipped as single-agent. Note whether /curate context was used and whether it helped narrow the candidate pool. Mention if any candidates are borderline and warrant a second opinion.>
```

**Item ordering**: PROMOTE first (most actionable), then DEFER (pending), then SKIP (closed).

---

## Phase 5: Placement Decision

For each PROMOTE candidate, determine where the rule file should live. Apply `rules/information-architecture.md` principles:

| Question | If Yes | If No |
|----------|--------|-------|
| Does every session need this? | `rules/<file>.md` (passive, auto-loaded) | Continue to next question |
| Do most sessions in this project need this? | Add to `CLAUDE.md` under relevant section | Continue |
| Do some sessions need this (project-specific)? | `.claude/rules/<file>.md` (project-local) | Keep as learnings only |

**Universal constraints** (applicable in any Claude Code project) → `rules/`
**Project-specific constraints** (only apply in this codebase) → `.claude/rules/`
**Orientation-level context** (high-level workflow) → `CLAUDE.md`

Record the placement rationale for each candidate in its `target:` and `placement-reason:` fields.

---

## Phase 6: Write-Back (Conditional)

After presenting the output, ask the user:

> "Promote these [N] candidates? I'll write rule files as shown in the Draft Rules section. (y/n/select)"

If `y`: Write all PROMOTE candidates.
If `n`: Stop. The pipe-format report is the primary artifact.
If `select`: List numbered candidates and ask which to write.

### 6a. Write New Rule Files

For each approved candidate, write the draft rule to the target path:

```
rules/<filename>.md
.claude/rules/<filename>.md
```

Use the exact markdown from the `### Draft Rules` section as the file content.

### 6b. Verify No Conflict

Before writing, check that the target file does not already exist:

```bash
ls <target-path> 2>/dev/null
```

If the file exists, stop and ask: "A file already exists at <target-path>. Overwrite, merge, or skip?"

Do not overwrite without explicit confirmation.

### 6c. Verify After Write

Read each written file and confirm:
- The content matches the approved draft
- The file is valid markdown with a proper heading

---

## Guidelines

1. **All six criteria must pass.** A pattern that is cross-agent, actionable, and stable but only survived 2 retro cycles is a DEFER, not a PROMOTE. The criteria protect against premature graduation.

2. **DEFER is not a failure.** A deferred candidate is valuable — it means the pattern is promising but needs more time. Record the gap (e.g., "needs 1 more retro cycle") so the next /promote run knows where to look first.

3. **Cite specific rule files for overlap.** When skipping a candidate because it overlaps an existing rule, name the file: "covered by rules/commits.md". Vague overlap claims cannot be verified.

4. **Placement follows IA principles.** Rules that every project needs go in `rules/`. Rules that only apply to this codebase go in `.claude/rules/`. When in doubt, prefer project-local — it is easier to promote than to demote.

5. **Draft rule text must be testable.** Before emitting a draft, verify it can be checked. "Prefer X over Y" — can a reviewer look at a file and determine if the rule was followed? If not, rephrase until it can.

6. **Don't promote single-agent patterns.** If only one agent has a pattern, it belongs in that agent's Core learnings, not in a rule. Rules multiply value across all agents and sessions — single-agent patterns do not benefit from that scope.

7. **Read /curate output if it exists in context.** The `cross-agent: true` flag from /curate is a strong pre-screening signal. Use it to prioritize the candidate pool rather than re-deriving from scratch.

8. **Git history is the ground truth for survival.** Self-reported `added:` dates in learnings files can be wrong. Always verify survival against git log of the learnings file. Note when git history is unavailable and state the limitation.

See also: /curate (upstream — produces scored learnings set that /promote consumes), /tend (orchestrates curate + promote in sequence), /retro (Phase 4f surfaces promotion candidates for /promote to evaluate), /consolidate (backlog review — analogous lifecycle for beads, not learnings).
