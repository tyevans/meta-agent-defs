# TODO App Demo: Primitives Showcase

A minimal TODO API with intentional issues designed to demonstrate composable primitives in action.

## What This Is

This is a static demo project for reading and analysis — NOT meant to be run. The codebase contains 6 deliberate issues across security, architecture, and testing that primitives are designed to discover.

## The TODO App

A simple Express REST API with:
- JWT authentication (weak secret)
- SQLite database (SQL injection vulnerabilities)
- CRUD operations for todos (no input validation)
- Minimal test coverage (missing error cases)

**Total lines of code**: ~260 across 7 files

## Intentional Issues

| Issue | Location | Type | Why It's Here |
|-------|----------|------|---------------|
| SQL injection | `src/db.js` | Security | String interpolation in queries |
| No input validation | `src/routes.js` | Security | Unchecked user input |
| Hardcoded JWT secret | `src/auth.js` | Security | `'secret123'` in source |
| Missing test cases | `test/app.test.js` | Testing | Only happy paths covered |
| Mixed concerns | `src/routes.js` | Architecture | Business logic in routes |
| No pagination | `src/routes.js` | Performance | List endpoint unbounded |

## Primitive Chains: 3 Scenarios

### Scenario 1: Security Audit

**Goal**: Find, categorize, prioritize, and verify security vulnerabilities.

**Chain**: gather → assess → rank → verify

```bash
# Step 1: Find all security issues
/gather security vulnerabilities in demos/todo-app/

# Step 2: Categorize by severity
/assess by severity using OWASP criteria

# Step 3: Prioritize by exploitability
/rank by exploitability

# Step 4: Verify the top 3 findings
/verify top 3 findings
```

**Why this chain**:
- `/gather` searches the codebase for security patterns (SQL injection, hardcoded secrets, etc.)
- `/assess` applies a discrete rubric (OWASP severity: Critical/High/Medium/Low) to categorize findings
- `/rank` orders by continuous score (exploitability = likelihood × impact) to prioritize fixes
- `/verify` checks if the top findings are genuine vulnerabilities (CONFIRMED) vs false positives (POSSIBLE)

**Expected output snippets**:

After `/gather`:
```
### Items

1. **SQL injection in getAllTodos** — String interpolation in query: `SELECT * FROM todos WHERE user_id = ${userId}`
   - source: src/db.js:19
   - confidence: CONFIRMED

2. **Hardcoded JWT secret** — Secret stored as plain string 'secret123'
   - source: src/auth.js:4
   - confidence: CONFIRMED

3. **No input validation on POST /todos** — Title parameter not validated
   - source: src/routes.js:33
   - confidence: CONFIRMED
```

After `/rank`:
```
### Items

1. **SQL injection in getAllTodos** — Exploitability score: 9.5/10 (trivial to exploit, direct data access)
2. **SQL injection in createTodo** — Exploitability score: 9.0/10 (trivial to exploit, can inject arbitrary SQL)
3. **Hardcoded JWT secret** — Exploitability score: 7.5/10 (requires token observation, enables full auth bypass)
```

---

### Scenario 2: Code Quality Review

**Goal**: Find architectural issues and compare refactoring approaches.

**Chain**: gather → filter → diff-ideas

```bash
# Step 1: Find all code quality issues
/gather code quality issues in demos/todo-app/

# Step 2: Focus on architecture only
/filter keep only architectural concerns

# Step 3: Compare refactoring strategies
/diff-ideas fix routes.js inline vs extract service layer
```

**Why this chain**:
- `/gather` finds all quality issues (naming, structure, coupling, duplication)
- `/filter` narrows to just architectural concerns (mixed responsibilities, layering violations)
- `/diff-ideas` compares two approaches side-by-side with tradeoffs for informed decision-making

**Expected output snippets**:

After `/filter`:
```
### Items

1. **Mixed concerns in routes.js** — Route handlers contain business logic and database calls
   - source: src/routes.js:1-70

2. **No service layer** — Database accessed directly from routes, limiting testability
   - source: src/routes.js:11,21,35,48,60
```

After `/diff-ideas`:
```
### Comparison

| Dimension | Inline Fix | Extract Service Layer |
|-----------|------------|----------------------|
| **Effort** | Low (1-2 hours) | Medium (4-6 hours) |
| **Testability** | No improvement | High improvement — can unit test service |
| **Maintainability** | Marginal improvement | Significant improvement — clear separation |
| **Risk** | Low — minimal changes | Medium — touches all routes |

### Summary

Inline fixes improve readability but don't address the root architectural issue. Extracting a service layer requires more upfront work but enables proper unit testing and cleaner separation of concerns. For a demo project, inline is sufficient. For production, service layer extraction is the correct long-term choice.
```

---

### Scenario 3: Test Gap Analysis

**Goal**: Identify untested code paths and edge cases, then synthesize into a prioritized list.

**Chain**: decompose → gather → gather → merge → distill

```bash
# Step 1: Break down test coverage needs
/decompose test coverage for demos/todo-app/

# Step 2: Find untested code paths
/gather untested code paths in demos/todo-app/src/

# Step 3: Find edge cases for the API
/gather edge cases for the TODO API

# Step 4: Combine both gather outputs
/merge

# Step 5: Synthesize to top 5 priorities
/distill to 5 bullets
```

**Why this chain**:
- `/decompose` breaks test coverage into measurable components (endpoints, error paths, edge cases)
- `/gather` (first call) finds specific untested code paths in the source
- `/gather` (second call) finds edge cases from the API's perspective
- `/merge` combines both lists into a unified view (the ONLY primitive that reads multiple pipe-format blocks)
- `/distill` reduces to the 5 most important gaps

**Expected output snippets**:

After first `/gather`:
```
### Items

1. **PUT /todos/:id** — No tests for update endpoint
   - source: src/routes.js:44-58

2. **DELETE /todos/:id** — No tests for delete endpoint
   - source: src/routes.js:60-68

3. **Error handling in auth middleware** — Invalid token path not tested
   - source: src/auth.js:20-22
```

After second `/gather`:
```
### Items

1. **Empty title in POST /todos** — No validation for missing or empty title field
2. **Very long title (>1000 chars)** — No length limit enforced
3. **SQL injection attempt** — Crafted input like `'; DROP TABLE todos; --`
4. **Invalid todo ID (non-numeric)** — GET /todos/abc not tested
```

After `/merge`:
```
### Items

1. **PUT /todos/:id** — No tests for update endpoint
   - source: src/routes.js:44-58

2. **DELETE /todos/:id** — No tests for delete endpoint
   - source: src/routes.js:60-68

3. **Error handling in auth middleware** — Invalid token path not tested
   - source: src/auth.js:20-22

4. **Empty title in POST /todos** — No validation for missing or empty title field

5. **Very long title (>1000 chars)** — No length limit enforced

6. **SQL injection attempt** — Crafted input like `'; DROP TABLE todos; --`

7. **Invalid todo ID (non-numeric)** — GET /todos/abc not tested
```

After `/distill`:
```
### Summary

Top 5 test gaps:
1. Missing CRUD tests: PUT and DELETE endpoints have zero coverage
2. Error path gaps: Invalid token, missing title, and database errors not tested
3. Input validation absent: No tests for edge cases like empty strings, oversized payloads, or type mismatches
4. Security untested: SQL injection attempts and auth bypass scenarios not verified
5. Edge case blindness: Non-numeric IDs, special characters, and unicode handling unchecked
```

---

## How Primitives Compose

Each primitive emits markdown in **pipe format** (structured output with numbered items). The next primitive in the chain reads that output from conversation context and operates on it.

**No file passing. No explicit piping syntax. Context IS the pipe.**

Key patterns from these scenarios:

1. **Gather first** — Almost every chain starts with `/gather` because other primitives need structured items to work on
2. **Assess for categories, rank for priority** — Use `/assess` when you need discrete bins (Critical/High/Medium), use `/rank` when you need ordering (1st, 2nd, 3rd)
3. **Merge reads multiple blocks** — The ONLY primitive that combines outputs from multiple prior primitives
4. **Distill at the end** — Reduces any pipe-format list to N essential bullets

## Next Steps

Try running these chains in a Claude Code session. The primitives will parse the source code, apply the analysis, and emit structured findings in pipe format that you can review and act on.

For more primitive combinations, see `/home/ty/workspace/tackline/rules/primitives-cookbook.md`.
