---
name: formalize
description: "Use after a human jam session to turn agreements into code. Converts shared artifacts into versioned event schemas, mocks, and validation config. Keywords: formalize, contracts, event schemas, boundaries, multi-human, event storming."
argument-hint: "<path to shared jam artifact>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Write, Edit, Bash(git:*), Task
context: inline
---

# Formalize: Event Contract Generation from Jam Artifacts

You are running the **formalize** skill — Phase 3 of the multi-human Event Storming workflow. Input: **$ARGUMENTS**

This skill consumes the shared jam artifact produced by a human jam session (seeded by `/storm-prep`) and generates machine-readable event contracts that `/integrate` will use to wire services together.

## When to Use

- After a human jam session has produced a shared artifact documenting bounded contexts, owned events, and consumed events
- When you need to translate verbal or whiteboard agreements into versioned event schemas
- When each team member needs their own contract-aware mocks and validation config for local development
- Before running `/sprint` agents — contracts gate all implementation tasks

## DO NOT Use When

- You have not completed a human jam session — schemas from assumptions are not contracts
- The shared artifact is incomplete (missing ownership, unclear boundaries, or unresolved disputes)
- You are trying to shortcut the jam session — formalize codifies agreement, it does not create it

## Named Law

**NO SCHEMA WITHOUT JAM ARTIFACT** — If you are formalizing from assumptions rather than a shared artifact, STOP. Return to `/storm-prep` and complete the jam session. Contracts without jam artifacts are guesses wearing a schema's clothing.

## What NOT to Do

- **Do not resolve ambiguities by assuming** — If the jam artifact says "TBD" or leaves ownership unclear, surface the ambiguity to the user and halt. Do not pick a side.
- **Do not treat silence as agreement** — An event that appears in diagrams but has no explicit owner is unresolved. Mark it POSSIBLE and flag it.
- **Do not skip confidence tagging** — Every field, every event, every boundary gets a confidence tag: CONFIRMED (explicitly agreed in session), LIKELY (implied by context or ownership patterns), POSSIBLE (assumed from usage without explicit agreement).
- **Do not generate schemas for another team's bounded context** — Each human runs formalize for their own context only. Cross-boundary schemas come from consuming, not owning.
- **Do not invent field names** — Use names as agreed in the jam artifact. If a field name is ambiguous, note the ambiguity and use the artifact's wording verbatim.

## Overview

```
$ARGUMENTS (shared jam artifact path)
    |
    v
Phase 0: Gate — artifact exists and readable?
    |
    v
Phase 1: Parse — extract this human's bounded context, owned events, consumed events
    |
    v
Phase 2: Schema generation — versioned YAML schemas with field definitions and confidence tags
    |
    v
Phase 3: Mock generation — mock producers for consumed events, stub consumers for provided events
    |
    v
Phase 4: Validation config — contract-aware validation for local /sprint agents
    |
    v
Output: contracts/agreed/*.schema.yaml + mocks/ + validation config + pipe-format summary
```

---

## Phase 0: Gate (Precondition Check)

**STOP if any gate fails. Do not proceed to Phase 1 until all gates pass.**

### Gate 1 — Artifact path provided

If `$ARGUMENTS` is empty, ask: "What is the path to the shared jam artifact? (e.g., `docs/jam/session-YYYY-MM-DD.md`)"

### Gate 2 — Artifact exists and is readable

Read the file at `$ARGUMENTS`. If the file does not exist or cannot be read:

> "Jam artifact not found at `<path>`. Ensure the shared artifact from your human jam session is committed and accessible before running /formalize. If you have not completed a jam session, run /storm-prep first."

### Gate 3 — Artifact has required structure

Scan the artifact for:
- A bounded context section identifying ownership assignments
- An event list with explicit owners (not just "discussed" or "TBD")
- At least one event with a clear owner and at least one field definition

If the artifact is missing any of these:

> "The jam artifact at `<path>` is missing required sections: [list what is missing]. Complete the human jam session and update the artifact before formalizing. Formalize codifies agreement — it cannot create it."

### Gate 4 — Identify this human's context

Ask (or detect from $ARGUMENTS if a context name was provided): "Which bounded context are you formalizing contracts for?"

State the detected context before proceeding: "Formalizing contracts for bounded context: **`<context-name>`**"

---

## Phase 1: Parse Shared Artifact

Read the full shared artifact. Extract three lists for **this human's bounded context only**:

### 1a. Owned Events

Events this context produces and owns — the events for which this team generates authoritative schemas.

For each owned event, extract:
- Event name
- Version (if stated; default to `v1` if not)
- All fields mentioned in the artifact, with types if stated
- Any business rules or constraints mentioned
- Confidence level for each element (CONFIRMED / LIKELY / POSSIBLE)

### 1b. Consumed Events

Events this context receives from other bounded contexts. This context does not own these schemas — it generates mocks for local testing only.

For each consumed event, extract:
- Event name
- Owning context (which team produces it)
- Fields this context is known to read
- Any SLA or ordering constraints mentioned

### 1c. Boundary Agreements

Explicit interface contracts agreed at the boundary:
- Which events cross which context boundaries
- Any versioning agreements (e.g., "Orders must not break before v2 migration completes")
- Any ordering or causality constraints

### Parse Report

Print a structured summary before proceeding to Phase 2:

```
## Parse Summary: <context-name>

**Owned events**: N
  - <event-name> (CONFIRMED | LIKELY | POSSIBLE)
  - ...

**Consumed events**: N
  - <event-name> from <owning-context>
  - ...

**Boundary agreements**: N
  - <agreement summary>
  - ...

**Unresolved items** (require user input before schema generation):
  - <item> — [what is unclear]
  - ...
```

**If unresolved items exist**, ask the user to resolve them before continuing. Do not proceed to Phase 2 with unresolved ownership or missing field definitions for CONFIRMED events.

---

## Phase 2: Schema Generation

Generate versioned event schemas as YAML for **owned events only**. Store each schema at `contracts/agreed/<event-name>.<version>.schema.yaml`.

Create `contracts/agreed/` if it does not exist.

### Schema Format

Each schema file follows this structure:

```yaml
# contracts/agreed/<event-name>.<version>.schema.yaml
schema:
  name: <EventName>
  version: "<version>"
  owner: "<bounded-context-name>"
  source: "<path-to-jam-artifact>"
  formalized: "<YYYY-MM-DD>"

description: >
  <One paragraph describing what this event means in the domain,
  when it is emitted, and what guarantees the producer makes.>

fields:
  - name: <field-name>
    type: <string | integer | boolean | object | array | uuid | timestamp | enum>
    required: true | false
    description: "<What this field means. Cite the jam artifact if named explicitly.>"
    confidence: CONFIRMED | LIKELY | POSSIBLE
    constraints:
      - <e.g., "non-empty string", "ISO 8601 timestamp", "must be a valid UUID">
    example: <example value>

  - name: <field-name>
    ...

envelope:
  topic: "<message-bus-topic or queue name, if agreed>"
  ordering-key: "<field used for ordering, if agreed>"
  idempotency-key: "<field used for deduplication, if agreed>"

unresolved:
  - "<description of any POSSIBLE-confidence element needing human sign-off>"
```

### Confidence Tagging Rules

- **CONFIRMED**: The jam artifact explicitly states this field, type, or constraint using the team's own words. Direct quote available.
- **LIKELY**: The field is implied by the event's purpose or by ownership patterns, but not explicitly named in the artifact.
- **POSSIBLE**: This element is assumed from usage patterns or analogous events. Needs human sign-off before implementation.

Tag every field with its confidence level. Do not omit tags on the grounds that "it is obvious."

### Schema Generation Loop

For each owned event:

1. Create the schema file at `contracts/agreed/<event-name>.<version>.schema.yaml`
2. Populate from the parse output in Phase 1a
3. Tag each field with confidence
4. Add any POSSIBLE-confidence items to the `unresolved` section
5. State: "Written: `contracts/agreed/<event-name>.<version>.schema.yaml` (N fields, M unresolved)"

After all schemas are written, list the schema files created.

---

## Phase 3: Mock Generation

Generate mock producers and stub consumers to enable local development without cross-team coordination.

### 3a. Mock Producers (for consumed events)

For each consumed event identified in Phase 1b, generate a mock producer that emits plausible test payloads. This lets local agents simulate upstream events without requiring the owning team's service.

Create `mocks/producers/<event-name>-mock.yaml`:

```yaml
# mocks/producers/<event-name>-mock.yaml
mock:
  event: <EventName>
  owner: <owning-context>
  warning: "This mock is for LOCAL TESTING ONLY. Do not use in integration or production."
  source-schema: "<path-to-owning-team-schema if available, else 'unknown — schema not yet received'>"

payloads:
  - label: "happy path"
    fields:
      <field>: <plausible-test-value>
      ...

  - label: "edge case: <description>"
    fields:
      <field>: <edge-case-value>
      ...
```

Notes:
- Use plausible but clearly synthetic values (e.g., `user-id: "test-user-001"`, not real UUIDs)
- Include at least one happy-path payload and one edge-case payload
- Mark the mock as LOCAL TESTING ONLY

### 3b. Stub Consumers (for owned events)

For each owned event, generate a stub consumer that documents the expected consumption pattern. This serves as a contract test target.

Create `mocks/consumers/<event-name>-stub.yaml`:

```yaml
# mocks/consumers/<event-name>-stub.yaml
stub:
  event: <EventName>
  version: "<version>"
  owner: "<this-context>"
  schema: "contracts/agreed/<event-name>.<version>.schema.yaml"

consumption-contract:
  reads-fields:
    - <field-name>: "<why this field matters to consumers>"
    ...
  ordering-guarantee: "<guaranteed | best-effort | none>"
  idempotency: "<idempotent | at-least-once | exactly-once | unknown>"

contract-tests:
  - name: "rejects events missing required fields"
    input: { <omit a required field> }
    expected: reject

  - name: "accepts valid happy-path event"
    input: { <all required fields with valid values> }
    expected: accept
```

---

## Phase 4: Validation Config

Generate a validation config that local `/sprint` agents can use to enforce contract compliance during development.

Create `contracts/validation.yaml`:

```yaml
# contracts/validation.yaml
# Generated by /formalize from <path-to-jam-artifact>
# DO NOT edit manually — re-run /formalize to regenerate

context: "<bounded-context-name>"
formalized: "<YYYY-MM-DD>"

schemas:
  - event: <EventName>
    version: "<version>"
    schema: "contracts/agreed/<event-name>.<version>.schema.yaml"
    role: producer  # this context produces this event
    enforce: strict  # reject events that fail schema validation

  - event: <ConsumedEventName>
    version: "unknown"  # version not yet confirmed from owning team
    mock: "mocks/producers/<event-name>-mock.yaml"
    role: consumer  # this context consumes this event
    enforce: warn   # warn on schema mismatches but do not fail

sprint-agent-instructions: |
  Before dispatching sprint agents that produce or consume events:
  1. Check contracts/validation.yaml to identify owned vs consumed events
  2. Owned events: schema at contracts/agreed/ is authoritative — validate against it
  3. Consumed events: use mocks/producers/ for local testing — do not call real endpoints
  4. Flag any POSSIBLE-confidence fields in schemas as needing human sign-off before shipping
  5. Do not treat consumed event mocks as confirmed schemas — verify with the owning team

unresolved-flags:
  - <description of each POSSIBLE-confidence item requiring human sign-off>
```

After writing `contracts/validation.yaml`, read it back and verify it was written correctly. State: "Verified: `contracts/validation.yaml` written with N owned events and M consumed events."

---

## Output Summary

After all phases complete, emit a pipe-format summary block. This output is consumed by `/integrate` as its Phase 0 input.

```markdown
## Formalize Output: <bounded-context-name>

**Source**: /formalize
**Input**: <path to shared jam artifact>
**Pipeline**: /storm-prep (human jam session) -> /formalize

### Items (N)

1. **<EventName> schema** — versioned schema at contracts/agreed/<event-name>.<version>.schema.yaml
   - source: contracts/agreed/<event-name>.<version>.schema.yaml
   - confidence: CONFIRMED | LIKELY

2. **<ConsumedEventName> mock** — mock producer for local testing
   - source: mocks/producers/<event-name>-mock.yaml
   - confidence: LIKELY

3. **Validation config** — sprint-agent contract enforcement rules
   - source: contracts/validation.yaml
   - confidence: CONFIRMED

[... one item per schema, mock, and unresolved flag]

N+1. **Unresolved: <element>** — requires human sign-off before implementation
   - source: <jam artifact section>
   - confidence: POSSIBLE

### Summary

<One paragraph: which bounded context was formalized, how many owned schemas were generated, how many consumed mocks, how many unresolved items require sign-off, and what the integration gate is.>
```

The pipe-format block is the handoff artifact for `/integrate`. It lists all schemas, mocks, and unresolved items that the integration phase must reconcile across teams.

---

## Guidelines

1. **Artifact is truth** — Every schema element traces to the jam artifact. When in doubt, cite the artifact line. If you cannot cite it, the element is POSSIBLE.
2. **One context per run** — Each human runs formalize for their own bounded context only. Do not generate schemas for events owned by other teams.
3. **Confidence is non-negotiable** — Every field gets a confidence tag. Fields without tags will silently mislead sprint agents.
4. **Mocks are not schemas** — A mock producer for a consumed event is a testing convenience, not an authoritative schema. Label it accordingly.
5. **Unresolved items are features, not bugs** — Surfacing POSSIBLE items for human sign-off is the skill working correctly. Do not suppress them to produce a "cleaner" output.
6. **Verify every write** — After writing each schema file, read it back to confirm it was written. Do not report success without verification.
7. **Contracts gate implementation** — Sprint agents should not implement producers or consumers until the relevant schema is at CONFIRMED confidence. The validation config encodes this gate.

## See Also

- `/storm-prep` — Phase 2 of the multi-human workflow: produces the shared jam artifact that this skill consumes. **Detection pattern for Phase 0**: look for a `## Bounded Contexts` section and explicit ownership tables in the shared artifact.
- `/integrate` — Phase 4 of the multi-human workflow: consumes this skill's pipe-format output to wire schemas across team boundaries. **Consumption interface**: reads the `## Formalize Output: <context>` pipe-format block and cross-references schemas from all participants.
- `/spec` — If agreement on a bounded context or event cannot be reached, use /spec to produce a formal design document before re-running the jam session.
- `/test-strategy` — Consumes the stub consumers and contract tests in `mocks/consumers/` to generate a full test plan for contract compliance.
