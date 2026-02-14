# Workbench

**Pre-built workflows that give structure to agent-driven development -- clone once, symlink into `~/.claude/`, use across every project.**

## What You Get

### Map an unfamiliar codebase in one command

    /blossom audit the event sourcing pipeline for consistency gaps

Blossom spawns parallel spike agents that read actual source files, trace call chains, and tag each finding with a confidence level. It produces a prioritized backlog of concrete tasks, not summaries of what might be wrong.

    Seeding epic: "Event sourcing pipeline audit"
    Dispatching 4 spikes: command-handlers, projections, event-store, saga-orchestration

    Spike results:
    1. CONFIRMED — ProjectionRebuilder skips tombstoned events (projections/rebuild.ts:47)
    2. CONFIRMED — No idempotency check on CommandBus.dispatch (handlers/bus.ts:112)
    3. LIKELY   — Saga timeout defaults to 0, may cause silent drops (sagas/orchestrator.ts:88)
    4. POSSIBLE — Event schema v2 migration has unused backward-compat shim

    3/4 spikes complete, 1 timed out (saga-orchestration — codebase too deep, recommend follow-up /fractal)
    Created 6 tasks under epic BL-42, dependency graph attached

### Hash out a design decision with an agent panel

    /meeting should we use event sourcing or CQRS-lite for the new billing module?

Meeting assembles 2 panelists with genuinely opposed perspectives (here: a consistency advocate and a simplicity advocate) and a facilitator. You steer the conversation, ask follow-ups, and cut off threads that aren't productive. The panel produces a decision record, not a compromise.

    Assembling panel...
      - Consistency Advocate: argues for full event sourcing with replay guarantees
      - Simplicity Advocate: argues for CQRS-lite with snapshot-only persistence

    [3 rounds of moderated discussion]

    Decision record:
      Recommendation: CQRS-lite for billing, with event log as audit trail (not source of truth)
      Key tradeoff: lose replay capability, gain 60% less operational complexity
      Dissent noted: Consistency Advocate flags audit log drift risk — mitigate with nightly reconciliation job

## Install

```bash
git clone https://github.com/tyevans/meta-agent-defs
cd meta-agent-defs
./install.sh
```

The installer symlinks everything into `~/.claude/`. It is idempotent -- rerun after pulling updates. Existing files are backed up with timestamps.

### Install Options

```bash
./install.sh                              # Global symlinks to ~/.claude/
./install.sh /path/to/project             # Project-local (agents + skills only)
./install.sh --hardlink                   # Hardlinks instead of symlinks
./install.sh --help                       # Show usage
```

### Uninstall

```bash
xargs rm -f < ~/.claude/.meta-agent-defs.manifest
```

To also remove MCP servers, see [docs/reference.md](docs/reference.md).

## Getting Started

**New project:**

1. Run the `project-bootstrapper` agent to set up CLAUDE.md, hooks, permissions, and rules
2. Run the `agent-generator` agent to create project-specific agents tailored to the codebase
3. Use `/blossom` to explore any area you want to understand or improve

**Existing project:**

1. Type `/blossom <what you want to explore>` and let it map the territory
2. Use `/consolidate` when the backlog feels noisy
3. Use `/session-health` when you want a gut check on context quality

**New machine:**

```bash
git clone https://github.com/tyevans/meta-agent-defs
cd meta-agent-defs && ./install.sh
```

Your entire Claude Code workflow travels with you.

## Skill Highlights

| Skill | What it does |
|-------|-------------|
| `/blossom <goal>` | Spawns parallel spikes to explore a codebase area; produces a prioritized task backlog with confidence levels |
| `/fractal <goal>` | Goal-directed recursive exploration that prunes dead ends; produces a focused synthesis |
| `/meeting <topic>` | Live multi-agent discussion with opposed perspectives; you steer, it produces a decision record |
| `/consensus <decision>` | Three independent proposals optimized for simplicity, performance, and maintainability; synthesized into a recommendation |
| `/tracer <feature>` | Iterative implementation -- thinnest working path first, then widens pass by pass |
| `/sprint [focus]` | Sprint planning with dispatch; assigns tasks to agents and tracks progress with a learning loop |
| `/session-health` | Self-diagnostic for context load, scope drift, and quality degradation; recommends continue, compact, or restart |
| `/review [target]` | Structured code review across correctness, security, style, architecture, and testing |

See the [full skill & agent catalog](docs/INDEX.md) for all 28 skills, 3 agents, and a decision tree.

## Common Workflows

Skills chain together into natural workflows:

```
 /assemble ──> /standup ──> /sprint ──> dispatch work ──> /standup ──> ...
  (form team)  (sync)       (plan)      (any skill)       (sync again)

 /blossom ─────────────> backlog ──────> pick a task ──> /tracer
   (explore)                                               (build)

 /meeting ──> /fractal ──────────> synthesis ──> /spec or /tracer
  (discuss)   (deep-dive)                        (design or build)
```

## Good to Know

- **Output quality scales with input specificity.** A vague goal produces vague findings; a precise goal with file paths or module names produces actionable ones.
- **Fork-context skills consume significant tokens.** `/blossom`, `/consensus`, and `/premortem` spawn subagents that each read source files. On large codebases, expect meaningful API usage.
- **These are LLM-driven workflows, not deterministic tools.** The same input may produce different findings across runs. Confidence levels (CONFIRMED, LIKELY, POSSIBLE) help you calibrate trust.
- **Everything works without beads installed.** You just lose backlog tracking. All exploration, coordination, and session skills function independently.

## Extending

**Add a skill:** Create `skills/<name>/SKILL.md` with YAML frontmatter (`name`, `description`, `allowed-tools`, `context`). Set `context: fork` for heavy workflows, omit it for inline. Rerun `./install.sh`.

**Add an agent:** Create `agents/<name>.md` with YAML frontmatter (`name`, `description`, `tools`, `model`). Rerun `./install.sh`.

**Add an MCP server:** Add an entry to `mcp-servers.json` with `command` and `args`. Rerun `./install.sh`.

**Add a hook:** Edit `settings.json` and add to the appropriate hook array (`SessionStart`, `PreCompact`, `PreToolUse`, `PostToolUse`, `SessionEnd`). Use `|| true` for any tool that might not be installed.

Global workflows, local expertise.

## Learn More

- [Full skill & agent catalog](docs/INDEX.md) -- all 28 skills, 3 agents, decision tree
- [Hooks, MCP servers, project structure, design philosophy](docs/reference.md)
- [Composable primitive patterns](docs/primitives-cookbook.md)
- [Team system guide](docs/team-system-guide.md)
