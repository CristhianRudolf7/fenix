---
name: security-rust-boundaries
description: Audit Rust unsafe and FFI boundaries, logical races, cancellation and externally reachable failure paths.
metadata:
  version: 1.0.0
  category: security
  tags: security, Rust, unsafe, FFI, Tokio, cancellation, panic, concurrencia
  aliases: auditoría Rust, unsafe FFI, cancelación asíncrona
  requires: Assigned target scope, permitted operations and source or test artifacts
  required_tools: file_read,files_search,command_run
  effects: mixed
---
# Rust Security Boundaries

## Scope

Use within the security worker's assigned objective, target and permitted operations. Load linked details only when relevant. Inherit runtime isolation and approvals; this procedure grants no additional authority.

## Inputs

Assigned objective, target revision/environment, relevant artifacts and permitted operations. Record missing inputs as blockers or coverage gaps.

## Workflow

Map public inputs and trust boundaries before inspecting implementation. For unsafe/FFI, identify ownership, initialization, alignment, aliasing, lifetimes and ABI assumptions, then find the safe caller capable of violating them. A SAFETY comment is a claim to check, not evidence by itself.

For safe Rust, review authorization, path handling, unbounded allocation, integer conversion, panics and async state transitions. Distinguish memory data races from logical races; compilation does not prove workflow correctness. Examine cancellation after partial effects, locks held across await, blocking work on executors and detached tasks.

Use targeted tests and available local diagnostics where useful, recording unsupported configurations. Do not declare every unwrap a vulnerability or invoke a nested reviewer tree. Load [Rust investigation cases](references/rust.md) for a focused checklist.

## Verification

Keep evidence tied to the tested revision and distinguish confirmed, suspected, dismissed and unassessed outcomes. Report limitations explicitly; do not equate an empty result with complete security.

## Limits

No nested delegation, automatic scope expansion or self-approval. Reuse configured tool permissions. This skill does not establish a new sandbox or certify the absence of vulnerabilities.

## Español

Procedimiento especializado de seguridad para el alcance asignado. Carga las referencias según la tarea, conserva evidencias verificables y separa fallos confirmados, sospechas y comprobaciones pendientes. Las correcciones requieren verificar también el funcionamiento legítimo.
