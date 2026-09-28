---
name: storage-and-archive-work
description: Inspect storage use and safely extract, package or reorganize archives with content verification.
metadata:
  version: 1.0.0
  category: operations
  tags: storage, and, archive, work
  aliases: zip, tar, archivos comprimidos, espacio disco, extraer
  required_tools: file_read,file_write,command_run
  requires: Installed archive tools and authorized filesystem scope
  effects: mixed
---
# Storage And Archive Work

## Scope

Inspect storage use and safely extract, package or reorganize archives with content verification.

## Inputs

Source/destination, formats, required preservation and cleanup request. Prerequisite: Installed archive tools and authorized filesystem scope.

## Workflow

1. Inventory actual sizes and archive members before extraction. Distinguish apparent size, disk allocation, hardlinks and filesystem capacity.
2. Reject paths escaping the destination, dangerous links and unexpected special files. Check available space against expanded size, not only compressed bytes.
3. Extract or package into a separate staging location; preserve required metadata without overwriting existing files blindly.
4. Compare manifest/counts and content hashes where appropriate. For cleanup, identify exact targets and dependencies before authorized removal.

## Verification

Open representative extracted files and verify archive integrity; retain the original until the requested result is confirmed.

## Limits

Do not delete caches, containers or unrelated user files merely because disk space is low.
