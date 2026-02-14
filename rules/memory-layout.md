# Memory Layout: Path Registry

Known paths under `memory/` where persistent state lives. Skills that read system state (like `/status`) should read from these paths rather than scraping ad-hoc.

## Paths

| Path | Purpose | Written By |
|------|---------|------------|
| `memory/sessions/last.md` | Last session state snapshot (overwritten each session) | SessionEnd hook |
| `memory/agents/<name>/learnings.md` | Agent-specific learnings (persistent, append-only) | /sprint, /retro |
| `memory/agents/<name>/archive.md` | Archived stale learnings (>21 days) | /retro |
| `memory/team/decisions.md` | Team decisions log | /meeting, manual |
| `memory/team/retro-history.md` | Retrospective summaries | /retro |
| `memory/MEMORY.md` | Project-level persistent memory | auto-memory system |

## Rules

1. **Read from known paths.** Skills that need system state should use the paths above, not invent new locations or scrape from unrelated files.
2. **No new write obligations.** Only subsystems that already persist state should write here. Stateless primitives (gather, distill, rank, etc.) must not acquire memory side-effects.
3. **Free-form markdown.** Files have no required sections or field-level schemas. Content structure is owned by the writing subsystem.
4. **Sessions: single file, overwritten.** `memory/sessions/last.md` is replaced each session, not accumulated.
5. **Agent learnings: tiered and capped.** Learnings files use Core + Task-Relevant tiers, capped at 60 lines total (30 + 30). Overflow moves to `archive.md`.
