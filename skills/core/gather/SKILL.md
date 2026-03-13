---
name: gather
description: "Use when you need to research a topic before analyzing it. Produces structured findings with sources. Feed output to distill, rank, or verify. Keywords: research, investigate, collect, find, search, explore."
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

Output in pipe format. Each finding must have a `source:` and `confidence:` per-item metadata.

## Parallel Collection Mode

**Trigger**: When the topic clearly spans 3 or more distinct source types (codebase files, web/external docs, and config/settings) and the breadth warrants concurrent collection.

Dispatch per the fan-out-protocol rule (loaded alongside this skill). Gather-specific details:

**Agents** (dispatch 2-3, one per source type):

| Agent | Scope | Prompt focus |
|-------|-------|-------------|
| Codebase | Implementation files, tests, inline docs | "Search the codebase for [topic]. Use Grep, Glob, Read." |
| Web/docs | External documentation, references, changelogs | "Search the web for [topic]. Use WebSearch, WebFetch." |
| Config | Env vars, deployment config, project settings | "Search for config related to [topic]. Look for .env, YAML/JSON, manifests." |

All agent prompts must end with: "Return findings as a numbered list: title, detail, source (file:line or URL), confidence (CONFIRMED/LIKELY/POSSIBLE)."

**Merge**: Deduplicate overlapping results, higher-confidence items take precedence. Emit as a single pipe-format block with `**Mode**: parallel (N agents)` after the Pipeline line.

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
- If composing with upstream pipe-format output, refine or expand those findings
- Keep findings concise — the distill primitive can synthesize later
- If the topic is vague or broad, gather what you can and note scope limits in the summary
- In parallel mode, cap agent count at 3 to avoid API throttling — merge any additional source types into the closest existing agent
