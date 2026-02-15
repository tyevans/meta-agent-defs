# Retrospective History

## Retro: 2026-02-15 (session 21 — hardening + ML model sprint)
- Tasks completed: 4 dispatched + 2 housekeeping (nh5v closed as done, 7pkx closed as superseded)
- Workflow: /sprint with 2 parallel rounds (rust-dev x2, ml-eng x2)
- Commits: 2 (1 feat, 1 chore sync)
- New learnings: 6 across 2 members (rust-dev +2 updated, ml-eng +2 new sections)
- Pruned: rust-dev from 62→57 lines (cache + precision study consolidated)
- Key insight: Superseded bead detection caught 2 beads this sprint (nh5v already shipped, 7pkx=eobx duplicate) — saves wasted dispatch tokens. Parallel dispatch within same role (2 rust-dev tasks on different modules, 2 ml-eng tasks on independent models) continues to work cleanly. 4/4 first-attempt successes (streak: 79+). git-intel now at 175 tests with streaming iterator + relative dates. commit-labeler now has 4 model implementations.
- Backlog state: 290/304 closed, 14 open (11 ready, 3 blocked)

## Retro: 2026-02-15 (signal system sprint)
- Tasks completed: 4 (wwzo signal struct, qn62 fix-chaser detector, 66r6 git-pulse signals, rl34 precision study)
- Commits: 7 (2 feat, 1 fix, 4 chore). Fix rate: 14%
- Sprint: 4 tasks, 2 serial (dependency chain) + 2 parallel (independent). Clean — 0 rework
- Precision study: 67% TP rate on 5 real repos, detector is conservative (0 signals on ripgrep/tokio/serde)
- Cross-project fix: git-intel now accessible from any project via absolute fallback path
- Learnings persisted: 7 new (6 rust-dev, 1 infra). rust-dev pruned from 63→60 lines (ML/ONNX consolidated)
- Key insight: meeting→sprint pipeline effective for Rust feature work; precision study as sprint task gates further investment

## Retro: 2026-02-15 (overnight session)
- Tasks completed: 5 (hpnx git-pulse ML auto-detect + 4 pattern improvements: ay3h, tj9y, 0su5, rvmh)
- Epic closed: 7fjp (ONNX integration, 12/12 children)
- Fix rate: 0% (6 feat commits, 0 fixes — clean sprint)
- Meeting: 2-panelist (User Advocate + Architect) on pattern improvements. Produced 4 action items executed in same session
- Key insight: meeting→sprint pipeline works for Rust tooling decisions. Panel tension produced clear go/no-go on each pattern
- Pattern system v2: fix-after-feat with file overlap (3 pairs vs noise), multi-edit chains capped+filtered (10), temporal clusters (12 detected)
- Learnings persisted: 4 rust-dev entries

## Retro: 2026-02-13
- Tasks completed: 10 (8 implementation + 1 already-done + 1 epic)
- Commits: 14 (5 recovering prior session + 9 new)
- New learnings: 9 across 3 members (infra +3, skill-author +1 consolidated, agent-author stable)
- Pruned/archived: 2 stale skill-author gotchas consolidated into 1 RESOLVED entry
- Key insight: serial dispatch with sonnet subagents produced 6/6 first-try successes on install.sh tasks; claude-in-claude subprocess dispatch later abandoned in favor of Task tool
- Backlog state: 75/75 closed, zero remaining

## Retro: 2026-02-13 (session 2)
- Tasks completed: 0 new (commit + push of prior session's work)
- Commits: 1 (feat: generalize team system for adoption across projects)
- New learnings: 0 (all learnings from prior session already persisted)
- Key insight: Prior session left 16 files uncommitted across 3 logical groups (global rules, beads-optional skills, templates). Committing as one coherent unit with a descriptive message was the right call -- the changes were tightly coupled.
- Backlog state: 78/80 closed, 2 open (P3 backlog items)

## Retro: 2026-02-13 (session 3 — composable primitives)
- Tasks completed: 20 (pipe format + 11 primitives + 4 workflow refactors + recipe docs + cookbook + 2 demos + review refactor)
- Commits: 6 on feat/composable-primitives branch
- New learnings: 6 for skill-author (composition patterns)
- Key insight: /rank → /sprint pipeline is a natural workflow — rank provides sequencing rationale, sprint provides execution loop. Serial dispatch with orchestrator review between tasks produced 7/7 clean first-attempt completions.
- Backlog state: 101/106 closed, 4 open (3 P3 + 1 blocked epic)

## Retro: 2026-02-13 (session 4 — sprint cleanup)
- Tasks completed: 5 (team system guide + interactive assemble + /merge primitive + demo repo + P2 epic closed)
- Commits: 2 on feat/composable-primitives branch
- New learnings: 4 across 2 members (skill-author +3, agent-author +1)
- Pruned: 2 stale skill-author entries updated (skill count, disable-model-invocation), 1 duplicate section header merged
- Key insight: Standup → sprint → retro cycle ran end-to-end smoothly. All 4 dispatches were first-attempt successes. The backlog is now 106/107 closed with only 1 bead remaining (P2 audit task).
- Backlog state: 106/107 closed, 1 open (P2 audit task)

## Retro: 2026-02-14 (session 5 — observable progress)
- Tasks completed: 6 (3 P1 hook changes + pipeline provenance + path registry + /status skill)
- Meeting: 4-agent product meeting (User Advocate, Architect, Skeptic, Pragmatist) — 3 rounds
- New learnings: 7 across 3 members (infra +4, skill-author +3, agent-author +1 updated)
- Key insight: "Never spend tokens on awareness — spend formatting on it." Two-tier model: free shell pulse (automatic) + opt-in LLM skills. Product meetings with genuine role tension produced concrete, shippable outcomes faster than solo planning.
- Backlog state: 115/115 closed, 0 open

## Retro: 2026-02-14 (session 6 — pipe format validation)
- Tasks completed: 5 (epic + 3 spec amendments + empirical validation)
- Meetings: 2 (Architect+Skeptic on spec gaps, Pragmatist+Scientist on results)
- Test runs: 8 (5 gather, 1 full chain, 1 disambiguation, 1 failure injection) — all passed
- New learnings: 0 new team learnings (session was validation-focused, not implementation)
- Key insight: Rule 9 (validation heuristic) works as a safety net — model self-reports intake AND flags corruption without being asked. Subagent turn limits (~30 calls for 3-step chain) require running multi-step chains in main session.
- Backlog state: 124/124 closed, 0 open

## Retro: 2026-02-14 (session 7 — sprint + disambiguation test)
- Tasks completed: 3 (2 dispatched to skill-author + 1 main-session disambiguation test)
- Commits: 2 (docs: dispatch guidance + chore: close beads)
- New learnings: 0 team (documentation session, no novel patterns)
- Pruned: 1 stale agent-author gotcha removed, 1 infra duplicate header merged
- Key insight: `from:` override disambiguation works — topic-keyword matching in $ARGUMENTS correctly selects non-recent pipe-format blocks. Sprint skill's serialize-by-default strategy confirmed again: 2/2 dispatches were first-attempt successes.
- Backlog state: 127/127 closed, 0 open

## Retro: 2026-02-14 (session 8 — cross-pollinate meeting tunings)
- Tasks completed: 6 (5 tasks + 1 epic close attempt; epic deferred with 2 P4 children)
- Workflow: /blossom (7 findings) → /sprint (5 dispatched, 2 P4 deferred)
- Commits: 1 (feat: cross-pollinate meeting tunings to workflow skills)
- New learnings: 4 for skill-author (preamble, characterization, adaptive lenses, pushback)
- Key insight: Characterization-over-procedure pattern ("You think like...") condensed agent prompts 50-74% while giving agents richer mental models. Extracting shared boilerplate to a rule file (Agent Preamble) eliminated cross-skill drift.
- Backlog state: 137/140 closed, 3 open (2 P4 speculative + 1 epic)

## Retro: 2026-02-14 (session 9 — sprint execution)
- Tasks completed: 3 (git-pulse.sh + fractal checkpoint + consensus debate round)
- Workflow: /sprint with serial dispatch (infra x1, skill-author x2)
- Commits: 1 (feat: add git-pulse.sh, fractal checkpoint, and consensus debate round)
- New learnings: 6 across 2 members (infra +2, skill-author +4)
- Key insight: Clean 3/3 first-attempt sprint. All P4 speculative beads now closed, unblocking the cross-pollination epic (meta-agent-defs-jim). git-pulse.sh shipped as shared utility for skills to consume.
- Backlog state: 157 total, 15 open (4 ready, 11 blocked). git-intel Rust CLI (P1) is the big remaining blocker.

## Retro: 2026-02-14 (session 11 — git-intel test + hardening sprint)
- Tasks completed: 2 (test suite + hardening pass)
- Workflow: /sprint with serial dispatch (rust-dev x2)
- Commits: 0 (uncommitted — session end will commit)
- New learnings: 5 for rust-dev (lib.rs split, Oid fixture pattern, test coverage, strict classify, churn borrow)
- Key insight: Test-first then hardening is a natural serial pair — the test suite gave confidence for all 6 hardening refactors. lib.rs extraction was an unexpected but strictly positive architectural change forced by Rust's integration test requirements. 2/2 first-attempt dispatch successes continues the streak.
- Backlog state: 172/179 closed, 7 open (5 ready, 2 blocked)

## Retro: 2026-02-14 (session 12 — sprint + daemon design meeting)
- Tasks completed: 5 (docs/INDEX update + /status churn heatmap + SessionStart volatility + /drift convergence detection + daemon design doc)
- Workflow: /sprint (5 beads, 3 serial + 2 parallel) → /meeting (Pragmatist vs Architect on daemon design)
- New learnings: 0 team (mature team, no novel patterns)
- Meeting outcome: No build from daemon design. Doc serves as decision gate (1000+ commits / 200ms measured latency). 3-line lib.rs comment to preserve module seam.
- Key insight: "Premature abstraction vs premature optimization" — 50 lines of cache code is cheap to write but forces format decisions before production feedback. "Cheap enough" isn't sufficient justification; "someone is actually waiting" is the right trigger.
- Backlog state: 177/179 closed, 2 open (1 ready P4 task, 1 blocked epic)

## Retro: 2026-02-14 (session 10 — git-intel MVP sprint)
- Tasks completed: 10 (MVP + 2 bug fixes + Cargo.lock + common.rs refactor + 3 skill enrichments + 2 new skills)
- Workflow: /assemble (add rust-dev) → /sprint → /review → /meeting (Architect vs Pragmatist) → create 6 beads → /sprint (10 dispatches)
- New learnings: 12 across 2 members (rust-dev +8 new file, skill-author +4)
- Team member added: rust-dev (opus, owns tools/git-intel/**)
- Key insight: Review→meeting→sprint pipeline emerged as a powerful code quality workflow. User's instinct to /assemble before dispatching Rust work created the right specialist. 10/10 first-attempt dispatch successes. Highest single-session bead throughput.
- Backlog state: 178 total, 8 open (6 ready, 2 blocked). All P1 work complete.

## Retro: 2026-02-14 (session 14/15 — commit-labeler Phase 1 sprint)
- Tasks completed: 12 (full Phase 1 consolidation: move, delete, create, extract, consolidate, migrate, rework)
- Workflow: /blossom (HF research) → /meeting (merge two plans) → create 21 beads → /assemble (add ml-eng) → /sprint (7 rounds)
- New learnings: 16 for ml-eng (entirely new member, bootstrapped from zero)
- Team member added: ml-eng (sonnet, owns tools/commit-labeler/**)
- Key insight: Meeting→bead-creation→sprint pipeline works for ML projects too. New team member (ml-eng) bootstrapped from zero learnings to 16 entries in one session — the learning loop is the product. 12/12 first-attempt dispatch successes. Parallel rounds (3 and 5) on non-overlapping files halved wall-clock time for those rounds.
- Backlog state: 221/263 closed, 42 open (22 ready). Phase 1 commit-labeler complete, Phase 2 (ModernBERT) ready.

## Retro: 2026-02-15 (session 16 — dual-track sprint)
- Tasks completed: 6 (4 rust-dev git-intel classifier/patterns + 2 ml-eng commit-labeler refactoring)
- Workflow: /sprint with 3 rounds of parallel dispatch (rust-dev + ml-eng in rounds 1-2, rust-dev + rust-dev in round 3)
- New learnings: 9 across 2 members (rust-dev +5, ml-eng +4)
- Key insight: Parallel dispatch across ownership areas (Rust + Python) works cleanly with zero conflicts. Round 3 parallelized two rust-dev tasks on different files (common.rs vs patterns.rs) — also safe. 6/6 first-attempt successes (cumulative streak: 60+). Clearing blocker beads (pxzb, 7tqk) to unblock Phase 2 ModernBERT gate was higher strategic value than more git-intel features.
- Backlog state: 227/263 closed, 36 open (20 ready, 16 blocked). Phase 2 decision gate (lwww) now unblocked.

## Retro: 2026-02-15 (session 17 — dual-track sprint: git-intel + commit-labeler Phase 2)
- Tasks completed: 5 (4 dispatched + 1 closed as superseded)
- Workflow: /sprint with 2 rounds of parallel dispatch (rust-dev + ml-eng)
- Commits: 0 (uncommitted — session end will commit)
- New learnings: 11 across 2 members (rust-dev +4, ml-eng +7)
- Key insight: Superseded bead detection saved a wasted dispatch — 6a4j (train_transformer.py) was already covered by lwww (models/transformer.py) from a different planning session. ml-eng over-produced docs (5 extra files beyond benchmark.py); orchestrator cleanup needed. 4/4 first-attempt dispatch successes (streak: 64+). git-intel now at 75 tests with cache + hotspots. commit-labeler Phase 2 gate shipped (transformer model + embedding benchmark).
- Backlog state: 232/263 closed, 31 open (23 ready, 8 blocked)

## Retro: 2026-02-15 (session 18 — dual-track sprint: git-intel features + commit-labeler eval/loss)
- Tasks completed: 6 (4 rust-dev git-intel features + 2 ml-eng commit-labeler enhancements)
- Workflow: /sprint with 3 dispatch rounds (2 parallel cross-track + 1 serial rust-dev)
- Commits: 0 (uncommitted — session end will commit)
- New learnings: 14 across 2 members (rust-dev +6, ml-eng +8)
- Key insight: Already-implemented features (hotspots --depth) caught by dispatch — agent found and fixed a depth=0 bug instead of reimplementing. This is the learning loop paying off: prior sprint shipped the feature, this sprint's agent read the code and found the edge case. 6/6 first-attempt successes (streak: 70+). git-intel at 118 tests. embed_mlp rewritten from sklearn to PyTorch for custom loss support — a significant but necessary architectural change.
- Backlog state: 238/263 closed, 25 open (18 ready, 7 blocked)

## Retro: 2026-02-15 (session 19 — commit-labeler training fix)
- Tasks completed: 1 (data.py multi-label normalization fix)
- Commits: 0 (uncommitted)
- New learnings: 0 team (orchestrator-only fix)
- Key insight: Schema drift between pipeline stages — labeling outputs "labels" (list) but training expected "label" (string). Normalizing in the canonical I/O module (data.py:load_labeled) was the right fix location. First baseline: tfidf-logreg 0.59 macro F1 on 1900 samples.
- Backlog state: 238/263 closed, 25 open (18 ready, 7 blocked)

## Retro: 2026-02-15 (session 20 — git-intel authors/trends sprint)
- Tasks completed: 5 (hotspot types + benchmark + authors + mailmap + trends)
- Workflow: /sprint with 3 rounds (2 parallel + 2 parallel + 1 serial), plus 4 cleanup commits from prior sessions
- Commits: 8 (4 feat, 4 chore)
- New learnings: 4 for rust-dev (bus factor, mailmap, two-pass hotspots, composition)
- Pruned: 8 rust-dev gotcha entries consolidated to 3 (65→56 lines)
- Key insight: Parallel rust-dev dispatch on shared files (common.rs, integration.rs) works when changes are additive (new modules + new tests, not edits to same functions). Round 2 proved this with zero merge conflicts. Subcommand composition pattern (trends reusing metrics::run() + churn::run()) scales cleanly. 5/5 first-attempt successes (streak: 75+).
- Backlog state: 260/280 closed, 20 open (16 ready, 4 blocked)

## Retro: 2026-02-14 (session 13 — git-intel integration audit)
- Tasks completed: 4 (2 doc structure updates + install.sh optional build + git-pulse.sh delegation)
- Workflow: /fractal (3 parallel handlers) → assess → /sprint (3 serial dispatches, all infra-owned)
- Commits: 0 (uncommitted — session end will commit)
- New learnings: 2 for infra (three-gate delegation pattern, --skip-* flag pattern)
- Key insight: Fractal→sprint pipeline is effective for audit-then-fix workflows. Fractal identifies gaps with evidence, sprint fixes them. All 4 dispatches were first-attempt successes (streak continues). install.sh now handles optional Rust toolchain gracefully — never blocks users without cargo.
- Backlog state: 183/183 closed, 0 open
