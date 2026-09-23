---
name: api-integration
description: Integrate external APIs with schema validation, pagination, retry discipline and duplicate-effect prevention.
metadata:
  version: 1.1.0
  category: engineering
  tags: api, integration
  aliases: integración API, webhook, paginación, rate limit, tool router, partial writes, Composio API, escrituras parciales
  required_tools: file_read,file_write,command_run
  requires: Documented API and authorized credentials provided through normal configuration
  effects: mixed
---
# Api Integration

## Scope

Integrate external APIs with schema validation, pagination, retry discipline and duplicate-effect prevention.

## Inputs

Operation, endpoint/version, auth mechanism and data/side-effect contract. Prerequisite: Documented API and authorized credentials provided through normal configuration.

## Workflow

1. Read primary API documentation and actual schemas. Keep provider IDs, request IDs and pagination cursors rather than inferring identity from labels.
2. Validate responses and classify authentication, validation, throttling and transient errors. Honor Retry-After where supported.
3. Retry only safe/idempotent operations or use the provider’s idempotency mechanism. After an ambiguous mutation, query state before resubmitting.
4. For webhooks, verify signature over the raw payload, deduplicate event IDs and handle out-of-order deliveries without assuming arrival order is business order.

For multi-service workflows, preserve a receipt for each step: target account/resource, request or idempotency ID, observed status, returned object ID and verification. Parallelize only independent operations. A batch is not a cross-provider transaction: after a partial failure, reconcile completed effects before resuming and perform compensating changes only within the task's scope. Persist cursors with their query/account context and distinguish a bounded sample from a complete export. Discover current router schemas; do not substitute a connector mentioned in an example for the requested provider.

## Verification

Exercise pagination, expired credentials, partial failures and repeated webhooks with fixtures or an authorized test environment.

## Limits

Do not hardcode secrets or invent provider fields. Adding this skill does not install an API connector.

## Español

Integra contratos reales, páginas y reintentos; verifica webhooks y evita duplicar efectos tras respuestas inciertas.
