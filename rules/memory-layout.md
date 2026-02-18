# Memory Layout: Path Registry

Known paths under `memory/` where persistent state lives. Skills that read system state (like `/status`) should read from these paths rather than scraping ad-hoc.

## Paths

| Path | Purpose | Written By |
|------|---------|------------|
| `memory/sessions/YYYY-MM-DDThh-mm-ssZ.md` | Session state snapshot, rolling buffer; last 3 kept | SessionEnd hook |
| `memory/sessions/last.md` | Copy of most recent session snapshot, for backward compat | SessionEnd hook |
| `memory/sessions/pre-compact.md` | Pre-compaction snapshot: in-progress tasks, recent commits, open questions | PreCompact hook |
| `memory/agents/<name>/learnings.md` | Agent-specific learnings (persistent, append-only) | /sprint, /retro |
| `memory/agents/<name>/archive.md` | Archived stale learnings (>21 days) | /retro |
| `memory/agents/<name>/challenges/` | Challenge definitions and outcome history for active learning | /active-learn |
| `memory/agents/<name>/training-log.md` | Training session summaries and improvement trajectory | /active-learn |
| `memory/agents/<name>/capability.yaml` | Agent capability profile (strengths/weaknesses with scores) | /active-learn, /diagnose-agent |
| `memory/team/decisions.md` | Team decisions log | /meeting, manual |
| `memory/team/retro-history.md` | Retrospective summaries | /retro |
| `memory/epics/<epic-id>/epic.md` | Epic state: spike findings, priority order, task IDs, critical path | /blossom |
| `memory/project/domain.md` | Project domain terminology and disambiguation rules | /domain |
| `memory/MEMORY.md` | Project-level persistent memory | auto-memory system |

## Rules

1. **Read from known paths.** Skills that need system state should use the paths above, not invent new locations or scrape from unrelated files.
2. **No new write obligations.** Only subsystems that already persist state should write here. Stateless primitives (gather, distill, rank, etc.) must not acquire memory side-effects.
3. **Free-form markdown.** Files have no required sections or field-level schemas. Content structure is owned by the writing subsystem.
4. **Sessions: rolling buffer, last 3.** `memory/sessions/` holds ISO-timestamped files (`YYYY-MM-DDThh-mm-ssZ.md`). The SessionEnd hook prunes to keep the 3 most recent timestamped files. `last.md` is always a copy of the most recent session for backward compatibility.
5. **Agent learnings: tiered and capped.** Learnings files use Core + Task-Relevant tiers, capped at 60 lines total (30 + 30). Overflow moves to `archive.md`.
