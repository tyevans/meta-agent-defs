# Tackline

**Composable workflows for agent-driven development.** Clone once, symlink into `~/.claude/`, use across every project.

*In naval semaphore, a tackline separates signal groups to prevent ambiguity. Here, it separates composed skill outputs -- each skill's result feeds the next through conversation context, like Unix pipes for knowledge work.*

## What It Looks Like

### Map an unfamiliar codebase

```
/blossom audit the event sourcing pipeline for consistency gaps
```

Blossom spawns parallel spike agents that read actual source files, trace call chains, and tag findings with confidence levels. Output is a prioritized task backlog, not a summary.

```
Seeding epic: "Event sourcing pipeline audit"
Dispatching 4 spikes: command-handlers, projections, event-store, saga-orchestration

Spike results:
1. CONFIRMED -- ProjectionRebuilder skips tombstoned events (projections/rebuild.ts:47)
2. CONFIRMED -- No idempotency check on CommandBus.dispatch (handlers/bus.ts:112)
3. LIKELY    -- Saga timeout defaults to 0, may cause silent drops (sagas/orchestrator.ts:88)
4. POSSIBLE  -- Event schema v2 migration has unused backward-compat shim

Created 6 tasks under epic BL-42, dependency graph attached
```

### Hash out a design decision

```
/meeting should we use event sourcing or CQRS-lite for the new billing module?
```

Assembles 2 panelists with genuinely opposed perspectives and a facilitator. You steer the conversation, ask follow-ups, cut threads that aren't productive. Output is a decision record, not a compromise.

### Research, distill, and prioritize

```
/gather authentication patterns in this codebase
/distill
/rank by security risk
```

Each skill's output feeds the next through conversation context. No file passing, no explicit piping -- context is the pipe.

## Install

```bash
git clone https://github.com/tyevans/tackline
cd tackline
./install.sh
```

Zero dependencies. The installer symlinks everything into `~/.claude/`. Rerun after pulling updates -- it's idempotent, existing files are backed up.

```bash
./install.sh /path/to/project    # Project-local install (agents + skills only)
./install.sh --hardlink           # Hardlinks instead of symlinks
```

### Uninstall

```bash
xargs rm -f < ~/.claude/.tackline.manifest
```

## Getting Started

**New project:** Run the `project-bootstrapper` agent to set up CLAUDE.md, hooks, and rules. Then run `agent-generator` to create project-specific agents. Start exploring with `/blossom`.

**Existing project:** Type `/blossom <what you want to explore>` to map the territory. Use `/consolidate` when the backlog grows noisy. Use `/session-health` for a context quality gut-check.

**New machine:** Clone, install, go. Your entire Claude Code workflow travels with you.

## Skills (40)

### Composable Primitives

Stateless skills that chain through conversation context. Output of any feeds the next.

| Skill | Purpose |
|-------|---------|
| `/gather` | Collect findings with sources and confidence levels |
| `/distill` | Reduce to essentials |
| `/rank` | Score and order by criteria |
| `/filter` | Binary keep/drop |
| `/assess` | Categorize by rubric (critical/warning/ok) |
| `/verify` | Check claims against evidence |
| `/critique` | Adversarial review -- what's wrong, missing, risky |
| `/diff-ideas` | Side-by-side tradeoff analysis |
| `/decompose` | Break into bounded sub-parts |
| `/plan` | Dependency-aware execution sequence |
| `/sketch` | Code skeleton with TODOs |
| `/merge` | Combine multiple outputs into one |

### Workflows

Orchestrated multi-step workflows with side effects.

| Skill | Purpose |
|-------|---------|
| `/blossom` | Spike-driven exploration -- produces epic + prioritized tasks |
| `/fractal` | Goal-directed recursive exploration with dead-end pruning |
| `/meeting` | Live multi-agent discussion with opposed perspectives |
| `/consensus` | Three independent proposals, synthesized |
| `/tracer` | Iterative implementation -- thinnest working path first |
| `/review` | Structured code review across 5 dimensions |
| `/sprint` | Dispatch work to agents with a learning loop |
| `/spec` | Progressive specification document |
| `/premortem` | Failure analysis before building |
| `/bootstrap` | Full project setup: infrastructure + agents |
| `/active-learn` | Adversarial training loop for agent improvement |

### Session & Team

| Skill | Purpose |
|-------|---------|
| `/status` | Unified dashboard: backlog, activity, team, last session |
| `/advise` | Proactive recommendations from git state, history, and signals |
| `/assemble` | Create a persistent learning team |
| `/standup` | Sync status, surface blockers |
| `/retro` | Session retrospective with persistent learnings |
| `/handoff` | Session transition capture |
| `/session-health` | Context load and drift diagnostic |

[Full catalog with decision tree and chain patterns](docs/INDEX.md)

## Common Chains

```
/gather -> /distill -> /rank           Research -> condense -> prioritize
/decompose -> /rank -> /plan           Break down -> prioritize -> sequence
/gather -> /critique -> /rank          Research -> stress-test -> prioritize
/assemble -> /standup -> /sprint       Form team -> sync -> dispatch
/blossom -> pick tasks -> /tracer      Explore -> build
/meeting -> /fractal -> /spec          Discuss -> deep-dive -> specify
```

## How It Actually Works

**Skills are LLM-driven, not deterministic.** The same input may produce different findings across runs. Confidence levels (CONFIRMED, LIKELY, POSSIBLE) help you calibrate trust. Output quality scales directly with input specificity -- a precise goal with file paths produces better results than a vague one.

**Fork-context skills spawn subagents.** `/blossom`, `/consensus`, and `/premortem` each dispatch agents that read source files independently. On large codebases, expect meaningful API usage. Inline skills (primitives, `/meeting`, `/status`) are lightweight.

**Everything works without extras.** No beads, no MCP servers required. You just get fewer features. Hooks degrade gracefully with `|| true`.

## Extending

**Add a skill:** Create `skills/<name>/SKILL.md` with YAML frontmatter. Rerun `./install.sh`.

**Add an agent:** Create `agents/<name>.md` with YAML frontmatter. Rerun `./install.sh`.

**Add a hook:** Edit `settings.json`. Use `|| true` for tools that might not be installed.

## Learn More

- [Full skill & agent catalog](docs/INDEX.md) -- 40 skills, 3 agents, decision tree, chain patterns
- [Technical reference](docs/reference.md) -- hooks, MCP servers, project structure, design philosophy
- [Composable primitive patterns](docs/primitives-cookbook.md) -- annotated walkthroughs
- [Team system guide](docs/team-system-guide.md) -- persistent learning teams

## License

[MIT](LICENSE)
