---
name: do
description: "Primary entrypoint for composable skills. Reads the canonical skill catalog, matches your goal to the right skill or pipeline, and executes it. Use when you have a goal and want the system to pick the right approach. Keywords: do, run, auto, go, execute, start, help me."
argument-hint: "<goal in natural language>"
disable-model-invocation: false
user-invocable: true
allowed-tools: [Read, Glob, Grep, Skill, Task, "Bash(bd:*)"]
context: inline
---

# Do: Goal-Directed Skill Dispatch

You are running the **do** skill — the primary entrypoint that reads the canonical skill catalog, selects the best skill or pipeline for the user's goal, and executes it.

**Goal:** $ARGUMENTS

## When to Use

- When the user has a goal but doesn't know which skill to run
- As the default entrypoint for any natural-language request that maps to a skill
- When a user wants the system to "just do it" without manually selecting skills

## How It Works

```
Read canonical catalog (docs/INDEX.md)
  -> Match goal to best skill or pipeline
    -> Confirm selection with user
      -> Execute via Skill tool
```

---

## Phase 0: Argument Gate

If `$ARGUMENTS` is empty, ask the user: "What do you want to do? Describe your goal in a sentence." Do not proceed without a goal.

---

## Phase 1: Load Canonical Catalog

Read `docs/INDEX.md` to get the full skill catalog. This is the single source of truth for available skills — do not glob individual SKILL.md files.

Extract from INDEX.md:
- **Decision Tree** — the "I want to..." section mapping goals to skills
- **Skills by Category** — the tables listing every skill with purpose and context type
- **Primitive Chain Patterns** — canonical composition sequences

---

## Phase 2: Match Goal to Skill or Pipeline

Using the catalog from Phase 1, match the user's goal to:

### Single-skill match
If the goal maps cleanly to one skill, select it. Prefer the most specific match. For example, "compare two auth approaches" maps to `/diff-ideas`, not `/gather`.

### Pipeline match
If the goal requires multiple steps, select a pipeline from the Primitive Chain Patterns or Decision Tree. Common patterns:

| Goal shape | Pipeline |
|-----------|----------|
| "Research X" | `/gather` -> `/distill` -> `/rank` |
| "Compare A vs B" | `/diff-ideas` |
| "Plan how to build X" | `/decompose` -> `/plan` |
| "Explore X" | `/blossom` or `/fractal` |
| "Review code" | `/review` |
| "Find what's wrong with X" | `/critique` |
| "Understand X deeply" | `/fractal` |
| "Build X iteratively" | `/tracer` |
| "Set up a new project" | `/bootstrap` |
| "What should I work on?" | `/advise` |

### Ambiguity resolution
If the goal could map to multiple skills with meaningfully different outcomes, present the top 2 options with a one-line tradeoff and ask which the user prefers. Do not present more than 2 options.

---

## Phase 3: Announce and Execute

### Single skill
State what you're running and why in one line, then invoke it:

```
Running /gather — your goal is a research question that needs information collection first.
```

Then call the Skill tool with the matched skill name and pass the user's original goal as args.

### Pipeline
For multi-step pipelines, announce the full pipeline, then execute each step sequentially. After each step completes, the output stays in context for the next step to consume (pipe format).

```
Pipeline: /gather -> /distill -> /rank
Starting with /gather to collect findings.
```

Execute each step via the Skill tool. Between steps, do not add commentary — let the pipe format flow. Only intervene if a step produces no items (abort the pipeline and explain why).

### Fork-context skills
If the matched skill has `context: fork`, it runs in an isolated context automatically. No special handling needed — just invoke it.

---

## Guidelines

- **Trust the catalog.** Match against docs/INDEX.md, not your training knowledge of what skills exist. If INDEX.md doesn't list it, it's not available.
- **Prefer action over explanation.** The whole point of /do is to execute, not recommend. Only present options when genuinely ambiguous.
- **Pass the original goal.** When invoking the Skill tool, pass the user's original `$ARGUMENTS` as the args, not a rewritten version.
- **Pipelines are sequential.** Execute one skill at a time. Each step's pipe-format output feeds the next via conversation context.
- **Abort on empty.** If a pipeline step produces 0 items, stop the pipeline and tell the user why rather than feeding nothing to the next step.
- **Don't over-match.** If the goal doesn't map to any skill (e.g., "make me a sandwich"), say so plainly rather than forcing a bad match.
