---
name: curate
description: "Optimize an agent's learnings for upcoming work: score each entry by relevance, archive stale or redundant entries, detect gaps where upcoming work needs knowledge not yet captured, and pull in relevant entries from archive or cross-agent sources. Use before sprints, after /retro, or when learnings feel bloated or misaligned. Keywords: curate, learnings, optimize, relevance, prune, archive, gap, score, triage."
argument-hint: "<agent-name>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Bash(ls:*), Write, Edit
---

# Curate: Optimize Agent Learnings for Upcoming Work

You are running **curate** — scoring an agent's learnings by relevance to upcoming tasks, archiving stale or already-covered entries, and filling gaps from archive or cross-agent sources.

**Target agent:** $ARGUMENTS

## When to Use

- Before a sprint — ensure the agent's learnings are tuned to what's coming
- After `/retro` adds new entries — prune stale entries to stay under the 60-line cap
- When `/tend` orchestrates the full lifecycle — curate runs first, promote reads the output
- When learnings feel bloated, stale, or out of sync with current work
- When an agent will be assigned tasks in a new domain area

## How It Works

```
Validate agent exists
  -> Load learnings + upcoming work + rules + agent definition
    -> Score each learning entry (HIGH / MEDIUM / LOW / PASSIVE)
      -> Detect gaps (file scope of upcoming work vs. learnings coverage)
        -> Pull candidates from archive or cross-agent sources for gaps
          -> Emit pipe-format report
            -> (Optional) Write back to learnings.md and archive.md
```

---

## Phase 0: Validate Agent

If `$ARGUMENTS` is empty:
1. Run `ls memory/agents/` to list available agents
2. Ask: "Which agent should I curate? Available: [names]"
3. Stop and wait for response

If `$ARGUMENTS` is provided:
1. Confirm `memory/agents/<name>/learnings.md` exists
2. If not found: "No learnings file at memory/agents/<name>/learnings.md. Has this agent been dispatched yet?"
3. Note whether `memory/agents/<name>/archive.md` exists (needed for Phase 4)
4. Note whether `.claude/team.yaml` exists (used in Phase 1d to read the agent definition)

---

## Phase 1: Load Context

### 1a. Agent Learnings

Read `memory/agents/<name>/learnings.md`. Note:
- Total line count (cap is 60 lines: 30 Core + 30 Task-Relevant)
- Each distinct entry (entries separated by blank lines or bullet markers)
- Entry provenance where present: `(added: YYYY-MM-DD, dispatch: <source>)`

### 1b. Upcoming Work

Gather the upcoming task signal from whichever sources are available:

```bash
bd ready 2>/dev/null
bd list --status=in_progress 2>/dev/null
ls memory/epics/*/epic.md 2>/dev/null
```

Read any epic state files found. Extract:
- **File scope**: which files or directories are mentioned in task descriptions and epic state
- **Domain areas**: what types of work (e.g., test authoring, skill writing, agent generation, hook scripting)
- **Keywords**: specific terms, tools, or patterns mentioned repeatedly

If beads are not available and no epics exist, use git signals:

```bash
git log --oneline -10
git diff --name-only HEAD~5..HEAD 2>/dev/null
```

Recent commit activity approximates upcoming work focus areas.

### 1c. Rules and Passive Context

Read all rule files to detect overlap with learnings:

```bash
ls rules/*.md .claude/rules/*.md 2>/dev/null
```

Read each rule file. Read `CLAUDE.md`. Build a list of topics already covered passively so learnings that duplicate them can be flagged as PASSIVE.

### 1d. Agent Definition

If `.claude/team.yaml` exists, read it and extract the target agent's entry:
- `role`: what the agent is responsible for
- `owns`: file glob patterns defining ownership
- `model`: complexity context

If team.yaml does not exist, skip this step.

---

## Phase 2: Score Each Learning Entry

For each entry in `learnings.md`, assign one of four scores:

| Score | Meaning | Action |
|-------|---------|--------|
| HIGH | Directly relates to files, domains, or tools in upcoming tasks | Keep |
| MEDIUM | Related to the general area of upcoming work but not specific tasks | Keep |
| LOW | No connection to upcoming work — relevant in general but not now | Archive |
| PASSIVE | Already covered by an existing rule or CLAUDE.md (redundant) | Archive with note |

### Scoring Heuristics

**HIGH:** The entry mentions a specific file, tool, command, or pattern that appears in upcoming task descriptions or epic state. Example: an entry about `bd create --parent` syntax is HIGH if epic management tasks are coming.

**MEDIUM:** The entry covers a category of work that's coming but doesn't match specific files. Example: entries about skill authoring conventions are MEDIUM when new skills are on the backlog.

**LOW:** The entry is about a past domain area no longer in scope, or is general background that won't be needed soon. Example: entries about a feature area with no upcoming beads and no recent commits.

**PASSIVE:** The entry states something already enforced by a rule in `rules/` or `CLAUDE.md`. Example: an entry "use conventional commits" is PASSIVE if `rules/commits.md` already covers this. When marking PASSIVE, cite the specific rule file.

---

## Phase 3: Detect Gaps

Compare the file scope and domain areas from Phase 1b against the topics covered by learnings entries scored HIGH or MEDIUM.

### 3a. Build Coverage Map

For each distinct domain area in upcoming work, note whether any HIGH or MEDIUM entry covers it:

```
Domain area: skill authoring
  Covered by: entry about frontmatter fields (HIGH), entry about phase structure (MEDIUM)
  Status: COVERED

Domain area: hook scripting
  Covered by: (none)
  Status: GAP
```

### 3b. Flag Gaps

A gap is a domain area where:
1. Upcoming work touches it (files in task scope, keywords in descriptions)
2. No HIGH or MEDIUM learnings entry covers it

Gaps are actionable: the agent will operate in this area without relevant learnings loaded.

### 3c. Check Archive for Gap Fills

If `memory/agents/<name>/archive.md` exists, read it. Scan for entries that match gap areas. These are **archive candidates** — entries that were previously archived but are now relevant again.

### 3d. Check Cross-Agent Sources

Scan other agents' learnings files for entries relevant to the gap areas:

```bash
ls memory/agents/*/learnings.md 2>/dev/null
```

Read each file that is not the target agent's. For each gap, check whether another agent has an entry covering it. Cross-agent candidates are flagged for potential pull-in. Note the source agent name — this is also relevant for `/promote` (patterns across agents indicate rule candidacy).

---

## Phase 4: Compose Output

Emit in pipe format per `rules/pipe-format.md`:

```markdown
## Curated Learnings: <agent-name>

**Source**: /curate
**Input**: <agent-name>
**Pipeline**: (none — working from direct input)

### Items (N)

1. **KEEP (HIGH): <entry title or first clause>** — <full entry text>
   - score: HIGH
   - reason: <why this is HIGH — reference to specific upcoming task or file>
   - source: memory/agents/<name>/learnings.md

2. **KEEP (MEDIUM): <entry title>** — <full entry text>
   - score: MEDIUM
   - reason: <why MEDIUM>
   - source: memory/agents/<name>/learnings.md

3. **ARCHIVE (LOW): <entry title>** — <full entry text>
   - score: LOW
   - reason: <no upcoming tasks touch this area>
   - source: memory/agents/<name>/learnings.md

4. **ARCHIVE (PASSIVE): <entry title>** — <full entry text>
   - score: PASSIVE
   - reason: covered by rule: <rule filename>
   - source: memory/agents/<name>/learnings.md

5. **ADD (from archive): <entry title>** — <entry text>
   - score: HIGH | MEDIUM
   - reason: <gap area this fills>
   - source: memory/agents/<name>/archive.md

6. **ADD (cross-agent): <entry title>** — <entry text>
   - score: HIGH | MEDIUM
   - reason: <gap area this fills; note if same entry exists in 2+ agents — promote candidate>
   - source: memory/agents/<other-name>/learnings.md
   - cross-agent: true

### Gaps (M)

For each gap with no fill candidate:

- **GAP: <domain area>** — upcoming tasks touch <files/patterns> but no entry covers this area. Consider adding a learning after the next dispatch in this area.

### Summary

<One paragraph: how many entries scored HIGH/MEDIUM/LOW/PASSIVE, how many gaps were found, how many fill candidates were identified from archive or cross-agent sources, and whether any cross-agent entries appear in 2+ agents (promote signals). Note overall learnings health — is the file well-targeted or diffuse?>
```

Order items: ADD first (most valuable new context), then KEEP (HIGH before MEDIUM), then ARCHIVE (LOW before PASSIVE). This puts what changes at the top for easy review.

---

## Phase 5: Write-Back (Conditional)

After presenting the output, ask the user:

> "Apply these changes to learnings.md and archive.md? (y/n)"

If the user approves:

### 5a. Update learnings.md

Write the updated file with:
- All KEEP (HIGH) and KEEP (MEDIUM) entries retained
- All ADD entries inserted (from archive or cross-agent sources)
- All ARCHIVE entries removed
- Entries organized into Core (30 lines max) and Task-Relevant (30 lines max) sections
- Total must not exceed 60 lines

When re-organizing:
- **Core**: high-reuse fundamentals that apply across most tasks for this agent
- **Task-Relevant**: entries specific to current upcoming work scope

### 5b. Update archive.md

Append each ARCHIVE entry to `memory/agents/<name>/archive.md`:

```markdown
- <entry text> (archived: YYYY-MM-DD, reason: LOW — no upcoming tasks in this area)
- <entry text> (archived: YYYY-MM-DD, reason: PASSIVE — covered by rule: rules/commits.md)
```

If `archive.md` does not exist, create it with a header line:

```markdown
# Archived Learnings: <agent-name>
```

### 5c. Verify

Read the updated `learnings.md` and confirm:
- Line count is within the 60-line cap
- All sections are properly structured
- No ARCHIVE entries remain in learnings.md

---

## Guidelines

1. **Score against upcoming work, not general value.** A learning can be deeply true and still score LOW because nothing in the upcoming sprint will trigger it. That's fine — archive it and re-evaluate after the next sprint.

2. **PASSIVE is not wrong — it's covered.** Marking an entry PASSIVE is a compliment: the pattern graduated to a rule. Archive it without judgment.

3. **Gaps are diagnostic, not failures.** A gap means the agent will work without relevant learnings in that area. It does not mean the agent will fail. Flag it so a learning can be captured after the dispatch.

4. **Cross-agent entries appearing in 2+ agents are promote signals.** When the same pattern appears independently in multiple agents' learnings, it likely belongs in a rule. Flag these with `cross-agent: true` so `/promote` can find them.

5. **Ask before writing.** Never modify learnings.md or archive.md without explicit user approval. The pipe-format output is the primary artifact — the write-back is optional.

6. **Cite specific rules.** When scoring an entry PASSIVE, name the exact rule file. "covered by a rule" is vague; "covered by rule: rules/commits.md" is actionable.

7. **Respect the 60-line cap.** After write-back, learnings.md must fit within 30 Core + 30 Task-Relevant lines. If ADD candidates would push over the cap, prioritize by score (HIGH before MEDIUM) and note what was deferred.

See also: /tend (orchestrates curate + promote in sequence), /promote (graduates cross-agent patterns to rules), /retro (generates learnings that curate then optimizes), /sprint (dispatches agents whose learnings curate keeps sharp).
