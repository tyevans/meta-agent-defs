---
name: drift
description: "Detect skill/agent convergence and divergence patterns across definition files. Find shared patterns to factor out, detect missed cross-pollination, compare section structure. Use after major skill edits, as part of /retro, or when definitions feel inconsistent. Keywords: drift, convergence, divergence, cross-pollination, consistency, compare."
argument-hint: "<category or glob pattern>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git:*), Bash(tools/git-intel/target/debug/git-intel:*)
---

# Drift: Definition Convergence/Divergence Analysis

You are running **drift** — detecting convergence and divergence patterns across definition files to find shared patterns that should be factored out and detect missed cross-pollination opportunities. Target: **$ARGUMENTS**

## When to Use

- After major skill or agent edits to check for cross-pollination opportunities
- As part of `/retro` to assess definition quality and consistency
- When definitions feel inconsistent or when noticing repeated patterns across files
- Before factoring out shared patterns (like Agent Preamble was factored into fan-out-protocol.md)
- When reviewing whether one file's improvements should propagate to similar files

## How It Works

```
Parse target → Read definitions → Compare structure → Git analysis → Report
```

---

## Phase 0: Parse Target

Resolve `$ARGUMENTS` to a file list:

| Input Format | Resolution |
|--------------|------------|
| Category name (e.g., "workflow skills") | Map to glob pattern (skills/*/SKILL.md excluding composable primitives) |
| "agents" | agents/*.md |
| "composable primitives" | skills/{gather,distill,rank,filter,assess,verify,diff-ideas,sketch,decompose,critique,plan,merge}/SKILL.md |
| Glob pattern (e.g., "skills/*/SKILL.md") | Use as-is |
| Explicit file list (space-separated paths) | Use as-is |

Category mappings:
- **"workflow skills"**: skills/*/SKILL.md excluding composable primitives
- **"composable primitives"**: skills/{gather,distill,rank,filter,assess,verify,diff-ideas,sketch,decompose,critique,plan,merge}/SKILL.md
- **"agents"**: agents/*.md
- **"global agents"**: agents/*.md (same as "agents" since this repo has only global definitions)

If target cannot be resolved to at least 2 files, error: "Need at least 2 files to compare. Got: [N]"

---

## Phase 1: Read Definitions

Read all resolved files. For each file, extract:
- Full content (for section comparison)
- Section headers (lines starting with `##` or `###`)
- Frontmatter fields (if present)
- File path relative to repo root

Build an inventory:
```
File 1: <path>
  Sections: [section1, section2, ...]
  Frontmatter: {field: value, ...}

File 2: <path>
  Sections: [...]
  ...
```

---

## Phase 2: Compare Structure

### 2a. Shared Sections

Find sections that appear in multiple files with similar names (fuzzy match on first 3 words to catch "When to Use" vs "When To Use This").

For each shared section:
- Count how many files have it
- Note which files are missing it (if >50% have it but some don't)

### 2b. Unique Sections

Identify sections that appear in only one file but might be useful in others (e.g., "Guidelines" section in one skill but missing from similar skills).

### 2c. Section Order Divergence

Check if shared sections appear in different orders across files. Example: one skill has "Guidelines" before "Phases", another has it after.

### 2d. Frontmatter Divergence

Compare frontmatter fields across files in the same category:
- Are `allowed-tools` similar for similar skills?
- Is `context: fork` used consistently?
- Are descriptions following the same pattern?

---

## Phase 3: Git Analysis

For significant divergences found in Phase 2, use git history to determine when the divergence started.

Check for `tools/git-intel/target/debug/git-intel`. If available:
```bash
git-intel patterns --repo . --focus convergence --files <file1> <file2>
```

Fallback to raw git:
```bash
# Find when a section was added to file1
git log -p --all -S "<section header>" -- <file1>

# Find last common edit across files
git log --oneline --follow -- <file1> <file2>
```

For each divergence, report:
- When did the files diverge? (commit hash + date)
- What changed in one file that wasn't propagated to the other?
- Which file got the improvement first?

---

## Phase 4: Report

Emit pipe-format output:

```markdown
## Drift analysis for <category>

**Source**: /drift
**Input**: <category or pattern analyzed>
**Pipeline**: (none — working from direct input)

### Items (N)

1. **Shared pattern: <section name>** — Found in [N] of [M] files, missing from [file1, file2]
   - source: <files-with-pattern>
   - confidence: CONFIRMED

2. **Divergence: <description>** — <file1> has <section/pattern> missing from <file2, file3>
   - source: <file1:section-line>, first added in <commit-hash> (<date>)
   - confidence: CONFIRMED

3. **Convergence candidate: <description>** — <files> trending toward similar structure in <section>
   - source: git history showing parallel evolution
   - confidence: LIKELY

4. **Frontmatter inconsistency: <field>** — Different values across similar files
   - source: <file1:value1, file2:value2>
   - confidence: CONFIRMED

### Summary

<One paragraph synthesis: overall consistency state, highest-impact cross-pollination opportunities, and whether factoring out shared patterns is warranted.>
```

---

## Guidelines

1. **Compare apples to apples.** Only compare files in the same category (workflow skills to workflow skills, agents to agents). Don't flag divergence between unrelated file types.
2. **Fuzzy section matching.** "When to Use" and "When To Use This" are the same section. Match on normalized form (lowercase, first 3 words).
3. **Threshold for "shared".** A pattern is "shared" if it appears in ≥3 files OR ≥50% of files in the category, whichever is smaller.
4. **Git timeline optional.** If git-intel is unavailable or git history is complex, skip timeline and focus on structural comparison.
5. **No false positives on intentional differences.** If files have legitimately different structures (e.g., composable primitives vs workflow skills), don't flag as divergence.
6. **Cite specific examples.** Every finding should reference actual file paths and section names, not abstract claims.
7. **Actionable output.** The summary should suggest concrete next steps: "Consider factoring <pattern> into <rule-file>" or "Cross-pollinate <section> from <file1> to <file2>."
