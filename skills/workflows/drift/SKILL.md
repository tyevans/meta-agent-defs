---
name: drift
description: "Use after major skill edits or when definitions feel inconsistent. Detects shared patterns to factor out, missed cross-pollination, and structural divergence. Keywords: drift, convergence, divergence, cross-pollination, consistency, compare."
argument-hint: "<category or glob pattern>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git:*), Task
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

**Source-breadth assessment**: After resolving the file list, count the total number of files. If the count exceeds 12, use **parallel mode** for Phase 1. Otherwise, use **serial mode (default)**.

---

## Phase 1: Read Definitions

### Serial mode (default, ≤12 files)

Read all resolved files directly. For each file, extract:
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

### In parallel mode (>12 files):

Split the resolved file list into two roughly equal halves. Dispatch 2 background Explore agents, one per half. Each agent reads its subset and returns a partial inventory in the same format as serial mode.

#### Agent prompt template

```
You are a drift inventory agent. Read the following definition files and extract
a structured inventory for each one.

Files to read:
<LIST OF FILE PATHS — one per line>

For each file, output a block in this exact format:

File: <path>
  Sections: [<comma-separated list of ## and ### headers>]
  Frontmatter: {<field>: <value>, ...}

Rules:
- List section headers in document order, normalized to their exact text
- For frontmatter, include all YAML key-value pairs
- If a file is missing or unreadable, output: File: <path> — UNREADABLE
- Output ONLY the inventory blocks — no preamble, no summary
```

Dispatch both agents concurrently with `run_in_background: true`. Collect their outputs, then merge the partial inventories by concatenating the blocks into a single unified inventory before proceeding to Phase 2.

**Merge rule**: If the same file appears in both outputs (e.g., due to a split overlap), keep the first agent's entry. Entries from Agent 1 take precedence over entries from Agent 2 for any path conflict.

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

### 2e. Convergence Alerts

Detect when recent commits to one file should trigger cross-pollination to sibling files.

**Check for recent changes:**
```bash
git log --since="7 days ago" --name-only --pretty=format:"%H|%ad|%s" --date=short -- <files-from-target-set>
```

For each recent commit that modified files in the target set:

1. **Extract changed sections** from the commit:
   ```bash
   git show <commit-hash> -- <file> | grep "^+##\|^-##"
   ```

2. **Check if changed sections exist in sibling files**: For each section header that was added or modified, check if the same section (fuzzy match on first 3 words) exists in other files in the target set.

3. **Flag cross-pollination candidates**: If a changed section appears in >50% of sibling files but was NOT updated in those siblings, flag it as a cross-pollination candidate.

4. **Format each alert**:
   - Commit hash and date
   - Section that was changed
   - File where the change happened
   - Sibling files that have the same section but weren't updated

**Scope**: Only check `feat:` and `fix:` commits (filter by commit message prefix). Ignore `chore:`, `docs:`, and `refactor:` commits as these are less likely to need cross-pollination.

---

## Phase 3: Git Analysis

For significant divergences found in Phase 2, use git history to determine when the divergence started.

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

4. **Cross-pollination needed: <section>** — Updated in <file1> (<commit>) but not in <file2, file3>
   - source: <commit-hash>, <date>
   - confidence: LIKELY

5. **Frontmatter inconsistency: <field>** — Different values across similar files
   - source: <file1:value1, file2:value2>
   - confidence: CONFIRMED

### Summary

<One paragraph synthesis: overall consistency state, highest-impact cross-pollination opportunities, and whether factoring out shared patterns is warranted.>
```

---

## Guidelines

1. **Compaction resilience**: Per `rules/memory-layout.md`, checkpoint at phase boundaries to `.claude/tackline/memory/scratch/drift-checkpoint.md`. In parallel mode, write the merged inventory to the checkpoint before proceeding to Phase 2 — agent outputs are not recoverable after context loss.
2. **Compare apples to apples.** Only compare files in the same category (workflow skills to workflow skills, agents to agents). Don't flag divergence between unrelated file types.
3. **Fuzzy section matching.** "When to Use" and "When To Use This" are the same section. Match on normalized form (lowercase, first 3 words).
4. **Threshold for "shared".** A pattern is "shared" if it appears in ≥3 files OR ≥50% of files in the category, whichever is smaller.
5. **Git timeline optional.** If git history is complex, skip timeline and focus on structural comparison.
6. **No false positives on intentional differences.** If files have legitimately different structures (e.g., composable primitives vs workflow skills), don't flag as divergence.
7. **Cite specific examples.** Every finding should reference actual file paths and section names, not abstract claims.
8. **Actionable output.** The summary should suggest concrete next steps: "Consider factoring <pattern> into <rule-file>" or "Cross-pollinate <section> from <file1> to <file2>."
9. **Cross-pollination alerts are LIKELY confidence (not CONFIRMED).** Section-name matching is fuzzy and the change may be intentionally file-specific. Manual review is required to determine if cross-pollination is warranted.
