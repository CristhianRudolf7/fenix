# Superpowers and the Fénix workflow core

[Español](../es/superpowers-comparison.md) · [Evidence manifest](../skills/superpowers-review.json) · [Discovery cases](../skills/superpowers-discovery-cases.json)

This report records the first 142-skill iteration. A subsequent [workflow-boundary review](workflow-skill-boundaries.md) reached 145 entries, the [four-repository review](repository-skills-comparison.md) reached 156, the [OpenClaw review](openclaw-skills-review.md) reached 164, and the current [ten-repository review](ten-repository-skills-review.md) reaches 167. The original measurements below remain historical evidence.

Reviewed on **2026-09-19**. Target: the Fénix agent in this repository, not the personal Codex installation used to edit it.

## Decision

**Fénix already has the capabilities covered by Superpowers, but several procedures and discovery triggers benefit from more depth.** The appropriate integration is an original revision of nine existing skills and five optional references. The catalog remains at 142 skills. Installing a second orchestration framework or copying all 15 upstream skills would duplicate responsibilities and introduce conflicting rules.

This is a finding about documented coverage, native integration and measured retrieval. It is **not proof that Fénix now solves more programming tasks or uses fewer tokens**. No live-model baseline/candidate benchmark was run for this change. The behavioral benefit of the deeper procedures remains an engineering hypothesis with explicit mechanisms and evaluation cases below.

## Source and review boundaries

The upstream repository is [obra/superpowers](https://github.com/obra/superpowers/tree/5bf4e78011075bcfc0dc295f0724994cd123ee71), version **6.4.1**, commit `5bf4e78011075bcfc0dc295f0724994cd123ee71`. Its commit timestamp is 2026-09-18 17:31:35 −07:00, or 2026-09-19 00:31:35 UTC. Pinning the commit makes this comparison reproducible even when upstream changes.

The review read all **15 skill entrypoints**, plus selected supporting references, plugin manifests and the session-start hook. It inventoried the test layout and read the Claude test-suite documentation. It did not execute upstream scripts, audit every helper, run its model evaluations or verify every supported host. The README points to a separately cloned evaluation project; that project was not evaluated here. The [MIT license](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/LICENSE) was inspected. No upstream bodies, scripts or assets are redistributed by this change; the manifest attributes the design source.

For Fénix, the review inventoried all 142 entrypoints and inspected the relevant engineering/coordination procedures, system instructions, skill packaging/search/loading/migration code and planning/recovery documentation. Inventory is not equivalent to a full quality audit of every unrelated office, research or business skill. The prior [66-repository survey](skill-source-review.md) had consulted sections of three Superpowers skills at the same commit; this focused review adds the complete 15-entrypoint comparison without rewriting that historical survey's counts.

Codex's session skills and Fénix's bundled skills are separate systems. Official OpenAI documentation describes on-demand skill loading in Codex; it does not establish what Fénix supports. Fénix support was checked in its own code. [Official skill documentation](https://learn.chatgpt.com/docs/build-skills).

## All 15 upstream skills

“Covered” means a matching outcome exists in local instructions or runtime code. It does not mean equal effectiveness across models. Local links below show the revised procedures; their exact pre-change versions are retained in test fixtures and identified in the evidence manifest.

| Superpowers skill | Existing Fénix coverage | Gap or tradeoff found | Integration decision |
| --- | --- | --- | --- |
| [using-superpowers](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/using-superpowers/SKILL.md) | `skill_search`, `skill_load`, `task_skill_guidance`; system routing | Discovery already exists. A universal mandatory trigger would expand context and process for unrelated tasks. | Retain native routing; improve relevant descriptions and bilingual aliases. No bootstrap hook. |
| [brainstorming](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/brainstorming/SKILL.md) | [requirements-and-planning](../../resources/skills/requirements-and-planning/SKILL.md), `architecture-design`, versioned plan briefs | Existing intent/assumption separation is strong; proportional design paths could be clearer. | Clarify bounded edits, boundary design and experiments. Preserve existing authorization instead of adding repeated design/spec/plan approvals. |
| [writing-plans](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/writing-plans/SKILL.md) | `requirements-and-planning`, native milestone DAG | Plans already have outcomes/dependencies; producer/consumer interfaces were not explicit in the skill. | Add exact interfaces, ownership and acceptance evidence. Keep the native plan as the record rather than another Markdown task database. |
| [executing-plans](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/executing-plans/SKILL.md) | Planner, worker journals, task contracts, checkpoint recovery; [software-implementation](../../resources/skills/software-implementation/SKILL.md) | Durable recovery already exists. More explicit links between deliverables and checks help handoffs. | Strengthen evidence and resumption guidance. Do not import shell ledgers or direct-master execution that conflicts with Fénix's coordination restrictions. |
| [subagent-driven-development](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/subagent-driven-development/SKILL.md) | [task-delegation](../../resources/skills/task-delegation/SKILL.md), one-level workers, adaptive plan review | Worker briefs can identify exact interfaces and distinguish implementation quality from plan approval. | Improve handoff contracts and scoped re-review. Keep native retry, model and permission policies; no obligatory reviewer for every task. |
| [dispatching-parallel-agents](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/dispatching-parallel-agents/SKILL.md) | `task-delegation`, runtime dependencies and effect coordination | Independent scope, ownership and integration checks were already covered. | Retain the mechanism; reinforce exact handoffs rather than add another delegation skill. |
| [systematic-debugging](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/systematic-debugging/SKILL.md) | [test-driven-repair](../../resources/skills/test-driven-repair/SKILL.md), `local-troubleshooting`, domain diagnostics | Missing depth for backward tracing, cross-component comparisons, rejected hypotheses and timing failures. | Add [root-cause investigation](../../resources/skills/test-driven-repair/references/root-cause-investigation.md), including bounded condition waits and suite-order diagnosis. |
| [test-driven-development](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/test-driven-development/SKILL.md) | `test-driven-repair`, [test-strategy](../../resources/skills/test-strategy/SKILL.md), `software-implementation` | Bug reproduction existed; new-behavior checks and independent expectations needed detail. | Add meaningful failure-before-fix checks and test sensitivity. Preserve working code and proportional checks for prose/cosmetic edits. |
| [verification-before-completion](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/verification-before-completion/SKILL.md) | System evidence rules, task receipts, command status and per-skill verification | Core principle already present; a reusable claim/evidence map was missing. | Add [completion evidence](../../resources/skills/software-implementation/references/completion-evidence.md). Freshness depends on relevant artifact changes, not which chat message contains the result. |
| [requesting-code-review](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/requesting-code-review/SKILL.md) | [code-review](../../resources/skills/code-review/SKILL.md), `task-delegation` | Exact diff scope and evidence existed; requirement coverage versus code quality deserved emphasis. | Add that distinction and scoped follow-up reviews. Keep review-only tasks read-only and extra reviewers proportional to uncertainty. |
| [receiving-code-review](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/receiving-code-review/SKILL.md) | `code-review` covered evidence-based disagreement, mostly from the reviewer's perspective | No explicit procedure for checking and dispositioning incoming feedback. | Add [review feedback](../../resources/skills/code-review/references/review-feedback.md): confirmed, unsupported, needs information or deferred. Independent corrections can continue. |
| [using-git-worktrees](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/using-git-worktrees/SKILL.md) | [git-worktree-coordination](../../resources/skills/git-worktree-coordination/SKILL.md) | Missing explicit reuse of existing isolation, ignored-path check and baseline attribution. | Add those checks, distinguish submodules and retain separate ports/data. No forced worktree or dependency installation for every edit. |
| [finishing-a-development-branch](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/finishing-a-development-branch/SKILL.md) | `git-worktree-coordination`, `git-recovery-and-change-management`, `release-and-ci` | Handoff state and cleanup ownership could be more precise. | Record base/head, dirty files, checks and integration state. Directory naming does not prove ownership; existing authorization determines merge/PR/cleanup. |
| [writing-skills](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/writing-skills/SKILL.md) | [skill-authoring](../../resources/skills/skill-authoring/SKILL.md), `experiment-and-benchmark-design`, metadata/search tests | Baseline/candidate evaluation was mentioned but had no focused procedure; discovery missed evaluation requests. | Add [behavioral evaluation](../../resources/skills/skill-authoring/references/behavioral-evaluation.md), pressure/negative cases and explicit evidence levels. |
| [diagnosing-superpowers](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/skills/diagnosing-superpowers/SKILL.md) | [local-troubleshooting](../../resources/skills/local-troubleshooting/SKILL.md), `conversation-recap`, task/history tools | General troubleshooting lacked agent-specific attribution and token/time accounting. | Add [agent workflow diagnosis](../../resources/skills/local-troubleshooting/references/agent-workflow-diagnosis.md) using native event references. No mandatory seven-analyst fan-out or automatic export. |

## What the comparison establishes

Fénix's breadth is greater in this checkout: its 142 skills include office documents, cloud connections, research, data, operations and personal work. Superpowers' 15 skills concentrate on software-development process. Those counts describe scope, not quality: many shallow skills can underperform a few detailed ones. Fénix's strongest relevant distinction is its implemented infrastructure: persisted plans and revisions, dependency scheduling, permission enforcement, event provenance, skill search, resource loading and process supervision. Superpowers primarily supplies instructions and host integrations; its text cannot add those capabilities to Fénix.

The useful additions are decision procedures with a specific mechanism. Tracing the first wrong value can reduce speculative fixes. Independently derived expected outputs can expose a test that merely repeats the implementation. Checking feedback against current consumers can prevent an incorrect review suggestion from being applied. Artifact-linked checks can prevent a stale pass from supporting a new completion claim. These are reasons to incorporate narrow guidance, not measured estimates of success-rate gains.

Several upstream rules were deliberately not adopted:

- Mandatory skill invocation before every response: Fénix already searches and loads on demand, and general conversation does not need an engineering pipeline.
- Repeated approval stages for bounded, already authorized work: ask for genuinely missing decisions and honor the runtime's concrete-action policy.
- Deleting working implementation to recreate TDD order: test sensitivity in an isolated copy instead.
- A whole suite after every small step or re-running unchanged evidence in the latest message: run required checks and those affected by changes; label their scope honestly.
- Mandatory fresh reviewers, model-tier changes or extra retry rounds: those belong to the configured runtime and task's needs.
- Another progress ledger beside native plans/checkpoints: competing records create reconciliation work.
- Treating a `.worktrees` path as proof of ownership: cleanup requires evidence that the workspace and its work may be removed.
- The optional upstream visual server: Fénix already has its own Design/browser path; no need to install another service or its assets.

These are compatibility and proportionality decisions for Fénix, not claims that the same choices suit every Superpowers user.

## Implementation and activation

Nine entrypoints were revised, five bilingual references added, and the nine pre-change entrypoints retained as migration fixtures. Their prior exact SHA-256 values were appended to `resources/skills-legacy-hashes.json`. The existing installer can therefore update known bundled originals while retaining customized files and disabled state. No new runtime workflow engine, external tool, startup hook, Codex configuration or permission was introduced.

There is a context cost: the nine entrypoints together grew from **20,019 to 27,978 UTF-8 bytes**; the optional references add **13,653 bytes**. These are file sizes, not tokenizer measurements or per-turn usage. Only selected skills/references load. This makes the tradeoff visible without claiming that additional instructions are free or that reduced rework has already offset their cost.

Bundling happens through [build.rs](../../crates/fenix-agent/build.rs); installation and discovery use [skills.rs](../../crates/fenix-agent/src/skills.rs). References remain outside the initial metadata suggestions and load through `skill_load` with its resource argument. Customized entrypoints remain intact and may need their owner to merge the relevant additions explicitly.

The changes are integrated in source and exercised in isolated runtime directories. A previously built/running Fénix binary does not acquire embedded changes merely because repository Markdown changed: normal rebuilding and starting the updated binary activates them for its configured skill directory. This task does not claim that a live service was rebuilt, restarted or deployed.

## Validation and limits

The pre-change skill suite passed 11 tests. A new frozen 18-query English/Spanish discovery set initially found the intended skill in the top three for 11 requests and missed it for 7. The paired test replays the nine baseline fixtures and then upgrades them inside the same runtime, holding the other skills and search implementation constant. The candidate retrieves the intended skill **first in all 18 cases**, compared with **11/18 in the top three** for the baseline. The final skill suite passes **14 tests**. Per-query results and check status are recorded in the [evidence manifest](../skills/superpowers-review.json).

The added tests exercise actual runtime retrieval, all nine exact-original upgrades, preservation of custom content and activation state, and all five references through the real resource loader. Existing tests cover malformed metadata, irrelevant queries, disabled skills and traversal boundaries. These are integration checks, not tests that simply grep for new prose.

| Check | Observed result |
| --- | --- |
| Same 18 queries, intended skill ranked first | Baseline 10/18; candidate 18/18 |
| Same 18 queries, intended skill in top three | Baseline 11/18; candidate 18/18 |
| Native skill suite | 14 passed, 0 failed |
| Changed skills through skill-creator validator | 9 passed, 0 failed |
| Rust library lints with warnings denied | Passed |
| Changed Rust file formatting and local links/hash integrity | Passed |
| Live-model task quality, tokens, cost and latency | Not measured |

The discovery set is a **development regression set**, not a held-out benchmark. Better rank here proves improved retrieval for these requests, not universal semantic understanding. The deterministic suite also cannot prove that a model follows the instructions once loaded. No success-rate, latency, token or monetary improvement is asserted for model execution.

To validate behavioral benefit, use the same Fénix/model/configuration on isolated baseline and candidate trials with independently scored artifacts. Include: a timing bug where increasing sleeps is tempting; an incorrect reviewer suggestion that breaks compatibility; a passing build that misses the original symptom; a stale test pass followed by an affecting edit; recovery after a completed effect; and a trivial edit that should not acquire extra approval stages. Add transfer cases not used to tune the text, repeat variable trials, and retain failures and missing usage. Only those results can justify a broader quality or efficiency claim.
