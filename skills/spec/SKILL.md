---
name: spec
description: "Create a structured specification document through progressive elaboration before implementation. Use when you need a complete design before coding, for features with multiple valid approaches, or when architectural alignment matters. Keywords: spec, frd, sdd, requirements, design, specification, document."
argument-hint: "<goal or feature to specify>"
disable-model-invocation: false
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

If `$ARGUMENTS` is empty or too vague, ask one clarifying question. The goal should fit in 2-3 sentences -- if longer, extract just the problem.

### 1b. Create the Spec File

Create `.specs/<name>.md` where `<name>` is a short kebab-case identifier from the goal. Create `.specs/` if needed.

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

Tell the user the absolute path and confirm the extracted goal.

---

## Phase 2: Populate Sections

Dispatch 3 background Task agents to populate the spec. Each agent gets a fresh context and specific sections to fill. All agents MUST read the actual codebase to ground their answers.

### Agent 1: Context & Prior Art (Explore)

**Instructions:**

> You are populating the Context & Constraints and Prior Art sections of a specification document.
>
> **Spec location:** `.specs/<name>.md`
>
> Your perspective: You think like an archaeologist of the codebase and a cartographer of constraints. You trace how similar problems were solved before, what patterns already exist, and what architectural, performance, security, and compatibility boundaries constrain the design. You surface existing code that the spec must acknowledge and the business rules it must respect.
>
> Follow the investigation protocol and report requirements from the Agent Preamble (fan-out-protocol rule).
>
> **Populate these sections:**
> - **Context & Constraints**: Current state and applicable constraints
> - **Prior Art**: Existing patterns, modules, or abstractions that solve similar problems
>
> When done, report your findings in structured Markdown. The orchestrator will write them into the spec.

### Agent 2: Proposed Approach & API Contract (Explore)

**Instructions:**

> You are populating the Proposed Approach, API/Interface Contract, and Data Model Changes sections of a specification document.
>
> **Spec location:** `.specs/<name>.md`
>
> Your perspective: You think like a system designer and contract author. You translate the problem into a high-level solution grounded in the project's existing conventions — what components change, what's created, how they interact. You define precise public surfaces (functions, endpoints, events, CLI commands) with clear inputs and outputs, following the project's API style. You identify new or modified data entities, or state explicitly when none are needed.
>
> Follow the investigation protocol and report requirements from the Agent Preamble (fan-out-protocol rule).
>
> **Populate these sections:**
> - **Proposed Approach**: High-level solution (conceptual, not implementation details)
> - **API/Interface Contract**: Public surfaces with signatures (pseudo-code is fine)
> - **Data Model Changes**: New or modified entities, fields, relationships (or "No data model changes")
>
> When done, report your findings in structured Markdown. The orchestrator will write them into the spec.

### Agent 3: Migration & Acceptance Criteria (Explore)

**Instructions:**

> You are populating the Migration/Rollout Plan and Acceptance Criteria sections of a specification document.
>
> **Spec location:** `.specs/<name>.md`
>
> Your perspective: You think like a deployment engineer and a test author. You identify how the change gets to production safely — backward compatibility concerns, feature flags, data migrations, schema changes — or state explicitly when standard deployment is sufficient. You define verifiable completion criteria (tests pass, docs updated, no regressions) grounded in the project's existing testing and deployment conventions, never vague success measures.
>
> Follow the investigation protocol and report requirements from the Agent Preamble (fan-out-protocol rule).
>
> **Populate these sections:**
> - **Migration/Rollout Plan**: Deployment strategy (or "No migration needed -- standard deployment")
> - **Acceptance Criteria**: Bulleted list of testable outcomes (each verifiable, never vague)
>
> When done, report your findings in structured Markdown. The orchestrator will write them into the spec.

### Dispatch and Wait

Launch all 3 agents with `run_in_background=true`. Wait for completion, then write each agent's findings into the spec using Edit.

---

## Phase 3: Refine and Validate

Dispatch 2 agents to validate the spec against the codebase -- one checks truth, one checks consistency.

### Agent 4: Truth Validation (Explore)

**Instructions:**

> You are validating a specification document against the actual codebase.
>
> **Spec location:** `.specs/<name>.md`
>
> Your perspective: You think like a fact-checker and implementation simulator. You verify every claim about existing code by reading the actual implementation -- catching untruths (spec says X exists but it doesn't), missing steps (spec skips necessary wiring/config/migration), and flaws (proposed approach won't work given how the code actually operates). You provide evidence and fixes for each issue.
>
> **Report each issue with:** Type (Untruth | Missing step | Flaw), Location (spec section), Issue, Evidence (file path), Fix. If accurate, report "No validation issues found."

### Agent 5: Conflict Detection (Explore)

**Instructions:**

> You are checking a specification document for conflicts with existing patterns.
>
> **Spec location:** `.specs/<name>.md`
>
> Your perspective: You think like a consistency guardian and pattern matcher. You compare the proposed design against the existing codebase, catching pattern conflicts (contradicts conventions), naming conflicts (clashes with existing entities), architectural conflicts (violates layer boundaries), and style conflicts (doesn't match the project's interface idioms). You provide evidence and alignment recommendations for each conflict.
>
> **Report each conflict with:** Type (Pattern | Naming | Architecture | Style), Conflict, Evidence (file paths), Recommendation. If none, report "No conflicts found."

### Dispatch and Wait

Launch both agents with `run_in_background=true`. Apply corrections from their reports using Edit. If no issues, proceed to Phase 4.

---

## Phase 4: Architecture Guardian Review

Dispatch a single agent as architecture guardian to review the complete spec and produce a PASS/FAIL verdict.

### Agent 6: Architecture Guardian (Explore)

**Instructions:**

> You are the architecture guardian for a specification document.
>
> **Spec location:** `.specs/<name>.md`
>
> Your perspective: You think like a senior architect protecting the codebase's long-term health. You evaluate the complete spec against five criteria: reuse over reinvention (uses existing code/patterns), pattern alignment (follows conventions), healthy growth (promotes maintainability, avoids tech debt), proper boundaries (respects layers, no shadow architectures), and completeness (all sections populated, criteria testable, non-requirements clear). You issue PASS (ready for implementation) or FAIL (requires fixes) with specific evidence.
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

Launch with `run_in_background=true`. If PASS, proceed to Phase 5. If FAIL, apply fixes and re-run the guardian. Iterate until PASS.

---

## Phase 5: Present the Spec

### 5a. Finalize

Read the final spec, change status from `DRAFT` to `READY FOR REVIEW`.

### 5b. Present to User

Show: spec location (absolute path), quality summary (sections populated, issues fixed, guardian iterations), and next steps (review, revise, or `/blossom` to generate implementation backlog).

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
