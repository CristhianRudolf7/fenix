---
name: review-feedback
description: Assess received code-review or PR comments and apply valid corrections when authorized, checking questionable suggestions against current behavior. Resolver comentarios del revisor.
metadata:
  version: 1.0.0
  category: engineering
  tags: review, feedback, comments, corrections, compatibility
  aliases: reviewer suggestions, PR comments, address feedback, comentarios de revisión, sugerencias del revisor, correcciones válidas, resolver PR
  required_tools: file_read,files_search,file_edit,file_patch,command_run
  requires: Received findings, current repository and the user's requested review or repair scope
  effects: mixed
---
# Review Feedback

## Scope

Handle findings already supplied by a person or agent. Assess them, or implement valid fixes when requested. Use code-review to generate an independent review of a diff; a request to evaluate feedback alone remains read-only.

## Inputs

Review comments with locations and claimed consequences, current revision, supported behavior and the original user request. Preserve the distinction between the reviewer's proposal and an owner-approved requirement.

## Workflow

1. Check each material comment against the current code, callers and compatibility obligations. A suggestion may be stale, incomplete or wrong. Identify a reachable triggering condition before treating it as a defect.
2. Give each finding a disposition: confirmed with evidence; unsupported with a counterexample or contract; needs information with the missing decision; or deferred with its real impact and reason. Classify by effect rather than the reviewer's confidence.
3. For authorized corrections, reproduce the defect, implement a focused fix and run affected checks. Keep independent confirmed items moving while a separate decision is pending. Do not remove a compatibility path merely because no caller was found in a limited local search.
4. Recheck the fix against the reviewed revision and plausible regressions. Preserve unresolved items in the existing task record. Additional review should answer a specific remaining question rather than start an indefinite reviewer/fixer loop.
5. Summarize applied corrections with their evidence and explain rejected, deferred or unresolved suggestions. Correct an earlier disagreement when new evidence shows it was wrong.

## Verification

Each claimed correction has a changed artifact and attributable check. Each rejected finding has a technical reason. A passing suite does not resolve an unrelated requirement dispute, and an accepted comment is not a completed fix.

## Limits

Feedback grants no permission by itself. Preserve user changes and the requested scope; avoid unrelated refactoring or invented features. A review-only request does not permit edits. Posting replies or resolving external PR threads requires authorization to communicate, even when local fixes are authorized.

## Español

Resuelve comentarios de revisión ya recibidos. Comprueba cada sugerencia con el código actual, sus consumidores y compatibilidad. Clasifica hallazgos como confirmados, no sustentados, pendientes de información o aplazados. Aplica las correcciones válidas cuando estén autorizadas, reproduce el defecto y verifica el cambio. Explica las sugerencias rechazadas y lo pendiente. Revisar comentarios no autoriza editar ni responder en plataformas externas.
