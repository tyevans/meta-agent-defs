# Setup Guide

Step-by-step setup for Tackline. Covers installation, shell configuration, task management, and verification.

For configuring `~/.claude/settings.json` (permissions, hooks, environment variables), see the [Settings Guide](settings.md).

## Prerequisites

- **git** (required)
- **Claude Code CLI** (`claude`) — [install instructions](https://docs.anthropic.com/en/docs/claude-code)
- **Node.js 18+** — required by Claude Code

## 1. Clone and Install

```bash
git clone https://github.com/tyevans/tackline
cd tackline
./install.sh
```

The installer symlinks agents, skills, rules, and templates into `~/.claude/`. It's idempotent — rerun after pulling updates.

### Install options

| Flag | Effect |
|------|--------|
| `./install.sh /path/to/project` | Project-local install (agents + skills only, into `project/.claude/`) |
| `./install.sh --hardlink` | Hardlinks instead of symlinks (for filesystems that don't support symlinks) |
| `./install.sh --rules-only` | Rules + templates only (use alongside the Tackline plugin) |
| `./install.sh --tacks` | Require tacks (`tk`) instead of auto-detecting |

## 2. Shell Aliases

Add aliases to your shell profile (`~/.zshrc`, `~/.bashrc`, etc.) to launch Claude Code with your preferred defaults.

### Basic aliases

```bash
alias claudes="claude /sandbox"
alias clauded="claude /sandbox --dangerously-skip-permissions"
```

### Parallel session support

If you run multiple Claude Code sessions in the same project directory (e.g., four terminals each driving different tasks), their checkpoint files will collide. To prevent this, inject a unique instance ID per launch:

```bash
alias claudes='CLAUDE_INSTANCE_ID=$(uuidgen || cat /proc/sys/kernel/random/uuid) claude /sandbox'
alias clauded='CLAUDE_INSTANCE_ID=$(uuidgen || cat /proc/sys/kernel/random/uuid) claude /sandbox --dangerously-skip-permissions'
```

When `CLAUDE_INSTANCE_ID` is set, skills like `/drive` namespace their scratch files (e.g., `memory/scratch/drive-state-<id>.md`) so parallel sessions don't overwrite each other's state.

**When you need this:** You're running `/drive` or other long-running skills in multiple terminals against the same project. Each terminal gets its own checkpoint state.

**When you don't:** You only ever have one Claude session per project at a time. The basic aliases work fine.

## 3. Task Management

Tackline works without a task manager but is better with one. Hooks and skills use `bd` (beads) or `tk` (tacks) for backlog tracking.

### Option A: Tacks (recommended for new users)

```bash
cargo install tacks
```

Tacks is simpler — local SQLite, no sync step, no export. Good for solo work.

### Option B: Beads

See the [beads documentation](https://github.com/tyevans/beads) for installation. Beads adds structured queries (`bd query`), stale detection (`bd stale`), and DAG validation (`bd swarm validate`). Better for larger projects and team workflows.

### Option C: Neither

Everything still works. Skills that reference `bd`/`tk` degrade gracefully. You just won't get automated backlog tracking.

## 4. MCP Servers

The installer registers four MCP servers with the Claude CLI:

| Server | Purpose |
|--------|---------|
| `playwright` | Browser automation |
| `sequential-thinking` | Structured multi-step reasoning |
| `context7` | Library documentation lookup |
| `memory` | Persistent knowledge graph |

These are optional. If `claude` CLI isn't found during install, MCP registration is skipped. You can add them later:

```bash
cd tackline
./install.sh  # re-run to pick up MCP servers
```

## 5. Verify Installation

```bash
# Check symlinks
ls -la ~/.claude/skills/ | head -20
ls -la ~/.claude/agents/
ls -la ~/.claude/rules/

# Start a session and verify skills load
claude
# Then type: /discover explore a codebase
```

You should see skills resolve (e.g., `/discover` recommends `/blossom` or `/fractal`).

## 6. First Session

### New project

```bash
cd your-project
claude
```

Then:

```
/bootstrap
```

This sets up `CLAUDE.md`, hooks, rules, and generates project-specific agents by analyzing your codebase.

### Existing project

```
/blossom <area you want to explore>
```

Blossom dispatches spike agents to map the territory and produces a prioritized task backlog.

### Just exploring

```
/gather <topic>
/distill
/rank by <criteria>
```

The primitive chain pattern: collect findings, condense, prioritize. Each skill's output feeds the next through conversation context.

## Troubleshooting

### "Command not found: claude"

Install Claude Code CLI first. Tackline is a workflow layer on top of Claude Code.

### Skills don't appear

Rerun `./install.sh` and check that `~/.claude/skills/` contains symlinks pointing to your tackline clone.

### Stale symlinks after moving the repo

The installer cleans up stale symlinks on each run. Just rerun `./install.sh` from the new location.

### Hooks reference bd/tk but neither is installed

This is fine. All hooks use `|| true` to fail gracefully. Install a task manager when you're ready.

## Uninstall

```bash
xargs rm -f < ~/.claude/.tackline.manifest
```

This removes all symlinked files. Your tackline clone is untouched.
