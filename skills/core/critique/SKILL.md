---
name: critique
description: "Adversarial review of any pipe-format output or claims. Finds what's wrong, what's missing, and what could fail. Premortem/devil's advocate pattern. Keywords: critique, review, adversarial, premortem, devil's advocate, what's wrong, what's missing, risks, flaws, gaps, problems."
argument-hint: "[focus area: security | scalability | clarity | 'critique' to review all]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob
context: inline
---

# Critique: Adversarial Review

You are running the **critique** primitive — adversarial review of outputs, claims, or proposals to find flaws, gaps, and risks. Focus: **$ARGUMENTS**

## When to Use

- After gather, distill, or rank to challenge the findings
- Before implementing a design or plan (premortem)
- When the user asks "what's wrong?", "what could fail?", or "devil's advocate"
- To stress-test decisions or proposals

## Process

1. **Find Input**: Search conversation context for prior primitive output (the `## ... / **Source**: /...` pattern). If found, critique those items and read the `**Pipeline**` field to construct provenance. Otherwise critique $ARGUMENTS directly.

2. **Apply Focus**: If $ARGUMENTS specifies a focus area (e.g., "security", "scalability"), prioritize criticisms in that dimension. If empty, critique everything.

3. **Identify Problems**: For each item or claim, ask:
   - **FLAW**: Is something incorrect, inconsistent, or contradictory?
   - **GAP**: Is something important missing or unaddressed?
   - **RISK**: Could something fail under certain conditions?

4. **Emit Structured Output** in pipe format:
   - **Header**: `## [Critique of ...]`
   - **Metadata**: `**Source**: /critique`, `**Input**: [one-line summary]`, `**Pipeline**: [upstream chain -> /critique (N items)]` or `(none — working from direct input)`
   - **Items**: Numbered list, each with verdict (FLAW/GAP/RISK), detail, and optional source
   - **Severity**: Table mapping each criticism to impact (HIGH/MEDIUM/LOW)
   - **Summary**: One paragraph synthesis

## Guidelines

- FLAW means something is objectively wrong — cite evidence
- GAP means something necessary is absent — explain why it matters
- RISK means something could fail — describe the failure scenario
- If input is sound, say so — "no critical issues found" is a valid critique
- Preserve source attribution from input if composing with a prior primitive
- One criticism per numbered item — don't bundle multiple issues
