---
name: curate
description: "Optimize an agent's learnings or audit project rules: score each entry by relevance, archive stale or redundant entries, detect gaps where upcoming work needs knowledge not yet captured. For learnings: pull in relevant entries from archive or cross-agent sources. For rules: assess passive context budget and flag redundancy. Use before sprints, after /retro, when learnings feel bloated, or when rules need a health check. Keywords: curate, learnings, rules, optimize, relevance, prune, archive, gap, score, triage, audit."
argument-hint: "<agent-name> | rules"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Bash(ls:*), Write, Edit
---

# Curate: Optimize Agent Learnings or Audit Project Rules

You are running **curate** — scoring entries by relevance to upcoming tasks, flagging stale or redundant items, and detecting gaps. Works in two modes:

- **Learnings mode** (`/curate <agent-name>`): optimize an agent's learnings, archive stale entries, fill gaps from archive or cross-agent sources
- **Rules mode** (`/curate rules`): audit project rule files for relevance, redundancy, and passive context budget

**Target:** $ARGUMENTS

## When to Use

- Before a sprint — ensure the agent's learnings are tuned to what's coming
- After `/retro` adds new entries — prune stale entries to stay under the 60-line cap
- When `/tend` orchestrates the full lifecycle — curate runs first, promote reads the output
- When learnings feel bloated, stale, or out of sync with current work
- When an agent will be assigned tasks in a new domain area
- When rules feel redundant, bloated, or misaligned with current work
- When passive context budget feels too large — audit total rule line count

## How It Works

```
Detect artifact type (learnings or rules)
  -> Load entries + upcoming work + cross-references
    -> Score each entry (HIGH / MEDIUM / LOW / PASSIVE)
      -> Detect gaps (upcoming work domains vs. coverage)
        -> [learnings] Pull candidates from archive or cross-agent sources
        -> [rules] Compute passive context budget
          -> Emit pipe-format report
            -> [learnings] (Optional) Write back to learnings.md and archive.md
            -> [rules] Write review manifest to memory/scratch/
```

---

## Phase 0: Detect Artifact Type and Validate

### 0a. Artifact Type Detection

If `$ARGUMENTS` equals `rules` (case-insensitive):
- Set **artifact type = rules**
- Proceed to validation step 0c

Otherwise:
- Set **artifact type = learnings**
- Proceed to validation step 0b

### 0b. Validate Agent (learnings mode)

If `$ARGUMENTS` is empty:
1. Run `ls memory/agents/` to list available agents
2. Ask: "Which agent should I curate? Available: [names]"
3. Stop and wait for response

If `$ARGUMENTS` is provided:
1. Confirm `memory/agents/<name>/learnings.md` exists
2. If not found: "No learnings file at memory/agents/<name>/learnings.md. Has this agent been dispatched yet?"
3. Note whether `memory/agents/<name>/archive.md` exists (needed for Phase 4)
4. Note whether `.claude/team.yaml` exists (used in Phase 1d to read the agent definition)

### 0c. Validate Rules (rules mode)

1. Check for rule files:
   ```bash
   ls rules/*.md .claude/rules/*.md 2>/dev/null
   ```
2. If no rule files found: "No rule files found in rules/ or .claude/rules/. Nothing to curate."
3. Count total rule files and note the count

---

## Phase 1: Load Context

### 1a. Load Primary Artifact

**Learnings mode:** Read `memory/agents/<name>/learnings.md`. Note:
- Total line count (cap is 60 lines: 30 Core + 30 Task-Relevant)
- Each distinct entry (entries separated by blank lines or bullet markers)
- Entry provenance where present: `(added: YYYY-MM-DD, dispatch: <source>)`

**Rules mode:** Read all rule files from both locations:
```bash
ls rules/*.md .claude/rules/*.md 2>/dev/null
```
For each rule file, note:
- Filename and path (global `rules/` vs project-local `.claude/rules/`)
- Title (first `#` heading)
- Line count
- Topics covered (key concepts, patterns, or constraints defined)

Also read `CLAUDE.md` and note any overlap between CLAUDE.md content and rule files.

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

### 1c. Cross-Reference Loading

**Learnings mode:** Read all rule files to detect overlap with learnings:

```bash
ls rules/*.md .claude/rules/*.md 2>/dev/null
```

Read each rule file. Read `CLAUDE.md`. Build a list of topics already covered passively so learnings that duplicate them can be flagged as PASSIVE.

**Rules mode:** Read agent learnings to find PASSIVE rules (rules whose content has been internalized by agents):

```bash
ls memory/agents/*/learnings.md 2>/dev/null
```

For each rule, check whether 3+ agents have learnings entries that restate the rule's constraints. A rule internalized by 3+ agents is a candidate for PASSIVE — the knowledge is already distributed and the rule may be adding redundant passive context cost.

### 1d. Agent Definition (learnings mode only)

**Skip this step in rules mode.**

If `.claude/team.yaml` exists, read it and extract the target agent's entry:
- `role`: what the agent is responsible for
- `owns`: file glob patterns defining ownership
- `model`: complexity context

If team.yaml does not exist, skip this step.

---

## Phase 2: Score Each Entry

For each entry in the primary artifact, assign one of four scores. The scoring table depends on artifact type.

### Learnings Mode Scoring

For each entry in `learnings.md`:

| Score | Meaning | Action |
|-------|---------|--------|
| HIGH | Directly relates to files, domains, or tools in upcoming tasks | Keep |
| MEDIUM | Related to the general area of upcoming work but not specific tasks | Keep |
| LOW | No connection to upcoming work — relevant in general but not now | Archive |
| PASSIVE | Already covered by an existing rule or CLAUDE.md (redundant) | Archive with note |

**Scoring Heuristics (learnings):**

**HIGH:** The entry mentions a specific file, tool, command, or pattern that appears in upcoming task descriptions or epic state. Example: an entry about `bd create --parent` syntax is HIGH if epic management tasks are coming.

**MEDIUM:** The entry covers a category of work that's coming but doesn't match specific files. Example: entries about skill authoring conventions are MEDIUM when new skills are on the backlog.

**LOW:** The entry is about a past domain area no longer in scope, or is general background that won't be needed soon. Example: entries about a feature area with no upcoming beads and no recent commits.

**PASSIVE:** The entry states something already enforced by a rule in `rules/` or `CLAUDE.md`. Example: an entry "use conventional commits" is PASSIVE if `rules/commits.md` already covers this. When marking PASSIVE, cite the specific rule file.

### Rules Mode Scoring

For each rule file:

| Score | Meaning | Action |
|-------|---------|--------|
| HIGH | Covers domains in upcoming work or addresses known failure modes | Keep |
| MEDIUM | Broadly useful across sessions, not specific to upcoming work | Keep |
| LOW | No upcoming work in the covered domain, no recent git activity in covered files | Review |
| PASSIVE | Duplicated by another rule, or internalized in 3+ agents' learnings | Review |

**Scoring Heuristics (rules):**

**HIGH:** The rule covers a domain area that upcoming tasks will operate in, or addresses a failure mode that has been observed recently. Example: `rules/pipe-format.md` is HIGH when composable primitives are being written or revised.

**MEDIUM:** The rule provides useful guardrails across many session types but isn't directly tied to upcoming work. Example: `rules/batch-safety.md` is MEDIUM when no batch operations are planned but the constraint prevents real damage.

**LOW:** The rule covers a domain with no upcoming beads, no in-progress work, and no recent git activity (last 30 days) in the files it governs. Example: a rule about database migrations when no migration work is planned or recent.

**PASSIVE:** The rule's constraints are either (a) duplicated by another rule file or CLAUDE.md content, or (b) internalized by 3+ agents in their learnings, making the passive context cost redundant. When marking PASSIVE, cite the duplicating source or the agents that have internalized it.

---

## Phase 3: Detect Gaps

Compare the file scope and domain areas from Phase 1b against the topics covered by entries scored HIGH or MEDIUM.

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
2. No HIGH or MEDIUM entry covers it

**Learnings mode:** Gaps mean the agent will operate in this area without relevant learnings loaded.

**Rules mode:** Gaps mean upcoming work domains lack guardrail rules. Check whether agent learnings in gap areas have promote potential (same pattern in 2+ agents) — these are rule candidates surfaced through gap analysis.

### 3c. Check Archive for Gap Fills (learnings mode only)

**Skip this step in rules mode.** Rules are not archived — they are reviewed and manually maintained.

If `memory/agents/<name>/archive.md` exists, read it. Scan for entries that match gap areas. These are **archive candidates** — entries that were previously archived but are now relevant again.

### 3d. Check Cross-Agent Sources (learnings mode only)

**Skip this step in rules mode.**

Scan other agents' learnings files for entries relevant to the gap areas:

```bash
ls memory/agents/*/learnings.md 2>/dev/null
```

Read each file that is not the target agent's. For each gap, check whether another agent has an entry covering it. Cross-agent candidates are flagged for potential pull-in. Note the source agent name — this is also relevant for `/promote` (patterns across agents indicate rule candidacy).

---

## Phase 4: Compose Output

Emit in pipe format per `rules/pipe-format.md`. The format depends on artifact type.

### Learnings Mode Output

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

### Rules Mode Output

```markdown
## Curated Rules: Project Rule Health

**Source**: /curate
**Input**: rules
**Pipeline**: (none — working from direct input)

### Items (N)

1. **KEEP (HIGH): <rule title>** — <one-line summary of what the rule constrains>
   - score: HIGH
   - reason: <covers upcoming work domain or known failure mode>
   - source: <rules/filename.md or .claude/rules/filename.md>
   - lines: <line count>

2. **KEEP (MEDIUM): <rule title>** — <summary>
   - score: MEDIUM
   - reason: <broadly useful, not specific to upcoming work>
   - source: <path>
   - lines: <line count>

3. **REVIEW (LOW): <rule title>** — <summary>
   - score: LOW
   - reason: <no upcoming tasks in covered domain, no recent git activity>
   - source: <path>
   - lines: <line count>

4. **REVIEW (PASSIVE): <rule title>** — <summary>
   - score: PASSIVE
   - reason: <duplicated by: rule/file or internalized by: agent1, agent2, agent3>
   - source: <path>
   - lines: <line count>

### Gaps (M)

For each gap:

- **GAP: <domain area>** — upcoming tasks touch <files/patterns> but no rule provides guardrails. [If agent learnings in this area appear in 2+ agents: "Promote candidate: pattern X found in agent1, agent2."]

### Passive Context Budget

| Metric | Value |
|--------|-------|
| Total rule files | <count> |
| Total lines (all rules) | <sum of all rule file line counts> |
| HIGH + MEDIUM lines | <sum> |
| LOW + PASSIVE lines | <sum> |
| Potential savings | <LOW + PASSIVE line count> lines across <count> files |

### Summary

<One paragraph: how many rules scored HIGH/MEDIUM/LOW/PASSIVE, total passive context load in lines, potential savings from reviewing LOW+PASSIVE rules, any gaps where upcoming work lacks guardrails, and whether any gaps have promote candidates from agent learnings.>
```

Order items: KEEP (HIGH before MEDIUM), then REVIEW (LOW before PASSIVE). Rules mode uses REVIEW instead of ARCHIVE — rules are flagged for human review, never auto-removed.

---

## Phase 5: Write-Back (Conditional)

### Learnings Mode Write-Back

After presenting the output, ask the user:

> "Apply these changes to learnings.md and archive.md? (y/n)"

If the user approves:

**5a. Update learnings.md**

Write the updated file with:
- All KEEP (HIGH) and KEEP (MEDIUM) entries retained
- All ADD entries inserted (from archive or cross-agent sources)
- All ARCHIVE entries removed
- Entries organized into Core (30 lines max) and Task-Relevant (30 lines max) sections
- Total must not exceed 60 lines

When re-organizing:
- **Core**: high-reuse fundamentals that apply across most tasks for this agent
- **Task-Relevant**: entries specific to current upcoming work scope

**5b. Update archive.md**

Append each ARCHIVE entry to `memory/agents/<name>/archive.md`:

```markdown
- <entry text> (archived: YYYY-MM-DD, reason: LOW — no upcoming tasks in this area)
- <entry text> (archived: YYYY-MM-DD, reason: PASSIVE — covered by rule: rules/commits.md)
```

If `archive.md` does not exist, create it with a header line:

```markdown
# Archived Learnings: <agent-name>
```

**5c. Verify**

Read the updated `learnings.md` and confirm:
- Line count is within the 60-line cap
- All sections are properly structured
- No ARCHIVE entries remain in learnings.md

### Rules Mode Write-Back (Report Only)

**Never auto-delete or auto-modify rule files.** Rules mode is report-only.

Write a review manifest to `memory/scratch/rules-review.md`:

```markdown
# Rules Review Manifest

Generated by /curate rules on YYYY-MM-DD

## Review Checklist

- [ ] **REVIEW (LOW): <rule title>** (<path>, <lines> lines) — <reason>
- [ ] **REVIEW (PASSIVE): <rule title>** (<path>, <lines> lines) — <reason>
...

## Gaps to Address

- [ ] **GAP: <domain area>** — <description> [promote candidate: <yes/no>]
...

## Passive Context Budget

Total: <lines> lines across <count> files
Potential savings: <lines> lines if LOW+PASSIVE rules are consolidated or removed
```

This manifest is a checklist for human review, not an automation target.

---

## Guidelines

1. **Score against upcoming work, not general value.** A learning can be deeply true and still score LOW because nothing in the upcoming sprint will trigger it. That's fine — archive it and re-evaluate after the next sprint.

2. **PASSIVE is not wrong — it's covered.** Marking an entry PASSIVE is a compliment: the pattern graduated to a rule. Archive it without judgment.

3. **Gaps are diagnostic, not failures.** A gap means the agent will work without relevant learnings in that area. It does not mean the agent will fail. Flag it so a learning can be captured after the dispatch.

4. **Cross-agent entries appearing in 2+ agents are promote signals.** When the same pattern appears independently in multiple agents' learnings, it likely belongs in a rule. Flag these with `cross-agent: true` so `/promote` can find them.

5. **Ask before writing.** Never modify learnings.md or archive.md without explicit user approval. The pipe-format output is the primary artifact — the write-back is optional.

6. **Cite specific rules.** When scoring an entry PASSIVE, name the exact rule file. "covered by a rule" is vague; "covered by rule: rules/commits.md" is actionable.

7. **Respect the 60-line cap.** After write-back, learnings.md must fit within 30 Core + 30 Task-Relevant lines. If ADD candidates would push over the cap, prioritize by score (HIGH before MEDIUM) and note what was deferred.

8. **Rules mode is conservative.** Flag rules for review, never delete or modify them automatically. REVIEW replaces ARCHIVE — rules are human-maintained artifacts with higher change cost than learnings.

9. **PASSIVE for rules means dedup or internalization.** A rule is PASSIVE when its constraints are either restated by another rule file (dedup) or when 3+ agents have internalized the pattern in their learnings (the knowledge is distributed). Both reduce the value of the passive context cost.

10. **Compose with /evolution for richer rule scoring.** If a rule's churn or stability is unclear, run `/evolution <rule-file>` first. The stability signal from /evolution improves LOW vs MEDIUM scoring accuracy.

See also: /tend (orchestrates curate + promote in sequence), /promote (graduates cross-agent patterns to rules), /retro (generates learnings that curate then optimizes), /sprint (dispatches agents whose learnings curate keeps sharp), /evolution (file change history for richer rule scoring).
