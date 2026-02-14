# Information Architecture Principles

How to organize knowledge, configuration, and documentation in Claude Code projects so that context reaches the model at the right time with minimal retrieval cost.

## Principles

### 1. Convention Over Documentation

When structure is predictable, don't document it — enforce it through naming and placement. `skills/<name>/SKILL.md` needs no lookup table because the pattern is the documentation.

### 2. Progressive Disclosure

Put the minimum viable context at the root; link to detail. CLAUDE.md gives orientation. Rules give constraints. Docs give depth. A reader (human or model) should be able to stop at any level and still function.

### 3. Passive Context Over Active Retrieval

Prefer files that load automatically (CLAUDE.md, rules/, hooks) over files that require a tool call to discover. Every retrieval step costs tokens and risks the model not looking. If something matters for every session, make it passive.

### 4. Centralize What Rots, Distribute What's Stable

Inventories, counts, and status belong in one place (MEMORY.md, INDEX.md) where staleness is visible and updates are cheap. Stable definitions (skills, agents, rules) live at their point of use where they rarely need cross-referencing.

### 5. Small Root, Distributed Detail

Root index files (CLAUDE.md, AGENTS.md, INDEX.md) stay small — orientation only. Full content lives in the artifact itself. This keeps passive-context budgets tight and avoids the "mega-file" anti-pattern.

### 6. Controlled Vocabulary

Use the same terms everywhere. A "skill" is always a skill (not a "command" or "tool"). A "rule" is always a rule (not a "guideline" or "policy"). Consistency reduces ambiguity for both humans and models.

## Applying the Principles

**When creating a new artifact:** Pick the right tier — passive (rules/), discoverable (docs/), or on-demand (deep in a subdirectory). Ask: "Does every session need this?" If yes, make it a rule. If most sessions, put it in CLAUDE.md. If some sessions, put it in docs/ and link from an index.

**When reorganizing:** Resist the urge to create new directories. Check if an existing location follows convention. A new `utils/` folder is almost always wrong — put it where the convention says it goes.

**When writing rules:** Keep them actionable and testable. "Prefer X over Y" is better than "Consider using X." Rules that can't be checked are just suggestions.
