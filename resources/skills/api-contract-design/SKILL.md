---
name: api-contract-design
description: Design or change API contracts with explicit inputs, errors, authorization, pagination and compatibility.
metadata:
  version: 1.1.0
  category: engineering
  tags: api, contract, design
  aliases: contrato API, endpoints, openapi, interfaz, idempotencia, consumer contract, event schema, contract drift, contrato compartido, deriva de esquema
  required_tools: file_read,files_search,file_write
  requires: Requirements, consumers and existing interface conventions
  effects: local-write
---
# Api Contract Design

## Scope

Use for public HTTP APIs or internal module boundaries. Implement through software-implementation once the contract is clear.

## Inputs

Actors, use cases, current consumers, data ownership and compatibility constraints.

## Workflow

1. Read existing callers and schemas. Define resource identities and which actor can read or change each resource; never rely only on client-side filtering.
2. Specify input types, required/optional/null semantics, defaults and validation errors. Describe state transitions and side effects explicitly.
3. For collections, choose stable ordering and pagination. For mutations, define retry behavior, idempotency keys where needed and concurrency checks.
4. Give concrete request/response examples including authorization failure, conflicts and missing objects. Keep HTTP and domain error semantics consistent.
5. Compare the change with existing clients. Add a migration/deprecation path only when the interface actually breaks; avoid inventing a new version for an internal refactor.

For independently maintained consumers, services or event payloads, load [shared contracts](references/shared-contracts.md). Use one authoritative artifact and compare both consumer expectations and real serialized provider outputs against it, including alternate/error paths and old-client compatibility.

## Verification

Check examples against the schema and exercise a representative consumer or contract test. Verify duplicate/retried requests do not create unintended effects.

## Limits

Do not force REST, GraphQL or a new dependency when existing requirements do not need it. Examples must not contain real credentials.

## Español

Define entradas, errores, permisos y reintentos con ejemplos verificables. Conserva compatibilidad y comprueba el contrato desde un consumidor real.

Cuando haya consumidores independientes, usa un contrato canónico y comprueba respuestas serializadas y expectativas del cliente contra el mismo esquema, incluidos errores y compatibilidad.
