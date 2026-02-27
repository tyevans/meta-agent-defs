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
- **Deploy something** -> /deploy (readiness gate, strategy selection, execution, monitoring, rollback)
- **Optimize something** -> /optimize (profile -> bottleneck -> fix -> measure -> iterate)
- **Test an implementation** -> /test-strategy (classify spec type, write tests, enforce red-green gates)
- **Review code** -> /review (structured code review)
- **File a bug** -> /bug (structured bug report to backlog)
- **Understand a definition's history** -> /evolution (file change history and stability) or /drift (cross-definition convergence/divergence)
- **Manage a team** -> /assemble (create) -> /standup (sync) -> /sprint (dispatch)
- **Understand an agent's capabilities** -> /diagnose-agent (struggle profile from historical evidence)
- **Generate training challenges** -> /challenge-gen (targeted challenges from struggle profile)
- **Run challenges and evaluate** -> /challenge-run (dispatch agent, evaluate performance)
- **Improve agent capabilities** -> /active-learn (full adversarial training loop: diagnose -> challenge -> evaluate -> learn)
- **Optimize agent learnings for upcoming work** -> /curate (score, prune, fill gaps) -> /promote (graduate to rules) — or use /tend to run all three
- **Audit project rules for health** -> /curate rules (score rules, passive context budget, gap detection)
- **Run a session** -> /status (orient) -> /advise (recommendations) -> ... work ... -> /retro (reflect) -> /handoff (transition)
- **Discuss with multiple perspectives** -> /meeting (interactive group dialogue)
- **Plan a goal with your team** -> /team-meeting (collaborative planning -> sprint-ready tasks)
- **Just do something** -> /do (match goal to skill/pipeline and execute it)
- **Find the right skill for a goal** -> /discover (semantic recommendation + pipeline suggestion)

## Skills by Category

Skills are organized into three layers in the repo (`skills/core/`, `skills/workflows/`, `skills/teams/`) and flattened on install.

### Core (16) — `skills/core/`

Composable primitives following [pipe format](../rules/pipe-format.md), plus the routing entrypoints. Stateless — output of any primitive feeds the next via conversation context.

| Skill | Purpose | Chain Position |
|-------|---------|----------------|
| /gather | Collect information with sources and confidence | Input |
| /decompose | Break a goal into bounded sub-parts | Transform |
| /distill | Reduce verbose input to essentials | Transform |
| /expand | Elaborate sparse items into detailed descriptions (inverse of distill) | Transform |
| /transform | Apply a rewrite instruction to every item independently (map) | Transform |
| /rank | Score and order items by criteria | Transform |
| /filter | Binary keep/drop by criterion | Transform |
| /assess | Categorize items by rubric (critical/warning/ok) | Transform |
| /verify | Check claims against evidence | Transform |
| /critique | Adversarial review — what's wrong, missing, risky | Transform |
| /diff-ideas | Compare two approaches side-by-side | Transform |
| /merge | Combine multiple pipe-format blocks into one | Transform |
| /plan | Dependency-aware execution sequence | Output |
| /sketch | Structural skeleton with TODOs (code, docs, configs, schemas) | Output |
| /do | Primary entrypoint: match goal to skill/pipeline and execute it | Router |
| /discover | Recommend skills or pipelines for a described goal | Router |

### Workflows (24) — `skills/workflows/`

Orchestrated multi-step workflows with side effects (file writes, agent dispatch, backlog updates).

| Skill | Purpose | Context |
|-------|---------|---------|
| /blossom | Spike-driven exploration, produces epic + tasks | fork |
| /fractal | Goal-directed recursive exploration | inline |
| /meeting | Multi-agent group discussion (uses native teams for real-time dialogue) | inline |
| /consensus | Three independent proposals + synthesis | fork |
| /premortem | Failure analysis with mitigations | fork |
| /spec | Progressive specification document | fork |
| /bootstrap | Full project setup: infrastructure + agents | fork |
| /tracer | Iterative thin-slice implementation | fork |
| /deploy | Structured deployment: readiness gate, strategy selection, execution, monitoring, rollback | fork |
| /review | Structured code review (5 dimensions) | fork |
| /bug | File structured bug reports to backlog | inline |
| /consolidate | Backlog dedup, stale detection, cleanup | fork |
| /session-health | Context load and drift diagnostic | inline |
| /handoff | Session transition capture | inline |
| /status | Unified dashboard: backlog, activity, team, last session | inline |
| /advise | Proactive recommendations from git state, session history, backlog | inline |
| /evolution | File change history, churn, stability analysis | inline |
| /drift | Cross-definition convergence/divergence detection | inline |
| /diagnose-agent | Agent struggle profile from learnings evolution + git signals | inline |
| /challenge-gen | Generate targeted training challenges from struggle profile | inline |
| /challenge-run | Execute challenges and evaluate agent performance | inline |
| /domain | Capture or query project-specific terminology and disambiguation rules | inline |
| /test-strategy | Classify spec type, write tests from specs, enforce red-green gates | fork |
| /optimize | Profile, identify bottleneck, implement fix, measure, iterate toward target | fork |

### Teams (9) — `skills/teams/`

Persistent team orchestration and the learning lifecycle (curation, promotion, retrospectives).

| Skill | Purpose | Context |
|-------|---------|---------|
| /assemble | Create team with roles and ownership | inline |
| /standup | Sync status, surface blockers (uses Task dispatch) | inline |
| /sprint | Dispatch work with learning loop (uses Task dispatch) | inline |
| /team-meeting | Goal-oriented planning with persistent team (uses native teams) | inline |
| /active-learn | Full adversarial training loop: diagnose, challenge, evaluate, learn | fork |
| /curate | Score learnings/rules by relevance, archive stale, detect gaps | inline |
| /promote | Graduate durable cross-agent learnings to rules | inline |
| /tend | Orchestrate full learning lifecycle: curate -> promote -> demotion | inline |
| /retro | Session retrospective with persistent learnings | inline |

## Skills by Context Type

**Inline (37):** gather, distill, expand, transform, rank, filter, assess, verify, sketch, merge, decompose, critique, plan, diff-ideas, do, discover, fractal, meeting, bug, session-health, handoff, status, advise, evolution, drift, diagnose-agent, challenge-gen, challenge-run, domain, assemble, standup, sprint, team-meeting, curate, promote, tend, retro

**Fork (12):** blossom, bootstrap, consensus, consolidate, deploy, optimize, premortem, review, spec, test-strategy, tracer, active-learn

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
/gather -> /transform as ticket titles  # Research -> rewrite each finding as a ticket
/gather -> /distill -> /expand         # Research -> condense -> elaborate key findings
/drift -> /rank -> /sprint            # Compare definitions -> prioritize fixes -> execute
/tracer -> /optimize                  # Implement correctly -> then make it fast
/diagnose-agent -> /challenge-gen     # Profile agent -> generate targeted challenges
/diagnose-agent -> /challenge-gen -> /challenge-run  # Profile -> challenges -> evaluate
/active-learn                                       # Full loop: diagnose -> challenge -> evaluate -> learn (all inline)
/curate -> /promote                                 # Optimize learnings -> graduate to rules
/curate rules                                       # Audit project rules: scores, budget, gaps
/tend                                               # Full lifecycle: curate agents -> curate rules -> promote -> demotion -> summary
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

## Agents

| Agent | Purpose | Scope |
|-------|---------|-------|
| agent-generator | Generate project-specific agents | Global |
| project-bootstrapper | Bootstrap projects with Claude Code setup | Global |
| code-reviewer | Read-only code review | Global |

8 additional project-local agents live in `.claude/agents/` for authoring, research, and maintenance tasks specific to this repo.

## Further Reading

- [Workflow Pipelines](pipelines.md) — 6 canonical end-to-end lifecycle pipelines (discovery, team, deep analysis, planning, recursive exploration, session continuity)
- [Design Notes](design-notes.md) — architecture and design decision notes
- [Domain Mapping](domains.md) — domain-to-artifact mapping (referenced by /curate and /promote)
- [Primitives Cookbook](primitives-cookbook.md) — detailed usage patterns and examples
- [Primitives Recipes](primitives-recipes.md) — end-to-end workflow recipes
- [Team System Guide](team-system-guide.md) — team lifecycle, learnings, and coordination
