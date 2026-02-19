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

If `$ARGUMENTS` is empty or too vague, ask one clarifying question about the trigger, expected outcome, or data flow. Do not over-question -- tracer discovers details through implementation, not upfront specification.

### 1b. Identify Layers and Boundaries

Read the project structure to identify every layer or boundary the feature must cross (UI, controller, service, data access, external systems, infrastructure). Use Glob and Read to discover where similar features are implemented and what patterns each layer follows.

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

Implement the absolute minimum at each layer to make **one successful end-to-end request** work. You think like a tunneler cutting through rock -- no error handling, no edge cases, no validation beyond null safety. Hard-code freely (test IDs, sample data, stubbed externals). Skip auth, logging, rate limiting. The only goal is proving the path exists.

### 2b. Implement Bottom-Up

Start at the deepest layer (database/external system) and work up to the UI so each layer has a working foundation. At each layer: match existing patterns, implement minimal code, verify it compiles.

### 2c. Verify End-to-End

Run the application and execute the happy path manually. If it fails, debug and fix before proceeding.

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

For each layer, identify what can fail: connection loss, timeouts, invalid input, missing resources, permission denied, network failures, resource exhaustion.

### 3b. Implement Error Handling

You think like a defensive programmer adding safety nets to a working path. For each failure mode: detect it, recover or report clearly to the caller, and propagate appropriately (never swallow silently). Follow the project's existing error conventions (exceptions vs. Result types, logging patterns, error response format).

### 3c. Verify Error Paths

Simulate each failure (disconnect DB, mock failing API, send invalid input) and verify correct handling, appropriate user-facing messages, and graceful degradation (no crashes, no corrupt state).

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

You think like a tester probing boundaries: empty inputs, boundary values (zero, max, empty sets), duplicate/concurrent operations, large inputs, and special characters (unicode, injection vectors).

### 4b. Add Input Validation

At each layer's entry point, validate required fields, types/formats, ranges, and sanitize input. Return clear validation errors.

### 4c. Replace Hard-Coded Values

Replace all Phase 2 hard-coded values (test IDs, sample data, stubs) with real implementations -- parameters, configuration, or dynamic generation.

### 4d. Handle Edge Cases

Implement handling for each identified edge case and verify the behavior.

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

Choose test levels appropriate for this feature (unit, integration, e2e). Follow project conventions. If the project has no tests, create at least basic integration tests for the core path.

### 5b. Write Tests

Match existing test style. Cover happy path (Phase 2), error scenarios (Phase 3), and edge cases (Phase 4) in priority order. Run tests, and if they fail, fix the implementation (not the tests).

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

### 6b. Persist Architectural Insights to Domain Memory

After completing the report, extract architectural findings discovered during implementation and persist them to `memory/project/domain.md`.

**What to extract** — look for concepts that emerged during the tracer that are project-specific and would help a future session understand this codebase:
- Layer names or boundary names specific to this project (e.g., "the service layer means X here, not Y")
- Naming conventions or patterns that differ from common usage
- Architectural terms that have a specific meaning in this codebase
- Data flow or integration patterns that are non-obvious
- Constraints or invariants discovered during implementation

**What NOT to extract**:
- General programming concepts with standard meanings
- Findings that are only relevant to the specific feature implemented (not the broader codebase)
- Speculation — only persist what was confirmed by reading actual code

**Format** — for each extracted concept, use the domain format exactly:

```
## <Term>
**Definition**: <one sentence — what this term means in this project>
**Disambiguation**: <how this meaning differs from common use, if applicable — omit line if not ambiguous>
**Added**: YYYY-MM-DD
```

**Process**:
1. Read `memory/project/domain.md` if it exists. If it does not, create it with the header:
   ```markdown
   # Project Domain Terminology

   Terms and disambiguation rules for this project. Maintained by `/domain`.

   ---

   ```
2. For each concept to persist:
   - Check if a `## <Term>` heading already exists (case-insensitive). If it does, skip — do not duplicate.
   - If new, append the entry below the last entry in the file.
3. If no architectural insights were found worth persisting, skip this step entirely — do not write an empty or placeholder entry.
4. After writing, confirm in the report output: "Persisted N domain concepts to `memory/project/domain.md`." (or "No new domain concepts to persist." if none).

### 6c. Create Follow-Up Tasks

If follow-up work was identified:

**If `.beads/` exists in the project root**, create beads tasks:

```bash
bd create --title="[follow-up task]" --type=task --priority=[0-4] \
  --description="Follow-up from tracer: [feature name]. [Details and context.]"
```

**If `.beads/` does not exist**, write follow-up tasks to `TODO.md` in the project root:

```markdown
## Follow-up from tracer: [feature name]

- [ ] [follow-up task] — [details and context]
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
10. **Create beads for follow-up work.** If scope is cut or limitations are discovered, document them as beads tasks (or in `TODO.md` if `.beads/` is not present).
