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
Determine scope (what to review)
  -> Read and understand changes
    -> Evaluate across dimensions
      -> Produce findings with severity
        -> Summary and next actions
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

---

## Phase 3: Evaluate

Review across these dimensions. For each dimension, note any findings.

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

## Phase 4: Produce Findings

### Severity Levels

| Severity | Meaning | Action |
|----------|---------|--------|
| **CRITICAL** | Bug, security flaw, data loss risk, or correctness issue | Must fix before merge |
| **WARNING** | Likely problem, missing validation, or fragile pattern | Should fix before merge |
| **SUGGESTION** | Better approach exists, readability improvement | Consider for this or follow-up |
| **NITPICK** | Style preference, minor inconsistency | Optional, author's discretion |

### Finding Format

For each finding:

```markdown
### [SEVERITY]: [brief title]

**Location**: `file/path.ext:line_number`
**Dimension**: [correctness | security | style | architecture | testing]

**Issue**: [What is wrong or could be better]

**Evidence**: [The specific code, pattern, or behavior that triggered this finding]

**Suggestion**: [What to do instead, with code example if applicable]
```

---

## Phase 5: Summary and Next Actions

### 5a. Review Summary

```markdown
## Review Summary

**Target**: [what was reviewed]
**Files reviewed**: N
**Findings**: C critical, W warnings, S suggestions, N nitpicks

### Verdict: [PASS | PASS WITH CONDITIONS | FAIL]

**Conditions** (if applicable):
- [ ] [condition that must be met before merge]

### Critical Findings
[list critical findings with file:line references]

### Warnings
[list warning findings with file:line references]

### Suggestions
[list suggestions]

### Nitpicks
[list nitpicks]

### What Looks Good
- [positive observations -- things done well worth noting]
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

### 5c. Session Close Reminder

```bash
bd sync --flush-only
git status
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
