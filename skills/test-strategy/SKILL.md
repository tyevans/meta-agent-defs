---
name: test-strategy
description: "Guide implementation agents through a structured testing workflow: classify knowledge source (codified/articulated/tacit), write tests from specs, enforce red-green gates, and fall back gracefully. Use when dispatching implementation tasks or before writing code. Keywords: test, TDD, BDD, testing, test-first, test-after, red-green, spec, acceptance criteria."
argument-hint: "[task description or bead reference]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git:*), Bash(bd:*)
context: fork
---

# Test Strategy: Structured Testing Workflow

You are running the **test-strategy** skill — a structured testing workflow that matches the testing approach to the nature of the specification. Task: **$ARGUMENTS**

## When to Use

- Before implementing a feature, bug fix, or refactor to determine the right testing approach
- When dispatching an implementation task to an agent and you want tests baked in
- When a task description includes acceptance criteria, user stories, or a bug repro
- When a formal spec (OpenAPI, proto, JSON Schema) exists and tests should be generated from it
- When no spec exists and the implementation is exploratory (tacit knowledge path)
- When an existing implementation needs retroactive test coverage

## Overview

```
Phase 0 — Classify
  Glob/grep for spec artifacts + scan task description
    -> evidence-based classification into epistemic level
      |
      +-- Codified (formal spec) ---> Phase 1 (generate mechanically)
      |                                    |
      +-- Articulated (NL spec)   ---> Phase 1 (transcribe faithfully)
      |                                    |
      +-- Tacit (no spec)         ---> Phase 3 (implement first)
                                          |
                                   Phase 2 — Red Gate
                                     confirm ALL new tests fail
                                       |
                                   Phase 3 — Green Implementation
                                     implement; 3 attempts max
                                       |
                                   Phase 4 — Test-After (tacit or escape)
                                     document boundaries discovered
```

---

## Phase 0: Classify — Determine the Epistemic Level

**This phase is near-zero cost. Do it before writing a single line of code or tests.**

### 0a. Scan for Spec Artifacts

Search the project for formal specification artifacts:

```bash
# OpenAPI / Swagger
glob: openapi.yaml, openapi.json, swagger.yaml, swagger.json, **/*.oas.yaml

# Protocol Buffers
glob: **/*.proto

# JSON Schema
glob: **/*.schema.json, **/schema.json

# TypeScript / GraphQL types that double as contracts
glob: **/*.d.ts, **/*.graphql, **/*.gql

# Existing tests for the target module
glob: **/*.test.*, **/*.spec.*, **/test_*.py, **/*_test.go
```

Read any spec artifacts found that are relevant to the task.

### 0b. Scan the Task Description

Parse `$ARGUMENTS` for:

- Concrete input/output pairs (e.g., "given X, expect Y")
- Acceptance criteria ("must", "should", "when … then")
- Bug repro steps ("reproduce by", "steps to trigger")
- Example data (JSON, SQL, curl commands)

### 0c. Classify

Assign one of three epistemic levels based on evidence:

| Level | Signal | Testing Approach |
|-------|--------|-----------------|
| **Codified** | Formal spec exists (OpenAPI, proto, JSON Schema, type interface) | Generate tests mechanically from the spec; spec is ground truth |
| **Articulated** | Natural-language spec with concrete examples (user story, bug report with repro, acceptance criteria) | Write tests first; translating spec → test IS the value |
| **Tacit** | No spec; behavior is emergent or exploratory | Implement first; write tests after to document what was built |

**Default to Tacit** when evidence is absent or only vague intent is expressed. Do not fabricate a spec.

### 0d. Read Existing Tests

Read the existing test files for the target module:

- Note the test framework (pytest, Jest, Go test, RSpec, etc.)
- Note fixture and factory patterns
- Note naming conventions (test class names, test method names)
- Note assertion style (assert, expect, should)

State your classification explicitly before proceeding:

```
Classification: [Codified | Articulated | Tacit]
Evidence: [what you found that supports this]
Test framework: [framework and conventions observed]
Proceeding to: [Phase 1 | Phase 3]
```

---

## Phase 1: Author Tests (Codified and Articulated paths only)

**Tacit classification skips directly to Phase 3.**

### 1a. Three-Facet Test Naming

Use this naming convention consistently:

- **Class / describe block** = *Given* (the precondition / system state)
- **Test method / it block** = *When* + *Then* (the action and expected outcome)

Example (Python / pytest):

```python
class GivenSearchEndpointWithIndexedDocuments:
    def test_when_query_matches_returns_ranked_results(self):
        ...

    def test_when_query_is_empty_returns_bad_request(self):
        ...

    def test_when_no_results_found_returns_empty_list_not_404(self):
        ...

class GivenSearchEndpointWithEmptyIndex:
    def test_when_any_query_returns_empty_list(self):
        ...
```

Example (Jest / TypeScript):

```typescript
describe('Given search endpoint with indexed documents', () => {
  it('when query matches, returns ranked results', () => { ... });
  it('when query is empty, returns 400 Bad Request', () => { ... });
  it('when no results found, returns empty list not 404', () => { ... });
});
```

### 1b. Trace Each Assertion to Its Source

Every test assertion must have a provenance comment linking it to the specification:

```python
# Per requirement: max 3 retry attempts (openapi.yaml: POST /auth/login -> x-max-retries)
assert response.status_code == 429
assert response.json()["retries_remaining"] == 0

# Per acceptance criteria: case-insensitive search (ticket AC-3)
assert results == search(query="PYTHON") == search(query="python")
```

For **Articulated** specs, if you encounter an ambiguity the spec does not address, flag it explicitly rather than silently picking an interpretation:

```python
# Ambiguity: spec does not address pagination when results == 0.
# Assuming empty list response (not 404). Confirm with product owner or remove.
assert response.json()["results"] == []
assert response.status_code == 200
```

### 1c. Coverage Cap

Write **8-10 tests maximum** per task. Cover:
1. The primary happy path
2. 2-3 significant edge cases from the spec
3. The key error/rejection paths

Explicitly avoid:
- Testing implementation internals (private helpers, internal state)
- Redundant variations (same assertion with slightly different input)
- Speculative tests for behavior not described in the spec

### 1d. Write the Test File

Write tests to the appropriate location following the project's conventions. Commit nothing yet.

---

## Phase 2: Red Gate

**Run tests. ALL new tests must fail.**

```bash
# Run only the new test file to check initial state
pytest path/to/test_module.py -v        # Python
npm test -- --testPathPattern=module    # Jest
go test ./pkg/module/... -run TestName  # Go
```

### 2a. Interpret Results

| Outcome | Interpretation | Action |
|---------|---------------|--------|
| All new tests fail (expected) | Gate PASSED — proceed to implementation | Continue to Phase 3 |
| A test passes before any implementation | Either existing code already handles it (test is redundant) OR the test has a logic bug | Investigate before continuing |
| A test errors with import/fixture failure | Infrastructure bug, not behavioral — fix it, re-run, does not break immutability | Fix setup only, re-run |

### 2b. Immutability Declaration

Once all new tests fail for behavioral reasons, declare the tests IMMUTABLE:

```
Red Gate: PASSED
Tests frozen at: [timestamp or git hash]
All N new tests fail for behavioral reasons (not infrastructure errors).
Tests are now IMMUTABLE. Assertions may only change with written justification
referencing a specific discovered constraint (file:line evidence).
```

**No assertion changes past this point without written justification.**

---

## Phase 3: Green Implementation

Implement the feature to make the frozen tests pass.

### 3a. Implementation Constraints

- Write the simplest implementation that satisfies the tests
- Do not modify assertions in the test file
- Follow project conventions (read surrounding code before writing new code)

### 3b. Three-Attempt Limit

Run the full test suite after each implementation attempt:

```bash
# Attempt N/3: run frozen tests
pytest path/to/test_module.py -v
```

**Attempt 1:** Initial implementation. If all pass, proceed to 3d.

**Attempt 2:** If tests still fail, read the failure messages carefully. Identify the gap. Revise implementation.

**Attempt 3:** Final attempt. If any test still fails after 3 attempts, trigger the escape valve (3c).

### 3c. Escape Valve — 3 Attempts Exhausted

If 3 attempts do not yield a green suite:

1. Mark failing tests as skipped with a standard marker:

```python
@pytest.mark.skip(reason="could not satisfy after 3 attempts — requires human review")
def test_when_edge_case_returns_expected():
    ...
```

2. Fall back to **Phase 4** (test-after) for the remaining behavior.
3. Flag for user review with a summary of what was attempted and what is still failing.

### 3d. Test File Modification Protocol

If during implementation you discover a genuine constraint that makes an assertion wrong (not just hard to pass), you may modify that assertion ONLY if:

- You have file:line evidence of the constraint
- You write justification in a comment immediately above the changed assertion

```python
# JUSTIFICATION: Changed sort order from descending to ascending.
# Existing convention at utils/sorting.py:42 — all list endpoints return ascending.
# Original assertion assumed descending (spec ambiguous on sort direction).
assert results == sorted(results, key=lambda r: r["score"])
```

Unexplained assertion changes are a test quality failure.

### 3e. Full Suite Regression Check

After all tests pass, run the full module test suite (not just the new tests) to confirm no regressions:

```bash
pytest path/to/module/ -v          # Python
npm test -- --testPathPattern=dir  # Jest
go test ./pkg/module/...           # Go
```

If regressions appear, fix them before declaring done.

---

## Phase 4: Test-After (Tacit path or escape valve)

Use this phase when:
- Classification was **Tacit** (no prior spec)
- 3 implementation attempts were exhausted (escape valve from Phase 3)

### 4a. What to Test

Test boundaries **discovered during implementation**, not internals invented for coverage:

| Test this | Skip this |
|-----------|-----------|
| Public function signatures with representative inputs/outputs | Private helper functions |
| Error paths that required explicit handling | Pure delegation (functions that just call another) |
| Integration points with external systems | Trivial accessors and getters |
| Observed edge cases that caused real handling decisions | Behavior not actually implemented |

### 4b. Provenance Comment

Every test-after test must carry a provenance comment:

```python
# Source: discovered during implementation, no prior spec
# Behavior: returns None when record not found (not raises KeyError)
def test_when_record_missing_returns_none_not_exception():
    result = find_record(id="nonexistent")
    assert result is None
```

### 4c. Naming

Apply the same three-facet convention (Given/When+Then) as Phase 1.

---

## Overhead Budget

The testing workflow has a **15-20% overhead budget** on tool calls relative to the implementation alone. If you find yourself spending more than that on test infrastructure, the task has likely grown beyond its original scope. Flag this and propose scoping the task down.

---

## Guidelines

- **Classify before writing a single line.** The epistemic level determines everything else. Getting this wrong wastes effort.
- **Trust the red gate.** A test that passes before implementation exists is either redundant or wrong. Investigate before continuing.
- **Immutability is the quality guarantee.** Tests are cheap to write and easy to make pass by writing the wrong assertion. The red gate + immutability combination prevents this failure mode.
- **Flag ambiguities, do not resolve them silently.** The translation from natural-language spec to assertion is where meaning gets lost. When the spec is ambiguous, write the assumption in a comment and mark it for confirmation.
- **Test behavior, not implementation.** If refactoring the internals should not break the tests, you are testing the right thing.
- **Cap tests at 10.** More tests do not mean better coverage — they mean more maintenance surface. Prioritize meaningful behavioral boundaries over quantity.
- **Match project conventions.** A test file that uses a different framework or assertion style than the surrounding tests will be ignored or deleted. Read before writing.
- **Escape valve is not failure.** Three attempts and fall back to test-after is the designed behavior for hard cases, not a concession. Document what was attempted so the next agent or human has context.
- **Provenance is non-optional.** Every assertion traces to a source (spec line, acceptance criterion, or discovered behavior). Untraceable assertions are deleted in review.
