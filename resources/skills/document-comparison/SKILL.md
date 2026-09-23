---
name: document-comparison
description: Compare document versions for substantive changes while preserving structure and evidence references.
metadata:
  version: 1.0.0
  category: office
  tags: document, comparison
  aliases: comparar documentos, versiones, diferencias, cambios contrato
  required_tools: file_read,file_write,command_run
  requires: Readable versions and format-appropriate extraction tools
  effects: mixed
---
# Document Comparison

## Scope

Compare document versions for substantive changes while preserving structure and evidence references.

## Inputs

Baseline/candidate, comparison scope and treatment of formatting changes. Prerequisite: Readable versions and format-appropriate extraction tools.

## Workflow

1. Identify the actual versions and retain their hashes/IDs. Extract with headings, pages, tables or cells preserved where possible.
2. Align corresponding sections before comparing text so moved content is not misreported as deletion plus invention.
3. Distinguish wording, numbers, obligations, definitions and formatting changes; trace cross-references affected by edits.
4. Produce a change summary with locations and unresolved extraction/layout limits; preserve original documents.

## Verification

Spot-check material differences against both source versions and account for missing/extra sections.

## Limits

Text extraction alone may miss tracked changes, comments or layout. Do not silently accept one version as authoritative.

## Español

Alinea secciones y distingue cambios sustantivos de movimientos o formato, verificando ambas fuentes.
