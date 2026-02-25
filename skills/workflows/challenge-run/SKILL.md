---
name: challenge-run
description: "Dispatch an agent on targeted challenges and evaluate performance across result, trap detection, and confidence calibration dimensions. Use after /challenge-gen to execute the training loop. Keywords: challenge, run, evaluate, active-learning, training, agent, performance, calibration."
argument-hint: "<agent-name>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Write, Grep, Glob, Task, "Bash(git:*)", "Bash(mkdir:*)", "Bash(wc:*)"
context: inline
---

# Challenge Run: Execute and Evaluate Agent Challenges

You are running **challenge-run** -- dispatching an agent on targeted challenges and evaluating performance. Target agent: **$ARGUMENTS**

## When to Use

- After /challenge-gen produces a challenge set -- execute the challenges and measure agent performance
- When testing whether an agent has improved on previously identified weaknesses
- Before /active-learn to produce evaluation data for the learning loop
- When you want objective evidence of agent capability rather than speculation

## How It Works

```
Detect upstream /challenge-gen output or load challenge file
  -> Read agent learnings + team manifest
    -> For each challenge, dispatch agent with scenario + criteria (NOT trap/truth)
      -> Evaluate agent output against hidden trap + ground truth
        -> Score: result, trap detection, confidence calibration
          -> Write evaluation results to memory/agents/<name>/challenges/
            -> Emit pipe format evaluation report
```

---

## Phase 0: Detect Input and Load Context

### 0a. Check for Upstream /challenge-gen Output

Search conversation context for the pipe-format pattern:

```
## Challenge Set: <agent-name>

**Source**: /challenge-gen
```

**If found:**
1. Read the upstream output. State: "Reading N items from /challenge-gen output above."
2. Extract: agent name, challenge items with scenarios, targeted weaknesses, difficulty levels
3. Record the upstream `**Pipeline**` field for provenance

**If not found:**
1. If `$ARGUMENTS` provides an agent name, look for challenge files (Phase 0b)
2. If `$ARGUMENTS` is empty, list available agents from `.claude/team.yaml` and ask the user which to target. Stop and wait.

### 0b. Load Challenge File from Disk

If no upstream pipe-format output was detected, search for the most recent challenge file:

```bash
# Find challenge files for this agent
git ls-files -- "memory/agents/$ARGUMENTS/challenges/*-challenges.md"
```

Also check untracked files with Glob: `memory/agents/<name>/challenges/*-challenges.md`

Select the most recent file (by timestamp in filename). Read it and extract all challenges with their full details: Scenario, Acceptance Criteria, Hidden Trap, Ground Truth.

If no challenge file exists, tell the user to run `/challenge-gen <agent-name>` first and stop.

### 0c. Read Agent Context

1. Read `.claude/team.yaml` and confirm the agent name exists in the `members` list
2. Extract the agent's role, model, and `owns` patterns
3. Read `memory/agents/<name>/learnings.md` if it exists

### 0d. Prerequisite Gate

Do not proceed until:
- [ ] Challenge set is loaded (from context or file) with at least 1 challenge
- [ ] Agent exists in team.yaml with role and model known
- [ ] Agent learnings read (or confirmed absent for new agents)

---

## Phase 1: Dispatch Challenges

Process challenges **serially** -- each dispatch is independent but serial execution avoids API throttling and lets you build a complete evaluation picture.

### 1a. Compose the Challenge Prompt

For each challenge, build a self-contained prompt for the target agent. The prompt MUST include:

1. **Agent identity**: Name, role, ownership areas (from team.yaml)
2. **Agent learnings**: Full contents of their learnings.md
3. **Challenge scenario**: The scenario section from the challenge definition
4. **Acceptance criteria**: The criteria section from the challenge definition
5. **Reflection protocol**: Ask the agent to self-assess at the end

**CRITICAL: Never include the Hidden Trap or Ground Truth in the agent prompt.** These are for evaluation only. Including them defeats the purpose of the challenge.

The prompt structure:

> You are **<agent-name>**, a <role> on the team.
>
> ## Your Learnings
>
> <contents of learnings.md>
>
> ## Task
>
> <scenario from challenge definition>
>
> ## Acceptance Criteria
>
> <acceptance criteria from challenge definition>
>
> ## Reflection
>
> After completing the task, provide:
> 1. **Approach**: What approach did you take and why?
> 2. **Confidence**: How confident are you in your solution? (LOW / MEDIUM / HIGH)
> 3. **Uncertainty**: What aspects are you least sure about?
> 4. **Alternative approaches**: What other approaches did you consider and why did you reject them?
> 5. **Risk areas**: Where might this solution break or be wrong?

### 1b. Dispatch

```
Task({
  subagent_type: "general-purpose",
  model: "<agent's model from team.yaml>",
  prompt: "<composed prompt from 1a>"
})
```

Use serial dispatch (no `run_in_background`). Each challenge is independent but evaluation benefits from sequential processing.

### 1c. Capture Output

Store the agent's full response for evaluation in Phase 2. Extract:
- The agent's solution/implementation
- Their self-reported confidence level
- Their identified risk areas and uncertainties
- Their reasoning about approach selection

---

## Phase 2: Evaluate Each Challenge

After each dispatch returns, evaluate the agent's output against the hidden evaluation criteria.

### 2a. Result Assessment (PASS / PARTIAL / FAIL)

Check the agent's output against the acceptance criteria:

**PASS**: All acceptance criteria are met. The solution is correct and complete.

**PARTIAL**: Some acceptance criteria are met but not all. The solution works for the main case but misses edge cases or has minor issues.

**FAIL**: The solution does not meet the primary acceptance criteria. Fundamental misunderstanding or incorrect approach.

**Objective checks (prefer these over subjective assessment):**
- For commit-replay challenges: Compare the agent's changes against the ground truth diff. Do they address the same root cause? Do they touch the same files? Is the fix semantically equivalent even if not identical?
- For edge-case challenges: Does the agent's solution handle the specific edge case described in the hidden trap?
- If the challenge involves code: Does the approach match the ground truth direction?

### 2b. Trap Detection (CAUGHT / PARTIAL / MISSED)

Evaluate whether the agent recognized the non-obvious difficulty:

**CAUGHT**: The agent explicitly identified the trap or the core difficulty. Their solution handles it correctly. Evidence: they mentioned it in their reasoning, risk areas, or uncertainties.

**PARTIAL**: The agent showed awareness of something being tricky in the right area but did not fully articulate the trap or only partially handled it. Evidence: their uncertainty or risk areas point in the right direction.

**MISSED**: The agent showed no awareness of the trap. Their solution fails on the trap scenario. Evidence: no mention in reasoning, high confidence despite the trap, solution breaks on the trap case.

### 2c. Confidence Calibration (OVER / UNDER / WELL_CALIBRATED)

Compare the agent's self-reported confidence against their actual performance:

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

Nuance: If the agent reported HIGH confidence but also correctly identified the trap in their risk areas, lean toward WELL_CALIBRATED -- they knew the risk even if their headline confidence was high.

### 2d. Reasoning Quality Assessment

Briefly assess the quality of the agent's reasoning process:
- Did they explore multiple approaches before committing?
- Did they identify relevant constraints and trade-offs?
- Was their rejection of alternative approaches well-reasoned?
- Did they demonstrate domain knowledge appropriate to their role?

### 2e. Growth Signal

Synthesize what this result tells us about the agent's capability relative to the targeted weakness:
- **Strength emerging**: The agent is developing competence in this area (PASS + CAUGHT)
- **Awareness growing**: The agent recognizes the difficulty but cannot yet solve it (PARTIAL + PARTIAL/CAUGHT)
- **Blind spot persists**: The agent does not recognize the difficulty (any + MISSED)
- **Overcorrection risk**: The agent is cautious in this area but their caution is misplaced (PASS + CAUGHT + UNDER)

---

## Phase 3: Persist Results

### 3a. Write Evaluation File

```bash
mkdir -p memory/agents/<name>/challenges
```

Write evaluation results to `memory/agents/<name>/challenges/<timestamp>-evaluation.md`:

```markdown
# Challenge Evaluation: <agent-name>

Evaluated: <date>
Challenge source: <challenge file path or "pipe-format context">
Pipeline: <upstream pipeline> -> /challenge-run (N evaluated)

## Challenge 1: <title>

**Result**: PASS | PARTIAL | FAIL
**Trap Detection**: CAUGHT | PARTIAL | MISSED
**Confidence Calibration**: OVER | UNDER | WELL_CALIBRATED
**Targeted Weakness**: <weakness name>
**Difficulty**: <level>

### Agent's Approach

<Brief summary of what the agent did>

### Evaluation Notes

<Why this result/trap/calibration rating was assigned. Cite specific evidence from the agent's output.>

### Growth Signal

<What this tells us about the agent's trajectory on this weakness>

---

## Challenge 2: ...

---

## Overall Assessment

- Results: X/N passed, Y partial, Z failed
- Trap detection: X caught, Y partial, Z missed
- Calibration: X well-calibrated, Y over-confident, Z under-confident
- Key insight: <one sentence about what these results reveal>
```

---

## Phase 4: Emit Pipe Format Output

Output the evaluation report in pipe format for downstream consumption by /active-learn:

```markdown
## Challenge Results: <agent-name>

**Source**: /challenge-run
**Input**: <agent-name>
**Pipeline**: <upstream pipeline> -> /challenge-run (N evaluated)

### Items (N)

1. **<challenge-title>: <PASS|PARTIAL|FAIL>** — <one-line summary of agent performance>
   - targeted_weakness: <from challenge>
   - result: PASS | PARTIAL | FAIL
   - trap_detection: CAUGHT | PARTIAL | MISSED
   - confidence_calibration: OVER | UNDER | WELL_CALIBRATED
   - reasoning_quality: <brief assessment>
   - growth_signal: <what this result reveals>

2. ...

### Evaluation Summary

| Challenge | Result | Trap | Confidence | Growth Signal |
|-----------|--------|------|------------|---------------|
| [title]   | [P/F]  | [C/P/M] | [O/U/W] | [signal] |

Overall: [X/N passed], [trap detection rate], [calibration assessment]

### Summary

<One paragraph synthesizing what these results reveal about the agent's growth trajectory, which weaknesses are improving, which persist, and what the next training cycle should focus on.>
```

---

## Guidelines

1. **Never leak evaluation criteria to the agent.** The Hidden Trap and Ground Truth sections exist solely for post-hoc evaluation. Including them in the dispatch prompt invalidates the challenge. This is the single most important constraint.
2. **Objective over subjective.** Prefer concrete checks (diff comparison, edge case handling, code correctness) over subjective quality assessments. When objective checks are available, use them first and let subjective assessment fill gaps.
3. **Serial dispatch.** Process one challenge at a time. This avoids API throttling and produces cleaner evaluation flow. Unlike blossom spikes, challenges are not independent explorations -- they are controlled experiments.
4. **Calibration matters as much as correctness.** An agent that fails but knows it will fail (LOW confidence + FAIL) is better calibrated than one that fails confidently (HIGH confidence + FAIL). Calibration predicts future reliability.
5. **Growth signals over grades.** The goal is not to score the agent but to understand their trajectory. "Blind spot persists" is more actionable than "FAIL." The evaluation feeds /active-learn, not a report card.
6. **Graceful degradation.** No upstream pipe format? Read from challenge files. No ground truth diff? Use acceptance criteria only. No learnings file? Dispatch without learnings. Always produce something useful.
7. **Evidence in evaluation notes.** Every rating must cite specific evidence from the agent's output. "MISSED" is not useful without "the agent did not mention X despite it being the core difficulty."
8. **Commit-replay fidelity.** For commit-replay challenges, compare the agent's approach direction against the ground truth diff. Semantic equivalence counts -- the agent does not need to produce an identical diff, just address the same root cause with a sound approach.
9. **Self-contained agent prompts.** The dispatched agent cannot read the skill file or access parent context. Everything the agent needs (identity, learnings, scenario, criteria, reflection protocol) must be in the prompt.
10. **Results persist for longitudinal tracking.** Evaluation files accumulate in the challenges directory so /active-learn can track improvement over multiple cycles.

See also: /challenge-gen (upstream producer), /diagnose-agent (struggle profiling), /active-learn (downstream consumer).
