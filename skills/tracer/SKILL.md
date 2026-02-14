---
name: tracer
description: "Build a feature iteratively by implementing the thinnest end-to-end path first, then widening pass by pass. Use when integration risk is high, for features crossing system boundaries, or when you want always-working increments. Keywords: tracer, iterative, incremental, end-to-end, thin slice, vertical slice."
argument-hint: "<feature to implement>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Write, Edit, Bash(bd:*), Bash(git:*), Bash(npm:*), Bash(npx:*), Task
context: fork
---

# Tracer: Iterative End-to-End Implementation

You are running the **Tracer** workflow -- a tracer bullet approach to feature implementation. Build the thinnest possible end-to-end path first, validate it works, then iteratively widen by adding one concern at a time. Feature: **$ARGUMENTS**

## When to Use

- When integration risk is high (feature crosses multiple boundaries or systems)
- For vertical slices that span UI, API, service, and persistence layers
- When you want always-working increments instead of big-bang integration
- When the feature has unclear requirements but a clear happy path
- When parallelizing work (teammates can widen different passes simultaneously)

## Overview

Tracer works in 6 phases. Each phase produces a working, committable increment.

```
Map the path (identify all layers/boundaries)
  -> Tracer (hardcoded happy path, no error handling)
    -> Widen: error handling (what breaks, how to recover)
      -> Widen: edge cases & validation (boundary conditions, replace hardcoded values)
        -> Widen: tests (verify all passes)
          -> Report (summary, gaps, remaining work)
```

---

## Phase 1: Map the Path

### 1a. Clarify the Feature

If `$ARGUMENTS` is empty or too vague, ask the user one clarifying question to understand:
- What user action triggers this feature?
- What is the expected outcome?
- What data flows through the system?

Do not over-question -- the point of tracer is to discover details through implementation, not upfront specification.

### 1b. Identify Layers and Boundaries

Read the project structure to understand the architecture. Identify every layer or boundary the feature must cross:

**Common layers:**
- **UI/Interface**: Component, screen, CLI command, or API endpoint
- **Controller/Handler**: Request routing, input parsing, response formatting
- **Service/Application**: Business logic, orchestration, domain operations
- **Data Access**: Repository, ORM, database queries
- **External Systems**: Third-party APIs, message queues, file storage
- **Infrastructure**: Configuration, logging, monitoring

Use Glob and Read to discover:
- Where similar features are implemented
- What patterns the codebase follows at each layer
- What dependencies exist between layers

### 1c. Create the Path Map

Produce a visual map showing each layer and what the tracer will touch:

```markdown
## Path Map: [feature name]

**User action:** [e.g., "User clicks 'Export' button"]
**Expected outcome:** [e.g., "CSV file downloads with filtered data"]

**Layers to cross:**
1. **UI Component** (`src/components/Export.tsx`)
   - Add export button, wire click handler

2. **API Client** (`src/api/export.ts`)
   - Add exportData(filters) method

3. **API Endpoint** (`src/routes/export.ts`)
   - POST /api/export handler

4. **Service Layer** (`src/services/ExportService.ts`)
   - generateExport(userId, filters) method

5. **Data Access** (`src/repositories/DataRepository.ts`)
   - fetchRecords(userId, filters) query

6. **Database**
   - Query existing records table

**External boundaries:**
- [None | Third-party API | File storage | etc.]

**Existing patterns to follow:**
- [e.g., "Import feature at src/features/import/ shows full layer structure"]
```

---

## Phase 2: Tracer (Happy Path)

### 2a. Implementation Strategy

Implement the absolute minimum at each layer to make **one successful end-to-end request** work. This is the tracer bullet.

**Rules for Phase 2:**
- No error handling (let it crash if something goes wrong)
- No edge case handling (only the happy path)
- No input validation beyond preventing null pointer errors
- Hard-code where it simplifies (test user ID, sample filters, mock data)
- Stub external systems with hard-coded responses
- Skip non-critical features (auth, logging, rate limiting)

**Goal:** Prove the path exists and works end-to-end.

### 2b. Implement Bottom-Up

Start at the deepest layer (database/external system) and work up to the UI. This ensures each layer has a working foundation when you implement it.

For each layer:
1. Read existing implementations in that layer to match patterns
2. Implement the minimal viable code
3. Verify it compiles/runs (fix syntax errors immediately)

### 2c. Verify End-to-End

Once all layers are wired:
1. Run the application (start server, launch UI, etc.)
2. Execute the feature's happy path manually
3. Verify the expected outcome occurs
4. If it fails, debug and fix before proceeding

### 2d. Commit the Tracer

```bash
git add [files modified]
git commit -m "$(cat <<'EOF'
tracer: [feature name] happy path

Implemented thinnest end-to-end path across [N] layers.
- [Layer 1]: [what was added]
- [Layer 2]: [what was added]
...

Hard-coded: [list any hard-coded values to replace in Phase 4]
No error handling, validation, or tests yet.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Phase 3: Widen — Error Handling

### 3a. Enumerate Failure Modes

For each layer, identify what can fail:

**Common failure modes:**
- Database connection lost
- External API timeout or error response
- Invalid input (malformed, missing fields)
- Resource not found (user, record, file)
- Permission denied
- Network failure
- Disk full / resource exhaustion

### 3b. Implement Error Handling

For each failure mode:
1. **Detect**: Add checks or try-catch blocks to detect the failure
2. **Recover or Report**: Either recover gracefully (retry, fallback) or report a clear error to the caller
3. **Propagate**: Ensure errors bubble up through layers appropriately (don't swallow errors silently)

**Follow project conventions:**
- Does the codebase use exceptions, error codes, or Result types?
- Where are errors logged?
- What format does the UI expect for error messages?

### 3c. Verify Error Paths

Test each error scenario:
1. Simulate the failure (disconnect DB, mock failing API, send invalid input)
2. Verify the error is handled correctly
3. Verify the user sees an appropriate error message
4. Verify the system recovers or fails gracefully (no crashes, no corrupt state)

### 3d. Commit Error Handling

```bash
git add [files modified]
git commit -m "$(cat <<'EOF'
tracer: [feature name] error handling

Added error handling at all layers:
- [Layer 1]: [what errors are handled]
- [Layer 2]: [what errors are handled]
...

Tested: [list failure scenarios verified]

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Phase 4: Widen — Edge Cases & Validation

### 4a. Identify Edge Cases

For each layer, identify boundary conditions and edge cases:

**Common edge cases:**
- Empty input (empty string, empty array, null)
- Boundary values (zero, negative numbers, max int, empty result set)
- Duplicate operations (retry, double-submit)
- Concurrent operations (race conditions)
- Large input (pagination, memory limits)
- Special characters (unicode, SQL injection, XSS)

### 4b. Add Input Validation

At each layer's entry point, add validation:
1. Check required fields are present
2. Validate types and formats (email, URL, date, etc.)
3. Validate ranges and boundaries
4. Sanitize input (escape special characters, trim whitespace)

Return clear validation errors to the caller.

### 4c. Replace Hard-Coded Values

Find all hard-coded values from Phase 2 (test user IDs, sample filters, mock data) and replace them with real implementations:
- Accept values as parameters
- Fetch values from configuration
- Generate values dynamically

### 4d. Handle Edge Cases

For each identified edge case:
1. Implement logic to handle it correctly
2. Verify the behavior (manually or with a quick test)

### 4e. Commit Edge Cases & Validation

```bash
git add [files modified]
git commit -m "$(cat <<'EOF'
tracer: [feature name] edge cases and validation

Added validation at all entry points:
- [Layer 1]: [what is validated]
- [Layer 2]: [what is validated]
...

Handled edge cases: [list]
Replaced hard-coded: [list]

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Phase 5: Widen — Tests

### 5a. Determine Test Coverage

Decide what level of testing is appropriate for this feature:

**Test levels:**
- **Unit tests**: Individual functions, pure logic, edge cases
- **Integration tests**: Layer interactions, database queries, external API calls
- **End-to-end tests**: Full user workflow, UI interactions

Follow project conventions. If the project has no tests, create at least basic integration tests for the core path.

### 5b. Write Tests

For each test level:
1. Read existing tests to match project style and tooling
2. Write tests covering:
   - Happy path (Phase 2)
   - Error scenarios (Phase 3)
   - Edge cases (Phase 4)
3. Run tests, verify they pass
4. If tests fail, fix the implementation (not the tests)

**Prioritize:**
- Critical path tests first (happy path, most common errors)
- Edge case tests second
- Exhaustive coverage third (only if time permits)

### 5c. Commit Tests

```bash
git add [files modified]
git commit -m "$(cat <<'EOF'
tracer: [feature name] tests

Added [unit/integration/e2e] tests:
- Happy path: [coverage]
- Error handling: [coverage]
- Edge cases: [coverage]

Test results: [summary of test run]

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Phase 6: Report

### 6a. Feature Summary

Produce a final report:

```markdown
## Tracer Report: [feature name]

### Implementation Summary

**Passes completed:**
1. **Tracer (Happy Path)**: [date/commit] — Implemented end-to-end in [N] layers
2. **Error Handling**: [date/commit] — Added handling for [N] failure modes
3. **Edge Cases & Validation**: [date/commit] — Validated [N] conditions, handled [M] edge cases
4. **Tests**: [date/commit] — Added [N] tests covering [unit/integration/e2e]

**Layers modified:**
- [Layer 1]: [files changed, LOC added]
- [Layer 2]: [files changed, LOC added]
...

**Commits:**
- [commit hash]: tracer: [feature] happy path
- [commit hash]: tracer: [feature] error handling
- [commit hash]: tracer: [feature] edge cases and validation
- [commit hash]: tracer: [feature] tests

### Test Coverage

| Test Level | Count | Status |
|------------|-------|--------|
| Unit | N | ✓ Passing |
| Integration | M | ✓ Passing |
| E2E | K | ✓ Passing |

**Scenarios covered:**
- Happy path: ✓
- [Error scenario 1]: ✓
- [Error scenario 2]: ✓
- [Edge case 1]: ✓
- [Edge case 2]: ✓

### Remaining Work

**Out of scope (intentional):**
- [Feature/concern not addressed in this tracer]
- [Reason it was descoped]

**Follow-up tasks:**
- [Task that should happen next]
- [Task that depends on this feature]

**Known limitations:**
- [Limitation 1 and why it exists]
- [Limitation 2 and why it exists]
```

### 6b. Create Follow-Up Tasks

If follow-up work was identified, create beads tasks:

```bash
bd create --title="[follow-up task]" --type=task --priority=[0-4] \
  --description="Follow-up from tracer: [feature name]. [Details and context.]"
```

---

## Guidelines

1. **Each phase produces a working increment.** Never proceed to the next phase if the current one is broken.
2. **Commit after every phase.** Commits provide rollback points and document the tracer's progression.
3. **Hard-coding in Phase 2 is expected.** The point is to prove the path, not build production code immediately.
4. **Bottom-up implementation reduces risk.** Each layer has a working foundation when you build on it.
5. **Phase order is flexible.** If tests before error handling makes more sense for the feature, reorder accordingly.
6. **Verify after every widening pass.** Run the feature manually after Phases 3, 4, and 5 to ensure nothing broke.
7. **Stop at any phase.** If the session ends early, you have working code at the last committed phase.
8. **Widen one concern at a time.** Don't mix error handling and validation in the same pass -- each pass is focused.
9. **Follow project conventions.** Match existing patterns for error handling, validation, testing, and file organization.
10. **Create beads for follow-up work.** If scope is cut or limitations are discovered, document them as beads tasks.
