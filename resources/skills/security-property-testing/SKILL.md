---
name: security-property-testing
description: Build invariant and state-transition tests for security boundaries, parsers and access policies.
metadata:
  version: 1.0.0
  category: security
  tags: security, property, fuzzing, proptest, state machine, invariant, propiedades
  aliases: invariantes de seguridad, pruebas generativas, secuencias de autorización
  requires: Assigned target scope, permitted operations and source or test artifacts
  required_tools: file_read,files_search,command_run
  effects: mixed
---
# Security Property Testing

## Scope

Use within the security worker's assigned objective, target and permitted operations. Load linked details only when relevant. Inherit runtime isolation and approvals; this procedure grants no additional authority.

## Inputs

Assigned objective, target revision/environment, relevant artifacts and permitted operations. Record missing inputs as blockers or coverage gaps.

## Workflow

Derive the security invariant from the actual requirement. Examples include separation between tenants, denial after revocation, normalized paths staying inside a root, or rejected requests having no protected effects. Include legitimate successful behavior so an implementation that denies everything cannot pass.

Choose example tests for known regressions and property/state-machine tests for large input or sequence spaces. Use the project's existing framework and deterministic seeds. Generate valid baselines as well as malformed inputs; validation failures can hide deeper access-control defects.

Keep fixtures synthetic, isolate state between runs and shrink failures to reproducible cases. Record seed, revision and command. A timeout or crashed harness is not a passing test. Load [stateful testing](references/stateful.md) for sessions, queues or concurrent workflows.

## Verification

Keep evidence tied to the tested revision and distinguish confirmed, suspected, dismissed and unassessed outcomes. Report limitations explicitly; do not equate an empty result with complete security.

## Limits

No nested delegation, automatic scope expansion or self-approval. Reuse configured tool permissions. This skill does not establish a new sandbox or certify the absence of vulnerabilities.

## Español

Procedimiento especializado de seguridad para el alcance asignado. Carga las referencias según la tarea, conserva evidencias verificables y separa fallos confirmados, sospechas y comprobaciones pendientes. Las correcciones requieren verificar también el funcionamiento legítimo.
