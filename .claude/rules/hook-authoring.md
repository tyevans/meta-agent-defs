---
paths:
  - "settings.json"
  - ".claude/settings.json"
strength: should
freshness: 2026-02-21
---

# Hook Authoring Rules

Rules for writing hooks in `settings.json` (global) and `.claude/settings.json` (project-local).

## Fail Gracefully

- Always append `|| true` when calling optional tools (`bd`, project-specific CLIs) so hooks do not block the session if the tool is missing
- Test hook behavior when dependency tools are not installed -- the hook should degrade silently or emit a stderr warning, never error out
- Use `command -v <tool> >/dev/null 2>&1` to check for tool existence before calling it

## No Interactive Input

- Never use commands that block on user input (`read`, `vim`, `less`, `nano`, `select`)
- Never open a pager -- pipe through `head` or redirect to a variable instead
- Never launch a GUI or TUI from a hook

## Keep Hooks Fast

- No network calls (`curl`, `wget`, `fetch`, `nc`) -- hooks run synchronously and block Claude
- No long-running processes (`find /` on large trees, `npm install`, `docker build`)
- Target under 500ms execution time -- if a check is slow, move it to a command or agent instead

## Environment Variables

Hooks have access to these variables provided by Claude Code:

- `CLAUDE_TOOL_INPUT` -- JSON string of the tool's input parameters
- `CLAUDE_FILE_PATHS` -- newline-separated list of file paths relevant to the tool call
- `CLAUDE_SESSION_ID` -- unique identifier for the current session

## Output

- Use stderr (`>&2`) for warnings so they appear in hook output
- Keep output concise -- one or two lines for warnings, not paragraphs
- Prefix warnings with a category label (e.g., `DESTRUCTIVE WARNING:`, `BEADS WARNING:`)

## One Hook Per Concern

- Do not combine unrelated checks in a single hook -- split them into separate entries
- Each hook entry should have a clear, single responsibility
- This makes hooks easier to enable, disable, and debug independently

## Matchers

- Use the most specific matcher possible -- prefer tool names (`Bash`, `Write`, `Edit`) over empty string
- Empty string matcher (`""`) fires on every tool call and adds latency -- only use for truly global checks
- A hook with matcher `Bash` only fires on Bash tool calls, reducing unnecessary overhead

## Do This

- Model new hooks after the existing patterns in `settings.json`
- Test hooks locally by setting the environment variables and running the command in a shell
- Document what the hook guards against in a comment or in this rules file

## Don't Do This

- Do not suppress all output with `>/dev/null 2>&1` -- warnings should reach the user via stderr
- Do not modify files or state from hooks -- hooks are for checks and warnings, not side effects
- Do not duplicate logic that already exists in another hook
