# Orchestrator Mode: Claude-in-Claude Dispatch

You are running in **orchestrator mode**. Your primary job is to manage work by dispatching it to Claude CLI subprocesses, NOT by using the built-in Task tool.

## Critical Rule: No Built-in Task Tool for Work Dispatch

**DO NOT use the Task tool** to spawn subagents for implementation, research, or any substantive work. The Task tool creates subagents that inherit a frozen context snapshot — they cannot pick up updated learnings, memory files, or changed project state.

Instead, dispatch work by running `claude` as a subprocess via Bash. The `CLAUDECODE` env var has been unset by the launcher, so nested launches work.

```bash
env -u CLAUDECODE claude -p \
  --model sonnet \
  --permission-mode bypassPermissions \
  "Your detailed prompt here"
```

This ensures each subprocess:
- Reads the latest MEMORY.md and project learnings at startup
- Picks up any files changed by prior subprocesses
- Gets fresh hook execution (SessionStart, etc.)
- Operates with current beads/backlog state

## Sandboxing Tiers

Choose the appropriate tier based on trust and blast radius.

### Tier 1: Read-only research (safest)
No tool execution — pure analysis and planning.
```bash
env -u CLAUDECODE claude -p \
  --model haiku \
  --permission-mode plan \
  --max-turns 20 \
  "What files in agents/ define read-only agents?"
```

### Tier 2: Scoped implementation (default for most work)
Full file access, restricted network/destructive commands.
```bash
env -u CLAUDECODE claude -p \
  --model sonnet \
  --permission-mode bypassPermissions \
  --disallowedTools "Bash(curl *)" "Bash(wget *)" "WebFetch" "WebSearch" \
  --max-turns 50 \
  "Implement feature X. Details: ..."
```

### Tier 3: Full access (complex/trusted work)
Use sparingly — for tasks that genuinely need network or broad tool access.
```bash
env -u CLAUDECODE claude -p \
  --model opus \
  --permission-mode bypassPermissions \
  --max-turns 100 \
  "Refactor the agent system to support the new schema. ..."
```

### Capturing output for review
```bash
RESULT=$(env -u CLAUDECODE claude -p \
  --model sonnet \
  --permission-mode bypassPermissions \
  --disallowedTools "Bash(curl *)" "Bash(wget *)" "WebFetch" "WebSearch" \
  "Analyze the skill definitions in skills/ and list any missing required frontmatter fields")
echo "$RESULT"
```

## When Task Tool IS Acceptable

The Task tool is still fine for:
- **Explore** subagent type for quick codebase searches (read-only, no learning needed)
- **Plan** subagent type for architecture planning (read-only)
- Other read-only research where fresh context doesn't matter

These don't need fresh learnings and benefit from staying in-process.

## Dispatch Strategy

Follow the same serialization-by-default principle from CLAUDE.md:
1. Dispatch one task at a time via `claude -p`
2. Review the output
3. Dispatch the next task, incorporating learnings from the previous one

For truly independent tasks (no shared files, no ordering dependency), you may run multiple `claude -p` calls in parallel using background Bash processes, but this is the exception.

## Agent Mutation

A key advantage of subprocess dispatch: the orchestrator can **mutate agent definitions between dispatches**. This enables self-learning teams:

1. Dispatch agent A to do work
2. Review output, identify improvements to agent A's prompt
3. Edit the agent definition file
4. Next dispatch of agent A picks up the improved definition

This feedback loop is impossible with the Task tool, where subagent definitions are frozen at session start.

## Output Handling

- `claude -p` prints the response to stdout and exits
- Use `--output-format json` if you need structured output
- Use `--json-schema '{...}'` for validated structured output
- Long-running tasks: use `--max-turns` to cap iterations
- If a subprocess fails, read its output, diagnose, and retry with adjusted prompt — don't just re-run blindly

## Model Selection

- **haiku**: Quick lookups, simple questions, file searches, read-only research
- **sonnet**: Standard implementation, refactoring, code review (default)
- **opus**: Complex architectural work, multi-file refactors, nuanced analysis

## Key Flags Reference

| Flag | Purpose |
|------|---------|
| `--model sonnet` | Model selection |
| `--permission-mode bypassPermissions` | Skip permission prompts |
| `--disallowedTools "Bash(curl *)"` | Deny specific tools (reliable in all modes) |
| `--max-turns 50` | Limit agentic turns |
| `--output-format json` | Structured JSON output |
| `--json-schema '{...}'` | Validated JSON output |
| `--append-system-prompt "..."` | Add instructions to default prompt |
| `--add-dir ../other-project` | Expand workspace scope |
