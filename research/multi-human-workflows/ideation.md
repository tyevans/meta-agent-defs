# Multi-Human Workflow System: Design Rationale

**Status:** Reference document — not user-facing
**Audience:** Future skill authors and system designers
**Created:** 2026-02-26

---

## 1. Problem Statement

Tackline's current model — one human orchestrating Claude agents — works well for solo projects. The human owns the full context, makes all architectural decisions, and Claude agents execute within that unified context.

This model breaks down when real projects involve multiple humans with different expertise working simultaneously. Examples:

- A frontend engineer and a backend engineer each running their own Claude Code sessions on the same product
- A data scientist and an ML engineer who need to agree on model I/O contracts before implementing independently
- A team doing a bounded context decomposition where different humans own different subdomains

The gap: there is no structured way to coordinate between humans who each have their own Claude Code sessions. When two humans produce separate artifacts, they need a forcing function to:

1. Surface where their implicit assumptions differ
2. Negotiate shared boundaries before divergence is baked into code
3. Generate machine-readable contracts from the negotiated outcome
4. Validate that parallel implementation stayed within the agreed contracts

Ad hoc coordination (Slack messages, shared docs, standups) lacks structure and produces no artifacts. The multi-human workflow system fills this gap with a phased, artifact-driven approach.

---

## 2. Methodology Choice: Event Storming

Event Storming (Alberto Brandolini) was selected as the methodological foundation for three reasons:

### Simultaneous output: understanding + artifacts

Most alignment techniques (whiteboard sessions, PRDs, architecture diagrams) produce understanding but not machine-readable artifacts. Event Storming, when adapted for this system, produces both: humans build shared mental models during the jam phase, and those models are immediately encoded as versioned YAML event schemas.

### Domain events as natural cross-boundary messages

Domain events are things that happened — past tense, immutable, owned by a bounded context. This maps cleanly to API contracts between human-owned domains:

- The event producer (one human's domain) defines what happened
- The event consumer (another human's domain) declares what it needs to know
- Mismatches surface immediately as schema conflicts, not runtime failures

### Bounded contexts map to human ownership

Conway's Law applies to human teams: system boundaries tend to follow communication boundaries. Event Storming surfaces bounded contexts explicitly, making the natural ownership split visible before anyone writes code. Each human can then own a bounded context and sprint independently.

### Phased structure maps naturally to agent workflows

Event Storming has a natural flow from chaotic exploration to structured formalization. This maps directly to a skill trilogy where each skill handles one phase transition:

```
Exploration (agents) → Human Jam → Formalization (agents) → Parallel Sprint → Integration (agents)
```

---

## 3. The 5-Phase Simplified Model

Event Storming in its full form is a multi-day workshop with physical sticky notes. The simplified model compresses this into a structure compatible with async, distributed teams using Claude Code.

### Phase 1: Prep (Independent Exploration)

Each human runs `/storm-prep` in their own Claude Code session. Agents analyze that human's domain — reading existing code, docs, and specs — and produce candidate domain events as YAML. No cross-human coordination happens here.

Output: `events/candidates/{role}.yaml` per human participant.

The goal is to surface what each human already knows about their domain before they talk to each other. This prevents the jam session from being dominated by whoever speaks first.

### Phase 2: Jam (Human Synchronization — NO AUTOMATION)

The humans meet (synchronously or async) and negotiate:

- Which events are real vs. implementation artifacts
- Where the bounded context boundaries fall
- Who owns which events
- Which events cross boundaries (integration events)

The output of this phase is a shared artifact: a jam session document (Markdown or YAML) capturing the agreed event list, ownership assignments, and boundary decisions.

This phase is deliberately not automated. The negotiation IS the value. Automating it would produce a plausible-looking output that hides the disagreements rather than resolving them.

### Phase 3: Formalize (Contract Generation)

One human (or both) runs `/formalize` against the jam artifact. Agents parse the negotiated event list and generate:

- Machine-readable event schemas (versioned YAML)
- Mock producers/consumers for each cross-boundary event
- Validation configuration for contract testing

Output is committed to a shared location (e.g., `contracts/events/`). Both humans' subsequent work depends on these contracts.

### Phase 4: Execute (Parallel Implementation)

Each human sprints in their bounded context using normal tackline workflows (`/status`, `/sprint`, etc.). The contracts from Phase 3 serve as the implementation target. Agents can reference the event schemas to generate stubs, validate outputs, and catch drift early.

No cross-human coordination is required during this phase by design. The contracts absorb the coordination cost.

### Phase 5: Integrate (Cross-Boundary Validation)

When implementation is ready for integration, either human runs `/integrate`. Agents:

- Load the contracts from Phase 3
- Inspect the actual implementations from both domains
- Compare against the agreed schemas
- Detect divergence (schema drift, missing events, semantic mismatches)
- Produce a pipe-format integration report with CONFIRMED/LIKELY/POSSIBLE confidence tags

The report drives the conversation about what to fix before merging.

---

## 4. Key Design Decisions

### Jam session is not automated

This was the central design decision. Early exploration considered generating a "suggested" boundary split from the two prep YAMLs. This was rejected because:

- The value of alignment is that humans understand and agree to the boundaries, not that a tool picked them
- Automated boundary detection would surface plausible but potentially wrong splits that teams might accept without scrutiny
- Human negotiation produces accountability: the humans who agreed own the decision

The system supports the jam session with structure (YAML templates, candidate event lists) but does not attempt to run it.

### Confidence tags match tackline's existing convention

Event schemas carry `CONFIRMED / LIKELY / POSSIBLE` confidence tags, consistent with the confidence convention used in tackline's spike findings (established in the blossom/fractal workflow). This avoids introducing a parallel vocabulary and lets agents that already understand the convention reason about contract confidence without new training.

### Event schemas are versioned YAML, not code

Contracts are stored as YAML files, not as code (TypeScript interfaces, Protobuf schemas, OpenAPI specs). Rationale:

- Portability: YAML is language-agnostic; teams using different languages share the same schema files
- Agent readability: LLM agents parse YAML more reliably than code-embedded schemas
- Versioning: YAML files version naturally in git with meaningful diffs
- Late binding: Code generation from YAML is easy; going the other direction is not

Teams that need code representations can generate them from YAML as a build step.

### Each human operates in their own Claude Code session

No shared session, no shared context window. This matches reality: humans on distributed teams do not share a terminal. The system is designed for async-first coordination, with synchronous touchpoints only at the jam phase boundary.

### Output flow: prep → human → formalize (not prep → formalize)

The output of `/storm-prep` (candidate events YAML) feeds into the jam session for humans to read, not directly into `/formalize`. `/formalize` consumes the jam artifact — the post-negotiation output — not the raw prep output.

This separation is important: it means `/formalize` always operates on negotiated, human-validated inputs. Bypassing the jam by piping prep output directly to formalize would produce contracts that haven't been agreed to.

---

## 5. Skill Trilogy Design

### /storm-prep

- **Type:** Inline skill
- **Trigger:** Each human runs independently before the jam session
- **Mechanism:** Dispatches exploration agents to analyze the human's domain
- **Output:** `events/candidates/{role}.yaml` — candidate domain events with confidence tags
- **Does not:** Contact other humans, read their prep output, or attempt boundary detection

The skill name uses "storm" as shorthand for Event Storming without requiring users to know the methodology.

### /formalize

- **Type:** Inline skill
- **Trigger:** Run after the jam session produces a shared artifact
- **Mechanism:** Parses the jam artifact; dispatches agents to generate schemas, mocks, and validation config
- **Input:** Jam session artifact (Markdown or YAML capturing negotiated decisions)
- **Output:** `contracts/events/*.yaml`, mock files, validation configuration
- **Does not:** Re-run boundary negotiation; trusts the jam output

### /integrate

- **Type:** Inline skill
- **Trigger:** Run when implementations are ready for cross-boundary validation
- **Mechanism:** Loads contracts from Phase 3; inspects implementations; diffs against schemas
- **Output:** Pipe-format integration report (CONFIRMED/LIKELY/POSSIBLE findings, divergence list, merge recommendations)
- **Does not:** Auto-fix divergence; produces a report for humans to act on

All three skills follow the pipe-format output contract for composability with other tackline primitives.

---

## 6. Future Extensions

### /negotiate

When `/integrate` finds divergence, the next step is renegotiating the affected contracts. `/negotiate` would automate the proposal side: given two divergent implementations and the original contract, generate candidate reconciliation options with tradeoff analysis. Humans still make the final call.

This extension is explicitly not part of the first iteration to keep the scope bounded and avoid the automated-negotiation trap described in Section 4.

### Integration with /meeting

The jam session (Phase 2) currently relies on humans using whatever tools they prefer. A `/meeting` integration could scaffold a structured jam session agenda, capture decisions in a standard format, and output a jam artifact directly. This would close the loop between tackline's meeting workflow and the multi-human workflow system.

### Multi-repo support

The current design assumes both humans work in the same repository. Real teams often work across repos. Multi-repo support would require:

- A shared contracts repository (separate from either implementation repo)
- Cross-repo event schema references
- `/integrate` pulling implementations from multiple remotes

This is a significant infrastructure change deferred to a later iteration.

---

## Appendix: Artifact Layout

```
project/
├── events/
│   └── candidates/
│       ├── backend.yaml       # Output of /storm-prep (backend human)
│       └── frontend.yaml      # Output of /storm-prep (frontend human)
├── jam/
│   └── session-2026-02-26.md  # Jam session artifact (human-produced)
└── contracts/
    └── events/
        ├── order-placed.yaml  # Generated by /formalize
        ├── payment-confirmed.yaml
        └── ...
```

Confidence tag usage in event schemas:

```yaml
event: order-placed
confidence: CONFIRMED   # Both humans agreed; schema is stable
owner: backend
consumers:
  - frontend
  - analytics
fields:
  order_id:
    type: string
    confidence: CONFIRMED
  discount_code:
    type: string
    confidence: POSSIBLE  # Not yet agreed — may be dropped
```
