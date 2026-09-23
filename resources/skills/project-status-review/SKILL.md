---
name: project-status-review
description: Summarize project progress and open commitments from task evidence without mixing contexts or creating
  duplicate work.
metadata:
  version: 1.0.0
  category: coordination
  tags: project, status, review
  aliases: estado proyecto, pendientes, revisión semanal, progreso, bloqueado
  required_tools: project_manage,task_inspect,history_search,memory_search
  requires: Project/task records and the requested time window
  effects: read-only
---
# Project Status Review

## Scope

Use for project or weekly status. Use conversation-recap for chat decisions and requirements-and-planning for changing the plan.

## Inputs

Project identity, period, audience and requested depth.

## Workflow

1. Resolve project names and aliases and keep global preferences separate from project facts. Ask if the target is ambiguous.
2. Read active plans, task state and recent evidence. Distinguish completed, in progress, blocked, cancelled and waiting for user input.
3. Reconcile stale notes against actual task events. A generated status message alone is not proof of an artifact or successful deployment.
4. Identify the next meaningful step, dependencies and aging commitments. Do not create a second task for work already queued or running.
5. Report a compact status with evidence links, unresolved decisions and relevant changes since the prior review.

## Verification

Use the same reporting window and timezone throughout. Note inaccessible projects and avoid implying a portfolio-wide review from one project.

## Limits

A status request does not authorize cancellations, rescheduling or external follow-ups. Do not copy mutable progress into durable memory as an immutable fact.

## Español

Resuelve el proyecto y consulta tareas reales. Separa estados y evidencia, detecta trabajo pendiente sin duplicarlo y evita cruzar contextos.
