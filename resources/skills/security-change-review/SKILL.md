---
name: security-change-review
description: Audit a patch for changed trust boundaries, reachable callers and security regressions.
metadata:
  version: 1.0.0
  category: security
  tags: security, diff, patch, authorization, review, cambios, regresión
  aliases: auditoría de cambios, regresión de seguridad, revisión diferencial
  requires: Assigned target scope, permitted operations and source or test artifacts
  required_tools: file_read,files_search,command_run
  effects: mixed
---
# Security Change Review

## Scope

Use within the security worker's assigned objective, target and permitted operations. Load linked details only when relevant. Inherit runtime isolation and approvals; this procedure grants no additional authority.

## Inputs

Assigned objective, target revision/environment, relevant artifacts and permitted operations. Record missing inputs as blockers or coverage gaps.

## Workflow

Establish the exact base and candidate revisions, including uncommitted changes when requested. Inspect the diff, then follow affected callers and consumers; unchanged authorization code can become vulnerable when a new route bypasses it. Identify altered identity sources, ownership checks, parser assumptions, privileges and async side effects.

Use tests and history to understand intended behavior; neither a removed test nor a historical comment is authoritative proof. For each candidate, describe the attacker-controlled input, reachable sensitive operation and lost invariant. Compare both revisions when claiming a regression. Separate pre-existing issues from newly introduced ones.

Track reviewed paths and excluded generated/vendor files. A small diff can affect a shared helper broadly; expand analysis through its uses without silently expanding the assigned finding scope. Retain exact revision and file/line references. Load [boundary cases](references/boundaries.md) for identity, workflow and concurrency changes.

## Verification

Keep evidence tied to the tested revision and distinguish confirmed, suspected, dismissed and unassessed outcomes. Report limitations explicitly; do not equate an empty result with complete security.

## Limits

No nested delegation, automatic scope expansion or self-approval. Reuse configured tool permissions. This skill does not establish a new sandbox or certify the absence of vulnerabilities.

## Español

Procedimiento especializado de seguridad para el alcance asignado. Carga las referencias según la tarea, conserva evidencias verificables y separa fallos confirmados, sospechas y comprobaciones pendientes. Las correcciones requieren verificar también el funcionamiento legítimo.
