---
paths:
  - "skills/**/*.md"
strength: should
freshness: 2026-02-21
---

# Skill Authoring Rules

Rules for writing and editing skill definition files.

## File Location

Skills live in `skills/<name>/SKILL.md`. The directory name becomes the slash command name (e.g., `skills/blossom/SKILL.md` -> `/blossom`).

## Required Frontmatter

Every skill file must have YAML frontmatter with these fields:

```yaml
---
name: lowercase-with-hyphens
description: "When and why to use this skill. Include keywords for auto-discovery."
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git:*)
---
```

### Frontmatter Fields

- **name**: Matches the directory name, lowercase with hyphens
- **description**: Quoted string explaining when to invoke. Include keywords at the end for auto-discovery by Claude
- **argument-hint**: Optional. Shows usage hint (e.g., `"<goal>"`, `"[target]"`)
- **disable-model-invocation**: Default `false` (auto-discoverable). Only set `true` if auto-invocation would be genuinely dangerous
- **user-invocable**: Set `true` so users can invoke via `/name`
- **allowed-tools**: Minimal tool list. Use `Bash(prefix:*)` syntax to restrict shell commands
- **context**: Set `fork` for skills that run in isolation (heavy exploration). Omit for inline skills

## Required Sections

Every skill MUST include:

1. **Title line** -- `# Name: Descriptive Subtitle`
2. **Context line** -- Reference `$ARGUMENTS` for user input
3. **When to Use** -- Bullet list of concrete scenarios
4. **Phased workflow** -- Numbered phases with clear progression
5. **Guidelines** -- Behavioral constraints and quality standards

## Do This

- Start the description with an action verb ("Run...", "Capture...", "Review...")
- Include keywords at the end of the description for auto-discovery
- Restrict `allowed-tools` to the minimum needed -- read-only skills should not have Write/Edit
- Use `context: fork` for skills that do heavy exploration to avoid polluting the main context
- Reference `rules/information-architecture.md` for placement decisions when choosing between passive (rules/), discoverable (docs/), or on-demand locations
- Reference `$ARGUMENTS` to accept user input
- Include a visual workflow diagram showing phase progression (ASCII flowchart)

## Iterative Mode (Optional)

Some skills support multi-turn follow-up: the user runs the skill once, reviews findings, then drills into a specific item without re-running the expensive gather phase. Add this section only when the skill has a distinct gather/explore phase whose output a user would plausibly want to investigate further.

### When to Include It

Include Iterative Mode when the skill:
- Has a gather or explore phase that is expensive (many tool calls, external reads, or agent dispatch)
- Produces a ranked or enumerated findings list the user may want to narrow
- Can meaningfully answer a follow-up question using only the already-gathered output, without re-running Phase 1

### Skill Body Template

Add an **Iterative Mode** section after the main workflow phases:

```markdown
## Iterative Mode

After the skill completes, users can resume to drill into a specific finding
without re-running the gather phase.

**How to resume:**
> /[skill-name] resume: <finding-id or follow-up question>

Or via the Task tool:
> Task(skill: [skill-name], resume: "<finding-id>: <follow-up query>")

**On resume, the skill should:**
1. Detect the `resume:` prefix in `$ARGUMENTS`
2. Skip Phase 1 (gather/explore) entirely
3. Re-read any output written to `memory/scratch/[skill-name]-output.md`
   (written at end of Phase 1 to survive context loss)
4. Apply the follow-up query to the already-gathered findings
5. Produce a focused output for the selected finding only
```

### State Preservation Convention

- At the end of the gather phase, write the full findings list to
  `memory/scratch/<skill-name>-output.md` so a resumed invocation can reload it
- Preserve: ranked findings, source file paths, confidence levels, and any decisions made
- Re-derive on resume: summaries, recommendations, and formatted output (cheap to regenerate)
- Delete the scratch file on session end or when the user explicitly closes the thread

### Output Advertising

When the skill completes its first run and iterative mode is supported, close with:

```
Resume available — to drill into finding N:
  /[skill-name] resume: N <your question>
```

## Don't Do This

- Do not include Write/Edit in allowed-tools unless the skill modifies files
- Do not set `disable-model-invocation: true` — it blocks the Skill tool and prevents programmatic invocation. All skills default to `false`
- Do not duplicate logic that exists in another skill -- factor shared patterns into agents instead
- Do not duplicate logic that exists in an agent -- if an agent already handles the workflow, don't recreate it as a skill
- Do not embed session-close boilerplate (git status, git commit) -- the SessionStart hook already injects the session close protocol into every session
