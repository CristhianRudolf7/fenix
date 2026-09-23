# Ten-repository review and original Fénix improvements

[Español](../es/ten-repository-skills-review.md) · [Evidence and reading coverage](../skills/ten-repository-review.json) · [Source inventory](../skills/ten-repository-inventory.json) · [Discovery cases](../skills/ten-repository-cases.json)

Reviewed on **2026-09-19**. The ten repositories come from the `skills` area, `valioso` category of the local `proyectos/repo/repo.sqlite` catalog, queried read-only. This review follows the [164-skill OpenClaw iteration](openclaw-skills-review.md).

## Result

Fénix now has **167 original skills**: three new workflows, eighteen procedural revisions and one additional calendar-discovery repair. Eight optional references and two original Python helpers provide depth without adding a provider-specific entry for every integration. The implementation changes 32 files inside the affected skill packages. No upstream framework, executable or asset was imported or run.

The demonstrated gains are more reliable skill discovery and executable checks for paired outcomes and video timelines. The new instructions also cover distinct tasks previously handled only by general procedures. **This is not evidence that Fénix is generally more intelligent than the upstream agents.** No live-model comparison, external account connection or production deployment was performed.

## What was actually read

All ten Git trees were enumerated at fixed commits: 16,181 files and **3,096 `SKILL.md` paths**. Those paths include language/platform duplicates, adapters and test fixtures, so they are not 3,096 independent capabilities. Of 115 selected files downloaded, **35 were read in full and 27 in specified sections**. The remaining downloads received at most structural triage; their bodies are not represented as audited. Linked dependencies outside those reads remain unreviewed.

Selections targeted a concrete Fénix gap, a weak existing procedure, representative integration contracts, or an apparently redundant mechanism. The evidence records reviewed paths and line ranges, download SHA-256, verified Git blob hashes, retrieval dates and every adoption decision. This is a detailed review of selected mechanisms in all ten repositories, **not a complete audit of every skill and dependency**.

| Repository snapshot | `SKILL.md` paths | Root license metadata | Main contribution assessed |
| --- | ---: | --- | --- |
| [microsoft/SkillOpt](https://github.com/microsoft/SkillOpt/tree/79124b37e9a6371e13b753f8bcd7adb1e493ade1) | 5 | MIT | Skill optimization and paired evaluation |
| [affaan-m/ECC](https://github.com/affaan-m/ECC/tree/07756cee15788a54506031462794ad645719b028) | 903 | MIT | Context retrieval, evaluation and skill maintenance |
| [garrytan/gstack](https://github.com/garrytan/gstack/tree/a6b3a57512ca6d5c6aa5b68f74f736195021f96e) | 65 | MIT | Product discovery, code review and browser QA |
| [Leonxlnx/taste-skill](https://github.com/Leonxlnx/taste-skill/tree/e79ca9ec7e071eb3a3b623c4fb752e853fc3ed58) | 13 | MIT | Visual direction and redesign |
| [google/skills](https://github.com/google/skills/tree/18152e0d310e4d7047e9c2ec25a37b0d22d6893e) | 151 | Apache-2.0 | Cloud identity, IAM and SecOps |
| [calesthio/OpenMontage](https://github.com/calesthio/OpenMontage/tree/08e2151fa02de28a5d6a312b3d575692bf147ad7) | 138 | AGPL-3.0; some child skills differ | Scene timing, narration and rendered media |
| [coreyhaines31/marketingskills](https://github.com/coreyhaines31/marketingskills/tree/5b2c0007766c6a1cf1d53fd8fc73e979e0821022) | 50 | MIT | Conversion, product claims and search visibility |
| [ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2) | 864 | Not detected at root | Connected-service contracts and provider workflows |
| [mukul975/Anthropic-Cybersecurity-Skills](https://github.com/mukul975/Anthropic-Cybersecurity-Skills/tree/54a798831d2266a3ca61ce68a7acb80b81160d57) | 818 | Apache-2.0 | Detection rules and MCP audit helpers |
| [zhaoxuya520/reverse-skill](https://github.com/zhaoxuya520/reverse-skill/tree/7e2097fd90d25c2f976f6eba26d6c00aa88051df) | 89 | MIT | Binary/protocol analysis and forensics |

License labels describe repository-level metadata, not permission to redistribute every nested resource. In particular, OpenMontage has mixed licensing and Composio's root metadata does not settle individual licenses. The cybersecurity repository is a third-party project; its name does not establish Anthropic affiliation.

## Decisions by repository

### 1. SkillOpt: retain outcomes, not rhetorical strength

Fénix already had skill authoring and experiment design, but lacked a local checker for paired repeated outcomes. SkillOpt's training/evaluation material makes revision, evaluation and selection explicit. Its [gate implementation](https://github.com/microsoft/SkillOpt/blob/79124b37e9a6371e13b753f8bcd7adb1e493ade1/skillopt/evaluation/gate.py) also contains an optional directive-density bonus; such wording is not evidence of a better task result.

The original Fénix adaptation ties a small edit to an observed failure and compares task/run identities. Missing attempts stay in the denominator; timeouts and errors remain visible. A critical trial cannot be averaged away by a better repetition. Selection and final-test results remain separate. Automatic overnight trace collection, automatic installation and prose-density rewards were excluded.

### 2. ECC: retrieve the evidence needed for the next decision

ECC's [iterative retrieval](https://github.com/affaan-m/ECC/blob/07756cee15788a54506031462794ad645719b028/skills/iterative-retrieval/SKILL.md), stocktake and evaluation procedures help maintain a large library. Fénix already covered task diagnosis, verification and contract design, so duplicating those skills would add little.

`repository-navigation` now tracks unresolved questions, supporting files/lines and remaining gaps, then refines searches using actual repository terminology. `skill-authoring` distinguishes retrieval failures from instruction or tool failures before editing. We did not import fixed search-pass limits, uncalibrated relevance scores, global preference promotion from silence, or file-size estimates presented as actual context consumption.

### 3. gstack: stronger evidence in review and browser QA

The useful additions are operational details: [review](https://github.com/garrytan/gstack/blob/a6b3a57512ca6d5c6aa5b68f74f736195021f96e/review/SKILL.md) checks consumers outside a diff and warns about apparently missing generated fields; browser QA retains reproducible state and issue evidence. Office-hours material distinguishes commercial discovery from building for other goals.

Fénix now checks enum/state consumers, validates generated ORM context, and records URL, identity, fixtures, action and resulting state for browser defects. Product discovery examines the user's actual workaround and what evidence would change the decision. Existing release/completion skills already covered delivery checks. The second orchestration/memory system, mandatory review teams, repeated question gates and synthetic overall scores were not adopted.

### 4. taste-skill: make visual decisions fit the brief

The [main design skill](https://github.com/Leonxlnx/taste-skill/blob/e79ca9ec7e071eb3a3b623c4fb752e853fc3ed58/skills/taste-skill/SKILL.md) supplies useful audience, reference and brand analysis. Fénix already had frontend, brand-token and animation workflows; the gap was a more concrete visual comparison procedure.

A new frontend reference now connects hierarchy, typography, density, assets and motion to the brief, then checks a representative rendered view before propagation. It preserves established design systems and accessibility. Prescribed stack/icon choices, automatic aesthetic changes and the redesign file's suggestions for fabricated organic-looking numbers or dates were excluded. Visual preference is not an objective quality score.

### 5. Google: explicit identities and detection prerequisites

Google's cloud, authentication, IAM and [SecOps](https://github.com/google/skills/blob/18152e0d310e4d7047e9c2ec25a37b0d22d6893e/skills/cloud/secops-detection-engineering/SKILL.md) procedures provide domain detail missing from generic deployment guidance. Fénix now separates project/location, deployer identity and runtime identity; IAM edits preserve unrelated bindings and concurrency tokens.

The new detection workflow separates parser success, backend behavior, historical results, rule creation and alert enablement. It requires real field mappings and benign negative cases. Named upstream MCP tools are not assumed available. A keyword scan is not treated as a security boundary, and absolute alert-count thresholds are not generalized across environments.

### 6. OpenMontage: exact timing with honest media evidence

The reviewed video workflows emphasize measured narration, scene planning and rendered inspection. Its [pacing helper](https://github.com/calesthio/OpenMontage/blob/08e2151fa02de28a5d6a312b3d575692bf147ad7/lib/verify_scene_pacing.py) compares cues with nearby visual landmarks; proximity alone can match the wrong event.

Fénix's independent helper instead requires explicit cue identities and validates rational frame rates, contiguous primary scenes, frame boundaries and paired audio/visual positions. The delivery reference separates timing declarations from actual render/decode, visual and listening checks. Reconstructed terminals must be labeled. No OpenMontage code, Remotion application, Modal dependency or rendering service was imported.

### 7. Marketing skills: measure successful journeys

Fénix already had copywriting, SEO, analytics and A/B analysis. The reviewed [CRO workflow](https://github.com/coreyhaines31/marketingskills/blob/5b2c0007766c6a1cf1d53fd8fc73e979e0821022/skills/cro/SKILL.md) and onboarding material justify a distinct conversion diagnostic: its inputs and output differ from writing copy or analyzing an experiment.

`conversion-optimization` defines population, events, deduplication and window; distinguishes broken instrumentation from abandonment; and uses qualified outcomes and guardrails. Marketing revisions reuse the existing product brief and verify proof claims. SEO revisions record engine/query/date/citation evidence. We excluded promised uplift, obligatory new context files and cancellation friction. Google's primary guidance contradicts universal special-file requirements for AI search; [no special AI file or schema is required there](https://developers.google.com/search/docs/appearance/ai-features).

### 8. Composio: useful connectivity, not hundreds of extra instructions

The snapshot contains 832 provider entrypoints under `composio-skills/`, plus other workflows. Calendar, Drive, Slack, Attio, Search Console, Ashby and Browserbase examples show that provider schemas and identity matter more than the repeated routing boilerplate. Existing Fénix workflows cover most of those business outcomes.

Two concrete corrections matter: the [googlebigquery entry](https://github.com/ComposioHQ/awesome-claude-skills/blob/be2a406907dbc61b73e6827ded415c96139d13a2/composio-skills/googlebigquery-automation/SKILL.md) actually describes Metabase operations; the Search Console entry overstates completeness from pagination. [Google's API](https://developers.google.com/webmaster-tools/v1/searchanalytics/query) explicitly limits returned data to top rows. Fénix now checks actual backends, account ownership, current schemas, partial writes and data coverage.

### 9. Cybersecurity skills: distinguish a rule from demonstrated coverage

The selected Sigma helper parses/converts rules and counts technique mappings; those operations do not establish behavioral detection coverage. The [MCP audit helper](https://github.com/mukul975/Anthropic-Cybersecurity-Skills/blob/54a798831d2266a3ca61ce68a7acb80b81160d57/skills/auditing-mcp-servers-for-tool-poisoning/scripts/agent.py) can launch a package runner in its static mode and start a stdio server for enumeration. Neither is an inert source read.

Fénix's detection skill retains conversion failures and requires positive, benign and boundary fixtures. Source review checks what a scanner actually executes. Agent security review follows untrusted content into memory, retrieval and effectful tool calls, checking authorization where the effect happens. Clean keyword scans and ATT&CK labels are not treated as proof of safety or coverage.

### 10. Reverse-skill: artifact analysis with traceable conclusions

The reviewed binary-diff, protocol and [forensic](https://github.com/zhaoxuya520/reverse-skill/blob/7e2097fd90d25c2f976f6eba26d6c00aa88051df/skills/digital-forensics/SKILL.md) workflows warrant a separate entrypoint: reconstructing a format or explaining a binary differs from source review and incident coordination.

`reverse-engineering` starts with artifact hashes, file format, architecture and static evidence, distinguishes offsets from virtual addresses, and anchors conclusions to functions, bytes or reassembled messages. Dynamic execution depends on an actual authorized environment; a skill cannot create isolation. Incident guidance adds acquisition provenance and clock uncertainty. Tool-specific installers, assumed sandbox access and a competing journal framework were excluded.

## Which Composio integrations would help Fénix?

**Yes, as a connection layer for services the user actually needs.** They do not need hundreds of separate Fénix skills. Reuse an existing suitable connection before adding another provider path. Tools available to the Codex session editing this repository are not automatically available in Fénix.

| Priority and need | Candidate services | Fénix workflow | Concrete contract to verify |
| --- | --- | --- | --- |
| High: everyday correspondence and scheduling | Gmail/Outlook, calendars | email-workflows, calendar-scheduling | Personal/work account, thread/event IDs, timezone, draft/send, notifications |
| High: work coordination | Slack/Teams, GitHub/Jira/Linear | team-messaging, issue-tracker-workflows | Workspace/project/channel, existing item identity, notification and duplicate effects |
| High when knowledge is distributed | Notion/Confluence, Drive and document services | knowledge-workspace-connectors, Google workflows | Search coverage, permissions, document/block IDs, structure preservation |
| Conditional: sales operations | Attio/HubSpot/Salesforce, Airtable | crm-workflows, data-analysis | Tenant, custom field types, associations, list membership, eventual consistency |
| Conditional: website measurement | Search Console/analytics | seo-review, analytics-instrumentation | Exact property, dimensions, window, top-row limits and data latency |
| Later, on a concrete need | Ashby or hosted browser services | hiring-and-interviews or browser workflows | Actual connector availability, record relationships, session identity and cost |

The new [connected-services reference](../../resources/skills/mcp-integration/references/connected-services.md) defines the implementation contract: discover real tools, bind user/tenant/accounts, choose execution transport, resolve provider schemas and verify each effect. Composio's [hosted MCP route](https://docs.composio.dev/docs/sessions-via-mcp) bypasses SDK execution modifiers and does not expose local session-bound custom tools. Authorization and logging must therefore operate on the path that actually executes calls; SDK hooks cannot be assumed to protect remote MCP.

No callable Composio/Rube tools were found in this editing session, and repository searches found no Composio configuration. This iteration adds guidance, not OAuth, a remote MCP client or an activated integration. A live connection needs the chosen service, compatible runtime, account binding and credentials. No external messages or mutations were made during the review.

## Exact changes and checks

The three new skills are [conversion-optimization](../../resources/skills/conversion-optimization/SKILL.md), [detection-engineering](../../resources/skills/detection-engineering/SKILL.md) and [reverse-engineering](../../resources/skills/reverse-engineering/SKILL.md). The evidence lists all nineteen updated parents, exact previous versions and hashes of all 32 changed resources. Calendar received discoverability aliases only; the other eighteen received procedural improvements.

| Updated skill | Added decision or verification |
| --- | --- |
| skill-authoring | Locate the failure before revising; preserve explicit preferences; separate tuning from final evaluation |
| experiment-and-benchmark-design | Pair repeated trials, keep failed/missing attempts and expose critical regressions |
| repository-navigation | Track questions, evidence and gaps; refine queries with repository vocabulary |
| code-review | Follow changed states outside the diff and verify generated symbols |
| frontend-design-and-accessibility | Ground visual choices in the brief and compare actual rendered views |
| browser-regression-testing | Confirm target identity/state and retain reproducible issue evidence |
| product-discovery | Examine the actual workaround and evidence that could reject the proposal |
| marketing-content | Reuse the product brief and substantiate quotes, metrics and claims |
| seo-review | Separate AI mentions, citations and conversions; disclose analytics limits |
| video-editing | Plan measured narration, scene provenance and explicit cue timing |
| cloud-deployment | Select actual project/location and distinguish deployer/runtime identities |
| identity-access-review | Examine effective access and preserve unrelated bindings/concurrency tokens |
| mcp-integration | Bind connected accounts and enforce controls on the actual execution path |
| api-integration | Retain per-step receipts and reconcile partial effects before retrying |
| crm-workflows | Resolve stable IDs, custom schemas and authoritative post-write state |
| incident-response | Preserve acquisition provenance and timestamp uncertainty |
| security-code-review | Follow untrusted input into actual effectful operations |
| skill-source-review | Inspect scanner execution and disclose incomplete dependency review |
| calendar-scheduling | Recognize meeting/time-zone queries through natural aliases |

The two helpers are [compare_outcomes.py](../../resources/skills/experiment-and-benchmark-design/scripts/compare_outcomes.py) and [check_timeline.py](../../resources/skills/video-editing/scripts/check_timeline.py). Both use Python's standard library and only read the supplied JSON. Neither runs an agent, fetches content, installs software, renders video or changes accounts.

| Check | Result | What it establishes |
| --- | --- | --- |
| Skill format, metadata, sections and resource limits | 167/167 | Consistent, loadable packages; not task accuracy |
| Native skill tests | 26 passed | Existing regressions, discovery, packaging, confined resources and activation behavior |
| Upgrade fixtures | 19 exact previous parents | Default upgrades preserve custom text and disabled state |
| Python helper tests | 24 passed | Missing/duplicate outcomes, critical regressions, invalid JSON/numbers, rational timing, gaps/overlaps and explicit cue pairing |
| Agent Clippy, library and tests | Passed with warnings denied | Rust checks for the tested target |

The frozen bilingual discovery corpus has 36 revised-workflow queries, 12 new-workflow queries and 14 neighboring-task guards. On the 50 queries whose expected skills already existed, top-three retrieval improved from **45/50 to 50/50**. The new entries match **12/12** queries; overall the candidate reaches **62/62 top-three and 61/62 first place**. No new entry wins a neighboring-task guard. Existing discovery suites also passed.

These are development cases: the first candidate missed three queries, and natural aliases were revised before the second run. The corpus was not changed, but it was used for tuning, so it is not an independent holdout. Both runs are retained. The helper suite also caught a real initial defect: averaging repeated trials hid a critical regression; the gate now checks each critical pair.

No live-model task success, latency or token-cost gain was measured. SIEM behavior, unknown-binary execution, live Composio connections and media perception remain untested. Those limits prevent a packaging or retrieval result from becoming an unsupported intelligence claim.

## Delivery

Cargo bundles the resources into the next Fénix build. Startup installs new skills and upgrades only recognized unchanged historical bodies; customized entries and activation state remain intact. Existing user references retain the runtime's preservation behavior. The running service was not restarted or deployed. The local catalog database and personal Codex skill installation were not modified.
