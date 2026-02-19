---
name: premortem
description: "Identify how a feature could fail before building it, then design mitigations into the implementation plan. Use for high-stakes features, security-sensitive changes, anything touching money/auth/data integrity, or when you want to surface hidden risks. Keywords: premortem, risk, failure, security, resilience, mitigation."
argument-hint: "<feature or change to stress-test>"
disable-model-invocation: false
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

Use Grep, Glob, and Read to understand what will change, what downstream systems it touches, who consumes it, and what happens today.

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

Dispatch 3 Explore agents with `run_in_background=true`, each with a different failure lens.

### Agent 1: Security Lens

> You are running a pre-mortem security analysis.
>
> **Feature**: [feature summary from Phase 1]
>
> **Your job**: Generate 3-5 concrete failure scenarios where this feature is exploited or causes a security breach.
>
> **Your perspective**: You think in terms of attack surfaces, trust boundaries, and data flows. You assume breach and look for what an attacker would exploit first — injection points, authentication bypasses, privilege escalation paths. You consider both technical vulnerabilities (OWASP Top 10) and process gaps (insecure defaults, missing validation, unsafe deserialization). You ask: "Where does untrusted data enter the system, and what can I do with it?"
>
> Follow the investigation protocol and report requirements from the Agent Preamble (fan-out-protocol rule).
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

> You are running a pre-mortem reliability analysis.
>
> **Feature**: [feature summary from Phase 1]
>
> **Your job**: Generate 3-5 concrete failure scenarios where this feature breaks in production.
>
> **Your perspective**: You think in terms of load, concurrency, and failure propagation. You assume things will run at 10x expected scale with dependencies flaking at the worst possible time. You look for race conditions, resource exhaustion, timeout chains, and state drift — anything that works in testing but fails under production chaos. You ask: "What happens when this runs with 1000 concurrent requests while the database is slow and the cache is stale?"
>
> Follow the investigation protocol and report requirements from the Agent Preamble (fan-out-protocol rule).
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

> You are running a pre-mortem user impact and business risk analysis.
>
> **Feature**: [feature summary from Phase 1]
>
> **Your job**: Generate 3-5 concrete failure scenarios where this feature harms users or the business.
>
> **Your perspective**: You think from the user's chair and the CFO's spreadsheet. You look for data loss, wrong results, confusing UX, and irreversible actions that break user trust. You consider compliance landmines (GDPR, HIPAA, PCI) and cost explosions (runaway cloud spend, cascading API charges). You ask: "What failure would show up in a user complaint, a compliance audit, or next quarter's financials — and can we roll this back if it goes wrong?"
>
> Follow the investigation protocol and report requirements from the Agent Preamble (fan-out-protocol rule).
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

Wait for all three concurrent agents to complete before proceeding.

---

## Phase 3: Prioritize Failures

### 3a. Rank and Present

Combine all scenarios into a single table sorted by (severity x likelihood):

```markdown
## Failure Scenario Rankings

| # | Title | Lens | Severity | Likelihood | Impact Summary |
|---|-------|------|----------|------------|----------------|
| 1 | SQL injection via search | Security | catastrophic | high | Attacker gains DB access |
| 2 | Race condition on balance | Reliability | catastrophic | medium | Data corruption in accounts |
| 3 | GDPR violation in logs | User/Business | major | high | Compliance penalty + user harm |
| ... | ... | ... | ... | ... | ... |
```

Ranking priority: catastrophic+high > catastrophic+medium = major+high > major+medium = catastrophic+low > everything else.

### 3b. Ask User to Prioritize

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

### 4b. Grounding Check + Sharpening Gate

Before finalizing, verify each mitigation: prevention actually blocks the failure, detection is realistic given the current observability stack, and the test scenario is concrete enough to implement. Refine until all three hold.

**Sharpening gate** (from /retro): Each Prevention/Detection/Verification item must pass concreteness tests:

1. **Prevention**: Name the specific file/function to modify, state the exact validation/check/guard (not "add proper validation"), make it implementable without design decisions.
   - ✗ "Add proper validation to the input handler"
   - ✓ "In src/api/search.ts:handleQuery(), add regex whitelist ^[a-zA-Z0-9 ]+$ before SQL interpolation at line 47"

2. **Detection**: Name the specific metric or log line (not "monitor for anomalies"), state the concrete threshold/alert (not "watch for issues"), confirm the metric exists in the current observability stack.
   - ✗ "Monitor for suspicious query patterns"
   - ✓ "Log event.type='sql_injection_blocked' with query hash, alert if count > 10/min in Datadog monitor 'search-sql-injection'"

3. **Verification**: Name the specific test file/type (not "test with various inputs"), state the concrete scenario and assertion (not "ensure it works"), make the pass/fail criteria explicit.
   - ✗ "Test the input validation with various edge cases"
   - ✓ "In tests/api/search.test.ts: POST /search with body={q: \"'; DROP TABLE--\"}, assert HTTP 400 + error.code='INVALID_QUERY_CHARS'"

If a mitigation item fails the gate, rewrite it until it passes. Each item must be implementable in one session without needing clarification.

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

Create an epic to group mitigations, then create each mitigation as a child:

```bash
bd create --title="EPIC: Pre-mortem mitigations for [feature]" --type=epic --priority=1
```

For each mitigation in the plan:

```bash
bd create --title="[MITIGATION]: [scenario title]" --type=task \
  --priority=<0-for-catastrophic,1-for-major,2-for-minor> \
  --parent=<mitigation-epic-id> \
  --description="Pre-mortem mitigation for [feature]. Failure scenario: [brief]. Prevention: [what to build]. Detection: [monitoring]. Test: [verification]. Effort: [S/M/L]."
```

Wire the mitigation epic as a dependency of the main feature task so mitigations complete before implementation:

```bash
bd dep add <feature-task> <mitigation-epic-id>
```

Create the feature task if it doesn't exist yet.

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

## See also

- `/spec` — formalize the design before or after premortem; premortem findings become constraints in the spec
- `/tracer` — trace mitigation tasks through implementation; feed the beads epic created by premortem into tracer
- `/test-strategy` — failure modes identified by premortem become concrete test cases; run after mitigations are designed
