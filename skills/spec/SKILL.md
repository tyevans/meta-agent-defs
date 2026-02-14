---
name: spec
description: "Create a structured specification document through progressive elaboration before implementation. Use when you need a complete design before coding, for features with multiple valid approaches, or when architectural alignment matters. Keywords: spec, frd, sdd, requirements, design, specification, document."
argument-hint: "<goal or feature to specify>"
disable-model-invocation: true
user-invocable: true
allowed-tools: Read, Grep, Glob, Write, Edit, Bash(bd:*), Bash(git:*), Task
context: fork
---

# Spec: Structured Specification Workflow

You are running the **Spec** workflow -- a Spec-Driven Development (SDD) pattern that converts a goal into a complete specification document through progressive elaboration. The user wants to specify: **$ARGUMENTS**

## When to Use

- Before starting implementation of a new feature or system
- When multiple valid implementation approaches exist and a design decision is needed
- When architectural alignment matters (changes touch core abstractions)
- When the change requires cross-cutting coordination (multiple layers, modules, or teams)
- When the user wants a design review before committing to an approach

## Overview

Spec works in 5 phases. Fresh agents populate and refine the document to prevent context contamination.

```
Seed spec doc with skeleton headings
  -> dispatch agents to populate sections (ground in actual codebase)
    -> dispatch agents to refine/validate populated spec
      -> architecture guardian reviews for alignment
        -> present complete spec with quality summary
```

---

## Phase 1: Seed the Spec Document

### 1a. Clarify the Goal

If `$ARGUMENTS` is empty or too vague, ask the user one clarifying question about what they want to build or change. Otherwise, proceed immediately.

The goal should fit in 2-3 sentences. If it's longer, you're mixing problem and solution -- extract just the problem.

### 1b. Create the Spec File

Create a new spec document at `.specs/<name>.md` where `<name>` is a short kebab-case identifier derived from the goal (e.g., "trust-delegation-api" for a trust delegation feature).

If the `.specs/` directory does not exist, create it:

```bash
mkdir -p .specs
```

### 1c. Write the Skeleton

The spec skeleton includes these sections. Write all headings now, populate content in Phase 2.

```markdown
# Spec: [Feature Name]

**Status:** DRAFT
**Created:** [YYYY-MM-DD]
**Author:** Claude Opus 4.6 (user-initiated)

---

## Problem Statement

[Human-provided goal from $ARGUMENTS]

---

## Context & Constraints

*To be populated: What is the current state? What architectural, performance, or business constraints apply?*

---

## Prior Art

*To be populated: What already exists in this codebase that is related? What patterns, modules, or abstractions already solve similar problems?*

---

## Proposed Approach

*To be populated: High-level design. What components change? What is created? How do they interact?*

---

## API / Interface Contract

*To be populated: Public surfaces (function signatures, endpoints, events, CLI commands). What does the outside world see?*

---

## Data Model Changes

*To be populated: New or modified entities, fields, relationships, migrations. If no data model changes, state "No data model changes."*

---

## Migration / Rollout Plan

*To be populated: How does this change get deployed? Are there backward compatibility concerns? Feature flags? Data migrations? If trivial, state "No migration needed."*

---

## Non-Requirements

*To be populated: What is explicitly OUT of scope for this spec? What won't be built?*

---

## Acceptance Criteria

*To be populated: How do we know this is done? Bulleted list of testable outcomes.*

---

## Open Questions

*To be populated: Unresolved decisions, areas needing input, risks that need mitigation.*

---
```

Write this skeleton to `.specs/<name>.md` now.

### 1d. Report the Spec Path

Tell the user where the spec document lives (absolute path) and confirm the goal you extracted from `$ARGUMENTS`.

---

## Phase 2: Populate Sections

Dispatch 3 background Task agents to populate the spec. Each agent gets a fresh context and specific sections to fill. All agents MUST read the actual codebase to ground their answers.

### Agent 1: Context & Prior Art

**Agent type:** Explore
**Sections to populate:** Context & Constraints, Prior Art

**Instructions for Agent 1:**

> You are populating the Context & Constraints and Prior Art sections of a specification document.
>
> **Spec location:** `.specs/<name>.md`
>
> **Your job:**
>
> 1. **Context & Constraints**: Read the project structure (README, architecture docs, key config files) to understand the current state. What constraints apply? (performance, architectural boundaries, compatibility, security, business rules)
>
> 2. **Prior Art**: Search the codebase for existing patterns, modules, or abstractions that solve similar problems. Use Glob and Grep to find relevant code. READ the actual implementations -- do not speculate. Report file paths and what you found.
>
> **Critical:** Ground every statement in actual code. Cite file paths. If you cannot find something, say "No existing implementation found" rather than guessing.
>
> When done, report your findings in structured Markdown. The orchestrator will write them into the spec.

### Agent 2: Proposed Approach & API Contract

**Agent type:** Explore
**Sections to populate:** Proposed Approach, API/Interface Contract, Data Model Changes

**Instructions for Agent 2:**

> You are populating the Proposed Approach, API/Interface Contract, and Data Model Changes sections of a specification document.
>
> **Spec location:** `.specs/<name>.md`
>
> **Your job:**
>
> 1. **Proposed Approach**: Design a high-level solution. What components change? What is created? How do they interact? Keep it conceptual (not implementation details).
>
> 2. **API/Interface Contract**: Define the public surfaces. What functions, endpoints, events, or CLI commands are exposed? Show signatures (pseudo-code is fine). Be precise about inputs and outputs.
>
> 3. **Data Model Changes**: Are new entities, fields, or relationships needed? Are existing ones modified? If no data model changes, state that explicitly.
>
> **Critical:** Read the codebase to understand existing API patterns and data modeling conventions. Follow the project's style.
>
> When done, report your findings in structured Markdown. The orchestrator will write them into the spec.

### Agent 3: Migration & Acceptance Criteria

**Agent type:** Explore
**Sections to populate:** Migration/Rollout Plan, Acceptance Criteria

**Instructions for Agent 3:**

> You are populating the Migration/Rollout Plan and Acceptance Criteria sections of a specification document.
>
> **Spec location:** `.specs/<name>.md`
>
> **Your job:**
>
> 1. **Migration/Rollout Plan**: How does this change get deployed? Are there backward compatibility concerns? Feature flags? Data migrations? Database schema changes? If the change is simple and requires no special rollout, state "No migration needed -- standard deployment."
>
> 2. **Acceptance Criteria**: How do we know this is done? Write a bulleted list of testable outcomes. Each criterion should be verifiable (not vague like "works well"). Examples:
>    - Unit tests pass for new API endpoints
>    - Integration test covers happy path and error cases
>    - Documentation updated in README
>    - No performance regression on benchmark suite
>
> **Critical:** Read the codebase to understand existing deployment and testing patterns. Match the project's conventions.
>
> When done, report your findings in structured Markdown. The orchestrator will write them into the spec.

### Dispatch and Wait

Launch all 3 agents with `run_in_background=true` via the Task tool. Wait for all 3 to complete before proceeding to Phase 3.

### Write Findings into Spec

As each agent completes, read its report and write the findings into the appropriate sections of `.specs/<name>.md` using the Edit tool.

---

## Phase 3: Refine and Validate

Dispatch 2 background Task agents to validate the populated spec against the codebase. They look for errors, omissions, and conflicts.

### Agent 4: Truth Validation

**Agent type:** Explore
**Task:** Validate spec claims against actual code

**Instructions for Agent 4:**

> You are validating a specification document against the actual codebase.
>
> **Spec location:** `.specs/<name>.md`
>
> **Your job:**
>
> Read the spec. For each claim about existing code (in Context, Prior Art, or Proposed Approach), verify it by reading the actual code. Look for:
>
> - **Untruths**: Spec says something exists that doesn't, or misrepresents how something works
> - **Missing steps**: Spec skips a necessary step (e.g., wiring in DI, updating config, migration)
> - **Flaws**: Proposed approach won't work given how the code actually works
>
> For each issue found, report:
> - **Type**: Untruth | Missing step | Flaw
> - **Location**: Which section of the spec
> - **Issue**: What is wrong
> - **Evidence**: File path + what you found (or didn't find)
> - **Fix**: What should the spec say instead
>
> If the spec is accurate, report "No validation issues found."

### Agent 5: Conflict Detection

**Agent type:** Explore
**Task:** Check spec against existing patterns and conventions

**Instructions for Agent 5:**

> You are checking a specification document for conflicts with existing patterns.
>
> **Spec location:** `.specs/<name>.md`
>
> **Your job:**
>
> Read the spec. Compare the Proposed Approach and API/Interface Contract sections against the existing codebase. Look for:
>
> - **Pattern conflicts**: Spec proposes a pattern that contradicts existing conventions
> - **Naming conflicts**: Spec uses names that clash with existing entities or modules
> - **Architectural conflicts**: Spec introduces dependencies that violate layer boundaries
> - **Style conflicts**: Spec proposes interfaces that don't match the project's style
>
> For each conflict found, report:
> - **Type**: Pattern | Naming | Architecture | Style
> - **Conflict**: What conflicts with what
> - **Evidence**: File paths showing existing convention
> - **Recommendation**: How to align the spec with existing patterns
>
> If no conflicts exist, report "No conflicts found."

### Dispatch and Wait

Launch both agents with `run_in_background=true`. Wait for both to complete.

### Apply Corrections

Read the reports from Agents 4 and 5. For each issue or conflict found, update the spec document using the Edit tool to fix the problem.

If no issues are found, proceed to Phase 4.

---

## Phase 4: Architecture Guardian Review

Dispatch a single background Task agent to act as architecture guardian. This agent reviews the complete spec and produces a PASS/FAIL verdict.

### Agent 6: Architecture Guardian

**Agent type:** Explore
**Task:** Final architecture alignment review

**Instructions for Agent 6:**

> You are the architecture guardian for a specification document.
>
> **Spec location:** `.specs/<name>.md`
>
> **Your job:**
>
> Read the complete spec (all sections). Evaluate it against these criteria:
>
> 1. **Reuse over reinvention**: Does the spec use existing code, patterns, and abstractions rather than building new ones?
> 2. **Pattern alignment**: Does the spec follow existing conventions in naming, structure, and idioms?
> 3. **Healthy growth**: Does the spec promote maintainability and extensibility, or does it create tech debt?
> 4. **Proper boundaries**: Does the spec respect layer boundaries and avoid creating shadow architectures?
> 5. **Completeness**: Are all sections populated? Are acceptance criteria testable? Are non-requirements clear?
>
> **Verdict:**
> - **PASS**: Spec is ready for implementation
> - **FAIL**: Spec has issues that must be fixed before implementation
>
> For each criterion, state PASS or FAIL and provide reasoning. If FAIL, list specific issues with file:line or section references.
>
> **Report format:**
>
> ```markdown
> ## Architecture Guardian Review
>
> **Verdict:** PASS | FAIL
>
> ### Criteria Evaluation
>
> 1. **Reuse over reinvention:** [PASS/FAIL] — [reasoning]
> 2. **Pattern alignment:** [PASS/FAIL] — [reasoning]
> 3. **Healthy growth:** [PASS/FAIL] — [reasoning]
> 4. **Proper boundaries:** [PASS/FAIL] — [reasoning]
> 5. **Completeness:** [PASS/FAIL] — [reasoning]
>
> ### Issues (if FAIL)
> - [Issue 1 with section reference]
> - [Issue 2 with section reference]
> ```

### Dispatch and Wait

Launch the guardian agent with `run_in_background=true`. Wait for its report.

### Handle Verdict

**If PASS:** Proceed to Phase 5.

**If FAIL:** Apply fixes to the spec based on the guardian's issues. After fixing, re-run the guardian (dispatch Agent 6 again with the same instructions). Iterate until PASS.

---

## Phase 5: Present the Spec

### 5a. Read the Final Spec

Read the complete `.specs/<name>.md` file to prepare the summary.

### 5b. Update Spec Status

Change the status line at the top of the spec from `DRAFT` to `READY FOR REVIEW`:

```markdown
**Status:** READY FOR REVIEW
```

### 5c. Present to User

Show the user:

1. **Spec location** (absolute path)
2. **Quality summary**:
   - Sections populated: N/N
   - Validation issues found and fixed: N
   - Guardian verdict: PASS (after N iterations)
3. **Next steps**:
   - Review the spec (share the file path)
   - If approved, create implementation tasks (suggest using `/blossom` to generate the backlog)
   - If changes needed, the user can edit `.specs/<name>.md` directly

### Example Output

```markdown
## Spec Complete: [Feature Name]

**Location:** `/path/to/project/.specs/<name>.md`

### Quality Summary
- Sections populated: 9/9 ✓
- Validation issues found and fixed: 3 (2 missing steps, 1 untruth)
- Conflicts detected and resolved: 1 (naming conflict with existing module)
- Guardian verdict: PASS (after 1 iteration)

### Next Steps
1. **Review the spec** — Read `.specs/<name>.md` and verify it matches your intent
2. **Revise if needed** — Edit the spec directly or ask me to refine specific sections
3. **Generate backlog** — When ready, run `/blossom` with the spec as context to create implementation tasks

The spec is ready for review. The design uses existing patterns from `lib/core/` and follows the project's API conventions. No data migrations required.
```

---

## Guidelines

1. **Fresh agents prevent contamination** — Each phase uses new agents with clean context
2. **Ground in reality** — Agents must READ actual code, not speculate about what exists
3. **The spec is the artifact** — It persists across context boundaries and survives context limits
4. **Non-Requirements prevent scope creep** — Explicitly stating what WON'T be built is as important as what will
5. **Guardian is the quality gate** — Iterate until PASS; don't bypass the guardian
6. **Spec before implementation** — The spec is a contract, not documentation of existing code
7. **Testable acceptance criteria** — Each criterion must be verifiable (not vague like "works well")
8. **Incremental population** — Populate, validate, refine, guard -- don't try to perfect in one pass
9. **Reuse over reinvention** — The guardian enforces use of existing patterns and abstractions
10. **Fresh context for each phase** — Background agents run independently to avoid bloating the orchestrator's context
