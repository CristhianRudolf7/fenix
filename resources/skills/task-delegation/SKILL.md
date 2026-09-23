---
name: task-delegation
description: Coordinate bounded subagent tasks with clear outcomes, isolated project context and evidence-based
  integration.
metadata:
  version: 1.1.0
  category: coordination
  tags: task, delegation
  aliases: subagentes, delegar, tareas paralelas, coordinación, maestro
  required_tools: tool_search
  requires: Coordinator runtime with authorized subagent tools
  effects: mixed
---
# Task Delegation

## Scope

Use when the master has a concrete independent subtask worth delegating. Workers cannot create more subagents; keep one delegation level.

## Inputs

Parent task/project, deliverable, owned files, relevant sources, dependencies and completion criteria.

## Workflow

1. Choose a bounded outcome, not a vague role. Keep sequential or trivial work local when coordination would cost more than it saves.
2. Provide only necessary project context, constraints and evidence references. Workers can search the skill catalog and load relevant skills on demand; do not inject the entire library.
3. Assign non-conflicting write ownership or use existing isolation support. Agree on shared interfaces before dependent implementations start.
4. Keep the master available to the user and monitor meaningful events. Forward corrections with task IDs and inspect current effects before cancellation or replanning.
5. Inspect worker results and integrate them against the original request. Report blockers after the runtime retry policy is exhausted; do not create replacement agents just to reset retries.

For implementation handoffs, include the exact requirement source, owned files, consumed/produced interfaces, validation command or criterion and relevant prior decisions. Return artifact/revision, observed checks, deviations and unresolved concerns. Keep requirements in one authoritative source rather than paraphrasing exact values differently in each brief.

Assess requirement coverage and code quality separately. A plan review validates the plan, not the resulting implementation. Use additional independent review for a concrete uncertainty or risk; ordinary successful changes do not need a fresh agent after every step. Reuse unchanged verification evidence and scope follow-up reviews to the fixes and plausible regressions.

## Verification

Require changed artifacts, checks, assumptions and unresolved issues in the handoff. A worker’s confident completion claim is not evidence by itself.

## Limits

Tool access and spawning remain enforced by the runtime. Do not invent delegation APIs or allow cross-project access through copied context.

## Español

Delega objetivos concretos con contexto mínimo y archivos asignados. Mantén al maestro disponible, un solo nivel de subagentes y verifica entregables con evidencia.

Incluye fuentes exactas, interfaces y criterios de comprobación. Solicita artefactos, resultados, desviaciones y dudas; distingue aprobación del plan, cumplimiento del resultado y calidad del código sin duplicar revisiones rutinarias.
