---
name: meeting
description: "Form an agent team and have an interactive group discussion to flesh out requirements, explore ideas, or get diverse perspectives on a topic. Use when you want to brainstorm with multiple viewpoints, clarify requirements through dialogue, or pressure-test an idea before committing. Keywords: discuss, brainstorm, requirements, team, dialogue, perspectives, workshop."
argument-hint: "<topic or question>"
disable-model-invocation: true
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

Based on the topic, select 3-4 roles that provide diverse, complementary perspectives. Good panels have creative tension -- roles that naturally challenge each other.

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
> You are in a live discussion. When the facilitator or another panelist sends you a message:
> 1. Respond from your role's perspective. Be direct and specific.
> 2. If you disagree with another panelist, say so and explain why.
> 3. If you have a question that would sharpen the discussion, ask it.
> 4. Keep responses to 2-4 paragraphs. This is a conversation, not a monograph.
>
> Before starting, check for prior learnings: if a file-based learnings file exists at `memory/agents/<role-name>/learnings.md`, read it for accumulated context.

### 1d. Opening Round

Once all panelists are spawned, broadcast the opening question:

```
SendMessage({
  type: "broadcast",
  content: "Welcome to this meeting on: [topic]. Opening question: [restate the topic as a specific question]. Please share your initial perspective.",
  summary: "Meeting opening: [topic]"
})
```

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

- **To all panelists**: Use `broadcast` for questions the whole panel should answer
- **To specific panelists**: Use `message` to `<role-name>` for targeted follow-ups
- **Debates**: Send both panelists the same prompt: "Debate: [point of disagreement]. [Panelist A] argues X. [Panelist B] argues Y. Make your best case."

### Keeping It Productive

- If a panelist gives a generic response, push back: "Be more specific. What exactly would you change/build/avoid?"
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
5. **3-4 panelists max.** More creates noise. Fewer lacks diversity.
6. **Meetings are cheap.** If the first panel doesn't have the right perspectives, close it and start a new one with different roles.
