# Reference

Detailed reference material for the Tackline toolkit. For a quick overview, see the [README](../README.md). For skill and agent details, see the [INDEX](INDEX.md).

## Hooks and Safety

Hooks are configured in `~/.claude/settings.json` (managed by the user, not by this repo) and run automatically across all projects.

| Hook | When | What |
|------|------|------|
| SessionStart | Every new session | Pre-session diagnostics (dirty tree, unpushed commits) |
| PreCompact | Before context compaction | Flushes session state so nothing is lost |
| PreToolUse (Bash) | Before destructive commands | Warns on `git reset --hard`, `git checkout .`, `git clean -f`, `rm -rf` |
| PostToolUse (Task) | After any subagent completes | Review gate reminder: verify deliverable before moving on |

All hooks fail gracefully with `|| true` for optional tools.

## MCP Servers

Four Model Context Protocol servers installed globally via `claude mcp add`.

| Server | What |
|--------|------|
| `playwright` | Browser automation for testing and scraping |
| `sequential-thinking` | Structured multi-step reasoning |
| `context7` | Up-to-date library documentation lookup |
| `memory` | Persistent knowledge graph across sessions |

MCP servers are defined in `mcp-servers.json` and installed via the plugin system.

## Project Structure

```
tackline/
  agents/
    agent-generator.md        # Generates project-specific agents from codebase analysis
    project-bootstrapper.md   # Full project setup for Claude Code
    code-reviewer.md          # Read-only code review across 4 quality dimensions
  skills/                     # 46 skills (symlinked to ~/.claude/skills/)
    active-learn/SKILL.md     # /active-learn -- adversarial training loop (fork)
    advise/SKILL.md           # /advise -- proactive session recommendations (inline)
    assemble/SKILL.md         # /assemble -- persistent team creation (inline)
    assess/SKILL.md           # /assess -- categorical rubric evaluation (inline)
    blossom/SKILL.md          # /blossom -- spike-driven exploration (fork)
    bootstrap/SKILL.md        # /bootstrap -- full project setup (fork)
    bug/SKILL.md              # /bug -- file structured bug reports (inline)
    challenge-gen/SKILL.md    # /challenge-gen -- generate training challenges (inline)
    challenge-run/SKILL.md    # /challenge-run -- execute and evaluate challenges (inline)
    consensus/SKILL.md        # /consensus -- multi-perspective design synthesis (fork)
    consolidate/SKILL.md      # /consolidate -- backlog review (fork)
    critique/SKILL.md         # /critique -- surface weaknesses (inline)
    curate/SKILL.md           # /curate -- score and optimize learnings/rules (inline)
    decompose/SKILL.md        # /decompose -- break into sub-parts (inline)
    diagnose-agent/SKILL.md   # /diagnose-agent -- agent struggle profile (inline)
    diff-ideas/SKILL.md       # /diff-ideas -- side-by-side tradeoffs (inline)
    discover/SKILL.md         # /discover -- recommend skills for a goal (inline)
    distill/SKILL.md          # /distill -- condense to essentials (inline)
    domain/SKILL.md           # /domain -- project terminology (inline)
    drift/SKILL.md            # /drift -- cross-definition convergence/divergence (inline)
    evolution/SKILL.md        # /evolution -- file change history and stability (inline)
    expand/SKILL.md           # /expand -- elaborate sparse items (inline)
    filter/SKILL.md           # /filter -- binary keep/drop (inline)
    fractal/SKILL.md          # /fractal -- goal-directed recursive exploration (inline)
    gather/SKILL.md           # /gather -- collect structured findings (inline)
    handoff/SKILL.md          # /handoff -- session transition (inline)
    meeting/SKILL.md          # /meeting -- interactive multi-agent dialogue (inline)
    merge/SKILL.md            # /merge -- combine pipe outputs (inline)
    plan/SKILL.md             # /plan -- ordered action plan (inline)
    premortem/SKILL.md        # /premortem -- risk-first failure analysis (fork)
    promote/SKILL.md          # /promote -- graduate learnings to rules (inline)
    rank/SKILL.md             # /rank -- score and order items (inline)
    retro/SKILL.md            # /retro -- session retrospective (inline)
    review/SKILL.md           # /review -- code review (fork)
    session-health/SKILL.md   # /session-health -- session diagnostic (inline)
    sketch/SKILL.md           # /sketch -- code skeleton with TODOs (inline)
    spec/SKILL.md             # /spec -- structured specification (fork)
    sprint/SKILL.md           # /sprint -- sprint planning + dispatch (inline)
    standup/SKILL.md          # /standup -- team status sync (inline)
    status/SKILL.md           # /status -- unified system dashboard (inline)
    team-meeting/SKILL.md     # /team-meeting -- goal-oriented team planning (inline)
    tend/SKILL.md             # /tend -- full learning lifecycle (inline)
    test-strategy/SKILL.md    # /test-strategy -- TDD/BDD workflow (fork)
    tracer/SKILL.md           # /tracer -- iterative end-to-end implementation (fork)
    transform/SKILL.md        # /transform -- rewrite each item independently (inline)
    verify/SKILL.md           # /verify -- fact-check claims (inline)
  docs/
    INDEX.md                  # Skill & agent navigator (decision tree, categories)
    design-notes.md           # Architecture and design decision notes
    domains.md                # Domain-to-artifact mapping
    pipelines.md              # Canonical end-to-end lifecycle pipelines
    primitives-cookbook.md     # Annotated walkthroughs of primitive chains
    primitives-recipes.md     # Common chain patterns
    team-system-guide.md      # Standalone adopter guide for team system
  demos/
    todo-app/                 # Intentionally buggy Express app for demo walkthroughs
  dev/
    lint.sh                   # Linting script
    new-agent.sh              # Scaffold a new agent definition
    new-skill.sh              # Scaffold a new skill definition
  rules/                      # Global rules (symlinked to ~/.claude/rules/)
    batch-safety.md           # Batch processing safety (chunk at 12 items)
    context-trust.md          # Trust user-provided context, don't re-investigate
    information-architecture.md  # IA principles for knowledge organization
    memory-layout.md          # Path registry for persistent state
    pipe-format.md            # Composable primitive output contract
    team-protocol.md          # Team manifest, spawn protocol, reflection schema
    test-conventions.md       # Testing conventions
  templates/
    teams/                    # Starter team.yaml files for common project types
  .claude/tackline/memory/    # Persistent state (sessions, learnings, epics)
  mcp-servers.json            # MCP server definitions (installed via plugin system)
  CLAUDE.md                   # Context file for working on this repo itself
  CONTRIBUTING.md             # Contribution guidelines
```

## Design Philosophy

Three principles shaped these workflows:

**1. Hooks are for guarantees, instructions are for guidance.** Hooks fire 100% of the time -- use them for things that must never be forgotten (syncing state, review gates). CLAUDE.md instructions are for things that should usually happen but require judgment. Skills are for on-demand workflows you invoke when you need them.

**2. Verify, don't speculate.** Every spike agent in `/blossom` is instructed to read the actual implementation, trace call chains, check callers and tests, and state a confidence level. CONFIRMED means the agent read the code and verified the issue. LIKELY means strong evidence but incomplete trace. POSSIBLE means it needs a deeper spike. This distinction matters -- it's the difference between a list of real work and a list of guesses.

**3. Parallelize with worktree isolation by default.** Independent tasks are dispatched concurrently, each agent in its own git worktree. This eliminates merge conflicts and context bloat. Serial dispatch is the fallback for tasks with true sequential dependencies — where each task needs the previous one's output. Agent teams are reserved for when agents must communicate mid-execution.
