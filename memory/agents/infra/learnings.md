# Learnings: infra

## Codebase Patterns
- install.sh is idempotent, supports symlinks (default) and hardlinks (--hardlink), global (~/.claude/) and project-local (./install.sh <dir>)
- settings.json at repo root is the global settings file, symlinked to ~/.claude/settings.json
- .claude/settings.json is project-local (not symlinked globally)
- Hooks in settings.json: SessionStart, PreCompact, PreToolUse, PostToolUse
- mcp-servers.json defines MCP servers installed globally via `claude mcp add --scope user`
- MCP config lives in ~/.claude.json (not symlinked)

## Gotchas
- All hooks use `|| true`, making failures silent -- by design for optional tools like bd
- Hook precedence between global and project-local settings.json is UNTESTED

## Codebase Patterns (new)
- Global SessionStart hook already detects .claude/team.yaml and displays team context prompt (added: 2026-02-13)
- memory/ directory is project-local, NOT handled by install.sh -- correct by design (added: 2026-02-13)
- .claude/team.yaml and memory/ are both committed to git, not ignored (added: 2026-02-13)
- install.sh writes .meta-agent-defs.manifest for reliable uninstall (both symlink and hardlink modes) (added: 2026-02-13)

## Preferences
- (none yet)

## Cross-Agent Notes
- (from agent-author) New global rules/ directory added — install.sh now symlinks rules/*.md to ~/.claude/rules/ following same pattern as agents/ and skills/ (added: 2026-02-13)
