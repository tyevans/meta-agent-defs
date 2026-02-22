---
paths:
  - "skills/**/SKILL.md"
strength: should
freshness: 2026-02-21
---

# Pipe Format: Composable Skill Output Contract

All composable primitives (gather, distill, expand, transform, rank, diff-ideas, sketch, verify, filter, assess, decompose, critique, plan, merge) follow this output format so any primitive's output can feed another primitive's input.

## Output Structure

Every primitive emits a markdown block with these sections:

```
## <verb-noun phrase describing output>

**Source**: /<skill-name>
**Input**: <what was asked, one line>
**Pipeline**: /gather (12 items) -> /distill (5 items)

### Items (N)

1. **<title>** — <detail>
   - source: <file:line, URL, or "conversation context">
   - confidence: CONFIRMED | LIKELY | POSSIBLE

2. **<title>** — <detail>
   ...

### Summary

<One paragraph synthesis of the items above.>
```

## Rules

1. **Items section is always a numbered list.** Even single-item outputs use `1.`
2. **Confidence is optional.** Only include when the skill deals with uncertain claims (gather, verify).
3. **Source is optional per item.** Include when the item traces to a specific location.
4. **Summary is always present.** One paragraph, no bullet points.
5. **No YAML, no JSON.** Markdown only — LLMs parse it natively.
6. **Skill-specific sections go between Items and Summary.** Example: /rank adds a `### Criteria` section; /diff-ideas adds a `### Comparison` table.
7. **Pipeline line is always present.** When upstream pipe-format output is detected in context, construct the chain from prior `**Pipeline**` and `**Source**` fields showing each step and its item count (e.g., `/gather (12 items) -> /distill (5 items)`). When no upstream output is detected, use `(none — working from direct input)`.
8. **Items heading includes count.** Use `### Items (N)` where N is the number of items. This lets downstream primitives cross-check reported vs actual item counts.
9. **Validate upstream intake.** Before processing upstream output, state: "Reading N items from /skill-name output above." This makes detection failures visible without tooling.

## Input Contract

Primitives accept input from two sources:
- **$ARGUMENTS** — direct user input (always available)
- **Conversation context** — output from a prior primitive (the skill reads upward in context)

When a primitive detects structured output from a prior primitive (the `## ... / **Source**: /...` pattern), it uses that as input. Otherwise it treats $ARGUMENTS as the sole input.

**Disambiguation:** When multiple pipe-format blocks exist in context, the most recent one is used. To override, pass an explicit reference in $ARGUMENTS (e.g., `/distill from:gather-auth`). This prevents silent wrong-input consumption in sessions with multiple gather runs.

## Composability

Primitives compose by running sequentially in the same conversation:
```
User: /gather auth patterns in this codebase
  -> structured output in pipe format
User: /distill
  -> reads gather output from context, emits distilled pipe format
User: /rank by security risk
  -> reads distill output from context, emits ranked pipe format
```

No file passing, no explicit piping syntax. Context IS the pipe.

## Known Limitations

1. **Compression boundary.** Pipe chains must complete within a single uncompressed context window. Context compression is not pipe-format-aware — it may summarize structured output into prose, silently breaking detection and losing items. For chains that may span compression events, write intermediate results to a file and pass the file path as $ARGUMENTS to the next primitive. See docs/INDEX.md "File-Based Intermediate Results" for the concrete pattern.
2. **Silent failure modes.** The format has no schema validation or checksums. If detection fails, the downstream primitive falls back to $ARGUMENTS without error. The item count in `### Items (N)` and the upstream intake statement (Rule 9) are the primary observability mechanisms — but they are self-reported, not enforced.
3. **Metadata drift.** The Pipeline provenance line and per-item source/confidence metadata are more drift-prone than the core numbered list structure. Missing metadata degrades provenance tracking but does not break composition. The numbered list is the only hard structural requirement.
