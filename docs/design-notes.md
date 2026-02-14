# Design Notes

Lightweight "why" records for non-obvious architectural decisions. Each entry captures the context and rationale so future contributors (human or model) don't have to reverse-engineer intent.

---

## Markdown pipe format over JSON

**Date:** 2025-01
**Context:** Composable primitives needed a shared output format so any primitive's output could feed another.

JSON was the obvious choice for structured interchange, but LLMs parse markdown natively and emit it fluently. JSON requires escaping, brace-matching, and tends to produce formatting errors under token pressure. Markdown numbered lists with bold titles give enough structure for downstream primitives to parse while remaining human-readable. The tradeoff is that markdown is less precise than JSON for machine consumers — but our only consumers are LLMs, which handle markdown better than JSON anyway.

---

## Skills replaced commands

**Date:** 2025-01
**Context:** The repo originally had a `commands/` directory with shell-invocable command definitions alongside the `skills/` directory.

The two systems overlapped: both defined slash-invocable workflows with tool restrictions. Maintaining both meant duplicated patterns, unclear placement decisions ("is this a command or a skill?"), and two sets of authoring rules. Skills won because they support frontmatter-driven discovery, context mode selection (fork vs inline), and integrate with Claude Code's native skill system. Commands were migrated to skills and the `commands/` directory was removed entirely rather than deprecated.

---

## `disable-model-invocation: false` everywhere

**Date:** 2025-02
**Context:** Some skills were initially set to `disable-model-invocation: true` to prevent Claude from auto-invoking them.

Setting this to `true` blocks the Skill tool entirely, which means even programmatic invocation (from /sprint dispatching work, or from other skills composing) fails silently. The cost of accidental auto-invocation is low (Claude invokes a skill when it shouldn't — minor token waste). The cost of blocking programmatic invocation is high (workflows break). Every skill now defaults to `false`.

---

## Fork vs inline context

**Date:** 2025-01
**Context:** Skills needed a way to run heavy exploration without polluting the orchestrator's context window.

Fork skills (blossom, consensus, review, etc.) run in an isolated context and return a summary. This prevents thousands of lines of exploration output from consuming the main session's context budget. Inline skills (primitives, team management, session tools) are lightweight enough to run in-place and benefit from seeing prior context. The heuristic: if a skill dispatches subagents or reads more than ~20 files, it should fork. If it transforms context that's already in the conversation, it should be inline.

---

## Learnings capped at 60 lines

**Date:** 2025-02
**Context:** Persistent agent learnings (`memory/agents/<name>/learnings.md`) accumulate across sessions and get injected into agent prompts.

Uncapped learnings would grow until they consumed significant prompt budget, degrading agent performance. The 60-line cap (30 core + 30 task-relevant) forces triage: only the most durable insights survive. Stale entries (>21 days without use) get archived to `archive.md`. This mirrors how human teams work — you remember the important patterns, not every detail from every sprint.

---

## Two-tier install (global + project-local)

**Date:** 2025-01
**Context:** Some definitions (rules, skills, agents) should apply everywhere. Others are specific to this repo.

Global artifacts (`rules/`, `skills/`, `agents/`, `settings.json`) are symlinked to `~/.claude/` by `install.sh` and load in every project. Project-local artifacts (`.claude/agents/`, `.claude/rules/`, `.claude/skills/`) only load when working in this repo. This separation means a skill like /gather works in any project, while a project-local agent like `skill-author` only appears when editing this repo. The alternative — everything global — would pollute other projects with meta-agent-defs-specific tooling.

---

## Orchestrator mode

**Date:** 2025-02
**Context:** The primary Claude Code session was doing everything: reading code, writing code, managing backlog, dispatching agents.

This caused context bloat (implementation details crowding out orchestration state) and scope creep (the session drifting from coordination into implementation). Restricting the primary session to orchestration-only — backlog management, task dispatch, coordination — keeps the context window focused on what matters: what needs doing and what's blocked. Implementation happens in subagents that have fresh, focused contexts. The constraint feels limiting but produces better outcomes because each context window stays within its competence.
