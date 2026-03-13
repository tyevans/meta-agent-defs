---
paths:
  - "skills/**/*.md"
  - "agents/**/*.md"
strength: should
freshness: 2026-03-12
---

# Team Protocol

Behavioral rules for persistent learning teams. Full schemas and templates: [docs/team-protocol-reference.md](../docs/team-protocol-reference.md).

## Team Manifest

Teams are defined in `.claude/team.yaml`. This is the single source of truth for team composition.

### Do

- Include all required fields for each member: `name`, `role`, `tools`, `owns`.
- Set `isolation: worktree` when the member's `owns` patterns do not overlap with other members and the member's work is self-contained.

### Do Not

- Do not define team composition anywhere other than `.claude/team.yaml`.
- Do not set `isolation: worktree` for members that read or write shared state files (e.g., `.claude/tackline/memory/team/decisions.md`).

## Dispatching

### Do

- Default to parallel dispatch with `isolation: "worktree"` and `run_in_background: true`.
- Always inject the member's learnings file (`.claude/tackline/memory/agents/<name>/learnings.md`) into the dispatch prompt. Agents never self-load their own learnings.
- Include the task description, role context, and reflection instructions in every dispatch prompt.
- Use the Task tool for dispatch. Use native TeamCreate only when agents must exchange messages during execution.

### Do Not

- Do not dispatch serially unless tasks have true sequential dependencies (each task needs the previous one's output).
- Do not use native TeamCreate just because tasks depend on each other — use your task tracker for ordering.
- Do not rely on TeamCreate to enforce `tools` or budget from `team.yaml` — pass `--allowedTools` explicitly.

## Learnings

### Do

- Store learnings at `.claude/tackline/memory/agents/<name>/learnings.md` using four categories: Codebase Patterns, Gotchas, Preferences, Cross-Agent Notes.
- Cap learnings files at 60 lines (30 core + 30 task-relevant).
- Merge duplicates, archive entries older than 21 days without recent references, and promote entries confirmed across 3+ sprints to `rules/` or CLAUDE.md.
- Resolve cross-agent notes within 14 days: acknowledge, act on, or discard with rationale.

### Do Not

- Do not let learnings files exceed 60 lines. Overflow goes to `.claude/tackline/memory/agents/<name>/archive.md`.
- Do not let agents write their own learnings files. The orchestrator parses reflections and writes learnings.

## Reflection

Every spawned team member must end its response with a structured reflection containing: `task_result` (status, summary, files_changed), `reflection` (what_worked, what_didnt, confidence), `suggested_learnings` (category, content, for_agent), `follow_up` (suggested_next, needs_human). Full JSON schema: [docs/team-protocol-reference.md](../docs/team-protocol-reference.md#reflection-schema).

## Agent Continuity

### Do

- Record the agent ID from Task tool returns after expensive phases (multi-turn research, large code generation).
- Write checkpoints to `.claude/tackline/memory/scratch/<skill-name>-agent-<member>.md`.
- Resume when: checkpoint exists, is less than 7 days old, task and files are unchanged, agent was paused or partial (not blocked/failed).

### Do Not

- Do not resume when the checkpoint is older than 7 days, the task definition changed, working files changed significantly, or the agent status was blocked/failed. Dispatch fresh instead.
- Do not keep stale checkpoints. Delete after successful completion. Checkpoints older than 14 days may be deleted during `/retro` without review.

## Memory MCP

File-based learnings are primary. Memory MCP is secondary — use it for cross-project queries, agent identity when file context is unavailable, and relation-based discovery.

### Do

- Check for `.claude/team.yaml` first. If present, use file-based learnings.
- Fall back to MCP only when no team manifest exists.
- Use entity naming `agent:<agent-name>` with type `agent-memory`.

### Do Not

- Do not use MCP as the primary persistence for within-project learnings.
- Do not remember session-specific context, unverified hunches, or information already in CLAUDE.md or rules via MCP.
