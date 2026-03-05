---
strength: should
paths: agents/**
---

# Sandbox Execution

Conventions for agents running inside Coder workspaces or other sandboxed environments.

## Detection

Check for sandbox context before assuming local execution:
- `CODER_AGENT_TOKEN` env var = running inside a Coder workspace
- `/workspace` as root working directory = containerized sandbox
- `coder` CLI available = can manage sibling workspaces

## Workspace Conventions

- **Working directory**: `/workspace/repo` (not `~` or `/home/user`)
- **Persistent storage**: `/workspace` volume survives workspace stop/restart
- **Ephemeral compute**: The container itself is recreated on start
- **Network**: May be restricted by Agent Boundaries — assume outbound HTTP is filtered

## Lifecycle Awareness

- Workspaces have a TTL (default 2 hours of inactivity)
- Long-running tasks should checkpoint progress to `/workspace` (persistent volume)
- Use `coder` CLI to extend deadline if more time is needed: `coder schedule extend --hours 1`

## Cross-Workspace Execution

When an agent needs to spawn work in another sandbox:
1. Use Coder MCP tools (if available) to create a workspace
2. Execute commands via `coder ssh <workspace> -- <command>`
3. Collect results and stop the workspace when done

## What NOT to Do

- Do not assume `~/.claude/` exists — use `/workspace/.claude/` in sandbox context
- Do not assume persistent network — checkpoint before long waits
- Do not store secrets in the workspace volume — use env vars injected by the template
