---
name: storm-prep
description: "Run domain exploration to generate candidate domain events for multi-human Event Storming. Each human's agents analyze their assigned domain and produce structured YAML with events, aggregates, and assumptions. Use before a human jam session to prepare independent findings. Keywords: event storming, domain events, prep, multi-human, exploration, DDD, aggregate, bounded context."
argument-hint: "<domain scope and project goal>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Write, Bash(bd:*), Bash(tk:*), Bash(git:*), Task
context: inline
---

# Storm-Prep: Domain Event Discovery for Event Storming

You are running the **Storm-Prep** workflow — Phase 1 of a multi-human Event Storming trilogy. Each human's Claude agents independently explore their assigned domain and produce candidate domain events as structured YAML, ready for a human jam session and for downstream formalization. Domain scope: **$ARGUMENTS**

## When to Use

- Before a collaborative Event Storming session and each participant needs independent domain findings
- When a team is mapping a system's domain model and wants agent-generated candidates for each bounded context
- When a human needs to understand the domain events in their assigned area before joining a cross-team session
- As the first step before `/formalize` (Phase 2) and a cross-human reconciliation session (Phase 3)

## Don't Use When

- No project goal or domain scope is defined — Phase 0 will stop you here
- You want to run Event Storming in a single-human context — use `/spec` or `/decompose` instead
- You have already completed storm-prep for this domain scope — re-running overwrites `events/candidates/{role}.yaml`

## Overview

Storm-prep works in 4 phases. The scope gate (Phase 0) must pass before exploration begins.

```
Scope gate (project goal + domain scope defined)
  -> Domain exploration (agents fan out across codebase/goal)
    -> Assumption surfacing (what this human assumes at boundaries)
      -> YAML output generation (events, aggregates, assumptions, confidence)
        -> events/candidates/{role}.yaml — consumed by jam session + /formalize
```

---

## Phase 0: Scope Gate

**Law: NO EXPLORATION WITHOUT SCOPE. This phase is a hard gate. If either check fails, stop here.**

### 0a. Parse Arguments

If `$ARGUMENTS` is empty or does not include both a domain scope and a project goal, ask:

> "What is your domain scope (the area you own in this session) and the overall project goal? Example: 'scope: order fulfillment, goal: rebuild checkout system'"

Do not proceed until both are provided.

### 0b. Identify Your Role

Extract a role identifier from the scope. Use the domain name in kebab-case (e.g., `order-fulfillment`, `user-auth`, `inventory`). This becomes the output filename: `events/candidates/{role}.yaml`.

State the role identifier explicitly: "Role identified: **{role}**. Output will be written to `events/candidates/{role}.yaml`."

### 0c. Check for Prior Output

If `events/candidates/{role}.yaml` already exists, read it and display the timestamp from the `generated_at` field. Ask:

> "A prior storm-prep run exists for this role (generated: {timestamp}). Overwrite it? (yes / no)"

If the user says no, exit. If yes, proceed.

### 0d. Gate Summary

```
## Scope Gate: [PASS | FAIL]

- Domain scope: [{scope}]
- Project goal: [{goal}]
- Role identifier: [{role}]
- Output path: [events/candidates/{role}.yaml]
- Prior output: [none | found — {timestamp}, overwriting]

[If FAIL]: Define scope and goal before running storm-prep.
[If PASS]: Proceeding to domain exploration.
```

---

## Phase 1: Domain Exploration

Dispatch 3 Explore agents concurrently to investigate the domain from different angles. Each agent looks for candidate domain events — things that **happened** in the system, expressed as past-tense verbs (e.g., `OrderPlaced`, `PaymentFailed`, `InventoryReserved`).

### Agent 1: Command Flow Explorer

#### Agent 1 prompt template

> You are a domain event explorer for an Event Storming session.
>
> **Project goal**: {goal}
> **Domain scope**: {scope}
>
> **Your job**: Discover candidate domain events by tracing what commands (user actions, API calls, UI triggers) cause state changes in this domain.
>
> **Your perspective**: You think in terms of cause and effect — what actions users or systems take, and what observable outcomes result. You look for command handlers, API endpoints, service methods, or workflow steps that write state. For each, you surface the domain event it produces. Events are named as past-tense verbs: `UserRegistered`, `OrderCancelled`, `PaymentProcessed`.
>
> **Investigation protocol:**
> 1. Use Glob and Grep to find command handlers, API routes, service methods, or controllers in this domain
> 2. For each, read the implementation to understand what state change it produces
> 3. Name the domain event as a past-tense verb (PascalCase)
> 4. Identify the aggregate (the entity whose state changes) and any data payload
> 5. Verify by checking if tests or consumers reference this state change
> 6. Distinguish CONFIRMED (code verified), LIKELY (strong evidence), POSSIBLE (inferred)
>
> **Report format:**
>
> ```
> ## Command-Flow Domain Events
>
> ### {EventName}
> - aggregate: {EntityName}
> - trigger: {command or action that causes this}
> - payload: [{field}: {type}, ...]
> - source: {file:line}
> - confidence: CONFIRMED | LIKELY | POSSIBLE
>
> [Repeat for each event found]
> ```
>
> Report up to 15 events. Prioritize events with the clearest implementation evidence.

### Agent 2: State Change Tracker

#### Agent 2 prompt template

> You are a domain event explorer for an Event Storming session.
>
> **Project goal**: {goal}
> **Domain scope**: {scope}
>
> **Your job**: Discover candidate domain events by tracing state transitions — what database writes, entity mutations, or persistence operations happen in this domain.
>
> **Your perspective**: You think like a database archaeologist. You look for INSERT/UPDATE/DELETE operations, ORM save/create/update/delete calls, event store writes, or any persistence layer mutations. Each meaningful state change is a candidate domain event. You surface what transitions are happening under the hood, even when the code doesn't name them as events explicitly.
>
> **Investigation protocol:**
> 1. Use Grep to find database writes (ORM methods, raw SQL, event store appends) in this domain's files
> 2. For each write, trace back to the domain concept whose state changes
> 3. Name the implied domain event (PascalCase past-tense verb)
> 4. Identify the aggregate and the fields that change
> 5. Check if there are existing event classes, event types, or message schemas that already name this event
> 6. Distinguish CONFIRMED (explicit event type found), LIKELY (state change clear but unnamed), POSSIBLE (inferred from data shape)
>
> **Report format:**
>
> ```
> ## State-Transition Domain Events
>
> ### {EventName}
> - aggregate: {EntityName}
> - state_change: {before state} -> {after state} (or: "new entity created" / "entity deleted")
> - existing_type: {class or message type if named in code, or "none"}
> - source: {file:line}
> - confidence: CONFIRMED | LIKELY | POSSIBLE
>
> [Repeat for each event found]
> ```
>
> Report up to 15 events. Prefer events with explicit existing event types.

### Agent 3: Integration Boundary Mapper

#### Agent 3 prompt template

> You are a domain event explorer for an Event Storming session.
>
> **Project goal**: {goal}
> **Domain scope**: {scope}
>
> **Your job**: Discover candidate domain events at integration boundaries — what does this domain publish to the outside world, and what does it consume from other domains?
>
> **Your perspective**: You think like a systems integrator. You look for message buses, event queues, webhooks, notification systems, or inter-service calls that carry state transitions. Integration points reveal domain events that matter enough to cross a boundary — these are often the most important events in the model.
>
> **Investigation protocol:**
> 1. Use Grep and Glob to find message publishers, event emitters, queue producers, webhook dispatchers, or outbound HTTP calls in this domain's files
> 2. For each integration point, identify what domain event is being communicated
> 3. Also find message consumers, event handlers, or queue subscribers that receive events from other domains — these reveal boundary assumptions
> 4. Name the published events (PascalCase past-tense) and label inbound events as "consumed from {other domain}"
> 5. Distinguish CONFIRMED (explicit message type found), LIKELY (integration code found but type inferred), POSSIBLE (integration suspected from naming patterns)
>
> **Report format:**
>
> ```
> ## Integration-Boundary Domain Events
>
> ### {EventName} [published | consumed from {domain}]
> - aggregate: {EntityName}
> - direction: outbound | inbound
> - channel: {message bus, queue name, webhook endpoint, or service call}
> - source: {file:line}
> - confidence: CONFIRMED | LIKELY | POSSIBLE
>
> [Repeat for each event found]
> ```
>
> Report up to 10 events per direction. Integration events often have higher confidence than internal events.

### 1b. Dispatch and Wait

Launch all 3 agents with `run_in_background: true`. Wait for all to complete before proceeding to Phase 2.

```
Task({
  subagent_type: "Explore",
  run_in_background: true,
  prompt: "<Agent 1 prompt with {goal} and {scope} substituted>"
})

Task({
  subagent_type: "Explore",
  run_in_background: true,
  prompt: "<Agent 2 prompt with {goal} and {scope} substituted>"
})

Task({
  subagent_type: "Explore",
  run_in_background: true,
  prompt: "<Agent 3 prompt with {goal} and {scope} substituted>"
})
```

### 1c. Consolidate Findings

Merge the three agent reports into a deduplicated event list. When the same event appears in multiple reports:
- Merge payload and state_change fields
- Take the highest confidence level
- Take all source references

Produce a working list: `N_total` candidate events from `N_command` command-flow, `N_state` state-transition, `N_integration` integration-boundary findings.

---

## Phase 2: Assumption Surfacing

Before generating output, make boundary assumptions explicit. These are what this human believes about adjacent domains — beliefs that other participants may contradict in the jam session.

### 2a. Identify Boundary Assumptions

For each event that crosses a domain boundary (inbound or outbound), or for aggregates that reference entities outside this scope, surface the assumption:

- **Existence assumptions**: "I assume `{Entity}` is owned by `{other domain}`"
- **Contract assumptions**: "I assume `{EventName}` carries `{field}` in its payload"
- **Ordering assumptions**: "I assume `{EventA}` always precedes `{EventB}`"
- **Ownership assumptions**: "I assume `{command or process}` belongs to this domain, not `{adjacent domain}`"

### 2b. Surface Implicit Decisions

Review the consolidated event list for any events where the domain ownership is ambiguous or where you made an implicit modeling decision. Flag these as `assumption_type: ownership`.

### 2c. Rate Assumption Confidence

Each assumption gets a confidence level:
- **CONFIRMED**: Verified by reading inter-domain contracts, API specs, or explicit team docs
- **LIKELY**: Strong evidence from code structure (e.g., foreign keys, import patterns)
- **POSSIBLE**: Inferred from naming or convention — needs explicit confirmation in jam session

---

## Phase 3: Output Generation

Generate the structured YAML artifact. Create `events/candidates/` if it does not exist.

### 3a. Build the YAML

```yaml
# events/candidates/{role}.yaml
# Generated by /storm-prep — Phase 1 of Event Storming workflow
# Consumed by: human jam session, /formalize skill
# Role: {role}
# Domain scope: {scope}
# Project goal: {goal}
# Generated at: {ISO 8601 timestamp}

metadata:
  role: "{role}"
  scope: "{scope}"
  goal: "{goal}"
  generated_at: "{timestamp}"
  event_count: {N}
  assumption_count: {M}

domain_events:
  - name: "{EventName}"
    aggregate: "{AggregateRoot}"
    trigger: "{command or action}"
    payload:
      - field: "{fieldName}"
        type: "{type}"
    state_change: "{before} -> {after}"   # omit if not applicable
    integration:
      direction: outbound | inbound | internal
      channel: "{channel name}"           # omit if internal
    sources:
      - "{file:line}"
    confidence: CONFIRMED | LIKELY | POSSIBLE
    notes: "{any modeling uncertainty or open question}"  # omit if none

  # ... one entry per candidate event

boundary_assumptions:
  - id: "BA-{N}"
    type: existence | contract | ordering | ownership
    statement: "{assumption as a declarative sentence}"
    affects_events:
      - "{EventName}"
    confidence: CONFIRMED | LIKELY | POSSIBLE
    verify_with: "{which other role or domain should confirm this}"

  # ... one entry per assumption
```

### 3b. Write the File

Write to `events/candidates/{role}.yaml`. Create the directory if needed.

After writing, read the file back and confirm:
- All domain events are present (`event_count` matches actual entries)
- All boundary assumptions are present (`assumption_count` matches actual entries)
- YAML is valid (no obvious syntax errors)

State: "Verified: `events/candidates/{role}.yaml` written with {N} events and {M} assumptions."

### 3c. Emit Pipe-Format Summary

Emit a pipe-format block so downstream skills (`/formalize`, `/gather`, `/rank`) can consume the findings.

**Consuming skill detection**: A downstream `/formalize` invocation should detect this output by looking for `**Source**: /storm-prep` in context. The `events/candidates/{role}.yaml` path in the **Input** line is the primary handoff artifact.

```
## Domain Events: {scope}

**Source**: /storm-prep
**Input**: {scope} / {goal}
**Pipeline**: (none — working from direct input)

### Items (N)

1. **{EventName}** — {aggregate}, {direction: internal | outbound | inbound}
   - source: events/candidates/{role}.yaml
   - confidence: CONFIRMED | LIKELY | POSSIBLE

2. **{EventName}** — ...

[... one item per domain event, ordered by confidence descending]

### Assumptions ({M})

1. **BA-1** — {statement}
   - type: {existence | contract | ordering | ownership}
   - verify_with: {other role}
   - confidence: CONFIRMED | LIKELY | POSSIBLE

[... one item per boundary assumption]

### Summary

[One paragraph: total events found, primary aggregates identified, key boundary assumptions that need confirmation in the jam session, and any modeling uncertainties that surfaced during exploration.]
```

---

## Guidelines

1. **Gate is mandatory.** Never skip Phase 0. Scope defines the exploration boundary — without it, agents will wander across the whole codebase.
2. **Events are past-tense facts.** An event describes something that happened, not a command or a state. `OrderPlaced` not `PlaceOrder`, `PaymentFailed` not `PaymentFailure`.
3. **Confidence levels are not optional.** Every event and every assumption must have a confidence level. POSSIBLE findings are still valuable — they surface questions for the jam session.
4. **Assumptions are first-class output.** Boundary assumptions are as important as the events themselves. A wrong assumption about domain ownership causes more rework than a missed event.
5. **Role naming is stable.** The role identifier becomes a file name and a coordination key. Use the domain name, not a person's name. Rename by deleting and re-running with a new scope.
6. **One human, one role, one file.** Each participant in the Event Storming session runs storm-prep for their assigned domain scope. Output files must not overlap — if two humans claim the same scope, that's a jam session topic.
7. **No cross-domain writes.** This skill writes only to `events/candidates/{role}.yaml`. It does not modify other roles' files.
8. **The jam session resolves conflicts.** If exploration surfaces events that seem to belong to another domain, add them with `confidence: POSSIBLE` and flag them in `boundary_assumptions`. Let the humans decide in the jam session.

## What NOT to Do

- **Do not invent events.** Every event in the output must trace to a source in the codebase, project docs, or goal description. Do not generate plausible-sounding events that have no evidence.
- **Do not resolve boundary conflicts unilaterally.** If an aggregate could belong to two domains, mark it as a `POSSIBLE` ownership assumption — do not silently assign it to this scope.
- **Do not use command names as events.** `PlaceOrder` is a command. `OrderPlaced` is an event. If you find yourself naming events without past tense, you're modeling commands, not events.
- **Do not skip assumption surfacing.** Skipping Phase 2 produces an event list that looks complete but hides the decisions embedded in it. Assumptions must be explicit.
- **Do not merge roles.** If you're preparing for two domain scopes, run storm-prep twice with different scopes — one file per scope. Do not combine scopes into one file.
- **Do not treat POSSIBLE as incomplete.** POSSIBLE confidence is valid output — it means "this needs confirmation." Do not suppress POSSIBLE findings to make the output look cleaner.

## See Also

- `/formalize` — Phase 2 of the trilogy: takes `events/candidates/*.yaml` from all participants and produces a unified domain model (aggregate map, bounded contexts, event flow diagram)
- `/spec` — use to design a single domain before or after Event Storming
- `/decompose` — break a large goal into sub-domains before running storm-prep per domain
- `/meeting` — facilitate the human jam session that follows storm-prep across participants
- `/gather` — feeds into storm-prep when the project goal needs research before domain exploration
