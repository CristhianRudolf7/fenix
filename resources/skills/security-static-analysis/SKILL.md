---
name: security-static-analysis
description: Run existing local security analyzers and interpret SARIF or Semgrep output with explicit coverage gaps.
metadata:
  version: 1.0.0
  category: security
  tags: security, SARIF, Semgrep, CodeQL, static analysis, scanner, análisis estático
  aliases: análisis estático, interpretar SARIF, resultados de Semgrep
  requires: Assigned target scope, permitted operations and source or test artifacts
  required_tools: file_read,files_search,command_run,security_scan_read
  effects: mixed
---
# Security Static Analysis

## Scope

Use within the security worker's assigned objective, target and permitted operations. Load linked details only when relevant. Inherit runtime isolation and approvals; this procedure grants no additional authority.

## Inputs

Assigned objective, target revision/environment, relevant artifacts and permitted operations. Record missing inputs as blockers or coverage gaps.

## Workflow

Inventory the repository languages, build configuration and existing scanner commands before selecting checks. Reuse installed tools and pinned local rule configurations. Do not require a commercial engine, download remote rules implicitly or upload source. Record engine version, rule/configuration identity, target revision, exclusions and invocation outcome.

Execute through command_run under the assignment's existing permissions. Preserve exit status, stderr and raw output. Tools differ on findings-related exit codes; inspect the documented tool behavior. A malformed report, missing dependency, unsupported language or failed analysis is not a clean scan.

Use security_scan_read on a local SARIF 2.1.0 or Semgrep JSON artifact to obtain bounded pages of candidates. Its coverage field reports output-level errors; retain process evidence separately. Validate important candidates against source before reporting them as confirmed. Load [scanner handling](references/scanners.md) when configuring or interpreting an analyzer.

## Verification

Keep evidence tied to the tested revision and distinguish confirmed, suspected, dismissed and unassessed outcomes. Report limitations explicitly; do not equate an empty result with complete security.

## Limits

No nested delegation, automatic scope expansion or self-approval. Reuse configured tool permissions. This skill does not establish a new sandbox or certify the absence of vulnerabilities.

## Español

Procedimiento especializado de seguridad para el alcance asignado. Carga las referencias según la tarea, conserva evidencias verificables y separa fallos confirmados, sospechas y comprobaciones pendientes. Las correcciones requieren verificar también el funcionamiento legítimo.
