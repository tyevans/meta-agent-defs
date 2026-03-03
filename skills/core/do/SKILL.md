---
name: do
description: "Primary entrypoint for composable skills. Reads the canonical skill catalog, matches your goal to the right skill or pipeline, and executes it. Use when you have a goal and want the system to pick the right approach. Keywords: do, run, auto, go, execute, start, help me."
argument-hint: "<goal in natural language>"
disable-model-invocation: false
user-invocable: true
allowed-tools: [Read, Glob, Grep, Skill, Task, "Bash(bd:*)", "Bash(tk:*)"]
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

### Pipeline (sequential, default)
For multi-step pipelines where each step depends on the previous output, announce the full pipeline and execute sequentially. After each step completes, the output stays in context for the next step to consume (pipe format).

```
Pipeline: /gather -> /distill -> /rank
Starting with /gather to collect findings.
```

Execute each step via the Skill tool. Between steps, do not add commentary — let the pipe format flow. Only intervene if a step produces no items (abort the pipeline and explain why).

Write a checkpoint to `memory/scratch/do-checkpoint.md` after each completed step so the pipeline can be resumed if interrupted:

```
# /do pipeline checkpoint
Goal: <original goal>
Pipeline: /gather -> /distill -> /rank
Completed: /gather
Remaining: /distill -> /rank
Last output: <one-line summary of last step's output>
```

Delete the checkpoint file when the pipeline completes successfully.

### Pipeline (parallel branches)
When the matched pipeline includes independent branches — steps that operate on unrelated topics or sources and don't need each other's output — dispatch them as background Task agents simultaneously instead of sequentially.

**Independence check:** Branches are independent when:
- They target different subjects (e.g., two `/gather` calls on unrelated topics)
- Neither branch's output is required as input to the other
- They produce outputs that are later merged or ranked together

**Parallel dispatch:**

```
Parallel branches detected: /gather [topic A] + /gather [topic B]
Dispatching both simultaneously, then merging outputs.
```

Dispatch each branch via Task with `run_in_background=true`:

```
Task({
  subagent_type: "general-purpose",
  run_in_background: true,
  prompt: "Run /[skill] with this goal: [branch-specific goal]. Return your full pipe-format output."
})
```

Wait for all parallel branches to complete, then merge their outputs in context before continuing to the next sequential step (e.g., `/rank` or `/distill` across combined results).

**Limit:** Dispatch at most 4 parallel branches. If more exist, serialize the excess.

### Fork-context skills
If the matched skill has `context: fork`, it runs in an isolated context automatically. No special handling needed — just invoke it.

---

## Iterative Mode (Resume)

If interrupted mid-pipeline, /do can be resumed from where it left off.

**How to resume:**
> /do resume: <follow-up or "continue">

**On resume, the skill should:**
1. Detect the `resume:` prefix in `$ARGUMENTS`
2. Read `memory/scratch/do-checkpoint.md` if it exists
3. Report which steps have already completed and what remains
4. Skip completed steps and continue from the first incomplete step
5. Treat checkpoint items as confirmed prior output — do not re-run them

If no checkpoint file exists, inform the user and restart from Phase 0.

---

## Guidelines

- **Trust the catalog.** Match against docs/INDEX.md, not your training knowledge of what skills exist. If INDEX.md doesn't list it, it's not available.
- **Prefer action over explanation.** The whole point of /do is to execute, not recommend. Only present options when genuinely ambiguous.
- **Pass the original goal.** When invoking the Skill tool, pass the user's original `$ARGUMENTS` as the args, not a rewritten version.
- **Sequential by default.** Execute pipelines one skill at a time unless branches are clearly independent. Each step's pipe-format output feeds the next via conversation context.
- **Parallel only when independent.** Do not parallelize steps that consume each other's output. If in doubt, serialize.
- **Checkpoint multi-step pipelines.** Write `memory/scratch/do-checkpoint.md` after each completed step in a pipeline with 3 or more steps.
- **Abort on empty.** If a pipeline step produces 0 items, stop the pipeline and tell the user why rather than feeding nothing to the next step.
- **Don't over-match.** If the goal doesn't map to any skill (e.g., "make me a sandwich"), say so plainly rather than forcing a bad match.
