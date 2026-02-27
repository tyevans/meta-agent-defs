# EPIC: Add tacks as alternative task manager backend

**Source**: /blossom
**Created**: 2026-02-26
**Goal**: Install tackline with option to use tacks (tk) instead of beads (bd)

## Design Decision

**Auto-detect at runtime**, not install-time templating. Hooks check for `.tacks/` vs `.beads/` and call `tk` or `bd` accordingly. Skills rely on a CLAUDE.md command mapping table — the model reads it and applies substitutions when following skill instructions.

## Tasks

### P0 — Shell execution layer

- [x] **Update install.sh: auto-detect backend** — Check for `tk` OR `bd` (either satisfies the recommendation). Add `--tacks` flag that makes install prefer/require tacks. Store backend choice in manifest header.
  - Scope: install.sh lines 245-248 (dependency check), argument parsing, manifest
  - Depends on: nothing

- [x] **Update hooks/hooks.json: auto-detect backend** — Replace all 6 `bd` call sites with `if [ -d .tacks ]; then tk ...; elif [ -d .beads ]; then bd ...; fi` pattern. Update `.beads/` directory detection to check both dirs. Update warning messages.
  - Scope: hooks/hooks.json (SessionStart, PreCompact, 2x PreToolUse, PostToolUse, SessionEnd)
  - Depends on: nothing

- [x] **Update .claude/settings.json hooks: auto-detect backend** — Same pattern for project-local hooks (SessionStart, PreCompact).
  - Scope: .claude/settings.json lines 23, 34
  - Depends on: nothing

### P1 — Model interpretation layer

- [x] **Add backlog tool abstraction to CLAUDE.md** — Add a "Backlog Tool" section with: (1) auto-detect convention description, (2) `bd` ↔ `tk` command mapping table, (3) key differences (tags vs types, priority scale). This is the foundation that lets skills work with either backend without individual rewrites.
  - Scope: CLAUDE.md (~20 lines added)
  - Depends on: nothing

- [x] **Update skills: directory checks** — Mechanical update of `.beads/` directory guards to `.beads/ OR .tacks/` in 23 heavy-user skills. Keep `bd` command examples (model substitutes via CLAUDE.md mapping).
  - Scope: 23 SKILL.md files (blossom, sprint, consolidate, standup, team-meeting, assemble, retro, curate, tend, spec, consensus, deploy, premortem, review, handoff, bug, fractal, advise, session-health, optimize, bootstrap, tracer, status)
  - Depends on: CLAUDE.md backlog abstraction (task above)

### P2 — Supporting materials

- [x] **Update agent tool lists** — Add `Bash(tk:*)` alongside `Bash(bd:*)` in 8 agent definitions.
  - Scope: agents/*.md, .claude/agents/*.md
  - Depends on: CLAUDE.md backlog abstraction

- [x] **Update docs** — Dual-tool references in docs/pipelines.md, docs/reference.md, docs/team-system-guide.md, docs/INDEX.md, docs/primitives-cookbook.md, docs/primitives-recipes.md, docs/demos/*.md
  - Scope: 8+ doc files
  - Depends on: CLAUDE.md backlog abstraction

- [x] **Update README.md** — Describe tacks as alternative, update dependency section, add tk to command examples.
  - Scope: README.md
  - Depends on: nothing

### P3 — Polish

- [x] **Update rules** — Minor text updates in rules/team-protocol.md (.beads/ → .beads/.tacks/), .claude/rules/hook-authoring.md (mention tk), .claude/rules/skill-authoring.md (Bash(tk:*) example), .claude/rules/commits.md (beads → backlog state)
  - Scope: 4 rule files
  - Depends on: nothing

- [x] **Update templates** — Minor text in templates/teams/README.md
  - Scope: 1 file
  - Depends on: nothing

## Command Mapping Reference

| bd command | tk equivalent | Notes |
|---|---|---|
| `bd init` | `tk init` | Direct equivalent |
| `bd create --title="..." --type=task` | `tk create "..." -t task` | tk uses tags, not types |
| `bd create --type=epic` | `tk create "..."` + create subtask | Epic tag auto-added on child creation |
| `bd create --parent=<id>` | `tk create --parent <id>` | Direct equivalent |
| `bd list` | `tk list` | Direct equivalent |
| `bd list --status=X` | `tk list -s X` | Direct equivalent |
| `bd ready` | `tk ready` | Direct equivalent |
| `bd show <id>` | `tk show <id>` | Direct equivalent |
| `bd update <id> --status=X` | `tk update <id> -s X` | Direct equivalent |
| `bd update <id> --notes="..."` | `tk update <id> --notes "..."` | Direct equivalent |
| `bd close <id>` | `tk close <id>` | Direct equivalent |
| `bd close <id> --reason=X` | `tk close <id> -r X` | Direct equivalent |
| `bd dep add <child> <parent>` | `tk dep add <child> <parent>` | Direct equivalent |
| `bd children <id>` | `tk children <id>` | Direct equivalent |
| `bd epic status` | `tk epic` | Direct equivalent |
| `bd stats` | `tk stats` | Direct equivalent |
| `bd blocked` | `tk blocked` | Direct equivalent |
| `bd prime` | `tk prime` | Direct equivalent |
| `bd sync` | _(not needed)_ | tk is local-only, no sync |
| `bd query "..."` | _(no equivalent)_ | Use `tk list --json \| jq` |
| `bd stale` | _(no equivalent)_ | Manual aging check |
| `bd swarm validate` | _(no equivalent)_ | Manual DAG inspection |
| `bd doctor` | _(no equivalent)_ | Not needed (simpler schema) |
| `bd dep cycles` | _(no equivalent)_ | Cycles caught at write-time only |

## Priority Scale
- bd: 0-4 (0=critical, 4=backlog)
- tk: 0-4 (0=critical, 4=backlog) — same scale, needs confirmation
