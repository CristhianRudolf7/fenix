---
name: security-code-review
description: Review application code for concrete authorization, injection, data exposure and isolation defects.
metadata:
  version: 1.3.0
  category: security
  tags: security, code, review
  aliases: seguridad código, vulnerabilidad, autorización, inyección, aislamiento, agent trust boundaries, prompt injection, tool poisoning, seguridad de agentes, cryptographic key management, nonce reuse, revisión criptográfica, auditoría de límites de confianza, llamadas a herramientas
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

For agent, RAG or MCP applications, load [agent trust-boundary checks](references/agent-boundaries.md). Follow untrusted content into retrieval, memory, tool arguments and rendered output; verify identity and authorization at the operation that produces the effect. A blocked wording example alone does not establish a secure execution boundary.

For cryptographic code or key-management review, load [cryptographic boundaries](references/cryptographic-boundaries.md). Review purpose, nonce lifecycle and failure handling rather than grading algorithm names alone. Use agent-security-evaluation for adversarial agent tests and api-security-testing for live API authorization checks.

## Focused follow-up

For an explicitly scoped patch audit, use security-change-review. Validate uncertain findings with security-finding-validation; load security-rust-boundaries for Rust-specific invariants. Those procedures are available in the shared catalog to any task that needs them. Use security-fix-validation when correction is part of the assignment.

## Verification

Retest the failure path after an authorized fix and check normal access still works. Redact secrets from evidence.

## Limits

This is not authorization for scanning unrelated hosts or exploiting live services. A clean review is not a certification.
