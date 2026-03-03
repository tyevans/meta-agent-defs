# Learnings: agent-author

## Core

- team-protocol.md uses tables for condition/mechanism mappings and numbered lists for sequential steps. New sections follow: `## Section`, `### Subsection`, body prose, then table or list. YAML frontmatter `freshness` is not modified on content-only additions (added: 2026-02-21, dispatch: tack-d008)

- When a task references auto-persistence or canonical paths, check memory-layout.md for the authoritative path table before writing rules — avoids drift between rule text and actual write targets (added: 2026-02-25, dispatch: tack-qyt8.1)
- The project-bootstrapper's Phase 4 hook JSON uses nested `hooks` arrays — when adding hook commands, add to the inner `hooks` array as sibling entries, not as a new outer array entry (added: 2026-02-25, dispatch: tack-qyt8.3)

- When adding an optional convention to a rules file, place it between affirmative ("Do This") and negative ("Don't Do This") sections — reads as an opt-in pattern rather than a requirement or constraint (added: 2026-02-25, dispatch: tack-1rj0.2)

- team-protocol.md has both Required and Optional field subsections under "Team Manifest" — new optional member fields go in the Optional Fields table and YAML example should show the field on one member only to signal optionality (added: 2026-02-25, dispatch: tack-pca6.6)

- When searching for phrases in definition files, the actual text may embed the phrase differently than expected (e.g., "beads dependencies" appears as "dependencies between beads") — always grep for individual terms and read surrounding context (added: 2026-02-26, dispatch: sprint-tacks-alt)

- When flipping tool ordering in docs, check for cases where one tool's name is used as a generic noun (e.g., "beads" meaning "backlog items") — those require tool-agnostic replacement, not just reordering (added: 2026-03-02, dispatch: tk-fbdc.6)
- README install flags may need semantic reversal (not just label swap) when the default tool changes — e.g., --tacks becomes --beads when tacks is the new default (added: 2026-03-02, dispatch: tk-fbdc.7)

## Task-Relevant
