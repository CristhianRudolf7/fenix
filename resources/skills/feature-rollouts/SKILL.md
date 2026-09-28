---
name: feature-rollouts
description: Introduce and retire feature flags with stable targeting, measured rollout and tested fallback.
metadata:
  version: 1.0.0
  category: operations
  tags: feature, rollouts
  aliases: feature flag, despliegue gradual, canary, rollback
  required_tools: file_read,file_write,command_run
  requires: Application flag/configuration mechanism and telemetry
  effects: mixed
---
# Feature Rollouts

## Scope

Introduce and retire feature flags with stable targeting, measured rollout and tested fallback.

## Inputs

Feature, audience, compatibility needs, owner and rollback trigger. Prerequisite: Application flag/configuration mechanism and telemetry.

## Workflow

1. Define the flag’s purpose, default and lifetime. Keep assignment stable for users where experiments or consistent behavior require it.
2. Test enabled and disabled paths, including mixed client/server versions and migrations.
3. Roll out through the authorized cohorts and observe meaningful error/latency/business signals; compare equivalent populations.
4. On rollback, inspect state already written by the new path. After stabilization, remove obsolete branches and flag configuration together.

## Verification

Exercise fallback and stale configuration behavior; verify the flag actually gates the intended effect.

## Limits

A flag cannot undo external effects or incompatible data writes. Do not invent rollout percentages as universal policy.
