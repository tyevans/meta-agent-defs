---
name: drive
description: "Use when you have a plan document and want autonomous implementation until it's done. Loops sprint/retro cycles, commits regularly, keeps going until complete or stopped. Keywords: drive, autopilot, autonomous, grind, build, implement, sustained, continuous."
argument-hint: "<path to plan document or epic>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Write, Edit, Grep, Glob, Bash(git:*), Bash(mkdir:*), Skill, Task
---

# Drive: Sustained Autonomous Implementation

You are running **Drive** — a sustained implementation loop that works through a plan document until complete or until the user explicitly stops you. Drive orchestrates sprint/retro cycles, delegates sub-tasks via `/do`, applies TDD where it fits, commits regularly, and keeps documentation current.

**Plan source:** $ARGUMENTS

## When to Use

- When a plan, spec, or product document exists and the work is "just execute"
- When the user wants you to go autonomous and keep building until stopped
- When multiple sprint cycles are needed to complete a body of work
- After `/blossom` or `/spec` has produced a plan that needs sustained execution

## Overview

```
Load plan + assess current state
  -> Decompose remaining work into sprint batch
    -> Execute batch via /sprint
      -> Run /retro after each sprint
        -> Update documentation for completed areas
          -> Loop back to assess (or stop if plan complete)
```

---

## Phase 0: Load Plan and Orient

### 0a. Resolve the Plan Document

If `$ARGUMENTS` is empty, check for plan documents in order:
1. `memory/epics/*/epic.md` — most recent epic
2. `.claude/plan.md` or `docs/plan.md`
3. Any `*design*.md` or `*spec*.md` in docs/

If nothing is found, ask the user: "What plan or document should I implement against?" Do not proceed without a plan.

If `$ARGUMENTS` is a file path, read that file.

### 0b. Read the Plan

Read the plan document end-to-end. Extract:
- **Vision/goal**: What the finished product looks like
- **Sections/features**: Discrete areas of work
- **Quality criteria**: Any stated UX, accessibility, or code quality standards
- **Constraints**: Technology choices, patterns, or limitations

### 0c. Assess Current State

Determine what already exists vs. what remains:

```bash
git log --oneline -20
```

Check your project's task tracker for current status (open tasks, in-progress work, blockers). Read key files referenced in the plan to see how much is already implemented. Build a mental map of **done**, **in-progress**, and **remaining**.

### 0d. Write Drive State

Write initial state to `memory/scratch/drive-state.md`:

```markdown
# Drive State

**Plan**: <path to plan document>
**Started**: <date>
**Sprint count**: 0

## Completed Areas
(none yet)

## Current Sprint Focus
(not yet planned)

## Remaining Areas
- <area 1>: <brief status>
- <area 2>: <brief status>
...
```

This file is the recovery point if context compacts. Update it after each sprint.

---

## Phase 1: Plan Sprint Batch

### 1a. Select Next Focus

From the remaining areas, select a batch of 3-8 tasks for the next sprint. Prioritize by:

1. **Dependencies**: What unblocks other work?
2. **User-facing value**: What delivers visible progress? Prefer features users will touch.
3. **Vertical slices**: Prefer complete thin slices over partial thick layers.
4. **Documentation gaps**: If a completed area lacks docs, include a doc task.

### 1b. Create Backlog Items

For each task in the batch, track it using your preferred task tracking approach. Record the title, description, and priority. Set up dependencies where ordering matters.

### 1c. Apply Quality Lenses

Before dispatching, review each task through these lenses:

- **UX Joy**: Will this be pleasant to use? Consider both power users and people who won't know where to start. If the task touches user-facing surfaces, the task description should include UX guidance.
- **TDD Fit**: Would this task benefit from test-first development? If yes, note "Apply /test-strategy" in the task description. Good candidates: business logic, data transformations, validation rules, state machines.
- **SOLID/DRY**: Are we about to duplicate something? Check if similar patterns exist. If a task would introduce repetition, redefine it to extract a shared abstraction first.
- **Documentation**: Will this task change behavior that's currently documented? If yes, include "Update docs" in the task description.

---

## Phase 2: Execute Sprint

Invoke `/sprint` with the focus area:

```
/sprint <focus area description>
```

Sprint handles team dispatch, learning loops, and progress tracking. Let it run to completion.

**If no team is assembled**: Sprint will detect this. Either:
- Run `/assemble` first if the project warrants a team
- Or work through tasks directly using `/do <task>` for each item, which routes to the appropriate skill

### Mid-Sprint Quality Checks

Between task dispatches (or between `/do` invocations), verify:
- **Commits are happening**: Check `git log --oneline -5` periodically. If 3+ tasks complete without commits, pause and commit.
- **Tests pass**: If the project has a test runner, run it after every 2-3 tasks. If tests fail, fix before continuing.
- **No drift from plan**: Reread the relevant section of the plan document if the work feels like it's wandering.

---

## Phase 3: Retro and Learn

After the sprint completes, run:

```
/retro sprint cycle N
```

Let retro capture what worked, what didn't, and persist durable learnings.

---

## Phase 4: Update Drive State and Documentation

### 4a. Update Drive State

Read and update `memory/scratch/drive-state.md`:
- Move completed areas from "Remaining" to "Completed"
- Update sprint count
- Note any blockers or discoveries
- Set the next sprint focus

### 4b. Update Documentation

For each area completed in this sprint, check whether documentation needs updating:

1. **README / getting started**: Does the new feature need mention?
2. **API docs / usage guides**: Are new workflows documented?
3. **Configuration docs**: Are new options explained?
4. **Architecture docs**: Did the structure change?

Fill gaps immediately. Documentation that trails implementation rots fast.

### 4c. Commit Documentation

```bash
git add <doc files>
git commit -m "docs: update documentation for <area>"
```

---

## Phase 5: Loop or Stop

### Check Completion

Reread the plan document. Compare against the "Completed Areas" in drive state.

**If remaining areas exist**: Return to Phase 1. Select the next batch and continue.

**If the plan appears complete**: Announce completion with a summary:

```markdown
## Drive Complete

**Plan**: <path>
**Sprints**: <count>
**Areas completed**: <list>

### What was built
<1-paragraph summary of what now exists>

### Documentation updated
<list of docs touched>

### Remaining gaps (if any)
<anything the plan called for that couldn't be completed, with explanation>
```

Then run a final `/retro` covering the full drive.

**If the user stops you**: Gracefully halt after the current task. Update drive state with where you stopped so a future `/drive resume` can pick up.

---

## Iterative Mode (Resume)

Drive supports resuming from a prior run.

**How to resume:**
> /drive resume: <plan path or "continue">

**On resume:**
1. Detect the `resume:` prefix in `$ARGUMENTS`
2. Read `memory/scratch/drive-state.md`
3. Report what sprints have completed and what remains
4. Skip completed areas and continue from the next remaining area
5. Return to Phase 1

---

## Guidelines

1. **The plan is the authority.** When in doubt about scope or priority, reread the plan. Don't invent features it doesn't call for.
2. **Commit early, commit often.** Every completed task should produce at least one commit. Don't batch up large uncommitted changes.
3. **UX for everyone.** Consider both the technically adept and the person who has no idea where to start. Interfaces should guide, not assume knowledge.
4. **TDD where it pays.** Not everything needs test-first. Business logic, validation, and state machines do. Markup, configuration, and glue code usually don't. Use `/test-strategy` for the former.
5. **SOLID and DRY are guardrails, not goals.** Extract abstractions when duplication is real (3+ instances), not when it's hypothetical. Prefer simple code over clever code.
6. **Documentation is part of done.** A feature without docs is a feature nobody can use. Update docs in the same sprint that builds the feature.
7. **Vertical slices over horizontal layers.** Build one complete thing rather than half of everything. Users can try a complete thin slice; they can't try a half-built layer.
8. **Compaction resilience.** This skill loops indefinitely. `memory/scratch/drive-state.md` is the recovery file. Update it religiously at Phase 4. If context compacts, reread it and resume.
9. **Respect the stop signal.** When the user says stop, stop cleanly. Update state, commit work, run retro. Don't try to squeeze in "just one more thing."

See also: /sprint (execution engine), /retro (post-sprint learning), /do (sub-task routing), /test-strategy (TDD workflow), /tracer (thin-slice implementation pattern), /handoff (session transition when drive spans sessions).
