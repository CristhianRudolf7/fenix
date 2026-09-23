# Four-repository skill review and Fénix implementation

> Historical 156-skill review. The current catalog has 167 entries after the [ten-repository review](ten-repository-skills-review.md); measurements below describe the earlier revision.

[Español](../es/repository-skills-comparison.md) · [50-entry mapping](../skills/repository-expansion-matrix.md) · [Machine-readable evidence](../skills/repository-expansion-review.json)

Reviewed on **2026-09-19**. The corrected request names `Imbad0202/academic-research-skills`, `greensock/gsap-skills`, `earthtojake/text-to-cad` and `huggingface/skills`. VoltAgent is outside this review. This extends the earlier [Superpowers comparison](superpowers-comparison.md).

## Decision and scope

The 145-entry Fénix library already covered general research, frontend work, training, datasets and deployment. It lacked dedicated procedures for systematic screening, claim-to-source auditing, animation lifecycle, CAD/fabrication, robot descriptions, Hub operations and browser inference. **Eleven original skills were added and nine existing skills strengthened: the catalog now has 156 entries.** The work includes six executable helpers, two reusable examples and 21 supporting references across the 20 affected skills.

There is concrete added capability: source/claim bindings can be checked, screening counts recomputed, a dimensioned part generated and measured after export, DXF and URDF invariants inspected, and animation cleanup exercised in a real browser. Wider task success, lower token cost and superiority over the upstream agents have **not** been established by a live-model comparison. A larger catalog is not itself evidence of a better agent.

All 50 `SKILL.md` paths, sizes and metadata were inventoried. The review covered repository READMEs, licenses, architecture, mode registries and selected workflow/reference sections in depth. It did not inspect every line of the 5,274 tracked files or execute the upstream runtimes. No upstream skill text, installers, hooks, viewer runtime or training/deployment scripts were bundled into Fénix. Local checkouts were used as reference material; upstream instructions did not become authority over this task.

## Source snapshots and portability

| Repository | Pinned revision | Entries | Root license | Integration implication |
| --- | --- | ---: | --- | --- |
| [academic-research-skills](https://github.com/Imbad0202/academic-research-skills/tree/1515a2192a6051f9a793cfd964bd05e3db20607c) | `1515a2192a605` | 4 | CC BY-NC 4.0 | Four large entrypoints, 27 named modes, host-specific orchestration. Use original procedures and existing Fénix plans. |
| [gsap-skills](https://github.com/greensock/gsap-skills/tree/aed9cfd3277740755f6bfc1155c7aa645403b760) | `aed9cfd327774` | 8 | MIT | Strong specialized guidance, but individual claims still need verification. Library licensing is separate from skill-text licensing. |
| [text-to-cad](https://github.com/earthtojake/text-to-cad/tree/4ebbd2cc6172e4c5d53560a5948df214192aefac) | `4ebbd2cc6172e` | 12 | MIT | Eleven product skills plus an internal viewer theme document. Much of the capability depends on Python/JS runtime code. |
| [huggingface/skills](https://github.com/huggingface/skills/tree/abc20ae526d8b4c0e4dff89f904adce28a4a0eb6) | `abc20ae526d8b` | 26 | Apache 2.0 | Twenty-five main skills plus `hf-mcp`; several require remote credentials, services, packages or particular host tools. |

Fénix limits a loaded entrypoint to 32,000 UTF-8 bytes. All four academic entrypoints exceed that bound (37,392–52,683 bytes), as do HF's `hf-cli` (33,105) and `huggingface-lora-space-builder` (33,828). Some upstream metadata also differs from Fénix's string-valued convention. The internal `smui` document has no skill frontmatter. Directly copying these directories would not produce a uniformly usable Fénix library.

The academic repository's noncommercial license is another reason not to vendor its text. HF's root has an Apache license, while some entrypoint headers refer to a `LICENSE.txt` not present in this snapshot; that inconsistency is recorded without inferring additional rights. Our implementation uses independently written instructions and code with linked sources. This is an integration decision, not a legal determination about every upstream dependency.

## Academic research: stronger evidence handling

The upstream organization has four entrypoints: `deep-research`, `academic-paper`, `academic-paper-reviewer` and `academic-pipeline`. The mode registry splits them into 8 research, 11 writing, 6 reviewing and 2 orchestration/resume modes. Useful distinctions include exploratory versus systematic work, source verification versus claim support, manuscript drafting versus review response, and experiment provenance versus scientific credibility.

Fénix already had `literature-review`, `scientific-writing`, `peer-review`, `bibliography-management` and `experiment-and-benchmark-design`. Creating another skill for every upstream mode would duplicate those destinations. The implementation instead adds:

- [systematic-review](../../resources/skills/systematic-review/SKILL.md): protocol and search provenance, deduplication, explicit decisions, missing full text, report/study identity and compatible synthesis. Its helper computes counts from a ledger. It does not decide eligibility, fabricate independent reviewers or establish PRISMA compliance. [PRISMA 2020](https://www.prisma-statement.org/prisma-2020) remains reporting guidance, not a certificate conferred by matching totals.
- [research-evidence-audit](../../resources/skills/research-evidence-audit/SKILL.md): exact manuscript/source hashes, line locators, quoted passages and reasoned statuses for supported, contradicted, overstated or unresolved claims. The helper checks binding integrity. It deliberately reports that semantic support and source authenticity are unverified by the tool.

The existing research skills now route to those procedures when needed. Writing has a research-to-paper lifecycle and a revision trail. Peer review separates criteria, evidence, severity and response disposition. Bibliography management distinguishes correct metadata from evidence that supports a sentence. Experiment design records the actual framework, task, split, prompts, scoring and artifact revisions.

Fixed multi-agent team sizes, upstream host hooks and repeated oversight approvals were not adopted. They require runtime support or impose costs without measured benefit for every task. Native plans, task history and permissions remain the coordination mechanism. We did not reproduce the upstream multi-agent pipeline or run a systematic review against a real research corpus as part of this implementation.

## GSAP: one focused animation workflow

The eight upstream entries cover core APIs, frameworks, React, performance, plugins, ScrollTrigger, timelines and utilities. Fénix's frontend skill covered layout and accessibility but lacked animation ownership, lifecycle and scroll measurement procedures. [web-animation](../../resources/skills/web-animation/SKILL.md) now owns that workflow; framework and plugin specifics load from two references. The frontend skill routes there when motion is material to the task.

The implementation covers scoped targets, timeline ownership, browser-only setup, teardown, responsive/reduced-motion changes, pinning order, refresh versus update, measured rendering cost and task-specific plugin selection. The reusable `scoped-reveal.mjs` takes an existing GSAP instance and component root, preserves readable static content and returns idempotent cleanup.

Two upstream claims warranted correction:

1. The pinned ScrollTrigger entry says lower `refreshPriority` refreshes first. The [official ordering documentation](https://gsap.com/docs/v3/Plugins/ScrollTrigger/static.sort()/) specifies the opposite: higher values come first. A browser check using real GSAP 3.15.0 confirms the order used in our reference.
2. The React entry suggests `contextSafe` makes callbacks inert after unmount. Its purpose is to associate newly created animations with the context; listener removal and stale asynchronous work still need explicit handling. Our reference follows [GSAP's React lifecycle guidance](https://gsap.com/resources/React/).

Eleven Chromium checks exercise scoping, final visibility, style restoration, repeat mounts, component independence, live and initial reduced motion, parameter rejection, priority order and a no-JavaScript baseline. They validate the bundled example and the priority claim, not every framework, plugin or application animation. No GSAP dependency was added to the Fénix product; the test dependency was isolated outside the repository.

## CAD: geometry tools with explicit runtime boundaries

This repository is more than a collection of prompts. Its `cadgen` package, viewer, export utilities and integrations supply much of its functionality. Copying its skill text while omitting those programs would leave broken instructions.

| Upstream functionality | Fénix implementation | Remaining boundary |
| --- | --- | --- |
| `cad`, `step-parts`, `cad-viewer` | [parametric-cad](../../resources/skills/parametric-cad/SKILL.md), an editable build123d plate, saved STEP/mesh inspection and component provenance | No cloned viewer/server, automatic catalog downloader or universal part generator. Kernel and viewer availability must be checked. |
| `dxf`, `sendcutsend` | [cad-drawing-and-fabrication](../../resources/skills/cad-drawing-and-fabrication/SKILL.md), units/layers/entities/bounds checks and process handoff | No supplier ordering, bend solver or comprehensive cut-profile certification. |
| `urdf`, `srdf`, `sdf` | [robot-description](../../resources/skills/robot-description/SKILL.md), URDF graph/limits/mimic/inertia checks and format-specific guidance | SRDF planning semantics and SDF simulation require their native tools; meshes and physics are not certified. |
| `dfam-check`, `gcode`, `bambu-labs` | [3d-printing-preparation](../../resources/skills/3d-printing-preparation/SKILL.md), mesh prerequisites, real slicer profiles and machine/job handoff | No universal G-code interpreter, physical printer driver, wall-thickness certification or actual print. |
| Internal `smui` theme | Existing frontend/design-token capabilities already handle local UI conventions | Upstream viewer-specific styling was excluded. |

The geometry test constructs a 40 × 30 × 4 mm plate with two 5 mm through-holes, exports STEP and STL, reopens both and compares dimensions and volume against the independent analytic expression `40*30*4 - 2*pi*(2.5**2)*4`. Other checks reject open meshes and wrong expected units/dimensions. DXF tests use saved drawings and malformed input. URDF tests cover invalid topology, limits, axes, cycles and nonfinite or nonpositive inertia calculations.

A valid solid or positive-definite tensor is only one check. Physical consistency, principal-moment constraints, collision geometry and simulation behavior remain separate. [MoveIt's URDF/SRDF guidance](https://moveit.picknik.ai/main/doc/examples/urdf_srdf/urdf_srdf_tutorial.html) explains the distinction between geometric descriptions and planning semantics. The local helper does not replace MoveIt or Gazebo.

## Hugging Face: integrate by task, not by vendor entrypoint

Four new destinations cover gaps: [huggingface-hub](../../resources/skills/huggingface-hub/SKILL.md), [model-selection](../../resources/skills/model-selection/SKILL.md), [ml-demo-apps](../../resources/skills/ml-demo-apps/SKILL.md) and [browser-ml-inference](../../resources/skills/browser-ml-inference/SKILL.md). Training, dataset preparation, deployment and evaluation stay in their existing Fénix skills, with added references.

| Source group | Useful addition | Adaptation and limitation |
| --- | --- | --- |
| CLI, datasets, papers/publishing, tool builder, MCP | Repository type and revision, selected files, config/split pagination, paper/artifact identity and job IDs | Use installed CLI/SDK/native discovery; no assumed `hf_jobs` tool or automatic session upload. Public read endpoints were exercised; authenticated writes were not. |
| Best model, `hf-mem`, local models | Task-specific comparison, exact GGUF artifact, weights/KV/overhead separation | Our estimator uses explicit inputs. It does not reproduce upstream HTTP Range/header inspection or all attention/cache architectures. A weight estimate is not proof of successful inference. |
| LLM trainer, TRL, vision trainer, sentence transformers, Trackio | Objective and label formats, negatives, image transforms, saved processors/adapters, held-out metrics and tracking visibility | No forced remote compute or mandatory Hub publication. No model was trained or downloaded for this review. |
| Gradio, Spaces, LoRA builder, ZeroGPU | Real inference outputs, session isolation, queue/cancellation, base/adapter compatibility and GPU boundaries | Hosting remains external. [ZeroGPU requirements](https://huggingface.co/docs/hub/spaces-zerogpu) and quotas must be checked when used; no fixed hardware promise is embedded. |
| Six AWS/SageMaker skills | Account/region, interpreter, role/quota checks, image compatibility, serving pathway and resource cleanup | Fold into deployment/environment/access workflows. No unconditional provider-image preference, assumed account or copied cloud deployment runtime. |
| Community evals | Versioned tasks, prompts, metric direction, independent baselines and output artifacts | No dependency on an absent `hugging-face-jobs` entry or a developer's `~/code/community-evals` checkout. |
| Transformers.js | Model/processor/ONNX compatibility, workers, WebGPU/WASM fallbacks, download/cache behavior and disposal | Instructions are implemented; actual browser model execution was not tested. Check the [current runtime documentation](https://huggingface.co/docs/transformers.js/en/index) for the selected version. |

Important departures are concrete. One upstream recommendation path estimates weights from parameter-size heuristics without establishing full runtime fit; our guide requires actual metadata and separates cache and overhead. Upstream `hf-mem` itself already has more sophisticated header inspection and experimental KV support, so our small estimator is not advertised as its full replacement. The trainer's immediate MCP submission/publication instructions are not appropriate when the user only authorized local preparation. Raw agent session folders are not an implicit dataset upload scope. Hosted tracking visibility is an explicit choice.

Three bounded unauthenticated requests succeeded: public model metadata with a resolved commit, dataset split discovery, and two rows from a named dataset/config/split. They confirm those request shapes and responses, not end-to-end Fénix model behavior or remote training/deployment. [Dataset Viewer rows](https://huggingface.co/docs/dataset-viewer/en/rows) are treated as a paginated preview, not a downloaded corpus.

## Selection, migration and executable evidence

The [65-case corpus](../skills/repository-expansion-cases.json) contains 44 bilingual new-capability prompts, nine prompts for revised existing skills and 12 neighboring-task checks. It was written after skill authoring and fixed before the first native run; it is a development regression set, not a held-out benchmark.

The baseline disables the eleven additions and restores exact snapshots of the nine revised parents. It is a controlled catalog comparison inside the current runtime, not a time-travel run of an old complete application build.

| Query group | Baseline expected top 1 / top 3 | Candidate expected top 1 / top 3 |
| --- | ---: | ---: |
| 44 new-capability cases | Unavailable by construction | 44 / 44 |
| 9 existing-skill cases | 6 / 9 | 9 / 9 |
| 12 neighboring cases | 11 / 11 | 11 / 11 |

No new skill becomes the first result for a neighboring case. The query “Repair a regression in this Rust parser.” misses `test-driven-repair` in the top three in both phases; `software-implementation` remains first. After the first paired run exposed this baseline miss, the guard assertion was corrected to preserve the old winner in such cases. The miss remains in the published metrics: candidate expected retrieval is **64/65**, not 65/65. The unavailable baseline entries cannot support a claim of a measured 0-to-100% task success improvement.

The final checks passed:

- **20 native Rust skill tests**, including existing regressions, the paired corpus, resource loading and upgrade behavior; Clippy with warnings denied also passed.
- **22 Python helper tests**, including real kernel/mesh/drawing operations and independent numeric expectations.
- **11 real-browser animation checks**, GSAP 3.15.0 and Chromium 153.0.8010.47.
- **3 bounded public Hub reads**, without credentials, writes, jobs or model downloads.

The first native attempt encountered a shared Cargo cache with an old generated 145-entry catalog. Re-running the build script regenerated the 156-entry bundle before the recorded successful run. The record distinguishes this build issue from the later baseline routing miss. Full application tests and live-LLM A/B evaluation were outside this skill change.

Nine original parent files are retained as migration fixtures, with their exact hashes registered. Tests confirm that known originals upgrade while custom text and disabled state survive. Helpers and references load through the native skill resource path; disabling a skill also prevents its resources from loading. No new global policy, permission grant or mandatory orchestration pipeline was introduced.

All 156 entrypoints passed format validation; the largest is 6,539 bytes. Resource file/bundle limits, 49 affected artifact hashes, nine migration hashes, both catalog memberships and 517 local documentation links also passed checks. These checks establish consistency of the delivered library, not semantic quality of every instruction.

To reproduce the executable checks from the repository root:

```bash
cargo test -p fenix-agent --lib skills:: --locked -- --nocapture
cargo clippy -p fenix-agent --lib --tests --locked -- -D warnings
PYTHONDONTWRITEBYTECODE=1 python3 tests/skills/test_repository_helpers.py
node tests/skills/test_web_animation.mjs /path/to/isolated/node_modules/gsap
```

The Python run used Python 3.13.5, build123d 0.11.1, trimesh 5.0.0, NumPy 2.5.2 and ezdxf 1.4.4. The browser test uses the workspace's Playwright package, `/usr/bin/chromium` and a separately installed GSAP 3.15.0 directory. The evidence manifest records package versions, test hashes, build-profile variables, individual helper/browser cases and the exact bounded public API requests. Remote responses may change; their recorded results describe this run.

## Cost, activation and further evidence

The eleven new entrypoints total **36,347 UTF-8 bytes**. The nine revised parents grew from **16,576 to 19,537 bytes**. Their combined supporting resources total **64,927 bytes**. These are artifact sizes, not tokenizer counts or per-turn costs. References and helpers are loaded on demand. External packages remain prerequisites rather than hidden installs.

The code and embedded resource catalog are ready for the next build/start of Fénix. This task did not deploy or restart a running service. It did not modify the separate Codex skill installation. A customized parent remains customized and may need a deliberate merge to receive new guidance.

The additions are sufficient for the specific identified workflow gaps at the documented evidence level. Further entrypoints should follow task traces showing a distinct procedure and retrieval need. The next useful validation is a held-out, representative end-to-end task set with artifact scoring, actual resource/tool availability and cost/latency measurement. Full CAD viewer parity, printer integration, automatic model-header analysis and hosted ML execution would be separate engineering projects, not capabilities obtained by adding more Markdown files.
