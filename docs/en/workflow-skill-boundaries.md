# Three distinct workflow skills

> Historical 145-skill review. The current catalog has 167 entries after the [ten-repository review](ten-repository-skills-review.md); measurements below describe the earlier revision.

[Español](../es/workflow-skill-boundaries.md) · [Initial comparison](superpowers-comparison.md) · [Recorded checks](../skills/workflow-boundaries-review.json)

## Revised decision

The library now has **145 skills**. The earlier 142-skill review established broad coverage and improved retrieval, but it did not establish an optimal count. On reassessing the boundaries, three workflows warrant their own discoverable entrypoints. The decision is based on different inputs, outputs and permitted actions, not a target number of skills.

| New skill | Concrete distinction | Previous location |
| --- | --- | --- |
| [agent-workflow-diagnosis](../../resources/skills/agent-workflow-diagnosis/SKILL.md) | Reads execution traces to explain repeated actions, missed skills and time/token usage. Its deliverable is an evidence-backed diagnosis. | Optional reference under local-troubleshooting, which also executes local repairs. |
| [completion-verification](../../resources/skills/completion-verification/SKILL.md) | Checks whether current delivered artifacts satisfy requirements, including documents and data. Its deliverable is a supported acceptance verdict. | Optional reference under software-implementation, whose primary trigger is creating software. |
| [review-feedback](../../resources/skills/review-feedback/SKILL.md) | Starts with received findings and checks or implements valid corrections according to the request. | Optional reference under code-review, whose declared purpose and effects were read-only review. |

For example, a request to check delivered spreadsheets should not need to select software implementation first. A request to apply valid PR corrections should not depend on a skill declaring read-only effects. A request to explain repeated agent actions should begin with trace analysis rather than service-repair commands. These are observable mismatches in the previous organization; they do not prove that a model actually made those mistakes.

## Integration

The three existing parent entrypoints now state narrower responsibilities and route to the specialist when appropriate. Ordinary implementation, local repairs and independent code review keep their original workflow. The new skills are self-contained and use existing native tools. Their effects metadata describes expected actions; runtime permissions still enforce actual access.

The older references remain available for customized or older installed skills that still refer to them. Current parent entrypoints route to the new canonical skills. Keeping those compatibility resources avoids breaking existing user text; it does not require loading both versions.

The parent skills' exact preceding hashes are registered for migration, with their bytes retained in `tests/fixtures/skills/workflow-v2/`. The installer updates known originals while preserving customized text and enabled state. The three new folders are bundled by the existing build script. No additional coordinator, hook, permission policy or provider was added. Updated resources become available when the updated binary starts; this change does not restart or deploy a live service.

## Validation approach

The [24 boundary cases](../skills/workflow-boundary-cases.json) were written before the new skill bodies: 18 requests for the three outcomes and six neighboring requests that should retain their existing procedure. The paired runtime test disables the new skills and installs exact previous parent bodies for the baseline, then upgrades the parents and enables the new skills for the candidate. All other embedded skills and the retrieval algorithm stay fixed within the test.

Baseline targets are the former parent skills; candidate targets are the new specialists. Consequently this compares access to the relevant procedure under two organizations, not identical skill identities. The negative cases check that the new procedures do not enter the first two suggestions for those six neighboring tasks. They are examples, not universal guarantees against false triggers.

The original retrieval corpus and the 18 earlier workflow cases remain regression checks. Two targets in the historical corpus now map to the corresponding new specialists; the historical JSON and recorded results remain unchanged. The suite also checks installation, resource loading, known-original upgrades, custom-content preservation and disabled state. It does not infer model behavior from words appearing in Markdown.

The final run passed **15/15 native skill tests**. All **145 entrypoints** passed format validation; their declared tools exist in the native runtime. Clippy passed for the library and test targets with warnings denied. The paired retrieval results were:

| Query group | Previous organization: top 1 / top 3 | Current organization: top 1 / top 3 |
| --- | ---: | ---: |
| 18 new workflow requests | 7/18 · 9/18 | 18/18 · 18/18 |
| 6 neighboring tasks | 3/6 · 5/6 | 5/6 · 6/6 |
| 18 historical workflow requests | 10/18 · 11/18 | 18/18 · 18/18 |

None of the new specialists appeared in the first two suggestions for the six neighboring tasks. The first candidate exposed a pre-existing miss for a Spanish CSV-parser request. Adding implementation aliases fixed that case; it is a development correction on a known query, not held-out evidence. Migration checks passed for the exact previous parents, customized content and disabled entries.

Validation used an isolated source snapshot because concurrent server-manifest edits required a lockfile update. Only the copy's lockfile received those package-dependency entries; dependency versions were preserved and this task did not change the workspace lockfile. Compilation also exposed an incompatible `renameat2` call in the concurrent skill-bundle importer. Passing its existing directory handles fixed compilation in both copies. The tested skill entrypoints, fixtures, migration registry and `skills.rs` matched the workspace. This validates the skill subsystem, not the entire concurrently modified application.

Exact results are preserved in [the check record](../skills/workflow-boundaries-review.json). These are development and integration checks, not held-out model evaluations. No live-model quality, cost, latency or token improvement is asserted. The new organization adds instructions and search candidates, so it has a context/selection cost as well as a plausible benefit.

## Why stop at these three for now?

Root-cause investigation remains a mode of test-driven-repair: the input is the observed defect and the output is a verified repair. Worktree setup and branch handoff share repository state, ownership and integration checks. Additional splits there would currently duplicate routing without an equally clear change of outcome. This is a revisable judgment, not a permanent cap of 145.

Future additions should follow task evidence: a recurring outcome with different prerequisites or verification, repeated misrouting, or a procedure that cannot be applied reliably through its current parent. General claims that the agent works better still require isolated, repeated model trials with independently scored results, as described in [behavioral evaluation](../../resources/skills/skill-authoring/references/behavioral-evaluation.md).
