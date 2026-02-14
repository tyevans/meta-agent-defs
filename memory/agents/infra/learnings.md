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
- When hooks share identical command strings, include the hook name key in edit context to disambiguate (added: 2026-02-14)
- Always validate JSON after settings.json edits: `python3 -c "import json; json.load(open(...))"` (added: 2026-02-14)

## Codebase Patterns (new)
- SessionStart hook has 3 logical sections: pulse card → last session context → bd prime (added: 2026-02-14)
- SessionEnd hook writes memory/sessions/last.md before running bd sync (added: 2026-02-14)
- `bd ready --limit 1` exists and works for mechanical "next task" suggestions (added: 2026-02-14)
- Global SessionStart hook already detects .claude/team.yaml and displays team context prompt (added: 2026-02-13)
- memory/ directory is project-local, NOT handled by install.sh -- correct by design (added: 2026-02-13)
- .claude/team.yaml and memory/ are both committed to git, not ignored (added: 2026-02-13)
- install.sh writes .meta-agent-defs.manifest for reliable uninstall (both symlink and hardlink modes) (added: 2026-02-13)

## Preferences
- (none yet)

## Cross-Agent Notes
- rules/ follows the global-portable symlink pattern (like agents/, skills/). Three-way symmetry: portable directory + install.sh symlink logic + .claude/ project override. Replicate this pattern when adding new global config categories. (triaged: 2026-02-14)
