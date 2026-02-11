# Blossom: Emergent Spike-Driven Exploration

You are running the **Blossom** workflow for this meta-agent-defs project. The user wants to explore: **$ARGUMENTS**

## Overview

Blossom converts a loosely-defined goal into a prioritized backlog through recursive discovery spikes.

```
Seed epic -> spawn spikes -> each spike produces tasks + deeper spikes -> consolidate -> verify -> report
```

## Spike Areas for This Project

When scoping spikes, consider these areas specific to meta-agent-defs:

- **Agent definitions** (`agents/*.md`): Coverage gaps, missing required sections, stale patterns
- **Skill definitions** (`skills/*/SKILL.md`): Missing workflows, unclear phases, broken bd integration
- **Hook configuration** (`settings.json`): Missing hooks, incorrect matchers, graceful failure
- **Installer** (`install.sh`): Edge cases, missing file types, backup handling
- **Cross-cutting concerns**: Consistency between agents/commands, naming conventions, documentation sync

## Process

1. Create an epic bead for the goal
2. Identify 3-6 spike areas from the list above (or discover new ones)
3. Dispatch spike agents to investigate each area (read files, check structure, verify patterns)
4. Create firm task beads from confirmed findings
5. Spawn deeper spikes for unresolved areas
6. Consolidate: dedup, fill gaps, wire dependencies
7. Report final backlog with priorities and recommended execution order

## Rules

- Spikes discover work; they never implement it
- Every finding must state confidence: CONFIRMED / LIKELY / POSSIBLE
- Epic depends on children: `bd dep add <epic> <child>`
- Stop at 20 spikes and reassess with the user if the goal is too broad
- Run `bd sync` before finishing
