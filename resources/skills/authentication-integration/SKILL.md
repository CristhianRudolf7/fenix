---
name: authentication-integration
description: Implement login, sessions or OAuth with correct identity binding, token handling and server-side authorization.
metadata:
  version: 1.1.0
  category: engineering
  tags: authentication, integration
  aliases: autenticación, login, oauth, sesión, permisos, JWT, issuer audience, token verification, validación de tokens
  required_tools: file_read,file_write,command_run
  requires: Application stack and identity-provider documentation
  effects: mixed
---
# Authentication Integration

## Scope

Implement login, sessions or OAuth with correct identity binding, token handling and server-side authorization.

## Inputs

Actors, login flow, trust boundaries, session policy and existing users. Prerequisite: Application stack and identity-provider documentation.

## Workflow

1. Distinguish authentication from resource authorization. Resolve the account/tenant on the server and prevent cross-account linking by email alone.
2. For OAuth, follow the provider’s supported flow with appropriate state/PKCE and redirect validation. Keep client secrets and refresh tokens out of browser bundles.
3. Design session expiry, logout, revocation and recovery together. Preserve cancellation and error paths without exposing tokens in URLs or logs.
4. Test invalid callbacks, reused codes, expired sessions and unauthorized resource access; preserve the existing user provisioning policy.

For JWT, OAuth or session hardening, load [token boundaries](references/token-boundaries.md). Pin issuer, audience and key trust; test refresh/revocation and account linking through the real boundary.

## Verification

Exercise the complete flow through the real callback/session boundary and verify isolation between two accounts.

## Limits

Do not add public account registration when only login was requested or treat an unsigned token payload as verified identity.

## Español

Separa identidad de autorización y prueba sesiones y aislamiento; conserva la política existente de creación de usuarios.
