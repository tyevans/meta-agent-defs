---
strength: must
freshness: 2026-03-13
---

# Issue Quality

When any skill or workflow creates an issue, task, or ticket in an external tracker (Tacks, Beads, Linear, GitHub Issues, etc.):

1. **Descriptive title.** Use `<affected artifact>: <what needs to happen>` format. Titles must be actionable — a reader should understand the work without opening the issue.
2. **Self-contained description.** Include enough context that someone unfamiliar with the originating conversation can act on the issue without asking clarifying questions. Reference specific file paths, function names, or code patterns when applicable.
3. **Definition of done.** Every issue must include a concrete, verifiable definition of done — a checklist of conditions that must be true for the issue to be considered complete. Prefer observable outcomes ("endpoint returns 200 with valid payload") over process statements ("code has been reviewed").
4. **Provenance.** State which skill or workflow produced the issue and when (e.g., "Filed via /retro on 2026-03-13"). If the issue originated from pipe-format output, include the pipeline chain.
5. **Priority/severity.** Include a priority or severity level using the tracker's conventions. Default to medium/P2 if context is ambiguous.

## What "Self-Contained" Means

A self-contained description includes:
- **Why** the work matters (the problem or motivation)
- **What** needs to change (specific files, behaviors, or interfaces)
- **How to verify** the change is correct (the definition of done)

It does NOT require:
- Full implementation details (the assignee determines approach)
- Exhaustive background (link to related issues instead of re-explaining)

## Rationale

Issues outlive the conversation that created them. Without sufficient detail, the assignee must re-investigate context that was already known at filing time — wasting effort and risking misinterpretation.

See also: /bug (structured bug reports), /team-meeting (task creation from planning).
