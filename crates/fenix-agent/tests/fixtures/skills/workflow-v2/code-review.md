---
name: code-review
description: Review code diffs or evaluate reviewer feedback and suggested fixes against requirements and repository evidence. Revisar código y comentarios del revisor.
metadata:
  version: 1.1.0
  category: engineering
  tags: code, review
  aliases: revisión de código, revisar PR, revisar cambios, code review, reviewer feedback, comentarios del revisor, sugerencias
  required_tools: file_read,files_search,command_run
  requires: Repository and explicit change scope
  effects: read-only
---
# Code Review

## Scope

Use to review a diff or evaluate received feedback. Review-only requests remain read-only. Use security-code-review when a focused security assessment is requested. Load [review feedback](references/review-feedback.md) when deciding whether or how to apply another reviewer's suggestions; implementation requires the user's existing or explicit authorization and appropriate tools.

## Inputs

Base/head or working-tree scope, requirements, changed files and relevant validation results.

## Workflow

1. Resolve the intended base and head and include uncommitted/untracked changes when in scope. Record the comparison used; an empty committed diff may omit current work.
2. Read changed code together with callers and tests. Evaluate user-visible correctness separately from conventions and maintainability.
3. Focus on introduced defects with a concrete trigger and effect. Trace data/control flow to establish impact; avoid speculative findings without a reachable path.
4. Run a targeted reproduction when useful. Report severity, file location, triggering conditions and a practical correction; separate suggestions from defects.
5. If another reviewer is used by the coordinator, provide only the requirements, exact diff and relevant evidence. Resolve disagreement with code/tests, not a vote.

Separate requirement coverage from defect analysis: green tests can miss a requested behavior, and a defect may affect a real supported input absent from the examples. Explain that impact without turning a preference or imagined feature into a requirement. On re-review, check the specific fix and regressions it could introduce; expand only for new evidence.

## Verification

Recheck each finding against current files and distinguish existing defects from new regressions. State review coverage and unexecuted checks.

## Limits

Do not edit files during a review-only request or claim absence of all bugs. Simple changes do not require mandatory additional agents.

## Español

Revisa un diff concreto y sus llamadas. Comunica defectos demostrables con ubicación e impacto; separa mejoras de estilo y resuelve desacuerdos mediante evidencia.

Ante comentarios del revisor, verifica cada sugerencia con el código actual y carga la referencia de respuesta. Revisar no autoriza editar. Separa cumplimiento de requisitos, defectos y preferencias; acota la segunda revisión a las correcciones y sus regresiones.
