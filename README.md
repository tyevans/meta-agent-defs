# Workbench

**Your portable Claude Code toolkit. Clone once, use everywhere.**

Every time you start a new project with Claude Code, you rebuild the same scaffolding: agents for code review, commands for exploration, hooks to keep state in sync. You rediscover the same patterns. You re-encode the same discipline. This repo eliminates that repetition.

Workbench is a single git repo of portable workflow definitions -- skills, agents, hooks, settings, and MCP server configs -- that symlink into `~/.claude/` and work across all projects. Clone it once, run the installer, and every Claude Code session gets your full toolkit.

## What It Feels Like

**You open a new project.** You have no idea how the codebase is organized, what's broken, or where to start. You type:

```
/blossom audit the event sourcing pipeline for gaps
```

Blossom seeds an epic, spawns discovery spikes across the codebase, and recursively explores until every area is mapped. Each finding is tagged CONFIRMED, LIKELY, or POSSIBLE -- because the agents read the actual code, not just grep for patterns. Twenty minutes later you have a prioritized backlog of firm tasks, a dependency graph, and a recommended execution order. You didn't write a single line of code, but you understand the entire problem.

**Three sessions later, the backlog has grown.** Spikes have spawned tasks, tasks have spawned subtasks, and some of the early items might already be done. You type:

```
/consolidate
```

Consolidate surveys every open task, merges duplicates, fills vertical-slice gaps (found a domain task with no infrastructure companion? it creates one), detects stale work by checking git history, cleans up transitive dependencies, and hands you a before/after health report.

**You've been working for two hours.** Responses feel shorter, searches feel broader, and you're not sure if the session is still sharp. You type:

```
/session-health
```

You get an honest self-diagnostic: context load, scope drift, quality indicators, and a concrete recommendation -- continue, compact, checkpoint, start fresh, or break remaining work into subagents.

## Install in 30 Seconds

```bash
git clone https://github.com/your-user/workbench
cd workbench
./install.sh
```

That's it. The installer symlinks everything into `~/.claude/`. It's idempotent -- rerun it after pulling updates, and existing links are refreshed. Existing files are backed up with timestamps so nothing is lost.

### Install Options

```bash
./install.sh                              # Global symlinks to ~/.claude/
./install.sh /path/to/project             # Project-local (agents + skills only)
./install.sh --hardlink                   # Hardlinks instead of symlinks
./install.sh /path/to/project --hardlink  # Combine options
./install.sh --help                       # Show usage
```

Project-local mode installs only agents and skills.

### Uninstall

The installer writes a manifest listing every installed file. To uninstall:

```bash
xargs rm -f < ~/.claude/.meta-agent-defs.manifest
```

To also remove MCP servers:

```bash
claude mcp remove playwright --scope user
claude mcp remove sequential-thinking --scope user
claude mcp remove context7 --scope user
claude mcp remove memory --scope user
```

## Skills

Skills are the primary feature. Type a slash command, get a structured workflow. They divide into two categories based on how they run.

### Workflow Skills

Heavy, isolated workflows that fork into their own context. Use these for deep exploration, design, and implementation.

| Skill | What It Does |
|-------|-------------|
| `/blossom <goal>` | Spike-driven exploration. Recursively investigates codebase, produces prioritized backlog of verified tasks. |
| `/fractal <goal>` | Goal-directed recursive exploration. Prunes paths that don't serve the goal. Produces focused synthesis. |
| `/consensus <decision>` | Multi-perspective design synthesis. Three agents propose solutions optimized for simplicity, performance, maintainability. |
| `/spec <feature>` | Structured specification through progressive elaboration -- populate, validate, refine, architecture review. |
| `/premortem <feature>` | Risk-first failure analysis. Three agents examine through security, reliability, and business lenses. |
| `/tracer <feature>` | Iterative end-to-end implementation. Thinnest working path first, then widens pass by pass. |
| `/consolidate [area]` | Backlog hygiene. Deduplicates, fills gaps, detects stale tasks, cleans dependencies. |
| `/review [target]` | Structured code review across correctness, security, style, architecture, testing. |

### Utility Skills

Lightweight, inline skills that run in your current context. Use these for coordination, diagnostics, and session management.

| Skill | What It Does |
|-------|-------------|
| `/meeting <topic>` | Interactive multi-agent dialogue. 2 panelists with opposed perspectives, facilitator-mediated. |
| `/assemble <project>` | Persistent team creation with role-based ownership and cross-session learnings. |
| `/standup [team]` | Team status sync. Each agent reports progress, blockers, planned next actions. |
| `/sprint [focus]` | Sprint planning + dispatch with learning loop. Assigns tasks, dispatches agents, persists improvements. |
| `/session-health` | Self-diagnostic. Assesses context load, scope drift, quality degradation. Auto-discoverable. |
| `/handoff [focus]` | Session transition. Captures backlog state, decisions, patterns, recommended next steps. |
| `/retro [focus]` | Session retrospective. Evaluates velocity, quality, process, then persists learnings to MEMORY.md. |

### Composable Primitives

Small, stateless skills that follow the [pipe format](rules/pipe-format.md) contract. Chain them together -- context IS the pipe.

| Skill | What It Does |
|-------|-------------|
| `/gather <topic>` | Collect information into structured findings with sources and confidence levels. |
| `/distill` | Reduce verbose input to essential points. Configurable target: bullets, paragraph, or count. |
| `/rank <criteria>` | Score and order items by user-specified criteria. |
| `/filter <criterion>` | Binary keep/drop decision per item. The grep of knowledge work. |
| `/assess <rubric>` | Evaluate items against a rubric with categorical verdicts (critical/warning/suggestion). |
| `/decompose <goal>` | Break a goal into bounded sub-parts with clear scope boundaries. |
| `/diff-ideas <A> vs <B>` | Side-by-side tradeoff analysis across dimensions with per-dimension winner. |
| `/sketch` | Produce a minimal code skeleton with structure and TODO placeholders. No implementation. |
| `/verify` | Fact-check claims against source code or documentation. VERIFIED, REFUTED, or UNCERTAIN. |
| `/critique` | Surface weaknesses, gaps, and risks in a proposal or plan. |
| `/plan` | Convert analysis into an ordered action plan with dependencies. |
| `/merge` | Combine multiple pipe-format outputs into a unified synthesis. |

## Agents

Three portable agents that work on any project.

| Agent | What It Does |
|-------|-------------|
| `agent-generator` | Analyzes a project's architecture and generates tailored project-specific agents in `.claude/agents/`. |
| `project-bootstrapper` | Full project setup: beads task management, CLAUDE.md, hooks, permissions, rules, memory directory. |
| `code-reviewer` | Reviews code changes for correctness, security, style, and architectural coherence. Read-only. |

## Hooks and Safety

The global `settings.json` wires up hooks that run automatically across all projects.

| Hook | When | What |
|------|------|------|
| SessionStart | Every new session | Pre-session diagnostics (dirty tree, unpushed commits) + loads beads context |
| PreCompact | Before context compaction | Flushes beads state so nothing is lost |
| PreToolUse (Bash) | Before `git push` | Warns if beads changes are uncommitted |
| PreToolUse (Bash) | Before destructive commands | Warns on `git reset --hard`, `git checkout .`, `git clean -f`, `rm -rf` |
| PostToolUse (Task) | After any subagent completes | Review gate reminder: verify deliverable before moving on |
| SessionEnd | Session closes | Auto-syncs beads state |

All hooks fail gracefully with `|| true` for optional tools, so projects without beads installed still work fine.

## MCP Servers

Four Model Context Protocol servers installed globally via `claude mcp add`.

| Server | What |
|--------|------|
| `playwright` | Browser automation for testing and scraping |
| `sequential-thinking` | Structured multi-step reasoning |
| `context7` | Up-to-date library documentation lookup |
| `memory` | Persistent knowledge graph across sessions |

MCP servers are defined in `mcp-servers.json`. Add or remove entries there and rerun `./install.sh`.

## OpenTelemetry

Claude Code exports metrics (token usage, cost, session counts, lines changed) and events (tool calls, API requests) via OpenTelemetry. Telemetry is enabled by default and sends to a local OTLP collector on `localhost:4317`.

To set up a collector:

1. Run any OTLP-compatible collector on `localhost:4317` (the pre-configured endpoint)
2. For a turnkey stack with Grafana dashboards, see [claude-code-otel](https://github.com/ColeMurray/claude-code-otel) (`docker compose up`)

To disable telemetry, set `CLAUDE_CODE_ENABLE_TELEMETRY=0` in `settings.json`.

Optional privacy controls (set in your environment, not in settings.json):

- `OTEL_LOG_USER_PROMPTS=1` -- include prompt content in events (redacted by default)
- `OTEL_LOG_TOOL_DETAILS=1` -- include MCP server/tool and skill names in events

For cloud backends (Honeycomb, Datadog, Grafana Cloud), override `OTEL_EXPORTER_OTLP_ENDPOINT` and add `OTEL_EXPORTER_OTLP_HEADERS` with your auth token.

## How Skills Compose

The skills connect into natural workflows depending on where you are in a project:

```
 /meeting ──> /fractal ──────────> synthesis ──> /spec or /tracer
  (discuss)   (deep-dive)                        (design or build)

 /blossom ─────────────> backlog ──────> pick a task ──> /tracer
   (explore)                                               (build)

 /consensus ──> /spec ──────────────────────────────────> /tracer
  (decide)     (design)                                    (build)

 /premortem ──> mitigations ──> /spec or /tracer
  (stress-test)                  (design or build with mitigations baked in)

 /assemble ──> /standup ──> /sprint ──> dispatch work ──> /standup ──> ...
  (form team)  (sync)       (plan)      (any skill)       (sync again)
```

**Common flows:**

- **Team-based project**: `/assemble` -> `/standup` -> `/sprint` -> dispatch work -> `/standup` (repeat)
- **Requirements unclear**: `/meeting` (brainstorm with panel) -> `/fractal` (deep-dive findings) -> `/spec` (design)
- **Greenfield feature**: `/consensus` (pick an approach) -> `/spec` (detailed design) -> `/tracer` (incremental implementation)
- **Exploration-first**: `/blossom` (discover scope) -> pick highest-priority task -> `/tracer` (implement it)
- **Focused understanding**: `/fractal` (goal-directed exploration) -> synthesis of findings -> decide next step
- **High-stakes change**: `/premortem` (surface risks) -> feed mitigations into `/spec` or `/tracer`
- **Just build it**: `/tracer` alone when the path is clear and you want always-working increments

Each skill produces artifacts (backlogs, specs, decision records, mitigation plans) that persist across sessions and feed into downstream skills.

## Design Philosophy

Three principles shaped these workflows:

**1. Hooks are for guarantees, instructions are for guidance.** Hooks fire 100% of the time -- use them for things that must never be forgotten (syncing state, review gates). CLAUDE.md instructions are for things that should usually happen but require judgment. Skills are for on-demand workflows you invoke when you need them.

**2. Verify, don't speculate.** Every spike agent in `/blossom` is instructed to read the actual implementation, trace call chains, check callers and tests, and state a confidence level. CONFIRMED means the agent read the code and verified the issue. LIKELY means strong evidence but incomplete trace. POSSIBLE means it needs a deeper spike. This distinction matters -- it's the difference between a backlog of real work and a backlog of guesses.

**3. Serialize by default, parallelize with teams.** For sequential work, the orchestrator dispatches one agent at a time, reviews its output, and learns from it before dispatching the next. For independent work (blossom spikes, parallel audits), agent teams run in separate contexts where throttling and context bloat don't apply.

## Project Structure

```
workbench/
  agents/
    agent-generator.md        # Generates project-specific agents from codebase analysis
    project-bootstrapper.md   # Full project setup for Claude Code + Beads
    code-reviewer.md          # Read-only code review across 4 quality dimensions
  skills/
    blossom/SKILL.md          # /blossom -- spike-driven exploration (fork)
    fractal/SKILL.md          # /fractal -- goal-directed recursive exploration (inline)
    meeting/SKILL.md          # /meeting -- interactive multi-agent dialogue (inline)
    assemble/SKILL.md         # /assemble -- persistent team creation (inline)
    standup/SKILL.md          # /standup -- team status sync (inline)
    sprint/SKILL.md           # /sprint -- sprint planning + dispatch (inline)
    consensus/SKILL.md        # /consensus -- multi-perspective design synthesis (fork)
    spec/SKILL.md             # /spec -- structured specification (fork)
    premortem/SKILL.md        # /premortem -- risk-first failure analysis (fork)
    tracer/SKILL.md           # /tracer -- iterative end-to-end implementation (fork)
    consolidate/SKILL.md      # /consolidate -- backlog review (fork)
    review/SKILL.md           # /review -- code review (fork)
    session-health/SKILL.md   # /session-health -- session diagnostic (inline)
    handoff/SKILL.md          # /handoff -- session transition (inline)
    retro/SKILL.md            # /retro -- session retrospective (inline)
    gather/SKILL.md           # /gather -- collect structured findings (inline)
    distill/SKILL.md          # /distill -- condense to essentials (inline)
    rank/SKILL.md             # /rank -- score and order items (inline)
    filter/SKILL.md           # /filter -- binary keep/drop (inline)
    assess/SKILL.md           # /assess -- categorical rubric evaluation (inline)
    decompose/SKILL.md        # /decompose -- break into sub-parts (inline)
    diff-ideas/SKILL.md       # /diff-ideas -- side-by-side tradeoffs (inline)
    sketch/SKILL.md           # /sketch -- code skeleton with TODOs (inline)
    verify/SKILL.md           # /verify -- fact-check claims (inline)
    critique/SKILL.md         # /critique -- surface weaknesses (inline)
    plan/SKILL.md             # /plan -- ordered action plan (inline)
    merge/SKILL.md            # /merge -- combine pipe outputs (inline)
  docs/
    primitives-cookbook.md     # Annotated walkthroughs of primitive chains
    primitives-recipes.md     # Common chain patterns
    team-system-guide.md      # Standalone adopter guide for team system
    demos/                    # Demo walkthroughs (build-a-feature, OSS audit)
  demos/
    todo-app/                 # Intentionally buggy Express app for demo walkthroughs
  rules/
    team-protocol.md          # Team manifest format, spawn protocol, reflection schema (global)
    pipe-format.md            # Composable primitive output contract (global)
  settings.json               # Global hooks, env vars, and feature flags
  mcp-servers.json            # MCP server definitions (installed globally by install.sh)
  install.sh                  # Symlink installer (idempotent, non-destructive)
  CLAUDE.md                   # Context file for working on this repo itself
```

## Getting Started

After installing, here's how to go from zero to productive.

**On a new project:**

1. Run the project bootstrapper agent to set up beads, CLAUDE.md, hooks, permissions, and rules
2. Run the agent generator to create project-specific agents tailored to the codebase
3. Use `/blossom` to explore any area you want to understand or improve

**On an existing project:**

1. Type `/blossom <what you want to explore>` and let it map the territory
2. Use `/consolidate` whenever the backlog feels noisy
3. Use `/session-health` when you've been working a while and want a gut check

**On a new machine:**

```bash
git clone https://github.com/your-user/workbench
cd workbench && ./install.sh
```

Your entire Claude Code workflow travels with you.

## Extending

**Add a skill:** Create `skills/<name>/SKILL.md` with YAML frontmatter (`name`, `description`, `allowed-tools`, `context`). Set `context: fork` for heavy workflows, omit it for inline. Rerun `./install.sh`.

**Add an agent:** Create `agents/<name>.md` with YAML frontmatter (`name`, `description`, `tools`, `model`). Rerun `./install.sh`.

**Add an MCP server:** Add an entry to `mcp-servers.json` with `command` and `args`. Rerun `./install.sh`.

**Add a hook:** Edit `settings.json` and add to the appropriate hook array (`SessionStart`, `PreCompact`, `PreToolUse`, `PostToolUse`, `SessionEnd`). Use `|| true` for any tool that might not be installed.

The real power comes from layering: these global definitions provide the workflow skeleton, while per-project `.claude/agents/` (created by the agent generator) provide project-specific intelligence. Global workflows, local expertise.
