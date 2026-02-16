# Learnings: skill-author

## Codebase Patterns
- Skills live in skills/<name>/SKILL.md with YAML frontmatter (name, description, allowed-tools, context)
- 32 skills: 20 workflow + 12 composable primitives. `context: fork` for isolation (blossom, consolidate, review); all use `disable-model-invocation: false`
- `$ARGUMENTS` is how skills receive user input from the slash command
- Skills that dispatch agents (blossom, consensus, premortem, spec) follow fan-out-protocol.md with Agent Preamble (added: 2026-02-14)
- Characterization-over-procedure: "You think like..." (2-3 sentences) outperforms procedural step lists for agent prompts (added: 2026-02-14)
- Consensus: adaptive 8-lens palette, optional debate round (Phase 2.5) with anonymized cross-proposal challenge (added: 2026-02-14)
- Anonymization in cross-proposal patterns: neutral labels (A/B/C), remove lens-identifying language (added: 2026-02-14)
- Blossom has pushback protocol in quality gate — re-prompts or flags for re-dispatch (added: 2026-02-14)

## Gotchas
- Skills cannot be invoked by subagents (Skill tool not available to them)
- `allowed-tools` uses `Bash(prefix:*)` syntax to restrict shell commands, not full command strings
- Optional deps handled in skill body, not frontmatter: `**If .beads/ exists**, [action]. **If not**, [fallback].` (added: 2026-02-13)

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
- Checkpoints in recursive skills: specify trigger precisely (depth transitions vs every iteration), default to non-blocking (safety net, not gate) (added: 2026-02-14)
- When borrowing patterns across skills, adapt semantics to target skill's constraints (e.g., fractal's immutable goal means "pivot" can't change the goal, only the exploration path) (added: 2026-02-14)

## Tool Permission Patterns
- Shell scripts in bin/ need explicit path permissions: `Bash(bin/git-pulse.sh:*)` — prefix matching like `Bash(git:*)` only works for actual shell commands (added: 2026-02-14)
- Same external tool serves different observability needs via time window: `--since="8 hours ago"` for session-scoped /retro, `--since="7 days ago"` for weekly /status (added: 2026-02-14)
- git-intel binary permission: `Bash(tools/git-intel/target/release/git-intel:*)` — skills that use git-intel need this in allowed-tools (added: 2026-02-14)

## Active Learning System
- Sprint skill now includes Agent: commit trailer for per-agent provenance tracking (added: 2026-02-16, dispatch: bead-ndsx)
- Learnings entries support optional `(dispatch: bead-xyz)` provenance field for tracing learnings to specific tasks (added: 2026-02-16, dispatch: bead-do61)
- /diagnose-agent: read-only analysis skill, outputs struggle profile in pipe format (WEAKNESS→GAP→STRENGTH ordering for downstream consumption) (added: 2026-02-16, dispatch: bead-57ya)
- When dispatching literal computed values to agents, explicitly state "use this value as-is" — agents tend to try recomputing things (added: 2026-02-16, dispatch: bead-ndsx)

## git-intel Integration
- Extending existing skills with git-intel: add conditional sections that check for binary, use JSON output, graceful fallback to raw git (added: 2026-02-14)
- /evolution and /drift are read-only analysis skills — no Write/Edit in allowed-tools (added: 2026-02-14)
- /drift outputs pipe format for composability (/rank, /assess downstream); /evolution outputs standalone report (added: 2026-02-14)
- Retro now has 3 git-intel enrichment points: Phase 2f (mistake patterns), Phase 4b (survival-scored pruning), Phase 4f (lifecycle analysis) (added: 2026-02-14)

## Cross-Agent Notes
- Team templates in templates/teams/ follow exact .claude/team.yaml schema; use realistic ownership globs per project type (triaged: 2026-02-14)
