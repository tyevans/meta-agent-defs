---
name: domain
description: "Use when a term is ambiguous or project-specific and needs a canonical definition. Also runs terminology audits to find stale or orphaned glossary entries. Keywords: terminology, glossary, definition, disambiguation, domain, terms, vocabulary, audit, stale."
argument-hint: "<term> = <definition> | query: <term> | audit"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Write, Edit, Grep, Glob
---

# Domain: Terminology Capture, Query, and Audit

You are running the **domain** skill — managing project-specific terminology in `.claude/tackline/memory/project/domain.md`. Input: **$ARGUMENTS**

## When to Use

- A term has a project-specific meaning that differs from its common use
- The same word means different things in different contexts and you need a canonical reference
- A concept recurs across sessions and needs a stable, shared definition
- You want to look up how this project uses a specific term (`query:` prefix)
- You suspect the glossary has stale entries that no longer match the codebase (`audit`)
- Another skill or agent referenced an ambiguous term — canonicalize it before proceeding

## Don't Use When

- The term is universally understood with no ambiguity in this project
- You want to store general project notes — use `.claude/tackline/memory/project/` directly instead
- The definition is speculative — only capture what is confirmed and stable

## Overview

```
$ARGUMENTS
    |
    v
Phase 0: Mode Detection
    |
    +---> capture --> Phase 1C: Resolve entry --> Phase 2C: Write + verify
    |
    +---> query   --> Phase 1Q: Look up term  --> emit result
    |
    +---> audit   --> Phase 1A: Scan + cross-check --> Phase 2A: Remove stale --> pipe-format output
```

---

## Phase 0: Mode Detection (Gate)

Parse `$ARGUMENTS` and determine mode. **Stop here if mode cannot be determined.**

| Signal in $ARGUMENTS | Mode |
|---|---|
| Starts with `query:` or `query ` | **query** |
| Is exactly `audit` or starts with `audit ` | **audit** |
| Contains `=`, `means`, `is`, or `refers to` | **capture** |
| Any other non-empty input | **capture** (default — treat as term definition) |
| Empty | Ask: "Did you want to capture a term, query an existing one, or run an audit?" then stop |

State the detected mode before proceeding: "Mode: **capture** / **query** / **audit**"

---

## Phase 1C: Resolve Entry (Capture Mode)

### Step 1 — Parse the input

Extract:
- **Term**: The word or phrase being defined
- **Definition**: The canonical meaning for this project
- **Disambiguation** (optional): When this meaning applies vs. alternate meanings

If the input is too vague to extract a clear term and definition, ask one clarifying question before continuing.

### Step 2 — Read domain.md

Read `.claude/tackline/memory/project/domain.md` if it exists.

- If the file does not exist, proceed to Phase 2C (you will create it).
- If the file exists, scan for an existing `## <Term>` heading (case-insensitive).

### Step 3 — Check for existing entry (Gate)

**If the term already exists:**
- Display the current entry.
- State: "Entry exists. Updating definition." then proceed to Phase 2C with update intent.

**If the term is new:**
- Proceed to Phase 2C with create intent.

### Step 4 — Gather codebase examples (optional but preferred)

Use Grep to search the codebase for the term (case-insensitive). Collect up to 3 file:line references where the term appears in a meaningful context (not just comments or string literals).

If examples are found, include them under `**Examples**` in the entry. If none found, omit the `**Examples**` line.

---

## Phase 2C: Write and Verify (Capture Mode)

### Write the entry

**Entry format:**

```
## <Term>
**Definition**: <one sentence>
**Disambiguation**: <when ambiguous, clarify which meaning is canonical here>
**Examples**: <file:line, file:line> (omit this line if no examples)
**Added**: YYYY-MM-DD
```

**If creating the file for the first time**, write the header first:

```markdown
# Project Domain Terminology

Terms and disambiguation rules for this project. Maintained by `/domain`.

---

```

Then append the entry below the separator.

**If the term exists**, use Edit to replace the existing entry block (from its `## <Term>` heading to the blank line before the next heading or end of file). Update `**Added**` only if the definition changed.

**If the term is new**, append using Edit (or Write if creating the file).

### Verify

After writing, read `.claude/tackline/memory/project/domain.md` and confirm the term appears under a `## <Term>` heading. State: "Verified: **<Term>** is present in domain.md."

If the read does not find the term, report the failure and do not claim success.

### Completion

Emit a one-line summary: "Captured: **<Term>** — <definition>."

---

## Phase 1Q: Look Up Term (Query Mode)

Strip the `query:` prefix and trim whitespace to get the lookup term.

Read `.claude/tackline/memory/project/domain.md`.

- If the file does not exist: "No domain terminology has been captured yet."
- Search for `## <Term>` heading (case-insensitive).
  - **Found**: Display the full entry verbatim. No extra commentary.
  - **Not found**: "Term not found: **<Term>**." Then list all `## ` headings from the file as available terms.

---

## Phase 1A: Scan for Stale Entries (Audit Mode)

**Goal**: Find entries in domain.md that are no longer referenced in the codebase — potential orphans or stale definitions.

### Step 1 — Read domain.md

If `.claude/tackline/memory/project/domain.md` does not exist: "No domain file found. Nothing to audit." Exit.

Extract all terms from `## ` headings. State: "N terms found in domain.md."

### Step 2 — Cross-check each term against codebase

For each term, run Grep (case-insensitive) across the codebase (excluding `.claude/tackline/memory/`). Record:
- **Active**: 1 or more matches found outside `.claude/tackline/memory/`
- **Stale**: 0 matches found (term does not appear in any source file)

### Step 3 — Emit audit findings in pipe format

```
## Domain Audit: Stale Entry Report

**Source**: /domain (audit)
**Input**: audit of .claude/tackline/memory/project/domain.md
**Pipeline**: (none — working from direct input)

### Items (N)

1. **<Term>** — stale: no codebase references found
   - source: .claude/tackline/memory/project/domain.md
   - confidence: LIKELY

2. **<Term>** — active: found in <file:line>
   - source: <file:line>
   - confidence: CONFIRMED

### Summary

<N-stale> of <N-total> terms appear to have no codebase references and may be stale or obsolete. <N-active> terms are actively used.
```

Use LIKELY (not CONFIRMED) for stale entries — absence of grep hits does not guarantee the term is truly unused (it may appear in non-text assets or external docs).

---

## Phase 2A: Remove Stale Entries (Audit Mode, only if stale entries found)

**Gate**: Only proceed if stale entries were found AND the user confirms removal.

After emitting the audit findings, ask: "Remove N stale entries? (yes / no / list which ones)"

**On confirmation:**

For each confirmed stale entry, use Edit to remove the `## <Term>` block (heading through the blank line before the next heading). Preserve all active entries exactly.

**Verify removal:**

After editing, read `.claude/tackline/memory/project/domain.md` and confirm none of the removed terms appear as `## ` headings. State: "Verified: removed terms are no longer present."

If any removed term still appears, report the failure explicitly.

---

## Guidelines

- Keep definitions to one sentence. Disambiguation can be a second sentence or short clause.
- Do not infer or fabricate definitions — only write what the user provides or what is confirmed in code.
- Preserve all existing entries when appending or editing.
- The canonical file path is `.claude/tackline/memory/project/domain.md`. Do not write to any other location.
- Use today's date for `**Added**` fields.
- Verification is not optional — always confirm writes succeeded by re-reading the file.
- In audit mode, LIKELY confidence is correct for stale entries — grep absence is strong evidence but not proof.
