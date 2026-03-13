---
strength: must
freshness: 2026-03-13
---

# Task Tracker Discovery

When a skill or workflow needs to interact with an external task tracker (Tacks, Beads, Linear, GitHub Issues, or any CLI-based tracker):

1. **Discover before invoking.** Before running any task tracker command, run the tracker's help command (e.g., `tacks --help`, `beads help`) to learn the available subcommands and their syntax. Do not guess or invent commands from memory.
2. **Drill into subcommands.** After discovering top-level commands, run help on the specific subcommand you need (e.g., `tacks create --help`) to learn required and optional flags before constructing the full command.
3. **Cache for the session.** Once you have discovered the correct syntax for an operation, you do not need to re-run help for the same operation within the same session. But do not carry assumed syntax across sessions — always discover fresh.

## Why

LLMs frequently hallucinate CLI syntax for unfamiliar or project-specific tools. The cost of two help calls (~1 second) is far lower than the cost of multiple failed invocations, error parsing, and retry loops that burn tokens and context window.

## Detecting the Tracker

If the project's task tracker is not obvious, check in this order:

1. **CLAUDE.md** — often names the tracker and its CLI command
2. **MCP servers** — check if an MCP tool provides task management (e.g., `mcp__tacks__*`)
3. **Shell availability** — run `which tacks beads linear gh 2>/dev/null` to detect installed CLIs
4. **Project config** — look for `.tacks/`, `.beads/`, `.linear/` directories

If no tracker is detected, skip tracker operations and note the absence — do not invent a fallback.
