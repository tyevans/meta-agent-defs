---
name: active-learn
description: "Run the full adversarial training loop for a team agent or in solo mode (no team required): diagnose weaknesses, generate challenges, execute rounds, and update learnings. The capstone of the active learning system. Keywords: active-learn, training, adversarial, improve, agent, loop, capability, tuning, solo."
argument-hint: "<agent-name> [rounds=3]"
disable-model-invocation: false
user-invocable: true
allowed-tools: [Read, Write, Grep, Glob, Task, WebSearch, WebFetch, "Bash(git:*)", "Bash(git-intel:*)", "Bash(mkdir:*)", "Bash(wc:*)"]
context: fork
---

# Active Learn: Adversarial Training Loop

You are running **active-learn** -- the full adversarial training loop that diagnoses an agent's weaknesses, generates targeted challenges, executes them in rounds, and persists confirmed learnings. This is the capstone orchestrator of the active learning system.

**Target agent:** $ARGUMENTS

## When to Use

- When an agent's performance is uneven and you want systematic improvement
- After noticing repeated failures in a domain -- run a structured training cycle
- When onboarding an agent to a new responsibility area
- When you want objective evidence of improvement (not just more learnings entries)
- As a periodic maintenance loop (e.g., monthly per agent) to prevent capability stagnation

## How It Works

```
Parse arguments (agent name, round count)
  -> Phase 1: Diagnose (inline profiling from learnings + git signals)
    -> Phase 2: Generate challenges (inline, 3-5 per round)
      -> Phase 3: Execute round (Task dispatch, serial, evaluate inline)
        -> Phase 4: Learn (update learnings.md, write capability.yaml)
          -> Re-diagnose: compare to previous struggle profile
            -> If new weaknesses found: loop to Phase 2
            -> If no new weaknesses or max rounds reached: emit training summary
```

---

## Phase 0: Parse Arguments and Validate

### 0a. Parse Input

Extract from `$ARGUMENTS`:
- **Agent name**: required (first token)
- **Round count**: optional (second token, format `rounds=N`, default 3)

Example inputs: `skill-author`, `rust-dev rounds=5`, `infra rounds=2`

If `$ARGUMENTS` is empty:
1. Read `.claude/team.yaml` and list available agent names
2. Ask the user: "Which agent should I train? Available: [names]"
3. Stop and wait for response

### 0b. Validate Agent (Team Mode or Solo Mode)

Determine operating mode by attempting to read `.claude/team.yaml`:

**If `.claude/team.yaml` exists AND the agent name is found in `members`:**
- Extract the agent's `role`, `model`, and `owns` patterns from the file
- Operating in **TEAM MODE** -- proceed normally

**If `.claude/team.yaml` does not exist OR the agent name is not found in it:**
- Operating in **SOLO MODE**
- Print to the user: "SOLO MODE: .claude/team.yaml not found or agent '<name>' not in team. Running as 'main-session'."
- Set learner persona to `main-session` for this run
- All file paths that would reference `memory/agents/<name>/` now use `memory/agents/solo/` instead
- The `role` is "main session", `model` is the current model, and `owns` is treated as the full project (no pattern filtering in git analysis)
- If `$ARGUMENTS` was empty and team.yaml also does not exist, skip listing available agents -- instead prompt: "Which topic or skill area should I train on? (No team.yaml found -- running in solo mode.)"

Read `memory/agents/<name>/learnings.md` (team mode) or `memory/agents/solo/learnings.md` (solo mode) if it exists -- note current line count as the baseline.

### 0c. Load Prior Capability Data

Check for existing capability tracking:
- Team mode: Read `memory/agents/<name>/capability.yaml` if it exists
- Solo mode: Read `memory/agents/solo/capability.yaml` if it exists
- Read any prior evaluation files from the appropriate challenges directory
- Note which weaknesses have been previously targeted (to avoid re-exercising resolved weaknesses)

### 0d. Prerequisite Gate

Do not proceed until:
- [ ] Mode determined (TEAM MODE or SOLO MODE) and announced to user
- [ ] Learner persona established: agent name (team) or `solo` (solo mode)
- [ ] Learnings file read (or confirmed absent for new agents)
- [ ] Round count established (parsed or default 3)
- [ ] Prior capability data loaded (or confirmed absent for first run)

---

## Phase 1: Diagnose (Inline Profiling)

Build a struggle profile from historical evidence. This embeds the approach from /diagnose-agent.

### 1a. Learnings Evolution

Read the agent's learnings history from git:

```bash
# Edit timeline
git log --oneline --follow -- memory/agents/<name>/learnings.md

# Actual diffs to see what changed
git log -p --follow -- memory/agents/<name>/learnings.md
```

Parse the diffs to extract:

- **Entry Survival**: Entries persisting across 3+ retro edits = confirmed strengths
- **Entry Churn**: Entries added then removed within 1-2 edits = false positives, suggest unstable knowledge
- **Category Distribution**: Which sections are growing? Gotchas growing = encountering problems. Codebase Patterns growing = building structural understanding. Sparse categories = knowledge gaps
- **Entry Velocity**: New entries per session. High (>2) = active learning. Low (<1) = plateau or stagnation

### 1b. Task Performance Signals

In team mode, read `.claude/team.yaml` to extract the agent's `owns` patterns. In solo mode, no `owns` filtering applies -- analyze all project files.

**If `command -v git-intel` succeeds:**

```bash
git-intel patterns --repo . --since 30d
git-intel churn --repo . --since 30d
git-intel hotspots --repo . --since 30d
```

Filter to files matching the agent's `owns` patterns. Look for:
- **fix_after_feat on owned files**: agent ships features that need immediate fixes
- **High churn on owned files**: instability in agent's domain
- **Hotspots in owned area**: concentrated activity (focus or thrashing)

**If git-intel is not available, fall back to raw git:**

```bash
git log --oneline --since="30 days ago" -- <owns-patterns>
```

Scan for `fix:` commits following `feat:` commits on the same files within 7 days.

### 1c. Dispatch Provenance (Conditional)

**Skip if learnings entries lack `dispatch:` fields.**

If entries contain `(dispatch: bead-xyz)` annotations:
1. Group learnings by dispatch source
2. Identify task types producing the most learnings (active growth areas)
3. Identify task types whose entries were later pruned (false confidence areas)

### 1d. Synthesize Struggle Profile

Combine signals into a ranked list:

- **WEAKNESS** items: ranked by severity (HIGH = multiple corroborating signals, MEDIUM = single strong signal, LOW = suggestive pattern)
- **GAP** items: ranked by coverage importance (gaps in owned areas rank higher)
- **STRENGTH** items: ranked by durability (long-surviving learnings with low churn)

**Difficulty Calibration:**
- Count durable learnings vs estimated domain complexity (file count and diversity in owned patterns)
- Map: <25% = novice, 25-50% = intermediate, 50-75% = expert, >75% = adversarial

Store the profile internally as the round-0 baseline. Order: WEAKNESS (HIGH first) -> GAP -> STRENGTH.

---

## Phase 2: Generate Challenges (Inline)

Generate 3-5 targeted challenges per round. This embeds the approach from /challenge-gen.

### 2a. Select Weakness Targets

From the current struggle profile, select weaknesses to target this round:
- Prioritize HIGH severity WEAKNESS and GAP items
- Skip weaknesses already resolved in prior rounds (if looping)
- Skip weaknesses targeted by prior /active-learn runs (from capability.yaml history)
- Each challenge targets a different weakness (no doubling up)

### 2b. Strategy A: Domain Edge Cases

For each selected weakness, search for real-world edge cases:

**Codebase edge cases:**
```bash
git log --oneline --all --since="60 days ago" -- <owns-patterns>
```

Look for complex patterns, error handling paths, configuration edge cases, cross-cutting concerns the learnings don't mention.

**External edge cases (WebSearch):**
- CVEs, security advisories in the technology stack
- Highly-voted edge case questions on Stack Overflow
- Framework-specific gotchas the agent may not have encountered

Verify each external finding applies to the codebase (check imports, packages, git history).

### 2c. Strategy B: Commit-Replay Candidates

**If `command -v git-intel` succeeds:**

```bash
git-intel patterns --repo . --since 60d
git-intel churn --repo . --since 60d
```

Filter to owned files. Prioritize fix_after_feat commits and high-churn files.

**If git-intel is not available:**

```bash
git log --format="%H %s" --since="60 days ago" -- <owns-patterns>
```

For each candidate commit:
```bash
git show --format="%B" --no-patch <commit-hash>
git show --format="" <commit-hash>
git show <commit-hash>^:<file-path>
```

### 2d. Assemble and Quality Gate

Combine candidates into 3-5 challenges. Every challenge MUST pass:
1. **Targets a specific weakness** from the current profile
2. **Has clear acceptance criteria** (pass/fail evaluable)
3. **Has a hidden trap** (non-obvious difficulty)
4. **Is grounded** in real code, real CVEs, real commits, or real patterns
5. **Is self-contained** (agent can attempt without full project context)

Drop any challenge that fails. If fewer than 3 survive, return to 2b/2c for more candidates.

Calibrate difficulty to the Difficulty Calibration level from Phase 1d.

### 2e. Write Challenge File

```bash
mkdir -p memory/agents/<name>/challenges   # team mode
mkdir -p memory/agents/solo/challenges     # solo mode
```

Write challenges to `memory/agents/<name>/challenges/<timestamp>-round-<N>-challenges.md` (team mode) or `memory/agents/solo/challenges/<timestamp>-round-<N>-challenges.md` (solo mode):

```markdown
# Round <N> Challenges for <agent-name>

Generated: <date>
Training cycle: /active-learn round <N>
Difficulty level: <novice | intermediate | expert | adversarial>
Targeted weaknesses: <list>

## Challenge 1: <title>

**Strategy**: edge-case | commit-replay
**Targeted weakness**: <WEAKNESS or GAP name>
**Difficulty**: <level>

### Scenario

<What the agent sees>

### Acceptance Criteria

<Pass/fail conditions>

### Hidden Trap

<What makes this hard -- NOT shown to agent>

### Ground Truth

<Correct approach -- for evaluation only>
```

---

## Phase 3: Execute Round (Task Dispatch + Inline Evaluation)

### 3a. Dispatch Each Challenge Serially

For each challenge, compose a self-contained prompt and dispatch via Task.

**Compose the prompt:**

In team mode use the agent name and role from team.yaml. In solo mode, substitute:
- Agent name: `main-session`
- Role: `main session practitioner`
- Learnings path: `memory/agents/solo/learnings.md`

> You are **<agent-name or "main-session">**, a <role> on the team.
>
> ## Your Learnings
>
> <contents of learnings.md>
>
> ## Task
>
> <scenario from challenge>
>
> ## Acceptance Criteria
>
> <acceptance criteria from challenge>
>
> ## Reflection
>
> After completing the task, provide:
> 1. **Approach**: What approach did you take and why?
> 2. **Confidence**: How confident are you in your solution? (LOW / MEDIUM / HIGH)
> 3. **Uncertainty**: What aspects are you least sure about?
> 4. **Alternative approaches**: What other approaches did you consider and why did you reject them?
> 5. **Risk areas**: Where might this solution break or be wrong?

**CRITICAL: Never include Hidden Trap or Ground Truth in the agent prompt.** This is the single most important constraint of the entire training loop. Including evaluation criteria defeats the challenge.

**Dispatch:**

Team mode:
```
Task({
  subagent_type: "general-purpose",
  model: "<agent's model from team.yaml>",
  prompt: "<composed prompt>"
})
```

Solo mode (no team.yaml): use the current session model:
```
Task({
  subagent_type: "general-purpose",
  model: "claude-sonnet-4-6",
  prompt: "<composed prompt>"
})
```

Serial dispatch only -- no `run_in_background`. Each challenge is a controlled experiment.

### 3b. Evaluate Each Challenge (Inline)

After each dispatch returns, evaluate on three dimensions:

**Result Assessment (PASS / PARTIAL / FAIL):**
- PASS: All acceptance criteria met, solution correct and complete
- PARTIAL: Some criteria met, main case works but misses edge cases
- FAIL: Primary criteria not met, fundamental misunderstanding

For commit-replay: compare against ground truth diff (semantic equivalence counts).
For edge-case: check if solution handles the specific trap scenario.

**Trap Detection (CAUGHT / PARTIAL / MISSED):**
- CAUGHT: Agent explicitly identified the trap, solution handles it
- PARTIAL: Agent showed awareness of difficulty in right area but incomplete
- MISSED: No awareness of trap, solution fails on trap scenario

**Confidence Calibration (OVER / UNDER / WELL_CALIBRATED):**

| Self-reported | Actual result | Calibration |
|--------------|---------------|-------------|
| HIGH | PASS | WELL_CALIBRATED |
| HIGH | PARTIAL | OVER |
| HIGH | FAIL | OVER |
| MEDIUM | PASS | UNDER (slightly) |
| MEDIUM | PARTIAL | WELL_CALIBRATED |
| MEDIUM | FAIL | OVER |
| LOW | PASS | UNDER |
| LOW | PARTIAL | WELL_CALIBRATED |
| LOW | FAIL | WELL_CALIBRATED |

Nuance: If agent reported HIGH confidence but correctly identified the trap in risk areas, lean toward WELL_CALIBRATED.

**Growth Signal:**
- **Strength emerging**: PASS + CAUGHT -- developing competence
- **Awareness growing**: PARTIAL + PARTIAL/CAUGHT -- recognizes difficulty, cannot yet solve
- **Blind spot persists**: any + MISSED -- does not recognize difficulty
- **Overcorrection risk**: PASS + CAUGHT + UNDER -- cautious but misplaced caution

Every rating MUST cite specific evidence from the agent's output. "MISSED" without explanation is not acceptable.

### 3c. Write Round Evaluation

Write to `memory/agents/<name>/challenges/<timestamp>-round-<N>-evaluation.md` (team mode) or `memory/agents/solo/challenges/<timestamp>-round-<N>-evaluation.md` (solo mode):

```markdown
# Round <N> Evaluation: <agent-name>

Evaluated: <date>
Training cycle: /active-learn round <N>

## Challenge 1: <title>

**Result**: PASS | PARTIAL | FAIL
**Trap Detection**: CAUGHT | PARTIAL | MISSED
**Confidence Calibration**: OVER | UNDER | WELL_CALIBRATED
**Targeted Weakness**: <weakness name>
**Growth Signal**: <signal>

### Agent's Approach
<Brief summary>

### Evaluation Notes
<Why these ratings. Cite evidence.>

---

## Round Summary

- Results: X/N passed, Y partial, Z failed
- Trap detection: X caught, Y partial, Z missed
- Calibration: X well-calibrated, Y over-confident, Z under-confident
```

---

## Phase 4: Learn and Update

### 4a. Extract Confirmed Learnings

From the round evaluation, extract learnings the agent should retain:

- **From PASS + CAUGHT challenges**: The agent already knows this -- no new learning needed, but confirm the strength
- **From PARTIAL challenges**: Extract the specific gap that caused partial failure -- this becomes a new Gotcha or Codebase Pattern entry
- **From FAIL + MISSED challenges**: Extract the blind spot -- this becomes a high-priority Gotcha entry
- **From OVER-confident challenges**: Extract the calibration lesson -- "When working on X, verify Y before committing to an approach"

### 4b. Update Learnings File

Read `memory/agents/<name>/learnings.md` (team mode) or `memory/agents/solo/learnings.md` (solo mode). Append new entries tagged with challenge provenance:

```markdown
- <learning description> (source: /active-learn round N, challenge: <title>, <date>)
```

Rules:
- Do not duplicate existing entries -- if a learning already exists in substance, skip it
- Respect the 60-line cap (30 core + 30 task-relevant). If at capacity, the entry must be more valuable than the weakest existing entry to displace it
- New entries go in the most appropriate section (Gotchas, Codebase Patterns, Preferences, etc.)

### 4c. Write Capability YAML

Write or update `memory/agents/<name>/capability.yaml` (team mode) or `memory/agents/solo/capability.yaml` (solo mode):

```yaml
# Capability tracking for <agent-name or "solo">
# Updated by /active-learn on <date>

agent: <name or "solo">   # "solo" when running without a team
last_updated: <ISO date>
total_rounds: <cumulative across all /active-learn runs>
total_challenges: <cumulative>

domains:
  <domain-area-1>:
    pass_rate: <X/N>
    trap_detection_rate: <X/N>
    calibration: <dominant pattern: over | under | well_calibrated>
    trajectory: improving | stable | declining
    last_tested: <ISO date>

  <domain-area-2>:
    pass_rate: <X/N>
    trap_detection_rate: <X/N>
    calibration: <pattern>
    trajectory: <direction>
    last_tested: <ISO date>

persistent_weaknesses:
  - <weakness that survived multiple rounds>
  - ...

confirmed_strengths:
  - <strength validated by PASS + CAUGHT>
  - ...

history:
  - date: <ISO date>
    rounds: <N>
    pass_rate: <overall X/N>
    new_learnings: <count added>
    early_stopped: <true/false>
    reason: <if early stopped, why>
```

Domain areas map to the agent's `owns` patterns and the weakness categories from the struggle profile. If capability.yaml already exists, merge new data -- increment cumulative counts, update pass rates per domain, append to history.

---

## Phase 5: Re-Diagnose and Loop Decision

### 5a. Re-Diagnose

After completing a round, re-run Phase 1 logic (lightweight -- focus on the round's results rather than full git analysis):

1. Compare the current struggle profile against the previous round's profile
2. Check: did any WEAKNESS items change severity? Did any GAP items get filled?
3. Check: did any NEW weaknesses surface that were not in the previous profile?

### 5b. Early Stopping Criteria

**Stop the loop if ANY of these are true:**
- **No new weaknesses**: The re-diagnosis found no weaknesses that were not already in the previous profile. The agent's capability frontier has not expanded.
- **All challenges passed**: The agent passed all challenges in the round with CAUGHT trap detection. Challenges are too easy -- the agent needs harder material from a fresh /diagnose-agent run.
- **Maximum rounds reached**: The configured round count (default 3) has been exhausted.
- **Learnings saturated**: The learnings file is at the 60-line cap and no new entries are more valuable than existing ones.

**Continue the loop if:**
- New weaknesses or shifted severities were found
- The agent showed PARTIAL results that could improve with updated learnings
- Rounds remain and there are untargeted weaknesses

If continuing: return to Phase 2 with the updated struggle profile. The new round targets weaknesses that emerged or persisted.

---

## Phase 6: Emit Training Summary

After the loop completes (early stop or max rounds), emit the final summary.

### 6a. Pipe Format Output

In solo mode, prepend the SOLO MODE label to the summary block so the user knows which mode was active.

```markdown
<!-- Solo mode only: -->
> **SOLO MODE** -- No team.yaml found. Ran as 'main-session'. Results written to memory/agents/solo/.

## Training Summary: <agent-name or "solo (main-session)">

**Source**: /active-learn
**Input**: <agent-name or "solo"> rounds=<N>
**Pipeline**: /active-learn (<rounds-completed> rounds, <total-challenges> challenges)

### Items (N)

1. **<domain/weakness>: <trajectory>** -- <description of how this area changed>
   - initial_severity: HIGH | MEDIUM | LOW
   - final_severity: HIGH | MEDIUM | LOW | RESOLVED
   - rounds_tested: <N>
   - pass_rate: <X/Y>
   - trap_detection: <X/Y>
   - growth_signal: <signal>

2. ...

### Improvement Trajectory

| Round | Challenges | Passed | Traps Caught | Calibration | New Learnings |
|-------|-----------|--------|--------------|-------------|---------------|
| 1 | N | X | Y | pattern | Z |
| 2 | N | X | Y | pattern | Z |
| ... | | | | | |

Overall trend: <improving | stable | declining>
Early stopped: <yes/no, reason>

### Persistent Weaknesses

<Weaknesses that survived all rounds without improvement. These need different intervention -- perhaps restructured ownership, pair-dispatch, or manual coaching.>

### Recommended Next Steps

<Concrete recommendations based on results:>
- If improving: "Schedule next /active-learn in 2 weeks to continue trajectory"
- If stable: "Agent may be at capability ceiling for current learnings format. Consider restructuring ownership or adding pair-dispatch for weak areas."
- If persistent weaknesses: "Weakness X survived N rounds. Consider: manual coaching on X, reducing X from ownership, or pairing with <other-agent> on X tasks."

### Summary

<One paragraph synthesizing the training cycle: how many rounds ran, what improved, what persists, and what the overall trajectory means for the agent's role on the team.>
```

### 6b. Final Capability Snapshot

Ensure capability.yaml is up to date with the final state (written in Phase 4c of the last round).

Report the file path so the user can review:
- Team mode: `memory/agents/<name>/capability.yaml`
- Solo mode: `memory/agents/solo/capability.yaml`

---

## Guidelines

1. **Never leak evaluation criteria to agents.** Hidden Trap and Ground Truth exist solely for post-hoc evaluation. Including them in dispatch prompts invalidates the entire training cycle. This is the single most important constraint.
2. **Serial dispatch, serial evaluation.** Process one challenge at a time within a round. This avoids API throttling and produces clean evaluation flow. Each challenge is a controlled experiment.
3. **Evidence in every rating.** Every evaluation rating (result, trap detection, calibration) must cite specific evidence from the agent's output. Ratings without evidence are noise.
4. **Honest calibration.** Do not inflate pass rates or downplay failures. The training loop's value comes from honest assessment -- a FAIL that leads to a good learning entry is a better outcome than a generous PARTIAL that teaches nothing.
5. **Respect the learnings cap.** The 60-line cap (30 core + 30 task-relevant) exists for a reason. New entries must earn their place. When at capacity, compare value explicitly before displacing.
6. **Graceful degradation.** No git-intel? Use raw git. No prior capability.yaml? Start fresh. No dispatch provenance? Skip Phase 1c. No WebSearch results? Use codebase-only edge cases. Always produce something useful.
7. **Early stop is success.** Reaching the early stop condition means the agent's capability frontier was found and exercised. Running all rounds is not inherently better than stopping at round 2.
8. **Challenges, not tests.** The goal is growth through deliberate practice, not measurement. Err toward slightly above the agent's level. Struggle produces learning.
9. **One weakness per challenge.** Compound challenges dilute the training signal. Each challenge targets exactly one WEAKNESS or GAP.
10. **Cumulative capability tracking.** capability.yaml accumulates across /active-learn runs. Never overwrite history -- append to it. This enables longitudinal trajectory analysis.
11. **Real over hypothetical.** Every challenge must be grounded in real code, real CVEs, real commits, or real community knowledge. Hypothetical scenarios do not transfer to real work.
12. **Self-contained agent prompts.** Dispatched agents cannot read the skill file or access parent context. Everything the agent needs must be in the prompt.

See also: /diagnose-agent (standalone profiling), /challenge-gen (standalone generation), /challenge-run (standalone execution), /sprint (task dispatch with learning loop).
