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
> Your perspective: [1-2 sentences describing what this role cares about and how they think]
>
> **IMPORTANT: When you receive a message, respond IMMEDIATELY with your substantive perspective.** Do NOT say "ready", "standing by", or "waiting for the opening question." Every message you receive IS the prompt — respond to it with 2-4 paragraphs of concrete analysis from your role's perspective.
>
> **CRITICAL: You MUST use the SendMessage tool to communicate.** Your plain text output is NOT visible to anyone. Every response you give must be sent via `SendMessage({ type: "message", recipient: "team-lead", content: "...", summary: "..." })`. Always send to **team-lead** (the facilitator) — never send directly to other panelists. If you do not call SendMessage, nobody will see what you said.
>
> You are in a live discussion. When the facilitator sends you a message:
> 1. Read the message carefully, then respond with substance using SendMessage to "team-lead".
> 2. Respond from your role's perspective. Be direct and specific.
> 3. If you disagree with another panelist's position (relayed by the facilitator), say so and explain why.
> 4. If you have a question that would sharpen the discussion, ask it.
> 5. Keep responses to 2-4 paragraphs. This is a conversation, not a monograph.
>
> Before starting, check for prior learnings: if a file-based learnings file exists at `memory/agents/<role-name>/learnings.md`, read it for accumulated context.

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
- **Tension exists →** Summarize the tension in 1-2 sentences. Send it via direct message to the panelist(s) whose position is most directly challenged by the tension, tagged `[Exchange N of LIMIT]`. Increment exchange counter. When their responses arrive, return to Step 1.
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

If action items emerged, offer to create beads:

```bash
bd create --title="[action item]" --type=task --priority=[0-4] \
  --description="From meeting on [topic]. Context: [relevant discussion point]"
```

---

## Guidelines

1. **User drives the agenda.** The facilitator presents options and the user chooses direction. Never auto-advance without user input.
2. **Creative tension is the goal.** A panel that agrees on everything is a bad panel. Pick roles that naturally challenge each other.
3. **Short turns.** Panelist responses should be 2-4 paragraphs, not essays. This is dialogue, not reporting.
4. **Summarize often.** After each round, distill what was said so the user can steer.
5. **2 panelists by default.** Pick the two most opposed perspectives for the topic. The user can request more mid-meeting — the facilitator spawns additional agents as needed. More than 4 creates noise.
6. **DMs over broadcasts.** Direct messages reliably produce substantive engagement. Broadcasts often produce empty acknowledgments. Use DMs for all questions that need real answers.
7. **Meetings are cheap.** If the first panel doesn't have the right perspectives, close it and start a new one with different roles.
