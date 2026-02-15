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

- SessionStart hook has 3 logical sections: pulse card → last session context → bd prime (added: 2026-02-14)
- SessionEnd hook writes memory/sessions/last.md before running bd sync (added: 2026-02-14)
- `bd ready --limit 1` exists and works for mechanical "next task" suggestions (added: 2026-02-14)
- Global SessionStart hook already detects .claude/team.yaml and displays team context prompt (added: 2026-02-13)
- memory/ directory is project-local, NOT handled by install.sh -- correct by design (added: 2026-02-13)
- .claude/team.yaml and memory/ are both committed to git, not ignored (added: 2026-02-13)
- install.sh writes .meta-agent-defs.manifest for reliable uninstall (both symlink and hardlink modes) (added: 2026-02-13)

## Preferences
- bin/git-pulse.sh is the shared entry point for git session metrics — skills should call it instead of raw git log (added: 2026-02-14)
- Bash parameter expansion `${VAR:+"$VAR"}` is essential for optional flags with spaces in git commands (added: 2026-02-14)
- Optional tool delegation pattern: four-gate check — binary exists + deps (jq) + input compatible (ISO date) + runtime feature validation (for optional features like ML). Runtime test catches shared-lib-missing failures that build-time detection misses. (updated: 2026-02-15)
- ONNX model canonical path: tools/data/onnx-model/model.onnx — git-pulse.sh auto-detects and passes --ml --model-dir to git-intel (added: 2026-02-15)
- install.sh optional build features: use `--skip-*` flags for CI, interactive prompt for local, never block install on missing toolchain (added: 2026-02-14)

## Cross-Agent Notes
- rules/ follows the global-portable symlink pattern (like agents/, skills/). Three-way symmetry: portable directory + install.sh symlink logic + .claude/ project override. Replicate this pattern when adding new global config categories. (triaged: 2026-02-14)
