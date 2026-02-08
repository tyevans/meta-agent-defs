# meta-agent-defs V5

Portable, symlink-installable Claude Code workflow definitions. Maintains your global agents, commands, hooks, and settings in a single git repo.

## What's Included

```
meta-agent-defs/
├── agents/
│   ├── agent-generator.md      # Generates project-specific agents from codebase analysis
│   └── project-bootstrapper.md # Bootstraps new projects with full Claude Code setup
├── commands/
│   └── blossom.md              # Spike-driven exploration workflow (/blossom)
├── settings.json               # Global hooks + env (SessionStart, PreCompact, guards)
├── install.sh                  # Symlink installer
└── README.md
```

## Install

```bash
git clone <this-repo> ~/workspace/meta-agent-defs
cd ~/workspace/meta-agent-defs
chmod +x install.sh
./install.sh
```

Re-running `install.sh` is idempotent. Existing regular files are backed up with timestamps.

## Uninstall

```bash
# Remove all symlinks pointing to this repo
find ~/.claude -type l -lname "$HOME/workspace/meta-agent-defs/*" -delete
```

## How It Works

The installer creates symlinks from `~/.claude/` into this repo:

```
~/.claude/agents/agent-generator.md      -> meta-agent-defs/agents/agent-generator.md
~/.claude/agents/project-bootstrapper.md -> meta-agent-defs/agents/project-bootstrapper.md
~/.claude/commands/blossom.md            -> meta-agent-defs/commands/blossom.md
~/.claude/settings.json                  -> meta-agent-defs/settings.json
```

Edit files in this repo, and changes are live immediately via the symlinks.

## V5 Key Features

### Agents

- **Agent Generator V5**: Generates project agents + supporting hooks + agent catalog (`AGENTS.md`). Every agent gets Investigation Protocol, Context Management, and Knowledge Transfer sections.
- **Project Bootstrapper V5**: Full 10-phase bootstrap — beads, CLAUDE.md, hooks, permissions, rules, memory directory, blossom command, initial tasks.

### Commands

- **Blossom V2**: 6-phase spike-driven exploration (Seed -> Spikes -> Consolidate -> Prioritize -> Verify -> Report). Confidence levels (CONFIRMED/LIKELY/POSSIBLE), vertical slice audits, critical path identification.

### Hooks (via settings.json)

| Hook | Trigger | Purpose |
|------|---------|---------|
| SessionStart | Session begins | Load beads context (`bd prime`) |
| PreCompact | Before compaction | Save beads state (`bd sync --flush-only`) |
| PreToolUse (Bash) | Before git push | Warn if beads changes uncommitted |
| PostToolUse (Task) | After agent completes | Review gate reminder |

### Design Principles

- **Hook-Rule-Skill boundary**: Hooks = must happen 100%. Rules = should usually happen. Skills = on-demand knowledge.
- **Serialized dispatching**: One agent at a time. Review output before dispatching next.
- **Spike confidence levels**: CONFIRMED > LIKELY > POSSIBLE. Read the code, don't guess.
- **Epic depends on children**: `bd dep add <epic> <child>`, never the reverse.

## Adding New Artifacts

1. Add files to the appropriate directory (`agents/`, `commands/`, etc.)
2. Re-run `./install.sh` to create new symlinks
3. Commit and push

## Extending to New Machines

```bash
# On a new machine
git clone <this-repo> ~/workspace/meta-agent-defs
cd ~/workspace/meta-agent-defs && ./install.sh
```

Your full Claude Code workflow is now portable across machines.
