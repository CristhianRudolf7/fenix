---
name: bibliography-management
description: Verify and maintain BibTeX or reference lists with stable identifiers and correct document linkage.
metadata:
  version: 1.1.0
  category: research
  tags: bibliography, management
  aliases: bibliografía, bibtex, DOI, referencias, citas
  required_tools: file_read,file_write,tool_search
  requires: Existing bibliography and primary metadata sources
  effects: local-write
---
# Bibliography Management

## Scope

Verify and maintain BibTeX or reference lists with stable identifiers and correct document linkage.

## Inputs

Citation files, desired style and documents that use the keys. Prerequisite: Existing bibliography and primary metadata sources.

## Workflow

1. Resolve DOI/arXiv/ISBN or other identifiers against primary records and distinguish editions or preprint revisions.
2. Deduplicate by identity, preserving distinct works with similar titles. Normalize fields without changing author order or losing accents.
3. Keep stable citation keys; when renaming, update actual callers and check unresolved references.
4. Export the requested format and retain uncertain fields as unresolved rather than inventing metadata.

Retain the checked source URL, access date and exact cited version alongside resolved metadata. Check reported corrections or retractions against primary records when relevant; an unavailable resolver leaves status unknown. For claim-to-passage verification, use research-evidence-audit instead of treating identifier resolution as evidence of support.

## Verification

Check identifier resolution, cited keys, duplicate records and rendered bibliography if compilation is available.

## Limits

A valid DOI is not evidence that a paper supports the attached claim; use literature-review for synthesis.

## Español

Verifica identidad y metadatos, conserva claves estables y no confundas una cita válida con evidencia de una afirmación.
