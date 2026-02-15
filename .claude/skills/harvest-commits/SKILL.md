---
name: harvest-commits
description: "Harvest commit messages from git repos and label them via batched Haiku calls to build a training dataset for commit classification. Keywords: dataset, label, training, commits, ML, harvest."
argument-hint: "<repo-path-or-url> [--labels feat,fix,refactor,chore,docs,test,perf,style,ci,build,other] [--limit 5000] [--output data/commits-labeled.jsonl]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Write, Grep, Glob, Bash(git:*), Bash(wc:*), Bash(mkdir:*), Bash(head:*), Bash(tail:*), Bash(jq:*), Task
---

# Harvest Commits: Build Labeled Training Data from Git History

You are running the **harvest-commits** skill — extracting commit messages from one or more git repositories, then labeling each message via batched Haiku calls to produce a JSONL training dataset for commit type classification.

Target: **$ARGUMENTS**

## When to Use

- Building a labeled dataset for training a commit classifier (fastText, logistic regression, etc.)
- Harvesting commit messages from open-source repos that don't use conventional commits
- Expanding an existing labeled dataset with new repos
- Quality-checking existing labels by re-labeling a sample

## Workflow

```
[Parse args] -> [Extract commits] -> [Batch into groups of 50]
     -> [Label each batch via Haiku] -> [Write JSONL output]
     -> [Report stats]
```

## Phase 0: Parse Arguments

Parse `$ARGUMENTS` for:
- **repo**: Path to a local repo, or a GitHub URL to clone. Multiple repos separated by spaces or commas
- **--labels**: Comma-separated label set (default: `feat,fix,refactor,chore,docs,test,perf,style,ci,build,other`)
- **--limit**: Max commits per repo (default: 5000)
- **--output**: Output file path (default: `data/commits-labeled.jsonl`)
- **--append**: If set, append to existing output file instead of overwriting

If a GitHub URL is provided, clone it to a temp directory first:
```bash
git clone --bare <url> /tmp/harvest-<repo-name>
```

## Phase 1: Extract Commit Messages

For each repo, extract commit messages:
```bash
git -C <repo-path> log --format='%H|||%an|||%ae|||%aI|||%s' --no-merges -n <limit>
```

Fields: hash, author name, author email, ISO date, subject line.

Count total messages. If total exceeds 500, inform the user and proceed. The batching in Phase 2 handles scale.

## Phase 2: Batch and Label via Haiku

Split messages into batches of **50 commits each**.

For each batch, dispatch a Haiku agent via Task:

```
Task({
  subagent_type: "general-purpose",
  model: "haiku",
  prompt: <see labeling prompt below>
})
```

### Labeling Prompt Template

Send each batch with this prompt to Haiku:

```
You are labeling git commit messages for a training dataset.

For each commit message below, classify it into exactly ONE of these categories:
{label_list}

Category definitions:
- feat: Adds new functionality, features, or capabilities
- fix: Corrects a bug, error, or broken behavior
- refactor: Restructures code without changing behavior
- chore: Maintenance, dependencies, config, build system
- docs: Documentation changes only
- test: Test additions or modifications only
- perf: Performance improvements
- style: Formatting, whitespace, naming (no logic change)
- ci: CI/CD pipeline changes
- build: Build system or external dependency changes
- other: Doesn't fit any category above

Respond with a JSON array. Each element: {"hash": "<hash>", "message": "<original message>", "label": "<category>", "confidence": <0.0-1.0>}

Only output the JSON array, nothing else.

Commits:
{numbered_list_of_commits}
```

### Batch Processing Rules

1. Process batches sequentially (serialize to avoid API throttling)
2. After every 10 batches (500 commits), write intermediate results to the output file
3. If a batch fails, log the error and retry once. If it fails again, skip and note in final report
4. Parse the JSON response from each batch and validate: every input hash must appear in output

## Phase 3: Write Output

Write results as JSONL to the output file (one JSON object per line):

```jsonl
{"hash":"abc123","message":"Fixed movie search not working","label":"fix","confidence":0.95,"repo":"Radarr","author":"user@example.com","date":"2024-03-15T10:30:00Z"}
{"hash":"def456","message":"Added series monitoring toggle","label":"feat","confidence":0.92,"repo":"Sonarr","author":"dev@example.com","date":"2024-03-14T08:15:00Z"}
```

Each line includes: hash, original message, label, confidence, repo name, author email, date.

## Phase 4: Report Stats

After all batches complete, output a summary:

```
## Harvest Complete

**Repos**: <list>
**Total commits**: N
**Labeled**: N (N skipped/failed)
**Output**: <file path>

### Label Distribution
| Label    | Count | %     |
|----------|-------|-------|
| feat     | 1234  | 24.7% |
| fix      | 987   | 19.7% |
| ...      |       |       |

### Confidence Distribution
- High (>0.9): N (N%)
- Medium (0.7-0.9): N (N%)
- Low (<0.7): N (N%)

### Low-Confidence Samples (review these)
- "some ambiguous commit message" -> labeled as `chore` (0.45)
- ...
```

## Guidelines

- **Batch size 50** is the sweet spot — small enough for Haiku to handle reliably, large enough to amortize overhead
- **Sequential batches** to respect API rate limits. Do NOT parallelize Haiku calls
- **Intermediate writes** every 500 commits protect against context loss
- **Confidence scores** from Haiku are self-reported — treat <0.7 as needing human review
- **Deduplication**: If --append is used and the output file exists, skip hashes already in the file
- **No merge commits**: The `--no-merges` flag filters these out — they're noise for classification
- The `other` label is a catch-all. If >30% of commits land there, the label set may need expansion for that repo's domain
