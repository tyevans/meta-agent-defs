---
name: discover
description: "Use when you want to browse available skills or find the right pipeline for a workflow. Lists skills matching your goal with composition suggestions. Keywords: recommend, find skill, which skill, what skill, help, navigate, discover, pipeline."
argument-hint: "<described goal or workflow>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Glob
---

# Discover: Goal-Based Skill Recommendation

You are running the **discover** skill — recommending the right skills or pipeline for a described goal. Goal: **$ARGUMENTS**

## When to Use

- When you are not sure which skill to reach for for a given workflow
- When you want to know which skills compose well for a multi-step pipeline
- When onboarding to the skill system and want an entry point
- When a user describes a goal in natural language and needs a skill recommendation

## Overview

```
Read all skill descriptions
  -> Match goal to 2-4 most relevant skills
    -> Identify canonical pipeline if applicable
      -> Output pipe-format recommendation
```

---

## Phase 1: Load Skill Descriptions

Use Glob to find all skill definition files:

```
skills/**/SKILL.md
```

For each file found, Read it and extract:
- The `name` field from YAML frontmatter
- The `description` field from YAML frontmatter

Do not read the full body of each skill — frontmatter only is sufficient for matching. If a file has no `description` field, skip it.

### Namespace Detection

If this skill was invoked as `/tl:discover` (plugin mode), prefix all skill names in the output with `tl:` (e.g., `/tl:gather` not `/gather`). If invoked as `/discover` (symlink mode), use unprefixed names.

---

## Phase 2: Match Goal to Skills

Semantically match **$ARGUMENTS** against the collected descriptions.

Select the **2 to 4 skills** that best fit the described goal. Prefer skills where the description's "when to use" language closely matches the goal's framing.

For each selected skill, note:
- **Skill name** (the slash command, e.g., `/gather`)
- **One-line description** drawn from the `description` field
- **Why it fits** — one sentence specific to the user's stated goal

### Canonical Pipeline Detection

Read `docs/INDEX.md` § Primitive Chain Patterns to check whether the goal maps to a known pipeline. If it does, include the full pipeline in the output.

---

## Phase 3: Emit Recommendation

Output in pipe format:

```markdown
## Skill Recommendations for: [goal summary]

**Source**: /discover
**Input**: [user's goal, one line]
**Pipeline**: (none — working from direct input)

### Items (N)

1. **/skill-name** — [one-line description from frontmatter]
   - Why it fits: [one sentence specific to the user's goal]

2. **/skill-name** — [one-line description from frontmatter]
   - Why it fits: [one sentence specific to the user's goal]

[... up to 4 items]

### Suggested Pipeline (if applicable)

If your goal maps to a canonical workflow:

`/skill-a` -> `/skill-b` -> `/skill-c`

[One sentence explaining the flow and what each step contributes.]

### Summary

[One paragraph. Recommend where to start, explain why the top skill fits best, and note any tradeoffs between options.]
```

---

## Guidelines

- **Prefer specificity over completeness.** Two well-matched recommendations beat five loosely-matched ones.
- **Name the tradeoff.** If two skills both fit, say which is better for breadth vs. depth, or exploration vs. execution.
- **Do not hardcode the skill list.** Always read from `skills/**/SKILL.md` at runtime so the recommendation reflects the actual installed skill set.
- **Pipeline is optional.** Only include the Suggested Pipeline section if the goal clearly maps to a multi-step workflow. Single-skill goals do not need a pipeline.
- **Argument handling**: If `$ARGUMENTS` is empty, ask the user to describe their goal in one sentence before proceeding.
