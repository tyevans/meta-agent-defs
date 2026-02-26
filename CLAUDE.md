# CLAUDE.md

Portable Claude Code workflow definitions (agents, skills, hooks, settings) maintained in a single git repo and installed to `~/.claude/` via symlinks.

## Operating Mode: Orchestrator

**Posture depends on project type:**
- **Content-only projects** (no `package.json`, `Cargo.toml`, `go.mod`, `pyproject.toml`, or `Makefile` at root): prefer direct implementation over subagent dispatch for simple edits.
- **Code projects**: orchestrator-only. Dispatch all implementation to subagents and manage the beads backlog.

This repo itself (tackline) is content-only -- direct edits to `.md` and `.json` files are preferred over spawning agents for trivial changes.

### Orchestrator Responsibilities

1. **Backlog Management**: Use `bd` commands to triage, prioritize, and track issues
2. **Task Dispatch**: Delegate implementation work to appropriate subagents via the Task tool
3. **Coordination**: Manage dependencies between tasks, unblock work, review agent outputs
4. **Session Management**: Run `bd sync` before completing sessions

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
xargs rm -f < ~/.claude/.tackline.manifest

# Beads
bd stats                        # Backlog overview
bd ready                        # Available work
bd create --title="..." --type=task
bd create --type=epic --title="..."  # Create epic
bd create --type=task --parent=<epic-id> --title="..."  # Child of epic
bd children <epic-id>           # List children
bd epic status                  # Epic completion progress
bd epic close-eligible          # Auto-close finished epics
bd swarm validate <epic-id>     # Validate epic DAG
bd dep cycles                   # Detect dependency cycles
bd sync                         # Export to JSONL before session end
bd query "status=open AND priority<=1"  # Find urgent open work
bd stale                        # Find forgotten issues
```

## Skill Quick Reference

| I want to... | Use |
|---|---|
| Explore something unknown | /blossom or /fractal |
| Research + prioritize | /gather -> /distill -> /rank |
| Compare approaches | /diff-ideas or /consensus |
| Plan before building | /decompose -> /plan -> /spec |
| Test an implementation | /test-strategy |
| Review code | /review |
| Track definition changes | /evolution or /drift |
| Run a session | /status -> ... -> /retro -> /handoff |
| Manage a team | /assemble -> /standup -> /sprint |
| Discuss with panels | /meeting |
| Plan a goal with your team | /team-meeting |
| Optimize agent learnings | /curate or /tend (curate + promote) |

All 46 skills: see [docs/INDEX.md](docs/INDEX.md). Composable primitives follow [pipe format](rules/pipe-format.md).

## Project Structure

```
tackline/
├── agents/
│   ├── agent-generator.md      # Generates project-specific agents
│   ├── project-bootstrapper.md # Bootstraps projects with full Claude Code setup
│   └── code-reviewer.md        # Read-only code review agent
├── bin/
│   └── rule-relevance.sh       # SessionStart hook: matches rules to file activity via paths: frontmatter
├── skills/                      # Skill definitions (flattened to ~/.claude/skills/ on install)
│   ├── core/                    # Composable primitives + entrypoints (16 skills)
│   │   ├── gather/, distill/, rank/, filter/, assess/, verify/
│   │   ├── expand/, transform/, decompose/, critique/, plan/
│   │   ├── merge/, diff-ideas/, sketch/
│   │   └── do/, discover/       # Skill routing entrypoints
│   ├── workflows/               # Exploration, review, lifecycle, session mgmt (22 skills)
│   │   ├── blossom/, fractal/, consolidate/, review/, bootstrap/
│   │   ├── meeting/, consensus/, premortem/, spec/, bug/
│   │   ├── advise/, handoff/, status/, session-health/, domain/
│   │   ├── evolution/, drift/, test-strategy/, tracer/
│   │   └── challenge-gen/, challenge-run/, diagnose-agent/
│   └── teams/                   # Team orchestration + learning lifecycle (9 skills)
│       ├── assemble/, standup/, sprint/, team-meeting/, active-learn/
│       └── curate/, promote/, tend/, retro/
├── docs/                        # Documentation (cookbook, recipes, team guide, INDEX)
│   ├── INDEX.md                 # Skill & agent navigator (decision tree, categories)
│   ├── design-notes.md          # Architecture and design decision notes
│   ├── domains.md               # Domain-to-artifact mapping
│   └── pipelines.md             # Canonical end-to-end lifecycle pipelines
├── demos/                       # Demo projects for primitive walkthroughs
├── dev/                         # Developer scripts
│   ├── lint.sh                  # Linting script
│   ├── new-agent.sh             # Scaffold a new agent definition
│   └── new-skill.sh             # Scaffold a new skill definition
├── rules/                       # Global rules (symlinked to ~/.claude/rules/)
│   ├── batch-safety.md          # Batch processing safety (chunk at 12 items)
│   ├── context-trust.md         # Trust user-provided context
│   ├── information-architecture.md  # IA principles for knowledge organization
│   ├── memory-layout.md         # Path registry for persistent state (paths: memory/**)
│   ├── pipe-format.md           # Composable primitive output contract (paths: skills/**/SKILL.md)
│   ├── team-protocol.md         # Team manifest, spawn protocol, reflection schema
│   └── test-conventions.md      # Testing conventions
├── memory/                      # Persistent state (sessions, learnings, epics)
├── templates/                   # Team templates (symlinked to ~/.claude/templates/)
│   └── teams/                   # Starter team.yaml files for common project types
├── mcp-servers.json            # MCP server definitions (installed globally)
├── install.sh                  # Symlink installer (idempotent)
├── .claude/                    # Project-local Claude Code config (NOT symlinked globally)
│   ├── AGENTS.md               # Agent catalog for project-local agents
│   ├── agents/                 # 8 project-local agents (authoring, research, maintenance)
│   ├── rules/                  # Architectural guardrails
│   └── skills/                 # Project-local skill overrides
├── CONTRIBUTING.md             # Contribution guidelines
└── .beads/                     # Task management state
```

## Architecture

- **No source code, no build system, no tests.** This is a content-only repo of Markdown definitions and JSON config.
- `install.sh` is idempotent. It backs up existing regular files before symlinking.
- `mcp-servers.json` defines MCP servers installed globally via `claude mcp add --scope user`. Config lives in `~/.claude.json` (not symlinked).

## Key Patterns

- All artifact files are Markdown with YAML frontmatter (agents, skills, rules)
- Agent frontmatter fields: `name`, `description`, `tools`, `model`, `permissionMode`
- Skill frontmatter fields: `name`, `description`, `allowed-tools`, `context`, `disable-model-invocation`
- Rule frontmatter fields: `strength` (must/should/may), `freshness` (date); some rules also have `paths` (glob patterns for conditional loading via `bin/rule-relevance.sh`)
- Hooks fail gracefully with `|| true` for optional tools (like `bd`)
- Epic hierarchy via `--parent`: `bd create --parent=<epic-id>` (not `bd dep add`). Use `bd dep add` only for cross-task ordering
- Confidence levels for spike findings: CONFIRMED > LIKELY > POSSIBLE
- If `memory/project/domain.md` exists, it contains project-specific terminology; consult it when a term is ambiguous

## Do Not Modify

- `.beads/` internals (use `bd` commands only)
- Symlink targets while symlinks are active (edit source files in this repo instead)
