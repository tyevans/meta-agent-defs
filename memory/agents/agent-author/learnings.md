# Learnings: agent-author

## Codebase Patterns
- Global agents in agents/*.md (3): agent-generator, code-reviewer, project-bootstrapper
- Project-local agents in .claude/agents/*.md (8): agent-author, definition-tester, effectiveness-auditor, installer-maintainer, pattern-researcher, settings-editor, skill-author, sync-auditor
- Agent frontmatter: name, description, tools, model, permissionMode
- Rules in .claude/rules/*.md (7): agent-authoring, agent-memory, commits, definition-of-done, fan-out-protocol, hook-authoring, security, skill-authoring
- Global rules in rules/*.md (3): team-protocol, pipe-format, memory-layout — symlinked to ~/.claude/rules/ by install.sh (added: 2026-02-14)
- Rules use `paths:` frontmatter to scope which files they apply to
- Memory protocol uses tiered structure: Core (30 lines, high-reuse fundamentals) + Task-Relevant (30 lines, context-specific). Consolidation triggers at 60 lines with 4 mechanisms: merge, archive (>21 days), promote (3+ sprints), validate cross-agent notes (14-day window) (added: 2026-02-13)

## Gotchas
- STALE (was: "Global agents violate authoring rules") -- as of 2026-02-13 audit, all 11 agents are fully compliant
- Root AGENTS.md name collides with .claude/AGENTS.md -- be careful with file placement
- Rules without `paths:` frontmatter (commits, security, definition-of-done) are intentionally global-scope (added: 2026-02-13)
- Retro skill uses 50-line warning + 60-line hard cap for learnings (not the old 120/150 thresholds) (added: 2026-02-13)

## Preferences
- Documentation guides benefit from concrete examples at every level: complete config files, sample outputs, before/after comparisons (added: 2026-02-13)

## Cross-Agent Notes
- install.sh's artifact installation follows a replicable pattern: create target directory, skip in project-local mode with messaging, include in stale cleanup, display in summary. Apply this pattern consistently when adding new global artifact types. (triaged: 2026-02-14)
