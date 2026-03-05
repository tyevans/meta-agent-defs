# Usage Guide

Practical workflows for getting things done with Tackline. Organized by what you're trying to accomplish, not by skill name.

For installation, see [Setup](setup.md). For reference material, see [Reference](reference.md) and [INDEX](INDEX.md).

---

## I just installed Tackline. Where do I start?

### If this is a new project

```
/bootstrap
```

Bootstrap analyzes your codebase and sets up CLAUDE.md, hooks, rules, and generates project-specific agents. It's the single command that makes everything else work better.

### If this is an existing project

```
/status
```

Status shows what's going on: git state, backlog health, last session summary. If this is your first session, it'll be sparse — that's fine.

Then explore the area you care about:

```
/blossom audit the authentication flow for security gaps
```

Blossom dispatches spike agents that read actual source files and produce a prioritized task backlog. This is the fastest way to map unfamiliar territory.

---

## Common Workflows

### "I need to understand how something works"

**Use the research chain.** Each skill reads the previous one's output from conversation context — no file passing needed.

```
/gather how the payment processing pipeline works
/distill to 5 key points
```

That's it for most cases. If you need to verify claims:

```
/verify
```

Verify checks each finding against source code and marks it VERIFIED, REFUTED, or UNCERTAIN.

### "I need to decide between two approaches"

```
/gather authentication libraries for Express.js
/distill to top 3 options
/diff-ideas Passport.js vs Auth0
```

Diff-ideas produces a comparison table with per-dimension winners and an overall recommendation. Add `/verify` afterward to fact-check pricing or feature claims.

For bigger decisions where you want opposing viewpoints:

```
/meeting should we use microservices or a modular monolith?
```

Meeting assembles panelists with genuinely different perspectives. You steer the conversation, ask follow-ups, and cut unproductive threads. Output is a decision record.

### "I need to find and fix problems"

```
/blossom audit error handling in the API layer
```

Blossom runs spike agents that trace call chains and tag findings with confidence levels:
- **CONFIRMED** — agent read the code and verified the issue
- **LIKELY** — strong evidence but incomplete trace
- **POSSIBLE** — needs deeper investigation

Output is an epic with prioritized tasks in your backlog. Then execute:

```
/sprint
```

Sprint reads the backlog, picks the highest-priority ready tasks, and dispatches agents to implement fixes.

### "I have a plan and want to execute it"

If you have a spec, design doc, or plan file:

```
/drive docs/plan.md
```

Drive runs sustained sprint/retro cycles until the plan is complete or you stop it. It delegates work via `/sprint`, runs retrospectives, updates documentation, and commits regularly. It's the autonomous mode.

If the plan is in your head, write it down first:

```
/spec billing module redesign
```

Spec produces a structured specification through progressive elaboration: requirements, constraints, open questions, acceptance criteria. Then pass it to drive.

### "I need to build a feature end-to-end"

For features where integration risk is high (crosses system boundaries, touches multiple services):

```
/tracer implement webhook delivery with retry logic
```

Tracer builds the thinnest working end-to-end path first, then widens pass by pass. You always have something that works — each iteration adds depth.

For features where the approach is clear and you want test-driven development:

```
/test-strategy implement input validation for the registration form
```

Test-strategy classifies the knowledge source, writes tests from specs, and enforces red-green gates.

### "I want to review code"

```
/review
```

Review runs a structured analysis across correctness, security, style, architecture, and testing dimensions. Good for pre-merge checks or getting a second opinion on changes.

For adversarial review (what's wrong, what's missing, what could fail):

```
/critique
```

Critique is the devil's advocate — it finds weaknesses in whatever's in context.

---

## Working with Teams

Teams are persistent learning agents that improve across sessions. Use them for larger projects where multiple agents need to coordinate.

### Setting up a team

```
/assemble
```

Assemble creates a team from a template (or interactively). Each member has a role, ownership area, and learnings file that persists across sessions.

### Running a work session with a team

```
/standup              # sync status, surface blockers
/sprint               # dispatch work to team members
/retro                # reflect on what worked and what didn't
```

Sprint injects each agent's learnings before dispatching, so agents get better over time. Retro captures what they learned and persists it.

### Optimizing team learnings

```
/tend
```

Tend runs the full learning lifecycle: scores entries by relevance, archives stale ones, detects gaps, and promotes durable patterns to project rules. Run it between sprints or after completing an epic.

---

## Session Management

Every session should follow this pattern:

```
/status               # orient: what happened last time, what's ready now
... work ...
/retro                # reflect: what was completed, what was learned
/handoff              # capture: save state for the next session
```

The key invariant: **end every session with `/retro` + `/handoff`**. Sessions that skip handoff force the next session to re-derive context from git history alone.

If a session feels degraded (responses getting worse, losing track of context):

```
/session-health
```

Session-health diagnoses context load, scope drift, and quality degradation. If it recommends a fresh session, run `/handoff` first.

---

## Primitive Chains

Primitives are stateless skills that chain through conversation context. The output of one feeds the next — no file passing, no explicit piping.

### The core primitives

| Primitive | What it does | When to use it |
|-----------|-------------|----------------|
| `/gather` | Collect findings with sources | Always first — other primitives need items to work on |
| `/distill` | Condense to essentials | When gather produced too much |
| `/rank` | Score and order by criteria | When you need priorities, not categories |
| `/assess` | Categorize by rubric (OK/WARNING/CRITICAL) | When you need severity levels, not rankings |
| `/filter` | Keep or drop items | After rank or assess, to narrow focus |
| `/verify` | Check claims against evidence | When you need to confirm before acting |
| `/critique` | Find what's wrong, missing, or risky | When you want adversarial pressure |
| `/decompose` | Break into bounded sub-parts | When a task is too big to tackle directly |
| `/plan` | Sequence by dependencies | When ordering matters |

### Common chains

```
/gather -> /distill -> /rank              # Research, condense, prioritize
/gather -> /assess -> /filter             # Audit, categorize, focus
/gather -> /distill -> /diff-ideas        # Research, condense, compare approaches
/decompose -> /rank -> /plan              # Break down, prioritize, sequence
/gather -> /critique -> /rank             # Research, stress-test, prioritize
```

You can skip steps. If gather gives you actionable items, go straight to rank. If distill leaves only 2 options, go straight to diff-ideas.

For annotated walkthroughs with full output examples, see [Primitives Cookbook](primitives-cookbook.md).

---

## Running Multiple Sessions in Parallel

If you run multiple Claude Code sessions against the same project (e.g., four terminals each working on different tasks), their checkpoint files will collide by default.

**Fix:** Use shell aliases that inject a unique instance ID per launch:

```bash
alias claudes='CLAUDE_INSTANCE_ID=$(uuidgen || cat /proc/sys/kernel/random/uuid) claude /sandbox'
alias clauded='CLAUDE_INSTANCE_ID=$(uuidgen || cat /proc/sys/kernel/random/uuid) claude /sandbox --dangerously-skip-permissions'
```

When `CLAUDE_INSTANCE_ID` is set, skills like `/drive` namespace their scratch files so parallel sessions don't overwrite each other's state.

See [Setup Guide — Parallel session support](setup.md#parallel-session-support) for details.

---

## Tips

1. **Be specific.** `/blossom audit error handling in the payment API` produces better results than `/blossom look at the codebase`. Provide file paths when you know them.

2. **Read the summary.** Every skill's summary paragraph tells you if you're done or what to do next. "These 4 approaches are equally viable — recommend diff-ideas comparison" is a signal, not filler.

3. **Trust but verify.** CONFIRMED findings have been traced to source code. LIKELY and POSSIBLE findings need more investigation. Don't treat them equally.

4. **Commit often during `/drive`.** Drive commits for you, but if you're working manually, commit after each meaningful change. Skills that checkpoint (`/drive`, `/sprint`) use scratch files that survive context compaction — but only if the work is committed.

5. **Use `/discover` when lost.** If you don't know which skill to use, describe your goal and `/discover` will recommend the right skill or chain.
