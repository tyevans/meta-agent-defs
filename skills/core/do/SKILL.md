---
name: do
description: "Use when you have a goal but don't know which skill fits. Routes to the right skill or pipeline. Keywords: do, run, auto, go, execute, start, help me."
argument-hint: "<goal in natural language>"
disable-model-invocation: false
user-invocable: true
allowed-tools: [Read, Glob, Grep, Skill, Task]
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

### Namespace Detection

If this skill was invoked as `/tl:do` (plugin mode), all skill invocations via the Skill tool must use the `tl:` prefix (e.g., `tl:gather` not `gather`). If invoked as `/do` (symlink mode), use unprefixed names. Detect the mode from how `$ARGUMENTS` was received — if the skill name in the invocation context includes a colon prefix, use that prefix for all downstream Skill tool calls.

---

## Phase 2: Match Goal to Skill or Pipeline

Using the catalog from Phase 1, match the user's goal to:

### Single-skill match
If the goal maps cleanly to one skill, select it. Prefer the most specific match. For example, "compare two auth approaches" maps to `/diff-ideas`, not `/gather`.

### Pipeline match
If the goal requires multiple steps, select a pipeline from the Primitive Chain Patterns or Decision Tree sections in INDEX.md (loaded in Phase 1).

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

### Pipeline (sequential, default)
For multi-step pipelines where each step depends on the previous output, announce the full pipeline and execute sequentially. After each step completes, the output stays in context for the next step to consume (pipe format).

```
Pipeline: /gather -> /distill -> /rank
Starting with /gather to collect findings.
```

Execute each step via the Skill tool. Between steps, do not add commentary — let the pipe format flow. Only intervene if a step produces no items (abort the pipeline and explain why).

For pipelines with 3+ steps, checkpoint per the compaction-resilience rule (`memory/scratch/do-checkpoint.md`). Delete the checkpoint on successful completion.

### Pipeline (parallel branches)
When the matched pipeline includes independent branches (e.g., two `/gather` calls on unrelated topics), dispatch them concurrently per the fan-out-protocol rule, then merge outputs before continuing to the next sequential step.

### Fork-context skills
If the matched skill has `context: fork`, it runs in an isolated context automatically. No special handling needed — just invoke it.

---

## Iterative Mode (Resume)

If interrupted mid-pipeline, /do can be resumed. On `resume:` prefix in `$ARGUMENTS`, read the checkpoint file, skip completed steps, continue from the first incomplete step.

---

## Guidelines

- **Trust the catalog.** Match against docs/INDEX.md, not your training knowledge. If INDEX.md doesn't list it, it's not available.
- **Prefer action over explanation.** Execute, don't recommend. Only present options when genuinely ambiguous.
- **Pass the original goal.** Pass the user's original `$ARGUMENTS` as args, not a rewritten version.
- **Sequential by default.** One skill at a time unless branches are clearly independent.
- **Abort on empty.** If a step produces 0 items, stop the pipeline and explain why.
- **Don't over-match.** If the goal doesn't map to any skill, say so plainly.
