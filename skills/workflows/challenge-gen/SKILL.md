---
name: challenge-gen
description: "Generate targeted challenges for agent improvement from a /diagnose-agent struggle profile. Produces domain edge-case and commit-replay challenges calibrated to the agent's weak areas. Use after /diagnose-agent to create training material. Keywords: challenge, training, improvement, active-learning, edge-case, replay, agent, weakness."
argument-hint: "<agent-name>"
disable-model-invocation: false
user-invocable: true
allowed-tools: [Read, Grep, Glob, Write, WebSearch, WebFetch, Task, "Bash(git:*)", "Bash(wc:*)", "Bash(mkdir:*)"]
context: inline
---

# Challenge Gen: Targeted Agent Training Challenges

You are running **challenge-gen** — generating targeted challenges to exercise an agent's weak areas. Target agent: **$ARGUMENTS**

## When to Use

- After /diagnose-agent produces a struggle profile — generate challenges that target identified weaknesses
- When an agent repeatedly struggles with specific task types and needs deliberate practice
- Before /active-learn to create a curated challenge set for a training cycle
- When onboarding an agent to a new domain and want calibrated difficulty ramps

## How It Works

**Serial (default):**
```
Detect upstream /diagnose-agent output (or accept $ARGUMENTS)
  -> Validate agent exists, extract ownership areas
    -> Strategy A: Research domain edge cases (WebSearch + codebase)
      -> Strategy B: Find commit-replay candidates (raw git)
        -> Assemble 3-5 challenges with quality gate
          -> Write challenges to memory/agents/<name>/challenges/
            -> Emit pipe format output
```

**Parallel mode** (pass `parallel` in `$ARGUMENTS`):
```
Detect upstream /diagnose-agent output (or accept $ARGUMENTS)
  -> Validate agent exists, extract ownership areas
    -> [concurrently]
         Agent A: Research domain edge cases (WebSearch + codebase)
         Agent B: Find commit-replay candidates (raw git)
    -> Collect results from both agents
      -> Assemble 3-5 challenges with quality gate
        -> Write challenges to memory/agents/<name>/challenges/
          -> Emit pipe format output
```

---

## Phase 0: Detect Input and Validate

### 0a. Check for Upstream /diagnose-agent Output

Search conversation context for the pipe-format pattern:

```
## Struggle Profile: <agent-name>

**Source**: /diagnose-agent
```

**If found:**
1. Read the upstream output. State: "Reading N items from /diagnose-agent output above."
2. Extract: agent name, WEAKNESS items, GAP items, STRENGTH items, difficulty calibration
3. Record the upstream `**Pipeline**` field for provenance

**If not found:**
1. If `$ARGUMENTS` provides an agent name, proceed with basic profiling (Phase 0b)
2. If `$ARGUMENTS` is empty, list available agents from `.claude/team.yaml` and ask the user which to target. Stop and wait.
3. Warn: "No /diagnose-agent output detected. Running with basic profiling — for richer challenges, run `/diagnose-agent <name>` first."

### 0b. Validate Agent and Extract Ownership

1. Read `.claude/team.yaml` and confirm the agent name exists in the `members` list
2. Extract the agent's `owns` patterns (file globs defining ownership areas)
3. Read `memory/agents/<name>/learnings.md` if it exists — scan for Gotchas and known weaknesses
4. If no upstream struggle profile exists, construct a minimal one from learnings Gotchas + sparse categories

---

## Parallel Dispatch Mode

In parallel mode (`$ARGUMENTS` contains `parallel`), run Phase 1 and Phase 2 as concurrent background Task agents after Phase 0 completes. This roughly halves wall-clock time because Strategy A (WebSearch + codebase) and Strategy B (git history) are fully independent.

**When to use parallel mode:** When the agent's owned area is broad (many files, many weaknesses) and both strategies are expected to yield candidates. For narrow profiles or small codebases, serial mode is cheaper and produces equivalent results.

**Assessment before dispatch:** After Phase 0 validates the agent and extracts ownership, confirm that both strategies have meaningful scope before launching both agents:
- Strategy A has scope if: ≥1 WEAKNESS or GAP item exists and the owned file globs match files in the repo
- Strategy B has scope if: the agent has owned files with recent commits (`git log --oneline --since="60 days ago" -- <owns-patterns>` returns ≥1 result)

If only one strategy has scope, fall back to serial for the active strategy only.

### Dispatch (parallel mode)

After Phase 0 completes, launch both agents concurrently:

**Strategy A agent:**

```
Task({
  subagent_type: "general-purpose",
  run_in_background: true,
  prompt: "<Strategy A agent prompt — see template below>"
})
```

**Strategy B agent:**

```
Task({
  subagent_type: "general-purpose",
  run_in_background: true,
  prompt: "<Strategy B agent prompt — see template below>"
})
```

Collect results from both agents before proceeding to Phase 3.

### Agent Prompt Templates

#### Strategy A: Domain Edge Cases Agent Prompt

```
You are running Strategy A of challenge-gen for agent: <agent-name>.

## Context

Agent owns: <owns-patterns>
Struggle profile source: <upstream source or "basic profiling">

Weaknesses to target:
<list each WEAKNESS item from struggle profile>

Gaps to target:
<list each GAP item from struggle profile>

Difficulty calibration: <novice | intermediate | expert | adversarial>

## Your Task

Research domain edge cases for the weaknesses and gaps listed above. You are Strategy A — do NOT use git history (that is Strategy B's job). Use WebSearch and codebase reading only.

### Step 1: Codebase Edge Cases

Search the agent's owned files for error-prone patterns:

- Files with high cyclomatic complexity (many conditionals, nested logic)
- Error handling paths that are rarely exercised
- Configuration edge cases (empty inputs, boundary values, unusual combinations)
- Cross-cutting concerns the agent's learnings don't mention

Use Grep and Glob to find files matching these patterns: <owns-patterns>

### Step 2: External Edge Cases

Use WebSearch to find real-world edge cases related to each weakness:
- Search for CVEs, security advisories, or known bugs in the technology stack
- Search Stack Overflow for highly-voted edge case questions in the domain
- Search GitHub issues for "unexpected behavior" or "edge case" in related projects
- Search for framework-specific gotchas the agent may not have encountered

For each external finding, verify it applies to the codebase by checking:
- Is the relevant technology/library actually used? (check package files, imports)
- Is the vulnerable pattern present in owned files?
- Has it already been addressed? (check git history for related fixes)

### Step 3: Select Best Edge Cases

From all candidates, select 2-3 that:
- Target a specific WEAKNESS or GAP from the profile above
- Have clear acceptance criteria (the "right answer" is verifiable)
- Contain a non-obvious trap (something that makes the naive solution wrong)
- Are grounded in real scenarios (not hypothetical)

## Output Format

Report your findings as a structured list. For each candidate:

**Edge Case <N>:** <title>
- targeted_weakness: <WEAKNESS or GAP name from profile>
- scenario: <what the agent is asked to do>
- trap: <the non-obvious thing that makes the naive solution wrong>
- acceptance_criteria: <how to evaluate pass/fail>
- grounding: <URL, file path, or CVE that makes this real>

If no candidates meet the quality bar, state: "No qualifying edge cases found — Strategy A produced 0 candidates."
```

#### Strategy B: Commit-Replay Agent Prompt

```
You are running Strategy B of challenge-gen for agent: <agent-name>.

## Context

Agent owns: <owns-patterns>
Struggle profile source: <upstream source or "basic profiling">

Weaknesses to target:
<list each WEAKNESS item from struggle profile>

Gaps to target:
<list each GAP item from struggle profile>

Difficulty calibration: <novice | intermediate | expert | adversarial>

## Your Task

Find commit-replay challenge candidates in the agent's ownership area. You are Strategy B — use git history only. Do NOT use WebSearch (that is Strategy A's job).

### Step 1: Find Candidate Commits

Run these git commands against the owned file patterns:

```bash
# Recent fix commits in owned files
git log --oneline --since="60 days ago" -- <owns-patterns> | head -30

# Find fix: commits that follow feat: commits on same files
git log --format="%H %s" --since="60 days ago" -- <owns-patterns>
```

Scan for `fix:` commits. For each, check if a `feat:` commit on the same files preceded it within 7 days.

Prioritize:
- `fix_after_feat` commits (the fix is the challenge — can the agent get it right the first time?)
- Commits touching frequently-changed files (areas where correctness is repeatedly hard)
- Commits with detailed messages (better context for the challenge prompt)

### Step 2: Extract Challenge Material

For each candidate commit:

```bash
git show --format="%B" --no-patch <commit-hash>   # commit message -> challenge prompt
git show --format="" <commit-hash>                  # diff -> ground truth
git show <commit-hash>^:<file-path>                 # file before commit -> starting point
git rev-parse <commit-hash>^                        # parent hash -> base_commit
```

Record the parent hash as `base_commit` — the state before the fix. This is what `/challenge-run` uses for worktree isolation.

### Step 3: Select Best Replay Candidates

From all candidates, select 1-2 that:
- Target a specific WEAKNESS or GAP from the profile above
- Have a self-contained diff (not sprawling across 10+ files)
- Have enough context in the commit message to understand intent
- The "before" state is a plausible starting point (not mid-refactor)

## Output Format

Report your findings as a structured list. For each candidate:

**Commit Replay <N>:** <title>
- targeted_weakness: <WEAKNESS or GAP name from profile>
- base_commit: <parent hash from git rev-parse>
- commit_hash: <the fix commit hash>
- scenario: <commit message or paraphrase — what the agent is asked to do>
- trap: <what makes the naive approach get this wrong>
- acceptance_criteria: <how to evaluate — match the actual diff>
- files_changed: <count and list of files in the diff>

If no candidates meet the quality bar, state: "No qualifying commit-replay candidates found — Strategy B produced 0 candidates."
```

---

## Phase 1: Research Domain Edge Cases — Strategy A (default)

For each WEAKNESS and GAP item from the struggle profile, research real-world edge cases in that domain.

### 1a. Codebase Edge Cases

Search the agent's owned files for patterns that are commonly error-prone:

```bash
# Find complex or unusual patterns in owned files
git log --oneline --all --since="60 days ago" -- <owns-patterns>
```

Look for:
- Files with high cyclomatic complexity (many conditionals, nested logic)
- Error handling paths that are rarely exercised
- Configuration edge cases (empty inputs, boundary values, unusual combinations)
- Cross-cutting concerns the agent's learnings don't mention

### 1b. External Edge Cases

Use WebSearch to find real-world edge cases related to each weakness:

- Search for CVEs, security advisories, or known bugs in the technology stack
- Search Stack Overflow for highly-voted edge case questions in the domain
- Search GitHub issues for "unexpected behavior" or "edge case" in related projects
- Search for framework-specific gotchas the agent may not have encountered

For each external finding, verify it applies to the codebase by checking:
- Is the relevant technology/library actually used? (check package files, imports)
- Is the vulnerable pattern present in owned files?
- Has it already been addressed? (check git history for related fixes)

### 1c. Select Best Edge Cases

From all candidates, select 2-3 that:
- Target a specific WEAKNESS or GAP from the struggle profile
- Have clear acceptance criteria (the "right answer" is verifiable)
- Contain a non-obvious trap (something that makes the naive solution wrong)
- Are grounded in real scenarios (not hypothetical)

---

## Phase 2: Find Commit-Replay Candidates — Strategy B (default)

Identify real commits in the agent's ownership area that can be turned into replay challenges.

### 2a. Find Candidate Commits

```bash
# Recent fix commits in owned files
git log --oneline --since="60 days ago" -- <owns-patterns> | head -30

# Find fix: commits that follow feat: commits on same files
git log --format="%H %s" --since="60 days ago" -- <owns-patterns>
```

Scan for `fix:` commits. For each, check if a `feat:` commit on the same files preceded it within 7 days.

Prioritize:
- `fix_after_feat` commits (the fix is the challenge — can the agent get it right the first time?)
- Commits touching frequently-changed files (areas where correctness is repeatedly hard)
- Commits with detailed messages (better context for the challenge prompt)

Filter results to files matching the agent's `owns` patterns.

### 2b. Extract Challenge Material

For each candidate commit:

```bash
# Get the commit message (this becomes the challenge prompt)
git show --format="%B" --no-patch <commit-hash>

# Get the diff (this is the ground truth, hidden from the agent)
git show --format="" <commit-hash>

# Get the file state BEFORE the commit (this is the starting point)
git show <commit-hash>^:<file-path>

# Get the parent commit hash (this is the base_commit for worktree isolation)
git rev-parse <commit-hash>^
```

Record the parent hash as the `base_commit` — the state of the repo before the fix was applied. This is the commit that `/challenge-run` will use to create a worktree at the pre-fix state.

### 2c. Select Best Replay Candidates

From all candidates, select 1-2 that:
- Target a specific WEAKNESS or GAP from the struggle profile
- Have a self-contained diff (not sprawling across 10+ files)
- Have enough context in the commit message to understand intent
- The "before" state is a plausible starting point (not mid-refactor)

---

## Phase 3: Assemble Challenges

Combine edge-case and commit-replay candidates into a final set of 3-5 challenges.

In parallel mode, wait for both background agents to complete, then collect their structured output before proceeding. Treat agent output sections ("Edge Case N:" and "Commit Replay N:") as the candidate pool that feeds Phase 3a and 3b below.

### 3a. Balance and Calibrate

- Mix strategies: at least 1 edge-case and 1 commit-replay (if candidates exist for both)
- Calibrate difficulty using the upstream Difficulty Calibration level:
  - **novice**: challenges should have clear signals and a single trap
  - **intermediate**: challenges should require connecting 2-3 concepts
  - **expert**: challenges should involve cross-cutting concerns or subtle interactions
  - **adversarial**: challenges should have multiple valid-looking approaches where only one is correct
- Each challenge must target a different WEAKNESS or GAP (no doubling up)

### 3b. Quality Gate

Every challenge MUST pass these checks before inclusion:
1. **Targets a specific weakness** — maps to a named WEAKNESS or GAP from the struggle profile
2. **Has clear acceptance criteria** — can evaluate agent output as pass/fail
3. **Has a hidden trap** — something non-obvious that the naive approach gets wrong
4. **Is grounded** — based on real code, real CVEs, real commits, or real Stack Overflow patterns (not hypothetical)
5. **Is self-contained** — the agent can attempt it without needing the full project context

Drop any challenge that fails a check. If fewer than 3 survive, return to Phase 1 or 2 for more candidates.

---

## Phase 4: Write and Emit

### 4a. Write Challenge File

```bash
mkdir -p memory/agents/<name>/challenges
```

Write challenges to `memory/agents/<name>/challenges/<timestamp>-challenges.md` using this structure:

```markdown
# Challenges for <agent-name>

Generated: <date>
Source profile: /diagnose-agent (or "basic profiling")
Difficulty level: <novice | intermediate | expert | adversarial>

## Challenge 1: <title>

**Strategy**: edge-case | commit-replay
**Base commit**: <parent-hash> *(commit-replay only)*
**Targeted weakness**: <WEAKNESS or GAP name from profile>
**Difficulty**: <level>

### Scenario

<What the agent is asked to do — this is what the agent sees>

### Acceptance Criteria

<How to evaluate the agent's output — pass/fail conditions>

### Hidden Trap

<The specific thing that makes this hard — NOT shown to the agent>

### Ground Truth

<The correct approach or actual diff — for evaluation only>

---

## Challenge 2: ...
```

### 4b. Emit Pipe Format Output

Output the challenge set in pipe format for downstream consumption:

```markdown
## Challenge Set: <agent-name>

**Source**: /challenge-gen
**Input**: <agent-name>
**Pipeline**: <upstream pipeline> -> /challenge-gen (N challenges)

### Items (N)

1. **<challenge-title>** — <one-line description>
   - scenario: <brief scenario summary>
   - targeted_weakness: <WEAKNESS or GAP name>
   - difficulty: novice | intermediate | expert | adversarial
   - acceptance_criteria: <how to evaluate>
   - hidden_trap: <what makes it hard>
   - strategy: edge-case | commit-replay
   - base_commit: <hash> *(commit-replay only)*

2. ...

### Difficulty Calibration

Agent coverage: <percentage from upstream or estimated>
Challenge level: <novice | intermediate | expert | adversarial>
Strategy mix: <N edge-case, M commit-replay>

### Summary

<One paragraph synthesizing the challenge set: what weaknesses are targeted, what strategies were used, what difficulty level, and what growth the challenges are designed to produce.>
```

---

## Guidelines

1. **Challenges, not tests.** Challenges are designed to produce growth through deliberate practice, not to measure current ability. Err toward slightly above the agent's level — struggle is the point.
2. **Real over hypothetical.** Every challenge must be grounded in real code, real CVEs, real commits, or real community knowledge. Hypothetical scenarios do not transfer to real work.
3. **One weakness per challenge.** Each challenge targets exactly one WEAKNESS or GAP. Compound challenges dilute the training signal.
4. **Hidden traps are essential.** A challenge without a non-obvious trap is just a task. The trap is what forces the agent to develop new patterns rather than applying existing ones.
5. **Graceful degradation.** No upstream profile? Do basic profiling. No web search results? Use codebase-only edge cases. Always produce something useful.
6. **Commit-replay integrity.** When extracting commit material, never modify the ground truth diff. The whole point is that the actual change is the answer. The `base_commit` must be a real commit hash from git history (output of `git rev-parse <commit>^`), never fabricated or modified — `/challenge-run` depends on it for worktree isolation.
7. **Do not show hidden traps to the agent.** The challenge file separates what the agent sees (scenario + acceptance criteria) from what the evaluator knows (hidden trap + ground truth). This separation is critical.
8. **Cap at 5 challenges.** More than 5 creates evaluation overhead without proportional learning benefit. If many weaknesses exist, prioritize by severity.
9. **Write provenance.** Record which profile items each challenge targets so /active-learn can track which weaknesses have been exercised and which remain untouched.

See also: /challenge-run (execute the challenges produced here against the agent — uses `base_commit` for worktree isolation in commit-replay challenges); /diagnose-agent (generate the struggle profile that feeds this skill); /active-learn (run a full training cycle using challenge sets).
