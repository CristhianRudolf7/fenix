---
name: cloud-file-sync
description: Transfer or reconcile local and cloud files with stable identity, conflict detection and resumable
  verification.
metadata:
  version: 1.0.0
  category: office
  tags: cloud, file, sync
  aliases: sincronizar archivos, Dropbox, OneDrive, S3, descargar, subir
  required_tools: tool_search
  requires: Authorized storage connector/CLI and local workspace access
  effects: external-write
---
# Cloud File Sync

## Scope

Transfer or reconcile local and cloud files with stable identity, conflict detection and resumable verification.

## Inputs

Source/destination, one-way or two-way intent, conflict policy and file scope. Prerequisite: Authorized storage connector/CLI and local workspace access.

## Workflow

1. Inventory paths, stable IDs, sizes and version/hash metadata. Distinguish a copy from synchronization and deletion propagation.
2. Compare content/version evidence; equal names or modification times do not prove identical files.
3. Transfer through staging or provider-supported resumable operations and retain source-to-destination mappings.
4. On conflict, preserve both versions unless the user’s policy selects one. Verify completion before advancing a sync cursor or deleting a source.

## Verification

Check file count, bytes and hashes where meaningful; report provider-specific checksum differences and unresolved conflicts.

## Limits

Do not infer permission to make files public or propagate deletions from a request to upload a folder.
