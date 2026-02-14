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

## Phase 2: Facilitate

This is the interactive core. The user drives the conversation.

### Facilitation Protocol

After each round of panelist responses:

1. **Summarize** the key points and tensions to the user (2-3 sentences)
2. **Ask the user** what to explore next. Offer options:
   - Follow up on a specific point
   - Ask the panel a new question
   - Have two panelists debate a point of disagreement
   - Move to a different aspect of the topic
   - Wrap up and synthesize

### Routing Messages

- **Prefer direct messages over broadcast.** DMs reliably produce substantive responses; broadcasts often get empty acknowledgments.
- **To all panelists**: Send individual `message` calls to each panelist in parallel. Only use `broadcast` for simple administrative messages (not for questions that need substantive answers).
- **To specific panelists**: Use `message` to `<role-name>` for targeted follow-ups
- **Relaying cross-panelist context**: When forwarding one panelist's point to another, include the substance: "[Panelist A] argued: [key point]. React from your perspective."
- **Debates**: Send both panelists the same prompt: "Debate: [point of disagreement]. [Panelist A] argues X. [Panelist B] argues Y. Make your best case."

### Keeping It Productive

- If a panelist gives a generic or empty response ("ready", "standing by"), push back with a direct message restating the question and demanding substance: "Respond NOW with your concrete analysis. Do not acknowledge — respond."
- If a panelist gives a vague response, push back: "Be more specific. What exactly would you change/build/avoid?"
- If the discussion goes circular, summarize what's settled and redirect to what's unresolved
- If a panelist's perspective hasn't been heard, explicitly invite them: "Skeptic, what concerns do you have about this direction?"

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
