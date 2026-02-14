# Skill & Agent Navigator

Quick reference for finding the right skill or agent. See also: [Cookbook](primitives-cookbook.md), [Recipes](primitives-recipes.md), [Team Guide](team-system-guide.md).

## Decision Tree

**I want to...**

- **Explore something unknown** -> /blossom (broad, spike-driven) or /fractal (focused, goal-directed)
- **Research a specific topic** -> /gather (collect findings) -> /distill (condense) -> /rank (prioritize)
- **Compare approaches** -> /diff-ideas (two options) or /consensus (three independent proposals)
- **Plan before building** -> /decompose (break down) -> /plan (sequence) -> /spec (full specification)
- **Assess risk** -> /premortem (failure analysis) or /critique (adversarial review)
- **Build something** -> /tracer (iterative end-to-end) or /sketch (skeleton only)
- **Review code** -> /review (structured code review)
- **Manage a team** -> /assemble (create) -> /standup (sync) -> /sprint (dispatch)
- **Run a session** -> /status (orient) -> ... work ... -> /retro (reflect) -> /handoff (transition)
- **Discuss with multiple perspectives** -> /meeting (interactive group dialogue)

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

### Workflow Skills (12)

Orchestrated multi-step workflows with side effects (file writes, agent dispatch, backlog updates).

| Skill | Purpose | Context |
|-------|---------|---------|
| /blossom | Spike-driven exploration, produces epic + tasks | fork |
| /fractal | Goal-directed recursive exploration | inline |
| /meeting | Multi-agent group discussion | inline |
| /consensus | Three independent proposals + synthesis | fork |
| /premortem | Failure analysis with mitigations | fork |
| /spec | Progressive specification document | fork |
| /tracer | Iterative thin-slice implementation | fork |
| /review | Structured code review (5 dimensions) | fork |
| /consolidate | Backlog dedup, stale detection, cleanup | fork |
| /session-health | Context load and drift diagnostic | inline |
| /retro | Session retrospective with persistent learnings | inline |
| /handoff | Session transition capture | inline |

### Team Skills (3)

Manage persistent learning teams across sessions.

| Skill | Purpose | Context |
|-------|---------|---------|
| /assemble | Create team with roles and ownership | inline |
| /standup | Sync status, surface blockers | inline |
| /sprint | Dispatch work with learning loop | inline |

### Session Skills (1)

| Skill | Purpose | Context |
|-------|---------|---------|
| /status | Unified dashboard: backlog, activity, team, last session | inline |

## Skills by Context Type

**Inline (21):** gather, distill, rank, filter, assess, verify, sketch, merge, decompose, critique, plan, diff-ideas, fractal, meeting, session-health, retro, handoff, assemble, standup, sprint, status

**Fork (7):** blossom, consensus, consolidate, premortem, review, spec, tracer

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
```

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
