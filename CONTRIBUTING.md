# Contributing to Tackline

Thanks for your interest in contributing. This guide covers the basics.

## Project Structure

Tackline is a **content-only repo** -- Markdown definitions and JSON config, no source code. The main artifact types:

- **Skills** (`skills/<name>/SKILL.md`) -- Workflow definitions invoked via `/name`
- **Agents** (`agents/<name>.md`) -- Agent definitions with YAML frontmatter
- **Rules** (`rules/<name>.md`) -- Global behavioral constraints
- **Settings** (`settings.json`) -- Hooks and environment variables

## Adding a Skill

1. Create `skills/<name>/SKILL.md` with YAML frontmatter:
   ```yaml
   ---
   name: your-skill
   description: What the skill does and when to use it
   allowed-tools: [Read, Grep, Glob, Write, Edit, Bash, Task]
   context: fork  # or omit for inline
   ---
   ```
2. Write the skill body with clear phase structure
3. Run `./install.sh` to verify symlink creation
4. Update `docs/INDEX.md` with the new skill entry

## Adding an Agent

1. Create `agents/<name>.md` with YAML frontmatter:
   ```yaml
   ---
   name: your-agent
   description: When to use this agent
   tools: [Read, Grep, Glob]
   model: sonnet
   ---
   ```
2. Include Investigation Protocol, Context Management, and Knowledge Transfer sections
3. Run `./install.sh` to verify symlink creation

## Commit Messages

We use [conventional commits](https://www.conventionalcommits.org/):

```
feat: add /your-skill for doing X
fix: correct template path in bootstrap agent
docs: update INDEX.md with new skill counts
chore: update gitignore patterns
refactor: simplify sprint dispatch logic
```

- Lowercase type prefix, no scope, no trailing period
- Subject line under 72 characters
- Describe what changed, not how

## Testing Changes

Since this is a content-only repo, "testing" means running your skill or agent in a live Claude Code session:

1. Run `./install.sh` to symlink your changes
2. Open a Claude Code session in any project
3. Invoke your skill (e.g., `/your-skill`) and verify it works
4. Run `dev/lint.sh` to check frontmatter and JSON validity

## Pull Requests

- Keep PRs focused -- one skill, one agent, or one fix per PR
- Include a brief description of what the change does and why
- If adding a new skill, mention how you tested it

## Questions?

Open an issue or start a discussion.
