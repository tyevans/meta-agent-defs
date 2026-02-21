---
strength: should
freshness: 2026-02-21
---

# Batch Safety

When processing a numbered collection of items (audit findings, files to review, migrations to run, tickets to close):

1. **Count before starting.** Determine the total item count before processing the first item.
2. **Warn at 12.** If the count exceeds 12, state the risk to the user and propose chunking into groups of 8-10.
3. **Write before continuing.** For each chunk, write completed results to `memory/scratch/<task-slug>-chunk-N.md` before starting the next chunk.
4. **Never silently continue past item 12** without having written prior results to a file.

The goal is to prevent context compaction from destroying completed work in long batch operations. Intermediate files make progress recoverable.

See also: /decompose (break large collections into bounded sub-parts before processing).
