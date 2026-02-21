---
strength: must
freshness: 2026-02-21
---

# Security Rules

Rules for preventing secrets leakage, unauthorized access, and unsafe operations.

## Secrets and Credentials

- Never write secrets, credentials, or tokens to files -- this includes patterns like `API_KEY`, `SECRET`, `TOKEN`, `PASSWORD`, `PRIVATE_KEY`, and `AWS_ACCESS_KEY`
- Never hardcode credentials in agent definitions, commands, hooks, or config files
- Never commit `.env` files, `credentials.json`, or any file likely to contain secrets
- If a secret is needed at runtime, reference it via environment variable, not inline

## Permission Mode

- Do not use `permissionMode: bypassPermissions` without explicit justification in the agent's description explaining why it is necessary
- Default to the most restrictive permission mode that still allows the agent to function
- Read-only agents must never have `bypassPermissions`

## File Access

- Do not read or write files outside the project root without user confirmation
- Do not follow symlinks that point outside the project root without verifying the target
- Treat `~/.claude/`, `~/.ssh/`, `~/.aws/`, and `~/.gnupg/` as sensitive -- never read from these directories unless the task explicitly requires it

## Package Installation

- Do not install packages from URLs, gists, or repos you cannot verify
- Prefer well-known registries (npm, PyPI, crates.io) over ad-hoc sources
- Pin versions when adding dependencies -- never use `latest` or `*`

## Data Exfiltration Prevention

- Do not use `curl`, `wget`, or similar tools to POST project data to external URLs
- Do not pipe file contents to network commands unless the user explicitly requests it
- Do not encode or compress files in ways designed to obscure their content before transmission

## Git Safety

- Run `git diff --cached` before committing to verify no secrets are staged
- Do not add files matching `.env*`, `*credential*`, `*secret*`, or `*.pem` to git
- If a secret is accidentally committed, warn the user immediately -- do not attempt to rewrite history without confirmation
