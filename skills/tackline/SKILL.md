---
name: tackline
description: >
  Use when you need to research, analyze, compare, plan, or structure thinking and aren't sure
  which primitive to reach for. Chains operations (gather, distill, rank, filter, verify, etc.)
  automatically based on your goal. Prefer this over individual primitives for multi-step work.
argument-hint: "<goal or operation chain>"
allowed-tools:
  - Read
  - Grep
  - Glob
  - Agent
  - Bash(git log:*, git show:*, git diff:*)
user-invocable: true
disable-model-invocation: false
context: inline
---

# Tackline

You are running **/tackline** — a unified composable knowledge-work engine. You have 14 operations you can perform inline (no separate skill invocations needed) and orchestration patterns for parallelizing work across agents.

**Goal**: $ARGUMENTS

---

## Phase 0 — Parse Intent

Read the goal. Determine which operations to apply and in what order.

**If $ARGUMENTS is empty**, ask for a goal in one sentence.

**If $ARGUMENTS names operations explicitly** (e.g., "gather auth patterns then rank by risk"), follow that chain.

**If $ARGUMENTS is a natural-language goal**, select operations yourself using the decision tree below.

### Decision Tree

| User wants to... | Operations |
|---|---|
| Research a topic | gather |
| Research then prioritize | gather → distill → rank |
| Research then fact-check | gather → filter → verify |
| Compare two approaches | gather (both) → diff-ideas |
| Break down a large goal | decompose → plan |
| Break down then implement | decompose → rank → plan |
| Stress-test an idea | gather → critique |
| Prototype a structure | gather → distill → sketch |
| Condense verbose input | distill |
| Elaborate sparse input | expand |
| Rewrite items to a format | transform |
| Categorize by severity/type | assess |
| Remove irrelevant items | filter |
| Combine scattered findings | merge |
| Check claims against reality | verify |
| Find flaws in a proposal | critique |
| Sequence by dependency | plan |

**Bias toward fewer operations.** If the goal can be achieved in one operation, use one. Don't chain gather → distill → rank when gather alone answers the question.

---

## Operations Reference

Each operation transforms input into structured output. All emit **pipe format** (see `rules/pipe-format.md`). Output of one operation feeds the next via conversation context.

### Input Operations

**gather** — Collect information with sources and confidence levels.
- Search codebase first (Grep/Glob/Read), web second.
- Every finding gets: title, detail, source (file:line or URL), confidence (CONFIRMED/LIKELY/POSSIBLE).
- For 3+ distinct source types, fan out 2-3 agents (see Orchestration below).

### Transform Operations

**distill** — Reduce to essentials. Arg: target count or topic filter ("to 5 bullets", "auth only").

**expand** — Add depth and evidence to sparse items. Inverse of distill. Ground new claims in sources.

**transform** — Rewrite every item independently per instruction ("as ticket titles", "as acceptance criteria"). Item count in = item count out.

**rank** — Score items 1-5 on stated criteria, reorder highest-to-lowest. Add criteria table. Break all ties.

**filter** — Binary keep/drop per item against a criterion. Renumber kept items. List dropped items separately.

**assess** — Assign categorical verdicts (CRITICAL/WARNING/OK or custom rubric). Group by category. Add rubric table.

**verify** — Check claims against code, git history, docs, or web. Verdict per claim: VERIFIED/REFUTED/UNCERTAIN with cited evidence. For 8+ claims, fan out up to 4 agents.

**critique** — Adversarial review. Classify each finding as FLAW (incorrect), GAP (missing), or RISK (could fail). Add severity table.

**merge** — Combine multiple pipe-format blocks from context. Dedup (>80% overlap or same title). Upgrade confidence when 2+ sources confirm. Combine source lists.

**diff-ideas** — Compare exactly two approaches across 4-6 dimensions. Produce comparison table with per-dimension winner and recommendation.

### Output Operations

**decompose** — Split into 3-6 MECE sub-parts with scope, boundary, and isolation hint (independent/shared-state). Ground in actual file structure.

**plan** — Sequence items by dependency. Output: execution order, dependency graph, parallel waves with isolation hints (worktree/shared).

**sketch** — Structural skeleton with TODO placeholders. No implementation — just bones. Code, doc, config, schema, or workflow.

---

## Guidelines

All output follows the pipe-format rule (loaded alongside this skill). For fan-out dispatch, follow the fan-out-protocol rule. For chains of 3+ operations, checkpoint per the compaction-resilience rule.


1. **Fewer operations is better.** Don't chain when one operation suffices.
2. **Code first, web second.** When researching, exhaust codebase sources before hitting the web.
3. **Ground in reality.** Decompose/plan should read actual file structure, not guess.
4. **Abort on empty.** If an operation produces 0 items, stop and report rather than continuing with nothing.
5. **Preserve provenance.** Source attribution and confidence flow through the entire chain.
6. **Be concrete.** Cite file:line, commit hashes, URLs. No hand-waving.
7. **Operation-specific sections** (criteria tables, rubrics, comparison tables, dependency graphs) go between Items and Summary.
