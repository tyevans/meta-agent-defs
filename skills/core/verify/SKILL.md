---
name: verify
description: "Use when prior findings contain claims that need fact-checking against code, docs, or reality. Marks each VERIFIED, REFUTED, or UNCERTAIN with cited evidence. Keywords: fact-check, validate, confirm, test, assert, prove, check."
argument-hint: "[claims to verify | 'verify' to check prior findings]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git log:*), Bash(git show:*), WebSearch, WebFetch, Task
---

# Verify: Claim and Assertion Checker

You are running the **verify** primitive — checking claims and assertions against source code, documentation, or reality. Claims: **$ARGUMENTS**

## When to Use

- After gathering findings to confirm which claims are accurate
- When evaluating assertions made in documentation, tickets, or discussions
- When composing with another primitive (verify reads findings from context)
- When the user asks you to "check", "validate", "confirm", or "fact-check" something

## Process

### Phase 0: Claim Count Assessment

Identify and count all claims to verify. Parse $ARGUMENTS or detect upstream pipe-format output in context. State: "N claims identified."

- **8+ claims → fan-out mode** (dispatch up to 4 background agents)
- **Fewer than 8 claims → serial mode (default)** (verify sequentially in-context)

---

### Serial Mode (default, fewer than 8 claims)

1. **Identify claims**: Parse $ARGUMENTS or upstream pipe-format output
2. **Gather evidence**: Use Grep/Read to check code, git log/show for history, WebSearch/WebFetch for external claims
3. **Assess each claim**: Mark as **VERIFIED** (evidence confirms), **REFUTED** (evidence contradicts), or **UNCERTAIN** (insufficient/conflicting evidence)
4. **Emit structured output** in pipe format with verification status prominent

---

### Fan-Out Mode (8+ claims)

Dispatch per the fan-out-protocol rule (loaded alongside this skill). Verify-specific details:

**Partitioning:** Divide claims into up to 4 balanced subsets. Group thematically related claims in the same agent to reduce redundant file reads. Sizing: 8–11 claims → 2 agents, 12–19 → 3, 20+ → 4.

**Agent prompt template** (include verbatim in each dispatched agent):

```
You are verifying a subset of claims against source code, documentation, or reality.

Claims to verify:
<numbered list of the claims assigned to this agent>

For each claim:
1. Search for evidence using Grep, Read, Glob, Bash(git log:*), Bash(git show:*), WebSearch, or WebFetch.
2. Mark the claim as VERIFIED (evidence confirms), REFUTED (evidence contradicts), or UNCERTAIN (insufficient/conflicting evidence).
3. Cite specific evidence: file:line, commit hash, or URL. Do not quote large blocks — cite precisely.

Code and git history are more authoritative than docs or web. Prioritize codebase evidence.
If evidence is conflicting, mark UNCERTAIN and explain the conflict briefly.

Return your results as a numbered list matching the claim numbers above. For each claim, include:
- Verdict: VERIFIED | REFUTED | UNCERTAIN
- Evidence: one-line citation (file:line, commit hash, or URL)
- Detail: one sentence explaining why
```

**Merge:** Combine all agent verdicts into a single ordered list. Resolve conflicts per Fan-Out Guidelines below.

## Output Format

Output in pipe format. Each item includes verification status (VERIFIED/REFUTED/UNCERTAIN), evidence, and source. In fan-out mode, add `<!-- fan-out: N agents, M conflicts resolved -->` after Pipeline.

Each verified or refuted claim must cite specific evidence (file:line, commit hash, URL, or doc reference). Refuted claims are as valuable as verified ones — highlight what's wrong and why.

## Decomposition

Verify is a convenience macro that bundles two primitive operations:

```
/gather evidence for claims -> /assess VERIFIED|REFUTED|UNCERTAIN
```

The tight coupling between "what are the claims" and "search for their specific evidence" makes this bundling practical — a standalone `/gather` would need the claims as context to know what evidence to search for, and `/assess` would need the evidence alongside the claims. The macro eliminates this coordination overhead.

If you need finer control (e.g., custom evidence sources or a different verdict rubric), decompose manually into `/gather` + `/assess`.

## Guidelines

- Code and git history are more authoritative than docs or web — prioritize codebase evidence
- Refuted claims should clearly state what was wrong and what the actual situation is
- If evidence is conflicting, mark UNCERTAIN and explain the conflict in the detail
- Keep verification evidence concise — cite specific lines or commit hashes rather than quoting large blocks

### Fan-Out Guidelines

- **Conflict resolution**: When two agents return different verdicts for the same claim, the stronger-evidence verdict wins. As a tiebreaker: REFUTED beats VERIFIED (false positives are more costly than missed confirmations) and both beat UNCERTAIN.
- **Claim partitioning**: Assign thematically related claims to the same agent to minimize redundant tool calls across agents.
- **Concurrency cap**: Launch at most 4 agents. For 8–11 claims, use 2 agents. For 12–19 claims, use 3 agents. For 20+ claims, use 4 agents.
- **Conflict annotation**: In the merged output, annotate any claim where agents disagreed with `[conflict resolved: <original verdicts>]` inline after the verdict.
