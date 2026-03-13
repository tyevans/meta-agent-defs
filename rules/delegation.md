---
strength: must
freshness: 2026-03-13
---

# Delegation

The primary session is a coordinator, not an implementer. Dispatch work to subagents and use skills to guide execution.

## Dispatch, Don't Do

Do not perform implementation work in the primary session even when it feels faster. The coordination overhead pays for itself through parallelism and better-guided execution.

- **Single task**: dispatch to one subagent (worktree isolation preferred)
- **Multiple independent tasks**: dispatch in parallel, each in its own worktree
- **Multi-faceted work requiring coordination**: use a team (TeamCreate) with direct messaging to resolve conflicts
- **Any task with a matching skill**: invoke the skill — it encodes domain knowledge that ad-hoc reasoning lacks

The only work the primary session performs directly is triage, coordination, and content-only edits in content-only repos (per CLAUDE.md operating mode).

## Concurrency by Default

Assume tasks can run in parallel unless there is a true data dependency (one task's output is the next task's input). When in doubt, parallelize — merge conflicts in isolated worktrees are cheaper than serial execution time.

| Situation | Approach |
|-----------|----------|
| Independent tasks, no shared files | Parallel worktree agents |
| Independent tasks, overlapping files | Parallel worktree agents (merge after) |
| Tasks needing mid-execution communication | Team with direct messaging |
| Strictly sequential dependency chain | Serial dispatch (the exception) |

## Skills Over Intuition

When a skill exists for the task at hand, use it. Skills encode tested workflows, phase gates, and output contracts that raw model reasoning does not reliably reproduce. Check `/discover` or the skill list before improvising a multi-step approach.

Key examples:
- **Before writing tests**: run `/test-strategy` to classify the epistemic level and select TDD vs test-after
- **Before large implementation**: run `/spec` or `/blossom` to scope the work
- **Before code review**: run `/review` for structured findings
- **Before deployment**: run `/deploy` for readiness checks and rollback criteria

## Exceptions

- Trivial content edits in content-only repos (fixing a typo, updating a date)
- Single-line clarification questions back to the user
- Reading files to understand context before dispatching
