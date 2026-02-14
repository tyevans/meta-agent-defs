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
- Skills can reference allowed-tools that might not be available at runtime; conditional logic in the body handles optional deps, not frontmatter (added: 2026-02-13)
- Optional dependency pattern: `**If .beads/ exists**, [action]. **If not**, [fallback].` — preserves structure while enabling graceful degradation (added: 2026-02-13)

## Composition Patterns
- Gather is almost always first — other primitives need structured items to work on (added: 2026-02-13)
- Assess categorizes (discrete rubric), rank orders (continuous score) — different tools for different needs (added: 2026-02-13)
- Verify belongs at chain ends for confirmation, not at the start for discovery (added: 2026-02-13)
- Decompose bridges analysis and execution — it's the step before task creation / agent dispatch (added: 2026-02-13)
- Cookbook examples must be grounded in real repo content — verify patterns exist before writing examples (added: 2026-02-13)
- 3 deep recipes beat 5 shallow — annotations need 120+ lines per recipe for full context (added: 2026-02-13)

## Preferences
- (none yet)

## Cross-Agent Notes
- Team templates in templates/teams/ follow exact .claude/team.yaml schema; use realistic ownership globs per project type (added: 2026-02-13)
