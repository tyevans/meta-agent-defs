# Goal
Verify that the git-intel Rust CLI (tools/git-intel/) is properly integrated across all skills, agents, and documentation. Identify gaps where git-intel should be referenced but isn't, incorrect references, or stale documentation.

## Dimensions
1. **Skills integration**: Which skills reference git-intel? Which should but don't?
2. **Agent integration**: Do any agents need git-intel awareness? Are tool paths correct?
3. **Documentation accuracy**: Do CLAUDE.md, README, docs/INDEX.md, and docs/reference.md accurately describe git-intel's capabilities and usage?
4. **Hook/config integration**: Are hooks and settings properly wired for git-intel?

## Boundaries
- Not auditing git-intel's internal Rust code quality (already shipped with 31 tests)
- Not redesigning git-intel's architecture (daemon design doc exists, decision gate set)
