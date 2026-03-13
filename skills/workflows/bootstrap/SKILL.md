---
name: bootstrap
description: "Use when starting a new project or adding Claude Code support to an existing one. Sets up CLAUDE.md, hooks, rules, and generates tailored agents. Keywords: bootstrap, scaffold, setup, new project, init, kickstart."
argument-hint: "<project path or description>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git:*), Bash(ls:*), Bash(test:*), Bash(mkdir:*), Task
context: inline
---

# Bootstrap: Full Project Setup Workflow

You are running the **Bootstrap** workflow -- a structured orchestration that converts a bare project into a fully equipped Claude Code workspace. The user wants to bootstrap: **$ARGUMENTS**

## I/O Contract

**Input**: Project path (absolute or `~`-prefixed) plus an optional description of the codebase. Provided via `$ARGUMENTS`.

**Output**: A pipe-format summary listing every artifact created or configured, with a quality status per phase. Downstream skills (`/blossom`, `/sprint`) can consume this to discover what was set up.

**Agent dependencies**:
- `project-bootstrapper` -- must be registered and reachable via the Task tool before running this skill
- `agent-generator` -- same requirement

If either agent is missing, stop in Phase 0 and tell the user to install tackline (`claude plugin install tackline@tacklines`).

## When to Use

- Starting a new project and want full Claude Code infrastructure from day one
- Adding Claude Code support to an existing codebase that has no `.claude/` setup
- Re-bootstrapping a project after major structural changes (existing `.claude/` will be detected and respected)
- Setting up a cloned repository that lacks the local config of the original

## Don't Use When

- The project already has a complete `.claude/` setup that works -- use `/drift` to detect gaps instead
- You only want to generate agents for an already-bootstrapped project -- call `agent-generator` directly
- You want to bootstrap a project in the same repo you are currently in -- both agents write to `$PROJECT_PATH` and assume it is an external directory; writing to the current repo risks corrupting the active session's context

## Overview

Bootstrap runs two agents sequentially. The bootstrapper's output informs the agent generator.

```
Phase 0: Resolve + gate
  -> Phase 1: Infrastructure (project-bootstrapper)
    -> Gate: required files exist and are valid
      -> Phase 2: Agent generation (agent-generator)
        -> Gate: agents have required frontmatter
          -> Phase 3: Verification pass
            -> Phase 4: Report (pipe-format summary)
```

---

## Phase 0: Resolve and Gate

### 0a. Resolve the Target Path

Determine the target project from `$ARGUMENTS`:

1. **If a path is given** (e.g., `/path/to/project` or `~/myproject`): expand `~` to the absolute home path
2. **If a name or topic is given** (e.g., "my new Rust CLI"): ask the user for the target directory path before proceeding
3. **If empty**: ask the user what project to bootstrap and where it lives before proceeding

Store the resolved absolute path as `$PROJECT_PATH`. All subsequent prompts use this value literally.

### 0b. Directory Gate (STOP if failed)

```bash
test -d "$PROJECT_PATH"
```

- **If the directory does not exist**: ask the user whether to create it with `mkdir -p "$PROJECT_PATH"`. If the user declines, stop here.
- **If the directory exists**: proceed.

### 0c. Agent Availability Gate (STOP if failed)

Verify that both required agents are accessible. The fastest check is to confirm the agent definition files are installed:

```bash
ls ~/.claude/agents/project-bootstrapper.md
ls ~/.claude/agents/agent-generator.md
```

If either file is missing, stop and report:

> Required agent `<name>` is not installed. Run `claude plugin install tackline@tacklines` to register it, then retry.

### 0d. Detect Project State

Check whether `.claude/` already exists in `$PROJECT_PATH`:

```bash
test -d "$PROJECT_PATH/.claude" && echo "existing" || echo "new"
```

- **New project**: report "bootstrapping from scratch"
- **Existing setup**: report "existing `.claude/` detected -- agents will read before modifying" and note this in all downstream prompts

---

## Phase 1: Infrastructure Setup

Dispatch the `project-bootstrapper` agent.

```
Task({
  subagent_type: "project-bootstrapper",
  mode: "bypassPermissions",
  prompt: "Bootstrap the project at $PROJECT_PATH.

Your job: Set up everything this project needs for effective Claude Code workflow.

Follow your full phase sequence (discovery, CLAUDE.md, hooks, permissions, rules, memory, skills, gitignore).

Important:
- Read the existing codebase thoroughly before generating any files
- Detect the language, framework, build system, and test framework from lockfiles and config
- Verify tool availability before generating hooks that depend on them
- [If existing setup was detected]: .claude/ already exists -- read every file before making any changes
- Keep CLAUDE.md under 200 lines

When complete, report ALL of the following:
1. Stack detected (language, framework, build system, test framework)
2. Which artifacts were created vs skipped (and why each was skipped)
3. Any tools that were missing and need manual installation
4. File paths of everything you created or modified"
})
```

### Phase 1 Gate (STOP if failed)

After the bootstrapper returns, verify the three required artifacts exist before proceeding to Phase 2:

```bash
test -f "$PROJECT_PATH/.claude/settings.json"
test -f "$PROJECT_PATH/CLAUDE.md"
test -d "$PROJECT_PATH/.claude/rules"
```

**Checks:**
- [ ] `.claude/settings.json` exists
- [ ] `CLAUDE.md` exists
- [ ] `.claude/rules/` directory exists with at least one file

**If any check fails:** Stop and surface the failure to the user. Show the bootstrapper's output and ask whether to retry or proceed anyway. Do not advance to Phase 2 on a broken infrastructure -- the agent generator will misread the project state.

**If all checks pass:** Extract and record the detected stack (language, framework, build system, test framework) from the bootstrapper's report. This is passed to Phase 2.

---

## Phase 2: Agent Generation

Dispatch the `agent-generator` agent. Pass the stack information extracted from Phase 1.

```
Task({
  subagent_type: "agent-generator",
  mode: "bypassPermissions",
  prompt: "Generate project-specific agents for the project at $PROJECT_PATH.

The project was just bootstrapped. Here is what the bootstrapper found:
[paste bootstrapper's stack detection and key findings here -- language, framework, patterns discovered]

Your job: Explore the project, understand its architecture and patterns, and generate a suite of tailored agents in .claude/agents/.

Follow your full phase sequence (discovery, strategy, generation, hooks, catalog, quality check).

Important:
- The project already has CLAUDE.md, hooks, and rules set up -- do not recreate these
- Focus on generating agents that are specific to THIS project's patterns
- Include Investigation Protocol, Context Management, and Knowledge Transfer sections in every agent
- Create .claude/AGENTS.md catalog
- Default to sonnet model unless the task clearly needs opus or haiku

When complete, report ALL of the following:
1. Which agents were generated and why each was chosen
2. Architectural patterns discovered that informed agent design
3. The absolute path to .claude/AGENTS.md
4. Any areas that were unclear and need human clarification"
})
```

### Phase 2 Gate (STOP if failed)

After the agent generator returns, verify agent files were created and are structurally valid:

```bash
ls "$PROJECT_PATH/.claude/agents/"*.md 2>/dev/null
test -f "$PROJECT_PATH/.claude/AGENTS.md"
```

**Checks (for each agent file):**
- [ ] `.claude/agents/` contains at least one `.md` file
- [ ] `.claude/AGENTS.md` catalog was created
- [ ] Each agent file has `name:` in its frontmatter
- [ ] Each agent file has `description:` in its frontmatter
- [ ] Each agent file has `tools:` in its frontmatter
- [ ] Each agent file has `model:` in its frontmatter

To verify frontmatter fields, grep each file:

```bash
grep -l "^name:" "$PROJECT_PATH/.claude/agents/"*.md
grep -l "^description:" "$PROJECT_PATH/.claude/agents/"*.md
grep -l "^tools:" "$PROJECT_PATH/.claude/agents/"*.md
grep -l "^model:" "$PROJECT_PATH/.claude/agents/"*.md
```

If any agent is missing required frontmatter fields, list the offending files and surface them to the user -- do not silently proceed with malformed agents.

---

## Phase 3: Verification Pass

Run verification checks on the complete setup. These are observable, external checks -- not self-reported by the agents.

### 3a. Hooks Executable

```bash
find "$PROJECT_PATH/.claude" -name "*.sh" -exec test -x {} \; -print
```

Any `.sh` files that are not executable should be listed. Run `chmod +x` on them or note them as a manual step.

### 3b. Settings JSON Valid

```bash
python3 -m json.tool "$PROJECT_PATH/.claude/settings.json" > /dev/null 2>&1 && echo "valid" || echo "invalid"
```

Or if Python is unavailable:

```bash
cat "$PROJECT_PATH/.claude/settings.json" | grep -c "hooks"
```

If `settings.json` is invalid JSON, report it and stop -- broken settings will silently disable all hooks.

### 3c. Verification Summary

Collect all check results into a pass/fail table for the Phase 4 report:

| Check | Result |
|-------|--------|
| `.claude/settings.json` exists | pass/fail |
| `CLAUDE.md` exists | pass/fail |
| `.claude/rules/` populated | pass/fail |
| `.claude/agents/` populated | pass/fail |
| `.claude/AGENTS.md` created | pass/fail |
| All agents have required frontmatter | pass/fail |
| Hook scripts executable | pass/fail |
| `settings.json` is valid JSON | pass/fail |

---

## Phase 4: Report

### 4a. Emit Pipe-Format Summary

```markdown
## Bootstrap Report: [project name]

**Source**: /bootstrap
**Input**: $ARGUMENTS
**Pipeline**: (none -- working from direct input)

### Items (N)

[One item per artifact created or configured. Format:]

1. **CLAUDE.md** -- project root orientation file, [line count] lines
   - source: $PROJECT_PATH/CLAUDE.md
   - confidence: CONFIRMED

2. **.claude/settings.json** -- hooks configuration
   - source: $PROJECT_PATH/.claude/settings.json
   - confidence: CONFIRMED

3. **.claude/rules/commits.md** -- commit message convention rule
   - source: $PROJECT_PATH/.claude/rules/commits.md
   - confidence: CONFIRMED

[... one item per file or directory created]

N. **[agent-name] agent** -- [purpose in one line]
   - source: $PROJECT_PATH/.claude/agents/[agent-name].md
   - confidence: CONFIRMED

### Verification Results

| Check | Result |
|-------|--------|
[paste verification table from Phase 3d]

### Manual Steps Required

[List any tools that were missing and need manual installation. "None" if clean.]

### Summary

[One paragraph: project type detected, what was set up, how many agents were generated, any issues to resolve.]
```

### 4b. Present to User

After the pipe-format block, present a human-readable next steps section:

```markdown
---

### Next Steps

1. **Review CLAUDE.md** -- adjust project description, commands, and key patterns to your preferences
2. **Review generated agents** -- read `.claude/AGENTS.md` for the full catalog
3. **Resolve manual steps** -- install any missing tools flagged above
4. **Orient the session** -- run `/status` to start working
```

---

## Completion Criteria

Bootstrap is complete when:

- [ ] All Phase 0 gates passed (directory exists, agents available)
- [ ] All Phase 1 gate checks pass (settings.json, CLAUDE.md, rules/)
- [ ] All Phase 2 gate checks pass (agents created with required frontmatter)
- [ ] Phase 3 verification table has no `fail` entries, or all failures are documented and presented to the user
- [ ] Pipe-format summary has been emitted
- [ ] Next steps have been presented

If any gate produced a hard stop that was not resolved, bootstrap is incomplete. The pipe-format summary should reflect the actual state reached, not an assumed success.

---

## Key Principles

1. **Sequential, not parallel.** The agent generator needs the bootstrapper's output to avoid recreating infrastructure and to understand the detected stack.
2. **Gates are mandatory, not advisory.** Each phase gate is a hard stop. Do not proceed past a failed gate without explicit user confirmation.
3. **Pass context forward.** Include the bootstrapper's findings in the agent generator's prompt so it doesn't re-discover what's already known.
4. **Surface failures explicitly.** If the bootstrapper fails, ask the user before proceeding rather than running the agent generator on a broken setup.
5. **Respect existing setup.** Both agents check for existing `.claude/` configuration and avoid overwriting intentional customizations.
6. **No worktree isolation.** Worktree isolation creates an isolated copy of the repo where the skill is invoked, not the target project. Both dispatched agents write to `$PROJECT_PATH` -- an external directory -- so worktree isolation would protect the wrong repo. Cross-project bootstrap writes cannot be sandboxed by worktrees.
7. **Compaction resilience.** Per `rules/memory-layout.md`, checkpoint at phase boundaries to `.claude/tackline/memory/scratch/bootstrap-checkpoint.md`.

---

## See Also

- `/assemble` -- create a persistent learning team after bootstrapping
- `/blossom` -- explore the newly set-up project to discover work and generate initial backlog
- `/sprint` -- execute the initial backlog items created during bootstrap
- `/drift` -- detect gaps in an already-bootstrapped project's Claude Code setup
