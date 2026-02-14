# Orchestrator Mode: Claude-in-Claude Dispatch

You are running in **orchestrator mode**. Your primary job is to manage work by dispatching it to Claude CLI subprocesses, NOT by using the built-in Task tool.

## Critical Rule: No Built-in Task Tool for Work Dispatch

**DO NOT use the Task tool** to spawn subagents for implementation, research, or any substantive work. The Task tool creates subagents that inherit a frozen context snapshot — they cannot pick up updated learnings, memory files, or changed project state.

Instead, dispatch work by running `claude` as a subprocess via Bash:

```bash
claude -p \
  --model sonnet \
  --permission-mode bypassPermissions \
  "Your detailed prompt here"
```

This ensures each subprocess:
- Reads the latest MEMORY.md and project learnings at startup
- Picks up any files changed by prior subprocesses
- Gets fresh hook execution (SessionStart, etc.)
- Operates with current beads/backlog state

## Dispatch Patterns

### Quick query (returns text, no file changes expected)
```bash
claude -p --model haiku "What files in agents/ define read-only agents?"
```

### Implementation task (needs file access)
```bash
claude -p \
  --model sonnet \
  --permission-mode bypassPermissions \
  "Implement feature X. Details: ..."
```

### Complex task needing specific tools
```bash
claude -p \
  --model opus \
  --permission-mode bypassPermissions \
  --allowed-tools "Read Edit Write Bash Grep Glob" \
  "Refactor the agent-author to support the new frontmatter schema. ..."
```

### Capturing output for review
```bash
RESULT=$(claude -p --model sonnet "Analyze the skill definitions in skills/ and list any that are missing required frontmatter fields")
echo "$RESULT"
```

## When Task Tool IS Acceptable

The Task tool is still fine for:
- **Explore** subagent type for quick codebase searches (read-only, no learning needed)
- **Plan** subagent type for architecture planning (read-only)
- Other read-only research where fresh context doesn't matter

## Dispatch Strategy

Follow the same serialization-by-default principle from CLAUDE.md:
1. Dispatch one task at a time via `claude -p`
2. Review the output
3. Dispatch the next task, incorporating learnings from the previous one

For truly independent tasks (no shared files, no ordering dependency), you may run multiple `claude -p` calls in parallel using background Bash processes, but this is the exception.

## Output Handling

- `claude -p` prints the response to stdout and exits
- Use `--output-format json` if you need structured output
- Long-running tasks: consider `--max-budget-usd` to cap costs
- If a subprocess fails, read its output, diagnose, and retry with adjusted prompt — don't just re-run blindly

## Model Selection

- **haiku**: Quick lookups, simple questions, file searches
- **sonnet**: Standard implementation, refactoring, code review (default)
- **opus**: Complex architectural work, multi-file refactors, nuanced analysis
