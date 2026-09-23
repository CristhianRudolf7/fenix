---
name: backup-and-restore
description: Create and verify backups or restore data with consistency, collision and recovery-time checks.
metadata:
  version: 1.0.0
  category: operations
  tags: backup, and, restore
  aliases: backup, respaldo, restaurar, recuperación, snapshot
  required_tools: file_read,file_write,command_run
  requires: Supported backup/restore tools and authorized storage
  effects: mixed
---
# Backup And Restore

## Scope

Create and verify backups or restore data with consistency, collision and recovery-time checks.

## Inputs

Data scope, recovery point, destination and overwrite constraints. Prerequisite: Supported backup/restore tools and authorized storage.

## Workflow

1. Identify mutable data and the consistency mechanism needed. Copying a live database file may omit journal/WAL state; use its supported backup method.
2. Record source/version, snapshot time, manifest and encryption-key requirements. Keep backup access separate from ordinary destructive permissions where feasible.
3. Restore into a separate destination first and compare records/files plus application readability. Verify that the required keys and dependencies are actually available.
4. For a requested live restore, inspect conflicts and pause writers as needed. Preserve recoverable current data and record any irreversible effects.

## Verification

Measure actual restored data, checksums where applicable and elapsed recovery time; test the consumer.

## Limits

A backup job success is not proof of restorability. Do not delete originals or overwrite live state unless that action is authorized.

## Español

Comprueba consistencia y restaura primero aparte; un backup exitoso no garantiza recuperación.
