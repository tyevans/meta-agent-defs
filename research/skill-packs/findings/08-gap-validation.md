# Gap Validation: Verb-Cluster Analysis Against Skill System

## Executive Summary

Analysis of 10 alleged gaps from verb-clustering against tackline's 46-skill inventory (16 core + 22 workflows + 9 teams).

**Final Count:**
- **Real Gaps**: 0
- **Partially Covered**: 2 (OPTIMIZE, DEPLOY/SHIP)
- **False Gaps**: 6 (TEST, REFACTOR, DEBUG, MENTOR/LEARN, AUTOMATE, ANALYZE)
- **Composite Gaps**: 2 (ORCHESTRATE, INTEGRATE)

**Verdict**: The verb clustering identified valid activity domains, but the skill system covers all of them—either directly, via composable pipelines, or through distributed architectural patterns. No new skill invention required. The clustering is more useful as a **vocabulary audit** (confirming user mental models align with available functionality) than a gap identification tool.

---

## Detailed Gap Validation

### 1. ORCHESTRATE (39 items) — COMPOSITE

**Alleged Gap**: Orchestration of multiple agents, tasks, or workflows.

**Skills Checked**:
- `/sprint` (core orchestration: dispatch tasks by ownership, inject learnings)
- `/assemble` (team roster assembly and role assignment)
- `/standup` (daily status sync and blockers)
- `/team-meeting` (persistent team goal planning)
- `/blossom` (spike-driven exploration with beads epic output)
- `/do` (goal-directed skill routing and execution)
- `/discover` (skill/pipeline recommendation)

**Coverage**: COMPLETE via pipeline.

**Evidence**:
- `/sprint` Phase 1 explicitly: "For each task: dispatch via Task(...) with ownership + learnings injection" (Phase 2). Persistent learnings.md files store accumulated knowledge across runs.
- `/assemble` creates a team.yaml manifest and injects role assignments.
- `/blossom` Phase 3: "Create a beads epic with children tasks from priorities."
- `/do` and `/discover` route goals to correct skills.

**Exact Pipeline**: `/assemble` → `/sprint` (with learnings loop) + `/standup` + `/team-meeting`

For complex exploration: `/blossom` (produces epic) → `/sprint` (executes epic tasks)

**Confidence**: HIGH — Orchestration is explicit in sprint's workflow and persistent learnings mechanism.

---

### 2. TEST (68 items) — FALSE GAP

**Alleged Gap**: Testing, quality assurance, test design, test execution, and test coverage.

**Skills Checked**:
- `/test-strategy` (comprehensive testing workflow)
- `/tracer` (includes test phase)
- `/spec` (includes test requirements)
- `/review` (includes testing dimension)
- `/challenge-run` (adversarial testing)

**Coverage**: COMPLETE.

**Evidence**:
- `/test-strategy` Phase 2-3: Classify test knowledge source (Codified/Articulated/Tacit), write tests from specs (or empirically for Tacit), enforce red-green gate. Phase 4: Coverage analysis. Explicit step-by-step testing workflow.
- `/tracer` Phase 5 ("Tests"): Write tests for happy path, error handling, edge cases. Enforce red-green gates.
- `/review` Dimension 3 ("Testing"): "Does the test suite cover happy path, error cases, and edge cases?"
- `/challenge-run` Phase 1: "Run adversarial challenges against agent, collect failure cases."

**Exact Pattern**: Single-skill solution via `/test-strategy`, or as part of `/tracer` phase structure.

**Confidence**: HIGH — Test strategy is a dedicated skill with explicit phases.

---

### 3. OPTIMIZE (41 items) — PARTIALLY COVERED

**Alleged Gap**: Performance optimization, resource efficiency, algorithmic improvements.

**Skills Checked**:
- `/tracer` (phase 6 includes "Report and iterate")
- `/premortem` (identifies failure modes, not optimization)
- `/fractal` (depth-first exploration)
- `/review` (code quality review)
- No dedicated `/optimize` skill found

**Coverage**: INDIRECT via `/tracer` widening phases.

**Evidence**:
- `/tracer` Phase 4 ("Edge cases and validation") and Phase 6 ("Report and iterate") are described as "widen" phases where non-happy-path scenarios are explored, but these focus on correctness/robustness, not performance.
- `/tracer` does not explicitly mention optimization passes for speed or resource use.
- `/premortem` identifies risk-first design but is failure-focused, not performance-focused.
- `/review` includes architecture dimension but no explicit performance review.

**What's Missing**: No dedicated skill for identifying bottlenecks, profiling results, or iterating toward optimization targets. `/tracer` widening could theoretically include performance work, but it's not explicitly stated.

**How to Close**: Either document that `/tracer` phase 6 can include performance widening (update skill.md), or create `/optimize` as a dedicated skill for profiling + iterative improvement.

**Confidence**: MEDIUM — Coverage is implicit and incomplete. Optimization is a known activity domain with no explicit skill.

---

### 4. DEPLOY/SHIP (29 items) — PARTIALLY COVERED

**Alleged Gap**: Deployment, release management, rollout strategies.

**Skills Checked**:
- `/tracer` (end-to-end implementation)
- `/test-strategy` (includes testing gates)
- `/spec` (includes acceptance criteria)
- No deployment-specific workflow found
- `/bootstrap` (project setup, not deployment)

**Coverage**: INDIRECT via `/tracer` + `/test-strategy` + integration/monitoring.

**Evidence**:
- `/tracer` Phase 5-6 includes tests and reporting, but does not mention deployment gates, rollout strategies, or health checks.
- `/test-strategy` Phase 4 covers test coverage but not deployment readiness.
- `/bootstrap` covers project setup, not the release workflow.
- No explicit skill for: canary deployments, blue-green strategies, rollback plans, or monitoring post-deployment.

**What's Missing**: Deployment-specific skill covering pre-deployment checklist, rollout strategy selection, health monitoring, and rollback criteria. Currently bundled into `/tracer` reporting phase without explicit guidance.

**How to Close**: Create `/deploy` or `/ship` skill with phases: readiness checklist, strategy selection, execution, monitoring, and rollback criteria. Or expand `/tracer` Phase 6 to include deployment as an explicit sub-phase.

**Confidence**: MEDIUM — Readiness is covered, but deployment execution and post-deployment monitoring are not.

---

### 5. REFACTOR (38 items) — FALSE GAP

**Alleged Gap**: Code restructuring, refactoring, codebase modernization.

**Skills Checked**:
- `/tracer` (iterative implementation with validation)
- `/review` (code quality analysis)
- `/evolution` (definition change tracking)

**Coverage**: COMPLETE via `/tracer`.

**Evidence**:
- `/tracer` Phase 1 ("Map the path"): Identify the thinnest end-to-end slice.
- Phase 2 ("Happy path tracer"): Implement that slice.
- Phases 3-5 ("Error handling", "Edge cases", "Tests"): Iteratively widen and validate.
- This workflow is explicitly designed for safe, incremental change with continuous validation—the core of refactoring.
- `/review` provides code quality gates.
- `/evolution` tracks definition changes across versions.

**Exact Pattern**: Run `/tracer` with the refactoring scope (module, layer, or subsystem) as the goal. The thinnest slice becomes the first refactored component; phases 3-5 widen safely.

**Confidence**: HIGH — Tracer's phase structure is explicitly designed for incremental change.

---

### 6. DEBUG (35 items) — FALSE GAP

**Alleged Gap**: Debugging, diagnosing failures, root cause analysis.

**Skills Checked**:
- `/fractal` (goal-directed recursive exploration)
- `/review` (code analysis)
- `/diagnose-agent` (agent behavior diagnosis)
- `/challenge-run` (failure case collection)
- `/gather` + `/distill` (information processing)

**Coverage**: COMPLETE via `/fractal` + `/review` + specialized agent diagnosis.

**Evidence**:
- `/fractal` Phase 1: "Define investigation goal (narrow and specific)." Phase 2: "Recursively explore" sub-areas. Phase 3: "User checkpoint for depth decisions." Designed for goal-directed diagnosis.
- `/review` Dimension 1 ("Correctness"): "Does the code do what the spec says?" Identifies logical errors.
- `/diagnose-agent` Phase 1: "Describe the agent's goal and observed failure." Phase 2: "Hypothesize root cause." Phase 3: "Test hypothesis against agent logs."
- `/challenge-run` systematically collects failure cases.

**Exact Pattern**:
- For implementation bugs: `/review` (code analysis) or `/fractal` (recursive investigation).
- For agent behavior bugs: `/diagnose-agent`.
- For unknown failure domains: `/blossom` (explore unknown) → spike findings → `/review` or `/fractal`.

**Confidence**: HIGH — Multiple skills cover debugging from different angles.

---

### 7. INTEGRATE (24 items) — COMPOSITE

**Alleged Gap**: Integration of components, systems, or subsystems.

**Skills Checked**:
- `/tracer` (phase 4: "Edge cases and validation" includes boundary crossing)
- `/spec` (integration acceptance criteria)
- `/review` (architecture dimension)
- `/blossom` (exploration producing integration tasks)
- `/test-strategy` (integration test classification)

**Coverage**: COMPLETE via pipeline.

**Evidence**:
- `/tracer` Phase 4 explicitly: "Validate against integration contracts." Edges between subsystems are treated as edge cases.
- `/spec` phase 5: "Integration acceptance criteria: Does component A conform to component B's contract?"
- `/test-strategy` Phase 2 classifies tests; integration tests are a category.
- `/review` Dimension 4 ("Architecture"): "Are dependencies clean? Do modules have clear contracts?"

**Exact Pipeline**: `/spec` (define contracts) → `/tracer` (implement with validation at boundaries) → `/test-strategy` (write integration tests) → `/review` (verify contracts held)

Or for discovery: `/blossom` (explore integration needs) → tasks in epic.

**Confidence**: HIGH — Integration is treated as a cross-cutting concern across spec, tracer, testing, and review.

---

### 8. AUTOMATE (31 items) — FALSE GAP

**Alleged Gap**: Automation, scripting, process automation.

**Skills Checked**:
- Architecture in CLAUDE.md: "Hooks fail gracefully with `|| true` for optional tools (like `bd`)"
- SessionStart hook pattern (auto-runs `bd prime`, rule-relevance matching)
- Project bootstrap automation
- No explicit skill, but handled via beads hooks and installation
- `/bootstrap` covers project setup automation

**Coverage**: COMPLETE via architectural patterns + hooks.

**Evidence**:
- CLAUDE.md: "Hooks auto-sync, run `bd sync` at session end" — beads hooks automatically commit state.
- `install.sh` is idempotent and automates symlink setup across projects.
- SessionStart hook auto-runs rule-relevance.sh and beads prime.
- `/bootstrap` Phase 1: "Infer project type from root files (package.json, Cargo.toml, Makefile, go.mod, pyproject.toml)." Phase 2: "Create directory structure." Phase 3: "Write starter configs and rules." Entire automation sequence.
- `.claude/` hooks auto-inject session-close protocol.

**Pattern**: Automation is handled at the infrastructure level (hooks, install script), not as a user-facing skill. This is correct: automation should be invisible, not require manual invocation.

**Confidence**: HIGH — Automation is systematized in the architecture. No user-facing skill needed.

---

### 9. MENTOR/LEARN (27 items) — FALSE GAP

**Alleged Gap**: Learning, knowledge transfer, mentoring, agent improvement.

**Skills Checked**:
- `/active-learn` (full adversarial training loop for agents)
- `/challenge-gen` (generates adversarial challenges)
- `/challenge-run` (runs challenges, collects failures)
- `/curate` (extracts insights from learnings)
- `/promote` (promotes learnings to prompt)
- `/tend` (composite: curate + promote)
- `/sprint` (injects learnings into task dispatch)

**Coverage**: COMPLETE with multi-phase learning loop.

**Evidence**:
- `/active-learn` Phases 1-6: Generate challenges (failing cases), run against agent, collect failures, curate insights, promote to prompt, verify improvement. Full closed-loop learning.
- `/sprint` Phase 1 explicitly: "Inject prior learnings (from learnings.md) into each task's prompt." Persistent learnings across team runs.
- Learnings.md files (rules/memory-layout.md) store accumulated knowledge: "path registry for persistent state."
- `/curate` Phase 2: "Extract insight from [learnings]" into actionable prompt updates.
- `/promote` injects curated insights into agent prompts.
- `/tend` = `/curate` + `/promote` (composite single-step).

**Exact Pattern**:
- For individual agent improvement: `/active-learn` (or `/challenge-gen` → `/challenge-run` → `/curate` → `/promote`).
- For team learning: `/sprint` automatically injects prior learnings. Teams persist learnings.md files that accumulate across sessions.

**Confidence**: HIGH — Learning loop is explicit and persistent. Full adversarial training available.

---

### 10. ANALYZE (44 items) — FALSE GAP

**Alleged Gap**: Analysis, data analysis, problem analysis, pattern detection.

**Skills Checked**:
- `/gather` (collect information from multiple sources)
- `/distill` (compress gathered findings)
- `/rank` (prioritize by criteria)
- `/filter` (subset by constraints)
- `/assess` (quality judgment)
- `/verify` (factual verification)
- `/fractal` (recursive exploration)
- `/review` (code analysis)
- `/evolution` (change tracking and analysis)
- `/drift` (cross-project pattern detection)

**Coverage**: COMPLETE with 6+ direct analysis skills.

**Evidence**:
- `/gather` Phase 1: "Find all [topic] references, uses, and definitions." Systematic data collection.
- `/distill` Phase 1: "Read N items from /gather output." Phase 2: "Synthesize into M categories." Compression and pattern detection.
- `/rank` adds prioritization by criteria.
- `/filter` adds constraint-based subsetting.
- `/assess` Phase 2: "For each item, provide a quality judgment: [CONFIRMED | LIKELY | POSSIBLE]." Confidence assessment.
- `/verify` Phase 1: "Check each claim by reading the actual code." Factual verification against source.
- `/fractal` adds recursive depth for complex unknowns.
- `/review` Dimensions 1-5 provide multi-dimensional code analysis.
- `/evolution` Phase 1: "Track definition changes across versions." Historical analysis.
- `/drift` Phase 1: "Find where definition D appears in [codebase]." Cross-cutting pattern detection.

**Exact Pattern**: Single skill for simple analysis: `/gather` → `/distill`. For prioritized analysis: `/gather` → `/distill` → `/rank`. For deep analysis: `/fractal`.

**Confidence**: HIGH — Analysis is core to the skill system. Six+ dedicated primitives.

---

## Summary Table

| Gap Name | Verdict | Pipeline or Location | Confidence |
|----------|---------|----------------------|------------|
| ORCHESTRATE | Composite | `/assemble` → `/sprint` + learnings loop | HIGH |
| TEST | False | `/test-strategy` skill | HIGH |
| OPTIMIZE | Partial | Indirect via `/tracer` phase 6; missing explicit skill | MEDIUM |
| DEPLOY/SHIP | Partial | Indirect via `/tracer` phase 6; missing deployment strategy skill | MEDIUM |
| REFACTOR | False | `/tracer` skill (thinnest slice → iterative widening) | HIGH |
| DEBUG | False | `/fractal` or `/review` or `/diagnose-agent` | HIGH |
| INTEGRATE | Composite | `/spec` → `/tracer` → `/test-strategy` → `/review` | HIGH |
| AUTOMATE | False | Architectural (hooks, install.sh, bootstrap) | HIGH |
| MENTOR/LEARN | False | `/active-learn` skill + persistent `/sprint` learnings | HIGH |
| ANALYZE | False | `/gather` → `/distill` → `/rank` (+ 4 more analysis skills) | HIGH |

---

## Conclusions

1. **No Real Gaps**: The verb clustering identified legitimate activity domains, but all 10 are covered by existing skills—either directly, via documented pipelines, or through architectural patterns.

2. **Two Partial Gaps** (OPTIMIZE, DEPLOY/SHIP):
   - **OPTIMIZE**: Optimization work could theoretically happen in `/tracer` phase 6, but it's not explicitly documented. Recommend either: (a) update `/tracer` skill.md to explicitly mention performance widening as a phase 6 activity, or (b) create a dedicated `/optimize` skill.
   - **DEPLOY/SHIP**: Deployment readiness is covered (via `/test-strategy` + `/tracer`), but the actual deployment workflow (strategy selection, canary deployments, rollback) is not explicit. Recommend creating a `/deploy` or `/ship` skill with explicit phases for deployment execution and post-deployment monitoring.

3. **Six False Gaps** (TEST, REFACTOR, DEBUG, AUTOMATE, MENTOR/LEARN, ANALYZE): These are fully covered and well-articulated in the skill system.

4. **Two Composite Gaps** (ORCHESTRATE, INTEGRATE): These are handled via documented multi-skill pipelines.

5. **Skill System Assessment**:
   - The system is **coverage-complete** for the activity domains identified in the verb clustering.
   - The system is **documentation-light** for OPTIMIZE and DEPLOY/SHIP; these should be elevated to explicit skills or skill phases.
   - The system uses **composability (pipe format)** and **persistence (learnings.md)** to handle coordination, which may be invisible to a clustering analysis that looks at individual skill names.

6. **Recommendations**:
   - **Create `/optimize` skill** (or document phase 6 of `/tracer` to explicitly include optimization).
   - **Create `/deploy` or `/ship` skill** with explicit deployment strategy and monitoring phases.
   - **No new verbs need invention**. All 10 activity domains are systemically addressed.
   - **The clustering is more useful as a vocabulary audit** (confirming user mental models match available functionality) than as a gap detection tool. Clustering finds user-level action words; it doesn't see composite pipelines or architectural patterns.

---

## Methodology Notes

Analysis was conducted by:
1. Reading all 46 skill definitions (16 core, 22 workflows, 9 teams).
2. Examining each skill's "When to Use", workflow phases, and outputs.
3. Checking for direct coverage, pipeline compositions, and architectural patterns.
4. Cross-referencing related skills (e.g., `/tracer` phases with `/test-strategy` and `/review`).
5. Verifying persistent state mechanisms (learnings.md, beads epics, team.yaml).

The analysis prioritized **actual code and documented workflows** over inference or training knowledge.

**Freshness**: Analysis conducted 2026-02-25 against CLAUDE.md, docs/INDEX.md, docs/pipelines.md, and all 46 SKILL.md files in skills/{core,workflows,teams}/*.
