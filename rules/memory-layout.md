---
paths:
  - ".claude/tackline/memory/**"
strength: should
freshness: 2026-03-12
---

# Memory Layout

Single source of truth for persistent state: where it lives, who writes it, and how to survive context compaction. Skills SHOULD reference this rule rather than inlining their own memory instructions.

## Root Path

All persistent state lives under `.claude/tackline/memory/`.

## Path Registry

| Path | Purpose | Written By |
|------|---------|------------|
| `sessions/YYYY-MM-DDThh-mm-ssZ.md` | Session state snapshot, rolling buffer; last 3 kept | SessionEnd hook |
| `sessions/last.md` | Copy of most recent session snapshot | SessionEnd hook |
| `sessions/pre-compact.md` | Pre-compaction snapshot: in-progress tasks, recent commits, open questions | PreCompact hook |
| `agents/<name>/learnings.md` | Agent-specific learnings (persistent, append-only) | /sprint, /retro, /curate |
| `agents/<name>/archive.md` | Archived stale learnings (>21 days) | /retro, /curate |
| `agents/<name>/challenges/` | Challenge definitions and outcome history | /active-learn |
| `agents/<name>/capability.yaml` | Agent capability profile (strengths/weaknesses) | /active-learn, /diagnose-agent |
| `team/decisions.md` | Cross-cutting team decisions log | /assemble, /team-meeting |
| `team/retro-history.md` | Retrospective history (append-only) | /retro |
| `epics/<epic-id>/epic.md` | Epic state: spike findings, priority order, task IDs, critical path | /blossom |
| `project/domain.md` | Project domain terminology and disambiguation rules | /domain |
| `scratch/<skill-name>-checkpoint.md` | Compaction checkpoint (ephemeral) | Multi-phase skills |

All paths are relative to `.claude/tackline/memory/`.

## Rules

1. **Read from known paths.** Skills that need system state should use the paths above, not invent new locations.
2. **No new write obligations.** Only subsystems that already persist state should write here. Stateless primitives (gather, distill, rank, etc.) must not acquire memory side-effects.
3. **Free-form markdown.** Files have no required sections or field-level schemas. Content structure is owned by the writing subsystem.
4. **Sessions: rolling buffer, last 3.** The SessionEnd hook prunes to keep the 3 most recent timestamped files. `last.md` is always a copy of the most recent.
5. **Agent learnings: tiered and capped.** Core + Task-Relevant tiers, capped at 60 lines total (30 + 30). Overflow moves to `archive.md`.
6. **Reference, don't repeat.** Skills should say "Per `rules/memory-layout.md`, checkpoint at phase boundaries" — not inline the full protocol.

## Checkpoint Protocol

Skills with 3+ phases SHOULD write intermediate state to `scratch/<skill-name>-checkpoint.md` at each phase boundary. The checkpoint captures:

- Which phases have completed
- Key findings or output items produced so far
- Any decisions made that affect later phases

### Recovery

When a skill detects a checkpoint file at startup:

1. Read the checkpoint
2. Report which phases are already complete
3. Skip completed phases and resume from the first incomplete phase
4. Treat checkpoint items as confirmed prior output — do not re-derive them

### PreCompact Hook

The PreCompact hook auto-persists the most recent pipe-format output block to `sessions/pre-compact.md`. This covers single-phase primitives without requiring explicit checkpoint writes.

### Scratch Cleanup

Files in `scratch/` are ephemeral:

- The skill SHOULD delete its checkpoint on successful completion
- The SessionEnd hook MAY sweep stale scratch files older than one session
- Do not treat scratch files as durable state
