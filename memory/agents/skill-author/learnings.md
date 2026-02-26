# Learnings: skill-author

## Core

- When writing inline skills consumed by downstream skills in the same conversation, document the consumption interface explicitly in the output format section AND the See Also line (added: 2026-02-20, dispatch: tack-yul3.5)
- Pipe-format interstitial sections (between Items and Summary) are valid for workflow skills that need a second-class finding set — use `### SectionName (M)` with its own count (added: 2026-02-20, dispatch: tack-yul3.5)
- When a skill detects upstream output from a sibling skill, the detection pattern (`## Header / **Source**: /skillname`) should be documented in BOTH the consuming skill's Phase 0 and the producing skill's output section (added: 2026-02-20, dispatch: tack-yul3.6)
- When adding a conditional mode to an existing skill, prefer inline "In X mode:" paragraphs over parallel section duplication — keeps the skill readable for normal-mode users and makes the conditional delta obvious to reviewers (added: 2026-02-21, dispatch: tack-l1vo.12)
- When inserting an enrichment phase before existing Phase 1, number it Phase 0 rather than renumbering downstream — keeps the change purely additive and avoids collateral cross-reference edits (added: 2026-02-25, dispatch: tack-2rli.8)
- When a skill has two dispatch modes (isolated vs non-isolated), show both Task call signatures side-by-side with bold conditional headers rather than using if/else pseudocode — skill files are instructions for Claude, not executable code (added: 2026-02-25, dispatch: tack-3a8f.1)
- When adding a new dispatch mode, label the existing mode as "(default)" to preserve backward compatibility and make the opt-in nature of the new mode clear (added: 2026-02-25, dispatch: tack-pca6.1)
- When a skill report section is conditional (only present when data exists), annotate it with an inline condition comment in the template rather than separate if/else blocks — keeps the template scannable (added: 2026-02-25, dispatch: tack-1rj0.3)
- When adding a resume mechanism to a guardian-style dispatch, annotate the max-iteration cap inline at the same location as the resume call — placing it separately creates a split reviewers have to reconcile mentally (added: 2026-02-25, dispatch: tack-1rj0.4)
- When adding a parallel mode to a serial primitive, introduce a source-breadth assessment as the new Phase 1 — makes the mode-selection decision transparent before any tool calls (added: 2026-02-25, dispatch: tack-2rli.1)
- When a fan-out skill needs merge conflict resolution, express the tiebreaker as a total order (e.g., REFUTED > VERIFIED > UNCERTAIN) rather than a conditional — covers all pairwise cases without branching prose (added: 2026-02-25, dispatch: tack-2rli.2)
- Agent prompt templates embedded in a skill body should use a header level lower than surrounding steps (e.g., `####`) to signal they are templates, not executable instructions for the orchestrator (added: 2026-02-25, dispatch: tack-2rli.2)
- When upgrading a dead-end branch in a quality gate (e.g., "Cannot re-prompt") to a retry mechanism, preserve the original fallback as the post-retry path — it remains valid for when the retry also fails (added: 2026-02-25, dispatch: tack-1rj0.5)
- When adding resume to a fan-out skill, add a "capture handler IDs at dispatch time" note near the original dispatch site — orchestrators need IDs available at the resume call site (added: 2026-02-25, dispatch: tack-1rj0.6)
- For depth-bounded recursion (fractal-style), set the resume boundary at depth N→N+1 only; deeper transitions spawn fresh to avoid cascading warm-context dependencies (added: 2026-02-25, dispatch: tack-1rj0.6)
- When inserting a new numbered subsection into an existing sequence, renumber downstream labels in the same edit to avoid a follow-up correction pass (added: 2026-02-25, dispatch: tack-1rj0.7)
- When a phase-0 resume check produces a phase collision (two sections numbered "Phase 0"), rename the original to Phase 1 and renumber all downstream phases — a named collision is worse than a renumber cascade (added: 2026-02-25, dispatch: tack-1rj0.8)

## Task-Relevant
