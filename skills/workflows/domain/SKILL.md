---
name: domain
description: "Capture or query project-specific terminology, disambiguation rules, and domain concepts that persist across sessions. Use when a term is ambiguous, project-specific, or needs a canonical definition. Keywords: terminology, glossary, definition, disambiguation, domain, terms, vocabulary."
argument-hint: "<term definition> | query: <term>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Write, Edit
---

# Domain: Terminology Capture and Query

You are running the **domain** skill — capturing or querying project-specific terminology. Input: **$ARGUMENTS**

## When to Use

- When a term has a project-specific meaning that differs from its common use
- When the same word means different things in different contexts (disambiguation needed)
- When a concept recurs across sessions and needs a stable, shared definition
- When the user types `/domain query: <term>` to look up an existing definition
- When another skill or conversation references an ambiguous term that should be canonicalized

## Mode Detection

Parse `$ARGUMENTS` to determine mode:

- If `$ARGUMENTS` starts with `query:` or `query ` — **Query mode**
- Otherwise — **Capture mode** (default)

---

## Phase 1: Determine Mode and Parse Input

### Capture Mode

Extract from `$ARGUMENTS`:
- **Term**: The word or phrase being defined (everything before "means", "is", "refers to", or similar connective)
- **Definition**: The canonical meaning for this project
- **Disambiguation**: Any clarification about when this meaning applies vs. alternate meanings (optional — infer from input if present)

If the input is ambiguous or too vague to extract a clear term and definition, ask one clarifying question before proceeding.

### Query Mode

Extract the term to look up from `$ARGUMENTS` (strip the `query:` prefix and trim whitespace).

---

## Phase 2: Read domain.md

Read `memory/project/domain.md` if it exists. If the file does not exist:
- In **Capture mode**: proceed to Phase 3 (you will create it)
- In **Query mode**: report that no domain terminology has been captured yet and exit

---

## Phase 3: Execute

### Capture Mode

Check if the term already exists in domain.md (case-insensitive match on the `## <Term>` heading).

**If term exists**: Update the existing entry using Edit — replace the Definition, Disambiguation (if provided), and update the Added date to today's date only if the definition changed. Do not add a duplicate entry.

**If term is new**: Append a new entry to domain.md using Edit (or Write if the file does not exist yet). Use this format exactly:

```
## <Term>
**Definition**: <one sentence>
**Disambiguation**: <when this term is ambiguous, clarify which meaning is canonical here>
**Examples**: <optional — omit this line entirely if no examples provided>
**Added**: YYYY-MM-DD
```

Omit the `**Examples**` line unless the user provided examples.

Use today's date (2026-02-18) for `**Added**`.

If creating the file for the first time, write a header first:

```markdown
# Project Domain Terminology

Terms and disambiguation rules for this project. Maintained by `/domain`.

---

```

Then append the entry below the separator.

### Query Mode

Search domain.md for the term (case-insensitive match on `## <Term>` headings).

**If found**: Display the full entry verbatim.

**If not found**: Report that the term was not found and list the terms that do exist (from `## ` headings in the file) so the user can see what's available.

---

## Guidelines

- Keep definitions to one sentence. Disambiguation can be a second sentence or short clause.
- Do not infer or fabricate definitions — only write what the user provides.
- Preserve all existing entries when appending or editing.
- The canonical file path is `memory/project/domain.md`. Do not write to any other location.
- After a successful capture, confirm with a one-line summary: "Captured: **<Term>** — <definition>."
- After a successful query, display the entry without extra commentary.
