# Learnings: skill-author

## Codebase Patterns
- Skills live in skills/<name>/SKILL.md with YAML frontmatter (name, description, allowed-tools, context)
- 28 skills: 15 workflow + 12 composable primitives + /status (gather, distill, rank, diff-ideas, sketch, verify, decompose, filter, assess, critique, plan, merge)
- Skills with `context: fork` run in isolation (blossom, consolidate, review); others run inline
- All skills use `disable-model-invocation: false` — the `true` setting blocks the Skill tool entirely
- `$ARGUMENTS` is how skills receive user input from the slash command
- 6 fully compliant reference templates: consensus, premortem, retro, review, spec, tracer (added: 2026-02-13)
- Skills that dispatch agents (blossom, consensus, premortem, spec) correctly follow fan-out-protocol.md (added: 2026-02-13)
- Fan-out-protocol.md has Agent Preamble section — skills reference it instead of repeating investigation boilerplate (added: 2026-02-14)
- Characterization-over-procedure pattern: 2-3 sentence "You think like..." characterization outperforms procedural step lists for agent prompts (added: 2026-02-14)
- Consensus uses adaptive 8-lens palette with selection logic; parameterized agent template instead of hardcoded 3 agents (added: 2026-02-14)
- Blossom has pushback protocol in quality gate — re-prompts teammates (team mode) or flags for re-dispatch (background mode) (added: 2026-02-14)

## Gotchas
- Skills cannot be invoked by subagents (Skill tool not available to them)
- `allowed-tools` uses `Bash(prefix:*)` syntax to restrict shell commands, not full command strings
- RESOLVED: user-invocable and "When to Use" gaps were fixed in commit 1fcc5d3 (2026-02-13)
- Skills can reference allowed-tools that might not be available at runtime; conditional logic in the body handles optional deps, not frontmatter (added: 2026-02-13)
- Optional dependency pattern: `**If .beads/ exists**, [action]. **If not**, [fallback].` — preserves structure while enabling graceful degradation (added: 2026-02-13)

## Pipeline Provenance
- All primitives use two-touch pattern: detect upstream Pipeline field during input, emit extended Pipeline during output (added: 2026-02-14)
- Merge uses `+` notation for showing merged branches: `/gather (8) + /gather (6) -> /merge (10)` (added: 2026-02-14)
- Pipeline line is mandatory in pipe format — `(none — working from direct input)` when no upstream (added: 2026-02-14)

## Composition Patterns
- Gather is almost always first — other primitives need structured items to work on (added: 2026-02-13)
- Assess categorizes (discrete rubric), rank orders (continuous score) — different tools for different needs (added: 2026-02-13)
- Verify belongs at chain ends for confirmation, not at the start for discovery (added: 2026-02-13)
- Decompose bridges analysis and execution — it's the step before task creation / agent dispatch (added: 2026-02-13)
- Cookbook examples must be grounded in real repo content — verify patterns exist before writing examples (added: 2026-02-13)
- 3 deep recipes beat 5 shallow — annotations need 120+ lines per recipe for full context (added: 2026-02-13)
- Demo projects need intentional issues subtle enough to be interesting but obvious enough for primitives to find (added: 2026-02-13)
- Merge is the only primitive that reads MULTIPLE pipe-format blocks from context; others read one (added: 2026-02-13)

## Preferences
- Interactive mode pattern: check if $ARGUMENTS is empty in Phase 0, fork to conversational flow that gathers info then rejoins main workflow at a later phase (added: 2026-02-13)

## Cross-Agent Notes
- Team templates in templates/teams/ follow exact .claude/team.yaml schema; use realistic ownership globs per project type (triaged: 2026-02-14)
