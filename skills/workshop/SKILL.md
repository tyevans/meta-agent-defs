---
name: workshop
description: "Run a research-backed facilitated discussion with user checkpoints at every decision point. Agents first perform deep investigation spikes (blossom-style) to ground their positions in evidence, then debate with the user validating or redirecting each settled position. Produces a documented markdown artifact of findings and deliberation. Use when you want evidence-based collaborative design with human-in-the-loop control. Keywords: workshop, deliberate, discuss, interactive, checkpoint, decision, validate, collaborative, human-in-the-loop, research, spike, investigate."
argument-hint: "<topic or question>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Write, Grep, Glob, Bash(bd:*), Bash(git:*), Task, SendMessage, TeamCreate, TeamDelete, AskUserQuestion
---

# Workshop: Research-Backed Facilitated Discussion

You are facilitating a **Workshop** — a structured process where **independent** agents first research a topic, then form positions, with the user as the active decision-maker who validates every settled position. Unlike a meeting (agents converse with each other), workshop agents are **completely isolated** — they never see each other's findings or positions. The facilitator mediates all information flow, and the user sees the full picture at every checkpoint. This prevents anchoring, groupthink, and authority bias.

**Topic:** $ARGUMENTS

## When to Use

- When decisions need to be grounded in actual evidence, not just opinions
- When you want agents to investigate before they argue
- When you have domain context that agents lack and want to inject it at the right moments
- When the stakes are high enough that positions should be backed by findings
- When you want a documented record of both the research and the decisions

## Visual Workflow

```
Phase 1: Assemble Panel
  │
  ▼
Phase 2: Research Spikes (agents investigate INDEPENDENTLY)
  ┌──────────┐    ┌──────────┐
  │ Agent A   │    │ Agent B   │   ← No shared context.
  │ researches│    │ researches│      Each sees only their
  │ from own  │    │ from own  │      role + the topic.
  │ perspective│   │ perspective│
  └─────┬─────┘   └─────┬─────┘
        │                │
        ▼                ▼
  Spike report A    Spike report B
        │                │
        └───────┬────────┘
                ▼
  ┌──────────────────┐
  │ USER CHECKPOINT   │  You see BOTH reports side-by-side.
  │ Approve / Adjust  │  Agents see only their own.
  └────────┬─────────┘
           │
           ▼
Phase 3: Opening Positions (each agent states position independently)
  │
  ▼
┌──────────────────────────────────────┐
│  Phase 4: Deliberation Cycle         │
│                                      │
│  Facilitator asks each agent         │
│  the same question independently     │
│           │                          │
│           ▼                          │
│  ┌─────────────────┐                 │
│  │ USER CHECKPOINT  │◄───────┐       │
│  │ See both positions        │       │
│  │ Accept / Redirect /       │       │
│  │ Add context / Challenge   │       │
│  └────────┬─────────┘       │       │
│           │                  │       │
│     ┌─────┴──────┐          │       │
│     │            │           │       │
│   Accept    Redirect/Add     │       │
│     │            │           │       │
│     ▼            ▼           │       │
│  Constraint   Only the       │       │
│  sent to      affected agent │       │
│  BOTH agents  hears it ──────┘       │
│  (no attrib)                         │
│                                      │
│  (repeat until all positions         │
│   settled or user concludes)         │
└──────────────────────────────────────┘
  │
  ▼
Phase 5: Document & Close
  → Write deliberation record with research + decisions to file
```

## Phase 1: Assemble the Panel

### 1a. Choose Roles

Based on the topic, select **2 roles** with genuinely opposed perspectives:

| Role | Perspective | Good for |
|------|------------|----------|
| Architect | System design, patterns, tradeoffs | Technical design discussions |
| Skeptic | Risk, edge cases, what could go wrong | Pressure-testing ideas |
| User Advocate | UX, developer experience, simplicity | Feature design |
| Domain Expert | Deep knowledge of the specific area | Domain-specific questions |
| Pragmatist | What's achievable, incremental path | Scoping and prioritization |
| Historian | Precedent, what's been tried before | Avoiding past mistakes |
| Innovator | Novel approaches, challenge assumptions | Breaking out of ruts |

Select two roles whose perspectives produce the most productive disagreement on this topic.

### 1b. Create the Team

```
TeamCreate({ team_name: "workshop-<short-topic-slug>" })
```

### 1c. Spawn Panelists

For each role, spawn a teammate:

```
Task({
  team_name: "workshop-<slug>",
  name: "<role-name>",
  subagent_type: "general-purpose",
  run_in_background: true,
  prompt: "<panelist prompt>"
})
```

**Panelist prompt template:**

> You are the **[Role]** in a workshop about: [topic]
>
> Your perspective: [2-3 sentences describing what this role cares about and how they think — be rich and specific with genuine characterization]
>
> **CRITICAL: You MUST use the SendMessage tool to communicate.** Your plain text output is NOT visible to anyone. Every response you give must be sent via `SendMessage({ type: "message", recipient: "team-lead", content: "...", summary: "..." })`. Always send to **team-lead** (the facilitator) — never send directly to other panelists. If you do not call SendMessage, nobody will see what you said.
>
> **Workshop rules:**
> 1. Always respond from your role's specific perspective, never as a generic assistant.
> 2. When the facilitator sends you a question, respond with 2-4 paragraphs of concrete, specific analysis.
> 3. **Ground every claim in evidence.** Cite file paths, line numbers, documentation, or research findings. Ungrounded opinions carry no weight in this workshop.
> 4. **You are independent.** You do not know what other panelists think or what they found. Form your positions based solely on your own research, your role's perspective, and any ground-truth constraints the facilitator provides.
> 5. When the facilitator states a **ground-truth constraint** (e.g., "The following has been accepted as settled: X"), treat it as a fixed requirement — build on it, do not argue against it. You will not be told who proposed it or why.
> 6. When asked to state your final position on a sub-question, be explicit and concrete: "My position is: [X] because [Y], supported by [evidence]."

### 1d. Initialize the Deliberation Log

Maintain an internal log structure throughout the workshop. This becomes the output artifact.

```
deliberation_log = {
  topic: "<topic>",
  panelists: [{ role, perspective }],
  research_findings: [],
  rounds: [],
  accepted_positions: [],
  user_contributions: []
}
```

---

## Phase 2: Research Spikes

Before any debate begins, each panelist investigates the topic from their perspective. This is the blossom-style mechanism that grounds positions in evidence.

### 2a. Define Research Scopes

For each panelist, define a **research spike** tailored to their role and the topic. The spike should direct them to investigate specific aspects relevant to their perspective.

Example: If the topic is "authentication strategy" and the roles are Architect and Skeptic:
- Architect spike: "Investigate existing auth patterns in the codebase, integration points, and architectural constraints. Map what exists and what's possible."
- Skeptic spike: "Investigate security vulnerabilities in current auth, edge cases in proposed approaches, and failure modes in similar systems. Find what could go wrong."

### 2b. Dispatch Research

Send each panelist their research instructions via DM:

```
SendMessage({
  type: "message",
  recipient: "<role-name>",
  content: "RESEARCH PHASE — Before we begin debating, investigate the topic from your perspective.

Your research assignment: [role-specific spike description]

Instructions:
1. Use Glob, Grep, and Read to find and examine relevant files, docs, or code.
2. For each finding, state your confidence level:
   - CONFIRMED: You read the source and verified it
   - LIKELY: Strong evidence but incomplete verification
   - POSSIBLE: Suspicious pattern needing deeper investigation
3. Cite specific file paths and line numbers for every finding.

Report your findings in this format:

## Research Findings: [Role] on [topic]

### Items

1. **[finding title]** — [what you found and how you verified]
   - source: [file:line or URL]
   - confidence: CONFIRMED | LIKELY | POSSIBLE
   - implication: [what this means for the topic from your perspective]

2. ...

### Gaps
- [What you could NOT find or verify — honest about the limits of your research]

### Initial Stance
Based on your research, state your preliminary position in 2-3 sentences. This will be your starting point for the debate.",
  summary: "Research spike for [role]"
})
```

Send to all panelists in parallel.

### 2c. Review Research

As spike reports arrive, validate them:

- [ ] Contains at least one finding with a confidence tag
- [ ] Contains at least one file:line or source citation
- [ ] Gaps section is present and honest
- [ ] Initial stance is concrete, not hedged

**If a report fails validation:** Push back with a DM: "Your research report is missing [specific check]. Revise and resend. I need concrete findings with citations before we can proceed."

### 2d. Research Checkpoint

After all spike reports arrive and pass validation, present a **research summary** to the user:

```
AskUserQuestion({
  questions: [{
    question: "Research complete. [Role 1] found [N] items (key finding: [most important]). [Role 2] found [M] items (key finding: [most important]). Notable gaps: [what neither could verify]. Ready to proceed to debate, or do you want to adjust research scope?",
    header: "Research",
    options: [
      { label: "Proceed", description: "Research is sufficient. Begin the deliberation with these findings as evidence base." },
      { label: "Deeper research", description: "Send agents back to investigate specific areas more thoroughly." },
      { label: "Add your findings", description: "You have research or context to add before debate begins." },
      { label: "Adjust scope", description: "The research revealed the topic should be reframed." }
    ],
    multiSelect: false
  }]
})
```

**Proceed →** Move to Phase 3.

**Deeper research →** Ask the user what to investigate further. Send targeted follow-up spikes to the relevant panelist(s). When results arrive, return to 2d.

**Add your findings →** Collect the user's input. Record it in the deliberation log as user-contributed evidence. Relay to each panelist independently: "Additional context for your research: [finding]. Factor this into your analysis." Do not reveal what the other panelist found. Return to 2d.

**Adjust scope →** Collect the user's reframing. Update the topic and send each panelist the revised scope independently with a request to re-examine their findings. Do not share one panelist's findings with the other. When they respond, return to 2d.

---

## Phase 3: Opening Positions

Now that research is complete, ask each panelist to state their opening position **grounded in their findings**:

```
SendMessage({
  type: "message",
  recipient: "<role-name>",
  content: "Research phase is complete. Based on your findings, state your opening position on: [topic restated as a specific question].

Requirements:
- 2-4 paragraphs, concrete and specific
- Cite at least 2 findings from your research as supporting evidence
- End with a clear position statement: 'My position is: [X]'
- If your research revealed something that challenges your role's typical perspective, say so honestly",
  summary: "Request opening position from [role]"
})
```

Send to all panelists in parallel.

---

## Phase 4: Deliberation Cycle

This is the core loop. It runs for every substantive position a panelist takes. Positions must now be **evidence-backed** — the facilitator should push back on any position that doesn't cite research findings.

### Step 1: Extract Position

When a panelist responds, extract:
- **Their position**: What they're arguing for (1-2 sentences)
- **Evidence cited**: Which research findings support it
- **Evidence quality**: Are the cited findings CONFIRMED or just POSSIBLE?
- **Tension**: Does this conflict with another panelist's position or with a previously accepted position?

### Step 2: User Checkpoint

Surface the position to the user for validation. Use `AskUserQuestion` every time a panelist settles on a position:

```
AskUserQuestion({
  questions: [{
    question: "[Role] proposes: [position summary]. Evidence: [1-2 key findings cited, with confidence levels]. [If tension exists: 'This conflicts with [other position], which cites [counter-evidence].' ] Do you agree with this position?",
    header: "Checkpoint",
    options: [
      { label: "Accept", description: "Lock this as an accepted position. Agents will build on it." },
      { label: "Redirect", description: "You disagree or want a different direction. You'll provide guidance." },
      { label: "Add context", description: "You have additional information that should inform this position." },
      { label: "Challenge evidence", description: "You question the research behind this position." },
      { label: "Skip", description: "No opinion — let the agents continue debating this point." }
    ],
    multiSelect: false
  }]
})
```

### Step 3: Process User Response

**Accept →** Record in the deliberation log as an ACCEPTED position with its supporting evidence. Notify each panelist independently as a **ground-truth constraint** — do not reveal who proposed it or what evidence supported it:

```
SendMessage({
  type: "message",
  recipient: "<role-name>",
  content: "Ground-truth constraint: The following has been accepted as settled: [position, stated neutrally without attribution]. Build on this for the remaining open questions. What follows from your perspective?",
  summary: "New constraint accepted"
})
```

Send to each panelist independently. The constraint framing ensures agents incorporate the decision without knowing whose position won.

**Redirect →** Ask the user for their preferred direction. Then relay **only to the panelist whose position is being redirected** — the other panelist is not informed:

```
SendMessage({
  type: "message",
  recipient: "<role-name>",
  content: "The user provides this direction: [user's input]. Revise your position taking this into account. Be concrete about what changes and cite evidence.",
  summary: "User redirect on [topic]"
})
```

When the panelist responds with a revised position, return to Step 2.

**Add context →** Collect the user's additional context. Relay to each panelist independently — same message, no cross-contamination:

```
SendMessage({
  type: "message",
  recipient: "<role-name>",
  content: "Additional context: [context]. Does this change your position? If so, state your revised position explicitly with supporting evidence.",
  summary: "User adds context"
})
```

When panelists respond, return to Step 2 for each revised position.

**Challenge evidence →** Collect the user's challenge. Send a targeted follow-up **only to the panelist whose evidence is challenged** — the other panelist is not informed:

```
SendMessage({
  type: "message",
  recipient: "<role-name>",
  content: "Your evidence is being questioned: [user's challenge]. Verify your finding by [reading the relevant code again / checking additional sources]. Report back with either a confirmed finding or a correction.",
  summary: "Evidence challenge"
})
```

When the panelist responds, present the verification result at the next checkpoint.

**Skip →** The user has no opinion on this position. Ask the **same question** to the other panelist independently — do not share what the first panelist concluded. The other panelist forms their own position on the same sub-question from their own research:

```
SendMessage({
  type: "message",
  recipient: "<other-role-name>",
  content: "[Restate the sub-question]. Based on your research, what is your position? Be concrete and cite your findings.",
  summary: "Independent position request"
})
```

When the second panelist responds, present **both positions side-by-side** to the user at the next checkpoint (Step 2) so the user can compare and decide.

### Step 4: Thread Progression

After processing a checkpoint:
- If there are unresolved sub-questions or new tensions, continue the cycle (return to Step 1).
- If the current thread is exhausted, present a synthesis and ask:

```
AskUserQuestion({
  questions: [{
    question: "Thread complete. [Summary of what's settled, with evidence basis]. What next?",
    header: "Direction",
    options: [
      { label: "New question", description: "Pose a new question to the panel." },
      { label: "Dig deeper", description: "Explore an unresolved tension further." },
      { label: "More research", description: "Send agents to investigate something the debate revealed." },
      { label: "Conclude", description: "Wrap up and produce the deliberation document." }
    ],
    multiSelect: false
  }]
})
```

- *New question* → User provides (or facilitator proposes) the next question. Send to panelists via DM. Return to Step 1.
- *Dig deeper* → Identify the most productive unresolved tension and formulate a targeted question. Send via DM. Return to Step 1.
- *More research* → Define a targeted follow-up spike. Send to the relevant panelist(s) as a mini research assignment (same format as Phase 2b but narrower). When findings arrive, present them at a checkpoint, then return to Step 1.
- *Conclude* → Proceed to Phase 5.

### Keeping It Productive

- If a panelist makes a claim without citing evidence: "Cite your research. What specific finding supports this?"
- If a panelist gives a generic response, push back: "Respond NOW with your concrete analysis grounded in your research findings."
- If a panelist's position is vague, push back: "Be more specific. What exactly would you do, why, and what evidence supports it?"
- If a panelist starts speculating about what "others might think" or hedging to cover all bases, redirect: "Stay in your lane. What does YOUR research tell you? What does YOUR perspective prioritize?"
- **Depth limit**: After 3 rounds with the same panelist on the same sub-question without a clear position shift, surface the current state to the user for a decision.

---

## Phase 5: Document and Close

### 5a. Write the Deliberation Record

Compile the full deliberation log into a markdown file. Write it to the working directory:

```
Write({
  file_path: "<working-directory>/workshop-<topic-slug>-<YYYY-MM-DD>.md",
  content: <deliberation document>
})
```

**Document structure:**

```markdown
# Workshop: [Topic]

**Date**: [YYYY-MM-DD]
**Panelists**: [Role 1] (perspective), [Role 2] (perspective)

## Research Findings

Evidence gathered during the research phase, organized by panelist.

### [Role 1] Findings

1. **[Finding title]** — [Detail]
   - source: [file:line or URL]
   - confidence: CONFIRMED | LIKELY | POSSIBLE
   - implication: [What this means]

2. ...

### [Role 2] Findings

1. ...

### User-Contributed Evidence

1. [Any findings or context the user added during research or deliberation]

### Research Gaps

- [What could not be verified by either panelist]

## Accepted Positions

Positions validated by the user during the workshop, with their evidence basis.

1. **[Position title]** — [Position statement]
   - *Evidence*: [Specific findings that support this, with confidence levels]
   - *Reasoning*: [Key arguments that led here]
   - *User input*: [Any context or redirection the user provided]
   - *Round accepted*: [N]

2. ...

## Open Questions

Positions that were debated but not resolved.

1. **[Question]**
   - *Position A*: [Summary] — supported by [evidence]
   - *Position B*: [Summary] — supported by [evidence]
   - *Status*: Tabled / Needs more research / Deferred
   - *What would resolve it*: [Specific investigation or decision needed]

## User Contributions

Context, redirections, evidence challenges, and decisions the user injected.

1. [Round N]: [What the user said and how it affected the discussion]
2. ...

## Discussion Log

### Research Phase

**[Role 1]**: [Summary of findings — N items, key discovery]
**[Role 2]**: [Summary of findings — N items, key discovery]
**User checkpoint**: [Proceed/Deeper/Add/Adjust] — [detail]

### Round 1: [Sub-question or thread topic]

**[Role 1]**: [Summary of position + evidence cited]
**[Role 2]**: [Summary of position + evidence cited]
**User checkpoint**: [Accept/Redirect/Add context/Challenge/Skip] — [detail]
**Outcome**: [What was decided]

### Round 2: ...

## Action Items

Concrete next steps that emerged from accepted positions.

- [ ] [Action item — must pass the sharpening gate: name the artifact, state what changes, make it assignable]
- [ ] ...

## Key Insight

[The single most valuable thing that emerged from the workshop — what did the research reveal that pure discussion would have missed?]
```

### 5b. Shutdown

Send shutdown requests to all panelists:

```
SendMessage({
  type: "shutdown_request",
  recipient: "<role-name>",
  content: "Workshop concluded, shutting down"
})
```

After all panelists confirm, delete the team:

```
TeamDelete()
```

### 5c. Report

Tell the user:
- Where the deliberation document was written
- Count of research findings, accepted positions, and open questions
- Offer to create beads from action items if any emerged

### 5d. Optional: Create Beads

If sharpened action items emerged, offer to create beads:

```bash
bd create --title="[action item]" --type=task --priority=[0-4] \
  --description="From workshop on [topic]. Evidence: [key findings]. Context: [relevant accepted position]"
```

---

## Guidelines

1. **Agents are completely independent.** Panelists never hear each other's research findings, positions, or reasoning. The facilitator mediates all information flow. Cross-pollination happens only through the user at checkpoints and through ground-truth constraints (accepted positions stated without attribution). This prevents anchoring, groupthink, and authority bias — each agent's position reflects only their role, their research, and their reasoning.
2. **Research before rhetoric.** Agents investigate before they argue. Positions without evidence get pushed back.
3. **User is a participant, not just a steering wheel.** Every settled position gets validated. The user's context and corrections are first-class inputs.
4. **Accepted positions become anonymous constraints.** Once the user accepts a position, it's relayed to agents as a ground-truth constraint without attribution. Agents build on it without knowing who proposed it.
5. **Evidence quality matters.** CONFIRMED findings outweigh LIKELY findings. POSSIBLE findings should trigger follow-up research, not settled positions.
6. **Creative tension emerges from independence.** Because agents can't see each other, they naturally arrive at different positions grounded in different evidence. The user sees the full picture; the agents see only their slice.
7. **Short turns.** Panelist responses should be 2-4 paragraphs. This is dialogue with the facilitator, not a debate between agents.
8. **2 panelists by default.** The user can request more mid-workshop.
9. **DMs only, never broadcasts.** Each agent is addressed individually. Never use broadcast — it implies a shared channel.
10. **The document is the deliverable.** Write it with enough context that someone reading it later understands what was found, what was decided, and why.
11. **Don't over-checkpoint.** Only checkpoint when there's a concrete position to validate, not when agents are still exploring or researching.
12. **Research can resume mid-debate.** If deliberation reveals an evidence gap, the user can send agents back to investigate via the "More research" option. Each agent investigates independently.

## See also

- `/meeting` — lighter-weight version without research phase or per-position checkpoints
- `/blossom` — pure exploration without deliberation; use when the goal is discovery, not decision
- `/consensus` — independent parallel proposals; agents don't research or debate
- `/spec` — formalize accepted positions from a workshop into a structured specification
- `/diff-ideas` — structured comparison of exactly two approaches; simpler than a full workshop
- `/gather` — if you only need research without deliberation, use gather directly
