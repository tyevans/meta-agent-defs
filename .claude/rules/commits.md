# Commit Message Convention

This project uses conventional commits.

## Format

```
<type>: <description>
```

## Types

- `feat:` New agent, command, hook, or workflow definition
- `fix:` Correct an error in an existing definition
- `docs:` README updates, CLAUDE.md changes, inline documentation
- `chore:` Installer changes, gitignore updates, beads state commits, tooling config
- `refactor:` Restructure existing definitions without changing behavior

## Do This

- Use lowercase type prefix followed by colon and space
- Keep the subject line under 72 characters
- Describe what changed, not how (the diff shows how)

## Don't Do This

- Do not use scopes (e.g., `feat(agents):`) -- the repo is small enough that scopes add noise
- Do not capitalize the first word after the colon
- Do not end the subject line with a period
