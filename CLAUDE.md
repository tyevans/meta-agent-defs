# CLAUDE.md

Portable Claude Code workflow definitions (agents, skills, hooks, settings) maintained in a single git repo and installed to `~/.claude/` via symlinks.

## Operating Mode: Orchestrator

**The primary Claude Code session operates as an orchestrator only.** Do not directly implement tasks -- dispatch work to specialized subagents and manage the beads backlog.

### Orchestrator Responsibilities

1. **Backlog Management**: Use `bd` commands to triage, prioritize, and track issues
2. **Task Dispatch**: Delegate implementation work to appropriate subagents via the Task tool
3. **Coordination**: Manage dependencies between tasks, unblock work, review agent outputs
4. **Session Management**: Run `bd sync --flush-only` before completing sessions

### Dispatching Strategy

**Default: serialize.** Dispatch one task at a time, review output, then dispatch next. This avoids API throttling and lets each task benefit from the last one's findings.

**When teams are enabled: parallelize.** Use agent teams for independent tasks (e.g., blossom spikes, parallel audits). Teams run in separate contexts so throttling and context bloat don't apply.

---

## Quick Reference

```bash
# Install (global symlinks to ~/.claude/)
./install.sh

# Install to a specific project
./install.sh /path/to/project

# Install with hardlinks instead of symlinks
./install.sh --hardlink

# Uninstall (uses manifest written during install)
xargs rm -f < ~/.claude/.meta-agent-defs.manifest

# Beads
bd stats                        # Backlog overview
bd ready                        # Available work
bd create --title="..." --type=task
bd sync --flush-only            # Save state before session end
```

## Skill Quick Reference

| I want to... | Use |
|---|---|
| Explore something unknown | /blossom or /fractal |
| Research + prioritize | /gather -> /distill -> /rank |
| Compare approaches | /diff-ideas or /consensus |
| Plan before building | /decompose -> /plan -> /spec |
| Review code | /review |
| Track definition changes | /evolution or /drift |
| Run a session | /status -> ... -> /retro -> /handoff |
| Manage a team | /assemble -> /standup -> /sprint |
| Discuss with panels | /meeting |
| Plan a goal with your team | /team-meeting |

All 31 skills: see [docs/INDEX.md](docs/INDEX.md). Composable primitives follow [pipe format](rules/pipe-format.md).

## Project Structure

```
meta-agent-defs/
├── agents/
│   ├── agent-generator.md      # Generates project-specific agents
│   ├── project-bootstrapper.md # Bootstraps projects with full Claude Code setup
│   └── code-reviewer.md        # Read-only code review agent
├── bin/
│   └── git-pulse.sh            # Shared entry point for git session metrics
├── skills/                      # Skill definitions (symlinked to ~/.claude/skills/)
│   ├── blossom/SKILL.md         # Spike-driven exploration (context: fork)
│   ├── fractal/SKILL.md         # Goal-directed recursive exploration (inline)
│   ├── meeting/SKILL.md         # Interactive multi-agent dialogue (inline)
│   ├── assemble/SKILL.md        # Persistent learning team creation (inline)
│   ├── standup/SKILL.md         # Team status sync with learning health (inline)
│   ├── sprint/SKILL.md          # Sprint planning + dispatch with learning loop (inline)
│   ├── consolidate/SKILL.md     # Backlog review (context: fork)
│   ├── session-health/SKILL.md  # Session diagnostic (inline, auto-discoverable)
│   ├── handoff/SKILL.md         # Session transition (inline)
│   ├── review/SKILL.md          # Code review (context: fork)
│   ├── retro/SKILL.md           # Session retrospective (inline)
│   └── <12 composable primitives>  # gather, distill, rank, etc. (inline)
├── docs/                        # Documentation (cookbook, recipes, team guide, INDEX)
│   └── INDEX.md                 # Skill & agent navigator (decision tree, categories)
├── demos/                       # Demo projects for primitive walkthroughs
├── rules/                       # Global rules (symlinked to ~/.claude/rules/)
│   ├── team-protocol.md         # Team manifest, spawn protocol, reflection schema
│   ├── pipe-format.md           # Composable primitive output contract
│   ├── information-architecture.md  # IA principles for knowledge organization
│   └── memory-layout.md         # Path registry for persistent state
├── templates/                   # Team templates (symlinked to ~/.claude/templates/)
│   └── teams/                   # Starter team.yaml files for common project types
├── tools/
│   └── git-intel/               # Rust CLI for git metrics (metrics, churn, lifecycle, patterns)
├── settings.json               # Global hooks + env (symlinked to ~/.claude/)
├── mcp-servers.json            # MCP server definitions (installed globally)
├── install.sh                  # Symlink installer (idempotent)
├── .claude/                    # Project-local Claude Code config (NOT symlinked globally)
│   ├── settings.json           # Project-specific hooks + permissions
│   ├── AGENTS.md               # Agent catalog for project-local agents
│   ├── agents/                 # 8 project-local agents (authoring, research, maintenance)
│   ├── rules/                  # Architectural guardrails
│   └── skills/                 # Project-local skill overrides
└── .beads/                     # Task management state
```

## Architecture

- **No source code, no build system, no tests.** This is a content-only repo of Markdown definitions and JSON config.
- `settings.json` at repo root is the **global** settings file symlinked to `~/.claude/settings.json`. It contains hooks (SessionStart, PreCompact, PreToolUse, PostToolUse) and env vars that apply to ALL projects.
- `.claude/settings.json` is the **project-local** settings file for working on this repo itself.
- `install.sh` is idempotent. It backs up existing regular files before symlinking.
- `mcp-servers.json` defines MCP servers installed globally via `claude mcp add --scope user`. Config lives in `~/.claude.json` (not symlinked).

## Key Patterns

- All artifact files are Markdown with YAML frontmatter (agents, skills)
- Agent frontmatter fields: `name`, `description`, `tools`, `model`, `permissionMode`
- Skill frontmatter fields: `name`, `description`, `allowed-tools`, `context`, `disable-model-invocation`
- Hooks fail gracefully with `|| true` for optional tools (like `bd`)
- Epic depends on children: `bd dep add <epic> <child>`, never the reverse
- Confidence levels for spike findings: CONFIRMED > LIKELY > POSSIBLE

## Do Not Modify

- `.beads/` internals (use `bd` commands only)
- Symlink targets while symlinks are active (edit source files in this repo instead)
