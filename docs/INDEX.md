# Skill & Agent Navigator

Quick reference for finding the right skill or agent. See also: [Cookbook](primitives-cookbook.md), [Recipes](primitives-recipes.md), [Team Guide](team-system-guide.md).

## Decision Tree

**I want to...**

- **Explore something unknown** -> /blossom (broad, spike-driven) or /fractal (focused, goal-directed)
- **Research a specific topic** -> /gather (collect findings) -> /distill (condense) -> /rank (prioritize)
- **Compare approaches** -> /diff-ideas (two options) or /consensus (three independent proposals)
- **Plan before building** -> /decompose (break down) -> /plan (sequence) -> /spec (full specification)
- **Assess risk** -> /premortem (failure analysis) or /critique (adversarial review)
- **Bootstrap a new project** -> /bootstrap (full setup: infrastructure + agents)
- **Build something** -> /tracer (iterative end-to-end) or /sketch (skeleton only)
- **Review code** -> /review (structured code review)
- **File a bug** -> /bug (structured bug report to beads backlog)
- **Understand a definition's history** -> /evolution (file change history and stability) or /drift (cross-definition convergence/divergence)
- **Manage a team** -> /assemble (create) -> /standup (sync) -> /sprint (dispatch)
- **Understand an agent's capabilities** -> /diagnose-agent (struggle profile from historical evidence)
- **Generate training challenges** -> /challenge-gen (targeted challenges from struggle profile)
- **Run challenges and evaluate** -> /challenge-run (dispatch agent, evaluate performance)
- **Improve agent capabilities** -> /active-learn (full adversarial training loop: diagnose -> challenge -> evaluate -> learn)
- **Run a session** -> /status (orient) -> /advise (recommendations) -> ... work ... -> /retro (reflect) -> /handoff (transition)
- **Discuss with multiple perspectives** -> /meeting (interactive group dialogue)
- **Plan a goal with your team** -> /team-meeting (collaborative planning -> sprint-ready tasks)

## Skills by Category

### Composable Primitives (12)

Stateless skills that follow [pipe format](../rules/pipe-format.md). Output of any primitive feeds the next via conversation context.

| Skill | Purpose | Chain Position |
|-------|---------|----------------|
| /gather | Collect information with sources and confidence | Input |
| /decompose | Break a goal into bounded sub-parts | Transform |
| /distill | Reduce verbose input to essentials | Transform |
| /rank | Score and order items by criteria | Transform |
| /filter | Binary keep/drop by criterion | Transform |
| /assess | Categorize items by rubric (critical/warning/ok) | Transform |
| /verify | Check claims against evidence | Transform |
| /critique | Adversarial review — what's wrong, missing, risky | Transform |
| /diff-ideas | Compare two approaches side-by-side | Transform |
| /merge | Combine multiple pipe-format blocks into one | Transform |
| /plan | Dependency-aware execution sequence | Output |
| /sketch | Minimal code skeleton with TODOs | Output |

### Workflow Skills (21)

Orchestrated multi-step workflows with side effects (file writes, agent dispatch, backlog updates).

| Skill | Purpose | Context |
|-------|---------|---------|
| /blossom | Spike-driven exploration, produces epic + tasks | fork |
| /fractal | Goal-directed recursive exploration | inline |
| /meeting | Multi-agent group discussion | inline |
| /team-meeting | Goal-oriented planning with persistent team | inline |
| /consensus | Three independent proposals + synthesis | fork |
| /premortem | Failure analysis with mitigations | fork |
| /spec | Progressive specification document | fork |
| /bootstrap | Full project setup: infrastructure + agents | fork |
| /tracer | Iterative thin-slice implementation | fork |
| /review | Structured code review (5 dimensions) | fork |
| /bug | File structured bug reports to beads backlog | inline |
| /consolidate | Backlog dedup, stale detection, cleanup | fork |
| /session-health | Context load and drift diagnostic | inline |
| /retro | Session retrospective with persistent learnings | inline |
| /handoff | Session transition capture | inline |
| /evolution | File change history, churn, stability analysis | inline |
| /drift | Cross-definition convergence/divergence detection | inline |
| /diagnose-agent | Agent struggle profile from learnings evolution + git signals | inline |
| /challenge-gen | Generate targeted training challenges from struggle profile | inline |
| /challenge-run | Execute challenges and evaluate agent performance | inline |
| /active-learn | Full adversarial training loop: diagnose, challenge, evaluate, learn | fork |

### Team Skills (3)

Manage persistent learning teams across sessions.

| Skill | Purpose | Context |
|-------|---------|---------|
| /assemble | Create team with roles and ownership | inline |
| /standup | Sync status, surface blockers | inline |
| /sprint | Dispatch work with learning loop | inline |

### Session Skills (2)

| Skill | Purpose | Context |
|-------|---------|---------|
| /status | Unified dashboard: backlog, activity, team, last session | inline |
| /advise | Proactive recommendations by composing git state, session history, backlog, and signals. Degrades gracefully — works with just git, richer with each layer present. | inline |

## Skills by Context Type

**Inline (29):** bug, gather, distill, rank, filter, assess, verify, sketch, merge, decompose, critique, plan, diff-ideas, fractal, meeting, team-meeting, session-health, retro, handoff, assemble, standup, sprint, status, advise, evolution, drift, diagnose-agent, challenge-gen, challenge-run

**Fork (9):** active-learn, blossom, bootstrap, consensus, consolidate, premortem, review, spec, tracer

Fork skills run in an isolated context to avoid polluting the main conversation. Use fork skills for heavy exploration; use inline skills for quick operations within the current flow.

## Primitive Chain Patterns

Common composition sequences (each step's output feeds the next via context):

```
/gather -> /distill -> /rank          # Research -> condense -> prioritize
/gather -> /filter -> /verify         # Research -> narrow -> fact-check
/decompose -> /rank -> /plan          # Break down -> prioritize -> sequence
/gather -> /critique -> /rank         # Research -> stress-test -> prioritize
/gather -> /diff-ideas                # Research -> compare two approaches
/gather -> /distill -> /sketch        # Research -> condense -> prototype
/critique -> /bug                     # Find flaws -> file as tracked issues
/review -> /bug                       # Code review -> file bugs from findings
/drift -> /rank -> /sprint            # Compare definitions -> prioritize fixes -> execute
/diagnose-agent -> /challenge-gen     # Profile agent -> generate targeted challenges
/diagnose-agent -> /challenge-gen -> /challenge-run  # Profile -> challenges -> evaluate
/active-learn                                       # Full loop: diagnose -> challenge -> evaluate -> learn (all inline)
```

**Note**: Multi-step chains (3+ primitives) are best run in the main session. If dispatching to a subagent, set `max_turns: 40` to avoid turn limits.

### File-Based Intermediate Results

For chains that may span compression events, save intermediate output to a file and pass the file path to the next primitive:

```bash
# Step 1: Run first primitive and save output
User: /gather authentication patterns in this codebase
  -> produces pipe-format output
User: Save the gather output above to /tmp/auth-findings.md

# Step 2: After compression or in a new session
User: /distill /tmp/auth-findings.md
  -> reads file instead of scanning context
User: /rank by security risk
  -> continues chain from distill output
```

This pattern is especially useful for:
- Long-running sessions where compression may occur between primitives
- Preserving research findings across session boundaries
- Sharing primitive output with subagents (pass file path in task prompt)

## Tools

| Tool | Purpose |
|------|---------|
| [git-intel](../tools/git-intel/README.md) | Git history analyzer outputting JSON for hooks, skills, and scripts. 7 subcommands: metrics, churn, lifecycle, patterns, hotspots, authors, trends. |
| [learnings-coverage](../tools/learnings-coverage/) | Semantic coverage analysis: what % of an agent's commit-message space is captured in their learnings? Embeds with sentence-transformers, reports gaps and well-covered areas. |
| [difficulty-calibration](../tools/difficulty-calibration/) | Track challenge outcomes and fit logistic regression to predict optimal difficulty level targeting ~70% success rate. 3 subcommands: record, recommend, report. |
| [knowledge-transfer](../tools/knowledge-transfer/) | Detect cross-agent learning transfer opportunities via sentence embeddings. Finds learnings from agent A that are relevant to agent B's scope but absent from B's knowledge. |

## Agents

| Agent | Purpose | Scope |
|-------|---------|-------|
| agent-generator | Generate project-specific agents | Global |
| project-bootstrapper | Bootstrap projects with Claude Code setup | Global |
| code-reviewer | Read-only code review | Global |

8 additional project-local agents live in `.claude/agents/` for authoring, research, and maintenance tasks specific to this repo.

## Further Reading

- [Primitives Cookbook](primitives-cookbook.md) — detailed usage patterns and examples
- [Primitives Recipes](primitives-recipes.md) — end-to-end workflow recipes
- [Team System Guide](team-system-guide.md) — team lifecycle, learnings, and coordination
