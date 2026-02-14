# Composable Primitive Recipes

Standard chains for common workflows. Primitives compose via conversation context — each primitive reads the output of the previous one. No piping syntax needed; context IS the pipe.

## 1. Decision Pipeline

**When to use**: Choosing between architectural approaches, libraries, or design patterns when the tradeoffs aren't obvious.

**Chain**: gather → distill → diff-ideas → verify

### Example Session

```
User: /gather authentication patterns in Next.js
  → 12 findings on session-based auth, JWT, OAuth providers, middleware patterns
User: /distill to 4 approaches
  → NextAuth.js, Clerk, custom JWT, Supabase Auth
User: /diff-ideas NextAuth.js vs Clerk
  → Comparison table across DX, pricing, self-hosting, customization
User: /verify
  → Checks claims about pricing tiers, middleware support, TypeScript quality
```

### Why This Chain

Gather casts a wide net, distill narrows to viable options, diff-ideas forces explicit tradeoff analysis, and verify confirms the decision is grounded in facts rather than marketing.

---

## 2. Architecture Audit

**When to use**: Understanding how a system is built, identifying risks, or planning a refactor.

**Chain**: gather → assess → filter → sketch

### Example Session

```
User: /gather state management patterns in this codebase
  → Redux in legacy components, Zustand in new features, useState scattered everywhere
User: /assess by migration risk
  → CRITICAL: 18 Redux files with tightly coupled logic; WARNING: 7 Zustand stores with duplicated state
User: /filter keep critical only
  → 18 Redux files requiring careful migration
User: /sketch unified Zustand migration path
  → File structure, store organization, migration steps as TODO-annotated skeleton
```

### Why This Chain

Gather finds the current state, assess categorizes by severity, filter focuses on the important parts, and sketch gives you a concrete migration path without full implementation.

---

## 3. Learning Pipeline

**When to use**: Researching unfamiliar topics, evaluating documentation claims, or onboarding to a new codebase.

**Chain**: gather → distill → verify

### Example Session

```
User: /gather how beads integration works in this repo
  → Findings on bd commands, hooks, session lifecycle, backlog sync
User: /distill to 6 bullets
  → bd create/close workflow, SessionStart hook injection, sync --flush-only, epic dependencies
User: /verify
  → VERIFIED: SessionStart hook exists in settings.json (line 14); REFUTED: "bd sync runs on every commit" (no git hook found)
```

### Why This Chain

Gather collects scattered information, distill extracts essentials, verify confirms which claims are accurate. Shorter than the decision pipeline because you're learning rather than choosing.

---

## 4. Tech Debt Triage

**When to use**: Backlog grooming, sprint planning, or deciding what to tackle next from a pile of issues.

**Chain**: gather → rank → filter → decompose

### Example Session

```
User: /gather open tech debt issues in GitHub
  → 23 issues tagged "tech-debt" spanning performance, DX, security, code quality
User: /rank by security risk and effort
  → Ranked list: SQL injection in search (HIGH risk, LOW effort) at top
User: /filter security-related only
  → 4 security issues (SQL injection, XSS, outdated deps, exposed API key)
User: /decompose SQL injection fix
  → 3 sub-parts: query parameterization, input validation, test coverage
```

### Why This Chain

Gather pulls in all candidates, rank prioritizes by multiple criteria, filter narrows to the area of concern, decompose breaks the winner into actionable chunks for task creation.

---

## 5. PR Review

**When to use**: Code review, design review, or evaluating a teammate's proposed changes.

**Chain**: gather → assess → filter

### Example Session

```
User: /gather changes in PR #47
  → 8 file changes: new auth middleware, route updates, test additions, config tweaks
User: /assess by severity
  → CRITICAL: middleware lacks rate limiting; WARNING: missing error handling in 2 routes; OK: tests cover happy path
User: /filter keep critical and warning
  → 3 items: rate limiting gap, error handling in /api/login, error handling in /api/signup
```

### Why This Chain

Gather inventories the changes, assess categorizes by risk, filter surfaces what blocks merge. No verify step because you're evaluating proposed code, not checking existing claims.

---

## Composition Tips

1. **Primitives read upward in context.** When you invoke `/distill` after `/gather`, distill automatically detects gather's pipe-format output and uses it as input.

2. **You can skip steps.** If gather already gives you actionable items, you don't need to distill. If distill leaves only 2 options, diff-ideas can read those directly.

3. **Primitives compose with themselves.** Running `/gather` twice (first for broad scan, second for deep-dive) is valid — the second gather refines the first's findings.

4. **Chains are guidelines, not rules.** The recipes above are starting points. Adapt based on what the prior primitive produced.

5. **Check the summary.** Each primitive's summary paragraph tells you if you're done or if another primitive would help. Example: "These 4 approaches are equally viable — recommend diff-ideas comparison" signals the next step.
