---
paths:
  - "skills/**/SKILL.md"
strength: should
freshness: 2026-03-11
---

# Pipe Format

Consistent output structure so any skill's output can feed another skill's input. Context IS the pipe — no file passing needed.

## Structure

```
## <verb-noun phrase>

**Source**: /<skill-name>
**Input**: <one-line description of what was processed>
**Pipeline**: <provenance chain> or (none — working from direct input)

### Items (N)

1. **<title>** — <detail>
2. **<title>** — <detail>

### Summary

<One paragraph synthesis.>
```

## Rules

1. **Items are always a numbered list.** Even single-item outputs.
2. **Source line is always present.** This is how downstream skills detect upstream output.
3. **Summary is always present.** One paragraph.
4. **Markdown only.** No YAML or JSON — LLMs parse markdown natively.
5. **Skill-specific sections go between Items and Summary.** /rank adds `### Criteria`, /diff-ideas adds `### Comparison`, etc.

## Input Detection

Skills accept input from `$ARGUMENTS` (user input) or conversation context (prior skill output). When a skill detects the `## ... / **Source**: /...` pattern above it in context, it uses that as input and reads the `**Pipeline**` field to construct provenance. Otherwise it uses `$ARGUMENTS`.

When multiple pipe-format blocks exist, the most recent one wins.

## Pipeline Provenance

The `**Pipeline**` field tracks the chain of skills that produced the current output. Each skill appends itself:

- **No upstream**: `(none — working from direct input)`
- **With upstream**: `<upstream chain> -> /<skill> (N items)`
- **Merged branches**: Use `+` to show merged inputs (e.g., `/gather (8) + /gather (6) -> /merge (10)`)

## Confidence Levels

When findings involve claims about code, docs, or external systems, tag each item:

- **CONFIRMED** — verified by reading code or docs; evidence cited
- **LIKELY** — strong evidence from multiple signals but incomplete verification
- **POSSIBLE** — suspicious pattern or weak evidence; needs deeper investigation

## Composition Rules

When a skill consumes upstream pipe-format output:

1. **Preserve source attribution** — carry forward `source:` metadata from input items
2. **Preserve confidence levels** — do not downgrade; upgrade only when new evidence supports it
3. **Maintain item identity** — unless the skill explicitly reorders (rank), filters (filter), or merges (merge), input count equals output count

## Limitation

Long chains risk context compression destroying structured output. For chains spanning 3+ operations, write intermediate results to `.claude/tackline/memory/scratch/` per `rules/memory-layout.md`.
