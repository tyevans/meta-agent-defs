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

2. **Jam** *(human activity, no skill)*: Humans meet and review each other's candidate events. Each side's prep output carries **boundary assumptions** — implicit expectations about what data crosses context lines: field names, types, which context owns which events. The Jam surfaces these assumptions and negotiates them while they're cheap to change. Humans rename conflicting terms, agree on payload shapes, and assign event ownership. The output is a **shared event catalog** — a single YAML file listing every cross-boundary event with its owning context, payload fields, and types. This catalog becomes the source of truth that `/formalize` converts into machine-readable contracts.

3. **Formalize** (`/formalize`): Each human feeds the jam session's shared artifact into Claude. Agents generate machine-readable contracts — event schemas, mock payloads, validation configuration — from the negotiated catalog.

4. **Execute** (`/sprint`): Normal sprint execution within each bounded context. Each human's agents run their standard sprint loop, but with contract-aware validation enabled. Violations surface immediately rather than at integration time.

5. **Integrate** (`/integrate`): Cross-boundary contract validation across all contexts. Conflict detection, schema compatibility checks, and a merge proposal that humans review before merging branches.

---

## Example Walkthrough

Product says "add Stripe payments." Two engineers are involved: one owns the payment backend, the other owns the checkout frontend.

**Prep.** Each engineer runs `/storm-prep` on their domain. The backend engineer gets candidate events like `PaymentInitiated`, `PaymentSucceeded`, `PaymentFailed`, `RefundRequested`. The frontend engineer gets `CheckoutStarted`, `CartAbandoned`, `PaymentFormSubmitted`. Neither has seen the other's list yet.

**Jam.** They meet for 30 minutes. The backend engineer says "`PaymentSucceeded` carries `paymentId`, `orderId`, `amount`, and `currency`." The frontend engineer says "We need `orderId` and `amount` to show the confirmation screen — can you guarantee those fields?" They write it down. They discover the backend was calling it `amount` (integer cents) while the frontend assumed `total` (decimal dollars). They agree on `amountCents: integer`. The output is a shared event catalog — a single YAML file both sides will reference.

**Formalize.** Each engineer runs `/formalize` on the shared catalog. Out come JSON schemas for every event, mock payloads for testing, and validation config their agents will use during the sprint. The `PaymentSucceeded` schema now has `amountCents: integer` locked in.

**Execute.** Each engineer sprints independently using `/sprint`. The backend builds the Stripe integration. The frontend builds the checkout flow. Their agents validate against the formalized contracts continuously. If the backend engineer's agent changes `amountCents` to `totalCents`, the contract check flags it immediately — not three days later at merge time.

**Integrate.** Both engineers are done. `/integrate` checks both branches against the contracts, flags any remaining drift, and proposes a merge. The engineers review the merge proposal and ship.

Without the contract phase, you discover at merge time that one side renamed a field. With it, you catch that during the sprint.

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

## Why Event Storming

Event Storming is a collaborative modeling technique where teams identify domain events — things that happened — as the primary unit of shared understanding.

It fits this workflow for two reasons:

- **Simultaneously builds shared understanding AND produces artifacts.** The Jam phase isn't just a meeting; it produces the shared event catalog that feeds `/formalize`. The artifact is the output, not the notes about the meeting.
- **Boundary-first design.** Events flow across context boundaries. Naming and owning events forces boundary negotiation early, when it's cheap, rather than late, when agents have already diverged.

---

For the step-by-step guide with concrete commands, see [Multi-Human How-To](multi-human-howto.md). For single-human workflows, see [Workflow Pipelines](pipelines.md). For team setup, see [Team System Guide](team-system-guide.md).
