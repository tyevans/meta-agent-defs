---
name: gather
description: "Collect information on a topic into structured findings with sources and confidence levels. The universal input primitive — use before distill, rank, or verify. Keywords: research, investigate, collect, find, search, explore."
argument-hint: "<topic or question>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, WebSearch, WebFetch, Task
---

# Gather: Structured Information Collection

You are running the **gather** primitive — collecting information on a topic into structured findings with sources and confidence levels. Topic: **$ARGUMENTS**

## When to Use

- When starting research on a new topic or question
- Before distilling, ranking, or verifying — gather is the universal input generator
- When refining or expanding findings from a prior primitive (gather can compose with itself)
- When the user asks you to "find", "investigate", "research", or "collect" information

## Process (default: serial)

### 1. Assess Source Breadth

Before searching, identify the distinct source types the topic spans:

- **Codebase** — implementation files, configs, tests
- **Web/docs** — external documentation, references, changelogs
- **Settings/config** — environment variables, deployment config, project-level settings

If the topic clearly spans **3 or more** of these source types, consider switching to **Parallel Collection Mode** (see below) to gather concurrently. Otherwise, proceed serially.

### 2. Search Code First

Use Grep, Glob, and Read to search the codebase for relevant information. Code is faster and more reliable than web search.

### 3. Search Web Second

If codebase results are insufficient or the topic requires external context, use WebSearch and WebFetch to gather additional information.

### 4. Emit Structured Findings

Output in pipe format:

- **Header**: `## [Findings on ...]`
- **Metadata**: `**Source**: /gather`, `**Input**: [one-line topic]`, `**Pipeline**: (none — working from direct input)` (or append to upstream pipeline if composing with prior primitive)
- **Items**: Numbered list with title, detail, source (file:line or URL), and confidence (CONFIRMED/LIKELY/POSSIBLE)
- **Summary**: One paragraph synthesis of all findings

Each finding must have a source. Use confidence levels when claims are uncertain.

## Parallel Collection Mode

**Trigger**: When the topic clearly spans 3 or more distinct source types (codebase files, web/external docs, and config/settings) and the breadth warrants concurrent collection.

**In parallel mode:**

1. Dispatch 2-3 background Task agents, each focused on one source type. Launch all at once (`run_in_background: true`):

   **Codebase agent** — searches implementation files, tests, and inline docs:
   ```
   Task({
     subagent_type: "Explore",
     run_in_background: true,
     prompt: "Search the codebase for [topic]. Use Grep, Glob, and Read.
              Return findings as a numbered list: title, detail, file:line source, confidence (CONFIRMED/LIKELY/POSSIBLE)."
   })
   ```

   **Web/docs agent** — searches external documentation and references:
   ```
   Task({
     subagent_type: "Explore",
     run_in_background: true,
     prompt: "Search the web for [topic]. Use WebSearch and WebFetch.
              Return findings as a numbered list: title, detail, URL source, confidence (CONFIRMED/LIKELY/POSSIBLE)."
   })
   ```

   **Config/settings agent** — searches environment variables, deployment config, and project-level settings (only when config is a distinct source type for this topic):
   ```
   Task({
     subagent_type: "Explore",
     run_in_background: true,
     prompt: "Search for config and settings related to [topic]. Look for .env files, YAML/JSON config, deployment manifests.
              Return findings as a numbered list: title, detail, file:line source, confidence (CONFIRMED/LIKELY/POSSIBLE)."
   })
   ```

2. Collect all agent results as they complete.

3. Merge findings into a single unified numbered list, deduplicating overlapping results and preserving per-item source and confidence metadata. Higher-confidence items from any agent take precedence when deduplicating.

4. Emit the merged set as a single pipe-format output block. Note the mode used: `**Mode**: parallel (N agents)` in the metadata block after the Pipeline line.

## Modes

This primitive has one implicit secondary behavior:

| Behavior | Description |
|----------|-------------|
| **Confidence labeling** | Every finding gets a CONFIRMED/LIKELY/POSSIBLE label. This is a lightweight /assess operation embedded in gather's output. |
| **Code-first priority** (default) | Search order: codebase first, web second. This is a domain bias optimized for software engineering, not a user-configurable parameter. |
| **Parallel collection** (opt-in) | When 3+ distinct source types are identified, dispatch concurrent agents per source type and merge results. |

Confidence labeling is embedded by design — separating it would require a `/gather -> /assess` chain for every collection, adding overhead without value. The code-first priority reflects the target domain; for non-code topics, web results will still be used when codebase results are insufficient.

## Guidelines

- Code sources are more reliable than web sources — prioritize codebase over web
- If composing with a prior primitive (detected via pipe format in context), refine or expand those findings
- Confidence levels: CONFIRMED (verified in code/docs), LIKELY (strong evidence), POSSIBLE (weak evidence or speculation)
- Keep findings concise — the distill primitive can synthesize later
- If the topic is vague or broad, gather what you can and note scope limits in the summary
- In parallel mode, cap agent count at 3 to avoid API throttling — merge any additional source types into the closest existing agent
