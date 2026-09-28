---
name: code-review
description: Review a concrete code diff against requirements and repository conventions, reporting actionable defects with evidence. Revisar diferencias de código sin modificar archivos.
metadata:
  version: 1.3.0
  category: engineering
  tags: code, review
  aliases: revisión de código, revisar PR, revisar cambios, code review, revisar diff, defectos, enum consumers, generated ORM fields, campos generados, cobertura de consumidores
  required_tools: file_read,files_search,command_run
  requires: Repository and explicit change scope
  effects: read-only
---
# Code Review

## Scope

Use to produce an independent review of a concrete diff. Review-only requests remain read-only. Use security-code-review when a focused security assessment is requested. For findings already supplied by another reviewer, discover and load review-feedback; that separate workflow distinguishes assessment from authorized corrections.

## Inputs

Base/head or working-tree scope, requirements, changed files and relevant validation results.

## Workflow

1. Resolve the intended base and head and include uncommitted/untracked changes when in scope. Record the comparison used; an empty committed diff may omit current work.
2. Read changed code together with callers and tests. Evaluate user-visible correctness separately from conventions and maintainability.
3. Focus on introduced defects with a concrete trigger and effect. Trace data/control flow to establish impact; avoid speculative findings without a reachable path.
4. Run a targeted reproduction when useful. Report severity, file location, triggering conditions and a practical correction; separate suggestions from defects.
5. If another reviewer is used by the coordinator, provide only the requirements, exact diff and relevant evidence. Resolve disagreement with code/tests, not a vote.

Separate requirement coverage from defect analysis: green tests can miss a requested behavior, and a defect may affect a real supported input absent from the examples. Explain that impact without turning a preference or imagined feature into a requirement. On re-review, check the specific fix and regressions it could introduce; expand only for new evidence.

For new enum values, states or schema fields, search consumers outside the diff: exhaustive matches, serialization, migrations, permissions, jobs and UI labels. Verify apparently missing symbols against generated code, framework metadata and actual dependency versions before reporting them. A search miss is an investigation lead. Describe observed evidence and unresolved preconditions rather than assigning invented confidence probabilities. A previously accepted risk can be reconsidered when its assumptions change.

## Verification

Recheck each finding against current files and distinguish existing defects from new regressions. State review coverage and unexecuted checks.

## Limits

Do not edit files during a review-only request or claim absence of all bugs. Simple changes do not require mandatory additional agents.
