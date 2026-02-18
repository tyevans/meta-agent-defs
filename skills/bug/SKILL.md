---
name: bug
description: "File a structured bug report in the beads backlog from a description, upstream pipe-format output (/critique, /review), or conversation context. Extracts affected artifact, repro steps, and priority. Keywords: bug, report, issue, defect, file, submit, track, broken."
argument-hint: "<description of the bug>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*)
context: inline
---

# Bug: File a Structured Bug Report

You are running the **bug** skill — filing a structured bug report in the beads backlog. Input: **$ARGUMENTS**

## When to Use

- After discovering a defect during manual testing or code review
- After /critique or /review surfaces a flaw worth tracking
- When the user says "file a bug", "this is broken", or "track this issue"
- To bridge findings from any pipe-format output into actionable beads issues

## Overview

```
Detect input [pipe-format or direct description]
  -> Extract details [artifact, symptom, repro, severity]
    -> Confirm with user [priority, scope]
      -> File in beads [bd create --type=bug]
        -> Report [pipe format]
```

---

## Phase 0: Detect Input Source

Search conversation context for prior pipe-format output (the `## ... / **Source**: /...` pattern). Prioritize items tagged as FLAW, RISK, GAP, or with severity HIGH/CRITICAL.

- **If pipe-format found**: Extract bug candidates from items. Each FLAW or HIGH-severity item is a candidate.
- **If no pipe-format found**: Use $ARGUMENTS as the bug description directly.

State what you found: "Reading N items from /skill-name output above" or "Working from direct description."

---

## Phase 1: Extract Bug Details

For each bug candidate, determine:

| Field | How to Extract |
|-------|---------------|
| **Artifact** | Which skill, agent, rule, hook, or file is affected? Search the codebase if unclear. |
| **Symptom** | What goes wrong? Incorrect output, crash, silent failure, missing behavior? |
| **Repro steps** | What sequence triggers the bug? Include the skill invocation or input if known. |
| **Expected behavior** | What should happen instead? |
| **Severity** | P0 (crash/data loss), P1 (wrong output), P2 (minor/cosmetic), P3 (edge case) |

If extracting from pipe-format, map upstream metadata:
- FLAW with HIGH severity -> P1
- FLAW with MEDIUM severity -> P2
- RISK with HIGH severity -> P1
- GAP -> P2 (missing behavior, not broken behavior)

If details are ambiguous, ask the user rather than guessing.

---

## Phase 2: Confirm Before Filing

Present the extracted bug report to the user for confirmation before creating the issue:

```
Bug: <title>
Artifact: <file or skill affected>
Severity: P<N>
Symptom: <what goes wrong>
Repro: <steps to trigger>
Expected: <correct behavior>
```

If multiple bugs were extracted from pipe-format input, present all of them and ask the user which to file. Do not silently file all of them.

---

## Phase 3: File in Beads

For each confirmed bug, create a beads issue:

```bash
bd create --title="<artifact>: <concise symptom>" --type=bug --priority=<N> \
  --description="<full description with symptom, repro steps, and expected behavior>"
```

Title format: `<affected artifact>: <what's broken>` (e.g., "/gather: silently drops items past 20 results")

If the bug relates to an existing epic, add it as a child:

```bash
bd create --title="<artifact>: <symptom>" --type=bug --priority=<N> \
  --parent=<epic-id> \
  --description="<full description>"
```

---

## Phase 4: Report

Emit the result in pipe format:

```markdown
## Filed Bug Reports

**Source**: /bug
**Input**: <one-line summary of input>
**Pipeline**: <upstream chain -> /bug (N items)> or (none — working from direct input)

### Items (N)

1. **<bead-id>: <title>** — P<N>, <artifact>
   - source: <file path or skill name>

### Summary

<One paragraph: what was filed, severity distribution, recommended next action (e.g., "run /sprint to dispatch fixes" or "add to next consolidation pass").>
```

---

## Guidelines

- **Always confirm before filing.** Never silently create issues from pipe-format input.
- **One bug per issue.** Do not bundle multiple symptoms into a single bead.
- **Title starts with the artifact name.** This makes the backlog scannable.
- **Preserve provenance.** If the bug came from /critique or /review output, include that in the description (e.g., "Discovered via /review on 2026-02-17").
- **Don't file duplicates.** Before creating, check `bd list --status=open` for existing issues with similar titles. If a match exists, note it to the user instead of filing.
- **Severity defaults to P2** if the user doesn't specify and context is ambiguous.
