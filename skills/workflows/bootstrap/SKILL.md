---
name: bootstrap
description: "Bootstrap a new project end-to-end: set up Claude Code infrastructure (CLAUDE.md, hooks, beads, rules, skills) then generate tailored project agents. Use when starting a new project from scratch or adding full Claude Code support to an existing codebase. Keywords: bootstrap, scaffold, setup, new project, init, kickstart."
argument-hint: "<project path or description>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*), Task
context: fork
---

# Bootstrap: Full Project Setup Workflow

You are running the **Bootstrap** workflow -- a two-phase orchestration that converts a bare project into a fully equipped Claude Code workspace. The user wants to bootstrap: **$ARGUMENTS**

## When to Use

- Starting a new project and want full Claude Code infrastructure from day one
- Adding Claude Code support to an existing codebase that has no `.claude/` setup
- Re-bootstrapping a project that needs a fresh setup (existing `.claude/` will be detected and respected)

## Overview

Bootstrap runs two agents sequentially. The bootstrapper's output informs the agent generator.

```
Resolve project path
  -> dispatch project-bootstrapper (CLAUDE.md, hooks, beads, rules, skills, memory)
    -> review bootstrapper output
      -> dispatch agent-generator (project-specific agents + catalog)
        -> report final setup summary
```

---

## Phase 0: Resolve the Project

Determine the target project from `$ARGUMENTS`:

1. **If a path is given** (e.g., `/path/to/project` or `~/myproject`): verify it exists
2. **If a name/topic is given** (e.g., "my new Rust CLI"): ask the user for the target directory path
3. **If empty**: ask the user what project to bootstrap and where it lives

Confirm the path exists before proceeding. If the directory doesn't exist, ask the user whether to create it.

Store the resolved absolute path as `$PROJECT_PATH` for use in agent prompts.

---

## Phase 1: Run Project Bootstrapper

Dispatch the project-bootstrapper agent to set up the project infrastructure.

```
Task({
  subagent_type: "project-bootstrapper",
  mode: "bypassPermissions",
  prompt: "Bootstrap the project at $PROJECT_PATH.

Your job: Set up everything this project needs for effective Claude Code + Beads workflow.

Follow your full phase sequence (discovery, beads init, CLAUDE.md, hooks, permissions, rules, memory, skills, gitignore, initial beads).

Important:
- Read the existing codebase thoroughly before generating any files
- Detect the language, framework, build system, and test framework from lockfiles and config
- Verify tool availability before generating hooks that depend on them
- If .claude/ already exists, read every file before making changes
- Keep CLAUDE.md under 200 lines

When complete, report:
1. Stack detected (language, framework, build system, test framework)
2. Which artifacts were created vs skipped (and why)
3. Any tools that were missing and need manual installation
4. File paths of everything you created"
})
```

### Review Bootstrapper Output

Before proceeding, verify the bootstrapper succeeded:

- [ ] `.claude/settings.json` was created with hooks
- [ ] `CLAUDE.md` was created with project-specific content
- [ ] `.claude/rules/` has at least `commits.md`

If the bootstrapper reported failures (missing tools, permission errors), surface them to the user and ask whether to proceed to agent generation or stop.

---

## Phase 2: Run Agent Generator

Dispatch the agent-generator agent to create project-specific agents. Pass along the stack information from the bootstrapper's report.

```
Task({
  subagent_type: "agent-generator",
  mode: "bypassPermissions",
  prompt: "Generate project-specific agents for the project at $PROJECT_PATH.

The project was just bootstrapped. Here is what the bootstrapper found:
[paste bootstrapper's stack detection and key findings]

Your job: Explore the project, understand its architecture and patterns, and generate a suite of tailored agents in .claude/agents/.

Follow your full phase sequence (discovery, beads integration, strategy, generation, hooks, catalog, quality check).

Important:
- The project already has CLAUDE.md, hooks, rules, and beads set up -- do not recreate these
- Focus on generating agents that are specific to THIS project's patterns
- Include Investigation Protocol, Context Management, and Knowledge Transfer sections in every agent
- Create .claude/AGENTS.md catalog
- Default to sonnet model unless the task clearly needs opus or haiku

When complete, report:
1. Which agents were generated and why
2. Architectural patterns discovered
3. The agent catalog location
4. Any areas that were unclear and need human clarification"
})
```

### Review Agent Generator Output

Verify agents were created properly:

- [ ] `.claude/agents/` directory exists with agent files
- [ ] `.claude/AGENTS.md` catalog was created
- [ ] Each agent has proper frontmatter (name, description, tools, model)
- [ ] Each agent includes the three required sections

---

## Phase 3: Report

Present the final bootstrap summary to the user:

```markdown
## Project bootstrapped: [project name]

### Infrastructure (Phase 1)
- **Stack**: [language, framework, build system, test framework]
- **Created**: [list of artifacts: CLAUDE.md, hooks, rules, skills, beads, memory]
- **Skipped**: [anything skipped and why]
- **Manual steps needed**: [any tools to install, etc.]

### Agents (Phase 2)
- **Generated**: [count] agents in `.claude/agents/`
- **Catalog**: `.claude/AGENTS.md`
- [table of agents with name, purpose, model]

### Next Steps
1. Review `CLAUDE.md` and adjust to your preferences
2. Review generated agents in `.claude/agents/`
3. Run `bd ready` to see initial backlog items
4. Start working with `/status` to orient
```

---

## Key Principles

1. **Sequential, not parallel.** The agent generator needs the bootstrapper's output to avoid recreating infrastructure and to understand the detected stack.
2. **Pass context forward.** Include the bootstrapper's findings in the agent generator's prompt so it doesn't re-discover what's already known.
3. **Surface failures early.** If the bootstrapper fails, ask the user before proceeding rather than running the agent generator on a broken setup.
4. **Respect existing setup.** Both agents check for existing `.claude/` configuration and avoid overwriting intentional customizations.

See also: /assemble (create a persistent learning team after bootstrapping). /blossom (explore the newly set-up project to discover work and generate initial backlog). /sprint (execute the initial backlog items created during bootstrap).
