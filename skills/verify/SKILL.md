---
name: verify
description: "Check claims and assertions against source code, documentation, or reality. Marks each claim as VERIFIED, REFUTED, or UNCERTAIN with evidence. Keywords: fact-check, validate, confirm, test, assert, prove, check."
argument-hint: "[claims to verify | 'verify' to check prior findings]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git log:*), Bash(git show:*), WebSearch, WebFetch
---

# Verify: Claim and Assertion Checker

You are running the **verify** primitive — checking claims and assertions against source code, documentation, or reality. Claims: **$ARGUMENTS**

## When to Use

- After gathering findings to confirm which claims are accurate
- When evaluating assertions made in documentation, tickets, or discussions
- When composing with another primitive (verify reads findings from context)
- When the user asks you to "check", "validate", "confirm", or "fact-check" something

## Process

1. **Identify claims**: Parse $ARGUMENTS or read findings from prior primitive output in context (detected via pipe format: `## ... / **Source**: /...`)
2. **Gather evidence**: Use Grep/Read to check code, git log/show for history, WebSearch/WebFetch for external claims
3. **Assess each claim**: Mark as **VERIFIED** (evidence confirms), **REFUTED** (evidence contradicts), or **UNCERTAIN** (insufficient/conflicting evidence)
4. **Emit structured output** in pipe format with verification status prominent

## Output Format

- **Header**: `## [Verification of ...]`
- **Metadata**: `**Source**: /verify`, `**Input**: [one-line claims summary]`
- **Items**: Numbered list with title, verification status (VERIFIED/REFUTED/UNCERTAIN), evidence, source (file:line or URL), and confidence (CONFIRMED for verified, POSSIBLE for uncertain)
- **Summary**: One paragraph synthesis of verification results

Each verified or refuted claim must cite specific evidence (file:line, commit hash, URL, or doc reference). Refuted claims are as valuable as verified ones — highlight what's wrong and why.

## Guidelines

- Code and git history are more authoritative than docs or web — prioritize codebase evidence
- Refuted claims should clearly state what was wrong and what the actual situation is
- If evidence is conflicting, mark UNCERTAIN and explain the conflict in the detail
- If composing with a prior primitive, verify all claims from that output
- Keep verification evidence concise — cite specific lines or commit hashes rather than quoting large blocks
