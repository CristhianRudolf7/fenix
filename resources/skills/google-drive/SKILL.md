---
name: google-drive
description: Find, organize, export and share files in Google Drive through an available authorized connection.
metadata:
  version: 1.1.0
  category: office
  tags: google, drive
  aliases: drive, carpeta compartida, buscar documento, archivos en la nube
  required_tools: tool_search
  requires: Authorized Google Drive connector or owner-enabled browser session
  effects: external-write
---
# Google Drive

## Scope

Use for cloud file discovery and organization. Use local-file-work for files already on disk; use the format-specific Google skill to edit native content.

## Inputs

Account, folder or shared drive, requested operation, destination, and recipients only when sharing is requested.

## Workflow

1. Discover available Drive operations and inspect their schemas. Verify the selected account and accessible drive; if no connection exists, identify the missing capability and offer to work from an export.
2. Search within the requested folder or drive. Follow pagination and distinguish same-named files using stable IDs, parents, MIME type and modification time. Read candidate metadata before choosing.
3. For downloads, distinguish ordinary binary downloads from exports of native Google files. Select the requested export format and verify the local result; never change sharing to bypass a failed download.
4. For moves or renames, retain source and destination IDs and inspect collisions. For sharing, use the exact recipient and requested role; existing access is not permission to make a file public.
5. After a timeout on a write, inspect the target before retrying. Return stable links and the actions actually completed.

## References

Load [the connection checklist](references/connection-checklist.md) only when access or OAuth setup is the blocker.
