---
name: security-fix-validation
description: Validate a security patch against the original failure, related variants and legitimate behavior.
metadata:
  version: 1.0.0
  category: security
  tags: security, patch validation, remediation, regression, fix, corrección
  aliases: validar parche de seguridad, verificar corrección, preservar funcionalidades
  requires: Assigned target scope, permitted operations and source or test artifacts
  required_tools: file_read,files_search,command_run
  effects: mixed
---
# Security Fix Validation

## Scope

Use within the security worker's assigned objective, target and permitted operations. Load linked details only when relevant. Inherit runtime isolation and approvals; this procedure grants no additional authority.

## Inputs

Assigned objective, target revision/environment, relevant artifacts and permitted operations. Record missing inputs as blockers or coverage gaps.

## Workflow

Capture the pre-fix failure and expected legitimate behavior before changing code when feasible. Confirm that the assignment includes correction, then make the smallest change addressing the root cause. Prefer fixing the authoritative boundary over adding one UI guard.

Replay the original case against the candidate version, test relevant adjacent variants and run affected ordinary tests. Verify dependencies and configuration used by those tests. A check that no longer reaches the guarded operation does not demonstrate a repair.

Compare supported capabilities before and after the patch. For a skill, preserve legitimate procedures and resources; text similarity cannot prove behavioral equivalence. Leave uncertain executable changes as proposals with missing validation clearly stated. Load [repair evidence](references/repair.md) before closing a finding.

## Verification

Keep evidence tied to the tested revision and distinguish confirmed, suspected, dismissed and unassessed outcomes. Report limitations explicitly; do not equate an empty result with complete security.

## Limits

No nested delegation, automatic scope expansion or self-approval. Reuse configured tool permissions. This skill does not establish a new sandbox or certify the absence of vulnerabilities.

## Español

Procedimiento especializado de seguridad para el alcance asignado. Carga las referencias según la tarea, conserva evidencias verificables y separa fallos confirmados, sospechas y comprobaciones pendientes. Las correcciones requieren verificar también el funcionamiento legítimo.
