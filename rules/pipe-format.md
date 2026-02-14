# Pipe Format: Composable Skill Output Contract

All composable primitives (gather, distill, rank, diff-ideas, sketch, verify, filter, assess, decompose, critique, plan, merge) follow this output format so any primitive's output can feed another primitive's input.

## Output Structure

Every primitive emits a markdown block with these sections:

```
## <verb-noun phrase describing output>

**Source**: /<skill-name>
**Input**: <what was asked, one line>
**Pipeline**: /gather (12 items) -> /distill (5 items)

### Items

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

## Input Contract

Primitives accept input from two sources:
- **$ARGUMENTS** — direct user input (always available)
- **Conversation context** — output from a prior primitive (the skill reads upward in context)

When a primitive detects structured output from a prior primitive (the `## ... / **Source**: /...` pattern), it uses that as input. Otherwise it treats $ARGUMENTS as the sole input.

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
