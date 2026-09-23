---
name: documents-word
description: Create Word DOCX reports with headings, paragraphs and tables; read and edit existing document text
  using native Fenix Documents.
metadata:
  version: 1.2.0
  category: office
  tags: documents, word
  aliases: word, docx, informe, carta, documento
  required_tools: document_read,document_create,document_edit
  requires: Native Fénix document tools
  effects: mixed
---
# Fénix Documents — Word

## Scope

Create Word DOCX reports with headings, paragraphs and tables; read and edit existing document text using native Fenix Documents.

## Inputs

Requested local DOCX artifact or edit, source content and target path.

## Workflow

Use native `document_*` tools; no Microsoft account, Office CLI, external office executable or provider is needed. Paths belong to the current task's file workspace. Runtime file approvals apply.

Create with `document_create`:
```json
{"path":"report.docx","document":{"format":"docx","paragraphs":[{"text":"Quarterly report","heading":1},{"text":"Revenue increased.","bold":false}],"tables":[[["Metric","Value"],["Revenue","120"]]]}}
```
`heading` is 0–6; paragraphs precede tables. Use explicit newline characters for line breaks. New files never overwrite existing work.

For an existing file, call `document_read` and follow `next_offset` only as needed. Keep its `sha256` and the relevant `part`. Edit using `document_edit` with `expected_sha256` and `operation`:
```json
{"action":"replace_text","part":"word/document.xml","expected":"Revenue increased.","replacement":"Revenue increased by 12%."}
```
The match must occur exactly once within one paragraph in that part. Formatting may split it into runs; Fénix handles that case. Unchanged parts, images and styles are preserved. Replacement text starts in the first matching run's style. A changed hash requires rereading; never bypass it by guessing a hash. Files are written atomically, but there is no revision-history restoration tool.

Read back the affected section and report the file path. Text extraction cannot certify page layout. Advanced layout, tracked changes, mail merge, PDF rendering and image insertion are not implemented by these tools; do not claim them or silently replace the file with a simplified document.

For visual delivery in Fénix, discover `design_templates`, `design_present`, `design_publish` and `design_import` with `tool_search`. Present a few template choices only when the choice matters; publish the finished structured result or import its workspace file so it appears in the chat panel and Design. Use the existing design ID and current revision for updates, preserve project scope, and label sample data. HTML previews disable scripts/network; Office imports extract content rather than reproduce original layout or recalculate formulas.

## Verification

Read back affected paragraphs/tables and preserve unrelated package parts.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Native document tools do not render pages or calculate formulas. Do not install OfficeCLI or a cloud office dependency as an implicit fallback.

## Español


Crea informes DOCX con títulos, párrafos y tablas sin proveedores externos. Lee antes de editar y usa el hash y la parte devueltos por `document_read`. Conserva el formato ajeno; una edición no equivale a revisar visualmente las páginas. No inventes datos ni prometas funciones de maquetación todavía no disponibles.
