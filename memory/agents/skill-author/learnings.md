# Learnings: skill-author

## Core

- When writing inline skills consumed by downstream skills in the same conversation, document the consumption interface explicitly in the output format section AND the See Also line (added: 2026-02-20, dispatch: tack-yul3.5)
- Pipe-format interstitial sections (between Items and Summary) are valid for workflow skills that need a second-class finding set — use `### SectionName (M)` with its own count (added: 2026-02-20, dispatch: tack-yul3.5)
- PASSIVE scoring (entry covered by a rule) must cite the specific rule filename at the scoring decision point, not just in guidelines (added: 2026-02-20, dispatch: tack-yul3.5)

- When a skill detects upstream output from a sibling skill, the detection pattern (`## Header / **Source**: /skillname`) should be documented in BOTH the consuming skill's Phase 0 and the producing skill's output section (added: 2026-02-20, dispatch: tack-yul3.6)
- Per-candidate criteria tables (PASS/FAIL per row) are more actionable than summary gates — they let users see exactly which criterion blocked a candidate (added: 2026-02-20, dispatch: tack-yul3.6)
- Placement decision logic from IA principles reads most clearly as a three-question decision table with target columns, not prose (added: 2026-02-20, dispatch: tack-yul3.6)

- When adding a conditional mode to an existing skill, prefer inline "In X mode:" paragraphs over parallel section duplication — keeps the skill readable for normal-mode users and makes the conditional delta obvious to reviewers (added: 2026-02-21, dispatch: tack-l1vo.12)

- When fixing a literal string bug in a skill file, grep the full file for all instances — skill files often repeat the same value in code blocks and guideline sections (added: 2026-02-25, dispatch: tack-2rli.10)
- When adding a new output section that bridges to a downstream skill, document the consumption contract in both the format block (structure) and the guidelines (semantics) — mirrors the producer/consumer detection pattern learning (added: 2026-02-25, dispatch: tack-pca6.2)
- When adding a conditional field to a multi-strategy template (e.g., edge-case vs commit-replay), use inline annotations like `*(commit-replay only)*` rather than conditional blocks or duplicated templates (added: 2026-02-25, dispatch: tack-3a8f.2)
- When a skill has two dispatch modes (isolated vs non-isolated), show both Task call signatures side-by-side with bold conditional headers rather than using if/else pseudocode — skill files are instructions for Claude, not executable code (added: 2026-02-25, dispatch: tack-3a8f.1)
- When adding a new dispatch mode, label the existing mode as "(default)" to preserve backward compatibility and make the opt-in nature of the new mode clear (added: 2026-02-25, dispatch: tack-pca6.1)

- When a skill report section is conditional (only present when data exists), annotate it with an inline condition comment in the template rather than separate if/else blocks — keeps the template scannable (added: 2026-02-25, dispatch: tack-1rj0.3)

- When adding a resume mechanism to a guardian-style dispatch, annotate the max-iteration cap inline at the same location as the resume call — placing it separately creates a split reviewers have to reconcile mentally (added: 2026-02-25, dispatch: tack-1rj0.4)

- When adding a parallel mode to a serial primitive, introduce a source-breadth assessment as the new Phase 1 — makes the mode-selection decision transparent before any tool calls (added: 2026-02-25, dispatch: tack-2rli.1)
- When a parallel mode wraps a Skill invocation, the Task prompt must embed all context the Skill would gather itself — subagents cannot invoke skills, so pass context from prior phases inline (added: 2026-02-25, dispatch: tack-2rli.5)
- When inserting an enrichment phase before existing Phase 1, number it Phase 0 rather than renumbering downstream — keeps the change purely additive and avoids collateral cross-reference edits (added: 2026-02-25, dispatch: tack-2rli.8)
- When adding a new layer to a multi-layer skill, update all four touch points: prose description, How It Works diagram, output format layer indicators, and missing-layers nudge section (added: 2026-02-25, dispatch: tack-2rli.6)
- For skills where all phases commit to the branch, use "abort = discard worktree" framing consistently in worktree isolation mode — emphasizes the clean-abort benefit (added: 2026-02-25, dispatch: tack-2rli.4)
- When a gap ticket references "Phase N (widening pass)", read the actual skill before accepting — the phase may serve a different purpose than stated, changing the design decision (added: 2026-02-25, dispatch: tack-2rli.13)

## Task-Relevant
