---
strength: must
freshness: 2026-02-21
---

# Context Trust

When the user provides ticket content, requirements, or scoped context for a task:

1. **Trust it as-is.** Do not re-read the ticket source, re-examine linked resources, or re-baseline information the user just provided.
2. **Stay scoped.** Do not broadly explore the codebase to "understand context" when the user has already scoped the task. Start narrow, widen only if the provided context is genuinely insufficient.
3. **Ask, don't investigate.** If something in the provided context seems incomplete or contradictory, ask the user rather than independently re-investigating.

This does not apply when context is stale (e.g., referenced files have changed since the ticket was created) or when the task explicitly requires investigation.

See also: /gather (structured information collection when investigation IS the task).
