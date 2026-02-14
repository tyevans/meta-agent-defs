---
name: review
description: "Run a structured code review across correctness, security, style, architecture, and testing dimensions. Use after subagent implementation, before merging PRs, when reviewing commit ranges, or for a second opinion on changes."
argument-hint: "[target: staged changes, commit, range, PR#]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git:*), Bash(gh:*), Bash(bd:*)
context: fork
---

# Review: Structured Code Review

You are running a **structured code review** -- a systematic examination of changes across multiple quality dimensions. Target: **$ARGUMENTS**

## When to Use

- After a subagent completes implementation work (the REVIEW GATE reminder)
- Before merging a PR or committing significant changes
- When reviewing a commit range for quality and correctness
- When the user wants a second opinion on code they wrote

## Overview

Review works in 5 phases:

```
Gather: Determine scope + read changes
  -> Assess: Evaluate across 5 dimensions
    -> Filter: Produce findings by severity
      -> Emit pipe format output + verdict
```

---

## Phase 1: Determine Scope

### 1a. Parse the Target

Interpret `$ARGUMENTS` to determine what to review:

| Input | Interpretation |
|-------|---------------|
| *(empty)* | Staged changes: `git diff --cached` |
| A file path | Changes in that specific file |
| A commit hash | That single commit: `git show <hash>` |
| A commit range (e.g., `abc..def`) | All commits in range: `git diff <range>` |
| A PR number (e.g., `#123`) | PR changes: `gh pr diff <number>` |
| `HEAD~N` | Last N commits: `git diff HEAD~N..HEAD` |

### 1b. Fetch the Diff

```bash
# Adjust based on 1a interpretation
git diff --cached          # staged changes (default)
git show <commit>          # single commit
git diff <range>           # commit range
gh pr diff <number>        # PR
```

### 1c. Identify Affected Files

List all files touched and categorize them:
- **New files**: Need full review
- **Modified files**: Focus on changed lines + surrounding context
- **Deleted files**: Verify no remaining references
- **Renamed/moved files**: Verify all imports/references updated

### 1d. Emit Scope Summary

Emit the scope in pipe format so users can interrupt and compose if needed:

```markdown
## Review Scope

**Source**: /review (scope)
**Input**: [target from $ARGUMENTS]

### Items

1. **New**: [file paths] — [count] files added
2. **Modified**: [file paths] — [count] files changed
3. **Deleted**: [file paths] — [count] files removed
4. **Renamed**: [old -> new paths] — [count] files moved

### Summary

[One sentence describing the overall scope and change type (feature, fix, refactor, etc.)]
```

---

## Phase 2: Read and Understand

### 2a. Read Each Changed File

For every file in the diff, read the full file (not just the diff hunks). Understanding the surrounding code is essential for judging whether changes are correct.

### 2b. Trace the Change

For each logical change:
- What is the intent? (bug fix, new feature, refactor, etc.)
- What code paths are affected?
- Who calls the changed code?
- What does the changed code call?

### 2c. Check Test Coverage

- Are there tests for the changed code?
- Do existing tests still pass with these changes?
- Are new tests needed for new behavior?

### 2d. Emit Understanding Summary

Emit what you learned in pipe format for composability:

```markdown
## Review Understanding

**Source**: /review (understand)
**Input**: [target from $ARGUMENTS]

### Items

1. **[Logical change 1]** — [intent: bug fix/feature/refactor] affecting [paths]
   - source: [file:line ranges]
   - test coverage: [present/missing/partial]

2. **[Logical change 2]** — [intent and affected areas]
   - source: [file:line ranges]
   - test coverage: [present/missing/partial]

...

### Summary

[One paragraph: what changed, why, and how thoroughly it is tested.]
```

---

## Phase 3: Assess — Evaluate Across Dimensions

Evaluate changes using the **assess** pattern: apply each dimension as a rubric, assign a categorical verdict (CRITICAL/WARNING/SUGGESTION/NITPICK) per finding.

### 3a. Correctness

- Does the code do what it claims to do?
- Are edge cases handled?
- Are error paths correct?
- Are return values and types correct?
- Is state mutation safe (no race conditions, no stale reads)?

### 3b. Security (OWASP Top 10)

- **Injection**: Are inputs sanitized before use in queries, commands, or templates?
- **Broken auth**: Are authorization checks present and correct?
- **Sensitive data exposure**: Are secrets, tokens, or PII handled safely?
- **XXE / deserialization**: Are external inputs parsed safely?
- **Access control**: Are permissions checked at the right layer?
- **Misconfiguration**: Are defaults secure?
- **XSS**: Are outputs escaped in user-facing contexts?
- **Insecure dependencies**: Are imported packages known-vulnerable?
- **Logging**: Are sensitive values excluded from logs?
- **SSRF**: Are user-supplied URLs validated?

### 3c. Style Consistency

- Does the code follow the project's existing conventions?
- Naming: consistent with surrounding code?
- Formatting: consistent indentation, line length, spacing?
- Idioms: using language/framework patterns correctly?
- Comments: present where needed, absent where obvious?

### 3d. Architectural Coherence

- Does the change fit the project's architecture?
- Are abstractions at the right level?
- Are dependencies flowing in the right direction?
- Is the change in the right layer/module?
- Does it introduce coupling that should not exist?

### 3e. Test Coverage Implications

- What is the testing strategy for this change?
- Are critical paths covered by tests?
- Are new edge cases introduced that need test coverage?
- Do tests verify behavior, not implementation details?

---

## Phase 4: Filter — Produce Findings in Pipe Format

Emit all findings as a pipe-format Items list, then apply the **filter** pattern to separate must-fix from optional.

### Severity Levels (Assess Rubric)

| Severity | Meaning | Action |
|----------|---------|--------|
| **CRITICAL** | Bug, security flaw, data loss risk, or correctness issue | Must fix before merge |
| **WARNING** | Likely problem, missing validation, or fragile pattern | Should fix before merge |
| **SUGGESTION** | Better approach exists, readability improvement | Consider for this or follow-up |
| **NITPICK** | Style preference, minor inconsistency | Optional, author's discretion |

### Finding Format (Pipe Format Items)

Each finding is a numbered item following pipe format:

```markdown
1. **CRITICAL: [brief title]** — [what is wrong and what to do instead]
   - source: file/path.ext:line_number
   - dimension: correctness | security | style | architecture | testing

2. **WARNING: [brief title]** — [issue and suggestion]
   - source: file/path.ext:line_number
   - dimension: correctness | security | style | architecture | testing
```

Group items by severity (CRITICAL first, then WARNING, SUGGESTION, NITPICK). Include evidence inline in the detail text. If a code example helps, include it after the item as an indented block.

---

## Phase 5: Summary — Emit Pipe Format Output

### 5a. Review Output

Emit the full review in pipe format:

```markdown
## Reviewed [target description]

**Source**: /review
**Input**: [what was reviewed, one line]

### Items

1. **CRITICAL: [title]** — [detail with evidence and suggestion]
   - source: file/path.ext:line_number
   - dimension: correctness

2. **WARNING: [title]** — [detail]
   - source: file/path.ext:line_number
   - dimension: security

...

### Verdict

| Verdict | Files Reviewed | Critical | Warning | Suggestion | Nitpick |
|---------|---------------|----------|---------|------------|---------|
| [PASS / PASS WITH CONDITIONS / FAIL] | N | C | W | S | N |

**Conditions** (if applicable):
- [ ] [condition that must be met before merge]

### What Looks Good

- [positive observations -- things done well worth noting]

### Summary

[One paragraph synthesis: overall quality assessment, risk areas,
and recommended next actions.]
```

### Verdict Criteria

| Verdict | When |
|---------|------|
| **PASS** | No critical or warning findings |
| **PASS WITH CONDITIONS** | No critical findings, warnings exist but are fixable |
| **FAIL** | One or more critical findings |

### 5b. Create Tasks for Findings

For CRITICAL and WARNING findings, create beads tasks so they are tracked:

```bash
bd create --title="[SEVERITY]: [finding title]" --type=task --priority=<1-for-critical,2-for-warning> \
  --description="Found during code review of [target]. Location: [file:line]. Issue: [description]. Suggestion: [fix]."
```

---

## Guidelines

- **Read the full file, not just the diff** -- context around changes is where bugs hide
- **Verify, don't assume** -- trace calls, check types, read tests before flagging an issue
- **Be specific** -- every finding needs a file:line reference and concrete evidence
- **Distinguish severity honestly** -- not everything is critical; not everything is a nitpick
- **Acknowledge what is done well** -- review is not just about finding faults
- **One finding per issue** -- don't bundle unrelated problems into a single finding
- **Focus on the change, not the codebase** -- pre-existing issues are out of scope unless the change makes them worse
