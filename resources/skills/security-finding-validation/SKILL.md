---
name: security-finding-validation
description: Verify suspected vulnerabilities, challenge false positives and find reachable variants.
metadata:
  version: 1.0.0
  category: security
  tags: security, false positive, evidence, variant, exploitability, falsos positivos
  aliases: falsos positivos, validar vulnerabilidad, buscar variantes
  requires: Assigned target scope, permitted operations and source or test artifacts
  required_tools: file_read,files_search,command_run
  effects: mixed
---
# Security Finding Validation

## Scope

Use within the task's assigned objective, target and permitted operations. Load linked details only when relevant. Inherit runtime isolation and approvals; this procedure grants no additional authority.

## Inputs

Assigned objective, target revision/environment, relevant artifacts and permitted operations. Record missing inputs as blockers or coverage gaps.

## Workflow

Start with a precise claim: actor, controllable input, preconditions, sensitive operation and violated policy. Follow the real path through callers and defenses. Ask what evidence would disprove the claim, then inspect it. Treat scanner results and model suggestions as candidates.

Use a harmless reproduction inside the assigned test scope when available. Otherwise provide an explicit source trace and explain what remains unverified. A tool failure or unreachable probe is inconclusive. Severity follows demonstrated impact and prerequisites, not scary API names.

Search neighboring implementations for the underlying invariant failure, not only matching text. Validate each variant independently. Deduplicate findings sharing the same cause and repair while preserving all affected locations. Load [evidence criteria](references/evidence.md) for difficult or disputed claims.

## Verification

Keep evidence tied to the tested revision and distinguish confirmed, suspected, dismissed and unassessed outcomes. Report limitations explicitly; do not equate an empty result with complete security.

## Limits

No nested delegation, automatic scope expansion or self-approval. Reuse configured tool permissions. This skill does not establish a new sandbox or certify the absence of vulnerabilities.
