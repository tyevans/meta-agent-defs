# Multi-Human Workflows: Introduction

A conceptual entry point for using tackline when two or more humans need to work different parts of a project simultaneously.

---

## The Problem

Single-human orchestration works well. You run `/blossom`, load a sprint, dispatch agents, and ship. But real projects often involve multiple humans with different expertise — a backend engineer, a frontend engineer, a data scientist — each owning a bounded context that must integrate cleanly with the others.

The failure mode isn't technical; it's contractual. Each human's agents sprint ahead independently, and the integration step reveals incompatible assumptions: mismatched event names, schema fields that don't exist, API shapes that were never negotiated.

Multi-human workflows solve this by inserting a shared contract phase **before** each sprint. The contract defines what flows across boundaries. Agents on both sides validate against it continuously. Integration becomes a verification step rather than a debugging session.

---

## The 5-Phase Lifecycle

| Phase | Who | Skill | Output |
|-------|-----|-------|--------|
| **Prep** | Agents | `/storm-prep` | Candidate domain events (YAML) per domain |
| **Jam** | Humans | *(meeting)* | Shared event catalog with ownership assignments |
| **Formalize** | Agents | `/formalize` | Machine-readable contracts: schemas, mocks, validation config |
| **Execute** | Humans + Agents | `/sprint` | Implementation within bounded context, contract-validated |
| **Integrate** | Agents | `/integrate` | Cross-boundary validation, conflict detection, merge proposal |

### Phase Details

1. **Prep** (`/storm-prep`): Each human's Claude agents independently explore their assigned domain using Event Storming methodology. They generate candidate domain events as structured YAML — no coordination yet. Each agent works with full autonomy inside its context.

2. **Jam** *(human activity, no skill)*: Humans meet and review each other's candidate events. They negotiate boundary ownership, rename conflicting terms, and agree on a shared event catalog. The output is a single artifact both sides will use.

3. **Formalize** (`/formalize`): Each human feeds the jam session's shared artifact into Claude. Agents generate machine-readable contracts — event schemas, mock payloads, validation configuration — from the negotiated catalog.

4. **Execute** (`/sprint`): Normal sprint execution within each bounded context. Each human's agents run their standard sprint loop, but with contract-aware validation enabled. Violations surface immediately rather than at integration time.

5. **Integrate** (`/integrate`): Cross-boundary contract validation across all contexts. Conflict detection, schema compatibility checks, and a merge proposal that humans review before merging branches.

---

## Why Event Storming

Event Storming is a collaborative modeling technique where teams identify domain events — things that happened — as the primary unit of shared understanding.

It fits this workflow for two reasons:

- **Simultaneously builds shared understanding AND produces artifacts.** The Jam phase isn't just a meeting; it produces the shared event catalog that feeds `/formalize`. The artifact is the output, not the notes about the meeting.
- **Boundary-first design.** Events flow across context boundaries. Naming and owning events forces boundary negotiation early, when it's cheap, rather than late, when agents have already diverged.

---

## When to Use Multi-Human Workflows

**Use multi-human workflows when:**
- Two or more humans are working different parts of the same system simultaneously
- Work spans context boundaries (frontend/backend, service A/service B, data pipeline stages)
- Integration failures would be expensive or block downstream work
- You need explicit contracts that agents on both sides can validate

**Stay single-human when:**
- One person owns the full stack for this sprint
- Context boundaries are internal — you're the only author
- The work is exploratory and contracts would constrain too early

Rule of thumb: if two humans would be blocked waiting for each other without a contract, use multi-human workflows.

---

## Swim-Lane Diagram

```
         PREP          JAM        FORMALIZE      EXECUTE      INTEGRATE
          |             |             |              |             |
Human A   |  reviews    | negotiates  | reviews      | sprints on  | reviews
          |  /storm-prep| boundaries  | contracts    | domain A    | merge prop
          |             |             |              |             |
Human B   |  reviews    | negotiates  | reviews      | sprints on  | reviews
          |  /storm-prep| boundaries  | contracts    | domain B    | merge prop
          |             |             |              |             |
Agents A  | /storm-prep |     ---     | /formalize   |  /sprint    | /integrate
          | (domain A)  |             | (domain A)   |  (domain A) |
          |             |             |              |             |
Agents B  | /storm-prep |     ---     | /formalize   |  /sprint    | /integrate
          | (domain B)  |             | (domain B)   |  (domain B) |
          |             |             |              |             |
Shared    |  candidate  | shared      | contracts    | validation  | conflict
Artifact  |  events     | event       | + schemas    | reports     | report +
          |  (YAML)     | catalog     | + mocks      |             | merge PR
```

Agent activity is automated. The Jam phase is human-only — it's the negotiation step that only humans can do well. All other phases are agent-heavy with humans in a review role.

---

## Relationship to Existing Tackline Primitives

Multi-human workflows sit **above** the existing team and sprint system:

```
multi-human workflows     <- this layer: /storm-prep, /formalize, /integrate
      |
  /sprint + /assemble     <- team orchestration layer
      |
  /blossom + primitives   <- exploration and composition layer
```

- `/sprint` still runs inside each bounded context during Execute. Multi-human workflows add the contract envelope around it.
- `/assemble` can be used to set up each human's agent team before Prep begins.
- `/blossom` is an alternative to `/storm-prep` for unconstrained exploration — use it when you don't yet know the domain boundaries.

For single-human workflows, see [Workflow Pipelines](pipelines.md). For team setup, see [Team System Guide](team-system-guide.md).
