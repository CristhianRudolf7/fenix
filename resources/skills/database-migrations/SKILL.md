---
name: database-migrations
description: Apply schema and data migrations with compatibility, restartability and verified recovery.
metadata:
  version: 1.0.0
  category: engineering
  tags: database, migrations
  aliases: migración base datos, backfill, cambiar esquema, rollback, schema migration, migrate database, columns, tables, data retention, migrar, columnas, tablas
  required_tools: file_read,file_write,command_run
  requires: Database tools and authorized target connection
  effects: mixed
---
# Database Migrations

## Scope

Apply schema and data migrations with compatibility, restartability and verified recovery.

## Inputs

Current/target schema, data volume, concurrent app versions and maintenance constraints. Prerequisite: Database tools and authorized target connection.

## Workflow

1. Inspect schema and data conditions before executing a migration. Record a recoverable snapshot or backup appropriate to the target and requested operation.
2. Prefer expand, backfill, validate, then contract when old/new application versions coexist. Separate irreversible data transformations from reversible DDL.
3. Make backfills restartable with stable keys and progress checkpoints; measure locks and batch cost on representative data.
4. Rehearse failure and resume on a disposable copy. Execute within authorization and inspect actual migration state after interruption before retrying.

## Verification

Reconcile row counts, constraints and sample values; test old/new consumers as required and verify the restore procedure separately.

## Limits

Transactions do not make every engine’s DDL reversible. Never mark a migration applied merely because the process exited.

## Español

Ensaya cambios y recuperación, conserva compatibilidad y verifica datos y estado tras interrupciones.
