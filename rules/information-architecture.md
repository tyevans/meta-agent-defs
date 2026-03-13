---
strength: should
freshness: 2026-03-12
---

# Information Architecture

How to place, find, and organize content in Claude Code projects.

## Context Tiers

Content belongs in exactly one tier based on how often sessions need it:

| Tier | Location | Loaded | Use for |
|------|----------|--------|---------|
| Always-on | `rules/`, hooks | Automatic, every session | Behavioral constraints, safety, formatting |
| Orientation | `CLAUDE.md`, `AGENTS.md` | Automatic, every session | Project shape, quick reference, navigation |
| Discoverable | `docs/`, indexes | On demand via tool call | Deep reference, catalogs, design rationale |
| Deep | subdirectories, artifacts | On demand via tool call | Full definitions, templates, raw content |

### Do

- Put behavioral constraints in `rules/`. If every session needs it, it must be passive context.
- Keep `CLAUDE.md` under 200 lines. It is orientation, not documentation.
- Keep index files (`AGENTS.md`, `INDEX.md`, `MEMORY.md`) as pointers only — link to content, do not inline it.
- Put detailed content in the artifact itself (`skills/<name>/SKILL.md`, `agents/<name>.md`), not in a central file.

### Do Not

- Do not duplicate content across tiers. One source of truth, one location.
- Do not put deep reference material in `CLAUDE.md` or `rules/` — it wastes passive context budget on content most sessions never need.
- Do not create index files that contain full artifact content. Indexes are maps, not warehouses.

## Naming and Placement

Use predictable structure so content can be found by convention instead of lookup.

### Do

- Follow existing directory conventions before creating new locations. `skills/<name>/SKILL.md` needs no registry because the pattern is the documentation.
- Use the project's controlled vocabulary consistently: "skill" (not "command" or "tool"), "rule" (not "guideline" or "policy"), "agent" (not "worker" or "bot").
- Place volatile content (inventories, counts, status) in centralized files (`MEMORY.md`, `INDEX.md`) where staleness is visible.
- Place stable content (definitions, schemas) at point of use where it rarely needs cross-referencing.

### Do Not

- Do not create new directories when an existing convention covers the use case. A new `utils/` folder is almost always wrong.
- Do not create a lookup table for something the naming convention already makes discoverable.
- Do not mix terminology — if the project calls it a "skill," never refer to it as a "command" in the same context.

## When Creating or Moving Content

1. **Determine the tier.** Ask: "Does every session need this?" → `rules/`. "Do most sessions need a pointer?" → `CLAUDE.md`. "Do some sessions need it?" → `docs/` with a link from an index. "Rarely needed?" → subdirectory.
2. **Check for an existing home.** Search for where similar content already lives before picking a new location.
3. **Write actionable content.** "Prefer X over Y" is testable. "Consider using X" is not. Rules that cannot be checked are suggestions — put them in docs, not rules.
4. **Update exactly one index.** If the content is discoverable or deep, add a pointer in the relevant index file. Do not add pointers in multiple places.
