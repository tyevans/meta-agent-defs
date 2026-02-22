---
name: gather
description: "Collect information on a topic into structured findings with sources and confidence levels. The universal input primitive — use before distill, rank, or verify. Keywords: research, investigate, collect, find, search, explore."
argument-hint: "<topic or question>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, WebSearch, WebFetch
---

# Gather: Structured Information Collection

You are running the **gather** primitive — collecting information on a topic into structured findings with sources and confidence levels. Topic: **$ARGUMENTS**

## When to Use

- When starting research on a new topic or question
- Before distilling, ranking, or verifying — gather is the universal input generator
- When refining or expanding findings from a prior primitive (gather can compose with itself)
- When the user asks you to "find", "investigate", "research", or "collect" information

## Process

### 1. Search Code First

Use Grep, Glob, and Read to search the codebase for relevant information. Code is faster and more reliable than web search.

### 2. Search Web Second

If codebase results are insufficient or the topic requires external context, use WebSearch and WebFetch to gather additional information.

### 3. Emit Structured Findings

Output in pipe format:

- **Header**: `## [Findings on ...]`
- **Metadata**: `**Source**: /gather`, `**Input**: [one-line topic]`, `**Pipeline**: (none — working from direct input)` (or append to upstream pipeline if composing with prior primitive)
- **Items**: Numbered list with title, detail, source (file:line or URL), and confidence (CONFIRMED/LIKELY/POSSIBLE)
- **Summary**: One paragraph synthesis of all findings

Each finding must have a source. Use confidence levels when claims are uncertain.

## Modes

This primitive has one implicit secondary behavior:

| Behavior | Description |
|----------|-------------|
| **Confidence labeling** | Every finding gets a CONFIRMED/LIKELY/POSSIBLE label. This is a lightweight /assess operation embedded in gather's output. |
| **Code-first priority** | Search order is hardcoded: codebase first, web second. This is a domain bias (optimized for software engineering), not a user-configurable parameter. |

Confidence labeling is embedded by design — separating it would require a `/gather -> /assess` chain for every collection, adding overhead without value. The code-first priority reflects the target domain; for non-code topics, web results will still be used when codebase results are insufficient.

## Guidelines

- Code sources are more reliable than web sources — prioritize codebase over web
- If composing with a prior primitive (detected via pipe format in context), refine or expand those findings
- Confidence levels: CONFIRMED (verified in code/docs), LIKELY (strong evidence), POSSIBLE (weak evidence or speculation)
- Keep findings concise — the distill primitive can synthesize later
- If the topic is vague or broad, gather what you can and note scope limits in the summary
