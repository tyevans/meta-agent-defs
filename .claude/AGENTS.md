# Agent Catalog (Project-Level)

Agents for working on the meta-agent-defs repo itself. These evaluate, research, and improve the definitions — not just maintain them.

Note: The **global** agents in `agents/` (agent-generator, project-bootstrapper) are for use in other projects. These **project-level** agents in `.claude/agents/` are for working on this repo.

## Research & Evaluation Agents

These agents question, test, and improve the quality of definitions. They are the reason this project exists — without them, we're just maintaining files.

| Agent | Purpose | Model | Invoke When |
|-------|---------|-------|-------------|
| pattern-researcher | Research what makes definitions effective — prompt engineering, instruction design, model behavior | opus | Questioning current patterns, designing a new category of definition, or diagnosing why a definition underperforms |
| definition-tester | Red-team definitions by simulating Claude's interpretation and finding ambiguities, gaps, and failure modes | opus | A definition is drafted and needs adversarial review, or a shipped definition is producing unexpected behavior |
| effectiveness-auditor | Evaluate whether definitions produce genuinely good outcomes, not just structural compliance | opus | Periodic quality audits, when a definition seems to underperform, or when reconsidering the authoring conventions themselves |

## Authoring Agents

These agents write definitions. They are informed by research — they don't just pattern-match against templates.

| Agent | Purpose | Model | Invoke When |
|-------|---------|-------|-------------|
| agent-author | Write agent definitions as prompt engineering, not template filling | opus | Creating a new agent or substantively revising an existing one |
| skill-author | Write skill definitions for workflow automation | sonnet | Creating a new skill or revising an existing skill |

## Maintenance Agents

Mechanical tasks that don't need research orientation — just correctness.

| Agent | Purpose | Model | Invoke When |
|-------|---------|-------|-------------|
| sync-auditor | Check cross-artifact consistency (docs match files) | haiku | After adding/removing/renaming files, or when docs seem stale |
| settings-editor | Edit settings.json files (hooks, permissions, env) | sonnet | Adding hooks, changing permissions, modifying environment config |
| installer-maintainer | Update install.sh for new artifact types or fixes | sonnet | Adding new artifact directories, fixing installation edge cases |

## Agent Capabilities

| Agent | Reads | Writes | Web Research | Runs Commands | Uses Beads |
|-------|-------|--------|-------------|--------------|-----------|
| pattern-researcher | Y | N | Y | N | Y |
| definition-tester | Y | N | N | N | Y |
| effectiveness-auditor | Y | N | N | git log, git diff | Y |
| agent-author | Y | Y | Y | N | Y |
| skill-author | Y | Y | N | N | Y |
| sync-auditor | Y | N | N | ls | Y |
| settings-editor | Y | Y | N | N | Y |
| installer-maintainer | Y | Y | N | ls, ./install.sh | Y |

## Workflows

### New Agent Definition
1. **pattern-researcher** — If this is a novel agent type, research what makes similar agents effective
2. **agent-author** — Write the definition, informed by research
3. **definition-tester** — Red-team the definition for ambiguities and failure modes
4. **effectiveness-auditor** — Evaluate whether the definition produces good outcomes (can run after the agent has been used a few times)
5. **sync-auditor** — Verify docs are updated

### New Skill Definition
1. **pattern-researcher** — If this is a novel workflow type, research effective patterns
2. **skill-author** — Write the skill definition
3. **definition-tester** — Stress-test the workflow against scenarios
4. **sync-auditor** — Verify docs are updated

### Periodic Quality Audit
1. **effectiveness-auditor** — Evaluate all definitions for actual effectiveness (not just compliance)
2. **pattern-researcher** — Research any findings that suggest the conventions themselves need updating
3. **agent-author** / **skill-author** — Revise definitions based on audit findings

### Diagnosing Underperformance
1. **definition-tester** — Test the specific definition that's underperforming
2. **pattern-researcher** — Research why the identified patterns might fail
3. **agent-author** / **skill-author** — Revise with new understanding

### Convention Review
1. **effectiveness-auditor** (convention audit scope) — Evaluate whether authoring rules are producing better definitions
2. **pattern-researcher** — Research alternatives to any conventions flagged as ceremonial
3. Update rules in `.claude/rules/` based on findings
