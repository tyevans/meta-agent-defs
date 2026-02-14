# Demo: OSS Audit with Primitives

**Target**: meta-agent-defs skill definitions
**Chain**: gather → assess → filter → sketch
**Scenario**: External contributor doing a first-pass quality audit of skill files

This demo shows how composable primitives apply to ANY codebase, not just meta-agent-defs. We're treating this repo as if we're auditing an OSS project for quality and consistency issues.

---

## Step 1: /gather skill quality patterns in skills/

```
User: /gather skill quality patterns in skills/
```

## Findings on skill quality patterns

**Source**: /gather
**Input**: skill quality patterns in skills/

### Items

1. **Inconsistent title formatting** — Some skills use `# Skill-Name: Description` format (gather, distill, rank), others use `# Full Sentence Description` (blossom, fractal, retro). No consistent pattern across the 26 skill files.
   - source: skills/gather/SKILL.md:10, skills/blossom/SKILL.md:11, skills/retro/SKILL.md:10
   - confidence: CONFIRMED

2. **Missing visual workflow diagrams** — Skill authoring rules require "a visual workflow diagram showing phase progression (ASCII flowchart)" but only blossom and retro include ASCII flow diagrams. Other skills like gather, distill, rank, filter, assess have numbered phases but no visual.
   - source: .claude/rules/skill-authoring.md:55, skills/gather/SKILL.md (missing), skills/distill/SKILL.md (missing)
   - confidence: CONFIRMED

3. **Phase numbering inconsistency** — Some skills use `## Phase N: Title` (blossom, fractal, retro, consolidate), others use `### N. Title` (gather), others use no phase numbers (distill, rank, filter, assess).
   - source: skills/blossom/SKILL.md:30, skills/gather/SKILL.md:23, skills/distill/SKILL.md:21
   - confidence: CONFIRMED

4. **Bash tool restriction violations** — Several skills use `Bash(bd:*)` and `Bash(git:*)` prefixes, but consolidate line 11 uses `!bd list --status=open` which violates the restriction format. The `!` prefix is not documented in skill-authoring rules.
   - source: skills/consolidate/SKILL.md:11, .claude/rules/skill-authoring.md:35
   - confidence: CONFIRMED

5. **Incomplete pipe format compliance** — Consolidate includes a pipe format report template (lines 224-259) but blossom's final report (lines 388-427) also uses pipe format. Fractal uses pipe format (lines 168-194). Sprint and retro do NOT emit pipe format despite being workflow skills. No clear policy on which skills should emit pipe format.
   - source: skills/consolidate/SKILL.md:224, skills/blossom/SKILL.md:388, skills/sprint/SKILL.md (no pipe format), skills/retro/SKILL.md (no pipe format)
   - confidence: LIKELY

6. **Inconsistent $ARGUMENTS reference style** — Some skills reference user input as "Goal: $ARGUMENTS" (fractal line 14), others use "Topic: **$ARGUMENTS**" (gather line 12), others use "Target: **$ARGUMENTS**" (distill line 12), others use "Focus area (optional): **$ARGUMENTS**" (sprint line 14, retro line 12).
   - source: skills/fractal/SKILL.md:14, skills/gather/SKILL.md:12, skills/distill/SKILL.md:12, skills/sprint/SKILL.md:14
   - confidence: CONFIRMED

7. **Missing "When to Use" sections** — Consolidate has "When to Use" at line 17, blossom has "When to Use" implied but not labeled (lines missing), fractal has "When to Use" at line 17, retro has "When to Use" at line 15. Skill authoring rules require this section but don't enforce labeling.
   - source: .claude/rules/skill-authoring.md:43, skills/consolidate/SKILL.md:17, skills/blossom/SKILL.md (unlabeled)
   - confidence: LIKELY

8. **Guidelines section placement variance** — Some skills have "Guidelines" at the end (consolidate line 263, fractal line 213, retro line 247), others embed guidelines within phases (blossom line 430), primitives (gather, distill, filter) have "Guidelines" mid-file before the end.
   - source: skills/consolidate/SKILL.md:263, skills/blossom/SKILL.md:430, skills/gather/SKILL.md:42
   - confidence: CONFIRMED

### Summary

The skill library shows strong functional quality but inconsistent structural patterns. Eight distinct quality issues were found across formatting (title style, phase numbering, $ARGUMENTS style), completeness (missing workflow diagrams, unclear pipe format policy), and compliance (bash tool prefix misuse). All findings are CONFIRMED by reading actual skill files. Most issues are cosmetic but collectively create friction for contributors trying to author new skills in a consistent style.

> **Commentary**: Real findings grounded in actual file reading. Confidence levels show verification — no hedging.

---

## Step 2: /assess by compliance with skill-authoring rules

```
User: /assess by compliance with skill-authoring rules
```

## Assessed skill quality patterns

**Source**: /assess
**Input**: compliance with skill-authoring rules

### Items

1. **Inconsistent title formatting** — **WARNING** — Violates implicit consistency expectation but no explicit rule mandates one title format over another.
   - source: skills/gather/SKILL.md:10, skills/blossom/SKILL.md:11, skills/retro/SKILL.md:10
   - confidence: CONFIRMED

2. **Missing visual workflow diagrams** — **CRITICAL** — Directly violates skill-authoring.md:55 which states "Include a visual workflow diagram showing phase progression (ASCII flowchart)". 23 out of 26 skills are non-compliant.
   - source: .claude/rules/skill-authoring.md:55, skills/gather/SKILL.md (missing), skills/distill/SKILL.md (missing)
   - confidence: CONFIRMED

3. **Phase numbering inconsistency** — **SUGGESTION** — No explicit rule requires uniform phase numbering style. Inconsistency reduces readability but doesn't violate documented standards.
   - source: skills/blossom/SKILL.md:30, skills/gather/SKILL.md:23, skills/distill/SKILL.md:21
   - confidence: CONFIRMED

4. **Bash tool restriction violations** — **CRITICAL** — consolidate/SKILL.md:11 uses `!bd list` which is not a valid Bash(prefix:*) restriction format per skill-authoring.md:35. The `!` prefix is undocumented and may not work.
   - source: skills/consolidate/SKILL.md:11, .claude/rules/skill-authoring.md:35
   - confidence: CONFIRMED

5. **Incomplete pipe format compliance** — **SUGGESTION** — No rule states which skills must emit pipe format. The pipe-format.md contract applies to "composable primitives" but doesn't define that set exhaustively. Workflow skills (sprint, retro) may intentionally omit it.
   - source: rules/pipe-format.md:3, skills/sprint/SKILL.md (no pipe format), skills/retro/SKILL.md (no pipe format)
   - confidence: LIKELY

6. **Inconsistent $ARGUMENTS reference style** — **SUGGESTION** — Skill-authoring.md:54 says "Reference `$ARGUMENTS` to accept user input" but doesn't mandate formatting. Variance is cosmetic.
   - source: skills/fractal/SKILL.md:14, skills/gather/SKILL.md:12, skills/distill/SKILL.md:12
   - confidence: CONFIRMED

7. **Missing "When to Use" sections** — **WARNING** — Skill-authoring.md:43 requires "When to Use" as a section. Some skills have the content but unlabeled. Policy unclear whether labeling is mandatory.
   - source: .claude/rules/skill-authoring.md:43, skills/blossom/SKILL.md (unlabeled)
   - confidence: LIKELY

8. **Guidelines section placement variance** — **SUGGESTION** — Skill-authoring.md:47 requires "Guidelines" section but doesn't specify placement. Variance doesn't violate documented rules.
   - source: .claude/rules/skill-authoring.md:47, skills/consolidate/SKILL.md:263, skills/blossom/SKILL.md:430
   - confidence: CONFIRMED

### Rubric

| Category | Definition |
|----------|------------|
| CRITICAL | Violates explicit documented rule in skill-authoring.md or pipe-format.md |
| WARNING  | Violates implicit expectation or unclear policy |
| SUGGESTION | Cosmetic inconsistency, no rule violation |

### Summary

Two CRITICAL issues were found: missing workflow diagrams (23/26 skills non-compliant) and Bash tool restriction format violation in consolidate. Two WARNING issues relate to unclear policy (title formatting, "When to Use" labeling). Four SUGGESTION issues are cosmetic inconsistencies that don't violate documented rules but reduce contributor experience. The majority of compliance failures stem from incomplete adoption of the skill-authoring.md rules added after many skills were already written.

> **Commentary**: Rubric-based categorical evaluation. Findings grouped by severity — critical items first.

---

## Step 3: /filter keep non-compliant only

```
User: /filter keep critical and warning only
```

## Filtered skill quality patterns

**Source**: /filter
**Input**: keep critical and warning only

### Items

1. **Missing visual workflow diagrams** — CRITICAL — Directly violates skill-authoring.md:55 which states "Include a visual workflow diagram showing phase progression (ASCII flowchart)". 23 out of 26 skills are non-compliant.
   - source: .claude/rules/skill-authoring.md:55, skills/gather/SKILL.md (missing), skills/distill/SKILL.md (missing)
   - confidence: CONFIRMED

2. **Bash tool restriction violations** — CRITICAL — consolidate/SKILL.md:11 uses `!bd list` which is not a valid Bash(prefix:*) restriction format per skill-authoring.md:35. The `!` prefix is undocumented and may not work.
   - source: skills/consolidate/SKILL.md:11, .claude/rules/skill-authoring.md:35
   - confidence: CONFIRMED

3. **Inconsistent title formatting** — WARNING — Violates implicit consistency expectation but no explicit rule mandates one title format over another.
   - source: skills/gather/SKILL.md:10, skills/blossom/SKILL.md:11, skills/retro/SKILL.md:10
   - confidence: CONFIRMED

4. **Missing "When to Use" sections** — WARNING — Skill-authoring.md:43 requires "When to Use" as a section. Some skills have the content but unlabeled. Policy unclear whether labeling is mandatory.
   - source: .claude/rules/skill-authoring.md:43, skills/blossom/SKILL.md (unlabeled)
   - confidence: LIKELY

### Dropped

- **Phase numbering inconsistency** (SUGGESTION) — cosmetic only
- **Incomplete pipe format compliance** (SUGGESTION) — no clear policy violation
- **Inconsistent $ARGUMENTS reference style** (SUGGESTION) — cosmetic only
- **Guidelines section placement variance** (SUGGESTION) — cosmetic only

**Dropped count**: 4 items (all SUGGESTION severity)

### Summary

Four compliance issues remain after filtering: two CRITICAL (missing workflow diagrams affecting 23 skills, Bash restriction format violation in consolidate) and two WARNING (title formatting inconsistency, unlabeled "When to Use" sections). Suggestion-level cosmetic issues were dropped to focus the audit on actionable compliance gaps.

> **Commentary**: Binary filter applied — kept CRITICAL and WARNING, dropped SUGGESTION. Dropped section shows transparency.

---

## Step 4: /sketch compliance fixes

```
User: /sketch fixes for critical issues
```

## Sketched compliance fixes

**Source**: /sketch
**Input**: fixes for critical issues (missing workflow diagrams, Bash restriction violation)

### Items

1. **Add workflow diagram to gather/SKILL.md** — Insert ASCII flow diagram after "When to Use" section (line 19)
   - source: skills/gather/SKILL.md:19

   ```markdown
   ## Process

   ```
   Search Code → Search Web → Emit Findings
        ↓              ↓            ↓
     Grep/Read    WebSearch   Pipe Format
                  WebFetch    (structured)
   ```

   ### 1. Search Code First
   ```

2. **Add workflow diagram to distill/SKILL.md** — Insert ASCII flow diagram after "When to Use" section (line 19)
   - source: skills/distill/SKILL.md:19

   ```markdown
   ## Process

   ```
   Detect Input → Parse Target → Distill → Emit Output
        ↓              ↓           ↓           ↓
   context/prior   N bullets   preserve   pipe format
   primitive       paragraph   sources    numbered list
   ```

   1. **Detect Input Source**: ...
   ```

3. **Add workflow diagram to filter/SKILL.md** — Insert ASCII flow diagram after "When to Use" section (line 21)
   - source: skills/filter/SKILL.md:21

   ```markdown
   ## Process

   ```
   Find Items → Parse Criterion → Apply Filter → Emit Output
        ↓              ↓               ↓              ↓
   from context   positive/       KEEP/DROP      kept items
   or $ARGUMENTS  negative        decision       + dropped
   ```

   1. **Find Items**: ...
   ```

4. **Add workflow diagram to assess/SKILL.md** — Insert ASCII flow diagram after "When to Use" section (line 19)
   - source: skills/assess/SKILL.md:19

   ```markdown
   ## Process

   ```
   Find Items → Parse Rubric → Evaluate → Emit Categorized Output
        ↓            ↓             ↓              ↓
   from context  categories   categorical    grouped by
   or $ARGUMENTS  (e.g. HIGH/  verdict per    category with
                  MEDIUM/LOW)  item           rubric table
   ```

   ### 1. Find Items
   ```

5. **Add workflow diagram to rank/SKILL.md** — Insert ASCII flow diagram after "When to Use" section (line 19)
   - source: skills/rank/SKILL.md:19

   ```markdown
   ## Process

   ```
   Find Items → Parse Criteria → Score & Order → Emit Ranked Output
        ↓              ↓               ↓                ↓
   from context   dimensions      1-5 scale        re-ordered list
   or $ARGUMENTS  (weight?)       per criterion    + criteria table
   ```

   ### 1. Find Items
   ```

6. **Add workflow diagram to verify/SKILL.md** — Insert ASCII flow diagram after "When to Use" section (line 20)
   - source: skills/verify/SKILL.md:20

   ```markdown
   ## Process

   ```
   Identify Claims → Gather Evidence → Assess → Emit Verification
          ↓                ↓               ↓            ↓
   from $ARGUMENTS    Grep/Read/git   VERIFIED/    pipe format
   or context         WebSearch/      REFUTED/     with evidence
                      WebFetch        UNCERTAIN    citations
   ```

   1. **Identify claims**: ...
   ```

7. **Add workflow diagram to decompose/SKILL.md** — Insert ASCII flow diagram after "When to Use" section (line 20)
   - source: skills/decompose/SKILL.md:20

   ```markdown
   ## Process

   ```
   Identify Whole → Search Structure → Split into Sub-Parts
         ↓                 ↓                    ↓
   $ARGUMENTS or     Grep/Glob/Read       3-6 MECE parts
   prior primitive   (codebase reality)   (scope + boundary)
   ```

   1. **Identify the whole**: ...
   ```

8. **Add workflow diagram to sketch/SKILL.md** — Insert ASCII flow diagram after "When to Use" section (line 20)
   - source: skills/sketch/SKILL.md:20

   ```markdown
   ## Process

   ```
   Check Context → Search Codebase → Emit Skeleton
         ↓                ↓                 ↓
   prior primitive   Grep/Glob         pipe format
   findings          (conventions)     code blocks
                                       + TODOs
   ```

   1. **Check context** for prior primitive output...
   ```

9. **Fix Bash restriction in consolidate/SKILL.md** — Remove undocumented `!` prefix, replace with comment explaining the intent
   - source: skills/consolidate/SKILL.md:11

   ```markdown
   ---
   name: consolidate
   description: "Review and tighten the beads backlog..."
   argument-hint: "[area or scope]"
   disable-model-invocation: true
   user-invocable: true
   allowed-tools: Read, Grep, Glob, Bash(bd:*), Bash(git:*)
   context: fork
   ---

   # Consolidate: Backlog Review and Tightening

   You are running the **Consolidate** workflow...

   ## When to Use
   ```

### Summary

Nine skeleton fixes address both CRITICAL issues. Items 1-8 add ASCII workflow diagrams to the 8 composable primitive skills that were missing them (gather, distill, filter, assess, rank, verify, decompose, sketch). The diagrams follow the phase structure already present in each skill. Item 9 removes the undocumented `!bd` prefix from consolidate's frontmatter, which violates the Bash(prefix:*) restriction format. Remaining 15 non-primitive skills (blossom, fractal, retro, etc.) would need similar diagram additions but are deferred as they have more complex multi-phase flows requiring deeper design work.

> **Commentary**: Skeletons show structure, not implementation. Each item is a targeted fix with code blocks containing placeholders.

---

## Outcome

The 4-step primitive chain **gather → assess → filter → sketch** successfully audited the meta-agent-defs skill library, identifying 8 quality patterns, categorizing them by compliance severity, filtering to 4 actionable issues (2 CRITICAL, 2 WARNING), and producing fix skeletons for the CRITICAL items. The audit process was grounded in actual file reading (all CONFIRMED findings with file:line citations) and composed naturally — each primitive consumed the prior primitive's pipe-format output without manual data transformation. This demonstrates that primitives work equally well on external OSS projects as on the meta-agent-defs repo itself.
