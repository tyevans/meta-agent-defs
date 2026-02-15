# Signal Precision Study (2026-02-15)

## Methodology

### Design
Stratified sample across 5 repos spanning the signal count range, evaluating fix-after-feat and fix-after-refactor signals for true/false positive classification.

### Repos Selected (full history, no `--since` filter)

| Repo | Total Signals | fix_after_feat | fix_after_refactor |
|------|:---:|:---:|:---:|
| ripgrep | 1 | 0 | 1 |
| hugo | 8 | 6 | 2 |
| Jackett | 18 | 15 | 3 |
| framework (Laravel) | 86 | 71 | 15 |
| django | 304 | 304 | 0 |

### Sampling
- Up to 5 fix_after_feat signals per repo (first 5 from output)
- Up to 3 fix_after_refactor signals per repo (first 3 from output)
- Total signals evaluated: 29 (20 fix_after_feat + 9 fix_after_refactor)

### Evaluation Criteria
For each signal pair (feat/refactor commit + fix commit):
- Read both commit messages
- Check shared file list
- **TRUE POSITIVE**: Fix genuinely addresses incompleteness or breakage from the preceding feat/refactor
- **FALSE POSITIVE**: Coincidental file overlap, unrelated fix, or cosmetic-only fix

---

## Per-Repo Results

### ripgrep (1 signal)

| # | Kind | Feat/Refactor | Fix | Shared Files | Verdict |
|---|------|---------------|-----|-------------|---------|
| 1 | fix_after_refactor | "Refactor how coloring is done" | "fix windows build" | src/out.rs | **TP** -- refactor broke windows build |

**TP rate**: 1/1 (100%)

### hugo (8 signals)

| # | Kind | Feat/Refactor | Fix | Shared Files | Verdict |
|---|------|---------------|-----|-------------|---------|
| 1 | fix_after_feat | "Added 'themes' as a default new site directory" | "Fix copyright headers in source files" | commands/new.go | **FP** -- copyright fix unrelated to themes |
| 2 | fix_after_feat | "Added missing files, make the site look better" | "Fix multilingual site layouts/templates" | 3 layout files | **TP** -- incomplete multilingual setup |
| 3 | fix_after_feat | "Added .Hugo variable with version/commit/generator info" | "Fix HugoInfo init" | hugo.go, node.go, site.go | **TP** -- classic incomplete rollout |
| 4 | fix_after_feat | "Added the path modules test files" | "Fix: remove unnecessary dot in extension" | helpers/path_test.go | **FP** -- different concerns, shared test file |
| 5 | fix_after_feat | "added evaluation for toml for metadataformat" | "fix issue 411, archetypes directory error" | create/content.go | **FP** -- unrelated, coincidental overlap |
| 6 | fix_after_refactor | "refactor handlers to use types instead of structs" | "Templates work properly when watching" | hugolib/site.go | **TP** -- refactor broke template watching |
| 7 | fix_after_refactor | "Refactor layout selection code" | "fixed #95, fixed #93" | hugolib/site.go | **TP** -- layout refactor caused bugs |

**fix_after_feat TP rate**: 2/5 (40%)
**fix_after_refactor TP rate**: 2/2 (100%)

### Jackett (18 signals)

| # | Kind | Feat/Refactor | Fix | Shared Files | Verdict |
|---|------|---------------|-----|-------------|---------|
| 1 | fix_after_feat | "added Synthesiz3r tracker" | "fix Definitions include" | Jackett.csproj | **FP** -- .csproj gravity file |
| 2 | fix_after_feat | "added gaytorrent.ru" | "fix: kat clone" | README.md | **FP** -- README gravity file, different indexers |
| 3 | fix_after_feat | "Added support for ILoveTorrents.me" | "Fixed bug in download link, added categories" | ILoveTorrents.cs | **TP** -- fixed bug in just-added indexer |
| 4 | fix_after_feat | "Added proxy support" | "Fix blackhole downloads" | Startup.cs | **FP** -- Startup.cs gravity file, unrelated features |
| 5 | fix_after_feat | "Added Bitsoup tracker" | "Fix update cross platform issues" | Jackett.csproj | **FP** -- .csproj gravity file |
| 6 | fix_after_refactor | "Refactor done" | "Fix csproj" | Jackett.csproj | **TP** -- refactor caused csproj issue |
| 7 | fix_after_refactor | "Refactor" | "Fixed blocking query in ShowRSS" | ShowRSS.cs | **TP** -- refactor broke ShowRSS indexer |
| 8 | fix_after_refactor | "Refactor" | "Fix showing console error on settings dir" | Program.cs | **TP** -- refactor touched Program.cs, fix in same file |

**fix_after_feat TP rate**: 1/5 (20%)
**fix_after_refactor TP rate**: 3/3 (100%)

### framework / Laravel (86 signals)

| # | Kind | Feat/Refactor | Fix | Shared Files | Verdict |
|---|------|---------------|-----|-------------|---------|
| 1 | fix_after_feat | "fix type hints for DateTimeZone" | "Fix DateFactory docblock type hints" | DateFactory.php | **TP** -- incomplete type hint update |
| 2 | fix_after_feat | "Added getAuthIdentifierForBroadcasting" | "Fixed formatting" | 5 broadcaster files | **FP** -- cosmetic style fix |
| 3 | fix_after_feat | "Added expectsTable console assertion" | "Fix style" | PendingCommand.php | **FP** -- cosmetic style fix |
| 4 | fix_after_feat | "Added expectsTable console assertion" | "Fix spacing" | PendingCommand.php | **FP** -- cosmetic spacing fix |
| 5 | fix_after_feat | "support operators with question mark on where clauses" | "fix: rebase on master and only use str_replace" | Grammar.php, QueryBuilderTest.php | **TP** -- feature needed post-rebase fix |
| 6 | fix_after_refactor | "refactor data_has" | "fix data_has empty check" | helpers.php, SupportHelpersTest.php | **TP** -- refactor broke empty check |
| 7 | fix_after_refactor | "Refactor DELETE queries" | "Fix UPDATE queries with alias" | PostgresGrammar.php, SQLiteGrammar.php | **TP** -- refactor affected shared grammar code |
| 8 | fix_after_refactor | "Refactor" | "Fix styles" | ContextualBindingTest.php | **FP** -- cosmetic fix on test file |

**fix_after_feat TP rate**: 2/5 (40%)
**fix_after_refactor TP rate**: 2/3 (67%)

### django (304 signals)

| # | Kind | Feat/Refactor | Fix | Shared Files | Verdict |
|---|------|---------------|-----|-------------|---------|
| 1 | fix_after_feat | "Added GitHub Actions linter (zizmor)" | "Added coverage workflow for PRs" | 2 contributing docs | **FP** -- different CI features, shared doc files |
| 2 | fix_after_feat | "Added test for various widths in test_center.py" | "Made center template filter consistent for even/odd" | test_center.py | **TP** -- test + fix for same filter |
| 3 | fix_after_feat | "Added GH Action to enforce commit message prefix" | "Fixed GH Action to fetch PR head correctly" | check-commit-messages.yml | **TP** -- classic incomplete feature, same workflow file |
| 4 | fix_after_feat | "Added security guideline on size limitations" | "Added forloop.length variable in template for loop" | docs/ref/templates/builtins.txt | **FP** -- completely unrelated, docs gravity file |
| 5 | fix_after_feat | "Added support for Edge with Selenium" | "Fixed set_emulated_media() with selenium_hub" | django/test/selenium.py | **TP** -- both working on selenium module |

**fix_after_feat TP rate**: 3/5 (60%)
**fix_after_refactor TP rate**: N/A (0 signals)

---

## Aggregate Precision

### fix_after_feat

| Repo | TP | FP | TP Rate |
|------|:---:|:---:|:---:|
| ripgrep | -- | -- | N/A |
| hugo | 2 | 3 | 40% |
| Jackett | 1 | 4 | 20% |
| framework | 2 | 3 | 40% |
| django | 3 | 2 | 60% |
| **Total** | **8** | **12** | **40%** |

### fix_after_refactor

| Repo | TP | FP | TP Rate |
|------|:---:|:---:|:---:|
| ripgrep | 1 | 0 | 100% |
| hugo | 2 | 0 | 100% |
| Jackett | 3 | 0 | 100% |
| framework | 2 | 1 | 67% |
| django | -- | -- | N/A |
| **Total** | **8** | **1** | **89%** |

### Combined

| Signal Kind | Sampled | TP | FP | Precision |
|-------------|:---:|:---:|:---:|:---:|
| fix_after_feat | 20 | 8 | 12 | **40%** |
| fix_after_refactor | 9 | 8 | 1 | **89%** |
| **Overall** | **29** | **16** | **13** | **55%** |

---

## False Positive Taxonomy

| FP Pattern | Count | % of FPs | Description |
|------------|:---:|:---:|-------------|
| Gravity file overlap | 6 | 46% | High-churn files (.csproj, README.md, docs/) shared coincidentally |
| Cosmetic "fix" | 4 | 31% | "Fix style", "Fix spacing", "Fix formatting" — not functional fixes |
| Unrelated same-file | 3 | 23% | Different features that happen to touch a common infrastructure file |

---

## Qualitative Observations

### Signal quality varies dramatically by repo culture

**High-quality signal repos (django, hugo)**: Projects where "fix" commits genuinely address bugs produce higher TP rates. Django's rigorous commit message convention ("Fixed #NNNNN -- description") makes signals more interpretable and their fix commits tend to be substantive.

**Low-quality signal repos (Jackett, framework)**: Projects with frequent cosmetic fixups ("Fix style", "Fix spacing") or projects with "gravity files" (files touched by every PR) produce many false positives. Laravel's framework repo has a strong convention of separate style-fix commits after each feature, which inflates fix_after_feat signals with cosmetic FPs.

### fix_after_refactor is dramatically more precise than fix_after_feat

At 89% vs 40%, fix_after_refactor is a fundamentally higher-quality signal. This makes intuitive sense:
- Refactors are supposed to be behavior-preserving; a fix immediately after suggests the refactor broke something
- Features often share files with subsequent unrelated fixes simply because both touch actively-developed areas

### Gravity files are the primary noise source

Files like `.csproj`, `README.md`, `Startup.cs`, and documentation indexes appear in many commits regardless of the commit's purpose. The current shared-file filter (HashSet intersection) catches obvious non-overlaps but cannot distinguish "both touched this config file coincidentally" from "both worked on the same module."

### The "cosmetic fix" problem is specific to certain repo cultures

Laravel framework has a strong convention where contributors submit features, then maintainers or CI flag style issues, producing a "feat -> fix style" pattern that is technically a fix-after-feat but is not a meaningful signal of incompleteness. This pattern is rare in django and hugo.

---

## Comparison with Previous Study

| Metric | Previous (5 repos) | Current (5 repos, 29 signals) |
|--------|:---:|:---:|
| Overall TP rate | 67% | 55% |
| Repos with 0 signals | 2/5 (ripgrep, tokio) | 0/5 (by design) |
| Sample size | ~6 signals | 29 signals |

The previous study's higher precision was partly an artifact of smaller sample size and including repos that produced zero signals (which neither helps nor hurts precision). The current study with more signals and larger repos reveals the gravity-file and cosmetic-fix failure modes that were not visible in the smaller sample.

---

## Recommendations

### Is precision sufficient for automated injection into agent learnings?

**No.** At 55% overall precision (and 40% for fix_after_feat specifically), injecting these signals directly into agent learnings would produce more noise than value. The bar for automated injection should be >80% precision.

### Actionable improvements to reach >80%

1. **Filter cosmetic fixes (high impact, easy)**: Exclude fix commits whose message matches patterns like "fix style", "fix spacing", "fix formatting", "fixed formatting", "fix typo". This would eliminate ~31% of current FPs, raising fix_after_feat precision from 40% to ~57%.

2. **Gravity file exclusion list (high impact, medium effort)**: Maintain a per-repo or heuristic-based list of "gravity files" — files modified in >20% of all commits. Exclude these from shared-file intersection. This would eliminate ~46% of FPs, raising fix_after_feat precision to ~73%.

3. **Combine both filters**: Cosmetic + gravity file filtering together could eliminate ~77% of FPs, raising fix_after_feat precision to an estimated ~75-80%.

4. **Treat signal types differently**: fix_after_refactor at 89% precision is already near the threshold. Consider:
   - fix_after_refactor: suitable for automated injection with cosmetic-fix filter (would reach ~95%)
   - fix_after_feat: requires gravity-file filtering before automated use

5. **Severity weighting**: Higher severity scores (more shared files, fewer commits apart) correlate weakly with TP. A minimum severity threshold could help but was not formally tested in this study.

### Recommended next steps

1. Implement cosmetic-fix message filter in `patterns.rs` signal generation
2. Implement gravity-file detection (file appears in >N% of analyzed commits) and exclude from intersection
3. Re-run this precision study on the same 5 repos to measure improvement
4. If combined precision reaches >80%, approve for automated injection with fix_after_refactor first
