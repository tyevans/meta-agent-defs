---
name: premortem
description: "Identify how a feature could fail before building it, then design mitigations into the implementation plan. Use for high-stakes features, security-sensitive changes, anything touching money/auth/data integrity, or when you want to surface hidden risks. Keywords: premortem, risk, failure, security, resilience, mitigation."
argument-hint: "<feature or change to stress-test>"
disable-model-invocation: true
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Task, AskUserQuestion
context: fork
---

# Premortem: Risk-First Design Workflow

You are running the **Premortem** workflow -- a pre-mortem analysis that surfaces failure modes before implementation begins. Feature to analyze: **$ARGUMENTS**

## When to Use

- Before implementing high-stakes features (payment processing, authentication, data sync)
- When a feature touches security-critical surfaces (auth boundaries, data access, external APIs)
- For changes involving money, PII, or data integrity
- When you want to shift from "how do we build this?" to "how could this fail?"
- Before committing to an architecture or approach that would be expensive to change

## Overview

Premortem works in 5 phases:

```
Understand the feature (what changes, what it touches)
  -> Generate failure scenarios (3 agents, different lenses)
    -> Prioritize with user (which failures matter most)
      -> Design mitigations (concrete code/test/monitoring changes)
        -> Report and create beads tasks
```

---

## Phase 1: Understand the Feature

### 1a. Clarify the Scope

If `$ARGUMENTS` is vague or incomplete, ask the user one clarifying question:
- What part of the system does this change?
- What existing behavior changes?
- Who uses this feature (internal, external, automated systems)?

Otherwise proceed immediately.

### 1b. Explore the Codebase

Use Grep, Glob, and Read to understand:
- **What will change**: Files and modules affected by this feature
- **What it touches**: Downstream systems, external APIs, data stores, queues
- **Who uses it**: Callers, consumers, UI surfaces that rely on this code
- **Current behavior**: What happens today before this feature exists

### 1c. Produce Feature Summary

Write a brief summary (3-5 sentences):

```markdown
## Feature Summary

**What it does**: [1-2 sentence description of the feature]

**What changes**: [Components/files/modules that will be modified or added]

**What it touches**: [Downstream systems, databases, APIs, queues, external services]

**Who uses it**: [End users, internal systems, other services]

**Current state**: [What happens today, what gap this feature fills]
```

---

## Phase 2: Generate Failure Scenarios

Dispatch 3 background agents using the Task tool, each with a different failure lens. All agents run concurrently in the background.

### Agent 1: Security Lens

```
Task({
  subagent_type: "Explore",
  run_in_background: true,
  prompt: "<security lens instructions>"
})
```

**Security lens instructions:**

> You are running a pre-mortem security analysis.
>
> **Feature**: [feature summary from Phase 1]
>
> **Your job**: Generate 3-5 concrete failure scenarios where this feature is exploited or causes a security breach. Focus on OWASP Top 10 risks:
>
> - **Injection**: Could an attacker inject SQL, commands, or code through this feature?
> - **Auth bypass**: Could this be used to access resources without proper authorization?
> - **Data leakage**: Could sensitive data be exposed through this feature?
> - **Privilege escalation**: Could this be exploited to gain higher privileges?
> - **SSRF**: Could this be manipulated to make requests to internal systems?
> - **XSS/XXE**: Could this inject malicious content into outputs?
> - **Deserialization**: Could untrusted data be deserialized unsafely?
> - **Misconfiguration**: Are defaults insecure?
>
> **Investigation protocol:**
>
> 1. Read the actual codebase areas identified in the feature summary
> 2. Trace data flows from user input to storage/output
> 3. Identify trust boundaries this feature crosses
> 4. Check existing auth/validation patterns -- are they applied here?
> 5. Look for similar code that has been exploited before (check git history for security fixes)
>
> **Report format:**
>
> ```
> ## Security Failure Scenarios
>
> ### [Scenario 1 Title]
> **Narrative**: [What goes wrong, step by step. Be concrete -- name files, functions, parameters]
> **Attack vector**: [How would an attacker trigger this?]
> **Impact**: [What gets compromised -- data, access, availability?]
> **Severity**: [catastrophic | major | minor]
> **Likelihood**: [high | medium | low]
> **Affected components**: [file:line references]
>
> [Repeat for each scenario]
> ```

### Agent 2: Reliability Lens

```
Task({
  subagent_type: "Explore",
  run_in_background: true,
  prompt: "<reliability lens instructions>"
})
```

**Reliability lens instructions:**

> You are running a pre-mortem reliability analysis.
>
> **Feature**: [feature summary from Phase 1]
>
> **Your job**: Generate 3-5 concrete failure scenarios where this feature breaks in production. Focus on operational failures:
>
> - **Race conditions**: Could concurrent access cause corruption or inconsistency?
> - **Data corruption**: Could partial writes, crashes, or retries leave data in a bad state?
> - **Cascade failures**: Could this failure propagate to other systems?
> - **Resource exhaustion**: Could this consume unbounded memory, disk, connections, or CPU?
> - **Timeout chains**: Could slow dependencies cause this to timeout, triggering retries that make it worse?
> - **State drift**: Could cached state become stale or inconsistent with source of truth?
> - **Idempotency**: Is retry-safety guaranteed where needed?
> - **Dependency failures**: What happens if downstream systems are unavailable?
>
> **Investigation protocol:**
>
> 1. Read the actual codebase areas identified in the feature summary
> 2. Trace the execution path under load (many concurrent requests)
> 3. Identify shared mutable state and check locking/synchronization
> 4. Check error handling -- are all failure paths covered?
> 5. Look for similar code that has caused outages (check git history for reliability fixes)
>
> **Report format:**
>
> ```
> ## Reliability Failure Scenarios
>
> ### [Scenario 1 Title]
> **Narrative**: [What goes wrong, step by step. Be concrete -- name files, functions, state]
> **Trigger**: [What conditions cause this failure?]
> **Impact**: [Downtime, data loss, degraded performance?]
> **Severity**: [catastrophic | major | minor]
> **Likelihood**: [high | medium | low]
> **Affected components**: [file:line references]
>
> [Repeat for each scenario]
> ```

### Agent 3: User/Business Lens

```
Task({
  subagent_type: "Explore",
  run_in_background: true,
  prompt: "<user/business lens instructions>"
})
```

**User/business lens instructions:**

> You are running a pre-mortem user impact and business risk analysis.
>
> **Feature**: [feature summary from Phase 1]
>
> **Your job**: Generate 3-5 concrete failure scenarios where this feature harms users or the business. Focus on user-facing and business risks:
>
> - **Data loss**: Could users lose work, content, or important data?
> - **Wrong results**: Could this produce incorrect calculations, reports, or decisions?
> - **Confusing UX**: Could the behavior be unexpected or misleading?
> - **Compliance violations**: Does this touch regulated data (GDPR, HIPAA, PCI, SOC2)?
> - **Cost explosions**: Could this trigger runaway spend on cloud resources or external APIs?
> - **Irreversible actions**: Are destructive operations safely guarded?
> - **Migration/rollback**: Can this be safely rolled out and rolled back?
> - **Accessibility**: Could this exclude users with disabilities?
>
> **Investigation protocol:**
>
> 1. Read the actual codebase areas identified in the feature summary
> 2. Trace user-facing flows and outputs
> 3. Identify irreversible or high-consequence actions
> 4. Check if this feature handles regulated data types
> 5. Look for similar code that has caused user complaints (check git history, issue trackers)
>
> **Report format:**
>
> ```
> ## User/Business Failure Scenarios
>
> ### [Scenario 1 Title]
> **Narrative**: [What goes wrong from the user or business perspective. Be concrete]
> **Trigger**: [What user action or business condition causes this?]
> **Impact**: [User harm, business cost, compliance risk?]
> **Severity**: [catastrophic | major | minor]
> **Likelihood**: [high | medium | low]
> **Affected components**: [file:line references]
>
> [Repeat for each scenario]
> ```

### 2b. Wait for Agent Reports

All three agents run concurrently. Wait for each to complete and send back its report. Do not proceed to Phase 3 until all three reports are received.

---

## Phase 3: Prioritize Failures

### 3a. Aggregate All Scenarios

Combine all scenarios from the three agent reports into a single ranked table.

### 3b. Rank and Present

Create a table sorted by (severity × likelihood):

```markdown
## Failure Scenario Rankings

| # | Title | Lens | Severity | Likelihood | Impact Summary |
|---|-------|------|----------|------------|----------------|
| 1 | SQL injection via search | Security | catastrophic | high | Attacker gains DB access |
| 2 | Race condition on balance | Reliability | catastrophic | medium | Data corruption in accounts |
| 3 | GDPR violation in logs | User/Business | major | high | Compliance penalty + user harm |
| ... | ... | ... | ... | ... | ... |
```

Ranking priority:
1. **catastrophic + high**: Top priority
2. **catastrophic + medium**: High priority
3. **major + high**: High priority
4. **major + medium** or **catastrophic + low**: Medium priority
5. **major + low** or **minor + high**: Lower priority
6. **minor + medium/low**: Lowest priority

### 3c. Ask User to Prioritize

Use AskUserQuestion to let the user select which scenarios to mitigate:

```
I found [N] failure scenarios across security, reliability, and user/business dimensions.

The table above shows all scenarios ranked by severity × likelihood.

**Default recommendation**: Mitigate all catastrophic + high-likelihood scenarios (items #1-#X).

Which scenarios would you like to design mitigations for?
- Option 1: Top [X] scenarios (all catastrophic + high-likelihood) [RECOMMENDED]
- Option 2: Top [Y] scenarios (include major + high-likelihood)
- Option 3: All scenarios
- Option 4: Let me select specific scenario numbers (e.g., 1,2,5,7)
```

Record the user's selection and proceed with only the selected scenarios.

---

## Phase 4: Design Mitigations

For each prioritized scenario (from the user's selection in Phase 3), design a specific, actionable mitigation plan.

### 4a. Mitigation Template

For each scenario, produce:

```markdown
### Mitigation: [Scenario Title]

**Original failure**: [Brief recap of the scenario]

**Prevention**: [Code change that prevents this failure from happening]
- What file/function to modify
- What validation, check, or guard to add
- What library or pattern to use

**Detection**: [Monitoring or alerting that catches this if prevention fails]
- What metric to track
- What threshold to alert on
- What log message to emit

**Verification**: [Test that proves the mitigation works]
- What test type (unit, integration, e2e)
- What scenario to simulate
- What assertion to make

**Estimated effort**: [S | M | L] (Small: <1 day, Medium: 1-3 days, Large: >3 days)
```

### 4b. Grounding Check

Before finalizing each mitigation, verify:
1. Does the prevention actually block the failure path?
2. Is the detection realistic given the current observability stack?
3. Is the test scenario concrete enough to implement?

If any answer is no, refine the mitigation until all three are yes.

### 4c. Compile Mitigation Plan

Produce a complete mitigation plan document covering all selected scenarios.

---

## Phase 5: Report and Create Tasks

### 5a. Present Complete Report

```markdown
## Pre-Mortem Report: [Feature Name]

### Feature Summary
[From Phase 1]

### Failure Analysis
- **Total scenarios identified**: N
- **Security failures**: X
- **Reliability failures**: Y
- **User/Business failures**: Z
- **Prioritized for mitigation**: M

### Prioritized Failure Scenarios
[Table from Phase 3 with user's selected scenarios highlighted]

### Mitigation Plan
[Full mitigation plan from Phase 4]

### Implementation Recommendations

**Order of implementation**:
1. [Highest priority mitigation and why it comes first]
2. [Next mitigation]
3. ...

**Architecture implications**:
[Any cross-cutting concerns or architectural changes needed to support mitigations]

**Testing strategy**:
[How to verify all mitigations work together]

**Rollout plan**:
[Suggested phasing, feature flags, or gradual rollout strategy if applicable]

### Next Steps
[What to do with this report -- feed into /tracer, /spec, or direct implementation]
```

### 5b. Create Beads Tasks

For each mitigation in the plan:

```bash
bd create --title="[MITIGATION]: [scenario title]" --type=task --priority=<0-for-catastrophic,1-for-major,2-for-minor> \
  --description="Pre-mortem mitigation for [feature]. Failure scenario: [brief]. Prevention: [what to build]. Detection: [monitoring]. Test: [verification]. Effort: [S/M/L]."
```

Wire dependencies so mitigations complete before the main feature task:

```bash
bd dep add <feature-task-id> <mitigation-task-id>
```

If the main feature task doesn't exist yet, create it:

```bash
bd create --title="Implement: [feature name]" --type=feature --priority=2 \
  --description="[feature summary from Phase 1]. All pre-mortem mitigations must complete before implementation."
```

### 5c. Session Close Reminder

Before finishing, run the session close protocol:

```bash
bd sync
git status
```

If there are beads state changes to commit:

```bash
git add .beads/
git commit -m "chore: premortem analysis for [feature name]"
```

---

## Guidelines

- **Concrete over abstract**: "SQL injection via search param at line X" beats "security could be compromised"
- **Read the code**: Agents must ground scenarios in actual implementation, not hypothetical risks
- **User decides**: Not every risk needs mitigation -- present options, let the human choose
- **Proportional response**: Minor/unlikely scenarios don't justify major architectural changes
- **This produces a plan**: Premortem outputs a mitigation backlog, not an implementation -- feed it into /tracer or /spec
- **No false confidence**: If a scenario is POSSIBLE but not CONFIRMED, say so and recommend a deeper spike
- **Verify mitigations**: Every mitigation needs prevention (code), detection (monitoring), and verification (test)
- **Fail safely**: Design mitigations to fail closed (deny by default) not open (allow by default)
