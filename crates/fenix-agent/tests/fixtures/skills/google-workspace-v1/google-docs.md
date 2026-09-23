---
name: google-docs
description: Read and edit Google Docs while preserving tabs, structure, formatting and existing content.
metadata:
  version: 1.0.0
  category: office
  tags: google, docs
  aliases: documentos google, editar documento compartido, google docs
  required_tools: tool_search
  requires: Authorized Google Docs connection
  effects: external-write
---
# Google Docs

## Scope

Use for native Google documents. Use documents-word for local DOCX creation or edits; converting formats can lose native features.

## Inputs

Document ID or link, requested section or change, source material and any template to preserve.

## Workflow

1. Discover read and edit capabilities and select the correct account. Read the current document including its tabs and relevant structural elements before preparing mutations.
2. Locate the requested heading, paragraph or table with an unambiguous anchor. If a phrase occurs twice, use surrounding text or ask which occurrence is intended.
3. Prefer narrow replacements or inserts. Use the provider revision guard when offered; refresh text indexes after earlier edits. Preserve tables, links, comments and unmodified tabs.
4. When filling a template, copy it using a native operation if available and alter only intended fields. Do not replace a rich document with a Markdown reconstruction unless explicitly requested.
5. On ambiguous write failure, reread the affected section before repeating; report unsupported elements rather than claiming preservation you cannot inspect.

## Verification

Check requested text, nearby structure, links and untouched sections. Return the actual document URL and any visual review still needed.

## Limits

Missing connection means remote edits cannot run. Rendering, comments and revision operations depend on discovered tools, not on the existence of this skill.

## Español

Lee pestañas y estructura antes de editar. Usa cambios localizados, controla revisiones y verifica el resultado sin reconstruir todo el documento.
