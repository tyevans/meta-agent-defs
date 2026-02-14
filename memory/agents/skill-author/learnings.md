# Learnings: skill-author

## Codebase Patterns
- Skills live in skills/<name>/SKILL.md with YAML frontmatter (name, description, allowed-tools, context)
- 15 skills currently: assemble, blossom, consensus, consolidate, fractal, handoff, meeting, premortem, retro, review, session-health, spec, sprint, standup, tracer
- Skills with `context: fork` run in isolation (blossom, consolidate, review); others run inline
- `disable-model-invocation: true` prevents auto-invocation; most skills use this
- `$ARGUMENTS` is how skills receive user input from the slash command
- 6 fully compliant reference templates: consensus, premortem, retro, review, spec, tracer (added: 2026-02-13)
- Skills that dispatch agents (blossom, consensus, premortem, spec) correctly follow fan-out-protocol.md (added: 2026-02-13)

## Gotchas
- Skills cannot be invoked by subagents (Skill tool not available to them)
- `allowed-tools` uses `Bash(prefix:*)` syntax to restrict shell commands, not full command strings
- RESOLVED: user-invocable and "When to Use" gaps were fixed in commit 1fcc5d3 (2026-02-13)

## Preferences
- (none yet)

## Cross-Agent Notes
- (none yet)
