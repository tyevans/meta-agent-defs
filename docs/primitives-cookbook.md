# Primitives Cookbook

Annotated walkthroughs of composable primitive chains applied to the tackline repository. Each recipe shows FULL pipe-format output at every step with commentary explaining the transformations.

For chain patterns without annotations, see [primitives-recipes.md](./primitives-recipes.md).

---

## Recipe 1: Audit Skill Consistency

**Goal**: Find skills that deviate from the SKILL.md authoring standards and understand what needs fixing.

**Chain**: gather → assess → filter

### Step 1: /gather skill frontmatter patterns

```markdown
## Findings on Skill Frontmatter Patterns

**Source**: /gather
**Input**: skill frontmatter patterns in skills/*/SKILL.md

### Items

1. **gather skill** — has name, description, argument-hint, disable-model-invocation (false), user-invocable (true), allowed-tools (Read, Grep, Glob, WebSearch, WebFetch)
   - source: /home/ty/workspace/tackline/skills/gather/SKILL.md:1-8
   - confidence: CONFIRMED

2. **distill skill** — has name, description, argument-hint, disable-model-invocation (false), user-invocable (true), allowed-tools (Read, Grep, Glob)
   - source: /home/ty/workspace/tackline/skills/distill/SKILL.md:1-8
   - confidence: CONFIRMED

3. **assess skill** — has name, description, argument-hint, disable-model-invocation (false), user-invocable (true), allowed-tools (Read, Grep, Glob)
   - source: /home/ty/workspace/tackline/skills/assess/SKILL.md:1-8
   - confidence: CONFIRMED

4. **blossom skill** — has name, description, argument-hint, disable-model-invocation (false), user-invocable (true), allowed-tools (Read, Grep, Glob, Bash(bd:*), Task, TeamCreate, TaskCreate, SendMessage), PLUS context: fork
   - source: /home/ty/workspace/tackline/skills/blossom/SKILL.md:1-11
   - confidence: CONFIRMED

5. **retro skill** — has name, description, disable-model-invocation (true), user-invocable (true), allowed-tools (Read, Grep, Glob, Write, Bash(bd:*)), NO argument-hint
   - source: /home/ty/workspace/tackline/skills/retro/SKILL.md:1-8
   - confidence: CONFIRMED

6. **sprint skill** — has name, description, disable-model-invocation (false), user-invocable (true), allowed-tools (Read, Grep, Glob, Task, Bash(bd:*)), NO argument-hint
   - source: /home/ty/workspace/tackline/skills/sprint/SKILL.md:1-8
   - confidence: CONFIRMED

7. **assemble skill** — has name, description, disable-model-invocation (false), user-invocable (true), allowed-tools (Read, Write, Edit, Grep, Glob, Bash(bd:*), TeamCreate, SendMessage), NO argument-hint
   - source: /home/ty/workspace/tackline/skills/assemble/SKILL.md:1-8
   - confidence: CONFIRMED

### Summary

Seven sampled skills show consistent use of required frontmatter fields (name, description, disable-model-invocation, user-invocable, allowed-tools), but three skills (retro, sprint, assemble) lack argument-hint. The composable primitives (gather, distill, assess) all use argument-hint to guide users, while workflow skills (blossom, retro, sprint, assemble) omit it. Blossom is the only skill using context: fork.
```

**Commentary**: Gather scanned 7 skills and found a pattern: composable primitives include argument-hint, workflow skills don't. Now we assess whether this inconsistency matters.

---

### Step 2: /assess by compliance with skill-authoring.md rules

```markdown
## Assessed Skill Frontmatter Compliance

**Source**: /assess
**Input**: by compliance with skill-authoring.md rules

### Items

1. **OK** — gather, distill, assess have all required fields and argument-hint for usability
   - source: skills/gather/SKILL.md, skills/distill/SKILL.md, skills/assess/SKILL.md
   - confidence: CONFIRMED

2. **OK** — blossom has all required fields, argument-hint, and context: fork for isolation
   - source: skills/blossom/SKILL.md
   - confidence: CONFIRMED

3. **WARNING** — retro missing argument-hint (skill-authoring.md lists it as "optional", so technically compliant)
   - source: skills/retro/SKILL.md:1-8
   - confidence: CONFIRMED

4. **WARNING** — sprint missing argument-hint
   - source: skills/sprint/SKILL.md:1-8
   - confidence: CONFIRMED

5. **WARNING** — assemble missing argument-hint
   - source: skills/assemble/SKILL.md:1-8
   - confidence: CONFIRMED

### Rubric

| Category | Definition |
|----------|------------|
| OK | All required fields present, argument-hint present for user-invocable skills |
| WARNING | Required fields present, but missing optional argument-hint that would improve UX |
| CRITICAL | Missing required frontmatter fields (name, description, allowed-tools, etc.) |

### Summary

Four skills (gather, distill, assess, blossom) are fully compliant with all frontmatter fields including argument-hint. Three skills (retro, sprint, assemble) are technically compliant but missing the optional argument-hint field, which makes the CLI less discoverable for users typing `/sprint <tab>` to see what arguments are expected. No skills are missing required fields. The WARNING-level findings are UX improvements, not compliance failures.
```

**Commentary**: Assess categorized findings by severity. We have 3 WARNING items (missing optional field) but no CRITICAL issues. Let's filter to surface only the skills that need attention.

---

### Step 3: /filter keep warning only

```markdown
## Filtered Skill Frontmatter Issues

**Source**: /filter
**Input**: keep warning only

### Items

1. **retro missing argument-hint** — skill-authoring.md lists argument-hint as optional, but including it improves UX
   - source: skills/retro/SKILL.md:1-8
   - confidence: CONFIRMED

2. **sprint missing argument-hint** — users typing `/sprint <tab>` see no hint about expected goal format
   - source: skills/sprint/SKILL.md:1-8
   - confidence: CONFIRMED

3. **assemble missing argument-hint** — users don't know whether to pass a team name, goal, or both
   - source: skills/assemble/SKILL.md:1-8
   - confidence: CONFIRMED

### Dropped

Dropped 2 items: the 4 OK-level skills (gather, distill, assess, blossom) don't need changes.

### Summary

Three workflow skills need argument-hint additions for better CLI discoverability. All three are technically compliant with skill-authoring.md (argument-hint is optional), but adding the field would improve UX when users type `/skillname <tab>` in the terminal. Recommend adding: retro (no args needed → empty string or ""), sprint ("sprint goal"), assemble ("team purpose").
```

**Commentary**: Filter narrowed 5 assessed items to the 3 that need action. Output is now actionable — we have specific file paths and recommended argument-hint values.

---

### Takeaway

Use this chain (gather → assess → filter) when auditing compliance with standards. Gather inventories current state, assess applies the rubric, filter surfaces what needs fixing. Works for code standards, documentation completeness, or security patterns.

---

## Recipe 2: Choose Agent Dispatch Strategy

**Goal**: Decide between serialized Task dispatch vs. parallel Team dispatch for implementing a multi-agent spike.

**Chain**: gather → distill → diff-ideas

### Step 1: /gather dispatch strategies in CLAUDE.md and team-protocol.md

```markdown
## Findings on Agent Dispatch Strategies

**Source**: /gather
**Input**: dispatch strategies in CLAUDE.md and team-protocol.md

### Items

1. **Serialize by default** — dispatch one task at a time via Task tool, review output, then dispatch next to avoid API throttling and allow each task to benefit from prior findings
   - source: /home/ty/workspace/tackline/CLAUDE.md:18
   - confidence: CONFIRMED

2. **Teams enable parallelization** — use TeamCreate + SendMessage for independent tasks when teams are enabled, running in separate contexts to avoid throttling and context bloat
   - source: /home/ty/workspace/tackline/CLAUDE.md:20
   - confidence: CONFIRMED

3. **Task tool with run_in_background** — for parallel dispatch without teams, use Task(..., run_in_background: true) to launch agents concurrently
   - source: /home/ty/workspace/tackline/.claude/rules/fan-out-protocol.md:14-19
   - confidence: CONFIRMED

4. **Teams for coordination** — when agents need to respond to each other (meeting, blossom with teams), use Teams; for independent parallel work, background Task agents are simpler
   - source: /home/ty/workspace/tackline/.claude/rules/fan-out-protocol.md:52-62
   - confidence: CONFIRMED

5. **Concurrency limit** — launch up to 4 agents at once; more risks API throttling
   - source: /home/ty/workspace/tackline/.claude/rules/fan-out-protocol.md:20
   - confidence: CONFIRMED

6. **Subagents cannot invoke skills** — all workflow logic must be embedded in the agent's prompt; Skill tool is not available to subagents
   - source: /home/ty/workspace/tackline/.claude/rules/fan-out-protocol.md:42
   - confidence: CONFIRMED

### Summary

The repo defines three dispatch patterns: serialize (one-at-a-time Task calls), parallel background Tasks (Task with run_in_background: true), and Teams (TeamCreate + SendMessage for coordination). Default is serialize to avoid throttling. Teams are for coordination (agents responding to each other); background Tasks are for independent work. Concurrency capped at 4 agents. Subagents can't invoke skills, so all instructions go in the prompt.
```

**Commentary**: Gather found the three dispatch approaches documented in the repo. Now we distill to the core tradeoffs.

---

### Step 2: /distill to 3 bullets

```markdown
## Distilled Dispatch Strategy Tradeoffs

**Source**: /distill
**Input**: to 3 bullets

### Items

1. **Serialize (default)** — one Task at a time, each benefits from prior findings; avoids API throttling; slower but safer
   - source: /home/ty/workspace/tackline/CLAUDE.md:18

2. **Parallel background Tasks** — launch up to 4 agents concurrently with run_in_background: true; for independent work only (agents can't coordinate); faster but risks throttling
   - source: /home/ty/workspace/tackline/.claude/rules/fan-out-protocol.md:14-20

3. **Teams** — TeamCreate + SendMessage for coordination between agents; separate contexts prevent bloat; best for complex multi-agent dialogue (meeting, blossom); adds orchestration overhead
   - source: /home/ty/workspace/tackline/CLAUDE.md:20, fan-out-protocol.md:52-62

### Summary

Three dispatch patterns trade off speed, coordination capability, and complexity. Serialize is safest (no throttling, sequential learning). Parallel background Tasks are fastest for independent work but risk API limits. Teams enable coordination (agents respond to each other) at the cost of orchestration overhead. Choose based on: do agents need to talk? (Teams) Do tasks build on each other? (serialize) Are they independent and time-sensitive? (parallel Tasks).
```

**Commentary**: Distill condensed 6 findings to 3 core options with their key tradeoffs. Now we compare the two most relevant: parallel Tasks vs. Teams.

---

### Step 3: /diff-ideas parallel background Tasks vs Teams

```markdown
## Comparison: Parallel Background Tasks vs Teams

**Source**: /diff-ideas
**Input**: parallel background Tasks vs Teams

### Items

1. **Parallel background Tasks** — Task tool with run_in_background: true; agents run concurrently, results collected via TaskOutput; agents are isolated, cannot coordinate; simpler orchestration (just collect results); capped at 4 concurrent to avoid throttling
   - source: /home/ty/workspace/tackline/.claude/rules/fan-out-protocol.md:14-20

2. **Teams** — TeamCreate + SendMessage; agents run in separate contexts, can coordinate via messages; supports complex dialogue (agents respond to each other); higher orchestration cost (manage team lifecycle, message routing); no hard concurrency cap (separate contexts avoid throttling)
   - source: /home/ty/workspace/tackline/CLAUDE.md:20, fan-out-protocol.md:52-62

### Comparison

| Dimension | Parallel Background Tasks | Teams | Winner |
|-----------|---------------------------|-------|--------|
| **Coordination** | Agents cannot communicate | Agents send/receive messages | Teams ✓ |
| **Simplicity** | Low overhead (just collect results) | Higher overhead (team lifecycle, message routing) | Tasks ✓ |
| **Concurrency** | Capped at 4 agents | No hard cap (separate contexts) | Teams ✓ |
| **Use case** | Independent parallel work | Multi-agent dialogue, coordination | Context-dependent |
| **Orchestration** | Minimal (launch, collect) | Moderate (manage team, route messages) | Tasks ✓ |
| **Throttling risk** | Higher (shared context) | Lower (separate contexts) | Teams ✓ |

### Summary

Parallel background Tasks win on simplicity and low orchestration overhead, making them ideal for independent work where agents don't need to coordinate (e.g., parallel code reviews, independent research spikes). Teams win on coordination capability, concurrency, and throttling resistance, making them better for complex workflows where agents respond to each other (e.g., meeting, blossom with team mode). For a multi-agent spike where findings are independent: use parallel Tasks. For a spike where agents build on each other's findings: use Teams.
```

**Commentary**: Diff-ideas produced a comparison table showing parallel Tasks are simpler, Teams enable coordination. The recommendation is context-dependent: independent work → Tasks, coordinated work → Teams.

---

### Takeaway

Use this chain (gather → distill → diff-ideas) when making technical decisions with multiple viable approaches. Gather finds the options, distill extracts the essentials, diff-ideas forces explicit tradeoff analysis with a structured comparison table. The output is decision-ready.

---

## Recipe 3: Triage Open Backlog

**Goal**: Prioritize open beads backlog items by value-effort ratio and identify what to tackle next.

**Chain**: gather → rank → decompose

### Step 1: /gather open beads issues

```markdown
## Findings on Open Beads Issues

**Source**: /gather
**Input**: open beads issues from bd stats

### Items

1. **9 open issues, 2 in-progress, 4 blocked** — total 106 issues (95 closed), 5 ready to work
   - source: bd stats output, conversation context
   - confidence: CONFIRMED

2. **Interactive /assemble skill** — implement interactive prompting for team member addition in /assemble workflow
   - source: .beads/ database (inferred from P3 backlog mention in project memory)
   - confidence: LIKELY

3. **Team documentation gaps** — improve documentation for team creation, learnings management, and cross-agent coordination
   - source: project memory MEMORY.md (mentioned as P3 backlog item)
   - confidence: LIKELY

4. **Hook precedence untested** — global vs project-local settings.json hook precedence is undocumented and untested
   - source: /home/ty/workspace/tackline/CLAUDE.md gotchas section (inferred from memory)
   - confidence: POSSIBLE

5. **No OTel/cost tracking** — remaining gap in system maturity, no observability or outcome measurement
   - source: project memory MEMORY.md remaining gaps
   - confidence: CONFIRMED

### Summary

Five open/deferred issues identified: interactive /assemble (P3), team docs (P3), hook precedence (untested), cost tracking (missing), plus 5 ready-to-work issues from bd stats. The backlog is small (9 open) with most work complete (95 closed). Two items are explicitly marked P3 (low priority), suggesting the backlog is well-triaged but not yet scheduled.
```

**Commentary**: Gather found 5 open/deferred issues from memory and beads stats. Now we rank by value-effort to prioritize.

---

### Step 2: /rank by user value and implementation effort

```markdown
## Ranked Open Backlog Items

**Source**: /rank
**Input**: by user value and implementation effort

### Items

1. **Team documentation gaps** — improve docs for team creation, learnings, cross-agent coordination
   - source: project memory MEMORY.md

2. **Hook precedence untested** — document and test global vs project-local settings.json hook behavior
   - source: CLAUDE.md gotchas

3. **Interactive /assemble skill** — add interactive prompting for team member roles in /assemble
   - source: .beads/ database (P3)

4. **No OTel/cost tracking** — add observability and cost measurement
   - source: project memory MEMORY.md gaps

### Criteria

| Item | User Value (1-5) | Effort (1-5, low=1) | Rank (avg) |
|------|------------------|---------------------|------------|
| Team documentation gaps | 5 | 2 | 3.5 |
| Hook precedence untested | 4 | 2 | 3.0 |
| Interactive /assemble skill | 3 | 3 | 3.0 |
| No OTel/cost tracking | 3 | 5 | 4.0 |

### Summary

Team docs rank highest (5 value, 2 effort) — users are actively using teams and hitting undocumented edge cases. Hook precedence ties with interactive /assemble (both 3.0 avg) but ranks higher due to being a correctness issue vs. UX enhancement. OTel/cost tracking ranks lowest (high effort, moderate value) — important for production use but not blocking current workflows. Recommend: tackle team docs first (high value, low effort), then hook precedence testing (medium value, low effort), defer OTel until usage scales.
```

**Commentary**: Rank scored items on value (user impact) and effort (implementation cost), then averaged to produce a priority order. Team docs are the clear winner (high value, low effort). Now we decompose the top item into actionable sub-parts.

---

### Step 3: /decompose team documentation gaps

```markdown
## Decomposition of Team Documentation Gaps

**Source**: /decompose
**Input**: team documentation gaps

### Items

1. **Team creation workflow** — document TeamCreate, team.yaml structure, member spawning via Task tool
   - Scope: /assemble skill walkthrough, team.yaml field reference, member spawn examples
   - Boundary: Does NOT include team templates (already documented in templates/teams/)

2. **Learnings management** — document learnings.md lifecycle, 60-line cap, core vs task-relevant split, stale entry archival
   - Scope: learnings file structure, cap enforcement rules, archival trigger (21 days), cross-agent notes triage (14 days)
   - Boundary: Does NOT include learning content guidelines (that's agent-author's domain)

3. **Cross-agent coordination** — document SendMessage protocol, message routing, idle state, peer DM visibility
   - Scope: message types (DM, broadcast, shutdown), idle notifications, DM summary in idle messages
   - Boundary: Does NOT include fan-out/fan-in protocol (already documented in fan-out-protocol.md)

### Summary

Team documentation gaps decompose into three independent areas: team creation (how to start), learnings management (how knowledge persists), and cross-agent coordination (how agents communicate). Each area is self-contained and can be documented independently. Focus on learnings management first — it's the most novel pattern and has the most edge cases (cap enforcement, staleness rules, cross-agent notes).
```

**Commentary**: Decompose split the top-ranked item (team docs) into 3 bounded sub-parts, each with clear scope and explicit boundaries. The output is ready for task creation or agent dispatch.

---

### Takeaway

Use this chain (gather → rank → decompose) for backlog triage and sprint planning. Gather pulls in candidates, rank prioritizes by multiple criteria, decompose breaks the winner into actionable chunks. The final output is dispatch-ready — create tasks for each decomposed sub-part or hand the whole chain to /sprint.

---

## Key Patterns Across Recipes

1. **Gather is always first** — other primitives need structured items to work on. If you skip gather, you're guessing.

2. **Assess vs rank** — assess categorizes (discrete buckets like OK/WARNING/CRITICAL), rank orders (continuous scores). Use assess for compliance/severity, rank for prioritization.

3. **Filter after assess/rank** — narrowing is cheaper than evaluating. Assess/rank everything first, then filter to what matters.

4. **Decompose bridges analysis and execution** — the last step before task creation or agent dispatch. It turns "what's wrong" into "what to do."

5. **Distill shortens chains** — if you need a quick decision, distill after gather and skip straight to diff-ideas. Fewer steps, faster cycle.

6. **Diff-ideas needs exactly 2 options** — if gather found 5 approaches, run rank or distill first to narrow to the top 2.

7. **Summaries signal next steps** — each primitive's summary paragraph tells you if you're done or what primitive comes next. Read summaries, don't skip them.
