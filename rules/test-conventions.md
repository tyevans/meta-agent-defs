---
strength: should
freshness: 2026-02-21
---

# Test Conventions

When writing tests, follow these conventions. They apply regardless of language or framework.

## Naming: Given / When / Then

Encode the three facets in structure, not in a framework:

- **Class or describe block name = Given** (precondition state)
- **Test method name = When + Then** (action + expected outcome)
- Inline comments reinforce the three facets when the method name alone isn't enough

```python
class TestSearchWithPopulatedDatabase:
    """Given a database with users Jane Doe, Jane Smith, and John Doe."""

    def test_search_by_first_name_returns_matching_users(self, populated_db):
        # When searching for "Jane"
        results = search_users(query="Jane", db=populated_db)
        # Then returns both Jane entries but not John
        assert {r.full_name for r in results} == {"Jane Doe", "Jane Smith"}
```

## Provenance Metadata

Every test class or module includes a one-line `Source:` comment. This is triage metadata — when a test breaks, Source tells you where the authority for expected behavior lives.

| Type | Example |
|------|---------|
| Codified | `Source: OpenAPI spec field constraints` |
| Articulated | `Source: USER-123 acceptance criteria` |
| Emergent | `Source: discovered during implementation, no prior spec` |

## Directory Organization

- Mirror the source tree under a `tests/` prefix
- Organize by behavioral domain: `tests/users/test_search.py`, not `tests/test_user_service.py`
- Group tests by precondition state using classes or describe blocks

## What to Test vs Skip

**Test:**
- Public function signatures and return types
- Error paths with explicit handling (raised exceptions, error codes)
- Integration points between components
- Branching logic — every if, loop guard, or external call that can go two ways

**Skip:**
- Private helpers called only by tested public functions
- Pure delegation (a method that only calls another with no logic)
- Trivial accessors and config wiring

**Heuristic:** If deleting the test would let a real bug ship undetected, keep it. If deleting would only let a harmless refactor proceed, cut it.

## Behavioral, Not Structural

- Assert on inputs and outputs at boundaries, never on internal method calls or internal data structures
- A pure refactor must not break any test — if it does, the test is testing implementation details, not behavior
- Do not assert on call counts, call order, or intermediate state unless those are the actual contract
