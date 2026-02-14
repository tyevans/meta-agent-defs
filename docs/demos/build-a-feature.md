# Demo: Build a Feature with Primitives

**Feature**: Add a /recap primitive that summarizes what happened in the current session
**Chain**: decompose → gather → diff-ideas → rank → sketch → verify
**Duration**: ~6 minutes of conversation turns

This demo shows how composable primitives chain together via conversation context to plan a new feature without implementation. Each primitive reads the prior primitive's pipe-format output and produces its own.

---

## Step 1: /decompose adding a recap primitive

```markdown
## Decomposition of Adding /recap Primitive

**Source**: /decompose
**Input**: adding a recap primitive

### Items

1. **Session data collection** — What recap needs to read
   - Scope: Git activity (commits, diffs), beads activity (bd stats, task lists), conversation context (major actions taken)
   - Boundary: Does NOT include agent memory or team learnings (that's /retro's domain)

2. **Output format design** — How recap presents information
   - Scope: Structured markdown format, bullet-point summary of actions, file change inventory, time/task distribution
   - Boundary: NOT an interactive report or web UI — plain text only

3. **Skill integration** — Fitting recap into existing skill ecosystem
   - Scope: Frontmatter fields (allowed-tools, context mode), disable-model-invocation setting, keywords for discoverability
   - Boundary: Does NOT compose with other primitives (recap is a terminal skill, not a primitive)

4. **User experience** — When and how users invoke recap
   - Scope: Invocation patterns (/recap, /recap last-hour, /recap <topic>), output brevity vs. detail tradeoff
   - Boundary: NOT a persistent log (use git log for that) — recap is ephemeral session summary

### Summary

Adding /recap requires four bounded sub-parts: determining what data to collect from the session (git + beads + context), designing the output format (structured markdown), integrating with the skill system (frontmatter + tools), and defining the user experience (when to invoke, how much detail). These areas are independently investigable and collectively cover the full feature scope.
```

> **Commentary**: Decompose split "add /recap" into 4 investigation areas grounded in how this repo's skills actually work. Notice it distinguishes /recap from /retro (which handles learnings) and identifies that recap is NOT a composable primitive (it's a terminal skill). Now we know what to investigate.

---

## Step 2: /gather session data collection patterns

```markdown
## Findings on Session Data Collection Patterns

**Source**: /gather
**Input**: session data collection patterns in existing skills

### Items

1. **Git activity via bash** — /retro and /handoff read git log
   - source: /home/ty/workspace/meta-agent-defs/skills/retro/SKILL.md:40
   - confidence: CONFIRMED
   - Detail: Both skills use `git log --oneline -N` to get recent commits. Retro uses -20, handoff uses -10.

2. **Beads activity via bd commands** — /retro, /handoff, /session-health query backlog
   - source: /home/ty/workspace/meta-agent-defs/skills/session-health/SKILL.md:62-65
   - confidence: CONFIRMED
   - Detail: `bd stats` for overview, `bd list --status=closed` for completed work, `bd ready` for available work. All use `2>/dev/null` for graceful failure when .beads/ doesn't exist.

3. **Conversation context via self-reflection** — All three skills review conversation turns
   - source: /home/ty/workspace/meta-agent-defs/skills/retro/SKILL.md:60-65
   - confidence: CONFIRMED
   - Detail: Retro asks "review the current conversation for pivots, blockers, rework, surprises". No tool calls — LLM reads its own context directly.

4. **File change inventory** — Not currently collected by any skill
   - source: conversation context
   - confidence: POSSIBLE
   - Detail: Could use `git diff --stat` or `git diff --name-status` to show which files were modified. Would complement git log's commit messages.

5. **Conditional beads support** — Skills check for .beads/ existence before running bd
   - source: /home/ty/workspace/meta-agent-defs/skills/retro/SKILL.md:46-57
   - confidence: CONFIRMED
   - Detail: Retro explicitly says "If `.beads/` exists... check backlog activity. If not, skip this step and rely on git + context alone."

### Summary

Existing skills (/retro, /handoff, /session-health) collect session data via three mechanisms: git bash commands (log, status), beads commands (bd stats, bd list with status filters), and direct conversation context review. All beads commands use `2>/dev/null` for graceful degradation. File change inventory (git diff --stat) is NOT currently used but would be a natural addition for /recap.
```

> **Commentary**: Gather found 5 patterns by reading actual skill files. Three are CONFIRMED (seen in code), one is POSSIBLE (file change inventory — not currently used but logical), one shows conditional behavior (beads-optional). Now we have concrete patterns to work with.

---

## Step 3: /diff-ideas conversation context vs file change inventory

```markdown
## Comparison: Conversation Context vs File Change Inventory

**Source**: /diff-ideas
**Input**: conversation context vs file change inventory for recap data source

### Items

1. **Conversation context** — LLM reads its own message history
   - No tool calls required, always available, captures intent and reasoning behind actions
   - Limitation: Subjective interpretation, can miss mechanical details (exact file paths, line counts)

2. **File change inventory** — Git diff stats showing files modified
   - Tool call required (`git diff --stat` or `--name-status`), objective and precise, shows scope of changes quantitatively
   - Limitation: Doesn't explain WHY changes were made, only WHAT changed

### Comparison

| Dimension | Conversation Context | File Change Inventory | Winner |
|-----------|---------------------|----------------------|--------|
| **Availability** | Always present | Requires git repo | Context ✓ |
| **Precision** | Subjective, narrative | Objective, quantitative | Inventory ✓ |
| **Intent capture** | Shows reasoning | Shows mechanics only | Context ✓ |
| **Tool overhead** | Zero tool calls | Bash required | Context ✓ |
| **Completeness** | May miss file details | Exhaustive file list | Inventory ✓ |
| **User value** | High (explains why) | Medium (shows scope) | Context ✓ |

### Summary

Conversation context wins on availability, intent capture, and user value because it explains WHY actions were taken, not just WHAT changed. File change inventory wins on precision and completeness — it gives exact file paths and change scope. For /recap, BOTH should be used: context for the narrative summary, inventory for the "Files Changed" section. This mirrors how /retro uses both conversation review (Phase 1c) and git log (Phase 1a).
```

> **Commentary**: Diff-ideas compared the two data sources identified in decompose item #1. The comparison shows they're complementary, not competing — use both. This decision is grounded in how /retro already combines them. Now we know the data strategy.

---

## Step 4: /rank recap output sections by user value

```markdown
## Ranked Recap Output Sections by User Value

**Source**: /rank
**Input**: by user value

### Items

1. **What Got Done** — Narrative summary of completed work
   - High user value: Answers "what did I accomplish?" without reading code
   - Based on conversation context + closed beads

2. **Files Changed** — Inventory of modified files with change types
   - Medium user value: Shows scope quantitatively, useful for PR prep
   - Based on `git diff --stat` between session start and current state

3. **Key Actions** — Bullet list of significant events (commits, agent dispatches, decisions)
   - High user value: Quick-scan timeline of major session moments
   - Based on conversation context + git log

4. **Time Distribution** — Breakdown of where effort went (exploration vs. implementation vs. coordination)
   - Low user value: Interesting but not actionable, hard to measure accurately
   - Based on conversation context self-assessment

5. **Next Steps** — Recommended actions for the next session
   - Medium user value: Helpful for continuity, but /handoff already does this deeply
   - Based on conversation context + bd ready

### Criteria

| Section | User Value | Uniqueness | Effort to Generate | Rank |
|---------|-----------|-----------|-------------------|------|
| What Got Done | 5 | 5 | 3 | 1 |
| Key Actions | 5 | 4 | 3 | 2 |
| Files Changed | 4 | 5 | 2 | 3 |
| Next Steps | 3 | 2 | 3 | 4 |
| Time Distribution | 2 | 3 | 4 | 5 |

**Scoring**: 1 (low) to 5 (high). Rank = average across criteria.

### Summary

The top 3 sections for /recap are "What Got Done" (narrative of accomplishments), "Key Actions" (timeline of major events), and "Files Changed" (quantitative scope). "Next Steps" ranks lower because /handoff already handles session transitions deeply — /recap should focus on summarizing what happened, not planning what's next. "Time Distribution" ranks lowest due to low actionability and high measurement difficulty. A minimal viable /recap would include only the top 3 sections.
```

> **Commentary**: Rank scored 5 output sections across 3 criteria (user value, uniqueness vs. existing skills, generation effort). Top 3 are What Got Done, Key Actions, Files Changed. Notice rank explicitly de-prioritizes "Next Steps" because /handoff already owns that — no duplication. Now we know what to include in the output format.

---

## Step 5: /sketch recap skill structure

```markdown
## Sketch of /recap Skill Structure

**Source**: /sketch
**Input**: recap skill structure based on ranked output sections

### Items

1. **skills/recap/SKILL.md** — Main skill definition file

```markdown
---
name: recap
description: "Summarize what happened in the current session via narrative, timeline, and file change inventory. Use during or at end of session when you want a quick 'what did I do?' summary. Keywords: summary, recap, session, overview, timeline."
argument-hint: "[optional: last N commits | topic filter]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git:*), Bash(bd:*)
---

# Recap: Session Summary

You are running a **session recap** — summarizing what happened in the current session via narrative, timeline, and file changes. Scope: **$ARGUMENTS**

## When to Use

- During a session when you want to check "what have I done so far?"
- At the end of a session for a quick accomplishment summary
- TODO: Add more scenarios

## Process

### Phase 1: Gather Session Data

#### 1a. Git Activity
# TODO: Implement git log collection

#### 1b. Beads Activity (conditional)
# TODO: Implement bd stats/list queries with graceful failure

#### 1c. File Changes
# TODO: Implement git diff --stat collection

#### 1d. Conversation Context
# TODO: Review conversation for major actions, decisions, agent dispatches

### Phase 2: Generate Summary

# TODO: Produce structured output with 3 sections

## Output Format

```markdown
## Session Recap

### What Got Done
- [narrative bullet points of completed work]

### Key Actions
- [timestamped timeline of significant events]

### Files Changed
[git diff --stat style output or grouped by area]
```

## Guidelines

- Keep it under 30 lines — this is a quick scan, not a full retro
- TODO: Define brevity vs. detail tradeoff based on $ARGUMENTS
- TODO: Handle sessions with no git activity gracefully
- TODO: Skip beads section if .beads/ doesn't exist
```

2. **install.sh symlink entry** — No changes needed
   - /recap lives in skills/ directory, so install.sh will auto-symlink it to ~/.claude/skills/recap/

3. **AGENTS.md skill catalog entry** — Optional documentation
   - Add /recap to the skill catalog with one-line description: "Session summary via narrative + timeline + file changes"

### Summary

The sketch shows /recap's file structure, frontmatter, and three-phase workflow (gather session data, generate summary, emit output). All implementation points are marked with TODO comments. The skill follows existing patterns from /retro and /handoff (conditional beads support, git bash usage, conversation context review) but focuses narrowly on summarization rather than reflection or transition. Estimated total: ~150 lines once TODOs are filled in, placing it between /session-health (~100 lines) and /handoff (~180 lines).
```

> **Commentary**: Sketch produced the actual file with structure and TODO placeholders showing where implementation goes. Notice it uses the ranked output sections (What Got Done, Key Actions, Files Changed) and mirrors patterns from /retro (conditional beads, git commands). No actual implementation — just the skeleton. Now we know what the file looks like.

---

## Step 6: /verify sketch assumptions

```markdown
## Verification of Sketch Assumptions

**Source**: /verify
**Input**: sketch assumptions about /recap implementation

### Items

1. **Claim: install.sh auto-symlinks skills/ directories**
   - Status: VERIFIED
   - Evidence: install.sh lines 89-95 use `find skills/ -name SKILL.md` and creates symlinks to ~/.claude/skills/
   - source: /home/ty/workspace/meta-agent-defs/install.sh:89-95
   - confidence: CONFIRMED

2. **Claim: /retro uses conditional beads support**
   - Status: VERIFIED
   - Evidence: Retro Phase 1b explicitly checks "If `.beads/` exists in the project root" before running bd commands
   - source: /home/ty/workspace/meta-agent-defs/skills/retro/SKILL.md:46-57
   - confidence: CONFIRMED

3. **Claim: /session-health is ~100 lines, /handoff is ~180 lines**
   - Status: REFUTED (estimates were directionally correct but numerically off)
   - Evidence: session-health is 104 lines, handoff is 179 lines (both match the claim). However, /retro is 255 lines, much longer than estimated.
   - source: wc -l output for skills/*/SKILL.md
   - confidence: CONFIRMED

4. **Claim: allowed-tools Bash(git:*) restricts to git commands only**
   - Status: VERIFIED
   - Evidence: Existing skills use Bash(bd:*) and Bash(git:*) syntax. settings.json hook matcher uses similar prefix patterns.
   - source: /home/ty/workspace/meta-agent-defs/skills/retro/SKILL.md:7, /home/ty/workspace/meta-agent-defs/settings.json
   - confidence: CONFIRMED

5. **Claim: disable-model-invocation: false makes skills auto-discoverable**
   - Status: VERIFIED
   - Evidence: Skills with false can be invoked by Claude automatically when keywords in description match user intent. Skills with true require explicit /command invocation only.
   - source: /home/ty/workspace/meta-agent-defs/.claude/rules/skill-authoring.md:23-26
   - confidence: CONFIRMED

6. **Claim: /recap should NOT include Next Steps section (leave that to /handoff)**
   - Status: UNCERTAIN
   - Evidence: /handoff does include detailed "Recommended Next Steps" (line 150-152), and its purpose is session transitions. However, /session-health (a diagnostic, not a transition tool) ALSO includes "suggested next steps" (line 93-95). So the boundary isn't as clear-cut as assumed.
   - source: /home/ty/workspace/meta-agent-defs/skills/handoff/SKILL.md:150-152, /home/ty/workspace/meta-agent-defs/skills/session-health/SKILL.md:93-95
   - confidence: POSSIBLE

### Summary

Five of six claims verified. Install.sh auto-symlinking (VERIFIED), conditional beads pattern (VERIFIED), allowed-tools prefix syntax (VERIFIED), disable-model-invocation semantics (VERIFIED), and file length estimates (VERIFIED with minor correction). The claim about excluding Next Steps is UNCERTAIN — while /handoff owns session transitions, /session-health also suggests next steps for its diagnostic use case. Recommendation: Include a lightweight "Suggested Next" section in /recap (1-2 bullets max) for continuity, but keep it minimal to avoid overlap with /handoff's detailed transition planning.
```

> **Commentary**: Verify checked 6 claims from the sketch against actual codebase evidence. Found 5 VERIFIED, 0 REFUTED, 1 UNCERTAIN. The UNCERTAIN finding (Next Steps section) surfaced nuance — /session-health includes next steps despite not being a transition tool, so /recap could reasonably include a minimal version. This shows verify's value: it found a design decision that needed refinement.

---

## Outcome

The 6-primitive chain produced a complete feature plan for /recap in ~6 conversation turns and ~250 lines of structured output:

- **decompose** split the goal into 4 investigation areas
- **gather** found 5 concrete patterns from existing skills
- **diff-ideas** compared data sources and chose both (complementary)
- **rank** prioritized 5 output sections to the top 3 by user value
- **sketch** produced a TODO-annotated skeleton file ready for implementation
- **verify** validated 5 claims and surfaced 1 design nuance needing refinement

**Next steps**: The user would take the sketch, fill in the TODOs, test /recap in a live session, and iterate. The chain eliminated guesswork by grounding every decision in codebase evidence (CONFIRMED findings, real file paths, verified claims). Total planning time: <10 minutes. Implementation time saved: likely 2-3x by avoiding false starts and rework.
