---
strength: should
freshness: 2026-02-25
---

# Compaction Resilience

Strategies for surviving context compaction in long-running skills and batch operations.

## Checkpoint Convention

Skills with 3 or more phases SHOULD write intermediate state to `memory/scratch/<skill-name>-checkpoint.md` at each phase boundary before continuing. The checkpoint captures:

- Which phases have completed
- Key findings or output items produced so far
- Any decisions made that affect later phases

### Instance-Scoped Checkpoints

When `$CLAUDE_INSTANCE_ID` is set (injected by shell aliases for parallel terminal sessions), skills that checkpoint SHOULD append the instance ID to the filename: `memory/scratch/<skill-name>-checkpoint-<CLAUDE_INSTANCE_ID>.md`. This prevents parallel sessions from clobbering each other's state. When `$CLAUDE_INSTANCE_ID` is not set, fall back to the unscoped filename.

## PreCompact Hook

The PreCompact hook auto-persists the most recent pipe-format output block to `memory/sessions/pre-compact.md`. This covers single-phase primitives and the final output of multi-phase skills without requiring explicit checkpoint writes.

## Recovery Pattern

When a skill detects a checkpoint file at startup:

1. Read `memory/scratch/<skill-name>-checkpoint.md`
2. Report which phases are already complete
3. Skip completed phases and resume from the first incomplete phase
4. Treat checkpoint items as confirmed prior output — do not re-derive them

## Scratch Cleanup

Files in `memory/scratch/` are ephemeral:

- The skill SHOULD delete its checkpoint on successful completion
- The SessionEnd hook MAY sweep stale scratch files older than one session
- Do not treat scratch files as durable state — they are not guaranteed to persist across sessions

See also: /batch-safety (chunking rule for collections over 12 items), pipe-format.md (output block format that PreCompact captures).
