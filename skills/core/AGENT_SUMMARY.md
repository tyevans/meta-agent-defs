# Skill Summaries

**Consider how these approaches can be composed into a workflow accomplishing your goal:**

- **do** – The primary dispatch skill that reads the canonical skill catalog, matches a natural‑language goal to the best single skill or multi‑step pipeline, and runs it automatically. An agent gives *do* a goal (e.g., “find the best testing strategy”) and receives a ready‑to‑run sequence, with a checkpoint after each step for resumability.

- **transform** – A map‑style rewrite that applies a user‑supplied instruction to every item in a list, one at a time, preserving numbering and provenance. Agents use it to convert findings into ticket titles, re‑phrase content for non‑technical audiences, or format outputs consistently.

- **sketch** – Generates an empty skeleton (files, sections, configs, schemas) with only `TODO` stubs, based on an argument or prior primitive output. Agents run *sketch* after a “gather → rank → distill” sequence to outline the structure of a new feature or document before filling in details.

- **rank** – Scores and orders items according to user‑chosen criteria (e.g., complexity, risk). An agent feeds *rank* a list and the desired dimensions; the result is a numbered table together with a concise rationale for the ordering.

- **expand** – Takes sparse points and elaborates each one into a full description, adding context, examples, or evidence while keeping the original numbering and source attribution. Agents use *expand* after a **distill** to flesh out distilled key take‑aways before drafting a spec.

- **distill** – Summarizes verbose input into a compact list or paragraph, optionally focused on a keyword. Agents use it to condense the output of a **gather** step into the most essential nuggets that can be further processed.

- **diff‑ideas** – Compares two alternatives side‑by‑side, scoring them on specified dimensions and producing a comparison table with a recommendation. An agent collects evidence for each alternative, runs *diff‑ideas*, and obtains a clear win‑or‑trade‑off summary.

- **assess** – Evaluates items against a rubric and assigns a categorical verdict (CRITICAL, WARNING, SUGGESTION) with a brief rationale. Agents invoke *assess* when triaging findings from a **gather** or **distill** step, creating a risk‑based prioritization.

- **critique** – Performs an adversarial review of any pipe‑format output, uncovering flaws, gaps, or risks. An agent can run *critique* on prior results to generate a structured list of issues, each labeled and graded by severity.

- **verify** – Checks each claim in a list for evidence, dispatching parallel agents to search source code, Git history, and the web; each claim is marked VERIFIED, REFUTED, or UNCERTAIN with citations. Agents use *verify* after drafting a set of assertions to ensure factual correctness before publishing.

- **decompose** – Breaks a large or vague task into 3–6 bounded, collectively exhaustive sub‑tasks with clear scopes and independence levels. An agent applies *decompose* to an overarching goal, then schedules the sub‑tasks for parallel or sequential execution.

- **merge** – Unifies all pipe‑format blocks from the current conversation (and optionally from the task list or message history) into a single, deduplicated, renumbered output, optionally filtering by topic. Agents use *merge* to clean up results from multiple primitives into a cohesive report ready for downstream use.

- **gather** – Collects evidence from the codebase, configuration files, and the web into a single list, optionally running in parallel collection mode if multiple source types are involved. After gathering, agents may feed *gather*’s output into *distill* or *rank* for further refinement.

- **plan** – Orders items by dependency rather than by importance, producing an execution schedule that respects prerequisites and highlights parallel work waves. An agent takes a list of tasks, figures out dependencies (often by inspecting code), and outputs a pipeline diagram with rationale for the sequencing.

- **discover** – Scans all skill front‑matter to create a recommendation engine that matches a user’s natural‑language goal to the top 2–4 most relevant skills or canonical pipelines. Agents invoke *discover* to surface which combination of skills will most effectively solve the stated goal.

- **filter** – Performs a binary keep‑or‑drop operation on an item list based on a clear criterion (e.g., “security‑related”, “confirmed only”). An agent feeds *filter* a list and a filter rule, then retains only the matched items, re‑enumerating them and recording dropped items in the output for transparency.
