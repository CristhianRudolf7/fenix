---
name: requirements-and-planning
description: Clarify broad requests and turn agreed outcomes into editable milestones without inventing requirements.
metadata:
  version: 1.0.0
  category: coordination
  tags: requirements, and, planning
  aliases: planificación, requisitos, objetivo general, hitos, plan de trabajo
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
3. Decompose into deliverables with dependencies and observable completion evidence. Keep project IDs and constraints attached to each item.
4. Use the runtime plan tools to record the current plan rather than creating a competing task database. Review risky or ambiguous milestones proportionately.
5. When the user redirects work, inspect completed and in-flight effects before changing remaining steps. Preserve history and make superseded requirements visible.

## Verification

Each milestone must serve the user outcome and have a feasible check. Surface blockers and assumptions rather than silently expanding scope.

## Limits

Do not require another agent to approve every trivial step. A plan entry marked complete needs actual evidence, not only a generated summary.

## Español

Aclara el objetivo sin inventar requisitos. Registra supuestos, dudas, dependencias y comprobaciones; permite modificar el plan conservando lo ya ejecutado.
