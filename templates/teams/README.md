# Team Templates

Starter `team.yaml` files for common project types. These templates help you quickly scaffold a team for your project using `/assemble`.

## Available Templates

- **web-fullstack.yaml** — Frontend + Backend + QA/Test roles for web applications
- **library.yaml** — Core + Docs + Release roles for library development
- **monorepo.yaml** — Platform/Infra + package owners for monorepo projects
- **data-pipeline.yaml** — Data engineer + ML/Analytics + Infra roles for data projects

## Usage

1. Copy a template to your project:
   ```bash
   cp ~/.claude/templates/teams/web-fullstack.yaml /path/to/project/.claude/team.yaml
   ```

2. Customize the template:
   - Replace `<project-name>` with your actual project name
   - Adjust ownership globs to match your file structure
   - Add/remove tools based on your workflow needs
   - Set budget and permission-mode to match your preferences

3. Assemble your team:
   ```bash
   cd /path/to/project
   /assemble
   ```

## Schema Reference

```yaml
team: <project-name>
description: "<team description>"

defaults:
  model: sonnet|opus|haiku
  budget: 0.50           # Cost limit per agent
  permission-mode: dontAsk|delegate|plan

members:
  - name: <role-name>
    role: "<role description>"
    tools: [Read, Write, Edit, Grep, Glob, "Bash(...)"]
    owns: ["<glob patterns>"]
```

## Notes

- Templates do NOT assume beads is available — teams work with or without it
- Ownership globs are used by agents to determine their areas of responsibility
- Tools list defines what each agent can do (restrict for security/focus)
- Budget prevents runaway costs (agents stop when budget is exceeded)
