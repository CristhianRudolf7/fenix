# gstack review and original Fénix improvements

[Español](../es/gstack-review.md)

Reviewed on 2026-09-21 against [garrytan/gstack at a6b3a57512ca6d5c6aa5b68f74f736195021f96e](https://github.com/garrytan/gstack/tree/a6b3a57512ca6d5c6aa5b68f74f736195021f96e). The checkout declares package version 1.87.4. This is a static capability and workflow comparison, not an upstream security certification or a measured claim that one agent is more intelligent.

The useful gaps are developer onboarding audits, evidence-based check reports and engineering retrospectives. Fénix now has three original skills for those outcomes and eight revised skills. The catalog grows from 177 to 180. Many gstack practices were already present: falsifiable product discovery, diff review with caller tracing, defect reproduction, visual comparisons, scoped memory, model experiments, and preservation of user requirements. Adding aliases for every upstream command would duplicate those capabilities.

## Coverage and reproducibility

The [source inventory](../research/gstack-source-inventory-2026-09-21.json) records paths, hashes and sizes. There are 61 `SKILL.md` files: 53 top-level workflows, one router, four OpenClaw adaptations, one browser example and two test fixtures. There are 102 templates including lazy sections. These counts describe this commit, not an advertised fixed number of tools.

The review inventoried all entrypoints and template section outlines, examined workflow instructions and selected linked sections, and inspected relevant architecture, browser/security boundaries, hook behavior and build/test declarations. Deep examination focused on DX, health, retrospectives, planning, QA, release evidence, documentation, context handling and skill authoring. Not every implementation file, generated preamble or template line was read; inventory/hashing is not full-text review. The upstream installer, hooks, browser, device bridge, model wrappers, remote memory and tests were not executed. The four OpenClaw adaptations and two template-only host/alias entries are accounted for below; test fixtures are not counted as product capabilities.

## Capability decisions

“Existing” means a matching Fénix workflow, not parity with an upstream executable. A skill cannot create a browser, device connection, OAuth integration, isolation boundary or cloud service.

| gstack surface | Function and assessment | Fénix decision |
| --- | --- | --- |
| `gstack` router | Selects workflows; generated startup also handles state and helper integration. | Existing `skill_search`/`skill_load`; retain narrow automatic candidates and on-demand resources. No global preamble import. |
| `office-hours` | Challenges demand assumptions and explores builder ideas. | Existing `product-discovery`, `user-research`, `competitive-research`; current workaround, contradictory evidence and stopping criteria are already covered. |
| `plan-ceo-review` | Reconsiders scope and evaluates product/system consequences. | Existing product/planning skills; preserve the user's chosen scope instead of assuming expansion is desirable. |
| `autoplan` | Combines product, design, DX and engineering reviews and records amendments. | Improve `requirements-and-planning`: retain accepted obligations, revisit affected dependencies and deliver the amended plan. No mandatory review chain or extra approval ceremony. |
| `plan-eng-review` | Architecture, tests, performance, recovery and distribution review. | Improve `architecture-design` with failure ownership and delivery/update paths. Existing `test-strategy` covers meaningful tests. |
| `spec` | Produces executable issue specifications and optional implementation handoff. | Existing `requirements-and-planning` and `issue-tracker-workflows`; filing issues remains a separate authorized effect. |
| `plan-design-review` | Evaluates planned interaction and visual choices. | Existing frontend and brand skills; preserve constraints and distinguish proposed behavior from rendered evidence. |
| `design-consultation` | Establishes a reusable visual system. | Existing `brand-and-design-tokens`; keep established brand and representative components. |
| `design-shotgun` | Generates alternatives and collects structured preference feedback. | Existing native design presentation/choice workflow; no mandatory paid generation, taste profiling or fixed variant count. |
| `design-html` | Converts design direction to HTML with a Pretext-specific path. | Existing frontend implementation and native design publication; do not mandate another rendering dependency. |
| `design-review` | Visual audit, repair and before/after verification. | Improve `frontend-design-and-accessibility`: audit-only stays informational; authorized fixes verify the same view and real interactions. |
| `plan-devex-review`, `devex-review` | Planned and live developer journeys, setup friction and recoverable errors. | New `developer-experience-review`, with separate plan/live evidence. No invented novice timing or universal score. |
| `browse` | Browser observation and interaction via Aside or a Chromium daemon. | Existing Fénix browser tools and `browser-navigation`. No parallel browser stack added. |
| `open-gstack-browser`; template alias `connect-chrome` | Visible browser launch and sidebar control. | Existing browser UI/runtime; external engine and extension installation are out of this skill change. |
| `setup-browser-cookies` | Imports desktop browser login state for the fallback engine. | Do not import host-wide cookies into a multiuser assistant. Keep existing user-bound browser sessions and explicit connection flows. |
| `pair-agent` | Pairs another agent to the browser, optionally over a tunnel. | Do not transfer broad browser/session authority. Existing `task-delegation` handles supported scoped workers; remote browser pairing is not implemented here. |
| `scrape` | Structured page extraction through browser state. | Existing `web-data-extraction` and native web research; keep provenance, schema validation and pagination where authorized. |
| `browser-skills/hackernews-frontpage` | Fixture-backed scraper example. | Existing native web/Hacker News retrieval; no hardcoded third-party scraper copied. A historical fixture cannot establish current site compatibility. |
| `skillify` | Turns an observed successful extraction into a reusable script and fixture. | Improve `skill-authoring` with execution provenance, changed-input checks, complete resource sets and secret-free fixtures. |
| `qa`, `qa-only` | Browser testing with repair or reporting modes. | Improve `browser-regression-testing`: explicit effect scope, persisted outcomes, replayable failures and bounded repair attempts. |
| `investigate` | Root-cause investigation followed by repair. | Existing `test-driven-repair` and `local-troubleshooting`; avoid a second debugging procedure. |
| `review` | Diff review, additional reviewers and finding handling. | Existing `code-review`/`review-feedback` already trace consumers and separate review from authorized correction. No auto-fixing during review-only work. |
| `codex`; template-only `claude-code` | External model review/challenge/consult wrappers. | Existing authorized delegation and model configuration; no new provider subscription, CLI login or mandatory independent reviewer. |
| `health` | Runs project checkers, tracks results and computes a composite score. | New `code-health-review`; keep actual exit status and missing coverage, omit arbitrary aggregate scores. |
| `benchmark` | Browser performance baselines and comparisons. | Improve `performance-diagnosis` with comparable browser conditions, missing metrics, failed navigation and retained baselines. |
| `benchmark-models` | Cross-provider latency, usage and optional judged quality. | Existing `experiment-and-benchmark-design` and `model-selection`; preserve trial identity, repeats, pricing provenance and evaluation limits. |
| `ship` | Tests, reviews, versioning, commits, push and PR creation. | Improve `release-and-ci` to bind evidence to final inputs, including manifests; no universal version scheme, staging all dirty files or implicit publication. |
| `setup-deploy` | Discovers and stores deployment configuration. | Existing `cloud-deployment` and release conventions; do not replace project configuration with gstack-specific files. |
| `land-and-deploy` | Merge, deployment tracking, canary and recovery. | Existing deployment/release skills; new release observation reference checks real artifact identity and uncertain remote outcomes. |
| `canary` | Finite post-deployment observations against a baseline. | New conditional reference in `release-and-ci`: pre-deploy vs monitoring-start distinction, error identities, bounded cadence and authorized rollback. |
| `landing-report` | Reports concurrent PR/version reservations. | Existing `project-status-review`, release and worktree coordination. No global scan of sibling user workspaces or four-part version allocator. |
| `document-generate`, `document-release` | Documentation creation and diff-driven maintenance. | Improve `technical-documentation` with public-surface coverage and diagram drift; update only useful document types and preserve history. |
| `retro` | Delivery/churn metrics and team retrospectives. | New `engineering-retrospective`: equal windows, actual delivery records and explicit denominators; omit inferred work hours and employee rankings. |
| `context-save`, `context-restore` | Persist and restore session summaries and Git state. | Existing task/checkpoint/history tools and `conversation-recap`; no competing context database or cross-project fallback. |
| `learn` | Searches, edits and exports accumulated lessons. | Existing `memory-and-reminders` and `knowledge-base-maintenance`; no automatic promotion of generated judgments to owner facts. |
| `plan-tune` | Tunes clarification behavior and stores declared/inferred profiles. | Existing explicit preferences and session authorization suffice; no psychographic profiling or silent preference inference. |
| `setup-gbrain`, `sync-gbrain` | Configures/synchronizes external code and memory indexes. | Existing repository navigation, memory and MCP workflows; no external transcript ingestion, database provisioning or automatic egress added. |
| `careful`, `freeze`, `guard`, `unfreeze` | Optional destructive-command warnings and directory edit hooks. | Keep native Fénix permissions/isolation. Upstream explicitly notes edit hooks do not restrict Bash writes; they are not tenant isolation. |
| `cso` | Security assessment with differentiated evidence and repair assurance. | Existing dedicated security skills/profile already separate candidate findings, reproduction and fix validation. No untested security runtime imported. |
| `ios-qa`, `ios-fix`, `ios-design-review` | Real iPhone testing using an embedded debug bridge, repair and visual review. | Existing mobile testing/development guidance; real-device support still needs an available authorized bridge. No device capability claimed from prose. |
| `ios-sync`, `ios-clean` | Maintains/removes the debug bridge and guards release builds. | Upstream-specific lifecycle; not installed. Keep debug tooling out of release artifacts when relevant to existing mobile work. |
| `diagram` | Editable diagram source and rendered artifacts. | Existing `diagram-authoring` and native design tools; no renderer copied. |
| `make-pdf` | Markdown/HTML typesetting and PDF export with layout controls. | Existing `pdf-workflows`, document and LaTeX skills cover relevant tasks with available tools. No claim that Fénix now has gstack's exact HTML typesetter or export fidelity. |
| `gstack-upgrade` | Updates the distribution and migrates its state. | Existing stock-skill installer and historical-hash migration; preserve edited files and disabled state. No auto-update of external code. |
| OpenClaw CEO/office-hours/investigate/retro adaptations | Alternate entrypoints for overlapping workflows. | No four duplicate skills. Apply the same scope/evidence decisions as their main counterparts. |

## Why these changes fit Fénix

The three new outcomes are different from adjacent skills. A DX audit asks whether a consumer can use an interface; a code-health report asks what configured checks actually say; an engineering retrospective examines delivery over time. None should replace product discovery, a diff review, a current-status request or a completion check. Their English/Spanish metadata and scope sections encode these distinctions.

The eight revised entrypoints are `requirements-and-planning`, `architecture-design`, `frontend-design-and-accessibility`, `browser-regression-testing`, `performance-diagnosis`, `release-and-ci`, `technical-documentation` and `skill-authoring`. Release observation is a separately loaded reference. All changes use Fénix tool names and existing mechanisms. No upstream prompt bodies, code, assets, installation commands or host-specific hooks were copied.

These are instruction improvements, not new enforcement. In particular, a skill's `effects` field does not sandbox execution; report-only checks can create temporary artifacts; and project test commands run code. Fénix's runtime remains responsible for identity, filesystem/network permissions and isolation. Upstream's architecture describes a single-user workspace browser; adding it wholesale would not establish SaaS tenant isolation.

## Deliberate differences

- Keep the owner's explicit requirements and existing authorization. Avoid mandatory repeated questions, universal review stages, expansion targets or forced commits for small edits.
- Preserve evidence granularity: observed, inferred, untested, failed and unavailable. A model's agreement or a numerical quality score is not independent validation.
- Do not infer working hours from commit spacing, actual PRs from commit counts, or productivity from changed lines. Empty periods and missing baselines are legitimate outcomes.
- Do not accept stale tests merely because edited files are called manifests or release metadata; those can change the artifact.
- Treat real browser sessions and externally synchronized memory as separate integrations with their own authority. None was enabled by this review.
- Use existing isolated workers and explicit task history rather than upstream telemetry, identity profiles or automatic transcript upload.

## Validation and activation

The 11 affected entrypoints passed the skill format validator. The new native regression suite evaluates 26 English/Spanish discovery cases against the original and revised catalog, including eight neighboring-task guards. A second test covers eight stock upgrades, owner customizations, disabled state, new skill loading and preservation of a customized release reference. Results and limits are recorded in [validation evidence](../research/gstack-validation-2026-09-21.json).

This is a development regression set, not an independent LLM task benchmark. It establishes format, retrieval and installation behavior, not a numerical gain in general intelligence. The source review did not run gstack or authenticated external integrations.

Cargo bundles new skills and resources automatically. Before revising the eight original entrypoints, their bytes were retained as migration fixtures and their SHA-256 hashes added to the stock upgrade list. Updated builds install new skills and upgrade only recognized unmodified copies while preserving activation state and customized references. The changes are in the repository; the active service was not restarted or deployed by this task.
