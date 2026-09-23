# ECC comparison and Fénix improvements

[Español](../es/ecc-review.md)

The review of [affaan-m/ECC](https://github.com/affaan-m/ECC) led to **five original skills and thirteen revised Fénix skills**, bringing the catalog from 181 to 186. Most ECC outcomes already have a Fénix entrypoint; the useful changes add distinct procedures or deepen specific decisions rather than importing the whole distribution.

## Scope and evidence

Source revision: [`2b6e839771e53096d8451a213d40dc64ec8acac0`](https://github.com/affaan-m/ECC/tree/2b6e839771e53096d8451a213d40dc64ec8acac0), inspected on 2026-09-21. The checkout contains **292 canonical `skills/*/SKILL.md` files** and **903 `SKILL.md` files overall**: 518 under translated documentation, 43 under `.kiro`, 39 under `.agents` and 11 under `.cursor`, in addition to the main 292. These copies/adaptations are not 903 independent capabilities.

All 292 canonical names, description excerpts and headings were screened and mapped to an explicit decision. **27 selected entrypoints were read in full**, covering the procedures that motivated changes and the agent-learning/evaluation mechanisms most relevant to Fénix. Selected auxiliary reads include the complete stocktake incremental scanner and compliance grader, the first 150 lines of its runner, the first 130 lines of the learning observer instructions, and observation/cost hook entries. The [source inventory](../skills/ecc-source-review.json) records review depth, hashes, revision, file-specific license declarations and unreviewed boundaries. The [292-row capability map](../skills/ecc-capability-map.md) makes every decision inspectable.

This is a complete catalog comparison with focused full-text review, not a claim that all 76,166 lines of canonical skill bodies, all translations, all scripts or all remote dependencies were read. No ECC installer, hook, helper, external service or model wrapper was executed. Downloading and hashing files are inventory operations, not security or behavior validation.

## New procedures

| Fénix skill | Practical addition | Boundary |
| --- | --- | --- |
| [automation-audit](../../resources/skills/automation-audit/SKILL.md) | Inventory jobs, hooks and integrations; distinguish configured, enabled, authenticated and observed execution; trace duplicate effects. | Does not start jobs merely to inspect them. |
| [notification-triage](../../resources/skills/notification-triage/SKILL.md) | Consolidate alert noise while retaining severity changes, recurrence, ownership and delivery evidence. | No new notification backend or implicit sending. |
| [llm-cost-optimization](../../resources/skills/llm-cost-optimization/SKILL.md) | Reconcile unique requests, retries and pricing; compare cost per successful task; distinguish preflight reservations from after-the-fact spend checks. | No automatic provider switch, purchase or hard spending cap from prose. |
| [patent-research](../../resources/skills/patent-research/SKILL.md) | Search patent documents with precise identifiers, family/date distinctions, official records and located technical evidence. | No legal opinion or implied completeness; no proprietary database connector added. |
| [inventory-planning](../../resources/skills/inventory-planning/SKILL.md) | Produce dated replenishment proposals with commitments, receipt timing, demand uncertainty and ordering constraints. | No automatic purchase or ERP mutation; no universal service/stock thresholds. |

The patent reference links current [USPTO search entrypoints](https://www.uspto.gov/patents/search), [WIPO PATENTSCOPE](https://www.wipo.int/en/web/patentscope) and the [European Patent Register](https://www.epo.org/en/searching-for-patents/legal/register). These portals have different coverage; the skill requires checking the relevant record and observation date rather than inferring global legal status.

## Existing skills improved

| Skill | Change and reason |
| --- | --- |
| `agent-workflow-diagnosis` | Trace the same request through context, execution, interpretation, rendering and persistence. Measure actual loaded context, not all installed files. |
| `api-integration` | Follow current in-repository connector conventions and verify registration/discovery, not just transport. |
| `api-contract-design` | Canonical consumer/provider artifact, serialized output checks, alternate paths and old-client compatibility. |
| `browser-regression-testing` | Inspect ordered state writes/resets and asynchronous results that undo otherwise successful clicks. |
| `cache-design` | Include processing revision/options and authorized scope in content-based keys; exclude partial/error results. |
| `customer-support` | Reconcile shipping, partial returns, replacement and refund evidence using current policies. |
| `data-pipelines` | Separate historical backfill and live arrivals; check row identity and transformation semantics alongside throughput. |
| `memory-and-reminders` | Preserve provenance and project scope for learned lessons; repetition or lack of correction is not independent confirmation. Use the owner's configured timezone rather than a fixed location. |
| `performance-diagnosis` | Bound optimization trials, preserve failed variants and compare freshness separately from response speed. |
| `scheduled-workflows` | Explicit stopping/progress criteria and reconciliation for repeated runs using existing runtime limits. |
| `skill-authoring` | Audit canonical capabilities and complete resource hashes; preserve review depth and distinguish missing usage from proven non-use. |
| `team-messaging` | Resolve destination/audience through the authorized connection and prevent forwarding private context beyond it. |
| `experiment-and-benchmark-design` | Check grader failure cases, separate outcome success from procedural adherence, and distinguish worktree separation from execution isolation. |

Eight new conditional references provide detail without loading it on every task. Thirteen previous stock entrypoints were retained as migration fixtures and their hashes added to the recognized-upgrade list. New builds update recognized unmodified copies and preserve owner edits and enabled/disabled state.

## What was not imported

The per-entry classification is: 143 outcome-level matches, 29 sources informing existing-skill improvements, nine sources grouped into five new procedures, 45 entries dependent on external integrations, 36 host-specific mechanisms, 28 deferred specializations and two deprecated compatibility entries. Several source entries can map to one Fénix skill; these numbers do not mean 292 separate implementations were added.

ECC's host hooks, Memory Vault, background learning observers, orchestration engines and installation commands were not adopted. Fénix already owns task identity, storage, isolation, discovery and execution. A hash of a Git remote, a directory or an agent label is not a substitute for SaaS tenant identity. Existing memory tools remain canonical; no cross-account synchronization or transcript export was enabled.

Specific design differences supported by the inspected source:

- `context-budget` uses fixed token heuristics. Fénix's procedure distinguishes actual loaded content, tokenizer estimates and provider usage; the installed catalog is not all prompt overhead.
- `cost-aware-llm-pipeline` illustrates checking accumulated cost before a call and recording it afterward. The new procedure explains that a hard cap also needs atomic pre-dispatch reservations, concurrency and retry accounting in the runtime. Embedded model prices are not treated as current billing evidence.
- `continuous-learning-v2` documents confidence increases from uncorrected behavior and project-to-global promotion. Fénix preserves the existing scoped admission rules, independent evidence and explicit corrections; it does not install that observer or treat confidence as authority.
- The inspected `skill-stocktake/scripts/quick-diff.sh` finds entrypoint changes through modification time. Fénix's maintenance guidance includes resource-content hashes and added/deleted files, which can reveal changes an entrypoint timestamp misses.
- The inspected `skill-comply` runner starts an external model CLI and permits generated setup commands including package managers in a temporary directory. Fénix retains actual execution isolation and separates instruction compliance from outcome correctness; neither a directory name nor a passing sequence grade establishes those properties.
- Commercial/logistics examples contain specific roles, thresholds and deadlines. The original Fénix procedures require the actual ledger/policy and applicable current terms. They do not assume the user runs a particular enterprise or adopt its hypothetical numerical targets.

Framework/language recipes, specialist healthcare, blockchain, manufacturing, genomics, media engines and external providers remain explicitly mapped with their limitations. A generic workflow match is not SDK/API parity or a compliance certification. No paid subscription or new connector was enabled.

The repository has an MIT root license, while some domain entrypoints explicitly declare Apache-2.0 and other entries record third-party origins. All new Fénix prose is original. No upstream prompt bodies, source code, fixtures or assets were copied into the runtime library.

## Validation and activation

Validation separates format, discovery/installation and model behavior. The new development corpus has **56 English/Spanish queries**: 20 for additions, 26 for revised workflows and ten neighboring-task guards. Baseline comparison disables the new skills, restores saved entrypoints and removes new references from that condition. Native tests also cover stock upgrades, customized bodies/references, disabled state and resource traversal rejection. Exact results are in the [validation record](../research/ecc-validation-2026-09-21.json).

These checks do not measure a general gain in model intelligence. No paid end-to-end model comparison or authenticated integration test was run. The code and bundled resources are updated; activating them in a running installation requires the normal application rollout. This task does not restart or deploy the service.

Executed results: **18/18 changed entrypoints passed format validation; 35/35 native skill tests passed**. All 56 candidate queries retrieved the intended skill in the top three (55 at rank one). For the 26 existing-workflow cases, top-three retrieval changed from 21 to 26; all ten neighboring-task guards passed in both conditions. The 20 new-skill cases have no same-skill baseline because those entries did not exist. These are development queries, not a held-out behavioral benchmark.
