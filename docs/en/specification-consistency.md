# Specification consistency in Fénix

[Español](../es/specification-consistency.md)

Fénix adapts selected ideas from [GitHub Spec Kit](https://github.com/github/spec-kit/tree/d4229c071c7ea3885b43e8a7739847300f618f13) as original instructions in three existing skills. No upstream CLI, workflow engine or prompts are installed. The changes target software with several requirements or shared contracts; small clear edits keep their direct workflow.

| Existing skill | Improvement |
| --- | --- |
| [Requirements and planning](../../resources/skills/requirements-and-planning/SKILL.md) | Review requirement quality and consistency across acceptance criteria, interfaces and milestones; separate measurable delivery outcomes from later business metrics. |
| [Software implementation](../../resources/skills/software-implementation/SKILL.md) | Carry source references and acceptance examples through implementation; preserve validation/error contracts and inspect coverage separately from passing tests. |
| [Completion verification](../../resources/skills/completion-verification/SKILL.md) | Distinguish observed gaps from unavailable evidence and attach bounded, deduplicated corrective work to the existing plan. |

The references load only for relevant tasks. Existing project contracts (`project_spec`), versioned plans and evidence tools remain authoritative. Proposed amendments are distinguished from applied changes; stable requirement IDs, user intent and valid completed work are preserved. Specification files are optional handoffs with a source/version, not another mandatory task database.

The source ideas are requirement-writing checks in [`checklist`](https://github.com/github/spec-kit/blob/d4229c071c7ea3885b43e8a7739847300f618f13/templates/commands/checklist.md), cross-artifact analysis in [`analyze`](https://github.com/github/spec-kit/blob/d4229c071c7ea3885b43e8a7739847300f618f13/templates/commands/analyze.md), and implementation reconciliation in [`converge`](https://github.com/github/spec-kit/blob/d4229c071c7ea3885b43e8a7739847300f618f13/templates/commands/converge.md). Fénix does not impose their full phase sequence or a new approval round. No correctness guarantee follows from an agent marking a checklist complete.

Bundled versions advance to planning 1.3.0, implementation 1.5.0 and completion verification 1.1.0. Exact previous stock bodies are eligible for the existing hash-based upgrade on resource installation; customized skills, disabled state and customized references are preserved. New references are bundled by the existing build script. Existing installations need a rebuilt runtime and resource installation/startup to receive these changes.

Validation covers metadata, native discovery, bundled references and stock migration/customization preservation. These checks establish packaging and routing behavior, not a measured improvement in model reasoning or task success. No backend control, production deployment or external service is added by this change.

[Validation record](../research/specification-consistency-validation-2026-09-21.json): 39 native skill tests passed, including the two new migration/discovery cases; three skill entrypoints passed metadata validation.
