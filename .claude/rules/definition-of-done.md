# Definition of Done

What "done" means for each type of change in this repo.

## New Agent (`agents/*.md`)

- [ ] YAML frontmatter includes all required fields: `name`, `description`, `tools`, `model`
- [ ] Description says WHEN to use the agent, not just what it does
- [ ] Includes Investigation Protocol, Context Management, and Knowledge Transfer sections
- [ ] Model selection matches task complexity (opus for high, sonnet for medium, haiku for low)
- [ ] Tools list is minimal -- read-only agents should not have Write/Edit
- [ ] `output-contract` included if output is parsed by another skill or agent (optional otherwise)
- [ ] Agent catalog (AGENTS.md in target project) updated if applicable
- [ ] `install.sh` re-run to verify symlink creation

## New Skill (`skills/<name>/SKILL.md`)

- [ ] YAML frontmatter includes required fields: `name`, `description`, `allowed-tools`, `context`
- [ ] Uses `$ARGUMENTS` for user input where applicable
- [ ] Has clear phase structure with numbered steps
- [ ] Includes Phase 0 or equivalent precondition gate if the skill has prerequisites (e.g., loaded context, clean git state, prior skill output)
- [ ] Tested by running the slash command in a live session
- [ ] `install.sh` re-run to verify symlink creation
- [ ] `docs/INDEX.md` updated with new skill entry

## Settings Change (`settings.json`)

- [ ] Hooks fail gracefully with `|| true` for optional tools
- [ ] No duplicate hook entries for the same matcher
- [ ] JSON is valid (no trailing commas, proper quoting)
- [ ] Re-run `install.sh` to refresh symlink

## Bug Fix

- [ ] Root cause identified and documented in commit message
- [ ] Fix verified by testing the affected workflow end-to-end

## Documentation Update

- [ ] CLAUDE.md stays under 200 lines
- [ ] README.md reflects current file structure
- [ ] No stale references to removed files or renamed fields
- [ ] `docs/INDEX.md` reflects current skill/agent inventory
