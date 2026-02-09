# meta-agent-defs

**Your Claude Code sessions are ephemeral. Your workflows shouldn't be.**

Every time you start a new project with Claude Code, you rebuild the same scaffolding: agents for code review, commands for exploration, hooks to keep state in sync. You rediscover the same patterns. You re-encode the same discipline. This repo eliminates that repetition.

`meta-agent-defs` is a single git repo of portable workflow definitions -- agents, slash commands, and hooks -- that symlink into `~/.claude/` and work everywhere. Clone it once, run the installer, and every Claude Code session on every project gets your full toolkit.

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
git clone https://github.com/your-user/meta-agent-defs
cd meta-agent-defs
./install.sh
```

That's it. The installer symlinks everything into `~/.claude/`. It's idempotent -- rerun it after pulling updates, and existing symlinks are refreshed. Existing files are backed up with timestamps so nothing is lost.

To uninstall symlinks (run from the repo directory):

```bash
find ~/.claude -type l -lname "$(pwd)/*" -delete
```

To also remove MCP servers:

```bash
claude mcp remove playwright --scope user
claude mcp remove sequential-thinking --scope user
claude mcp remove context7 --scope user
claude mcp remove memory --scope user
```

## What Gets Installed

The installer creates symlinks from `~/.claude/` to this repo. Edit files here, and changes are live immediately.

### Skills (Slash Commands)

**Workflow skills** -- composable workflows for different phases of AI-assisted development:

| Skill | Mode | What It Does |
|-------|------|-------------|
| `/blossom <goal>` | fork | Spike-driven exploration. Takes a vague goal and recursively investigates the codebase, producing a prioritized backlog of verified tasks. |
| `/consensus <decision>` | fork | Multi-perspective design synthesis. Three independent agents propose solutions optimized for simplicity, performance, and maintainability, then surfaces agreements and tensions. |
| `/spec <feature>` | fork | Structured specification. Produces a complete design document through progressive elaboration -- populate, validate, refine, architecture review. |
| `/premortem <feature>` | fork | Risk-first analysis. Three agents examine a feature through security, reliability, and user/business lenses, then designs concrete mitigations. |
| `/tracer <feature>` | fork | Iterative end-to-end implementation. Builds the thinnest working path first, then widens pass by pass (error handling, edge cases, tests). |

**Utility skills** -- session lifecycle and maintenance:

| Skill | Mode | What It Does |
|-------|------|-------------|
| `/consolidate [area]` | fork | Backlog hygiene. Deduplicates, fills vertical-slice gaps, detects stale tasks, cleans up dependencies. |
| `/review [target]` | fork | Structured code review. Examines staged changes, commits, or PRs across correctness, security, style, architecture, and test coverage. |
| `/session-health` | inline | Self-diagnostic. Assesses context load, scope drift, and quality degradation. Auto-discoverable by Claude. |
| `/handoff [focus]` | inline | Session transition. Captures backlog state, decisions, patterns, and recommended next steps. |
| `/retro [focus]` | inline | Session retrospective. Evaluates velocity, quality, process, blockers, and discoveries, then persists durable learnings to MEMORY.md. |

Skills use the modern Claude Code skills format (`skills/<name>/SKILL.md`) with YAML frontmatter for `context: fork` isolation, `allowed-tools` restrictions, and auto-discovery via descriptions. Legacy `commands/*.md` files are kept as fallbacks.

### Agents

| Agent | What It Does |
|-------|-------------|
| `agent-generator` | Analyzes a project's architecture and generates a tailored suite of project-specific agents in `.claude/agents/`, complete with investigation protocols, context management strategies, and an agent catalog. |
| `project-bootstrapper` | Full 10-phase project setup: beads task management, CLAUDE.md, hooks, permissions, architectural rules, memory directory, and initial tasks. Takes a bare repo to fully-configured Claude Code workspace. |
| `code-reviewer` | Reviews code changes for correctness, security, style, and architectural coherence. Read-only -- identifies and reports issues but does not modify code. |

### Settings & Hooks

The global `settings.json` wires up hooks that run across all projects:

| Hook | When It Fires | What It Does |
|------|--------------|-------------|
| **SessionStart** | Every new session | Pre-session diagnostics (dirty tree, unpushed commits) + loads beads context |
| **PreCompact** | Before context compaction | Flushes beads state so nothing is lost |
| **PreToolUse** (Bash) | Before `git push` | Warns if beads changes are uncommitted |
| **PreToolUse** (Bash) | Before destructive commands | Warns on `git reset --hard`, `git checkout .`, `git clean -f`, `rm -rf`, direct `.beads/` edits |
| **PostToolUse** (Task) | After any subagent completes | Review gate reminder: verify the deliverable before moving on |
| **SessionEnd** | Session closes | Auto-syncs beads state (`bd sync --flush-only`) so nothing is lost |

Also enables the experimental agent teams feature, MCP tool search (`auto:5` defers tool loading until referenced), and OpenTelemetry telemetry (see [OpenTelemetry](#opentelemetry) below).

### MCP Servers

The installer registers these Model Context Protocol servers globally (requires `claude` CLI):

| Server | What It Does |
|--------|-------------|
| `playwright` | Browser automation for testing and scraping via Playwright |
| `sequential-thinking` | Structured multi-step reasoning for complex problems |
| `context7` | Up-to-date library documentation lookup |
| `memory` | Persistent knowledge graph across sessions |

MCP servers are defined in `mcp-servers.json`. Add or remove entries there and rerun `./install.sh`.

### OpenTelemetry

Claude Code exports metrics (token usage, cost, session counts, lines changed) and events (tool calls, API requests, prompts) via OpenTelemetry. Telemetry is **enabled by default** in `settings.json` and sends to a local OTLP collector on `localhost:4317`.

To set up a collector:

1. Run any OTLP-compatible collector on `localhost:4317` (the pre-configured endpoint)
2. For a turnkey stack with Grafana dashboards, see [claude-code-otel](https://github.com/ColeMurray/claude-code-otel) (`docker compose up`)

To disable telemetry, set `CLAUDE_CODE_ENABLE_TELEMETRY=0` in `settings.json`.

Optional privacy controls (set in your environment, not in settings.json):

- `OTEL_LOG_USER_PROMPTS=1` -- include prompt content in events (redacted by default)
- `OTEL_LOG_TOOL_DETAILS=1` -- include MCP server/tool and skill names in events

For cloud backends (Honeycomb, Datadog, Grafana Cloud), override `OTEL_EXPORTER_OTLP_ENDPOINT` and add `OTEL_EXPORTER_OTLP_HEADERS` with your auth token.

## Workflow Skill Library

The five workflow skills form a composable library for different phases and situations in AI-assisted development. Each skill is self-contained, but they chain naturally.

### Quick Reference

| Skill | One-Line Description | When to Use | Produces |
|-------|---------------------|-------------|----------|
| `/blossom` | Recursive spike-driven exploration | You don't know what to build yet | Prioritized backlog with verified tasks |
| `/consensus` | Multi-perspective design synthesis | Multiple valid approaches, unclear trade-offs | Decision record with agreements and tensions |
| `/spec` | Structured specification through progressive elaboration | You know what to build but need a complete design | Spec document (`.specs/*.md`) reviewed by architecture guardian |
| `/premortem` | Risk-first failure analysis | High-stakes feature, security-sensitive change | Ranked failure scenarios with concrete mitigations |
| `/tracer` | Iterative end-to-end implementation | Integration risk is high, feature crosses boundaries | Working code in committable increments |

### How They Compose

The skills connect into natural workflows depending on where you are in a project:

```
 /blossom ─────────────> backlog ──────> pick a task ──> /tracer
   (explore)                                               (build)

 /consensus ──> /spec ──────────────────────────────────> /tracer
  (decide)     (design)                                    (build)

 /premortem ──> mitigations ──> /spec or /tracer
  (stress-test)                  (design or build with mitigations baked in)
```

**Common flows:**

- **Greenfield feature**: `/consensus` (pick an approach) -> `/spec` (detailed design) -> `/tracer` (incremental implementation)
- **Exploration-first**: `/blossom` (discover scope) -> pick highest-priority task -> `/tracer` (implement it)
- **High-stakes change**: `/premortem` (surface risks) -> feed mitigations into `/spec` or `/tracer`
- **Just build it**: `/tracer` alone when the path is clear and you want always-working increments

Each skill produces artifacts (backlogs, specs, decision records, mitigation plans) that persist across sessions and feed into downstream skills.

## The Design Thinking

Three principles shaped these workflows:

**1. Hooks are for guarantees, instructions are for guidance.** Hooks fire 100% of the time -- use them for things that must never be forgotten (syncing state, review gates). CLAUDE.md instructions are for things that should usually happen but require judgment. Slash commands are for on-demand workflows you invoke when you need them.

**2. Verify, don't speculate.** Every spike agent in `/blossom` is instructed to read the actual implementation, trace call chains, check callers and tests, and state a confidence level. CONFIRMED means the agent read the code and verified the issue. LIKELY means strong evidence but incomplete trace. POSSIBLE means it needs a deeper spike. This distinction matters -- it's the difference between a backlog of real work and a backlog of guesses.

**3. Serialize by default, parallelize with teams.** For sequential work, the orchestrator dispatches one agent at a time, reviews its output, and learns from it before dispatching the next. For independent work (blossom spikes, parallel audits), agent teams run in separate contexts where throttling and context bloat don't apply.

## Project Structure

```
meta-agent-defs/
  agents/
    agent-generator.md        # Generates project-specific agents from codebase analysis
    project-bootstrapper.md   # 10-phase project setup for Claude Code + Beads
    code-reviewer.md          # Read-only code review across 4 quality dimensions
  skills/
    blossom/SKILL.md          # /blossom -- spike-driven exploration (fork)
    consensus/SKILL.md        # /consensus -- multi-perspective design synthesis (fork)
    spec/SKILL.md             # /spec -- structured specification (fork)
    premortem/SKILL.md        # /premortem -- risk-first failure analysis (fork)
    tracer/SKILL.md           # /tracer -- iterative end-to-end implementation (fork)
    consolidate/SKILL.md      # /consolidate -- backlog review (fork)
    session-health/SKILL.md   # /session-health -- session diagnostic (inline)
    handoff/SKILL.md          # /handoff -- session transition (inline)
    review/SKILL.md           # /review -- code review (fork)
    retro/SKILL.md            # /retro -- session retrospective (inline)
  commands/                   # Legacy fallbacks (same content as skills)
  settings.json               # Global hooks, env vars, and feature flags
  mcp-servers.json            # MCP server definitions (installed globally by install.sh)
  install.sh                  # Symlink installer (idempotent, non-destructive)
  CLAUDE.md                   # Context file for working on this repo itself
```

## Getting Started

After installing, here's how to go from zero to productive:

**On a new project:**

1. Run the project bootstrapper agent to set up beads, CLAUDE.md, hooks, permissions, and rules
2. Run the agent generator to create project-specific agents tailored to the codebase
3. Use `/blossom` to explore any area you want to understand or improve

**On an existing project:**

1. Type `/blossom <what you want to explore>` and let it map the territory
2. Use `/consolidate` whenever the backlog feels noisy
3. Use `/session-health` when you've been working a while and want a gut check

**Adding to a new machine:**

```bash
git clone <this-repo>
cd meta-agent-defs && ./install.sh
```

Your entire Claude Code workflow travels with you.

## Extending

Add new agents to `agents/`, new commands to `commands/`, new MCP servers to `mcp-servers.json`, or modify `settings.json` for new hooks. Rerun `./install.sh` to pick up new files. Commit and push to share across machines.

The real power comes from layering: these global definitions provide the workflow skeleton, while per-project `.claude/agents/` (created by the agent generator) provide project-specific intelligence. Global workflows, local expertise.
