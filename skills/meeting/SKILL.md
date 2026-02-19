---
name: meeting
description: "Form an agent team and have an interactive group discussion to flesh out requirements, explore ideas, or get diverse perspectives on a topic. Use when you want to brainstorm with multiple viewpoints, clarify requirements through dialogue, or pressure-test an idea before committing. Keywords: discuss, brainstorm, requirements, team, dialogue, perspectives, workshop."
argument-hint: "<topic or question>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Task, SendMessage, TeamCreate, TeamDelete, AskUserQuestion
---

# Meeting: Interactive Multi-Agent Dialogue

You are facilitating a **Meeting** -- an interactive group discussion with a panel of agents. Unlike fan-out workflows (consensus, premortem) where agents work independently and return reports, a meeting is a live conversation where the user drives the agenda and agents respond to each other.

**Topic:** $ARGUMENTS

## When to Use

- When you want to brainstorm with multiple perspectives on a design problem
- When clarifying requirements through dialogue before committing to an approach
- When pressure-testing an idea by having different roles challenge it
- When exploring trade-offs through interactive discussion rather than parallel reports
- When the user wants to steer the conversation and ask follow-ups in real time

## Phase 1: Assemble the Panel

### 1a. Choose Roles

Based on the topic, select **2 roles** that provide the most opposed perspectives. Two panelists with genuine tension produce better dialogue than 3-4 with diluted positions. The user can request additional panelists mid-meeting if needed.

**Role templates** (pick or adapt):

| Role | Perspective | Good for |
|------|------------|----------|
| Architect | System design, patterns, tradeoffs | Technical design discussions |
| Skeptic | Risk, edge cases, what could go wrong | Pressure-testing ideas |
| User Advocate | UX, developer experience, simplicity | Feature design |
| Domain Expert | Deep knowledge of the specific area | Domain-specific questions |
| Pragmatist | What's achievable, incremental path | Scoping and prioritization |
| Historian | Precedent, what's been tried before | Avoiding past mistakes |
| Innovator | Novel approaches, challenge assumptions | Breaking out of ruts |

Select the two roles whose perspectives would produce the most productive disagreement on this specific topic. Never select two roles that optimize for the same underlying value.

### 1b. Create the Team

```
TeamCreate({ team_name: "meeting-<short-topic-slug>" })
```

### 1c. Spawn Panelists

For each role, spawn a teammate:

```
Task({
  team_name: "meeting-<slug>",
  name: "<role-name>",
  subagent_type: "general-purpose",
  run_in_background: true,
  prompt: "<panelist prompt -- see below>"
})
```

**Panelist prompt template:**

> You are the **[Role]** in a meeting about: [topic]
>
> Your perspective: [2-3 sentences describing what this role cares about and how they think — be rich and specific with genuine characterization]
>
> **CRITICAL: You MUST use the SendMessage tool to communicate.** Your plain text output is NOT visible to anyone. Every response you give must be sent via `SendMessage({ type: "message", recipient: "team-lead", content: "...", summary: "..." })`. Always send to **team-lead** (the facilitator) — never send directly to other panelists. If you do not call SendMessage, nobody will see what you said.
>
> Always respond from your role's specific perspective, never as a generic assistant. When the facilitator sends you a message, respond with 2-4 paragraphs of concrete, specific analysis. If you disagree with another panelist's position, say so and explain why.

### 1d. Opening Round

Once all panelists are spawned, send the opening question as **direct messages** to each panelist (not broadcast). Direct messages reliably trigger substantive engagement; broadcasts often produce empty acknowledgments.

```
SendMessage({
  type: "message",
  recipient: "<role-name>",
  content: "[Role], here is the topic: [topic]. [Restate as a specific question]. Share your initial perspective — 2-4 paragraphs, be concrete and specific.",
  summary: "Opening question to [role]"
})
```

Send to all panelists in parallel (multiple SendMessage calls in one response).

---

## Phase 2: Facilitation Cycle

After Phase 1 opening responses arrive, run this cycle for every panelist response. Maintain an **exchange counter** (starts at 0, resets at each checkpoint). The depth limit defaults to 3.

**Step 1. Read and extract.** What is this panelist's key claim? Does it introduce tension — a disagreement, counterargument, or novel angle — that another panelist has not yet addressed?

**Step 2. Relay or skip.**
- **Tension exists →** Summarize the tension in 1-2 sentences. When relaying to a panelist, address them by their role name but **anonymize the source of opposing arguments** — describe the analytical frame, not the role that produced it (e.g., "Skeptic, from a systems-design perspective, the argument is that X..." rather than "Skeptic, the Architect argues X..."). This strips identity persistence and authority signals while preserving the analytical context that makes engagement meaningful. Never frame convergence as "Role A and Role B agree" — instead describe converging reasoning paths. Preserve the core argument faithfully before posing the next question. If a panelist makes a specific factual claim central to the disagreement, briefly verify it using available tools before relaying. Send it via direct message to the panelist(s) whose position is most directly challenged by the tension, tagged `[Exchange N of LIMIT]`. Increment exchange counter. When their responses arrive, return to Step 1.
- **No tension →** Proceed to Step 3.

If the exchange counter has reached the depth limit, skip relay regardless and proceed to Step 3.

**Step 3. Checkpoint with user.** Synthesize the thread in 2-3 sentences: what was discussed, where panelists agreed, what tension remains. Then ask:

> **Continue** this thread, **pivot** to a new question, or **conclude** the meeting?

- *Continue* → Reset exchange counter. Formulate a follow-up question informed by the discussion so far. Send to each panelist via direct message (not broadcast). When responses arrive, return to Step 1.
- *Pivot* → Reset exchange counter. Ask the user for the new topic (or propose one from unresolved tensions). Send to each panelist via direct message (not broadcast). When responses arrive, return to Step 1.
- *Conclude* → Proceed to Phase 3.
- *Freeform direction* → If the user gives a specific direction that doesn't match these options (e.g., "Ask the Architect about X"), treat it as a targeted follow-up. Send the user's question to the relevant panelist(s) via direct message, then return to Step 1.

### Keeping It Productive

- If a panelist gives a generic or empty response ("ready", "standing by"), push back with a direct message restating the question and demanding substance: "Respond NOW with your concrete analysis. Do not acknowledge — respond."
- If a panelist gives a vague response, push back: "Be more specific. What exactly would you change/build/avoid?"
- If the discussion goes circular, summarize what's settled and redirect to what's unresolved

---

## Phase 3: Synthesize and Close

When the user says to wrap up (or after 4-5 rounds if the conversation naturally winds down):

### 3a. Meeting Summary

Ask one panelist to draft a summary, or write it yourself:

```markdown
## Meeting Summary: [topic]

### Consensus Points
[What the panel agreed on]

### Open Tensions
[Disagreements that weren't resolved]

### Decisions Made
[Any concrete decisions the user expressed during the meeting]

### Action Items
[Concrete next steps, if any emerged]

**Sharpening gate** — every action item must pass three tests:

1. **Name the specific code/file/workflow** where the problem/opportunity exists
2. **State what concretely should change** (a function to add, a check to insert, a pattern to adopt)
3. **Make it assignable** — could an agent implement this in one session without design decisions?

Example:
- ❌ "Investigate the auth refactor further"
- ✅ "Spike JWT vs session cookies in src/auth/provider.ts, produce decision doc with latency + security tradeoffs"

Drop items that can't be sharpened, or convert to investigation beads with explicit research questions.

### Key Insight
[The single most valuable thing that emerged from the discussion]
```

### 3b. Shutdown

Send shutdown requests to all panelists:

```
SendMessage({
  type: "shutdown_request",
  recipient: "<role-name>",
  content: "Meeting concluded, shutting down"
})
```

After all panelists confirm, delete the team:

```
TeamDelete()
```

### 3c. Optional: Create Beads

If sharpened action items emerged, offer to create beads:

```bash
bd create --title="[action item]" --type=task --priority=[0-4] \
  --description="From meeting on [topic]. Context: [relevant discussion point]"
```

Use the sharpened form as the title. The 3-test pattern ensures beads are immediately dispatchable.

---

## Guidelines

1. **User drives the agenda.** The facilitator presents options and the user chooses direction. Never auto-advance without user input.
2. **Creative tension is the goal.** A panel that agrees on everything is a bad panel. Pick roles that naturally challenge each other.
3. **Short turns.** Panelist responses should be 2-4 paragraphs, not essays. This is dialogue, not reporting.
4. **Summarize often.** After each round, distill what was said so the user can steer.
5. **2 panelists by default.** Pick the two most opposed perspectives for the topic. The user can request more mid-meeting — the facilitator spawns additional agents as needed. More than 4 creates noise.
6. **DMs over broadcasts.** Direct messages reliably produce substantive engagement. Broadcasts often produce empty acknowledgments. Use DMs for all questions that need real answers.
7. **Meetings are cheap.** If the first panel doesn't have the right perspectives, close it and start a new one with different roles.
8. **Anonymize sources, preserve lenses.** When relaying arguments between panelists, describe the analytical frame ("from a structural-analysis perspective") rather than the source role ("the Architect argues"). This prevents authority bias and identity persistence (halo effects, commitment pressure across exchanges) while preserving the epistemic context that makes engagement meaningful. The facilitator maintains internal attribution for coverage tracking and thread coherence.

## See also

- `/rank` — rank options or action items produced by the meeting when prioritization is needed
- `/diff-ideas` — structured alternative when comparing exactly two approaches; avoids the full panel setup
- `/consensus` — similar multi-agent pattern but agents work independently and return written positions rather than conversing
- `/spec` — formalize decisions reached in a meeting into a structured specification document
