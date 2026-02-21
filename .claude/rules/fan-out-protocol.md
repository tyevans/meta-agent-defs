---
paths:
  - "skills/**/*.md"
strength: should
freshness: 2026-02-21
---

# Fan-Out/Fan-In Protocol

Standard pattern for dispatching multiple agents and collecting their results. Used by blossom, fractal, spec, consensus, premortem, meeting, and standup.

## The Pattern

```
1. Frame: Define what each agent investigates (roles, areas, perspectives)
2. Dispatch: Launch agents via Task(run_in_background=true)
3. Collect: Retrieve results as agents complete
4. Synthesize: Combine results into a unified output
```

## Dispatch

Launch agents concurrently using background Task agents:

```
Task({
  subagent_type: "<type>",
  run_in_background: true,
  model: "<model>",
  prompt: "<agent instructions>"
})
```

- **subagent_type**: "Explore" for read-only investigation, "general-purpose" for tasks needing writes
- **model**: "haiku" for lightweight tasks (standup), "sonnet" for standard work, "opus" for complex analysis
- **Concurrency**: Launch up to 4 agents at once. More risks API throttling.

## Agent Instructions

Every dispatched agent prompt MUST include:

1. **Role/area**: What the agent is responsible for
2. **Goal context**: Why this investigation matters (from the parent skill's goal)
3. **Report format**: Exact structure for the agent's output
4. **Constraints**: Word limits, scope boundaries, what NOT to do

Keep agent prompts self-contained. Agents cannot read the skill file or access the parent's context -- everything they need must be in the prompt.

## Collecting Results

Results come back via TaskOutput or (for teams) SendMessage. Process each result as it arrives rather than waiting for all to complete -- this allows early termination.

## Key Constraints

- **Subagents cannot invoke skills.** The Skill tool is not available to subagents. All workflow logic must be embedded in the agent's prompt.
- **Subagents cannot see each other.** Unless using Teams, dispatched agents are isolated. They cannot read each other's results.
- **Teams enable coordination.** When agents need to respond to each other (meeting, blossom with teams), use TeamCreate + SendMessage. For independent parallel work, background Task agents are simpler.

## When to Use Teams vs Background Agents

| Scenario | Use |
|----------|-----|
| Agents work independently, results collected by orchestrator | Background Task agents |
| Agents need to respond to each other | Teams + SendMessage |
| Agents need to be reused for multiple tasks | Teams (reuse via SendMessage) |
| Simple parallel dispatch, <5 agents | Background Task agents |
| Complex coordination, 6+ agents | Teams |

## Agent Preamble

Standard investigation instructions for dispatched agents. Include these in agent prompts when applicable:

**Investigation protocol:**

1. Read the actual codebase areas identified in your scope -- use Glob, Grep, and Read to find and examine relevant files
2. Do not speculate or guess about what exists -- verify by reading the actual implementation
3. Be concrete and specific -- cite file paths (with line numbers when relevant), function names, and actual code patterns
4. When you find something, verify it by reading surrounding code:
   - Check callers/consumers to understand usage
   - Check if tests cover it
   - Check configuration, wiring, or integration points
5. Ground every statement in actual code -- if you cannot verify something by reading files, say "could not verify" rather than guessing
6. Never flag something as uncertain if you can verify it by reading one more file

**Report requirements:**

- Every claim about existing code must reference a specific file path
- Distinguish between CONFIRMED findings (verified by reading code), LIKELY findings (strong evidence but incomplete verification), and POSSIBLE findings (suspicious patterns needing deeper investigation)
- When reporting what does NOT exist, state what you searched for and how you verified the absence
