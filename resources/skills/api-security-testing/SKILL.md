---
name: api-security-testing
description: Test API authorization, protected properties and session boundaries with controlled requests and verified effects. Pruebas BOLA, IDOR, asignación masiva y WebSocket entre usuarios.
metadata:
  version: 1.0.0
  category: security
  tags: api, security, authorization, testing
  aliases: BOLA, IDOR, mass assignment, broken access control, WebSocket authorization, autorización API, campos protegidos, pruebas entre usuarios
  required_tools: file_read,file_write,command_run
  requires: Authorized API target, actual endpoint contracts and isolated test accounts/resources
  effects: mixed
---
# API Security Testing

## Scope

Use for security regression tests against an application API. Use api-integration to build a connector and security-code-review for source-only analysis. Ordinary integration work does not imply a penetration test.

## Inputs

Allowed endpoints/methods, accounts and role/tenant relationships, expected sharing policy, disposable resources, side-effect budget and observation/cleanup access. Reuse established authorization and clarify only missing intrusive scope.

## Workflow

1. Derive routes and schemas from the actual implementation/specification and capture a successful intended operation. Establish two distinct ordinary principals plus the roles needed for the requested test. Shared/public resources need different expectations from private resources.
2. Build a principal × action × resource/property matrix. Test own objects, another user's objects in the same tenant, another tenant, anonymous access, expired/revoked sessions and intended sharing where applicable. Include parent/child ownership, exports, downloads, async job status/results and list/search paths.
3. Use identifiers from the synthetic accounts instead of enumerating strangers' data. Change one ownership or role input at a time across path, query, body and headers. UUID unpredictability is not authorization. Test both access and denial with valid requests so validation errors do not hide a missing permission check.
4. For mass assignment, compare allowed editable fields against server-owned owner/tenant/role/billing fields. Check nested inputs and independently reread state with the resource owner; an echoed field does not prove a saved mutation. For batch or asynchronous operations, inspect each effect even if the final response is an error.
5. For WebSocket or streaming APIs, test subscription/message authorization after the handshake, session revocation, reconnect and replay cursors. Evaluate cross-origin behavior with the actual browser credential transport; a CLI bearer token and forged Origin do not reproduce ambient-cookie browser access.
6. Bound request rate, duration and data size. Use inert local sinks for injection/SSRF checks; redirects and out-of-band destinations must remain inside scope. Resource exhaustion and destructive payloads require their own authorized test conditions, not an automatic scanner preset.
7. Record redacted request, caller identity, expected policy, response body, observed state/effect, timing and cleanup. Classify confirmed violation, verified denial, inconclusive or not tested. Retest fixes and legitimate access.

## Verification

A 200 status may contain a denial; a 403 can follow a partial effect. Assert protected data is absent, state remains unchanged and asynchronous deliveries do not leak. Connection errors, invalid fixtures and missing telemetry cannot pass a security test.

## Limits

No scanner subscription or live exploit is required. Do not send credentials to substituted origins, disable TLS verification to make tests pass, or declare the whole API secure from a sampled matrix.

## Español

Prueba una matriz de identidad, acción, recurso y propiedad con cuentas controladas. Verifica datos y efectos reales, incluidos trabajos diferidos y WebSocket tras revocar sesiones. Los códigos HTTP, campos reflejados o errores de conexión no bastan para concluir. Mantén límites de ejecución y comprueba que el acceso legítimo sigue funcionando.
