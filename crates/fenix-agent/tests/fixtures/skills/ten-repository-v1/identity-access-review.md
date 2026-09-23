---
name: identity-access-review
description: Review account, role and service-identity permissions against actual resource needs.
metadata:
  version: 1.0.0
  category: security
  tags: identity, access, review
  aliases: IAM, RBAC, permisos, acceso, privilegios, cuentas
  required_tools: file_read,file_write,command_run
  requires: Authorized identity/policy inventory and target-system semantics
  effects: local-write
---
# Identity Access Review

## Scope

Review account, role and service-identity permissions against actual resource needs.

## Inputs

Accounts, resources, ownership, intended actions and review scope. Prerequisite: Authorized identity/policy inventory and target-system semantics.

## Workflow

1. Map effective access including group inheritance, role chaining and resource policies, not just direct grants.
2. Compare permissions to observed responsibilities and separate human from service identities; identify stale or unowned access.
3. Check cross-tenant/resource boundaries and credential lifecycle. Treat missing logs as unknown rather than proof of nonuse.
4. Propose minimal changes with impact and recovery paths; execute revocation/grants only within authorization.

## Verification

Test intended access and denial with the correct identity and inspect residual inherited permissions.

## Limits

Do not lock out the owner or grant administrator access as a troubleshooting shortcut.

## Español

Revisa permisos efectivos e indirectos y comprueba tanto acceso permitido como denegado antes de cambiar privilegios.
