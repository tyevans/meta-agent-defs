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
# Add the marketplace (one-time), then install
claude plugin marketplace add tyevans/tackline
claude plugin install tackline@tackline

# Install global rules (plugin system doesn't support rules yet)
# No local checkout needed — fetch directly from GitHub:
curl -fsSL https://raw.githubusercontent.com/tyevans/tackline/main/dev/install-rules.sh | bash
# Or, from a local checkout:
# bash /path/to/tackline/dev/install-rules.sh
```

### Uninstall

```bash
claude plugin uninstall tackline@tackline
# Remove rule symlinks (leaves non-tackline rules intact)
for f in /path/to/tackline/rules/*.md; do rm -f ~/.claude/rules/"$(basename "$f")"; done
```

## Getting Started

**New project:** Run the `project-bootstrapper` agent to set up CLAUDE.md, hooks, and rules. Then run `agent-generator` to create project-specific agents. Start exploring with `/blossom`.

**Existing project:** Type `/blossom <what you want to explore>` to map the territory. Use `/consolidate` when the backlog grows noisy. Use `/session-health` for a context quality gut-check.

**New machine:** Clone, install, go. Your entire Claude Code workflow travels with you.

## Skills (48)

**14 composable primitives** -- stateless skills that chain through conversation context (`/gather`, `/distill`, `/rank`, `/filter`, `/assess`, `/verify`, `/critique`, `/diff-ideas`, `/decompose`, `/plan`, `/expand`, `/transform`, `/sketch`, `/merge`).

**25 workflows** -- orchestrated multi-step workflows with side effects (`/blossom`, `/fractal`, `/meeting`, `/consensus`, `/tracer`, `/review`, `/sprint`, `/spec`, `/premortem`, `/bootstrap`, `/drive`, `/deploy`, `/optimize`, `/test-strategy`, `/evolution`, `/drift`, `/consolidate`, `/bug`, `/domain`, `/active-learn`, `/diagnose-agent`, `/challenge-gen`, `/challenge-run`, `/storm-prep`, `/formalize`).

**9 session & team skills** -- `/status`, `/advise`, `/assemble`, `/standup`, `/retro`, `/handoff`, `/session-health`, `/team-meeting`, `/discover`.

[Full catalog with decision tree and chain patterns](docs/INDEX.md)

## Common Chains

```
/gather -> /distill -> /rank       Research -> condense -> prioritize
/decompose -> /plan -> /spec       Break down -> sequence -> specify
/blossom -> /drive                 Explore -> autonomously implement
```

[All chain patterns](docs/INDEX.md#primitive-chain-patterns)

## How It Actually Works

**Skills are LLM-driven, not deterministic.** The same input may produce different findings across runs. Confidence levels (CONFIRMED, LIKELY, POSSIBLE) help you calibrate trust. Output quality scales directly with input specificity -- a precise goal with file paths produces better results than a vague one.

**Fork-context skills spawn subagents.** `/blossom`, `/consensus`, and `/premortem` each dispatch agents that read source files independently. On large codebases, expect meaningful API usage. Inline skills (primitives, `/meeting`, `/status`) are lightweight.

**Everything works without extras.** No MCP servers required. You just get fewer features. Hooks degrade gracefully with `|| true`.

## Extending

**Add a skill:** Create `skills/<name>/SKILL.md` with YAML frontmatter. Run `claude plugin update tackline@tackline`.

**Add an agent:** Create `agents/<name>.md` with YAML frontmatter. Run `claude plugin update tackline@tackline`.

## Learn More

- [Full skill & agent catalog](docs/INDEX.md) -- 46 skills, 3 agents, decision tree, chain patterns
- [Technical reference](docs/reference.md) -- hooks, MCP servers, project structure, design philosophy
- [Composable primitive patterns](docs/primitives-cookbook.md) -- annotated walkthroughs
- [Team system guide](docs/team-system-guide.md) -- persistent learning teams

## License

[MIT](LICENSE)
