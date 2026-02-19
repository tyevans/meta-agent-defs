# Workflow Pipelines

End-to-end lifecycle workflows for common scenarios. Each pipeline documents the skill sequence, what state passes between steps, when to use it, and any prerequisites.

For primitive-only composition chains (gather -> distill -> rank, etc.), see [Primitive Chain Patterns](INDEX.md#primitive-chain-patterns) in the navigator and [Primitives Recipes](primitives-recipes.md) for annotated examples.

---

## 1. Discovery to Execution

**When to use**: You have a vague goal or unknown solution space and need to go from "explore the problem" to "shipped work" within a session or sprint. Use when you don't yet have enough information to plan.

**Prerequisites**: An active backlog (`bd stats` should return a valid state). No existing epic for the topic.

### Steps

1. `/blossom <topic>` — Runs subagent spikes to explore the solution space. Produces an epic with child tasks written to the backlog via `bd create`. Each task includes a confidence level (CONFIRMED/LIKELY/POSSIBLE).
2. `/sprint` — Reads the backlog (`bd ready`), selects the highest-priority tasks from the blossom epic, dispatches subagents to implement them. Produces learnings and a session summary.
3. `/retro` — Reads session learnings and git activity. Produces a retrospective written to `memory/agents/<name>/learnings.md` and a summary for the team.

### State Flow

| From | To | Via |
|------|----|-----|
| `/blossom` | `/sprint` | `bd` backlog — blossom writes tasks; sprint reads `bd ready` |
| `/sprint` | `/retro` | In-session context — sprint surfaces learnings inline; retro reads them |
| `/retro` | next session | `memory/agents/<name>/learnings.md` — persistent file written by retro |

---

## 2. Team Lifecycle

**When to use**: You have a persistent learning team and need to run a full operational cycle: onboard or sync the team, do the work, reflect, and hand off cleanly. Use when coordinating multiple agents across sessions.

**Prerequisites**: A `team.yaml` exists (see `/assemble`). Team members have learnings files under `memory/agents/`.

### Steps

1. `/assemble` — Creates or re-onboards a team from a `team.yaml` template. Assigns roles, surfaces each agent's current learnings, and establishes ownership of backlog areas.
2. `/standup` — Syncs team status: reads each agent's `learnings.md`, surfaces blockers, reports backlog health. Produces a status summary used to focus the sprint.
3. `/sprint` — Dispatches work to team members based on ownership and backlog priority. Runs the learning loop: agents record what they learned in `memory/agents/<name>/learnings.md`.
4. `/retro` — Retrospective across the full team: what worked, what broke, what to change. Updates `memory/team/retro-history.md` and individual agent learnings.
5. `/handoff` — Captures end-of-session state to `memory/sessions/last.md` so the next session can resume cleanly.

### State Flow

| From | To | Via |
|------|----|-----|
| `/assemble` | `/standup` | In-memory team roster + `memory/agents/<name>/learnings.md` |
| `/standup` | `/sprint` | Standup summary in context — surfaces blockers and priority signals |
| `/sprint` | `/retro` | `memory/agents/<name>/learnings.md` — written during sprint |
| `/retro` | `/handoff` | Context — retro produces the session narrative handoff reads |
| `/handoff` | next session `/status` | `memory/sessions/last.md` — last session state file |

---

## 3. Deep Analysis

**When to use**: You need to go from raw information gathering through a full research-to-specification cycle. Use when a problem is too complex to plan without first understanding the space deeply.

**Prerequisites**: A researchable question or codebase area. Sufficient context window — this is the longest single-session pipeline.

### Steps

1. `/gather <topic>` — Collects findings with sources and confidence levels in pipe-format output. Cast a wide net here.
2. `/distill` — Reads gather output from context. Condenses to the essential findings. Reduces noise before ranking.
3. `/rank <criteria>` — Reads distill output from context. Scores and orders items by the stated criteria (risk, effort, impact, etc.).
4. `/spec` — Reads ranked output from context. Produces a progressive specification document: requirements, constraints, open questions, acceptance criteria. Written to a file.
5. `/tracer` — Reads the spec. Implements a thin end-to-end slice to validate the specification against reality before full implementation.

### State Flow

| From | To | Via |
|------|----|-----|
| `/gather` | `/distill` | Pipe-format output in context |
| `/distill` | `/rank` | Pipe-format output in context |
| `/rank` | `/spec` | Pipe-format output in context (spec reads the ranked list) |
| `/spec` | `/tracer` | Spec file on disk — tracer reads the written spec document |

**Compression note**: If the session is long enough to risk context compression between `/rank` and `/spec`, save the rank output to a file first. See [File-Based Intermediate Results](INDEX.md#file-based-intermediate-results).

---

## 4. Collaborative Planning

**When to use**: You need to align a team or multiple perspectives on an approach before executing. Use when the "what to build" question is contested or unclear, or when you want dissenting views surfaced before committing to a plan.

**Prerequisites**: A clear goal statement. Optionally a persistent team assembled via `/assemble`.

### Steps

1. `/meeting <goal>` — Runs an interactive multi-agent dialogue around the goal. Each agent brings a perspective. Produces a discussion transcript and emerging consensus or key disagreements.
2. `/decompose` — Reads meeting output from context (or takes the agreed goal as input). Breaks the goal into bounded sub-parts with clear interfaces.
3. `/plan` — Reads decompose output from context. Produces a dependency-aware execution sequence: what to build in what order, with which skills or agents.
4. `/sprint` — Reads the plan and creates or populates backlog tasks via `bd create`. Dispatches agents to execute the plan.

### State Flow

| From | To | Via |
|------|----|-----|
| `/meeting` | `/decompose` | Context — decompose reads the meeting summary and agreed framing |
| `/decompose` | `/plan` | Pipe-format output in context |
| `/plan` | `/sprint` | Context — sprint reads the plan and translates to `bd` tasks |

---

## 5. Recursive Exploration

**When to use**: You have a goal but the exploration path itself is unknown — you need to discover what to explore as you go. Use when `/blossom` alone is too broad and you need goal-directed recursion that narrows at each level.

**Prerequisites**: A clear goal statement (not just a topic). `/fractal` works best with a concrete question like "what prevents us from shipping X?" rather than "explore X."

### Steps

1. `/fractal <goal>` — Recursively breaks down the goal into sub-questions, exploring each leaf. Stays inline (no forked context). Produces a structured map of findings organized by depth.
2. `/blossom <focused area>` — Takes the most uncertain or highest-value area surfaced by fractal and runs spike subagents against it. Produces tasks for the specific area.
3. `/sprint` — Dispatches the blossom tasks. Agents implement the spike findings.

### State Flow

| From | To | Via |
|------|----|-----|
| `/fractal` | `/blossom` | Context — you pass the specific area fractal flagged as highest uncertainty |
| `/blossom` | `/sprint` | `bd` backlog — blossom writes tasks; sprint reads `bd ready` |

**When to skip `/blossom`**: If `/fractal` produces sufficient task-level clarity, go directly to `/sprint` and skip `/blossom`. Blossom is only needed when fractal surfaces an area requiring dedicated spike subagents.

---

## 6. Session Continuity

**When to use**: Every session. This is the standard session lifecycle — how you start work, do work, reflect, and ensure the next session can resume without re-orientation overhead.

**Prerequisites**: `memory/sessions/last.md` exists after the first session (subsequent sessions only).

### Steps

1. `/status` — Reads `memory/sessions/last.md`, `bd stats`, git activity, and team state. Produces a unified dashboard: what happened last session, what's ready now, what's blocked.
2. **Work** — Use any skills appropriate to the tasks at hand. The middle of the session is unconstrained.
3. `/retro` — Reflects on the session: what was completed, what was learned, what to change. Writes to `memory/agents/<name>/learnings.md` and `memory/team/retro-history.md`.
4. `/handoff` — Captures current state to `memory/sessions/last.md`. Includes backlog state, open threads, and a one-paragraph summary for the next session's `/status` to read.
5. **(Next session) `/status`** — Reads the handoff file. The cycle repeats.

### State Flow

| From | To | Via |
|------|----|-----|
| previous `/handoff` | `/status` | `memory/sessions/last.md` |
| work | `/retro` | In-session context — git activity, task completions, inline learnings |
| `/retro` | `/handoff` | Context — retro narrative is the input to handoff |
| `/handoff` | next `/status` | `memory/sessions/last.md` |

**The key invariant**: Every session ends with `/retro` + `/handoff`. Sessions that skip handoff break the continuity chain and force the next session to re-derive context from git history alone.
