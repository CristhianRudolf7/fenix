---
name: legacy-migration
description: Migrate legacy code or frameworks incrementally while preserving observable behavior and rollback options.
metadata:
  version: 1.0.0
  category: engineering
  tags: legacy, migration
  aliases: migración código, modernizar, legacy, reemplazar framework
  required_tools: file_read,file_write,command_run
  requires: Readable old/new implementation and a reproducible consumer
  effects: mixed
---
# Legacy Migration

## Scope

Migrate legacy code or frameworks incrementally while preserving observable behavior and rollback options.

## Inputs

Migration boundary, compatibility promises, consumers and cutover scope. Prerequisite: Readable old/new implementation and a reproducible consumer.

## Workflow

1. Inventory public behavior and callers before selecting a replacement. Capture representative examples including errors and edge behavior, not only happy paths.
2. Introduce an adapter or bounded slice when it allows old and new paths to coexist. Avoid mixing a migration with unrelated redesign.
3. Compare outputs, side effects and performance on equivalent inputs. If dual-running, keep secondary effects disabled so validation does not duplicate writes.
4. Plan cutover, rollback and retirement of old state explicitly; a schema change may make binary rollback insufficient.

## Verification

Run consumer compatibility checks and a migration rehearsal; document intentional behavior differences.

## Limits

Do not preserve an observed bug as a requirement without checking intent, or promise zero downtime without evidence.

## Español

Migra por límites comprobables, compara efectos y define un retorno compatible con los datos.
