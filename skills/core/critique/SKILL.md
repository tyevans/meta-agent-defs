---
name: critique
description: "Use when you want to stress-test a proposal, plan, or findings before acting on them. Finds flaws, gaps, and risks as devil's advocate. Keywords: critique, review, adversarial, premortem, devil's advocate, what's wrong, what's missing, risks, flaws, gaps, problems."
argument-hint: "[focus area: security | scalability | clarity | 'critique' to review all]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Task
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

1. **Find Input**: Detect upstream pipe-format output in context. If found, critique those items. Otherwise critique $ARGUMENTS directly.

2. **Apply Focus**: If $ARGUMENTS specifies a focus area (e.g., "security", "scalability"), prioritize criticisms in that dimension. If empty, critique everything.

3. **Identify Problems**: For each item or claim, ask:
   - **FLAW**: Is something incorrect, inconsistent, or contradictory?
   - **GAP**: Is something important missing or unaddressed?
   - **RISK**: Could something fail under certain conditions?

4. **Emit Structured Output** in pipe format. Each item tagged with verdict (FLAW/GAP/RISK). Include a `### Severity` section (table: criticism to impact HIGH/MEDIUM/LOW) between Items and Summary.

## Guidelines

- FLAW means something is objectively wrong — cite evidence
- GAP means something necessary is absent — explain why it matters
- RISK means something could fail — describe the failure scenario
- If input is sound, say so — "no critical issues found" is a valid critique
- One criticism per numbered item — don't bundle multiple issues

## Panel Mode (opt-in)

Panel mode dispatches 2–3 background agents with distinct adversarial lenses for high-stakes input. It is **not automatic** — trigger it only when $ARGUMENTS contains language like "deep critique", "thorough review", "stress test this", or an explicit request for a panel.

**Default (inline):** Run the Process steps above in a single pass.

**In panel mode:** Replace step 3 with a fan-out/fan-in sequence per the fan-out-protocol rule (loaded alongside this skill). Critique-specific details:

### Panel Lenses

Pick 2–3 based on subject matter:

| Lens | Focus |
|---|---|
| Feasibility | Can this actually be built/executed given constraints? |
| Failure Modes | What sequences of events cause this to break or harm? |
| Scope Gaps | What requirements, edge cases, or stakeholders are unaddressed? |

### Agent Prompt Template

```
You are an adversarial reviewer applying the [LENS] lens.

Subject: [paste or summarize the input being critiqued]

Your task:
1. Apply your lens exclusively — do not cover what other lenses would find
2. Produce a numbered list of findings, each labeled FLAW / GAP / RISK
3. For each finding: state the verdict, explain the problem concisely, cite evidence where possible
4. Rate each finding HIGH / MEDIUM / LOW severity
5. End with a one-sentence summary

Scope: [LENS description from table above]
Limit: 5–8 findings maximum. Be specific, not exhaustive.
```

### Panel Merge

1. **Deduplicate**: substantively identical findings across lenses → keep the most specific wording
2. **Resolve conflicts**: if agents disagree on severity, apply total order: `Feasibility > Failure Modes > Scope Gaps`
3. **Label provenance**: annotate each finding with its source lens in parentheses
4. **Emit** in pipe format with `**Pipeline**` noting `(panel: N agents)`
