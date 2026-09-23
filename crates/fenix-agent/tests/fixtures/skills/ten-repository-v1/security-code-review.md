---
name: security-code-review
description: Review application code for concrete authorization, injection, data exposure and isolation defects.
metadata:
  version: 1.0.0
  category: security
  tags: security, code, review
  aliases: seguridad código, vulnerabilidad, autorización, inyección, aislamiento
  required_tools: file_read,files_search,command_run
  requires: Authorized repository or supplied code
  effects: read-only
---
# Security Code Review

## Scope

Use for defensive source review. Use threat-modeling for design risks and dependency-and-secret-audit for dependency or credential inventory.

## Inputs

Code scope, trust boundaries, deployment assumptions and assets at risk.

## Workflow

1. Map entrypoints, identities and data ownership. Trace authorization checks through service and storage layers, including background jobs and tenant boundaries.
2. Follow untrusted input into queries, shell commands, paths, templates, redirects and deserialization. Check validation at the actual sink, not only the UI.
3. Inspect session lifecycle, error output, logs and file permissions for exposure. Review concurrency and stale-state checks around sensitive mutations.
4. For each finding, provide a minimal reachable scenario, location, preconditions, impact and a focused fix. Use local harmless reproductions when appropriate.
5. Prioritize by demonstrated impact and likelihood. Distinguish verified defects, plausible concerns and controls not assessed.

## Verification

Retest the failure path after an authorized fix and check normal access still works. Redact secrets from evidence.

## Limits

This is not authorization for scanning unrelated hosts or exploiting live services. A clean review is not a certification.

## Español

Revisa entradas y límites de confianza, siguiendo datos hasta operaciones sensibles. Informa fallos alcanzables con evidencia y corrige dentro del alcance autorizado.
