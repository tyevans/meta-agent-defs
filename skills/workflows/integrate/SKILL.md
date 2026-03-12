---
name: integrate
description: "Use after parallel bounded-context implementation to verify contracts still hold. Checks compliance, runs cross-boundary tests, detects conflicts, and proposes merge order. Keywords: integrate, contracts, merge, cross-boundary, validation, multi-human, event storming."
argument-hint: "<path to contracts directory>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git:*), Task
context: inline
---

# Integrate: Cross-Boundary Contract Validation and Merge Proposal

You are running **Integrate** — Phase 5 of a multi-human Event Storming workflow. After parallel bounded-context implementation, validate that each context honors its contracts, verify cross-boundary event flows, detect conflicts, and produce an actionable integration plan. Contracts directory: **$ARGUMENTS**

**Named law: NO MERGE WITHOUT VALIDATION** — if proposing integration without running cross-boundary checks, STOP.

## When to Use

- After parallel implementation of bounded contexts from an Event Storming session
- When each context team has delivered code and contract schema files
- Before merging bounded context branches to confirm integration assumptions hold
- When contracts have changed during implementation and cross-boundary alignment must be re-verified

## Don't Use When

- Implementations are not yet complete — at least one context must be missing its deliverables
- No contract schemas exist — this skill validates existing contracts; it does not author them (use /formalize first)
- You need to negotiate new contract terms — use /negotiate for contested contracts

## Overview

Integrate works in 5 phases. Phase 0 is a hard gate.

```
Precondition gate (all contexts complete + contracts present)
  -> Phase 1: Contract compliance (each context vs. its own schemas)
    -> Phase 2: Cross-boundary integration test (event flows end-to-end)
      -> Phase 3: Conflict detection (structural and semantic divergence)
        -> Phase 4: Merge proposal (ordering, resolution, verification steps)
```

**Consumes**: /formalize output — contract schemas in `$ARGUMENTS` (provided/consumed event schemas per bounded context).
**Produces**: pipe-format integration report with CONFIRMED/LIKELY/POSSIBLE findings, plus amendment proposals for contracts needing revision.

---

## Phase 0: Precondition Gate

**This phase is a hard gate. If any check fails, STOP and report what must be resolved before running /integrate.**

### 0a. Locate Contracts Directory

If `$ARGUMENTS` is empty, scan for a contracts directory:

```bash
ls -d contracts/ schemas/ event-contracts/ bounded-contexts/ 2>/dev/null
```

If no contracts directory is found, stop:

> "No contracts directory found. Run /formalize to generate event schemas for each bounded context, then re-run /integrate with the contracts path."

If found, set the contracts path and continue.

### 0b. Enumerate Bounded Contexts

Read the contracts directory to enumerate known bounded contexts. Each context should have at least a provided-events schema and a consumed-events schema:

```bash
ls -1 <contracts-path>/
```

Expected layout (produced by /formalize):

```
contracts/
  <context-name>/
    provided-events.md    # events this context emits
    consumed-events.md    # events this context expects to receive
    README.md             # optional — context summary
```

If no context subdirectories exist, stop:

> "Contracts directory exists but contains no per-context schema directories. Verify /formalize was run and produced context-level schema files."

### 0c. Check Implementation Presence

For each enumerated bounded context, verify that an implementation exists. Accept any of these signals:

- A directory matching the context name at the repo root or in `src/`
- A git branch named after the context (check with `git branch --list "*<context-name>*"`)
- A completed task in your project's task tracker referencing the context name

Flag any context that has a contract but no detectable implementation. Do not stop — proceed with partial validation and mark missing implementations as BLOCKED findings in Phase 1.

### 0d. Gate Summary

```
## Precondition Gate: [PASS | PARTIAL | FAIL]

Contracts path: <path>

Bounded contexts detected:
- <context-name>: [implementation found | BLOCKED — no implementation detected]
- <context-name>: ...

[If FAIL]: No contracts or no contexts detected. Cannot proceed.
[If PARTIAL]: N of M contexts have implementations. Proceeding with available contexts. Blocked contexts listed as BLOCKED in findings.
[If PASS]: All M contexts have implementations. Proceeding to Phase 1.
```

---

## Phase 1: Contract Compliance Check

For each bounded context with a detected implementation, validate that the implementation honors its provided and consumed event schemas.

### 1a. Dispatch Compliance Agents

If there are 3 or more contexts, dispatch one background Explore agent per context (up to 4 concurrently). For 1-2 contexts, validate inline.

#### Agent prompt template

```
You are a contract compliance agent for the bounded context: <context-name>

Your task: verify that the implementation of <context-name> honors the contracts defined in its schema files.

**Schema files:**
- Provided events: <contracts-path>/<context-name>/provided-events.md
- Consumed events: <contracts-path>/<context-name>/consumed-events.md

**Implementation location:** <implementation-path>

**Investigation protocol:**
1. Read both schema files to extract the full event inventory (event names, fields, field types).
2. Search the implementation for each provided event: look for event emission points (event publish, dispatch, emit, or equivalent patterns). Verify the emitted payload matches the schema fields.
3. Search the implementation for each consumed event: look for event subscription or handler registration. Verify the handler expects the schema fields.
4. Ground every finding in actual file paths and line numbers.

**Report format (use exactly):**

### Compliance Report: <context-name>

**Verdict:** PASS | FAIL | PARTIAL

#### Provided Events
For each event in provided-events.md:
- **<EventName>**: COMPLIANT | MISSING | SCHEMA_MISMATCH
  - evidence: <file:line or "not found">
  - detail: <what matches or what diverges>

#### Consumed Events
For each event in consumed-events.md:
- **<EventName>**: COMPLIANT | MISSING | SCHEMA_MISMATCH
  - evidence: <file:line or "not found">
  - detail: <what matches or what diverges>

#### Summary
<One paragraph: overall compliance verdict and the most critical violations.>
```

### 1b. Collect Compliance Results

Collect all agent reports. Tally by context:

| Context | Verdict | Violations |
|---------|---------|-----------|
| \<name\> | PASS/FAIL/PARTIAL | N |

---

## Phase 2: Cross-Boundary Integration Test

Verify that event flows work end-to-end across context boundaries. Each provided event from one context must land correctly in the consumed-events schema of the context that subscribes to it.

### 2a. Build the Event Flow Graph

Scan all `provided-events.md` and `consumed-events.md` files to build a cross-context event flow map:

```
<EventName>: <producer-context> -> [<consumer-context>, ...]
```

Flag any event that:
- Is provided by one context but consumed by zero others (orphan producer)
- Is consumed by one context but provided by zero others (orphan consumer)
- Has schema field mismatches between the producer's provided schema and the consumer's consumed schema

### 2b. Validate Each Cross-Boundary Flow

For each non-orphan event in the flow graph:

1. Read the producer's `provided-events.md` entry for the event
2. Read the consumer's `consumed-events.md` entry for the event
3. Compare field names and types — exact field name match required; type widening is LIKELY-flagged; type narrowing or extra required fields are structural conflicts

Report each flow:

```
<EventName>: <producer> -> <consumer>
  schema alignment: MATCH | MISMATCH
  [if mismatch]: fields diverging: <field> — producer: <type>, consumer: <type>
```

### 2c. Integration Test Summary

```
## Cross-Boundary Flows: [N flows verified]

- Clean flows: N
- Orphan producers: N (events emitted but no consumer declared)
- Orphan consumers: N (events expected but no producer declared)
- Schema mismatches: N
```

---

## Phase 3: Conflict Detection

Surface where implementations diverged from contracts — both structural (field/type mismatches) and semantic (naming or intent divergence).

### 3a. Structural Conflicts

From Phase 1 and Phase 2 findings, collect all SCHEMA_MISMATCH verdicts. For each:

- **Field addition**: Producer added fields not in the contract schema → LIKELY — consumer may ignore safely, but schema is stale
- **Field removal**: Producer dropped fields the consumer requires → CONFIRMED conflict
- **Type change**: Field type changed in one direction → CONFIRMED if narrowing; LIKELY if widening
- **Rename**: Field renamed — detected by finding a close-match field in one direction but not the other → POSSIBLE

### 3b. Semantic Conflicts

Scan provided-events and consumed-events names across contexts for naming collisions:

- Two contexts providing an event with the same name → ambiguous routing
- A context consuming an event name that exists in no provider → naming drift (POSSIBLE — may be a rename)

```bash
grep -r "eventName\|event_name\|EventName" <contracts-path> 2>/dev/null
```

Check git history for contract file changes that may have introduced drift:

```bash
git log --oneline --since="30 days ago" -- <contracts-path>
```

For each recent schema change, check if both producer and consumer schema files were updated in the same commit. If only one was updated, flag as LIKELY semantic conflict.

### 3c. Conflict Report

Classify each conflict by severity:

| Severity | Meaning |
|----------|---------|
| FATAL | Breaks integration — required field missing, event flow severed |
| SERIOUS | Degrades behavior — type mismatch, ambiguous routing |
| ADVISORY | Cosmetic or recoverable — field rename candidate, orphan event |

Total-order tiebreaker within severity: FATAL > SERIOUS > ADVISORY, then alphabetical by context name.

---

## Phase 4: Merge Proposal

Produce an integration plan with conflict resolution, merge ordering, and verification steps.

### 4a. Amendment Proposals

For each FATAL or SERIOUS conflict from Phase 3, produce an amendment proposal for the relevant contract schema:

```
## Amendment Proposal: <context-name>/<schema-file>

**Conflict**: <one-line description>
**Severity**: FATAL | SERIOUS
**Proposed change**:
  - Before: <current field/type/name>
  - After: <corrected field/type/name>
**Requires agreement from**: <producer context>, <consumer context>
**Feed into**: /negotiate (if contested) or apply directly (if unambiguous fix)
```

<!-- Amendment proposals section is conditional — omit if Phase 3 found no FATAL or SERIOUS conflicts -->

### 4b. Merge Ordering

Propose a merge order for bounded context branches that minimizes integration risk:

1. Contexts with no FATAL conflicts and no cross-boundary dependencies → merge first
2. Contexts with SERIOUS conflicts but clear amendment paths → merge after amendments are applied
3. Contexts with FATAL conflicts → BLOCKED until amendments are agreed

State the recommended merge sequence as a numbered list with rationale.

### 4c. Pre-Merge Verification Checklist

```
## Pre-Merge Verification

Before merging each context, verify:

- [ ] All FATAL conflicts resolved (amendments applied and schemas regenerated)
- [ ] All SERIOUS conflicts addressed or explicitly accepted
- [ ] Cross-boundary event flows re-verified after amendments
- [ ] Contract schema files updated to reflect final agreed-upon state
- [ ] Each context's implementation re-validated against amended schemas (re-run Phase 1)
- [ ] Tasks for unresolved ADVISORY findings created (see Phase 4d)
```

### 4d. Track Unresolved Findings

For ADVISORY findings that are deferred rather than blocked, create follow-up tasks using your preferred task tracking approach:

- Title: `integrate: advisory — <finding summary>`
- Description: `Post-integration follow-up: <detail>. Source: /integrate Phase 3 conflict report.`
- Priority: low

---

## Phase 5: Integration Report (Pipe Format)

Emit the integration report in pipe format. Detecting /formalize output in context? State it: "Reading N contract schemas from /formalize output above."

```markdown
## Integration report for <contexts list>

**Source**: /integrate
**Input**: <contracts path from $ARGUMENTS>
**Pipeline**: /formalize (N schemas) -> /integrate (N findings)

### Items (N)

<!-- One item per CONFIRMED/LIKELY/POSSIBLE finding. Order: FATAL first, then SERIOUS, then ADVISORY. -->

1. **[FATAL] <conflict title>** — <one-line description>
   - context: <context-name>
   - source: <contracts-path/context/schema-file>
   - confidence: CONFIRMED | LIKELY | POSSIBLE
   - amendment: see Phase 4a proposal for <context-name>

2. **[SERIOUS] <conflict title>** — <one-line description>
   - context: <context-name>
   - source: <contracts-path/context/schema-file>
   - confidence: CONFIRMED | LIKELY | POSSIBLE

3. **[ADVISORY] <finding title>** — <one-line description>
   - context: <context-name>
   - source: <contracts-path/context/schema-file>
   - confidence: POSSIBLE
   <!-- Advisory findings only present when Phase 3 detected them -->

### Merge Order

1. <context-name> — <rationale>
2. <context-name> — <rationale>
...

### Summary

<One paragraph: overall integration health, count of FATAL/SERIOUS/ADVISORY findings, whether integration can proceed, and what must be resolved first.>
```

---

## Guidelines

1. **Gate is mandatory.** Never skip Phase 0. Missing implementations or missing contracts must be surfaced before validation begins — guessing produces false confidence.
2. **NO MERGE WITHOUT VALIDATION.** The named law applies to this skill itself: never emit a merge proposal without completing Phase 1 (compliance) and Phase 2 (cross-boundary flows) first.
3. **Ground every finding in file evidence.** Every CONFIRMED finding must cite a file path. LIKELY findings must cite circumstantial evidence. POSSIBLE findings must state what was searched for and why the pattern is suspicious.
4. **Severity ordering is total.** Within each severity tier, order by context name alphabetically. This makes the report reproducible across runs.
5. **Amendment proposals feed /negotiate.** When a FATAL or SERIOUS conflict requires agreement from multiple context teams, create an amendment proposal and note that it feeds into /negotiate. Do not silently resolve contested changes.
6. **Orphan events are not errors by default.** An orphan producer (event emitted but no consumer declared) is ADVISORY — it may be an integration point with a system outside the scope of this session. An orphan consumer is SERIOUS — the implementation expects an event that no producer will deliver.
7. **Partial validation is better than no validation.** If only some contexts have implementations, run Phase 1 for available contexts and mark the rest BLOCKED. Report partial results.
8. **Schema mismatches vs. implementation gaps.** Distinguish between a context that emits an event with wrong fields (schema mismatch — the contract needs amendment) and a context that never emits the event at all (missing implementation — the code needs to be written).

## See Also

- `/formalize` — produces the contract schemas consumed by Phase 0 of this skill; run /formalize before /integrate
- `/negotiate` — for contested amendments where context teams disagree on field names or event semantics
- `/spec` — for designing integration architecture before contracts are written
- `/review` — code review companion; run /review on each bounded context before running /integrate
- `/deploy` — after integration is validated and merged, use /deploy for production rollout
