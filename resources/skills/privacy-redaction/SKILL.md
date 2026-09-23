---
name: privacy-redaction
description: Prepare shareable copies by removing sensitive content while preserving usable evidence.
metadata:
  version: 1.0.0
  category: security
  tags: privacy, redaction
  aliases: redactar secretos, anonimizar, datos personales, censurar
  required_tools: file_read,file_write,command_run
  requires: Source artifacts and appropriate format inspection tools
  effects: local-write
---
# Privacy Redaction

## Scope

Prepare shareable copies by removing sensitive content while preserving usable evidence.

## Inputs

Audience, sensitive fields, retention needs and output format. Prerequisite: Source artifacts and appropriate format inspection tools.

## Workflow

1. Inventory sensitive data in text, metadata, comments, hidden sheets, attachments and embedded objects, not only visible prose.
2. Create a separate shareable copy and use consistent pseudonyms where relationships must remain understandable; keep any mapping separately protected.
3. Remove underlying content rather than merely covering it visually. Check logs and filenames for indirect disclosure.
4. Inspect the exported artifact using extraction and format-specific checks, and state remaining re-identification risks.

## Verification

Search for known sensitive values and inspect hidden/metadata layers relevant to the format.

## Limits

Pattern matching cannot guarantee anonymity; do not claim complete redaction when the tool cannot inspect all layers.

## Español

Redacta contenido y metadatos en una copia, no solo la vista, y verifica extracción y posibles identificadores indirectos.
