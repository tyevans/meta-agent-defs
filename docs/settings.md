# Configuring settings.json

Claude Code reads settings from `~/.claude/settings.json` (user-level) and `project/.claude/settings.json` (project-level). This guide covers the settings that matter most for Tackline workflows.

Tackline's `install.sh` does **not** touch your settings.json — you own this file entirely. This guide gives you a starting point and explains what each section does.

---

## Starter Configuration

Copy this as a starting point and customize to your stack:

```json
{
  "permissions": {
    "allow": [
      "Bash(git status:*)",
      "Bash(git diff:*)",
      "Bash(git log:*)",
      "Bash(git show:*)",
      "Bash(git branch:*)",
      "Bash(git add:*)",
      "Bash(git commit:*)",
      "Bash(git mv:*)",
      "Bash(tree:*)"
    ],
    "defaultMode": "default"
  },
  "hooks": {}
}
```

This pre-approves read-only git commands and basic git operations so Claude doesn't ask permission every time it runs `git status`. Everything else still prompts.

---

## Permissions

### How permission patterns work

The `permissions.allow` array uses `Tool(pattern:*)` syntax. When Claude wants to run a command, it checks this list. If the command matches, it runs without prompting.

```json
"Bash(git log:*)"       // matches: git log --oneline -20, git log --all, etc.
"Bash(npm run test:*)"  // matches: npm run test, npm run test -- --watch, etc.
"Bash(docker compose:*)" // matches: docker compose up, docker compose down, etc.
```

### What to pre-approve

**Always safe to allow** — these are read-only or low-risk:

```json
"Bash(git status:*)",
"Bash(git diff:*)",
"Bash(git log:*)",
"Bash(git show:*)",
"Bash(git branch:*)",
"Bash(tree:*)"
```

**Allow if you trust your workflow** — these modify state but are standard development operations:

```json
"Bash(git add:*)",
"Bash(git commit:*)",
"Bash(git checkout:*)",
"Bash(git push:*)"
```

**Project-specific** — add your build tools, test runners, and task manager:

```json
"Bash(npm run:*)",        // Node.js
"Bash(cargo test:*)",     // Rust
"Bash(uv run pytest:*)",  // Python with uv
"Bash(just:*)",           // Justfile runner
"Bash(bd:*)",             // beads task manager
"Bash(tk:*)"              // tacks task manager
```

### What NOT to pre-approve

Don't allow broad patterns like `Bash(rm:*)`, `Bash(sudo:*)`, or `Bash(*)`. The permission prompt exists for a reason — it's the moment where you verify Claude isn't about to do something destructive.

### Permission modes

```json
"defaultMode": "default"  // Prompt for unapproved tools (recommended)
```

Other options: `"allowAll"` (no prompts — use only for throwaway environments) and `"deny"` (block everything not explicitly allowed).

---

## Environment Variables

The `env` block sets variables available to Claude and all hooks:

```json
"env": {
  "CLAUDE_CODE_ENABLE_TELEMETRY": "1"
}
```

### Tackline-relevant env vars

| Variable | What it does | Recommended |
|----------|-------------|-------------|
| `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` | Enables agent teams (TeamCreate, SendMessage) | `"1"` — required for `/meeting`, `/blossom` with teams |
| `ENABLE_TOOL_SEARCH` | Defers rarely-used tools to save context | `"auto:5"` — auto-defer, return up to 5 results |
| `CLAUDE_CODE_ENABLE_TELEMETRY` | Sends usage telemetry to Anthropic | Your choice |

#### Tool Search

Tool search defers MCP tools (playwright, memory, etc.) so they don't consume context window on every turn. With `"auto:5"`, tools load on demand when Claude needs them. This matters because Tackline installs several MCP servers — without tool search, every server's tools eat into your context budget from the start.

```json
"env": {
  "ENABLE_TOOL_SEARCH": "auto:5"
}
```

---

## Hooks

Hooks run shell commands at specific lifecycle points. Tackline's recommended hooks are installed in this section. See the [Reference](reference.md) for the full hook table.

### Hook types

| Hook | When it fires | Use for |
|------|---------------|---------|
| `SessionStart` | New session begins | Load context, check git state, prime backlog |
| `SessionEnd` | Session closes | Sync backlog, write session log |
| `PreCompact` | Before context compaction | Flush in-progress state to disk |
| `PreToolUse` | Before a tool executes | Safety checks, sync before push |
| `PostToolUse` | After a tool executes | Review gates, telemetry |
| `UserPromptSubmit` | After user sends a message | Inject domain context |

### Recommended hooks for Tackline

The hooks below are what Tackline is built around. Add them to your `settings.json` — or use them as a starting point and customize.

**SessionStart** — orient at the start of every session:

```json
"SessionStart": [
  {
    "matcher": "",
    "hooks": [
      {
        "type": "command",
        "command": "echo '=== Session Start ==='; git log --oneline -3 2>/dev/null | sed 's/^/Last:    /'; echo \"Tree:    $(git status --short 2>/dev/null | wc -l) uncommitted files\"; bd status --no-activity 2>/dev/null || tk stats 2>/dev/null || true; echo '=== ==='"
      }
    ]
  }
]
```

**PreToolUse** — warn on destructive commands:

```json
"PreToolUse": [
  {
    "matcher": "Bash",
    "hooks": [
      {
        "type": "command",
        "command": "INPUT=\"$CLAUDE_TOOL_INPUT\"; if echo \"$INPUT\" | grep -qE '\"git reset --hard' 2>/dev/null; then echo 'WARNING: git reset --hard will discard all uncommitted changes.' >&2; elif echo \"$INPUT\" | grep -qE '\"git checkout \\.' 2>/dev/null; then echo 'WARNING: git checkout . will discard all unstaged changes.' >&2; elif echo \"$INPUT\" | grep -qE '\"git clean -f' 2>/dev/null; then echo 'WARNING: git clean -f will permanently delete untracked files.' >&2; fi"
      }
    ]
  }
]
```

**PostToolUse** — review gate after subagent completion:

```json
"PostToolUse": [
  {
    "matcher": "Task",
    "hooks": [
      {
        "type": "command",
        "command": "echo 'REVIEW GATE: Agent completed. Verify deliverable before proceeding.' >&2"
      }
    ]
  }
]
```

---

## Plugins

```json
"enabledPlugins": {
  "frontend-design@claude-plugins-official": true
}
```

Plugins extend Claude Code with additional skills and hooks. If you install Tackline as a plugin (instead of via symlinks), use `--rules-only` with the installer to avoid duplicating skills.

---

## Attributions

Control the co-author line added to git commits:

```json
"attributions": {
  "commit": "",
  "pr": ""
}
```

Set to empty strings to disable co-author attribution. Set to a format string to customize it. Default adds `Co-Authored-By: Claude` to commits.

---

## Full Example

A complete settings.json for a Python project using uv, beads, and all Tackline hooks:

```json
{
  "env": {
    "CLAUDE_CODE_ENABLE_TELEMETRY": "1",
    "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1",
    "ENABLE_TOOL_SEARCH": "auto:5"
  },
  "permissions": {
    "allow": [
      "Bash(uv sync:*)",
      "Bash(uv run pytest:*)",
      "Bash(uv run python:*)",
      "Bash(uv run ruff:*)",
      "Bash(uv run mypy:*)",
      "Bash(bd:*)",
      "Bash(git status:*)",
      "Bash(git diff:*)",
      "Bash(git log:*)",
      "Bash(git show:*)",
      "Bash(git branch:*)",
      "Bash(git add:*)",
      "Bash(git commit:*)",
      "Bash(git push:*)",
      "Bash(tree:*)"
    ],
    "defaultMode": "default"
  },
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "echo '=== Session Start ==='; git log --oneline -3 2>/dev/null | sed 's/^/Last:    /'; echo \"Tree:    $(git status --short 2>/dev/null | wc -l) uncommitted files\"; bd status --no-activity 2>/dev/null || tk stats 2>/dev/null || true; echo '=== ==='"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "INPUT=\"$CLAUDE_TOOL_INPUT\"; if echo \"$INPUT\" | grep -qE '\"git reset --hard' 2>/dev/null; then echo 'WARNING: git reset --hard will discard all uncommitted changes.' >&2; elif echo \"$INPUT\" | grep -qE '\"git checkout \\.' 2>/dev/null; then echo 'WARNING: git checkout . will discard all unstaged changes.' >&2; elif echo \"$INPUT\" | grep -qE '\"git clean -f' 2>/dev/null; then echo 'WARNING: git clean -f will permanently delete untracked files.' >&2; fi"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Task",
        "hooks": [
          {
            "type": "command",
            "command": "echo 'REVIEW GATE: Agent completed. Verify deliverable before proceeding.' >&2"
          }
        ]
      }
    ]
  }
}
```

Replace the `uv`/`bd` patterns with your own stack's equivalents (npm, cargo, just, tk, etc.).
