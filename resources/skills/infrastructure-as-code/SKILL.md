---
name: infrastructure-as-code
description: Review and apply infrastructure-as-code changes with state ownership and replacement awareness.
metadata:
  version: 1.0.0
  category: operations
  tags: infrastructure, as, code
  aliases: terraform, pulumi, infraestructura como código, plan
  required_tools: file_read,file_write,command_run
  requires: Existing IaC toolchain and authorized backend/provider access
  effects: mixed
---
# Infrastructure As Code

## Scope

Review and apply infrastructure-as-code changes with state ownership and replacement awareness.

## Inputs

Workspace, state/backend, target resources and requested change. Prerequisite: Existing IaC toolchain and authorized backend/provider access.

## Workflow

1. Inspect provider versions, modules, variables and state ownership. Never select a workspace from its name alone when accounts differ.
2. Produce a plan and inspect creates, replacements, deletions and sensitive outputs. Investigate unexpected drift before applying.
3. Use the backend’s locking/concurrency mechanism. Keep imported resources and moved addresses explicit rather than recreating them accidentally.
4. Apply only the reviewed authorized change; after interruption, refresh actual state and inspect partial effects before retrying.

## Verification

Compare resulting state to the plan and verify the service behavior the resources support.

## Limits

State may contain secrets. A successful plan is not a deployed change, and an old plan may be invalid after concurrent updates.
