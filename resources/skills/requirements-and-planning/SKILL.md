---
name: requirements-and-planning
description: Clarify broad requests, detect ambiguous or conflicting requirements, and turn agreed outcomes into editable milestones without inventing requirements.
metadata:
  version: 1.3.0
  category: coordination
  tags: requirements, and, planning
  aliases: planificación, requisitos, objetivo general, hitos, plan de trabajo, requisitos ambiguos, contradicciones de especificación, specification consistency
  required_tools: plan_manage,ask_user
  requires: User objective and available project context
  effects: local-write
---
# Requirements And Planning

## Scope

Use for work that benefits from milestones. A small direct task may need only a concise intended outcome.

## Inputs

Original user request, existing decisions, constraints, current project and completion evidence.

## Workflow

1. Restate the requested outcome from the original messages. Separate explicit requirements, inferred assumptions and implementation choices.
2. Expose doubts and ask focused questions where the answer changes scope or correctness. Continue independent work while a dependent decision is pending.
3. Decompose into deliverables with dependencies and observable completion evidence. Keep project IDs and constraints attached to each item. For dependent code changes, name the producing and consuming interfaces, owned files and required inputs/outputs; check adjacent milestones agree on exact signatures or formats. Include a representative acceptance check and relevant failure case without prewriting the entire implementation.
4. Use the runtime plan tools to record the current plan rather than creating a competing task database. Review risky or ambiguous milestones proportionately.
5. When the user redirects work, inspect completed and in-flight effects before changing remaining steps. Preserve history and make superseded requirements visible.

Scale design work to the uncertainty: a clear bounded edit can proceed directly; a new boundary needs alternatives and explicit tradeoffs; an experiment needs a question and a success criterion. Reuse the user's supplied intent and authorization. Do not impose a second approval ceremony for an already authorized design or require a worktree, plan file or commit for every change. Recover completed milestones and their evidence before resuming after compaction.

For a substantial plan review, apply only the relevant perspectives: user value, interaction, developer-facing integration and engineering feasibility. Preserve accepted requirements with their verification and record why any are revised or deferred. After changing a premise or interface, revisit dependent estimates, tests and delivery steps; a review of an earlier plan does not validate the amended one. End with the current implementable plan and unresolved decisions, not only commentary about the plan. Distinguish a proposed behavior from something already present in source.

## Verification

Each milestone must serve the user outcome and have a feasible check. Surface blockers and assumptions rather than silently expanding scope.

For software work spanning several requirements, contracts or delivery stages, load [specification consistency](references/specification-consistency.md). Check requirement quality and agreement between the current specification, interfaces and planned tasks before implementation. Keep findings in the existing plan/review; a clean checklist is not execution evidence. A small clear edit does not need this expanded review.

## Limits

Do not require another agent to approve every trivial step. A plan entry marked complete needs actual evidence, not only a generated summary.

## Español

Aclara el objetivo sin inventar requisitos. Registra supuestos, dudas, dependencias y comprobaciones; permite modificar el plan conservando lo ya ejecutado.

En hitos dependientes, comprueba interfaces productoras y consumidoras, archivos y criterios de aceptación. Ajusta el diseño a la incertidumbre y conserva las autorizaciones existentes; recupera evidencia antes de repetir trabajo tras una compactación.

En revisiones amplias, conserva requisitos aceptados y revisa las dependencias afectadas por cada cambio. Entrega el plan actualizado; una revisión anterior no valida automáticamente la nueva versión.

Para software con varios requisitos o contratos, consulta la referencia de coherencia: detecta ambigüedades, contradicciones y tareas sin relación con el objetivo antes de implementar. Usa los registros existentes y evita este proceso ampliado para cambios sencillos. Revisar requisitos no demuestra que el programa funcione.
